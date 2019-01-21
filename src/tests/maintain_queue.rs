use failure::Error;
use rusoto_cloudwatch::MetricDatum;
use super::metric_datum;

#[test]
fn handles_empty_queue() -> Result<(), Error> {
  let mut queue = Vec::<MetricDatum>::new();
  crate::maintain_queue(&mut queue, 10, Box::new(|_: &Vec<MetricDatum>| Ok(0 as usize)))
}

#[test]
fn culls_queue_according_to_closure() -> Result<(), Error> {
  let mut queue = vec![metric_datum(), metric_datum(), metric_datum()];
  crate::maintain_queue(&mut queue, 10, Box::new(|_: &Vec<MetricDatum>| Ok(2 as usize)))?;
  assert_eq!(1, queue.len());
  Ok(())
}

#[test]
fn culls_queue_from_start() -> Result<(), Error> {
  let mut queue = vec![metric_datum(), metric_datum(), metric_datum()];
  queue[2].value = Some(26.0);
  crate::maintain_queue(&mut queue, 10, Box::new(|_: &Vec<MetricDatum>| Ok(2 as usize)))?;
  assert_eq!(Some(26.0), queue[0].value);
  Ok(())
}

#[test]
fn culls_queue_according_to_max_queue_size() -> Result<(), Error> {
  let mut queue = vec![metric_datum(), metric_datum(), metric_datum()];
  crate::maintain_queue(&mut queue, 2, Box::new(|_: &Vec<MetricDatum>| Ok(0 as usize)))?;
  assert_eq!(2, queue.len());
  Ok(())
}

#[test]
fn culls_queue_according_to_minimum() -> Result<(), Error> {
  let mut queue = vec![metric_datum(), metric_datum(), metric_datum()];
  crate::maintain_queue(&mut queue, 2, Box::new(|_: &Vec<MetricDatum>| Ok(1 as usize)))?;
  assert_eq!(2, queue.len());
  Ok(())
}
