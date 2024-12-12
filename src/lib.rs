use nom::branch::alt;
use nom::bytes::complete::{tag, tag_no_case, take_while};
use nom::character::complete::{char, satisfy};
use nom::combinator::{consumed, opt, recognize};
use nom::error::ParseError;
use nom::multi::{fold_many0, fold_many1, many0, many1, separated_list0};
use nom::number::complete::recognize_float;
use nom::sequence::tuple;
use nom::{AsChar, IResult, InputTakeAtPosition, Parser};
use private::Input;

// RFC 5234 B.1.
pub const DQUOTE: char = '"';
pub const SP: char = ' ';
pub const LF: char = '\n';

// https://github.com/prometheus/OpenMetrics/blob/main/specification/OpenMetrics.md#abnf

#[derive(Clone, Debug, PartialEq)]
pub struct Exposition<I> {
    pub metricset: (I, Metricset<I>),
}
pub fn exposition<I, E>(input: I) -> IResult<I, Exposition<I>, E>
where
    I: Input,
    E: ParseError<I>,
{
    tuple((
        consumed(metricset),
        char(HASH),
        char(SP),
        tag(EOF),
        opt(char(LF)),
    ))
    .map(|(metricset, _, _, _, _)| Exposition { metricset })
    .parse(input)
}

#[derive(Clone, Debug, PartialEq)]
pub struct Metricset<I> {
    pub metricfamily: Vec<(I, Metricfamily<I>)>,
}
pub fn metricset<I, E>(input: I) -> IResult<I, Metricset<I>, E>
where
    I: Input,
    E: ParseError<I>,
{
    many0(consumed(metricfamily))
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
    I: Input,
    E: ParseError<I>,
{
    alt((
        tuple((many1(consumed(metric_descriptor)), many0(consumed(metric)))),
        tuple((many0(consumed(metric_descriptor)), many1(consumed(metric)))),
    ))
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
        help_escaped_string: (I, HelpEscapedString<I>),
    },
    Unit {
        metricname: I,
        metricname_char: I,
    },
}
pub fn metric_descriptor<I, E>(input: I) -> IResult<I, MetricDescriptor<I>, E>
where
    I: Input,
    E: ParseError<I>,
{
    alt((
        tuple((
            char(HASH),
            char(SP),
            tag(TYPE),
            char(SP),
            metricname,
            char(SP),
            consumed(metric_type),
            char(LF),
        ))
        .map(
            |(_, _, _, _, metricname, _, metric_type, _)| MetricDescriptor::Type {
                metricname,
                metric_type,
            },
        ),
        tuple((
            char(HASH),
            char(SP),
            tag(HELP),
            char(SP),
            metricname,
            char(SP),
            consumed(help_escaped_string),
            char(LF),
        ))
        .map(|(_, _, _, _, metricname, _, help_escaped_string, _)| {
            MetricDescriptor::Help {
                metricname,
                help_escaped_string,
            }
        }),
        tuple((
            char(HASH),
            char(SP),
            tag(UNIT),
            char(SP),
            metricname,
            char(SP),
            take_while(|c: <I as InputTakeAtPosition>::Item| is_metricname_char(c.as_char())),
            char(LF),
        ))
        .map(
            |(_, _, _, _, metricname, _, metricname_char, _)| MetricDescriptor::Unit {
                metricname,
                metricname_char,
            },
        ),
    ))
    .parse(input)
}

#[derive(Clone, Debug, PartialEq)]
pub struct Metric<I> {
    pub sample: [(I, Sample<I>); 1],
}
pub fn metric<I, E>(input: I) -> IResult<I, Metric<I>, E>
where
    I: Input,
    E: ParseError<I>,
{
    consumed(sample)
        .map(|sample| Metric { sample: [sample] })
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
    I: Input,
    E: ParseError<I>,
{
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
    ))
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
    I: Input,
    E: ParseError<I>,
{
    tuple((
        metricname,
        opt(consumed(labels)),
        char(SP),
        number,
        opt(tuple((char(SP), timestamp))),
        opt(consumed(exemplar)),
        char(LF),
    ))
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
    I: Input,
    E: ParseError<I>,
{
    tuple((
        char(SP),
        char(HASH),
        char(SP),
        consumed(labels),
        char(SP),
        number,
        opt(tuple((char(SP), timestamp))),
    ))
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
    I: Input,
    E: ParseError<I>,
{
    tuple((
        char('{'),
        separated_list0(char(COMMA), consumed(label)),
        char('}'),
    ))
    .map(|(_, label, _)| Labels { label })
    .parse(input)
}

#[derive(Clone, Debug, PartialEq)]
pub struct Label<I> {
    pub label_name: I,
    pub help_escaped_string: (I, EscapedString<I>),
}
pub fn label<I, E>(input: I) -> IResult<I, Label<I>, E>
where
    I: Input,
    E: ParseError<I>,
{
    tuple((
        label_name,
        char(EQ),
        char(DQUOTE),
        consumed(escaped_string),
        char(DQUOTE),
    ))
    .map(|(label_name, _, _, help_escaped_string, _)| Label {
        label_name,
        help_escaped_string,
    })
    .parse(input)
}

pub fn number<I, E>(input: I) -> IResult<I, I, E>
where
    I: Input,
    E: ParseError<I>,
{
    alt((
        realnumber,
        recognize(tuple((
            opt(satisfy(is_sign)),
            alt((tag_no_case("inf"), tag_no_case("infinity"))),
        ))),
        recognize(tag_no_case("nan")),
    ))
    .parse(input)
}

pub use self::realnumber as timestamp;

pub fn realnumber<I, E>(input: I) -> IResult<I, I, E>
where
    I: Input,
    E: ParseError<I>,
{
    recognize_float.parse(input)
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
    I: Input,
    E: ParseError<I>,
{
    recognize(tuple((
        satisfy(is_metricname_initial_char),
        fold_many0(satisfy(is_metricname_char), || (), |_, _| ()),
    )))
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
    I: Input,
    E: ParseError<I>,
{
    recognize(tuple((
        satisfy(is_label_name_initial_char),
        fold_many0(satisfy(is_label_name_char), || (), |_, _| ()),
    )))
    .parse(input)
}

fn is_label_name_char(c: char) -> bool {
    is_label_name_initial_char(c) || c.is_ascii_digit()
}

fn is_label_name_initial_char(c: char) -> bool {
    c.is_ascii_alphabetic()
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
    I: Input,
    E: ParseError<I>,
{
    many0(consumed(alt((
        recognize(fold_many1(
            alt((
                satisfy(is_normal_char).map(|_| ()),
                tuple((char(BS), satisfy(|c| is_normal_char(c) && c != 'n'))).map(|_| ()),
            )),
            || (),
            |_, _| (),
        ))
        .map(EscapedStringFragment::Normal),
        tuple((char(BS), char('n'))).map(|_| EscapedStringFragment::Lf),
        tuple((char(BS), char(DQUOTE))).map(|_| EscapedStringFragment::Dquote),
        tuple((char(BS), char(BS))).map(|_| EscapedStringFragment::Bs),
    ))))
    .map(EscapedString)
    .parse(input)
}

fn is_normal_char(c: char) -> bool {
    c != LF && c != DQUOTE && c != BS
}

// https://github.com/prometheus/client_python/blob/92b23970f032cbc990aa0e501708c425708e51ea/prometheus_client/parser.py#L32-L41
#[derive(Clone, Debug, PartialEq)]
pub struct HelpEscapedString<I>(pub Vec<(I, HelpEscapedStringFragment<I>)>);
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum HelpEscapedStringFragment<I> {
    Normal(I),
    Lf,
    Dquote,
    Bs,
}
pub fn help_escaped_string<I, E>(input: I) -> IResult<I, HelpEscapedString<I>, E>
where
    I: Input,
    E: ParseError<I>,
{
    many0(consumed(alt((
        recognize(fold_many1(
            alt((
                satisfy(is_help_normal_char).map(|_| ()),
                tuple((char(BS), satisfy(|c| is_help_normal_char(c) && c != 'n'))).map(|_| ()),
            )),
            || (),
            |_, _| (),
        ))
        .map(HelpEscapedStringFragment::Normal),
        tuple((char(BS), char('n'))).map(|_| HelpEscapedStringFragment::Lf),
        tuple((char(BS), char(BS))).map(|_| HelpEscapedStringFragment::Bs),
    ))))
    .map(HelpEscapedString)
    .parse(input)
}

fn is_help_normal_char(c: char) -> bool {
    c != LF && c != BS
}

mod private {
    use nom::{
        AsChar, Compare, InputIter, InputLength, InputTake, InputTakeAtPosition, Offset, Slice,
    };
    use std::ops::{RangeFrom, RangeTo};

    pub trait Input:
        Clone
        + Compare<&'static str>
        + InputIter<Item: AsChar>
        + InputLength
        + InputTake
        + InputTakeAtPosition<Item: AsChar>
        + Offset
        + Slice<RangeFrom<usize>>
        + Slice<RangeTo<usize>>
    {
    }
    impl<I> Input for I where
        I: Clone
            + Compare<&'static str>
            + InputIter<Item: AsChar>
            + InputLength
            + InputTake
            + InputTakeAtPosition<Item: AsChar>
            + Offset
            + Slice<RangeFrom<usize>>
            + Slice<RangeTo<usize>>
    {
    }
}

#[cfg(test)]
mod tests;
