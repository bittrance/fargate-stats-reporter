use rusoto_cloudwatch::{CloudWatchClient, Dimension, MetricDatum};
use rusoto_mock::{MockCredentialsProvider, MockRequestDispatcher};

#[test]
fn cloudwatch_server_side_error_is_readable() {
  let cw = CloudWatchClient::new_with(
    MockRequestDispatcher::with_status(400).with_body(
      r#"<ErrorResponse xmlns="http://monitoring.amazonaws.com/doc/2010-08-01/">
      <Error>
        <Type>Sender</Type>
        <Code>AccessDenied</Code>
        <Message>User: foobar is not authorized to perform: cloudwatch:PutMetricData</Message>
      </Error>
      <RequestId>0bac1fb9-182f-11e9-93a4-8ba10ca20155</RequestId>
      </ErrorResponse>"#
    ),
    MockCredentialsProvider,
    Default::default()
  );
  let data = vec![
    MetricDatum {
      dimensions: Some(vec![Dimension { name: "".to_owned(), value: "".to_owned() }]),
      metric_name: "max_usage".to_owned(),
      timestamp: Some("".to_owned()),
      unit: Some("Bytes".to_owned()),
      value: Some(25.0),
      ..Default::default()
    }
  ];
  match crate::report_to_cloudwatch(&cw, "testing", data) {
    Ok(_) => panic!("Expected failed request to return err"),
    Err(msg) => assert!(format!("{}", msg).contains("foobar is not authorized")),
  };
}
