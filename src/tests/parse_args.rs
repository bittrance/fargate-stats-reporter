fn with_mandatory(mut extra: Vec<String>) -> Vec<String> {
  let mut args = vec!["-n".to_owned(), "some-namespace".to_owned()];
  args.append(&mut extra);
  args
}

#[test]
fn namespace_is_mandatory() {
  match crate::parse_args(&Vec::<String>::new()) {
    Ok(_) => panic!("Expected failure messag"),
    Err(_) => (),
  };
}

#[test]
fn minimum_configuration() {
  let args = with_mandatory(Vec::<String>::new());
  assert_eq!(
    crate::Configuration {
      base_url: "http://169.254.170.2".to_owned(),
      log_level: 1,
      namespace: "some-namespace".to_owned(),
    },
    crate::parse_args(&args).unwrap()
  );
}

#[test]
fn info_log_level() {
  let args = with_mandatory(vec!["-l".to_owned(), "2".to_owned()]);
  assert_eq!(
    crate::Configuration {
      base_url: "http://169.254.170.2".to_owned(),
      log_level: 2,
      namespace: "some-namespace".to_owned(),
    },
    crate::parse_args(&args).unwrap()
  );
}
