use crate::ast::Vocative;

use super::utils::{AnalysisContext, Analyzable};

#[derive(Clone, Debug)]
pub struct AnalyzedVocative {
    pub node: Vocative,
    pub hover_text: String,
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
