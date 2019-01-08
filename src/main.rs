extern crate args;
extern crate failure;
extern crate getopts;
extern crate log;
extern crate reqwest;
extern crate rusoto_cloudwatch;
extern crate rusoto_core;
extern crate serde_json;

use args::Args;
use failure::Error;
use getopts::Occur;
use log::debug;
use rusoto_cloudwatch::{CloudWatch, CloudWatchClient, Dimension, MetricDatum, PutMetricDataInput};
use rusoto_core::Region;
use serde_json::Value;
use std::collections::HashMap;
use std::env::args;
use std::iter::FromIterator;
use std::thread::sleep;
use std::time::Duration;

#[cfg(test)] pub mod tests;

#[derive(Debug, PartialEq)]
pub struct Configuration {
  base_url: String,
  namespace: String,
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
  timestamp: String,
}

const DIMENSIONS_TO_COLLECT: [(&str, &str); 1] = [
  ("/Name", "task"),
];

pub fn task_metadata(base_url: &str) -> Result<HashMap<String, Metadata>, Error> {
  let containers: Vec<Value> = match reqwest::get(&format!("{}/v2/metadata", base_url))?
    .json::<Value>()?
    .get("Containers") {
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

const METRICS_TO_COLLECT: [(&str, &str, &str); 1] = [
  ("/memory_stats/max_usage", "max_usage", "Bytes"),
];

pub fn container_stats(base_url: &str) -> Result<Vec<Stats>, Error> {
  let body: Value = reqwest::get(&format!("{}/v2/stats", base_url))?
      .json()?;
  let stats = body.as_object().unwrap().iter()
    .map(|(id, stats)|
      Stats {
        container_id: id.clone(),
        metrics: METRICS_TO_COLLECT.iter().map(|(p, n, u)| Metric {
          name: String::from(*n),
          unit: String::from(*u),
          value: stats.pointer(p).unwrap().as_f64().unwrap(),
        }).collect(),
        timestamp: stats["read"].as_str().unwrap().to_owned(),
      }
    )
    .collect();
  Ok(stats)
}

type Metrics = Vec<MetricDatum>;

pub fn metrics_from_stats(stats: Vec<Stats>, metadata: &HashMap<String, Metadata>) -> Metrics {
  stats.into_iter()
    .filter(|c| metadata.contains_key(&c.container_id))
    .flat_map(|s| {
      let dimensions = &metadata.get(&s.container_id).unwrap().dimensions;
      let timestamp = s.timestamp;
      s.metrics.into_iter().map(move |m|
        MetricDatum {
          dimensions: Some(dimensions.clone()),
          metric_name: m.name,
          timestamp: Some(timestamp.clone()),
          unit: Some(m.unit),
          value: Some(m.value),
          ..Default::default()
        }
      )
    })
    .collect()
}

pub fn report_to_cloudwatch(client: &impl CloudWatch, namespace: &str, data: Metrics) -> Result<(), Error> {
  for datum in data {
    client.put_metric_data(PutMetricDataInput {
      namespace: String::from(namespace),
      metric_data: vec![datum]
    }).sync()?;
  }
  Ok(())
}

const PROGRAM_DESC: &'static str = "";

pub fn parse_args(args: &Vec<String>) -> Result<Configuration, Error> {
  let mut argparser = Args::new("fargate-stats-reporter", PROGRAM_DESC);
  argparser.option(
    "n",
    "metric-namespace",
    "Namespace under which to report metrics",
    "NAMESPACE",
    Occur::Req,
    None);
  argparser.option(
    "e",
    "metadata-endpoint",
    "HTTP base URL where /v2/metadata and /v2/stats can be found",
    "BASE URL",
    Occur::Optional,
    Some("http://169.254.170.2".to_owned())
  );

  argparser.parse(args)?;

  Ok(Configuration {
    base_url: argparser.value_of("metadata-endpoint")?,
    namespace: argparser.value_of("metric-namespace")?,
  })
}

fn main() -> Result<(), Error> {
  let configuration = parse_args(&args().collect())?;
  let client = CloudWatchClient::new(Region::default());
  loop {
    let metadata = task_metadata(&configuration.base_url)?;
    let stats = container_stats(&configuration.base_url)?;
    let metric_count = stats.iter().map(|s| s.metrics.len() as i32).sum::<i32>();
    let metrics = metrics_from_stats(stats, &metadata);
    report_to_cloudwatch(&client, &configuration.namespace, metrics)?;
    debug!("Reported {} metrics on {} containers", metric_count, metadata.len());
    sleep(Duration::from_millis(5000));
  }
}
