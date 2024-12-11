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

#[rstest::rstest]
fn test_openmetrics_testdata(
    #[base_dir = "./OpenMetrics/tests/testdata/parsers"]
    #[files("*/test.json")]
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

    let test = serde_json::from_reader::<_, Test>(File::open(&path).unwrap()).unwrap();
    dbg!(&test);

    assert_eq!(test.type_, "text");

    let input = fs::read_to_string(path.with_file_name(&test.file)).unwrap();

    if test.should_parse {
        complete::<_, _, VerboseError<_>, _>(super::exposition)(input.as_str()).unwrap();
    } else {
        // complete::<_, _, VerboseError<_>, _>(super::exposition)(input.as_str()).unwrap_err();
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

seq_macro::seq!(I in 0..3 {
    #[rstest::rstest]
    #(#[case(I)])*
    fn test_metric_descriptor(#[case] i: usize) {
        test_metric_descriptor_impl(i);
    }
});
fn test_metric_descriptor_impl(i: usize) {
    let cases = [
        (
            "# TYPE foo counter\n",
            super::MetricDescriptor::Type {
                metricname: "foo",
                metric_type: ("counter", super::MetricType::Counter),
            },
        ),
        (
            "# UNIT bar seconds\n",
            super::MetricDescriptor::Unit {
                metricname: "bar",
                metricname_char: "seconds",
            },
        ),
        (
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
        ),
    ];
    let (input, expected) = cases.into_iter().nth(i).unwrap();
    check(super::metric_descriptor, input, expected);
}

#[rstest::rstest]
#[case("counter", super::super::MetricType::Counter)]
#[case("gauge", super::super::MetricType::Gauge)]
#[case("histogram", super::super::MetricType::Histogram)]
#[case("gaugehistogram", super::super::MetricType::Gaugehistogram)]
#[case("stateset", super::super::MetricType::Stateset)]
#[case("info", super::super::MetricType::Info)]
#[case("summary", super::super::MetricType::Summary)]
#[case("unknown", super::super::MetricType::Unknown)]
fn test_metric_type(#[case] input: &str, #[case] expected: super::MetricType) {
    check(super::metric_type, input, expected);
}

#[rstest::rstest]
// https://github.com/prometheus/OpenMetrics/blob/main/specification/OpenMetrics.md#numbers
#[case("23")]
#[case("0042")]
#[case("1341298465647914")]
#[case("03.123421")]
#[case("1.89e-7")]
fn test_number(#[case] input: &str) {
    check(super::number, input, input);
}

seq_macro::seq!(I in 0..10 {
    #[rstest::rstest]
    #(#[case(I)])*
    fn test_help_escaped_string(#[case] i: usize) {
        test_help_escaped_string_impl(i);
    }
});
fn test_help_escaped_string_impl(i: usize) {
    let cases = [
        // https://github.com/prometheus/OpenMetrics/blob/main/tests/testdata/parsers/help_escaping/metrics
        (
            "foo",
            super::HelpEscapedString(vec![(
                "foo",
                super::HelpEscapedStringFragment::Normal("foo"),
            )]),
        ),
        (
            r#"\foo"#,
            super::HelpEscapedString(vec![(
                r#"\foo"#,
                super::HelpEscapedStringFragment::Normal(r#"\foo"#),
            )]),
        ),
        (
            r#"\\foo"#,
            super::HelpEscapedString(vec![
                (r#"\\"#, super::HelpEscapedStringFragment::Bs),
                ("foo", super::HelpEscapedStringFragment::Normal("foo")),
            ]),
        ),
        (
            r#"foo\\"#,
            super::HelpEscapedString(vec![
                ("foo", super::HelpEscapedStringFragment::Normal("foo")),
                (r#"\\"#, super::HelpEscapedStringFragment::Bs),
            ]),
        ),
        (
            r#"\\"#,
            super::HelpEscapedString(vec![(r#"\\"#, super::HelpEscapedStringFragment::Bs)]),
        ),
        (
            r#"\n"#,
            super::HelpEscapedString(vec![(r#"\n"#, super::HelpEscapedStringFragment::Lf)]),
        ),
        (
            r#"\\n"#,
            super::HelpEscapedString(vec![
                (r#"\\"#, super::HelpEscapedStringFragment::Bs),
                ("n", super::HelpEscapedStringFragment::Normal("n")),
            ]),
        ),
        (
            r#"\\\n"#,
            super::HelpEscapedString(vec![
                (r#"\\"#, super::HelpEscapedStringFragment::Bs),
                (r#"\n"#, super::HelpEscapedStringFragment::Lf),
            ]),
        ),
        (
            r#"\""#,
            super::HelpEscapedString(vec![(
                r#"\""#,
                super::HelpEscapedStringFragment::Normal(r#"\""#),
            )]),
        ),
        (
            r#"\\""#,
            super::HelpEscapedString(vec![
                (r#"\\"#, super::HelpEscapedStringFragment::Bs),
                (r#"""#, super::HelpEscapedStringFragment::Normal(r#"""#)),
            ]),
        ),
    ];
    let (input, expected) = cases.into_iter().nth(i).unwrap();
    check(super::help_escaped_string, input, expected);
}
