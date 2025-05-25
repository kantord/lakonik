use nom::Parser;
use nom::character::complete::{alphanumeric1, char};
use nom::character::complete::{multispace1, one_of};
use nom::combinator::{all_consuming, map, opt, recognize};
use nom::multi::{many1, separated_list1};
use nom::sequence::preceded;
use nom::{IResult, branch::alt};
use nom_locate::{LocatedSpan, position};

pub type Span<'a> = LocatedSpan<&'a str>;

/// Names the entity you are talking to
#[derive(Debug, PartialEq)]
pub struct Vocative<'a> {
    position: Span<'a>,
    pub name: String,
}

/// Verbs are actions/capabilities the entity is expected to perform
#[derive(Debug, PartialEq)]
pub struct Verb<'a> {
    position: Span<'a>,
    pub name: String,
}

/// Generic parts that can contain objects or free form text
#[derive(Debug, PartialEq)]
pub struct Part<'a> {
    position: Span<'a>,
    pub text: String,
}

/// The fully-parsed sentence. Describes a prompt.
#[derive(Debug, PartialEq)]
pub struct Sentence<'a> {
    position: Span<'a>,
    pub vocative: Vocative<'a>,
    pub verb: Verb<'a>,
    pub parts: Vec<Part<'a>>,
}

fn lowercase_char(input: Span) -> IResult<Span, char> {
    one_of("abcdefghijklmnopqrstuvwxyz").parse(input)
}

fn digit(input: Span) -> IResult<Span, char> {
    one_of("0123456789").parse(input)
}

fn lowercase_or_digit(input: Span) -> IResult<Span, char> {
    alt((lowercase_char, digit)).parse(input)
}

fn dash(input: Span) -> IResult<Span, char> {
    char('-').parse(input)
}

fn lowercase_name_char(input: Span) -> IResult<Span, char> {
    alt((lowercase_or_digit, digit, dash)).parse(input)
}

fn lowercase_name_tail(input: Span) -> IResult<Span, Span> {
    recognize(many1(lowercase_name_char)).parse(input)
}

fn lowercase_name(input: Span) -> IResult<Span, Span> {
    recognize((lowercase_char, lowercase_name_tail)).parse(input)
}

fn vocative(input: Span) -> IResult<Span, Vocative> {
    let (_, position) = position(input)?;

    map(lowercase_name, |name| Vocative {
        position,
        name: name.to_string(),
    })
    .parse(input)
}

fn verb(input: Span) -> IResult<Span, Verb> {
    let (_, position) = position(input)?;

    map(lowercase_name, |name| Verb {
        position,
        name: name.to_string(),
    })
    .parse(input)
}

fn part(input: Span) -> IResult<Span, Part> {
    let (_, position) = position(input)?;

    map(alphanumeric1, |text: Span| Part {
        position,
        text: text.to_string(),
    })
    .parse(input)
}

fn parts(input: Span) -> IResult<Span, Vec<Part>> {
    separated_list1(multispace1, part).parse(input)
}

fn maybe_parts(input: Span) -> IResult<Span, Vec<Part>> {
    map(opt(preceded(multispace1, parts)), |opt_vec| {
        opt_vec.unwrap_or_default()
    })
    .parse(input)
}

fn sentence(input: Span) -> IResult<Span, Sentence> {
    let (_, position) = position(input)?;

    map(
        (vocative, multispace1, verb, maybe_parts),
        |(vocative, _, verb, parts)| Sentence {
            position,
            vocative,
            verb,
            parts,
        },
    )
    .parse(input)
}

pub fn parse_statement(input: Span) -> IResult<Span, Sentence> {
    all_consuming(sentence).parse(input)
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;

    #[rstest]
    #[case("john run", "john", "run")]
    #[case("alice42 jump", "alice42", "jump")]
    #[case("my-name jump-fast", "my-name", "jump-fast")]
    #[case("bob123 fly", "bob123", "fly")]
    #[case("john-doe123 run-fast", "john-doe123", "run-fast")]
    #[case("john- run", "john-", "run")]
    fn test_parse_statement_success(
        #[case] input: &str,
        #[case] vocative: &str,
        #[case] verb: &str,
    ) {
        let expected = Sentence {
            vocative: Vocative {
                name: vocative.to_string(),
            },
            verb: Verb {
                name: verb.to_string(),
            },
            parts: vec![],
        };

        assert_eq!(parse_statement(input), Ok(("", expected)));
    }

    #[rstest]
    #[case("42run")]
    #[case("john run!")]
    #[case(" run")]
    #[case("")]
    #[case("alice! jump")]
    fn test_parse_statement_failure(#[case] input: &str) {
        assert!(parse_statement(input).is_err());
    }
}
