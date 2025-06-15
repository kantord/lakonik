use crate::ast::{Part, Verb};
use crate::hir::sentence::AnalyzedSentence;
use lsp_types::{CompletionItem, CompletionList, CompletionResponse, Position};

/// Get completion items for a given position in a document.
/// Returns None if the document is not valid or if the position is not in a valid location for completions.
pub fn get_completions(doc: &AnalyzedSentence, pos: &Position) -> Option<CompletionResponse> {
    // Check if we're after the verb (which means we're in the parts section)
    let verb_range = match &doc.verb.node {
        Verb::Simple(v) => &v.range,
        Verb::Assignment(v) => &v.range,
    };

    // Only provide completions if we're after the verb
    if pos.line < verb_range.end.line || 
       (pos.line == verb_range.end.line && pos.character <= verb_range.end.character) {
        return None;
    }

    // Get the current line's text up to the cursor position
    let line_text = doc.node.parts.iter()
        .find_map(|part| {
            if let Part::Freeform(p) = part {
                if p.range.start.line == pos.line {
                    Some(p.text.clone())
                } else {
                    None
                }
            } else {
                None
            }
        })
        .unwrap_or_default();

    // Get the prefix up to the cursor position
    let prefix = if pos.character as usize <= line_text.len() {
        &line_text[..pos.character as usize]
    } else {
        &line_text
    };

    // Filter completion items based on the prefix
    let items = vec![
        CompletionItem {
            label: "foo".to_string(),
            kind: Some(lsp_types::CompletionItemKind::TEXT),
            ..Default::default()
        },
        CompletionItem {
            label: "bar".to_string(),
            kind: Some(lsp_types::CompletionItemKind::TEXT),
            ..Default::default()
        },
        CompletionItem {
            label: "lorem".to_string(),
            kind: Some(lsp_types::CompletionItemKind::TEXT),
            ..Default::default()
        },
    ].into_iter()
    .filter(|item| item.label.starts_with(prefix))
    .collect();

    Some(CompletionResponse::List(CompletionList {
        is_incomplete: false,
        items,
    }))
} 