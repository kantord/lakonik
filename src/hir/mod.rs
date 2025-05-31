#![allow(dead_code)]
use crate::ast::{Part, Sentence, Verb, Vocative};

pub struct AnalysisContext {}

pub trait Analyzable {
    type AnalyzedNode;

    fn analyze(&self, ctx: &mut AnalysisContext) -> Self::AnalyzedNode;
}

#[derive(Clone)]
pub struct AnalyzedVocative {
    pub node: Vocative,
    pub hover_text: String,
}

#[derive(Clone)]
pub struct AnalyzedPart {
    pub node: Part,
    pub hover_text: String,
}

#[derive(Clone)]
pub struct AnalyzedVerb {
    pub node: Verb,
    pub hover_text: String,
}

#[derive(Clone)]
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
        AnalyzedPart {
            node: self.clone(),
            hover_text: "This is a part".to_string(),
        }
    }
}

impl Analyzable for Verb {
    type AnalyzedNode = AnalyzedVerb;

    fn analyze(&self, _ctx: &mut AnalysisContext) -> Self::AnalyzedNode {
        AnalyzedVerb {
            node: self.clone(),
            hover_text: "This is a part".to_string(),
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
