use chrono::DateTime;
use rusoto_cloudwatch::{Dimension, MetricDatum};
use std::collections::HashMap;

fn stats() -> crate::Stats {
  crate::Stats {
    container_id: "ze-id".to_owned(),
    metrics: vec![crate::Metric {
      name: "max_usage".to_owned(),
      unit: "Bytes".to_owned(),
      value: 0.25
    }],
    timestamp: DateTime::parse_from_rfc3339("2019-01-07T23:15:48.677482816Z").unwrap(),
  }
}

#[test]
fn stats_to_cw_metrics() {
  let mut metrics = Vec::<MetricDatum>::new();
  let stats = vec![stats()];
  let mut metadata = HashMap::<String, crate::Metadata>::new();
  metadata.insert(
    "ze-id".to_owned(),
    crate::Metadata {
      container_id: "ze-id".to_owned(),
      dimensions: vec![
        Dimension {
          name: "task".to_owned(),
          value: "some-container".to_owned()
        }
      ]
    }
  );

  let expected = vec![
    MetricDatum {
      dimensions: Some(vec![
        Dimension {
          name: "task".to_owned(),
          value: "some-container".to_owned()
        }
      ]),
      metric_name: "max_usage".to_owned(),
      timestamp: Some("2019-01-07T23:15:48.677+00:00".to_owned()),
      unit: Some("Bytes".to_owned()),
      value: Some(0.25),
      ..Default::default()
    }
  ];
  crate::metrics_from_stats(&mut metrics, stats, &metadata);
  assert_eq!(expected, metrics);
}

#[test]
fn container_is_unknown() {
  let mut metrics = Vec::<MetricDatum>::new();
  let stats = vec![stats()];
  let metadata = HashMap::<String, crate::Metadata>::new();
  crate::metrics_from_stats(&mut metrics, stats, &metadata);
  assert_eq!(Vec::<MetricDatum>::new(), metrics);
}
