use chrono::DateTime;
use rusoto_cloudwatch::{Dimension, MetricDatum};
use std::collections::HashMap;

#[test]
fn stats_to_cw_metrics() {
  let stats = vec![
    crate::Stats {
      container_id: "ze-id".to_owned(),
      metrics: vec![crate::Metric {
        name: "max_usage".to_owned(),
        unit: "Bytes".to_owned(),
        value: 0.25
      }],
      timestamp: DateTime::parse_from_rfc3339("2019-01-07T23:15:48.677482816Z").unwrap(),
    }
  ];
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
  assert_eq!(expected, crate::metrics_from_stats(stats, &metadata));
}

#[test]
fn container_is_unknown() {
  let stats = vec![
    crate::Stats {
      container_id: "ze-id".to_owned(),
      metrics: vec![crate::Metric {
        name: "max_usage".to_owned(),
        unit: "Bytes".to_owned(),
        value: 0.25
      }],
      timestamp: DateTime::parse_from_rfc3339("2019-01-07T23:15:48.677482816Z").unwrap(),
    }
  ];
  let metadata = HashMap::<String, crate::Metadata>::new();
  assert_eq!(Vec::<MetricDatum>::new(), crate::metrics_from_stats(stats, &metadata));
}
