extern crate rusoto_mock;
extern crate serde_urlencoded;

mod container_stats;
mod maintain_queue;
mod metrics_from_stats;
mod config;
mod cloudwatch;
mod task_metadata;

use rusoto_cloudwatch::{Dimension, MetricDatum};

fn metric_datum() -> MetricDatum {
  MetricDatum {
    dimensions: Some(vec![Dimension { name: "container".to_owned(), value: "ze-id".to_owned() }]),
    metric_name: "max_usage".to_owned(),
    timestamp: Some("ze-time".to_owned()),
    unit: Some("Bytes".to_owned()),
    value: Some(25.0),
    ..Default::default()
  }
}
