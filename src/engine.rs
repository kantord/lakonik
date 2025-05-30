use std::io::Read;

use crate::{
    ast::{Part, Sentence, Span, parse_statement},
    templates::build_environment,
};
use duct::cmd;
use minijinja::{Environment, context};
use serde::Serialize;

pub fn parse(input: &str) -> Sentence {
    let span = Span::new(input);
    let (_, sentence) = parse_statement(span).expect("could not parse input");

    sentence
}

pub fn format_cmd_result(code: &str, environment: &Environment) -> String {
    let cmd = cmd!("bash", "-c", code);
    let mut reader = cmd
        .stderr_to_stdout()
        .reader()
        .unwrap_or_else(|_| panic!("could not run command: {}", code));
    let mut result = String::new();

    reader
        .read_to_string(&mut result)
        .expect("could not read command output");

    let template = environment.get_template("parts/shell").unwrap();
    let context = context! {
        code,
        result,
    };

    template.render(context).unwrap()
}

pub fn extract_description(sentence: &Sentence, environment: &Environment) -> String {
    

    sentence
        .parts
        .iter()
        .filter_map(|part| match part {
            Part::Freeform(part) => Some(part.text.clone()),
            Part::InlineShell(part) => Some(format_cmd_result(part.code.as_str(), environment)),
            _ => None,
        })
        .collect::<Vec<_>>()
        .join(" ")
}

pub fn build_prompt(result: &Sentence) -> String {
    let environment = build_environment();

    let description = extract_description(result, &environment);
    let context = context! {
        description,
    };
    let template = environment
        .get_template(&format!("verbs/{}", &result.verb.name))
        .unwrap();
    

    template.render(context).unwrap()
}

pub fn extract_attachments(sentence: &Sentence) -> Vec<Attachment> {
    sentence
        .parts
        .iter()
        .filter_map(|p| {
            if let Part::FilePath(file_part) = p {
                Some(Attachment::File(FileAttachment {
                    path: file_part.path.clone(),
                }))
            } else {
                None
            }
        })
        .collect()
}

#[derive(Debug, PartialEq, Serialize)]
pub struct FileAttachment {
    pub path: String,
}

#[derive(Debug, PartialEq, Serialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum Attachment {
    File(FileAttachment),
}

#[derive(Debug, PartialEq, Serialize)]
pub struct PromptBuilderResult {
    pub ast: Sentence,
    pub attachments: Vec<Attachment>,
    pub prompt: String,
}

pub fn run_prompt_builder(raw_input: &str) -> PromptBuilderResult {
    let ast = parse(raw_input);
    let prompt = build_prompt(&ast);
    let attachments = extract_attachments(&ast);

    PromptBuilderResult {
        ast,
        attachments,
        prompt,
    }
}
