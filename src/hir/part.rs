use crate::ast::{FilePathPart, FreeformPart, InlineShellPart, Part};

use super::utils::{AnalysisContext, Analyzable};

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
