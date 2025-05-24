use nom::Parser;
use nom::character::complete::{alphanumeric1, char};
use nom::character::complete::{multispace1, one_of};
use nom::combinator::{all_consuming, map, opt, recognize};
use nom::multi::{many1, separated_list1};
use nom::sequence::preceded;
use nom::{IResult, branch::alt};

/// Names the entity you are talking to
#[derive(Debug, PartialEq)]
pub struct Vocative {
    pub name: String,
}

/// Verbs are actions/capabilities the entity is expected to perform
#[derive(Debug, PartialEq)]
pub struct Verb {
    pub name: String,
}

/// Generic parts that can contain objects or free form text
#[derive(Debug, PartialEq)]
pub struct Part {
    pub text: String,
}

/// The fully-parsed sentence. Describes a prompt.
#[derive(Debug, PartialEq)]
pub struct Sentence {
    pub vocative: Vocative,
    pub verb: Verb,
    pub parts: Vec<Part>,
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

fn part(input: &str) -> IResult<&str, Part> {
    map(alphanumeric1, |text: &str| Part {
        text: text.to_string(),
    })
    .parse(input)
}

fn parts(input: &str) -> IResult<&str, Vec<Part>> {
    separated_list1(multispace1, part).parse(input)
}

fn maybe_parts(input: &str) -> IResult<&str, Vec<Part>> {
    map(opt(preceded(multispace1, parts)), |opt_vec| {
        opt_vec.unwrap_or_default()
    })
    .parse(input)
}

fn sentence(input: &str) -> IResult<&str, Sentence> {
    map(
        (vocative, multispace1, verb, maybe_parts),
        |(vocative, _, verb, parts)| Sentence {
            vocative,
            verb,
            parts,
        },
    )
    .parse(input)
}

pub fn parse_statement(input: &str) -> IResult<&str, Sentence> {
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
