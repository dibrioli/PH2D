//! Text-cursor + click-to-byte-offset helpers used by all text-editing
//! widgets (TextInput, NumberInput hex-buffer, Combobox query, TextArea).
//!
//! Extracted from [`super`] (Track A2). Two responsibilities:
//!
//! 1. **Char-boundary walking** — `prev_char_boundary`,
//!    `next_char_boundary`, `nearest_char_boundary` snap byte
//!    offsets to UTF-8 char boundaries so caret arithmetic over
//!    multi-byte content (any non-ASCII text) never panics.
//! 2. **Click → byte offset** — `byte_offset_from_click_xy` reads
//!    the widget's painted text + per-widget layout offsets and
//!    returns the byte offset closest to the click. Multi-line aware
//!    (TextArea); falls back to a `font_size * APPROX_ADVANCE_RATIO`
//!    heuristic when no `TextSystem` is available (test fixtures).
//! 3. **Caret-place** — `place_text_caret` writes the resolved offset
//!    into the matching state variant and seeds (or preserves) the
//!    selection anchor depending on whether the click was a Down or
//!    a Move during drag-select.

use super::super::{InteractiveState, WidgetStore};
use super::Rect;
use ph2d_a11y::NodeId;
use ph2d_text::TextSystem;

/// Approximate character advance per em — matches the painter's
/// caret-position formula. Used for drag-to-select byte-offset
/// computation without dragging text_system through dispatch.
pub(super) const APPROX_ADVANCE_RATIO: f32 = 0.55;

pub(super) fn prev_char_boundary(s: &str, mut i: usize) -> usize {
    if i == 0 {
        return 0;
    }
    i -= 1;
    while i > 0 && !s.is_char_boundary(i) {
        i -= 1;
    }
    i
}

pub(super) fn next_char_boundary(s: &str, mut i: usize) -> usize {
    let len = s.len();
    if i >= len {
        return len;
    }
    i += 1;
    while i < len && !s.is_char_boundary(i) {
        i += 1;
    }
    i
}

pub(super) fn nearest_char_boundary(s: &str, mut i: usize) -> usize {
    let len = s.len();
    if i >= len {
        return len;
    }
    while i > 0 && !s.is_char_boundary(i) {
        i -= 1;
    }
    i
}

/// Resolve the byte offset of the caret that should follow a click
/// at `(click_x, click_y)` on the widget identified by `id`.
///
/// Dispatches by widget kind: TextInput (single-line or `\n`-bearing
/// TextArea), NumberInput, Combobox. For multi-line content snaps to
/// end-of-line if `click_x` is past the last glyph on that line.
/// Returns `0` for non-text widgets.
pub(super) fn byte_offset_from_click_xy(
    store: &WidgetStore,
    id: NodeId,
    rect: Rect,
    click_x: f32,
    click_y: f32,
    text_system: Option<&mut TextSystem>,
) -> usize {
    // Per-widget text + layout parameters. Font sizes MUST match
    // the painters' tokens exactly — a 1 px mismatch shifts every
    // measured prefix and the caret lands on the wrong byte. All
    // text widgets paint at `TypeToken::Base.px()` except the hex
    // field which uses `TypeToken::Sm.px()`.
    //
    // `multiline` = true when the painter is the `TextArea` 3+ row
    // layout (inferred from `\n` content since the dispatch has no
    // widget-kind discriminator).
    use ph2d_tokens::TypeToken;
    let font_base = TypeToken::Base.px();
    let font_sm = TypeToken::Sm.px();
    let (text, text_start_x, text_start_y, font_size, multiline) = match store.get(id) {
        Some(InteractiveState::TextInput { text, .. }) => {
            let is_hex = store.blender_hex_parent(id).is_some();
            if is_hex {
                // Hex field paints label "Hex" + value text at Sm.
                (text.as_str(), rect.x + 8.0 + 36.0, rect.y, font_sm, false)
            } else if text.contains('\n') {
                // TextArea: pad_x = Spacing::Lg, pad_y = Spacing::Md,
                // line_h = font_size + 4 (matches the painter).
                (text.as_str(), rect.x + 12.0, rect.y + 8.0, font_base, true)
            } else {
                (text.as_str(), rect.x + 12.0, rect.y, font_base, false)
            }
        }
        Some(InteractiveState::NumberInput { buffer, .. }) => {
            // Plain NumberInput uses Spacing::Lg pad. Channel chips
            // are centered — their click→byte offset depends on the
            // current text width which we don't measure here, so we
            // approximate by treating the chip as if text starts at
            // its left padding.
            (buffer.as_str(), rect.x + 12.0, rect.y, font_base, false)
        }
        Some(InteractiveState::Combobox { query, .. }) => {
            // Combobox text sits AFTER the search icon + gap, not at
            // the left edge of the pill. Mirrors the painter math
            // `inner_x = rect.x + pad_x + icon_size + Spacing::Md`.
            let icon_size = (rect.h * 0.5).clamp(14.0, 18.0);
            let inner_x = rect.x + 12.0 + icon_size + 8.0;
            (query.as_str(), inner_x, rect.y, font_base, false)
        }
        _ => return 0,
    };
    if multiline {
        // Determine which `\n`-separated line was clicked from the
        // y-coordinate relative to the text-area inner top. Then
        // snap to end-of-line if the click lands past the last
        // glyph — fixes the "clicking right of short line on line
        // 1 lands the caret at end of line 2" feel reported by the
        // user (TextArea bug log).
        let line_h = font_size + 4.0;
        let rel_y = (click_y - text_start_y).max(0.0);
        let mut line_idx = (rel_y / line_h).floor() as usize;
        let line_count = text.split('\n').count();
        if line_count > 0 && line_idx >= line_count {
            line_idx = line_count - 1;
        }

        let mut line_start: usize = 0;
        for (i, line) in text.split('\n').enumerate() {
            let line_end = line_start + line.len();
            if i == line_idx {
                let local =
                    nearest_byte_on_line(line, font_size, text_start_x, click_x, text_system);
                return line_start + local;
            }
            line_start = line_end + 1; // +1 for the '\n'
        }
        return text.len();
    }

    nearest_byte_on_line(text, font_size, text_start_x, click_x, text_system)
}

/// For a single line `text` rendered at `font_size` starting at
/// pixel `text_start_x`, return the byte offset whose glyph boundary
/// is closest to `click_x`. With a real `TextSystem`, this means
/// pixel-perfect "caret lands where you clicked" UX. Without one,
/// falls back to a `font_size * APPROX_ADVANCE_RATIO` heuristic
/// (off by 1–2 chars on proportional fonts but tolerable for tests).
///
/// Snaps to **end-of-line** when `click_x` is past the last glyph —
/// never returns a byte past `text.len()`.
pub(super) fn nearest_byte_on_line(
    text: &str,
    font_size: f32,
    text_start_x: f32,
    click_x: f32,
    text_system: Option<&mut TextSystem>,
) -> usize {
    if let Some(ts) = text_system {
        // Walk every char boundary, layout the prefix, and pick the
        // boundary whose right edge is closest to click_x. O(n²)
        // for an n-char line but n is small (single-line content
        // for any reasonable input) and the parley LayoutContext
        // pools its allocations, so the actual cost per click is
        // microseconds.
        let target = (click_x - text_start_x).max(0.0);
        let mut best_byte: usize = 0;
        let mut best_dist = f32::INFINITY;
        for (idx, _) in text.char_indices() {
            // `prefix_width` includes trailing whitespace; the
            // naked `layout(...).width()` trimmed it, which broke
            // click→caret on lines that contained spaces.
            let w = ts.prefix_width(&text[..idx], font_size);
            let dist = (w - target).abs();
            if dist < best_dist {
                best_dist = dist;
                best_byte = idx;
            }
        }
        // Also consider the end-of-string boundary.
        let end_w = ts.prefix_width(text, font_size);
        let dist = (end_w - target).abs();
        if dist < best_dist {
            best_byte = text.len();
        }
        return best_byte;
    }
    // Fallback heuristic (no text_system).
    let advance = font_size * APPROX_ADVANCE_RATIO;
    if advance <= 0.0 {
        return 0;
    }
    let rel_x = (click_x - text_start_x).max(0.0);
    let approx_chars = (rel_x / advance).round() as usize;
    approx_chars.min(text.len())
}

/// Place the caret at byte offset `offset` on the TextInput /
/// NumberInput widget at `id`. When `seed_anchor` is true (single
/// Down event), the selection_anchor is reset to the new caret —
/// any prior selection collapses. When false (Move during drag),
/// the anchor is preserved so the selection extends from anchor →
/// new caret. No-op for non-text widgets.
pub(super) fn place_text_caret(
    store: &mut WidgetStore,
    id: NodeId,
    offset: usize,
    seed_anchor: bool,
) {
    let (text, caret, selection_anchor): (&str, &mut usize, &mut Option<usize>) =
        match store.get_mut(id) {
            Some(InteractiveState::TextInput {
                text,
                caret,
                selection_anchor,
                ..
            }) => (text.as_str(), caret, selection_anchor),
            Some(InteractiveState::NumberInput {
                buffer,
                caret,
                selection_anchor,
                ..
            }) => (buffer.as_str(), caret, selection_anchor),
            Some(InteractiveState::Combobox {
                query,
                caret,
                selection_anchor,
                ..
            }) => (query.as_str(), caret, selection_anchor),
            _ => return,
        };
    let bounded = offset.min(text.len());
    let snapped = nearest_char_boundary(text, bounded);
    *caret = snapped;
    if seed_anchor {
        *selection_anchor = Some(snapped);
    }
}
