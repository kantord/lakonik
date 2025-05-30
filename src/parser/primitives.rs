use nom::Parser;
use nom::character::complete::char;
use nom::character::complete::one_of;
use nom::combinator::recognize;
use nom::multi::many1;
use nom::{IResult, branch::alt};

use super::utils::Span;

pub fn lowercase_char(input: Span) -> IResult<Span, char> {
    one_of("abcdefghijklmnopqrstuvwxyz").parse(input)
}

pub fn digit(input: Span) -> IResult<Span, char> {
    one_of("0123456789").parse(input)
}

pub fn lowercase_or_digit(input: Span) -> IResult<Span, char> {
    alt((lowercase_char, digit)).parse(input)
}

pub fn dash(input: Span) -> IResult<Span, char> {
    char('-').parse(input)
}

pub fn lowercase_name_char(input: Span) -> IResult<Span, char> {
    alt((lowercase_or_digit, digit, dash)).parse(input)
}

pub fn lowercase_name_tail(input: Span) -> IResult<Span, Span> {
    recognize(many1(lowercase_name_char)).parse(input)
}

pub fn lowercase_name(input: Span) -> IResult<Span, Span> {
    recognize((lowercase_char, lowercase_name_tail)).parse(input)
}
