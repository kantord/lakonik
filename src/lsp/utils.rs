use crate::ast::{Sentence, Span, parse_statement};
use crate::hir::utils::{AnalysisContext, Analyzable};
use lsp_types::Url;
use std::collections::HashMap;

use super::DocumentState;

pub fn parse(input: &str) -> Option<Sentence> {
    let span = Span::new(input);
    parse_statement(span).ok().map(|(_, sentence)| sentence)
}

pub fn update_document(docs: &mut HashMap<Url, DocumentState>, uri: Url, text: String) {
    if let Some(ast) = parse(&text) {
        let analyzed = ast.analyze(&mut AnalysisContext {});
        eprintln!("Parsed document: {analyzed:?}");
        docs.insert(uri, DocumentState { analyzed });
    } else {
        docs.remove(&uri);
        tracing::warn!("Could not parse document: {}", uri);
    }
}
