use crate::ast::{Part, Verb};
use crate::hir::sentence::AnalyzedSentence;
use lsp_types::{CompletionItem, CompletionList, CompletionResponse, Position};
use std::process::Command;
use std::env;

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

    // Get executables from PATH
    let path = env::var("PATH").unwrap_or_default();
    let mut executables = Vec::new();

    for dir in path.split(':') {
        if let Ok(output) = Command::new("ls")
            .arg("-1")  // One entry per line
            .arg(dir)
            .output() 
        {
            if let Ok(output_str) = String::from_utf8(output.stdout) {
                for line in output_str.lines() {
                    let name = line.trim();
                    if !name.is_empty() {
                        executables.push(name.to_string());
                    }
                }
            }
        }
    }

    // Remove duplicates and sort
    executables.sort();
    executables.dedup();

    // Predefined words for completion
    let predefined_words = vec![
        "foo".to_string(),
        "bar".to_string(),
        "lorem".to_string(),
    ];

    // Combine both types of completions
    let mut items = Vec::new();

    // Add system command completions
    items.extend(executables.into_iter()
        .filter(|name| name.starts_with(prefix))
        .map(|name| {
            CompletionItem {
                label: format!("$({})", name),
                kind: Some(lsp_types::CompletionItemKind::FUNCTION),
                detail: Some(format!("Execute command: {}", name)),
                ..Default::default()
            }
        }));

    // Add predefined word completions
    items.extend(predefined_words.into_iter()
        .filter(|word| word.starts_with(prefix))
        .map(|word| {
            CompletionItem {
                label: word,
                kind: Some(lsp_types::CompletionItemKind::TEXT),
                ..Default::default()
            }
        }));

    Some(CompletionResponse::List(CompletionList {
        is_incomplete: false,
        items,
    }))
} 