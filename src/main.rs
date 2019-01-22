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

use failure::Error;
use log::{info, warn};
use reqwest::Client as HttpClient;
use rusoto_cloudwatch::CloudWatchClient;
use rusoto_core::Region;
use std::cmp::max;
use std::env::args;
use std::process::exit;
use std::thread::sleep;
use std::time::Duration;

pub mod cloudwatch;
pub mod config;
pub mod metadata_v2;
#[cfg(test)] pub mod tests;

pub fn maintain_queue<F>(queue: &mut cloudwatch::Metrics, max_size: usize, action: Box<F>) -> Result<(), Error>
    where F: Fn(&cloudwatch::Metrics) -> Result<usize, Error> {
  let processed_items = action(&queue)?;
  let queue_overflow = max(queue.len() as isize - max_size as isize, 0) as usize;
  queue.drain(..max(processed_items, queue_overflow));
  Ok(())
}

fn setup_logging(configuration: &config::Configuration) -> Result<(), Error> {
  stderrlog::new()
    .module(module_path!())
    .verbosity(configuration.log_level)
    .init()?;
  Ok(())
}

fn main() -> Result<(), Error> {
  let configuration = match config::parse_args(&args().collect())? {
    config::RunMode::Help(usage) => {
      println!("{}", usage);
      exit(0);
    },
    config::RunMode::Normal(configuration) => configuration,
  };
  setup_logging(&configuration)?;
  warn!("Starting with configuration {:?}", configuration);
  let mut metrics_queue = cloudwatch::Metrics::new();
  let client = CloudWatchClient::new(Region::default());
  let http = HttpClient::builder()
    .timeout(Duration::from_secs(2))
    .build()?;
  loop {
    let metadata = metadata_v2::task_metadata(&http, &configuration.base_url)?;
    let stats = metadata_v2::container_stats(&http, &configuration.base_url)?;
    cloudwatch::metrics_from_stats(&mut metrics_queue, stats, &metadata);
    maintain_queue(&mut metrics_queue, configuration.queue_size, Box::new(|metrics: &cloudwatch::Metrics| {
      let sent_metrics = cloudwatch::report_to_cloudwatch(&client, &configuration.namespace, &metrics)?;
      info!("Reported {}/{} metrics on {} containers", sent_metrics, metrics.len(), metadata.len());
      Ok(sent_metrics)
    }))?;
    sleep(configuration.interval);
  }
}
