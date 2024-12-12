use nom::error::Error;
use nom::{Finish, Parser};
use std::fmt::Debug;
use std::fs::{self, File};
use std::path::PathBuf;

#[track_caller]
fn check<'a, F>(mut f: F, input: &'a str, expected: F::Output)
where
    F: Parser<&'a str, Error = Error<&'a str>>,
    F::Output: Debug + PartialEq,
{
    assert_eq!(f.parse(input).finish(), Ok(("", expected)));
}

#[rstest::rstest]
#[case("counter", crate::MetricType::Counter)]
#[case("gauge", crate::MetricType::Gauge)]
#[case("histogram", crate::MetricType::Histogram)]
#[case("gaugehistogram", crate::MetricType::Gaugehistogram)]
#[case("stateset", crate::MetricType::Stateset)]
#[case("info", crate::MetricType::Info)]
#[case("summary", crate::MetricType::Summary)]
#[case("unknown", crate::MetricType::Unknown)]
fn test_metric_type(#[case] input: &str, #[case] expected: crate::MetricType) {
    check(crate::metric_type, input, expected);
}

// https://github.com/prometheus/OpenMetrics/blob/main/specification/OpenMetrics.md#numbers
#[rstest::rstest]
#[case("23")]
#[case("0042")]
#[case("1341298465647914")]
#[case("03.123421")]
#[case("1.89e-7")]
fn test_number(#[case] input: &str) {
    check(crate::number, input, input);
}

#[rstest::rstest]
fn test_testdata(
    #[base_dir = "./OpenMetrics/tests/testdata/parsers"]
    #[files("*")]
    // https://github.com/prometheus/OpenMetrics/issues/288
    #[exclude(r#"^help_escaping$"#)]
    path: PathBuf,
) {
    #[derive(Debug, serde::Deserialize)]
    struct Test {
        #[serde(rename = "type")]
        type_: String,
        file: PathBuf,
        #[serde(rename = "shouldParse")]
        should_parse: bool,
    }

    let test =
        serde_json::from_reader::<_, Test>(File::open(path.join("test.json")).unwrap()).unwrap();

    assert_eq!(test.type_, "text");
    let input = fs::read_to_string(path.join(&test.file)).unwrap();

    let exposition = crate::exposition::<_, Error<_>>
        .parse(input.as_str())
        .finish();

    if test.should_parse {
        exposition.unwrap();
    }
}
