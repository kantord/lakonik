use nom::Parser;
use nom::bytes::complete::{tag, take_until};
use nom::character::complete::alphanumeric1;
use nom::character::complete::multispace1;
use nom::combinator::{all_consuming, map, opt};
use nom::multi::separated_list1;
use nom::sequence::{delimited, preceded};
use nom::{IResult, branch::alt};
use serde::Serialize;

use super::primitives::{file_path, lowercase_name};
use super::utils::{SourceRange, Span, range};

/// Names the entity you are talking to
#[derive(Debug, PartialEq, Serialize)]
#[serde(tag = "type", rename = "vocative")]
pub struct Vocative {
    range: SourceRange,
    pub name: String,
}

/// Verbs are actions/capabilities the entity is expected to perform
#[derive(Debug, PartialEq, Serialize)]
#[serde(tag = "type", rename = "verb")]
pub struct Verb {
    range: SourceRange,
    pub name: String,
}

/// Contains free-from text
#[derive(Debug, PartialEq, Serialize)]
#[serde(tag = "type", rename = "freeform")]
pub struct FreeformPart {
    range: SourceRange,
    pub text: String,
}

/// Contains a file path
#[derive(Debug, PartialEq, Serialize)]
#[serde(tag = "type", rename = "filepath")]
pub struct FilePathPart {
    range: SourceRange,
    pub path: String,
}

/// Contains an inline shell script
#[derive(Debug, PartialEq, Serialize)]
#[serde(tag = "type", rename = "inline_shell")]
pub struct InlineShellPart {
    range: SourceRange,
    pub code: String,
}

/// Generic parts that can contain objects or free form text
#[derive(Debug, PartialEq, Serialize)]
pub enum Part {
    Freeform(FreeformPart),
    FilePath(FilePathPart),
    InlineShell(InlineShellPart),
}

/// The fully-parsed sentence. Describes a prompt.
#[derive(Debug, PartialEq, Serialize)]
#[serde(tag = "type", rename = "sentence")]
pub struct Sentence {
    range: SourceRange,
    pub vocative: Vocative,
    pub verb: Verb,
    pub parts: Vec<Part>,
}

fn vocative(input: Span) -> IResult<Span, Vocative> {
    let range = range(input);

    map(lowercase_name, |name| Vocative {
        range,
        name: name.to_string(),
    })
    .parse(input)
}

fn verb(input: Span) -> IResult<Span, Verb> {
    let range = range(input);

    map(lowercase_name, |name| Verb {
        range,
        name: name.to_string(),
    })
    .parse(input)
}

fn freeform_part(input: Span) -> IResult<Span, FreeformPart> {
    let range = range(input);

    map(alphanumeric1, |text: Span| FreeformPart {
        range,
        text: text.to_string(),
    })
    .parse(input)
}

fn filepath_part(input: Span) -> IResult<Span, FilePathPart> {
    let range = range(input);

    map(preceded(tag("@"), file_path), move |s: Span| FilePathPart {
        range,
        path: s.fragment().to_string(),
    })
    .parse(input)
}

fn inline_shell_part(input: Span) -> IResult<Span, InlineShellPart> {
    let range = range(input);

    map(
        delimited(tag("$("), take_until(")"), tag(")")),
        |code: Span| InlineShellPart {
            range,
            code: code.to_string(),
        },
    )
    .parse(input)
}

fn part(input: Span) -> IResult<Span, Part> {
    alt((
        map(filepath_part, Part::FilePath),
        map(freeform_part, Part::Freeform),
        map(inline_shell_part, Part::InlineShell),
    ))
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
    let range = range(input);

    map(
        (vocative, multispace1, verb, maybe_parts),
        |(vocative, _, verb, parts)| Sentence {
            range,
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
    use insta::assert_yaml_snapshot;
    use rstest::rstest;

    #[rstest]
    #[case("john run")]
    #[case("alice42 jump")]
    #[case("my-name jump-fast")]
    #[case("bob123 fly")]
    #[case("john-doe123 run-fast")]
    #[case("john- run")]
    #[case("qwen3 create @hello.txt")]
    #[case("qwen3 edit @hello.txt foo")]
    #[case("qwen3 edit foo @hello.txt bar")]
    #[case("qwen3 delete bar @hello.txt")]
    #[case("qwen3 delete @hello.txt")]
    #[case("qwen3 create $(  tree .)")]
    #[case("qwen3 edit $(ls) foo")]
    #[case("qwen3 edit foo $(find . | grep hello | grep py) bar")]
    #[case("qwen3 delete bar $(git diff)")]
    #[case("qwen3 summarize $(curl https://google.com)")]
    fn parse_statement_snapshot(#[case] input: &str) {
        let mut s = insta::Settings::clone_current();
        s.set_snapshot_suffix(input.replace(' ', "_").to_string());
        let _guard = s.bind_to_scope();
        let (rest, sentence) = parse_statement(Span::new(input)).expect("parser should succeed");
        assert_eq!(*rest.fragment(), "");

        assert_yaml_snapshot!(sentence);
    }

    #[rstest]
    #[case("42run")]
    #[case("john run!")]
    #[case(" run")]
    #[case("")]
    #[case("alice! jump")]
    fn test_parse_statement_failure(#[case] input: &str) {
        assert!(parse_statement(Span::new(input)).is_err());
    }
}
