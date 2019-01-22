use args::Args;
use failure::Error;
use getopts::Occur;
use std::time::Duration;

#[derive(Debug, Default, PartialEq)]
pub struct Configuration {
  pub base_url: String,
  pub interval: Duration,
  pub log_level: usize,
  pub namespace: String,
  pub queue_size: usize,
}

const PROGRAM_DESC: &'static str = "Small daemon to report selected Docker stats as Cloudwatch metrics.";

pub enum RunMode {
  Normal(Configuration),
  Help(String),
}

pub fn parse_args(args: &Vec<String>) -> Result<RunMode, Error> {
  let mut argparser = Args::new("fargate-stats-reporter", PROGRAM_DESC);
  argparser.option(
    "n",
    "metric-namespace",
    "Namespace under which to report metrics",
    "NAMESPACE",
    Occur::Optional,
    None);
  argparser.option(
    "e",
    "metadata-endpoint",
    "HTTP base URL where /v2/metadata and /v2/stats can be found",
    "BASE URL",
    Occur::Optional,
    Some("http://169.254.170.2".to_owned())
  );
  argparser.option(
    "i",
    "interval",
    "Interval between reports to CloudWatch",
    "SECONDS",
    Occur::Optional,
    Some("60".to_owned())
  );
  argparser.option(
    "l",
    "log-level",
    "Increase logging verbosity (0 = error, 4 = trace)",
    "NUM",
    Occur::Optional,
    Some("1".to_owned())
  );
  argparser.option(
    "q",
    "queue-size",
    "Number of metric datums to keep in queue during communication outages",
    "QUEUE_SIZE",
    Occur::Optional,
    Some("100".to_owned())
  );
  argparser.flag("h", "help", "Print this help and exit");

  argparser.parse(args)?;

  if argparser.value_of("help")? {
    Ok(RunMode::Help(argparser.full_usage()))
  } else {
    Ok(RunMode::Normal(Configuration {
      base_url: argparser.value_of("metadata-endpoint")?,
      interval: Duration::from_secs(argparser.value_of("interval")?),
      log_level: argparser.value_of("log-level")?,
      namespace: argparser.value_of("metric-namespace")?,
      queue_size: argparser.value_of("queue-size")?,
    }))
  }
}
