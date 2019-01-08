#[test]
fn namespace_is_mandatory() {
  match crate::parse_args(&Vec::<String>::new()) {
    Ok(_) => panic!("Expected failure messag"),
    Err(_) => (),
  };
}

#[test]
fn minimum_configuration() {
  let args = vec!["-n".to_owned(), "some-namespace".to_owned()];
  assert_eq!(
    crate::Configuration {
      base_url: "http://169.254.170.2".to_owned(),
      namespace: "some-namespace".to_owned(),
    },
    crate::parse_args(&args).unwrap()
  );
}
