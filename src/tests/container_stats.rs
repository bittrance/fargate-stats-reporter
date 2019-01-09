use mockito::mock;
use serde_json::json;

#[test]
fn no_containers() {
  let reply = json!({});

  let _stats_api = mock("GET", "/v2/stats")
    .with_status(200)
    .with_header("content-type", "application/json")
    .with_body(reply.to_string())
    .create();

  assert_eq!(
    crate::container_stats(&mockito::server_url()).unwrap(),
    Vec::<crate::Stats>::new()
  );
}

#[test]
fn extrace_container_stats() {
  let reply = json!({
    "ze-id": {
      "read": "2019-01-07T23:15:48.677482816Z",
      "memory_stats": {
        "max_usage": 0.25
      }
    }
  });

  let _stats_api = mock("GET", "/v2/stats")
    .with_status(200)
    .with_header("content-type", "application/json")
    .with_body(reply.to_string())
    .create();

  let expected = vec![
    crate::Stats {
      container_id: "ze-id".to_owned(),
      metrics: vec![crate::Metric {
        name: "max_usage".to_owned(),
        unit: "Bytes".to_owned(),
        value: 0.25
      }],
      timestamp: "2019-01-07T23:15:48.677482816Z".to_owned(),
    }
  ];
  let actual = crate::container_stats(&mockito::server_url()).unwrap();
  assert_eq!(expected, actual);
}

#[test]
fn new_container_is_null() {
  let reply = json!({"ze-id": null});

  let _stats_api = mock("GET", "/v2/stats")
    .with_status(200)
    .with_header("content-type", "application/json")
    .with_body(reply.to_string())
    .create();

  assert_eq!(
    crate::container_stats(&mockito::server_url()).unwrap(),
    Vec::<crate::Stats>::new()
  );
}
