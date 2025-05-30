use nom_locate::LocatedSpan;
use serde::Serialize;

pub type Span<'a> = LocatedSpan<&'a str>;

#[derive(Debug, PartialEq, Clone, Copy, Serialize)]
pub struct SourcePosition {
    pub line: u32,
    pub offset: usize,
}

#[derive(Debug, PartialEq, Clone, Copy, Serialize)]
pub struct SourceRange {
    pub start: SourcePosition,
    pub end: SourcePosition,
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
        start: SourcePosition {
            line: start_line,
            offset: start_offset,
        },
        end: SourcePosition {
            line: end_line,
            offset: end_offset,
        },
    }
}
