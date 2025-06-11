use lsp_types::Range;

use crate::{
    ast::Verb,
    templates::{get_all_templates, get_user_templates},
};

use super::utils::{AnalysisContext, Analyzable, Analyzed};

#[derive(Debug, Clone)]
pub struct AnalyzedVerb {
    pub node: Verb,
    pub template_name: String,
    pub hover_text: String,
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

impl Analyzable for Verb {
    type AnalyzedNode = AnalyzedVerb;

    fn analyze(&self, _ctx: &mut AnalysisContext) -> Self::AnalyzedNode {
        let template_name = match self {
            Verb::Simple(node) => node.name.clone(),
            Verb::Assignment(node) => node.name.clone(),
        };
        let template_name = format!("verbs/{}", template_name);
        let mut template_source = get_all_templates()
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

impl Analyzed for AnalyzedVerb {
    fn get_range(&self) -> &Range {
        match &self.node {
            Verb::Simple(node) => &node.range,
            Verb::Assignment(node) => &node.range,
        }
    }
}
