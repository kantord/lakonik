use lsp_types::Range;

pub struct AnalysisContext {}

pub trait Analyzable {
    type AnalyzedNode;

    fn analyze(&self, ctx: &mut AnalysisContext) -> Self::AnalyzedNode;
}

pub trait Analyzed {
    fn get_range(&self) -> &Range;
}
