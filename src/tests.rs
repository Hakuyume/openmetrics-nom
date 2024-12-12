mod openmetrics_testdata;

use nom::combinator::complete;
use nom::error::VerboseError;
use nom::{Finish, Parser};
use std::fmt::Debug;

#[test]
fn test_overall_structure() {
    // https://github.com/prometheus/OpenMetrics/blob/main/specification/OpenMetrics.md#overall-structure
    let input = include_str!("tests/overall_structure.txt");
    complete(crate::exposition)
        .parse(input)
        .finish()
        .map_err(|e| nom::error::convert_error(input, e))
        .unwrap();
}

#[track_caller]
fn check<'a, O, F>(f: F, input: &'a str, expected: O)
where
    O: Debug + PartialEq,
    F: Parser<&'a str, O, VerboseError<&'a str>>,
{
    assert_eq!(
        complete(f)
            .parse(input)
            .finish()
            .map_err(|e| nom::error::convert_error(input, e)),
        Ok(("", expected))
    );
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
            crate::MetricDescriptor::Type {
                metricname: "foo",
                metric_type: ("counter", crate::MetricType::Counter),
            },
        ),
        (
            "# UNIT bar seconds\n",
            crate::MetricDescriptor::Unit {
                metricname: "bar",
                metricname_char: "seconds",
            },
        ),
        (
            "# HELP baz baz is qux.\n",
            crate::MetricDescriptor::Help {
                metricname: "baz",
                help_escaped_string: (
                    "baz is qux.",
                    crate::HelpEscapedString(vec![(
                        "baz is qux.",
                        crate::HelpEscapedStringFragment::Normal("baz is qux."),
                    )]),
                ),
            },
        ),
    ];
    let (input, expected) = cases.into_iter().nth(i).unwrap();
    check(crate::metric_descriptor, input, expected);
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

#[rstest::rstest]
// https://github.com/prometheus/OpenMetrics/blob/main/specification/OpenMetrics.md#numbers
#[case("23")]
#[case("0042")]
#[case("1341298465647914")]
#[case("03.123421")]
#[case("1.89e-7")]
fn test_number(#[case] input: &str) {
    check(crate::number, input, input);
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
            crate::HelpEscapedString(vec![(
                "foo",
                crate::HelpEscapedStringFragment::Normal("foo"),
            )]),
        ),
        (
            r#"\foo"#,
            crate::HelpEscapedString(vec![(
                r#"\foo"#,
                crate::HelpEscapedStringFragment::Normal(r#"\foo"#),
            )]),
        ),
        (
            r#"\\foo"#,
            crate::HelpEscapedString(vec![
                (r#"\\"#, crate::HelpEscapedStringFragment::Bs),
                ("foo", crate::HelpEscapedStringFragment::Normal("foo")),
            ]),
        ),
        (
            r#"foo\\"#,
            crate::HelpEscapedString(vec![
                ("foo", crate::HelpEscapedStringFragment::Normal("foo")),
                (r#"\\"#, crate::HelpEscapedStringFragment::Bs),
            ]),
        ),
        (
            r#"\\"#,
            crate::HelpEscapedString(vec![(r#"\\"#, crate::HelpEscapedStringFragment::Bs)]),
        ),
        (
            r#"\n"#,
            crate::HelpEscapedString(vec![(r#"\n"#, crate::HelpEscapedStringFragment::Lf)]),
        ),
        (
            r#"\\n"#,
            crate::HelpEscapedString(vec![
                (r#"\\"#, crate::HelpEscapedStringFragment::Bs),
                ("n", crate::HelpEscapedStringFragment::Normal("n")),
            ]),
        ),
        (
            r#"\\\n"#,
            crate::HelpEscapedString(vec![
                (r#"\\"#, crate::HelpEscapedStringFragment::Bs),
                (r#"\n"#, crate::HelpEscapedStringFragment::Lf),
            ]),
        ),
        (
            r#"\""#,
            crate::HelpEscapedString(vec![(
                r#"\""#,
                crate::HelpEscapedStringFragment::Normal(r#"\""#),
            )]),
        ),
        (
            r#"\\""#,
            crate::HelpEscapedString(vec![
                (r#"\\"#, crate::HelpEscapedStringFragment::Bs),
                (r#"""#, crate::HelpEscapedStringFragment::Normal(r#"""#)),
            ]),
        ),
    ];
    let (input, expected) = cases.into_iter().nth(i).unwrap();
    check(crate::help_escaped_string, input, expected);
}
