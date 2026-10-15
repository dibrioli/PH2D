//! UTF-8 byte offsets ↔ LSP positions (zero-based line, UTF-16 code units — the encoding luau-lsp
//! negotiates). Line terminators per the LSP spec: `\n`, `\r\n` and a lone `\r`.

/// An LSP position: zero-based line and UTF-16 offset inside that line.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Default)]
pub struct Position {
    pub line: u32,
    pub character: u32,
}

impl Position {
    pub const fn new(line: u32, character: u32) -> Self {
        Self { line, character }
    }
}

/// A half-open LSP range.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Range {
    pub start: Position,
    pub end: Position,
}

fn floor_boundary(text: &str, byte: usize) -> usize {
    let mut b = byte.min(text.len());
    while !text.is_char_boundary(b) {
        b -= 1;
    }
    b
}

/// Byte offset → position. An offset inside a code point snaps back to its start; past the end
/// clamps to the end; between `\r` and `\n` it is the end of that line.
pub fn byte_to_position(text: &str, byte: usize) -> Position {
    let byte = floor_boundary(text, byte);
    let mut pos = Position::default();
    let mut chars = text[..byte].chars().peekable();
    while let Some(c) = chars.next() {
        match c {
            '\n' => pos = Position::new(pos.line + 1, 0),
            // `\r\n` counts once (on the `\n`); a `\r` cut off by `byte` is still part of the ending.
            '\r' if chars.peek() == Some(&'\n') => {}
            '\r' if chars.peek().is_none() && text[byte..].starts_with('\n') => {}
            '\r' => pos = Position::new(pos.line + 1, 0),
            _ => pos.character += c.len_utf16() as u32,
        }
    }
    pos
}

/// Position → byte offset. A line past the end gives `text.len()`; a character past the end of the
/// line gives the end of the line (before its terminator); inside a surrogate pair gives the start
/// of that code point.
pub fn position_to_byte(text: &str, pos: Position) -> usize {
    let mut start = 0usize;
    for _ in 0..pos.line {
        match next_line_start(text, start) {
            Some(s) => start = s,
            None => return text.len(),
        }
    }
    let mut units = 0u32;
    for (i, c) in text[start..].char_indices() {
        if c == '\n' || c == '\r' {
            return start + i;
        }
        let w = c.len_utf16() as u32;
        if units + w > pos.character {
            return start + i;
        }
        units += w;
    }
    text.len()
}

fn next_line_start(text: &str, from: usize) -> Option<usize> {
    let bytes = text.as_bytes();
    let mut i = from;
    while i < bytes.len() {
        match bytes[i] {
            b'\n' => return Some(i + 1),
            b'\r' if bytes.get(i + 1) == Some(&b'\n') => return Some(i + 2),
            b'\r' => return Some(i + 1),
            _ => i += 1,
        }
    }
    None
}

/// The position just past the last character (what an append edits at).
pub fn end_position(text: &str) -> Position {
    byte_to_position(text, text.len())
}
