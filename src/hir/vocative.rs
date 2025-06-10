use lsp_types::Range;

use crate::ast::Vocative;

use super::{
    part::AnalyzedPart,
    utils::{AnalysisContext, Analyzable, Analyzed},
};

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

impl Analyzed for AnalyzedPart {
    fn get_range(&self) -> &Range {
        match self {
            AnalyzedPart::Freeform(part) => &part.node.range,
            AnalyzedPart::FilePath(part) => &part.node.range,
            AnalyzedPart::InlineShell(part) => &part.node.range,
        }
    }
}

impl Analyzed for AnalyzedVocative {
    fn get_range(&self) -> &Range {
        &self.node.range
    }
}
