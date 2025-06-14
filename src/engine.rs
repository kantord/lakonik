use std::io::Read;

use crate::{
    ast::{Part, Sentence, Span, parse_statement},
    hir::{
        part::AnalyzedPart,
        sentence::AnalyzedSentence,
        utils::{AnalysisContext, Analyzable},
    },
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
        .unwrap_or_else(|_| panic!("could not run command: {code}"));
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
        .filter_map(|part| match &part {
            AnalyzedPart::Freeform(part) => Some(part.node.text.clone()),
            AnalyzedPart::InlineShell(part) => {
                Some(format_cmd_result(part.node.code.as_str(), environment))
            }
            _ => None,
        })
        .collect::<Vec<_>>()
        .join(" ")
}

pub fn build_prompt(result: &AnalyzedSentence) -> String {
    result.verb.ensure_template();
    let environment = build_environment();

    let description = extract_description(result, &environment);
    let context = context! {
        description,
    };
    let template = environment
        .get_template(&result.verb.template_name)
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
    #[case("robot ~testverbdeleteme1=(test template delete me: ) $(expr 5 - 3)")]
    #[case("robot ~testverbdeleteme2 = (hello)")]
    fn parse_statement_snapshot(#[case] input: &str) {
        let tmp = tempfile::tempdir().unwrap();
        unsafe {
            std::env::set_var("LAKONIK_CONFIG", tmp.path());
        }
        let user_templates = crate::templates::get_user_templates();
        let test_templates = user_templates
            .filter(|t| t.path.starts_with("verbs/testverbdeleteme"))
            .collect::<Vec<_>>();
        for template in test_templates {
            crate::templates::delete_user_template(&template.path);
        }

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
