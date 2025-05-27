use crate::parser::{Part, Sentence, Span, parse_statement};

pub fn parse(input: &str) -> Sentence {
    let span = Span::new(input);
    let (_, sentence) = parse_statement(span).expect("could not parse input");

    sentence
}

pub fn extract_description(sentence: &Sentence) -> String {
    let description = sentence
        .parts
        .iter()
        .filter_map(|part| match part {
            Part::Freeform(free) => Some(free.text.as_str()),
            _ => None,
        })
        .collect::<Vec<_>>()
        .join(" ");

    description
}
