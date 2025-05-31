use std::io::Read;

use crate::{
    ast::{Part, Sentence, Span, parse_statement},
    hir::{AnalysisContext, Analyzable, AnalyzedSentence},
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

pub fn extract_description(sentence: &AnalyzedSentence, environment: &Environment) -> String {
    sentence
        .parts
        .iter()
        .filter_map(|part| match &part.node {
            Part::Freeform(part) => Some(part.text.clone()),
            Part::InlineShell(part) => Some(format_cmd_result(part.code.as_str(), environment)),
            _ => None,
        })
        .collect::<Vec<_>>()
        .join(" ")
}

pub fn build_prompt(result: &AnalyzedSentence) -> String {
    let environment = build_environment();

    let description = extract_description(result, &environment);
    let context = context! {
        description,
    };
    let template = environment
        .get_template(&format!("verbs/{}", &result.verb.node.name))
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
    let hir = ast.analyze(&mut AnalysisContext {});
    let prompt = build_prompt(&hir);
    let attachments = extract_attachments(&ast);

    PromptBuilderResult {
        ast,
        attachments,
        prompt,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use insta::assert_yaml_snapshot;
    use rstest::rstest;

    #[derive(Debug, PartialEq, Serialize)]
    struct SimplifiedPromptBuilderResult {
        attachments: Vec<Attachment>,
        prompt: String,
    }

    #[rstest]
    #[case("qwen3 create bar $(expr 2 + 3)")]
    #[case("qwen3 create $(expr 5 - 3)")]
    #[case("qwen3 create $(echo \"hello\nworld\" | grep world)")]
    fn parse_statement_snapshot(#[case] input: &str) {
        let mut s = insta::Settings::clone_current();
        s.set_snapshot_suffix(input.replace(' ', "_").to_string());
        let _guard = s.bind_to_scope();
        let results = run_prompt_builder(input);
        let simplified_result = SimplifiedPromptBuilderResult {
            attachments: results.attachments,
            prompt: results.prompt,
        };

        assert_yaml_snapshot!(simplified_result);
    }
}
