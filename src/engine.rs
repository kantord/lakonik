use crate::parser::{Sentence, Span, parse_statement};

pub fn parse(input: &str) -> Sentence {
    let span = Span::new(input);
    let (_, sentence) = parse_statement(span).expect("could not parse input");

    sentence
}
