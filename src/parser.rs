use nom::bytes::complete::tag;
use nom::Parser;
use nom::character::complete::multispace1;
use nom::sequence::delimited;
use nom::{IResult, branch::alt};

fn vocative(input: &str) -> IResult<&str, &str> {
    alt((tag("foo"), tag("bar"))).parse(input)
}


fn verb(input: &str) -> IResult<&str, &str> {
    alt((tag("ipsum"), tag("dolor"))).parse(input)
}

pub fn parse_sentence(input: &str) -> IResult<&str, &str> {
    delimited(
        vocative,
        multispace1,
        verb,
    )
    .parse(input)
}
