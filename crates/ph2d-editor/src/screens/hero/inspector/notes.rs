//! Sticky-note painters (single editable note + its title/body
//! sub-painters).
//!
//! Extracted from [`super`] (Track C4). Notes are right-click-created
//! highlights pinned above Inspector sections. Each painted note
//! registers three hit rects — the whole slot for the right-click
//! background-color menu, plus title + body sub-rects for the
//! TextInput focus + edit pipeline.

use super::{NOTE_BODY_IDS, NOTE_SLOT_IDS, NOTE_TITLE_IDS, read_text_input};
use crate::interaction::{HitIndex, NoteData, WidgetStore};
use crate::paint::{fill_rounded_rect, paint_text};
use crate::widget::TextInputState;
use crate::zones::Rect;
use ph2d_a11y::NodeId;
use ph2d_text::TextSystem;
use ph2d_tokens::{Radius, Spacing, StrokeToken, TypeToken};
use ph2d_vector::VectorScene;

/// Horizontal inset between a note's rect edge and where text drawn
/// inside it actually starts. Matches the non-hex `TextInput`
/// dispatch math in `byte_offset_from_click_xy` (`rect.x + 12.0`) so
/// click→caret + drag-select route to the byte under the visible
/// cursor.
const NOTE_TEXT_PAD_X: f32 = Spacing::Lg.px();

/// Vertical inset for multi-line note body painting. Mirrors the
/// `TextArea` dispatch math (`text_start_y = rect.y + 8.0`).
const NOTE_TEXT_PAD_Y: f32 = Spacing::Md.px();

/// Paint a single sticky-note. Editable: the title + body each
/// have their own TextInput state in the store
/// (`NOTE_TITLE_IDS[slot]` + `NOTE_BODY_IDS[slot]`). Single click
/// on either focuses it; double-click selects all; typing edits.
/// Dark glyphs hardcoded `#212121` for contrast over the light
/// highlighter bg regardless of theme.
///
/// Registers three hit rects:
///   - The whole note slot (`NOTE_SLOT_IDS[slot]`) for right-click
///     → background-color menu. Registered FIRST so the more
///     specific title/body rects layered above win pointer hits
///     within them.
///   - The title sub-rect (`NOTE_TITLE_IDS[slot]`) for focus + edit.
///   - The body sub-rect (`NOTE_BODY_IDS[slot]`).
#[allow(clippy::too_many_arguments)]
pub(super) fn paint_one_note(
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    hit_index: &mut HitIndex,
    store: &WidgetStore,
    x: f32,
    w: f32,
    y: &mut f32,
    note: &NoteData,
    slot: usize,
) {
    // Font sizes MUST match the dispatch's hardcoded `TypeToken::
    // Base.px()` (13 px) so click→caret mapping uses the same
    // glyph width as the painter. Using `Sm`/`Xs` caused the
    // dispatch to measure prefixes at the wrong font size and put
    // the caret 1–3 chars off (the user's "mapeamento errado do
    // mouse"). Same lesson as docs/UI_Bugs §3.3.
    let pad = Spacing::Md.px();
    let title_font = TypeToken::Base.px();
    let body_font = TypeToken::Base.px();
    let title_h = title_font + Spacing::Md.px();
    // Body holds ~3 lines. Painter starts at rect.y + NOTE_TEXT_PAD_Y
    // (=8) and uses line_h = body_font + 4. Total height needs both
    // the top inset and a small bottom buffer or the third line gets
    // clipped under the note's rounded bottom edge.
    let body_h = NOTE_TEXT_PAD_Y * 2.0 + (body_font + Spacing::Xs.px()) * 3.0; // LITERAL-PX-OK: 3 lines (line count)
    let note_h = title_h + body_h + pad * 2.0;
    let r = Rect::new(x, *y, w, note_h);
    if let Some(slot_id) = NOTE_SLOT_IDS.get(slot) {
        hit_index.register(*slot_id, r);
    }
    let rgba = crate::screens::hero::context_menu_overlay::HIGHLIGHTER_RGBA
        [note.color_idx.min(4) as usize];
    let bg = ph2d_vector::Color::from_rgba8(rgba[0], rgba[1], rgba[2], rgba[3]); // LITERAL-COLOR-OK: user-color — HIGHLIGHTER_RGBA palette (note background)
    fill_rounded_rect(scene, r, Radius::Md.px(), bg);

    let dark = ph2d_vector::Color::from_rgba8(0x21, 0x21, 0x21, 0xFF); // LITERAL-COLOR-OK: note-text — dark glyph fixed across themes because BG is the user's highlighter color, not a theme token
    // Title row.
    let title_rect = Rect::new(r.x + pad, r.y + pad, r.w - pad * 2.0, title_h);
    if let Some(title_id) = NOTE_TITLE_IDS.get(slot) {
        hit_index.register(*title_id, title_rect);
        paint_note_editable_line(
            scene,
            text_system,
            store,
            *title_id,
            title_rect,
            title_font,
            dark,
            "Title",
        );
    }
    // Body region — multi-line below the title.
    let body_rect = Rect::new(r.x + pad, r.y + pad + title_h, r.w - pad * 2.0, body_h);
    if let Some(body_id) = NOTE_BODY_IDS.get(slot) {
        hit_index.register(*body_id, body_rect);
        paint_note_editable_multiline(
            scene,
            text_system,
            store,
            *body_id,
            body_rect,
            body_font,
            dark,
            "Notes…",
        );
    }
    *y += note_h + Spacing::Md.px();
}

/// Paint an editable single-line text field with no chrome —
/// dark glyphs + caret + selection on a transparent background.
/// Reads the TextInput state at `id` from the store.
///
/// CRITICAL: text origin matches the TextInput dispatch contract
/// (`text_start_x = rect.x + 12`, `text_start_y = rect.y`) — without
/// this alignment, `byte_offset_from_click_xy` measures from a
/// different origin than the painter and click→caret + drag-select
/// land on the wrong byte. Vertical centering still happens, but
/// click→byte ignores y for single-line widgets.
#[allow(clippy::too_many_arguments)]
fn paint_note_editable_line(
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    store: &WidgetStore,
    id: NodeId,
    rect: Rect,
    font_size: f32,
    fg: ph2d_vector::Color,
    placeholder: &str,
) {
    let (state, text, caret, anchor) = read_text_input(store, id);
    let focused = state == TextInputState::Focused;
    let text_x = rect.x + NOTE_TEXT_PAD_X;
    let text_w = (rect.w - NOTE_TEXT_PAD_X).max(0.0);
    let text_y = rect.y + (rect.h - font_size) * 0.5;
    // Selection highlight (drawn under glyphs).
    if focused
        && !text.is_empty()
        && let Some(a) = anchor
        && a != caret
    {
        let (s, e) = if a < caret { (a, caret) } else { (caret, a) };
        let s = s.min(text.len());
        let e = e.min(text.len());
        let prefix_w = text_system.prefix_width(&text[..s], font_size);
        let mid_w = if s == e {
            0.0
        } else {
            text_system.prefix_width(&text[s..e], font_size)
        };
        let sel_x = text_x + prefix_w;
        let sel_w = mid_w.min(text_x + text_w - sel_x).max(0.0);
        if sel_w > 0.0 {
            let sel = Rect::new(
                sel_x,
                rect.y + 2.0,
                sel_w,
                (rect.h - Spacing::Xs.px()).max(2.0),
            );
            // Translucent dark wash for the selection so the highlighter
            // bg still shows through.
            let sel_color = ph2d_vector::Color::from_rgba8(0x21, 0x21, 0x21, 0x33); // LITERAL-COLOR-OK: note-selection — translucent dark wash, theme-invariant (BG is user-highlighter)
            fill_rounded_rect(scene, sel, 1.0, sel_color);
        }
    }
    // Visible text (or placeholder if empty and not focused).
    let displayed: &str = if text.is_empty() && !focused {
        placeholder
    } else {
        text
    };
    let display_color = if text.is_empty() && !focused {
        ph2d_vector::Color::from_rgba8(0x21, 0x21, 0x21, 0x80) // LITERAL-COLOR-OK: note-placeholder — dim dark text, theme-invariant (BG is user-highlighter)
    } else {
        fg
    };
    paint_text(
        text_system,
        scene,
        displayed,
        text_x,
        text_y,
        font_size,
        text_w,
        display_color,
    );
    // Caret — only when focused.
    if focused {
        let caret_byte = caret.min(text.len());
        let prefix_w = text_system.prefix_width(&text[..caret_byte], font_size);
        let caret_rect = Rect::new(
            (text_x + prefix_w).min(text_x + text_w),
            rect.y + 2.0,
            StrokeToken::Default.px(),
            (rect.h - Spacing::Xs.px()).max(2.0),
        );
        fill_rounded_rect(scene, caret_rect, 0.75, fg); // LITERAL-PX-OK: caret half-width radius (smooth rounded caret)
    }
}

/// Paint an editable multi-line text region with no chrome. Splits
/// the text on `\n` and renders each line; caret + selection mirror
/// the same per-line math the `TextArea` widget uses.
#[allow(clippy::too_many_arguments)]
fn paint_note_editable_multiline(
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    store: &WidgetStore,
    id: NodeId,
    rect: Rect,
    font_size: f32,
    fg: ph2d_vector::Color,
    placeholder: &str,
) {
    let (state, text, caret, anchor) = read_text_input(store, id);
    let focused = state == TextInputState::Focused;
    // line_h MUST match the dispatch's TextArea math:
    // `byte_offset_from_click_xy` uses `font_size + 4.0`. Drift
    // breaks click-y → line index. Origin offsets (text_x, text_y0)
    // also MUST match dispatch (`rect.x + 12`, `rect.y + 8`) so
    // click→caret lands at the visible glyph boundary.
    let line_h = font_size + Spacing::Xs.px();
    let text_x = rect.x + NOTE_TEXT_PAD_X;
    let text_y0 = rect.y + NOTE_TEXT_PAD_Y;
    let text_w = (rect.w - NOTE_TEXT_PAD_X).max(0.0);
    if text.is_empty() && !focused {
        paint_text(
            text_system,
            scene,
            placeholder,
            text_x,
            text_y0,
            font_size,
            text_w,
            ph2d_vector::Color::from_rgba8(0x21, 0x21, 0x21, 0x80), // LITERAL-COLOR-OK: note-placeholder multiline — same dim dark as single-line; theme-invariant
        );
        return;
    }
    // Selection highlight (drawn under glyphs). Supports spanning
    // multiple lines: paints one box per visible line covered by the
    // range. Mirrors `TextArea`'s widget selection math.
    if focused
        && let Some(a) = anchor
        && a != caret
    {
        let (s, e) = if a < caret { (a, caret) } else { (caret, a) };
        let s = s.min(text.len());
        let e = e.min(text.len());
        let sel_color = ph2d_vector::Color::from_rgba8(0x21, 0x21, 0x21, 0x33); // LITERAL-COLOR-OK: note-selection multiline — same translucent dark as single-line
        let mut line_start = 0_usize;
        for (i, line) in text.split('\n').enumerate() {
            let line_end = line_start + line.len();
            // Overlap of [s, e] with this line's byte range.
            let seg_s = s.max(line_start);
            let seg_e = e.min(line_end);
            if seg_s < seg_e {
                let local_s = seg_s - line_start;
                let local_e = seg_e - line_start;
                let prefix_w = text_system.prefix_width(&line[..local_s], font_size);
                let mid_w = text_system.prefix_width(&line[local_s..local_e], font_size);
                let sel_x = text_x + prefix_w;
                let sel_w = mid_w.min(text_x + text_w - sel_x).max(0.0);
                if sel_w > 0.0 {
                    let sel = Rect::new(sel_x, text_y0 + i as f32 * line_h, sel_w, line_h);
                    fill_rounded_rect(scene, sel, 1.0, sel_color);
                }
            }
            line_start = line_end + 1;
        }
    }
    for (i, line) in text.split('\n').enumerate() {
        paint_text(
            text_system,
            scene,
            line,
            text_x,
            text_y0 + i as f32 * line_h,
            font_size,
            text_w,
            fg,
        );
    }
    if focused {
        // Caret on the line containing the caret byte.
        let caret_byte = caret.min(text.len());
        let mut line_start = 0_usize;
        let mut line_idx = 0_usize;
        let mut line_text: &str = "";
        for line in text.split('\n') {
            let line_end = line_start + line.len();
            if caret_byte <= line_end {
                line_text = line;
                break;
            }
            line_start = line_end + 1;
            line_idx += 1;
        }
        let local = caret_byte.saturating_sub(line_start).min(line_text.len());
        let prefix_w = text_system.prefix_width(&line_text[..local], font_size);
        let caret_rect = Rect::new(
            (text_x + prefix_w).min(text_x + text_w),
            text_y0 + line_idx as f32 * line_h,
            StrokeToken::Default.px(),
            (line_h - 2.0).max(2.0),
        );
        fill_rounded_rect(scene, caret_rect, 0.75, fg); // LITERAL-PX-OK: caret half-width radius
    }
}
