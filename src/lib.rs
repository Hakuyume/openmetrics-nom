use nom::branch::alt;
use nom::bytes::complete::{tag, tag_no_case, take_while};
use nom::character::complete::{char, satisfy};
use nom::combinator::{consumed, opt, recognize};
use nom::error::{ContextError, ParseError, context};
use nom::multi::{fold_many0, fold_many1, many0, many1, separated_list0};
use nom::number::complete::recognize_float;
use nom::{AsChar, Compare, IResult, Input, Offset, Parser};

// RFC 5234 B.1.
const DQUOTE: char = '"';
const SP: char = ' ';
const LF: char = '\n';

// https://github.com/prometheus/OpenMetrics/blob/main/specification/OpenMetrics.md#abnf

#[derive(Clone, Debug, PartialEq)]
pub struct Exposition<I> {
    pub metricset: (I, Metricset<I>),
}
pub fn exposition<I, E>(input: I) -> IResult<I, Exposition<I>, E>
where
    I: Compare<&'static str> + Input + Offset,
    I::Item: AsChar,
    E: ContextError<I> + ParseError<I>,
{
    context(
        "exposition",
        (
            consumed(metricset),
            char(HASH),
            char(SP),
            tag(EOF),
            opt(char(LF)),
        ),
    )
    .map(|(metricset, _, _, _, _)| Exposition { metricset })
    .parse(input)
}

#[derive(Clone, Debug, PartialEq)]
pub struct Metricset<I> {
    pub metricfamily: Vec<(I, Metricfamily<I>)>,
}
pub fn metricset<I, E>(input: I) -> IResult<I, Metricset<I>, E>
where
    I: Compare<&'static str> + Input + Offset,
    I::Item: AsChar,
    E: ContextError<I> + ParseError<I>,
{
    context("metricset", many0(consumed(metricfamily)))
        .map(|metricfamily| Metricset { metricfamily })
        .parse(input)
}

#[derive(Clone, Debug, PartialEq)]
pub struct Metricfamily<I> {
    pub metric_descriptor: Vec<(I, MetricDescriptor<I>)>,
    pub metric: Vec<(I, Metric<I>)>,
}
pub fn metricfamily<I, E>(input: I) -> IResult<I, Metricfamily<I>, E>
where
    I: Compare<&'static str> + Input + Offset,
    I::Item: AsChar,
    E: ContextError<I> + ParseError<I>,
{
    context(
        "metricfamily",
        alt((
            (many1(consumed(metric_descriptor)), many0(consumed(metric))),
            (many0(consumed(metric_descriptor)), many1(consumed(metric))),
        )),
    )
    .map(|(metric_descriptor, metric)| Metricfamily {
        metric_descriptor,
        metric,
    })
    .parse(input)
}

#[derive(Clone, Debug, PartialEq)]
pub enum MetricDescriptor<I> {
    Type {
        metricname: I,
        metric_type: (I, MetricType),
    },
    Help {
        metricname: I,
        escaped_string: (I, EscapedString<I>),
    },
    Unit {
        metricname: I,
        metricname_char: I,
    },
}
pub fn metric_descriptor<I, E>(input: I) -> IResult<I, MetricDescriptor<I>, E>
where
    I: Compare<&'static str> + Input + Offset,
    I::Item: AsChar,
    E: ContextError<I> + ParseError<I>,
{
    context(
        "metric_descriptor",
        alt((
            (
                char(HASH),
                char(SP),
                tag(TYPE),
                char(SP),
                metricname,
                char(SP),
                consumed(metric_type),
                char(LF),
            )
                .map(|(_, _, _, _, metricname, _, metric_type, _)| {
                    MetricDescriptor::Type {
                        metricname,
                        metric_type,
                    }
                }),
            (
                char(HASH),
                char(SP),
                tag(HELP),
                char(SP),
                metricname,
                char(SP),
                consumed(escaped_string),
                char(LF),
            )
                .map(|(_, _, _, _, metricname, _, escaped_string, _)| {
                    MetricDescriptor::Help {
                        metricname,
                        escaped_string,
                    }
                }),
            (
                char(HASH),
                char(SP),
                tag(UNIT),
                char(SP),
                metricname,
                char(SP),
                take_while(|c: I::Item| is_metricname_char(c.as_char())),
                char(LF),
            )
                .map(|(_, _, _, _, metricname, _, metricname_char, _)| {
                    MetricDescriptor::Unit {
                        metricname,
                        metricname_char,
                    }
                }),
        )),
    )
    .parse(input)
}

#[derive(Clone, Debug, PartialEq)]
pub struct Metric<I> {
    pub sample: Vec<(I, Sample<I>)>,
}
pub fn metric<I, E>(input: I) -> IResult<I, Metric<I>, E>
where
    I: Compare<&'static str> + Input + Offset,
    I::Item: AsChar,
    E: ContextError<I> + ParseError<I>,
{
    context("metric", many1(consumed(sample)))
        .map(|sample| Metric { sample })
        .parse(input)
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum MetricType {
    Counter,
    Gauge,
    Histogram,
    Gaugehistogram,
    Stateset,
    Info,
    Summary,
    Unknown,
}
pub fn metric_type<I, E>(input: I) -> IResult<I, MetricType, E>
where
    I: Compare<&'static str> + Input + Offset,
    I::Item: AsChar,
    E: ContextError<I> + ParseError<I>,
{
    context(
        "metric_type",
        alt((
            tag(COUNTER).map(|_| MetricType::Counter),
            // try `gaugehistogram` before `gauge`
            tag(GAUGEHISTOGRAM).map(|_| MetricType::Gaugehistogram),
            tag(GAUGE).map(|_| MetricType::Gauge),
            tag(HISTOGRAM).map(|_| MetricType::Histogram),
            tag(STATESET).map(|_| MetricType::Stateset),
            tag(INFO).map(|_| MetricType::Info),
            tag(SUMMARY).map(|_| MetricType::Summary),
            tag(UNKNOWN).map(|_| MetricType::Unknown),
        )),
    )
    .parse(input)
}

#[derive(Clone, Debug, PartialEq)]
pub struct Sample<I> {
    pub metricname: I,
    pub labels: Option<(I, Labels<I>)>,
    pub number: I,
    pub timestamp: Option<I>,
    pub exemplar: Option<(I, Exemplar<I>)>,
}
pub fn sample<I, E>(input: I) -> IResult<I, Sample<I>, E>
where
    I: Compare<&'static str> + Input + Offset,
    I::Item: AsChar,
    E: ContextError<I> + ParseError<I>,
{
    context(
        "sample",
        (
            metricname,
            opt(consumed(labels)),
            char(SP),
            number,
            opt((char(SP), timestamp)),
            opt(consumed(exemplar)),
            char(LF),
        ),
    )
    .map(
        |(metricname, labels, _, number, timestamp, exemplar, _)| Sample {
            metricname,
            labels,
            number,
            timestamp: timestamp.map(|(_, timestamp)| timestamp),
            exemplar,
        },
    )
    .parse(input)
}

#[derive(Clone, Debug, PartialEq)]
pub struct Exemplar<I> {
    pub labels: (I, Labels<I>),
    pub number: I,
    pub timestamp: Option<I>,
}
pub fn exemplar<I, E>(input: I) -> IResult<I, Exemplar<I>, E>
where
    I: Compare<&'static str> + Input + Offset,
    I::Item: AsChar,
    E: ContextError<I> + ParseError<I>,
{
    context(
        "exemplar",
        (
            char(SP),
            char(HASH),
            char(SP),
            consumed(labels),
            char(SP),
            number,
            opt((char(SP), timestamp)),
        ),
    )
    .map(|(_, _, _, labels, _, number, timestamp)| Exemplar {
        labels,
        number,
        timestamp: timestamp.map(|(_, timestamp)| timestamp),
    })
    .parse(input)
}

#[derive(Clone, Debug, PartialEq)]
pub struct Labels<I> {
    pub label: Vec<(I, Label<I>)>,
}
pub fn labels<I, E>(input: I) -> IResult<I, Labels<I>, E>
where
    I: Input + Offset,
    I::Item: AsChar,
    E: ContextError<I> + ParseError<I>,
{
    context(
        "labels",
        (
            char('{'),
            separated_list0(char(COMMA), consumed(label)),
            char('}'),
        ),
    )
    .map(|(_, label, _)| Labels { label })
    .parse(input)
}

#[derive(Clone, Debug, PartialEq)]
pub struct Label<I> {
    pub label_name: I,
    pub escaped_string: (I, EscapedString<I>),
}
pub fn label<I, E>(input: I) -> IResult<I, Label<I>, E>
where
    I: Input + Offset,
    I::Item: AsChar,
    E: ContextError<I> + ParseError<I>,
{
    context(
        "label",
        (
            label_name,
            char(EQ),
            char(DQUOTE),
            consumed(escaped_string),
            char(DQUOTE),
        ),
    )
    .map(|(label_name, _, _, escaped_string, _)| Label {
        label_name,
        escaped_string,
    })
    .parse(input)
}

pub fn number<I, E>(input: I) -> IResult<I, I, E>
where
    I: Compare<&'static str> + Input + Offset,
    I::Item: AsChar,
    E: ContextError<I> + ParseError<I>,
{
    context(
        "number",
        alt((
            realnumber,
            recognize((
                opt(satisfy(is_sign)),
                alt((tag_no_case("inf"), tag_no_case("infinity"))),
            )),
            recognize(tag_no_case("nan")),
        )),
    )
    .parse(input)
}

pub use self::realnumber as timestamp;

pub fn realnumber<I, E>(input: I) -> IResult<I, I, E>
where
    I: Input + Offset,
    I::Item: AsChar,
    E: ContextError<I> + ParseError<I>,
{
    context("realnumber", recognize_float).parse(input)
}

const EOF: &str = "EOF";
const TYPE: &str = "TYPE";
const HELP: &str = "HELP";
const UNIT: &str = "UNIT";

const COUNTER: &str = "counter";
const GAUGE: &str = "gauge";
const HISTOGRAM: &str = "histogram";
const GAUGEHISTOGRAM: &str = "gaugehistogram";
const STATESET: &str = "stateset";
const INFO: &str = "info";
const SUMMARY: &str = "summary";
const UNKNOWN: &str = "unknown";

const BS: char = '\\';
const EQ: char = '=';
const COMMA: char = ',';
const HASH: char = '#';

fn is_sign(c: char) -> bool {
    c == '-' || c == '+'
}

pub fn metricname<I, E>(input: I) -> IResult<I, I, E>
where
    I: Input + Offset,
    I::Item: AsChar,
    E: ContextError<I> + ParseError<I>,
{
    context(
        "metricname",
        recognize((
            satisfy(is_metricname_initial_char),
            fold_many0(satisfy(is_metricname_char), || (), |_, _| ()),
        )),
    )
    .parse(input)
}

fn is_metricname_char(c: char) -> bool {
    is_metricname_initial_char(c) || c.is_ascii_digit()
}

fn is_metricname_initial_char(c: char) -> bool {
    c.is_ascii_alphabetic() || c == '_' || c == ':'
}

pub fn label_name<I, E>(input: I) -> IResult<I, I, E>
where
    I: Input + Offset,
    I::Item: AsChar,
    E: ContextError<I> + ParseError<I>,
{
    context(
        "label_name",
        recognize((
            satisfy(is_label_name_initial_char),
            fold_many0(satisfy(is_label_name_char), || (), |_, _| ()),
        )),
    )
    .parse(input)
}

fn is_label_name_char(c: char) -> bool {
    is_label_name_initial_char(c) || c.is_ascii_digit()
}

fn is_label_name_initial_char(c: char) -> bool {
    c.is_ascii_alphabetic() || c == '_'
}

#[derive(Clone, Debug, PartialEq)]
pub struct EscapedString<I>(pub Vec<(I, EscapedStringFragment<I>)>);
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum EscapedStringFragment<I> {
    Normal(I),
    Lf,
    Dquote,
    Bs,
}
pub fn escaped_string<I, E>(input: I) -> IResult<I, EscapedString<I>, E>
where
    I: Input + Offset,
    I::Item: AsChar,
    E: ContextError<I> + ParseError<I>,
{
    context(
        "escaped_string",
        many0(consumed(alt((
            recognize(fold_many1(
                alt((
                    satisfy(is_normal_char).map(|_| ()),
                    (char(BS), satisfy(|c| is_normal_char(c) && c != 'n')).map(|_| ()),
                )),
                || (),
                |_, _| (),
            ))
            .map(EscapedStringFragment::Normal),
            (char(BS), char('n')).map(|_| EscapedStringFragment::Lf),
            (char(BS), char(DQUOTE)).map(|_| EscapedStringFragment::Dquote),
            (char(BS), char(BS)).map(|_| EscapedStringFragment::Bs),
        )))),
    )
    .map(EscapedString)
    .parse(input)
}

fn is_normal_char(c: char) -> bool {
    c != LF && c != DQUOTE && c != BS
}

#[cfg(test)]
mod tests;
