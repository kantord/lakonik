pub struct AnalysisContext {}

pub trait Analyzable {
    type AnalyzedNode;

    fn analyze(&self, ctx: &mut AnalysisContext) -> Self::AnalyzedNode;
}
