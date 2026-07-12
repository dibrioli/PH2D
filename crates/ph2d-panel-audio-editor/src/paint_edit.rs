//! The Edit-ops section of the Audio Editor panel (W2).
//!
//! Whole-clip one-shot ops (Undo/Redo · Normalize/LUFS · Reverse/DC · Gain−/Gain+ ·
//! Invert) then the selection range ops (Trim/Cut · Fade In/Out · Silence). Each
//! button dims — and refuses in the seam — when it has nothing to act on (no clip /
//! no history / no selection). Ends by delegating to the effects rack.

use crate::paint::{ClippedHits, ROW_H, button, toggle};
use crate::{
    AEDIT_COPY, AEDIT_CUT, AEDIT_DC, AEDIT_FADE_IN, AEDIT_FADE_OUT, AEDIT_GAIN_DOWN, AEDIT_GAIN_UP,
    AEDIT_INVERT, AEDIT_MONO, AEDIT_NORM_LUFS, AEDIT_NORMALIZE, AEDIT_PASTE, AEDIT_REDO,
    AEDIT_REVERSE, AEDIT_SILENCE, AEDIT_TRIM, AEDIT_UNDO, loop_state, snapshot,
};
use ph2d_a11y::NodeId;
use ph2d_editor_core::zones::Rect;
use ph2d_text::TextSystem;
use ph2d_tokens::{Spacing, Theme};
use ph2d_vector::VectorScene;

/// The Edit ops block: whole-clip (Undo/Redo · Normalize/LUFS · Reverse/DC ·
/// Gain−/Gain+ · Invert) then the selection range ops (Trim/Cut · Fade In/Out ·
/// Silence). Buttons dim when unavailable (no clip / no history / no selection).
#[allow(clippy::too_many_arguments)]
pub(crate) fn paint_edit_section(
    mut y: f32,
    x: f32,
    w: f32,
    loaded: bool,
    undo_ok: bool,
    redo_ok: bool,
    has_sel: bool,
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    hit_index: &mut ClippedHits,
) -> f32 {
    let gap = Spacing::Sm.px();
    let half = ((w - gap) * 0.5).max(1.0);
    // (label, id, enabled) pairs, laid out two-per-row (last row is single).
    let rows: [[(&str, NodeId, bool); 2]; 4] = [
        [("Undo", AEDIT_UNDO, undo_ok), ("Redo", AEDIT_REDO, redo_ok)],
        [
            ("Normalize", AEDIT_NORMALIZE, loaded),
            ("Norm LUFS", AEDIT_NORM_LUFS, loaded),
        ],
        [
            ("Reverse", AEDIT_REVERSE, loaded),
            ("Rm DC", AEDIT_DC, loaded),
        ],
        [
            ("Gain \u{2212}", AEDIT_GAIN_DOWN, loaded),
            ("Gain +", AEDIT_GAIN_UP, loaded),
        ],
    ];
    for row in rows {
        for (i, (label, id, enabled)) in row.into_iter().enumerate() {
            let bx = x + i as f32 * (half + gap);
            button(
                Rect::new(bx, y, half, ROW_H),
                label,
                enabled,
                id,
                scene,
                text_system,
                theme,
                hit_index,
            );
        }
        y += ROW_H + gap;
    }
    // Invert | Force Mono (downmix the whole clip for 3D positional audio).
    button(
        Rect::new(x, y, half, ROW_H),
        "Invert",
        loaded,
        AEDIT_INVERT,
        scene,
        text_system,
        theme,
        hit_index,
    );
    toggle(
        Rect::new(x + half + gap, y, half, ROW_H),
        "Force Mono",
        loop_state::mono_on(),
        loaded,
        AEDIT_MONO,
        scene,
        text_system,
        theme,
        hit_index,
    );
    y += ROW_H + Spacing::Md.px();

    // Selection range ops — enabled only when a waveform selection exists (drag on
    // the overlay to make one).
    // Paste is the one op here that does NOT need a selection — it needs something to paste. A
    // Paste button lit with an empty clipboard is a button that lies.
    let has_clip = snapshot::has_clipboard();
    let range_rows: [[(&str, NodeId, bool); 2]; 3] = [
        [("Trim", AEDIT_TRIM, has_sel), ("Cut", AEDIT_CUT, has_sel)],
        [
            ("Copy", AEDIT_COPY, has_sel),
            ("Paste", AEDIT_PASTE, has_clip),
        ],
        [
            ("Fade In", AEDIT_FADE_IN, has_sel),
            ("Fade Out", AEDIT_FADE_OUT, has_sel),
        ],
    ];
    for row in range_rows {
        for (i, (label, id, enabled)) in row.into_iter().enumerate() {
            let bx = x + i as f32 * (half + gap);
            button(
                Rect::new(bx, y, half, ROW_H),
                label,
                enabled,
                id,
                scene,
                text_system,
                theme,
                hit_index,
            );
        }
        y += ROW_H + gap;
    }
    button(
        Rect::new(x, y, w, ROW_H),
        "Silence",
        has_sel,
        AEDIT_SILENCE,
        scene,
        text_system,
        theme,
        hit_index,
    );
    // The effects rack used to be painted from here. It is its own SECTION now
    // (`paint_sections`), so delegating to it as well drew the whole rack twice — and
    // since `HitIndex::hit` walks back-to-front, the copy up here was a ghost: painted,
    // and unclickable. Enio spotted the duplicate on sight (2026-07-12);
    // `no_control_is_painted_twice` keeps it spotted.
    y + ROW_H + Spacing::Md.px()
}
