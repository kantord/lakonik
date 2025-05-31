#![allow(dead_code)]
use crate::ast::{Part, Sentence, Verb, Vocative};

pub struct AnalysisContext {}

pub trait Analyzable<'a> {
    type AnalyzedNode: 'a;

    fn analyze(&'a self, ctx: &mut AnalysisContext) -> Self::AnalyzedNode;
}

pub struct AnalyzedVocative<'a> {
    pub node: &'a Vocative,
    pub hover_text: String,
}

pub struct AnalyzedPart<'a> {
    pub node: &'a Part,
    pub hover_text: String,
}

pub struct AnalyzedVerb<'a> {
    pub node: &'a Verb,
    pub hover_text: String,
}

pub struct AnalyzedSentence<'a> {
    pub node: &'a Sentence,
    pub hover_text: String,
    pub vocative: AnalyzedVocative<'a>,
    pub verb: AnalyzedVerb<'a>,
    pub parts: Vec<AnalyzedPart<'a>>,
}

impl<'a> Analyzable<'a> for Vocative {
    type AnalyzedNode = AnalyzedVocative<'a>;

    fn analyze(&'a self, _ctx: &mut AnalysisContext) -> Self::AnalyzedNode {
        AnalyzedVocative {
            node: self,
            hover_text: format!("Hover text for vocative: {}", self.name),
        }
    }
}

impl<'a> Analyzable<'a> for Part {
    type AnalyzedNode = AnalyzedPart<'a>;

    fn analyze(&'a self, _ctx: &mut AnalysisContext) -> Self::AnalyzedNode {
        AnalyzedPart {
            node: self,
            hover_text: "This is a part".to_string(),
        }
    }
}

impl<'a> Analyzable<'a> for Verb {
    type AnalyzedNode = AnalyzedVerb<'a>;

    fn analyze(&'a self, _ctx: &mut AnalysisContext) -> Self::AnalyzedNode {
        AnalyzedVerb {
            node: self,
            hover_text: "This is a part".to_string(),
        }
    }
}

impl<'a> Analyzable<'a> for Sentence {
    type AnalyzedNode = AnalyzedSentence<'a>;

    fn analyze(&'a self, _ctx: &mut AnalysisContext) -> Self::AnalyzedNode {
        AnalyzedSentence {
            node: self,
            hover_text: "This is a part".to_string(),
            verb: self.verb.analyze(_ctx),
            vocative: self.vocative.analyze(_ctx),
            parts: self.parts.iter().map(|part| part.analyze(_ctx)).collect(),
        }
    }
}
