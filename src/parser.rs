use nom::Parser;
use nom::character::complete::char;
use nom::character::complete::{multispace1, one_of};
use nom::combinator::{map, recognize};
use nom::multi::many1;
use nom::{IResult, branch::alt};

/// Names the entity you are talking to
#[derive(Debug)]
struct Vocative {
    name: String,
}

/// Verbs are actions/capabilities the entity is expected to perform
#[derive(Debug)]
struct Verb {
    name: String,
}

/// The fully-parsed sentence. Describes a prompt.
#[derive(Debug)]
pub struct Sentence {
    vocative: Vocative,
    verb: Verb,
}

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

fn vocative(input: &str) -> IResult<&str, Vocative> {
    map(lowercase_name, |name| Vocative {
        name: name.to_string(),
    })
    .parse(input)
}

fn verb(input: &str) -> IResult<&str, Verb> {
    map(lowercase_name, |name| Verb {
        name: name.to_string(),
    })
    .parse(input)
}

fn sentence(input: &str) -> IResult<&str, Sentence> {
    map((vocative, multispace1, verb), |(vocative, _, verb)| {
        Sentence { vocative, verb }
    })
    .parse(input)
}

pub fn parse_statement(input: &str) -> IResult<&str, Sentence> {
    sentence.parse(input)
}
