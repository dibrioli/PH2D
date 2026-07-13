//! The Edit-ops section of the Audio Editor panel (W2).
//!
//! Opens with the **toolbar** — the basic operations, the ones you reach for without thinking:
//! the three tools (Select · Move · Scale), the clipboard (Cut · Copy · Paste), and the structure
//! (Split · Clear Cuts). Then the whole-clip ops (Undo/Redo · Normalize/LUFS · Reverse/DC ·
//! Gain−/Gain+ · Invert), then the selection range ops (Trim · Fade In/Out · Silence).
//!
//! Each button dims — and refuses in the seam — when it has nothing to act on (no clip / no
//! history / no selection / nothing on the clipboard / no cuts). Ends by delegating to the
//! effects rack.

use crate::paint::{ClippedHits, ROW_H, button, toggle};
use crate::tool_state::{self, EditTool};
use crate::{
    AEDIT_COPY, AEDIT_CUT, AEDIT_CUTS_CLEAR, AEDIT_DC, AEDIT_FADE_IN, AEDIT_FADE_OUT,
    AEDIT_GAIN_DOWN, AEDIT_GAIN_UP, AEDIT_INVERT, AEDIT_MONO, AEDIT_NORM_LUFS, AEDIT_NORMALIZE,
    AEDIT_PASTE, AEDIT_REDO, AEDIT_REVERSE, AEDIT_SILENCE, AEDIT_SPLIT_PLAYHEAD, AEDIT_TOOL_MOVE,
    AEDIT_TOOL_SCALE, AEDIT_TOOL_SELECT, AEDIT_TRIM, AEDIT_UNDO, loop_state, snapshot,
};
use ph2d_a11y::NodeId;
use ph2d_editor_core::zones::Rect;
use ph2d_text::TextSystem;
use ph2d_tokens::{Spacing, Theme};
use ph2d_vector::VectorScene;

/// The toolbar lays its rows out three across. A **count**, not a dimension — there is no design
/// token for "how many buttons fit on a line".
const TOOL_COLS: f32 = 3.0; // LITERAL-PX-OK: a column count, not a design value

/// The toolbar at the top of the section: the tools, then the clipboard, then the structure.
///
/// The three tools are a **group**: exactly one is armed, and clicking one arms it rather than
/// toggling it off — a pointer that means nothing is a pointer that does nothing.
#[allow(clippy::too_many_arguments)]
fn paint_toolbar(
    mut y: f32,
    x: f32,
    w: f32,
    loaded: bool,
    has_sel: bool,
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    hit_index: &mut ClippedHits,
) -> f32 {
    let gap = Spacing::Sm.px();
    let third = ((w - gap * (TOOL_COLS - 1.0)) / TOOL_COLS).max(1.0);
    let half = ((w - gap) * 0.5).max(1.0);
    let armed = tool_state::tool();

    // Row 1 — the tools. Move needs somewhere to drop a piece, so it stays dim until the clip is
    // actually cut; Scale works on the single uncut piece too (that is "shorten the whole take").
    let tools: [(&str, NodeId, EditTool, bool); 3] = [
        ("Select", AEDIT_TOOL_SELECT, EditTool::Select, loaded),
        (
            "Move",
            AEDIT_TOOL_MOVE,
            EditTool::Move,
            loaded && tool_state::pieces() > 1,
        ),
        ("Scale", AEDIT_TOOL_SCALE, EditTool::Scale, loaded),
    ];
    for (i, (label, id, t, enabled)) in tools.into_iter().enumerate() {
        toggle(
            Rect::new(x + i as f32 * (third + gap), y, third, ROW_H),
            label,
            armed == t,
            enabled,
            id,
            scene,
            text_system,
            theme,
            hit_index,
        );
    }
    y += ROW_H + gap;

    // Row 2 — the clipboard. Paste is the one op that does NOT need a selection: it needs
    // something to paste. A Paste button lit with an empty clipboard is a button that lies.
    let clip_row: [(&str, NodeId, bool); 3] = [
        ("Cut", AEDIT_CUT, has_sel),
        ("Copy", AEDIT_COPY, has_sel),
        ("Paste", AEDIT_PASTE, snapshot::has_clipboard()),
    ];
    for (i, (label, id, enabled)) in clip_row.into_iter().enumerate() {
        button(
            Rect::new(x + i as f32 * (third + gap), y, third, ROW_H),
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

    // Row 3 — structure. Split cuts at the playhead; Clear Cuts heals every seam (the audio stays
    // wherever you dragged it to — the pieces were never separate buffers).
    button(
        Rect::new(x, y, half, ROW_H),
        "Split",
        loaded,
        AEDIT_SPLIT_PLAYHEAD,
        scene,
        text_system,
        theme,
        hit_index,
    );
    button(
        Rect::new(x + half + gap, y, half, ROW_H),
        "Clear Cuts",
        tool_state::has_cuts(),
        AEDIT_CUTS_CLEAR,
        scene,
        text_system,
        theme,
        hit_index,
    );
    y + ROW_H + Spacing::Md.px()
}

/// The Edit ops block: the toolbar (tools · clipboard · structure), then whole-clip
/// (Undo/Redo · Normalize/LUFS · Reverse/DC · Gain−/Gain+ · Invert), then the selection
/// range ops (Trim · Fade In/Out · Silence). Buttons dim when unavailable.
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
    y = paint_toolbar(
        y,
        x,
        w,
        loaded,
        has_sel,
        scene,
        text_system,
        theme,
        hit_index,
    );

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

    // Selection range ops — enabled only when a waveform selection exists (drag on the overlay
    // with the Select tool to make one). Cut/Copy/Paste used to live down here; they are basic
    // operations, so they moved up to the toolbar where the hand reaches for them.
    let range_rows: [[(&str, NodeId, bool); 2]; 2] = [
        [
            ("Trim", AEDIT_TRIM, has_sel),
            ("Silence", AEDIT_SILENCE, has_sel),
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
    // The effects rack used to be painted from here. It is its own SECTION now
    // (`paint_sections`), so delegating to it as well drew the whole rack twice — and
    // since `HitIndex::hit` walks back-to-front, the copy up here was a ghost: painted,
    // and unclickable. Enio spotted the duplicate on sight (2026-07-12);
    // `no_control_is_painted_twice` keeps it spotted.
    y + Spacing::Md.px()
}
