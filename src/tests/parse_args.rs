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
fn help_option() {
  crate::parse_args(&vec!["-h".to_owned()]).unwrap();
}

#[test]
fn minimum_configuration() {
  let args = with_mandatory(Vec::<String>::new());
  if let crate::RunMode::Normal(res) = crate::parse_args(&args).unwrap() {
    assert_eq!("http://169.254.170.2", res.base_url);
    assert_eq!(1, res.log_level);
    assert_eq!("some-namespace", res.namespace);
  } else {
    panic!("Expected a RunMode::Normal");
  }
}

#[test]
fn info_log_level() {
  let args = with_mandatory(vec!["-l".to_owned(), "2".to_owned()]);
  if let crate::RunMode::Normal(res) = crate::parse_args(&args).unwrap() {
    assert_eq!(2, res.log_level);
  } else {
    panic!("Expected a RunMode::Normal");
  }
}

#[test]
fn print_help() {
  if let crate::RunMode::Help(res) = crate::parse_args(&vec!["-h".to_owned()]).unwrap() {
    assert!(res.contains("Usage:"));
  } else {
    panic!("Expected a RunMode::Normal");
  }
}
