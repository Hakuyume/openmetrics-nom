use nom::branch::alt;
use nom::bytes::complete::{tag, tag_no_case};
use nom::character::complete::{char, digit0, digit1, satisfy};
use nom::combinator::{consumed, opt, recognize};
use nom::error::ParseError;
use nom::multi::{many0, many1, separated_list0};
use nom::sequence::tuple;
use nom::{
    AsChar, Compare, IResult, InputIter, InputLength, InputTake, InputTakeAtPosition, Offset,
    Parser, Slice,
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

// RFC 5234 B.1.
pub const DQUOTE: char = '"';
pub const SP: char = ' ';
pub const LF: char = '\n';

// https://github.com/prometheus/OpenMetrics/blob/main/specification/OpenMetrics.md#abnf

pub struct Exposition<I> {
    pub consumed: I,
    pub metricset: Metricset<I>,
}
pub fn exposition<I, E>(input: I) -> IResult<I, Exposition<I>, E>
where
    I: Input,
    E: ParseError<I>,
{
    consumed(tuple((
        metricset,
        char(HASH),
        char(SP),
        tag(EOF),
        opt(char(LF)),
    )))
    .map(|(consumed, (metricset, _, _, _, _))| Exposition {
        consumed,
        metricset,
    })
    .parse(input)
}

#[derive(Clone, Debug, PartialEq)]
pub struct Metricset<I> {
    pub consumed: I,
    pub metricfamily: Vec<Metricfamily<I>>,
}
pub fn metricset<I, E>(input: I) -> IResult<I, Metricset<I>, E>
where
    I: Input,
    E: ParseError<I>,
{
    consumed(many0(metricfamily))
        .map(|(consumed, metricfamily)| Metricset {
            consumed,
            metricfamily,
        })
        .parse(input)
}

#[derive(Clone, Debug, PartialEq)]
pub struct Metricfamily<I> {
    pub consumed: I,
    pub metric_descriptor: Vec<MetricDescriptor<I>>,
    pub metric: Vec<Sample<I>>,
}
pub fn metricfamily<I, E>(input: I) -> IResult<I, Metricfamily<I>, E>
where
    I: Input,
    E: ParseError<I>,
{
    consumed(alt((
        tuple((many1(metric_descriptor), many0(sample))),
        tuple((many0(metric_descriptor), many1(sample))),
    )))
    .map(|(consumed, (metric_descriptor, metric))| Metricfamily {
        consumed,
        metric_descriptor,
        metric,
    })
    .parse(input)
}

#[derive(Clone, Debug, PartialEq)]
pub enum MetricDescriptor<I> {
    Type {
        consumed: I,
        metricname: I,
        metric_type: MetricType,
    },
    Help {
        consumed: I,
        metricname: I,
        escaped_string: I,
    },
    Unit {
        consumed: I,
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
        consumed(tuple((
            char(HASH),
            char(SP),
            tag(TYPE),
            char(SP),
            metricname,
            char(SP),
            metric_type,
            char(LF),
        )))
        .map(|(consumed, (_, _, _, _, metricname, _, metric_type, _))| {
            MetricDescriptor::Type {
                consumed,
                metricname,
                metric_type,
            }
        }),
        consumed(tuple((
            char(HASH),
            char(SP),
            tag(HELP),
            char(SP),
            metricname,
            char(SP),
            escaped_string,
            char(LF),
        )))
        .map(
            |(consumed, (_, _, _, _, metricname, _, escaped_string, _))| MetricDescriptor::Help {
                consumed,
                metricname,
                escaped_string,
            },
        ),
        consumed(tuple((
            char(HASH),
            char(SP),
            tag(UNIT),
            char(SP),
            metricname,
            char(SP),
            recognize(many0(metricname_char)),
            char(LF),
        )))
        .map(
            |(consumed, (_, _, _, _, metricname, _, metricname_char, _))| MetricDescriptor::Unit {
                consumed,
                metricname,
                metricname_char,
            },
        ),
    ))
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
        tag(GAUGE).map(|_| MetricType::Gauge),
        tag(HISTOGRAM).map(|_| MetricType::Histogram),
        tag(GAUGEHISTOGRAM).map(|_| MetricType::Gaugehistogram),
        tag(STATESET).map(|_| MetricType::Stateset),
        tag(INFO).map(|_| MetricType::Info),
        tag(SUMMARY).map(|_| MetricType::Summary),
        tag(UNKNOWN).map(|_| MetricType::Unknown),
    ))
    .parse(input)
}

#[derive(Clone, Debug, PartialEq)]
pub struct Sample<I> {
    pub consumed: I,
    pub metricname: I,
    pub labels: Option<Labels<I>>,
    pub number: Number<I>,
    pub timestamp: Option<I>,
    pub exemplar: Option<Exemplar<I>>,
}
pub fn sample<I, E>(input: I) -> IResult<I, Sample<I>, E>
where
    I: Input,
    E: ParseError<I>,
{
    consumed(tuple((
        metricname,
        opt(labels),
        char(SP),
        number,
        opt(tuple((char(SP), timestamp))),
        opt(exemplar),
        char(LF),
    )))
    .map(
        |(consumed, (metricname, labels, _, number, timestamp, exemplar, _))| Sample {
            consumed,
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
    pub consumed: I,
    pub labels: Labels<I>,
    pub number: Number<I>,
    pub timestamp: Option<I>,
}
pub fn exemplar<I, E>(input: I) -> IResult<I, Exemplar<I>, E>
where
    I: Input,
    E: ParseError<I>,
{
    consumed(tuple((
        char(SP),
        char(HASH),
        char(SP),
        labels,
        char(SP),
        number,
        opt(tuple((char(SP), timestamp))),
    )))
    .map(
        |(consumed, (_, _, _, labels, _, number, timestamp))| Exemplar {
            consumed,
            labels,
            number,
            timestamp: timestamp.map(|(_, timestamp)| timestamp),
        },
    )
    .parse(input)
}

#[derive(Clone, Debug, PartialEq)]
pub struct Labels<I> {
    pub consumed: I,
    pub labels: Vec<Label<I>>,
}
pub fn labels<I, E>(input: I) -> IResult<I, Labels<I>, E>
where
    I: Input,
    E: ParseError<I>,
{
    consumed(tuple((
        char('{'),
        separated_list0(char(COMMA), label),
        char('}'),
    )))
    .map(|(consumed, (_, labels, _))| Labels { consumed, labels })
    .parse(input)
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Label<I> {
    pub consumed: I,
    pub label_name: I,
    pub escaped_string: I,
}
pub fn label<I, E>(input: I) -> IResult<I, Label<I>, E>
where
    I: Input,
    E: ParseError<I>,
{
    consumed(tuple((
        label_name,
        char(EQ),
        char(DQUOTE),
        escaped_string,
        char(DQUOTE),
    )))
    .map(|(consumed, (label_name, _, _, escaped_string, _))| Label {
        consumed,
        label_name,
        escaped_string,
    })
    .parse(input)
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Number<I> {
    Real(I),
    Inf(I),
    Nan(I),
}
pub fn number<I, E>(input: I) -> IResult<I, Number<I>, E>
where
    I: Input,
    E: ParseError<I>,
{
    alt((
        realnumber.map(Number::Real),
        recognize(tuple((
            opt(sign),
            alt((tag_no_case("inf"), tag_no_case("infinity"))),
        )))
        .map(Number::Inf),
        tag_no_case("nan").map(Number::Nan),
    ))
    .parse(input)
}

pub use self::realnumber as timestamp;

pub fn realnumber<I, E>(input: I) -> IResult<I, I, E>
where
    I: Input,
    E: ParseError<I>,
{
    let exp = || tuple((alt((char('e'), char('E'))), opt(sign), digit1));
    alt((
        recognize(tuple((
            opt(sign),
            digit1,
            opt(tuple((char('.'), digit0))),
            opt(exp()),
        ))),
        recognize(tuple((opt(sign), char('.'), digit1, opt(exp())))),
    ))
    .parse(input)
}

pub const EOF: &str = "EOF";
pub const TYPE: &str = "TYPE";
pub const HELP: &str = "HELP";
pub const UNIT: &str = "UNIT";

pub const COUNTER: &str = "counter";
pub const GAUGE: &str = "gauge";
pub const HISTOGRAM: &str = "histogram";
pub const GAUGEHISTOGRAM: &str = "gaugehistogram";
pub const STATESET: &str = "stateset";
pub const INFO: &str = "info";
pub const SUMMARY: &str = "summary";
pub const UNKNOWN: &str = "unknown";

pub const BS: char = '\\';
pub const EQ: char = '=';
pub const COMMA: char = ',';
pub const HASH: char = '#';

pub fn sign<I, E>(input: I) -> IResult<I, char, E>
where
    I: Input,
    E: ParseError<I>,
{
    alt((char('-'), char('+'))).parse(input)
}

pub fn metricname<I, E>(input: I) -> IResult<I, I, E>
where
    I: Input,
    E: ParseError<I>,
{
    recognize(tuple((metricname_initial_char, many0(metricname_char)))).parse(input)
}

pub fn metricname_char<I, E>(input: I) -> IResult<I, char, E>
where
    I: Input,
    E: ParseError<I>,
{
    alt((metricname_initial_char, satisfy(|c| c.is_ascii_digit()))).parse(input)
}

pub fn metricname_initial_char<I, E>(input: I) -> IResult<I, char, E>
where
    I: Input,
    E: ParseError<I>,
{
    alt((satisfy(|c| c.is_ascii_alphabetic()), char('_'), char(':'))).parse(input)
}

pub fn label_name<I, E>(input: I) -> IResult<I, I, E>
where
    I: Input,
    E: ParseError<I>,
{
    recognize(tuple((label_name_initial_char, many0(label_name_char)))).parse(input)
}

pub fn label_name_char<I, E>(input: I) -> IResult<I, char, E>
where
    I: Input,
    E: ParseError<I>,
{
    alt((label_name_initial_char, satisfy(|c| c.is_ascii_digit()))).parse(input)
}

pub fn label_name_initial_char<I, E>(input: I) -> IResult<I, char, E>
where
    I: Input,
    E: ParseError<I>,
{
    alt((satisfy(|c| c.is_ascii_alphabetic()), char('_'))).parse(input)
}

pub fn escaped_string<I, E>(input: I) -> IResult<I, I, E>
where
    I: Input,
    E: ParseError<I>,
{
    let normal_char = || satisfy(|c| c != LF && c != DQUOTE && c != BS);
    let escaped_char = alt((
        recognize(normal_char()),
        recognize(tuple((char(BS), alt((char('n'), char(DQUOTE), char(BS)))))),
        recognize(tuple((char(BS), normal_char()))),
    ));
    recognize(many0(escaped_char)).parse(input)
}

#[cfg(test)]
mod tests;
