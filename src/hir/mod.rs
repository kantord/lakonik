#![allow(dead_code)]
use lsp_types::Range;

use crate::{
    ast::{FilePathPart, FreeformPart, InlineShellPart, Part, Sentence, Verb, Vocative},
    templates::{get_built_in_templates, get_user_templates},
};

pub struct AnalysisContext {}

pub trait Analyzable {
    type AnalyzedNode;

    fn analyze(&self, ctx: &mut AnalysisContext) -> Self::AnalyzedNode;
}

pub trait Analyzed {
    fn get_range(&self) -> &Range;
}

#[derive(Clone, Debug)]
pub struct AnalyzedVocative {
    pub node: Vocative,
    pub hover_text: String,
}

#[derive(Clone, Debug)]
pub struct AnalyzedFreeformPart {
    pub node: FreeformPart,
    pub hover_text: String,
}

#[derive(Clone, Debug)]
pub struct AnalyzedFilePathPart {
    pub node: FilePathPart,
    pub hover_text: String,
}

#[derive(Clone, Debug)]
pub struct AnalyzedInlineShellPart {
    pub node: InlineShellPart,
    pub hover_text: String,
}

#[derive(Clone, Debug)]
pub enum AnalyzedPart {
    Freeform(AnalyzedFreeformPart),
    FilePath(AnalyzedFilePathPart),
    InlineShell(AnalyzedInlineShellPart),
}

#[derive(Debug, Clone)]
pub struct AnalyzedVerb {
    pub node: Verb,
    pub template_name: String,
    pub hover_text: String,
}

#[derive(Clone, Debug)]
pub struct AnalyzedSentence {
    pub node: Sentence,
    pub hover_text: String,
    pub vocative: AnalyzedVocative,
    pub verb: AnalyzedVerb,
    pub parts: Vec<AnalyzedPart>,
}

impl Analyzable for Vocative {
    type AnalyzedNode = AnalyzedVocative;

    fn analyze(&self, _ctx: &mut AnalysisContext) -> Self::AnalyzedNode {
        AnalyzedVocative {
            node: self.clone(),
            hover_text: format!("Hover text for vocative: {}", self.name),
        }
    }
}

impl Analyzable for Part {
    type AnalyzedNode = AnalyzedPart;

    fn analyze(&self, _ctx: &mut AnalysisContext) -> Self::AnalyzedNode {
        match self {
            Part::Freeform(part) => AnalyzedPart::Freeform(AnalyzedFreeformPart {
                node: part.clone(),
                hover_text: "This is a part".to_string(),
            }),
            Part::FilePath(part) => AnalyzedPart::FilePath(AnalyzedFilePathPart {
                node: part.clone(),
                hover_text: "This is a file path part".to_string(),
            }),
            Part::InlineShell(part) => AnalyzedPart::InlineShell(AnalyzedInlineShellPart {
                node: part.clone(),
                hover_text: format!("Will expand to the results of `{}`", { part.code.clone() })
                    .to_string(),
            }),
        }
    }
}

impl Analyzable for Verb {
    type AnalyzedNode = AnalyzedVerb;

    fn analyze(&self, _ctx: &mut AnalysisContext) -> Self::AnalyzedNode {
        let template_name = match self {
            Verb::Simple(node) => node.name.clone(),
            Verb::Assignment(node) => node.name.clone(),
        };
        let template_name = format!("verbs/{}", template_name);
        let mut template_source = get_built_in_templates()
            .find(|t| t.path == template_name)
            .map(|t| t.contents.clone())
            .unwrap_or_else(|| "*N/A*".to_string());

        if let Verb::Assignment(node) = self {
            template_source = node.value.clone();
        }

        let hover_text = format!(
            "_Verb_ **{}**\n\n```\n{}\n```",
            template_name, template_source
        );

        AnalyzedVerb {
            node: self.clone(),
            template_name,
            hover_text,
        }
    }
}

impl Analyzable for Sentence {
    type AnalyzedNode = AnalyzedSentence;

    fn analyze(&self, _ctx: &mut AnalysisContext) -> Self::AnalyzedNode {
        AnalyzedSentence {
            node: self.clone(),
            hover_text: "This is a part".to_string(),
            verb: self.verb.analyze(_ctx),
            vocative: self.vocative.analyze(_ctx),
            parts: self.parts.iter().map(|part| part.analyze(_ctx)).collect(),
        }
    }
}

impl AnalyzedVerb {
    /// Creates a template if it doeds not exist but can be created
    pub fn ensure_template(&self) {
        match &self.node {
            Verb::Simple(_) => (),
            Verb::Assignment(node) => {
                let template_name = format!("verbs/{}", node.name);

                if !get_user_templates().any(|t| t.path == template_name) {
                    crate::templates::create_user_template(&template_name, &node.value);
                }
            }
        }
    }
}

impl Analyzed for AnalyzedVocative {
    fn get_range(&self) -> &Range {
        &self.node.range
    }
}

impl Analyzed for AnalyzedVerb {
    fn get_range(&self) -> &Range {
        match &self.node {
            Verb::Simple(node) => &node.range,
            Verb::Assignment(node) => &node.range,
        }
    }
}

impl Analyzed for AnalyzedPart {
    fn get_range(&self) -> &Range {
        match self {
            AnalyzedPart::Freeform(part) => &part.node.range,
            AnalyzedPart::FilePath(part) => &part.node.range,
            AnalyzedPart::InlineShell(part) => &part.node.range,
        }
    }
}
