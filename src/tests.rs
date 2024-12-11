use nom::combinator::complete;
use nom::error::VerboseError;
use nom::Parser;
use std::fmt::Debug;
use std::fs::{self, File};
use std::path::PathBuf;

#[test]
fn test_overall_structure() {
    // https://github.com/prometheus/OpenMetrics/blob/main/specification/OpenMetrics.md#overall-structure
    let input = include_str!("overall_structure.txt");
    complete::<_, _, VerboseError<_>, _>(super::exposition)(input).unwrap();
}

fn openmetrics_testdata() -> Vec<(PathBuf, bool, String)> {
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
        .map(|entry| {
            let entry = entry.unwrap();
            assert!(entry.metadata().unwrap().is_dir());
            let test = serde_json::from_reader::<_, Test>(
                File::open(entry.path().join("test.json")).unwrap(),
            )
            .unwrap();
            assert_eq!(test.type_, "text");
            let input = fs::read_to_string(entry.path().join(&test.file)).unwrap();
            (entry.path(), test.should_parse, input)
        })
        .collect()
}

#[test]
fn test_openmetrics_testdata_ok() {
    for (path, should_parse, input) in openmetrics_testdata() {
        dbg!(path, should_parse);
        if should_parse {
            complete::<_, _, VerboseError<_>, _>(super::exposition)(input.as_str()).unwrap();
        }
    }
}

#[track_caller]
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
        "# TYPE foo counter\n",
        super::MetricDescriptor::Type {
            metricname: "foo",
            metric_type: ("counter", super::MetricType::Counter),
        },
    );
    check(
        super::metric_descriptor,
        "# UNIT bar seconds\n",
        super::MetricDescriptor::Unit {
            metricname: "bar",
            metricname_char: "seconds",
        },
    );
    check(
        super::metric_descriptor,
        "# HELP baz baz is qux.\n",
        super::MetricDescriptor::Help {
            metricname: "baz",
            escaped_string: (
                "baz is qux.",
                super::HelpEscapedString(vec![(
                    "baz is qux.",
                    super::HelpEscapedStringFragment::Normal("baz is qux."),
                )]),
            ),
        },
    );
}

#[test]
fn test_metric_type() {
    check(super::metric_type, "counter", super::MetricType::Counter);
    check(super::metric_type, "gauge", super::MetricType::Gauge);
    check(
        super::metric_type,
        "histogram",
        super::MetricType::Histogram,
    );
    check(
        super::metric_type,
        "gaugehistogram",
        super::MetricType::Gaugehistogram,
    );
    check(super::metric_type, "stateset", super::MetricType::Stateset);
    check(super::metric_type, "info", super::MetricType::Info);
    check(super::metric_type, "summary", super::MetricType::Summary);
    check(super::metric_type, "unknown", super::MetricType::Unknown);
}

#[test]
fn test_number() {
    // https://github.com/prometheus/OpenMetrics/blob/main/specification/OpenMetrics.md#numbers
    check(super::number, "23", "23");
    check(super::number, "0042", "0042");
    check(super::number, "1341298465647914", "1341298465647914");
    check(super::number, "03.123421", "03.123421");
    check(super::number, "1.89e-7", "1.89e-7");
}

#[test]
fn test_help_escaped_string() {
    // https://github.com/prometheus/OpenMetrics/blob/main/tests/testdata/parsers/help_escaping/metrics
    check(
        super::help_escaped_string,
        "foo",
        super::HelpEscapedString(vec![(
            "foo",
            super::HelpEscapedStringFragment::Normal("foo"),
        )]),
    );
    check(
        super::help_escaped_string,
        r#"\foo"#,
        super::HelpEscapedString(vec![(
            r#"\foo"#,
            super::HelpEscapedStringFragment::Normal(r#"\foo"#),
        )]),
    );
    check(
        super::help_escaped_string,
        r#"\\foo"#,
        super::HelpEscapedString(vec![
            (r#"\\"#, super::HelpEscapedStringFragment::Bs),
            ("foo", super::HelpEscapedStringFragment::Normal("foo")),
        ]),
    );
    check(
        super::help_escaped_string,
        r#"foo\\"#,
        super::HelpEscapedString(vec![
            ("foo", super::HelpEscapedStringFragment::Normal("foo")),
            (r#"\\"#, super::HelpEscapedStringFragment::Bs),
        ]),
    );
    check(
        super::help_escaped_string,
        r#"\\"#,
        super::HelpEscapedString(vec![(r#"\\"#, super::HelpEscapedStringFragment::Bs)]),
    );
    check(
        super::help_escaped_string,
        r#"\n"#,
        super::HelpEscapedString(vec![(r#"\n"#, super::HelpEscapedStringFragment::Lf)]),
    );
    check(
        super::help_escaped_string,
        r#"\\n"#,
        super::HelpEscapedString(vec![
            (r#"\\"#, super::HelpEscapedStringFragment::Bs),
            ("n", super::HelpEscapedStringFragment::Normal("n")),
        ]),
    );
    check(
        super::help_escaped_string,
        r#"\\\n"#,
        super::HelpEscapedString(vec![
            (r#"\\"#, super::HelpEscapedStringFragment::Bs),
            (r#"\n"#, super::HelpEscapedStringFragment::Lf),
        ]),
    );
    check(
        super::help_escaped_string,
        r#"\""#,
        super::HelpEscapedString(vec![(
            r#"\""#,
            super::HelpEscapedStringFragment::Normal(r#"\""#),
        )]),
    );
    check(
        super::help_escaped_string,
        r#"\\""#,
        super::HelpEscapedString(vec![
            (r#"\\"#, super::HelpEscapedStringFragment::Bs),
            (r#"""#, super::HelpEscapedStringFragment::Normal(r#"""#)),
        ]),
    );
}
