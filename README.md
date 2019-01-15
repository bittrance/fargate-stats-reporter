# Fargate stats reporter

AWS ECS only provides Cloudwatch metrics for services. However, in some cases (notably ETL), you may want to collect e.g. memory usage. ECS provides access to the Docker stats API via a special endpoint. Fargate stats reporter reads selected stats from this API and forwards them as Cloudwatch metrics (i.e. calling PutMetricData).

```
$ ./fargate-stats-reporter --help
Usage: fargate-stats-reporter [-n NAMESPACE] [-e BASE URL] [-l NUM] [-h]

Small daemon to report selected Docker stats as Cloudwatch metrics.

Options:
    -n, --metric-namespace NAMESPACE
                        Namespace under which to report metrics
    -e, --metadata-endpoint BASE URL
                        HTTP base URL where /v2/metadata and /v2/stats can be
                        found
    -l, --log-level NUM Increase logging verbosity (0 = error, 4 = trace)
    -h, --help          Print this help and exit
```

Fargate stats reporter is available as a minimalistic Docker image via Docker Hub, see https://hub.docker.com/r/bittrance/fargate-stats-reporter.
