use crate::{
    parser::{Part, Sentence, Span, parse_statement},
    verbs::build_environment,
};
use minijinja::context;
use serde::Serialize;

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

pub fn build_prompt(result: &Sentence) -> String {
    let description = extract_description(&result);

    let environment = build_environment();
    let context = context! {
        description,
    };
    let template = environment
        .get_template(&format!("verbs/{}", &result.verb.name))
        .unwrap();
    let prompt = template.render(context).unwrap();

    prompt
}

#[derive(Debug, PartialEq, Serialize)]
pub struct PromptBuilderResult {
    pub ast: Sentence,
    pub prompt: String,
}

pub fn run_prompt_builder(raw_input: &str) -> PromptBuilderResult {
    let ast = parse(raw_input);
    let prompt = build_prompt(&ast);

    return PromptBuilderResult { ast, prompt };
}
