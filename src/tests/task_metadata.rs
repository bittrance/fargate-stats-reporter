use mockito::mock;
use reqwest::Client as HttpClient;
use rusoto_cloudwatch::Dimension;
use serde_json::json;
use std::collections::HashMap;

#[test]
fn no_containers() {
  let http = HttpClient::new();
  let reply = json!({"Containers": []});

  let _metadata_api = mock("GET", "/v2/metadata")
    .with_status(200)
    .with_header("content-type", "application/json")
    .with_body(reply.to_string())
    .create();

  assert_eq!(
    HashMap::<String, crate::Metadata>::new(),
    crate::task_metadata(&http, &mockito::server_url()).unwrap()
  );
}

#[test]
fn with_containers() {
  let http = HttpClient::new();
  let reply = json!({"Containers": [
    {
      "DockerId": "ze-id",
      "Name": "some-container",
      "Ignore": "this"
    },
    {
      "DockerId": "ze-id",
      "Name": "~internal~ecs~pause",
    }
  ]});

  let _metadata_api = mock("GET", "/v2/metadata")
    .with_status(200)
    .with_header("content-type", "application/json")
    .with_body(reply.to_string())
    .create();

  let actual = crate::task_metadata(&http, &mockito::server_url()).unwrap();
  let mut expected = HashMap::<String, crate::Metadata>::new();
  expected.insert(
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
  assert_eq!(expected, actual);
}
