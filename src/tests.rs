use nom::combinator::complete;
use nom::error::VerboseError;
use nom::Parser;
use std::fmt::Debug;

fn check<'a, F, O>(mut f: F, input: &'a str, expected: O)
where
    F: Parser<&'a str, O, VerboseError<&'a str>>,
    O: Debug + PartialEq,
{
    assert_eq!(f.parse(input), Ok(("", expected)));
}

#[test]
fn test_overall_structure() {
    let input = include_str!("overall_structure.txt");
    complete::<_, _, VerboseError<_>, _>(super::exposition)(input).unwrap();
}

#[test]
fn test_metric_descriptor() {
    check(
        super::metric_descriptor,
        "# TYPE acme_http_router_request_seconds summary\n",
        super::MetricDescriptor::Type {
            consumed: "# TYPE acme_http_router_request_seconds summary\n",
            metricname: "acme_http_router_request_seconds",
            metric_type: super::MetricType::Summary("summary"),
        },
    );
    check(
        super::metric_descriptor,
        "# UNIT acme_http_router_request_seconds seconds\n",
        super::MetricDescriptor::Unit {
            consumed: "# UNIT acme_http_router_request_seconds seconds\n",
            metricname: "acme_http_router_request_seconds",
            metricname_char: "seconds",
        },
    );
    check(
        super::metric_descriptor,
        "# HELP acme_http_router_request_seconds Latency though all of ACME's HTTP request router.\n",
        super::MetricDescriptor::Help {
            consumed: "# HELP acme_http_router_request_seconds Latency though all of ACME's HTTP request router.\n",
            metricname: "acme_http_router_request_seconds",
            escaped_string: "Latency though all of ACME's HTTP request router.",
        },
    );
}

#[test]
fn test_sample() {
    check(
        super::sample,
        "acme_http_router_request_seconds_sum{path=\"/api/v1\",method=\"GET\"} 9036.32\n",
        super::Sample {
            consumed:
                "acme_http_router_request_seconds_sum{path=\"/api/v1\",method=\"GET\"} 9036.32\n",
            metricname: "acme_http_router_request_seconds_sum",
            labels: Some(super::Labels {
                consumed: r#"{path="/api/v1",method="GET"}"#,
                labels: vec![
                    super::Label {
                        consumed: r#"path="/api/v1""#,
                        label_name: "path",
                        escaped_string: "/api/v1",
                    },
                    super::Label {
                        consumed: r#"method="GET""#,
                        label_name: "method",
                        escaped_string: "GET",
                    },
                ],
            }),
            number: super::Number::Real("9036.32"),
            timestamp: None,
            exemplar: None,
        },
    );
}

#[test]
fn test_metricname() {
    check(
        super::metricname,
        "acme_http_router_request_seconds_sum",
        "acme_http_router_request_seconds_sum",
    );
}

#[test]
fn test_label_name() {
    check(super::escaped_string, "path", "path");
}

#[test]
fn test_escaped_string() {
    check(super::escaped_string, "9036.32", "9036.32");
    check(super::escaped_string, "69", "69");
    check(super::escaped_string, "4.20072246e+06", "4.20072246e+06");
}
