use nom::Parser;
use nom::character::char;
use nom::character::complete::{multispace1, one_of};
use nom::combinator::recognize;
use nom::multi::many1;
use nom::{IResult, branch::alt};

fn lowercase_char(input: &str) -> IResult<&str, char> {
    one_of("abcdefghijklmnopqrstuvwxyz").parse(input)
}

fn digit(input: &str) -> IResult<&str, char> {
    one_of("0123456789").parse(input)
}

fn lowercase_or_digit(input: &str) -> IResult<&str, char> {
    alt((lowercase_char, digit)).parse(input)
}

fn dash(input: &str) -> IResult<&str, char> {
    char('-').parse(input)
}

fn lowercase_name_char(input: &str) -> IResult<&str, char> {
    alt((lowercase_or_digit, digit, dash)).parse(input)
}

fn lowercase_name_tail(input: &str) -> IResult<&str, &str> {
    recognize(many1(lowercase_name_char)).parse(input)
}

fn lowercase_name(input: &str) -> IResult<&str, &str> {
    recognize((lowercase_char, lowercase_name_tail)).parse(input)
}

fn vocative(input: &str) -> IResult<&str, &str> {
    lowercase_name.parse(input)
}

fn verb(input: &str) -> IResult<&str, &str> {
    lowercase_name.parse(input)
}

pub fn parse_sentence(input: &str) -> IResult<&str, &str> {
    recognize((vocative, multispace1, verb)).parse(input)
}
