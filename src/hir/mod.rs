#![allow(dead_code)]
use lsp_types::Range;

use crate::ast::{FilePathPart, FreeformPart, InlineShellPart, Part, Sentence, Verb, Vocative};

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
        AnalyzedVerb {
            node: self.clone(),
            hover_text: "This is a verb".to_string(),
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
    pub fn get_template_name(&self) -> &str {
        match &self.node {
            Verb::Simple(node) => &node.name,
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
