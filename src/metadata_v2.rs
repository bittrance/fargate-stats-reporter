use chrono::{DateTime, FixedOffset};
use failure::Error;
use log::debug;
use reqwest::Client as HttpClient;
use rusoto_cloudwatch::Dimension;
use serde_json::Value;
use std::iter::FromIterator;
use std::collections::HashMap;

#[derive(Debug, PartialEq)]
pub struct Metadata {
  #[allow(dead_code)]
  pub container_id: String,
  pub dimensions: Vec<Dimension>,
}

#[derive(Debug, PartialEq)]
pub struct Metric {
  pub name: String,
  pub unit: String,
  pub value: f64,
}

#[derive(Debug, PartialEq)]
pub struct Stats {
  pub container_id: String,
  pub metrics: Vec<Metric>,
  pub timestamp: DateTime<FixedOffset>,
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
