use insta::assert_yaml_snapshot;
use nom::Parser;
use nom::bytes::complete::{tag, take_until, take_while1};
use nom::character::complete::{alphanumeric1, char};
use nom::character::complete::{multispace1, one_of};
use nom::combinator::{all_consuming, map, opt, recognize};
use nom::multi::{many1, separated_list1};
use nom::sequence::{delimited, preceded};
use nom::{IResult, branch::alt};
use nom_locate::LocatedSpan;
use serde::Serialize;

pub type Span<'a> = LocatedSpan<&'a str>;

#[derive(Debug, PartialEq, Clone, Copy, Serialize)]
pub struct SourcePosition {
    pub line: u32,
    pub offset: usize,
}

#[derive(Debug, PartialEq, Clone, Copy, Serialize)]
pub struct SourceRange {
    pub start: SourcePosition,
    pub end: SourcePosition,
}

pub fn range(span: Span) -> SourceRange {
    let start_offset = span.location_offset();
    let start_line = span.location_line();

    let mut end_offset = start_offset;
    let mut end_line = start_line;

    for ch in span.fragment().chars() {
        end_offset += ch.len_utf8();
        if ch == '\n' {
            end_line += 1;
        }
    }

    SourceRange {
        start: SourcePosition {
            line: start_line,
            offset: start_offset,
        },
        end: SourcePosition {
            line: end_line,
            offset: end_offset,
        },
    }
}

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

    map(
        preceded(tag("@"), take_while1(|c: char| !c.is_whitespace())),
        move |s: Span| FilePathPart {
            range: range.clone(),
            path: s.fragment().to_string(),
        },
    )
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
        map(filepath_part, |p| Part::FilePath(p)),
        map(freeform_part, |p| Part::Freeform(p)),
        map(inline_shell_part, |p| Part::InlineShell(p)),
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
    fn parse_statement_snapshot(#[case] input: &str) {
        let mut s = insta::Settings::clone_current();
        s.set_snapshot_suffix(format!("{}", input.replace(' ', "_")));
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
