use nom::combinator::complete;
use nom::error::VerboseError;
use nom::Parser;
use std::fmt::Debug;
use std::fs::{self, File};
use std::path::PathBuf;

fn check<'a, O, F>(mut f: F, input: &'a str, expected: O)
where
    O: Debug + PartialEq,
    F: Parser<&'a str, O, VerboseError<&'a str>>,
{
    assert_eq!(f.parse(input), Ok(("", expected)));
}

#[test]
fn test_metric_descriptor() {
    check(
        super::metric_descriptor,
        "# TYPE acme_http_router_request_seconds summary\n",
        super::MetricDescriptor::Type {
            metricname: "acme_http_router_request_seconds",
            metric_type: ("summary", super::MetricType::Summary),
        },
    );
    check(
        super::metric_descriptor,
        "# UNIT acme_http_router_request_seconds seconds\n",
        super::MetricDescriptor::Unit {
            metricname: "acme_http_router_request_seconds",
            metricname_char: "seconds",
        },
    );
    check(
        super::metric_descriptor,
        "# HELP acme_http_router_request_seconds Latency though all of ACME's HTTP request router.\n",
        super::MetricDescriptor::Help {
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
            metricname: "acme_http_router_request_seconds_sum",
            labels: Some((
                r#"{path="/api/v1",method="GET"}"#,
                super::Labels {
                    label: vec![
                        (
                            r#"path="/api/v1""#,
                            super::Label {
                                label_name: "path",
                                escaped_string: "/api/v1",
                            },
                        ),
                        (
                            r#"method="GET""#,
                            super::Label {
                                label_name: "method",
                                escaped_string: "GET",
                            },
                        ),
                    ],
                },
            )),
            number: "9036.32",
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

#[test]
fn test_overall_structure() {
    // https://github.com/prometheus/OpenMetrics/blob/main/specification/OpenMetrics.md#overall-structure
    let input = include_str!("overall_structure.txt");
    complete::<_, _, VerboseError<_>, _>(super::exposition)(input).unwrap();
}

fn openmetrics_testdata() -> Vec<(PathBuf, bool, Vec<u8>)> {
    #[derive(serde::Deserialize)]
    struct Test {
        #[serde(rename = "type")]
        type_: String,
        file: PathBuf,
        #[serde(rename = "shouldParse")]
        should_parse: bool,
    }

    fs::read_dir("./OpenMetrics/tests/testdata/parsers")
        .unwrap()
        .into_iter()
        .map(|entry| {
            let entry = entry.unwrap();
            assert!(entry.metadata().unwrap().is_dir());
            let test = serde_json::from_reader::<_, Test>(
                File::open(entry.path().join("test.json")).unwrap(),
            )
            .unwrap();
            assert_eq!(test.type_, "text");
            let input = fs::read(entry.path().join(&test.file)).unwrap();
            (entry.path(), test.should_parse, input)
        })
        .collect()
}

#[test]
fn test_openmetrics_testdata_ok() {
    for (path, should_parse, input) in openmetrics_testdata() {
        dbg!(path, should_parse);
        if should_parse {
            complete::<_, _, VerboseError<_>, _>(super::exposition)(&input[..]).unwrap();
        }
    }
}
