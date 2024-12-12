use nom::error::{ContextError, ParseError};
use nom::{AsChar, Compare, InputIter, InputLength, InputTake, InputTakeAtPosition, Offset, Slice};
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

pub trait Error<I>: ContextError<I> + ParseError<I> {}
impl<I, E> Error<I> for E where E: ContextError<I> + ParseError<I> {}
