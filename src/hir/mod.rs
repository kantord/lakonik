#![allow(dead_code)]

pub mod part;
pub mod utils;
pub mod verb;
pub mod vocative;

use lsp_types::Range;
use part::AnalyzedPart;
use utils::{AnalysisContext, Analyzable, Analyzed};
use verb::AnalyzedVerb;
use vocative::AnalyzedVocative;

use crate::ast::Sentence;

#[derive(Clone, Debug)]
pub struct AnalyzedSentence {
    pub node: Sentence,
    pub hover_text: String,
    pub vocative: AnalyzedVocative,
    pub verb: AnalyzedVerb,
    pub parts: Vec<AnalyzedPart>,
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

impl Analyzed for AnalyzedVocative {
    fn get_range(&self) -> &Range {
        &self.node.range
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
