use crate::metadata_v2;
use failure::{Error, format_err};
use log::warn;
use rusoto_cloudwatch::{CloudWatch, MetricDatum, PutMetricDataError, PutMetricDataInput};
use std::collections::HashMap;

pub type Metrics = Vec<MetricDatum>;

pub fn metrics_from_stats(metrics: &mut Metrics, stats: Vec<metadata_v2::Stats>, metadata: &HashMap<String, metadata_v2::Metadata>) {
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
