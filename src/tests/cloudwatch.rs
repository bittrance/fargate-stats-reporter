use crate::cloudwatch;
use rusoto_cloudwatch::CloudWatchClient;
use rusoto_core::HttpDispatchError;
use rusoto_core::param::Params;
use rusoto_core::signature::{SignedRequest, SignedRequestPayload};
use rusoto_mock::{MockCredentialsProvider, MockRequestDispatcher};
use serde_urlencoded;
use std::iter::repeat;
use std::sync::{Arc, Mutex};
use super::metric_datum;

fn client_with_http_status(status: u16) -> CloudWatchClient {
  CloudWatchClient::new_with(
    MockRequestDispatcher::with_status(status),
    MockCredentialsProvider,
    Default::default()
  )
}

fn client_with_checker<F>(checker: F) -> CloudWatchClient where F: Fn(Params) + Send + Sync + 'static {
  CloudWatchClient::new_with(
    MockRequestDispatcher::with_status(200).with_request_checker(move |req: &SignedRequest|
      if let Some(SignedRequestPayload::Buffer(ref buffer)) = req.payload {
        let params: Params = serde_urlencoded::from_bytes(buffer).unwrap();
        checker(params);
      } else {
        panic!("Unexpected request.payload: {:?}", req.payload);
      }
    ),
    MockCredentialsProvider,
    Default::default()
  )
}

#[test]
fn posts_metric_data_to_cloudwatch() {
  let cw = client_with_checker(|params: Params| {
    assert_eq!(params.get("Namespace"), Some(&Some("testing".to_owned())));
    assert_eq!(params.get("MetricData.member.1.Value"), Some(&Some("25".to_owned())));
  });
  let mut data = vec![metric_datum()];
  cloudwatch::report_to_cloudwatch(&cw, "testing", &mut data).unwrap();
}

#[test]
fn sends_batches_of_20_metrics() {
  let count = Arc::new(Mutex::new(0));
  let copy = count.clone();
  let cw = client_with_checker(move |params: Params| {
    assert_eq!(params.get("Namespace"), Some(&Some("testing".to_owned())));
    assert_eq!(params.get("MetricData.member.20.Value"), Some(&Some("25".to_owned())));
    *count.lock().unwrap() += 1;
  });
  let data = repeat(metric_datum()).take(40).collect();
  cloudwatch::report_to_cloudwatch(&cw, "testing", &data).unwrap();
  assert_eq!(2, *copy.lock().unwrap());
}

#[test]
fn says_count_items_were_sent() {
  let cw = client_with_http_status(200);
  let data = vec![metric_datum(), metric_datum()];
  assert_eq!(2, cloudwatch::report_to_cloudwatch(&cw, "testing", &data).unwrap());
}

#[test]
fn says_zero_items_were_processed_on_dispatch_error() {
  let cw = CloudWatchClient::new_with(
    MockRequestDispatcher::with_dispatch_error(HttpDispatchError::new("boom!".to_owned())),
    MockCredentialsProvider,
    Default::default()
  );
  let data = vec![metric_datum()];
  assert_eq!(0, cloudwatch::report_to_cloudwatch(&cw, "testing", &data).unwrap());
}

#[test]
fn cloudwatch_server_side_error_is_readable() {
  let cw = CloudWatchClient::new_with(
    MockRequestDispatcher::with_status(400).with_body(
      r#"<ErrorResponse xmlns="http://monitoring.amazonaws.com/doc/2010-08-01/">
      <Error>
        <Type>Sender</Type>
        <Code>MissingParameter</Code>
        <Message>some message</Message>
      </Error>
      <RequestId>uuid</RequestId>
      </ErrorResponse>"#
    ),
    MockCredentialsProvider,
    Default::default()
  );
  let mut data = vec![metric_datum()];
  match cloudwatch::report_to_cloudwatch(&cw, "testing", &mut data) {
    Ok(_) => panic!("Expected failed request to return err"),
    Err(msg) => assert!(format!("{}", msg).contains("some message")),
  };
}
