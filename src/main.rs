extern crate args;
extern crate chrono;
extern crate failure;
extern crate getopts;
extern crate log;
extern crate reqwest;
extern crate rusoto_cloudwatch;
extern crate rusoto_core;
extern crate serde_json;
extern crate stderrlog;

use args::Args;
use chrono::{DateTime, FixedOffset};
use failure::{Error, format_err};
use getopts::Occur;
use log::{debug, info, warn};
use reqwest::Client as HttpClient;
use rusoto_cloudwatch::{CloudWatch, CloudWatchClient, Dimension, MetricDatum, PutMetricDataError, PutMetricDataInput};
use rusoto_core::Region;
use serde_json::Value;
use std::cmp::max;
use std::collections::HashMap;
use std::env::args;
use std::iter::FromIterator;
use std::process::exit;
use std::thread::sleep;
use std::time::Duration;

#[cfg(test)] pub mod tests;

#[derive(Debug, Default, PartialEq)]
pub struct Configuration {
  base_url: String,
  interval: Duration,
  log_level: usize,
  namespace: String,
  queue_size: usize,
}

#[derive(Debug, PartialEq)]
pub struct Metadata {
  #[allow(dead_code)]
  container_id: String,
  dimensions: Vec<Dimension>,
}

#[derive(Debug, PartialEq)]
pub struct Metric {
  name: String,
  unit: String,
  value: f64,
}

#[derive(Debug, PartialEq)]
pub struct Stats {
  container_id: String,
  metrics: Vec<Metric>,
  timestamp: DateTime<FixedOffset>,
}

const DIMENSIONS_TO_COLLECT: [(&str, &str); 1] = [
  ("/Name", "task"),
];

pub fn task_metadata(http: &HttpClient, base_url: &str) -> Result<HashMap<String, Metadata>, Error> {
  let body: Value = http.get(&format!("{}/v2/metadata", base_url)).send()?.json()?;
  debug!("Received metadata {}", body);
  let containers: Vec<Value> = match body.get("Containers") {
      Some(c) => c.as_array().unwrap().clone(),
      None => Vec::new(),
    };
  let metadata_pairs = containers.iter()
    .filter(|container| !container["Name"].as_str().unwrap().starts_with("~internal"))
    .map(|container| (
      container["DockerId"].as_str().unwrap().to_owned(),
      Metadata {
        container_id: container["DockerId"].as_str().unwrap().to_owned(),
        dimensions: DIMENSIONS_TO_COLLECT.iter().map(|(p, n)|
          Dimension {
            name: String::from(*n),
            value: String::from(container.pointer(p).unwrap().as_str().unwrap()),
          }
        ).collect(),
      })
    );
  Ok(HashMap::from_iter(metadata_pairs))
}

const METRICS_TO_COLLECT: [(&str, &str, &str); 2] = [
  ("/memory_stats/max_usage", "max_usage", "Bytes"),
  ("/memory_stats/usage", "usage", "Bytes"),
];

pub fn container_stats(http: &HttpClient, base_url: &str) -> Result<Vec<Stats>, Error> {
  let body: Value = http.get(&format!("{}/v2/stats", base_url)).send()?.json()?;
  debug!("Received stats {}", body);
  let stats = body.as_object().unwrap().iter()
    .filter(|(_, stats)| !stats.is_null())
    .map(|(id, stats)|
      Stats {
        container_id: id.clone(),
        metrics: METRICS_TO_COLLECT.iter().map(|(p, n, u)| Metric {
          name: String::from(*n),
          unit: String::from(*u),
          value: stats.pointer(p).unwrap().as_f64().unwrap(),
        }).collect(),
        timestamp: DateTime::parse_from_rfc3339(stats["read"].as_str().unwrap()).unwrap(),
      }
    )
    .collect();
  Ok(stats)
}

type Metrics = Vec<MetricDatum>;

pub fn metrics_from_stats(metrics: &mut Metrics, stats: Vec<Stats>, metadata: &HashMap<String, Metadata>) {
  stats.into_iter()
    .filter(|s| metadata.contains_key(&s.container_id))
    .flat_map(|s| {
      let dimensions = &metadata.get(&s.container_id).unwrap().dimensions;
      let timestamp = s.timestamp;
      s.metrics.into_iter().map(move |m|
        MetricDatum {
          dimensions: Some(dimensions.clone()),
          metric_name: m.name,
          timestamp: Some(format!("{}", timestamp.format("%FT%T%.3f%:z"))),
          unit: Some(m.unit),
          value: Some(m.value),
          ..Default::default()
        }
      )
    })
    .for_each(|m| metrics.push(m));
}

pub fn report_to_cloudwatch(client: &impl CloudWatch, namespace: &str, data: &Metrics) -> Result<usize, Error> {
  let mut start_index = 0;
  while data.len() > start_index {
    let chunk: Metrics = data.iter().skip(start_index).take(20).map(|l| l.clone()).collect();
    let chunk_size = chunk.len();
    match client.put_metric_data(PutMetricDataInput {
      namespace: String::from(namespace),
      metric_data: chunk,
    }).sync() {
      Ok(_) => {
        start_index += chunk_size;
        Ok(())
      },
      Err(err) => match classify_cloudwatch_error(err) {
        Action::Retry(cause) => {
          warn!("Retrying error {}", cause);
          break
        },
        Action::Fail(err) => Err(err),
      }
    }?;
  }
  Ok(start_index)
}

pub enum Action {
  Retry(String),
  Fail(Error),
}

pub fn classify_cloudwatch_error(error: PutMetricDataError) -> Action {
  match error {
    PutMetricDataError::HttpDispatch(err) => Action::Retry(format!("{:?}", err)),
    PutMetricDataError::InternalServiceFault(message) => Action::Retry(message),
    PutMetricDataError::Unknown(response) => Action::Retry(String::from_utf8(response.body).unwrap()),
    err => Action::Fail(format_err!("{}", err)),
  }
}

pub fn maintain_queue<F>(queue: &mut Metrics, max_size: usize, action: Box<F>) -> Result<(), Error>
    where F: Fn(&Metrics) -> Result<usize, Error> {
  let processed_items = action(&queue)?;
  let queue_overflow = max(queue.len() as isize - max_size as isize, 0) as usize;
  queue.drain(..max(processed_items, queue_overflow));
  Ok(())
}

const PROGRAM_DESC: &'static str = "Small daemon to report selected Docker stats as Cloudwatch metrics.";

pub enum RunMode {
  Normal(Configuration),
  Help(String),
}

pub fn parse_args(args: &Vec<String>) -> Result<RunMode, Error> {
  let mut argparser = Args::new("fargate-stats-reporter", PROGRAM_DESC);
  argparser.option(
    "n",
    "metric-namespace",
    "Namespace under which to report metrics",
    "NAMESPACE",
    Occur::Optional,
    None);
  argparser.option(
    "e",
    "metadata-endpoint",
    "HTTP base URL where /v2/metadata and /v2/stats can be found",
    "BASE URL",
    Occur::Optional,
    Some("http://169.254.170.2".to_owned())
  );
  argparser.option(
    "i",
    "interval",
    "Interval between reports to CloudWatch",
    "SECONDS",
    Occur::Optional,
    Some("60".to_owned())
  );
  argparser.option(
    "l",
    "log-level",
    "Increase logging verbosity (0 = error, 4 = trace)",
    "NUM",
    Occur::Optional,
    Some("1".to_owned())
  );
  argparser.option(
    "q",
    "queue-size",
    "Number of metric datums to keep in queue during communication outages",
    "QUEUE_SIZE",
    Occur::Optional,
    Some("100".to_owned())
  );
  argparser.flag("h", "help", "Print this help and exit");

  argparser.parse(args)?;

  if argparser.value_of("help")? {
    Ok(RunMode::Help(argparser.full_usage()))
  } else {
    Ok(RunMode::Normal(Configuration {
      base_url: argparser.value_of("metadata-endpoint")?,
      interval: Duration::from_secs(argparser.value_of("interval")?),
      log_level: argparser.value_of("log-level")?,
      namespace: argparser.value_of("metric-namespace")?,
      queue_size: argparser.value_of("queue-size")?,
    }))
  }
}

fn setup_logging(configuration: &Configuration) -> Result<(), Error> {
  stderrlog::new()
    .module(module_path!())
    .verbosity(configuration.log_level)
    .init()?;
  Ok(())
}

fn main() -> Result<(), Error> {
  let configuration = match parse_args(&args().collect())? {
    RunMode::Help(usage) => {
      println!("{}", usage);
      exit(0);
    },
    RunMode::Normal(configuration) => configuration,
  };
  setup_logging(&configuration)?;
  warn!("Starting with configuration {:?}", configuration);
  let mut metrics_queue = Metrics::new();
  let client = CloudWatchClient::new(Region::default());
  let http = HttpClient::builder()
    .timeout(Duration::from_secs(2))
    .build()?;
  loop {
let metadata = task_metadata(&http, &configuration.base_url)?;
let stats = container_stats(&http, &configuration.base_url)?;
    metrics_from_stats(&mut metrics_queue, stats, &metadata);
    maintain_queue(&mut metrics_queue, configuration.queue_size, Box::new(|metrics: &Metrics| {
      let sent_metrics = report_to_cloudwatch(&client, &configuration.namespace, &metrics)?;
      info!("Reported {}/{} metrics on {} containers", sent_metrics, metrics.len(), metadata.len());
      Ok(sent_metrics)
    }))?;
    sleep(configuration.interval);
  }
}
