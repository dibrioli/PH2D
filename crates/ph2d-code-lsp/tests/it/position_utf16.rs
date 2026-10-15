//! UTF-8 byte offset ↔ LSP (line, UTF-16) position: astral emoji (2 units, 4 bytes), accented
//! letters (1 unit, 2 bytes), CRLF / lone CR line endings, and clamping.

use ph2d_code_lsp::{Position, byte_to_position, end_position, position_to_byte};

fn p(line: u32, character: u32) -> Position {
    Position::new(line, character)
}

/// Every char boundary maps to a position and back to the same byte.
fn round_trips(text: &str) {
    for (b, _) in text.char_indices().chain([(text.len(), ' ')]) {
        let pos = byte_to_position(text, b);
        let back = position_to_byte(text, pos);
        // A boundary between `\r` and `\n` is the same position as the `\r` boundary.
        let same_line_end = text[..b].ends_with('\r') && text[b..].starts_with('\n');
        if !same_line_end {
            assert_eq!(back, b, "{text:?}: byte {b} → {pos:?} → {back}");
        }
    }
}

#[test]
fn utf16_emoji_counts_two_units_and_four_bytes() {
    let t = "a😀b";
    assert_eq!(byte_to_position(t, 1), p(0, 1));
    assert_eq!(byte_to_position(t, 5), p(0, 3));
    assert_eq!(position_to_byte(t, p(0, 3)), 5);
    // Inside the surrogate pair → start of the emoji; inside its UTF-8 bytes → start too.
    assert_eq!(position_to_byte(t, p(0, 2)), 1);
    assert_eq!(byte_to_position(t, 3), p(0, 1));
    round_trips(t);
}

#[test]
fn utf16_accented_letters_count_one_unit_two_bytes() {
    let t = "ação = \"olá\"\nçé";
    assert_eq!(byte_to_position(t, "ação".len()), p(0, 4));
    assert_eq!(position_to_byte(t, p(0, 4)), "ação".len());
    let second = t.find('\n').unwrap() + 1;
    assert_eq!(byte_to_position(t, second + "ç".len()), p(1, 1));
    assert_eq!(position_to_byte(t, p(1, 2)), t.len());
    round_trips(t);
}

#[test]
fn utf16_crlf_and_lone_cr_end_lines_once() {
    let t = "ab\r\ncd\rñ\n";
    assert_eq!(byte_to_position(t, 4), p(1, 0)); // after "\r\n"
    assert_eq!(byte_to_position(t, 3), p(0, 2)); // between \r and \n = end of line 0
    assert_eq!(byte_to_position(t, 7), p(2, 0)); // after lone "\r"
    assert_eq!(position_to_byte(t, p(1, 0)), 4);
    assert_eq!(position_to_byte(t, p(2, 0)), 7);
    assert_eq!(position_to_byte(t, p(0, 99)), 2); // past the line end → before its terminator
    assert_eq!(end_position(t), p(3, 0));
    round_trips(t);
}

#[test]
fn utf16_clamps_past_the_end() {
    let t = "x\ny";
    assert_eq!(position_to_byte(t, p(9, 0)), t.len());
    assert_eq!(byte_to_position(t, 99), p(1, 1));
    assert_eq!(position_to_byte("", p(0, 0)), 0);
    assert_eq!(byte_to_position("", 0), p(0, 0));
}
