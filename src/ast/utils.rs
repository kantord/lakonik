use lsp_types::Position;
use nom_locate::LocatedSpan;
use serde::Serialize;

pub type Span<'a> = LocatedSpan<&'a str>;

#[derive(Debug, PartialEq, Clone, Copy, Serialize)]
pub struct SourceRange {
    pub start: Position,
    pub end: Position,
}

pub fn range(span: Span) -> SourceRange {
    let start_offset = span.location_offset();
    let start_line = span.location_line();

    let mut end_offset = start_offset;
    let mut end_line = start_line;

    for ch in span.fragment().chars() {
        end_offset += ch.len_utf8();
        if ch == '\n' {
            end_line += 1;
        }
    }

    SourceRange {
        start: Position {
            line: start_line - 1,
            character: start_offset as u32,
        },
        end: Position {
            line: end_line - 1,
            character: end_offset as u32,
        },
    }
}
