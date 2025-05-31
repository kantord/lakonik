use lsp_types::{Position, Range};
use nom_locate::LocatedSpan;

pub type Span<'a> = LocatedSpan<&'a str>;

pub fn range(span: Span) -> Range {
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

    Range {
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

pub trait RangeContainsPosition {
    fn contains_position(&self, pos: &Position) -> bool;
}

impl RangeContainsPosition for Range {
    fn contains_position(&self, pos: &Position) -> bool {
        (self.start.line < pos.line
            || (self.start.line == pos.line && self.start.character <= pos.character))
            && (pos.line < self.end.line
                || (pos.line == self.end.line && pos.character < self.end.character))
    }
}
