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

use crate::paint::{ClippedHits, ROW_H, button_in_group, toggle_in_group};
use crate::tool_state::{self, EditTool};
use crate::{
    AEDIT_COPY, AEDIT_CUT, AEDIT_CUTS_CLEAR, AEDIT_DC, AEDIT_FADE_IN, AEDIT_FADE_OUT,
    AEDIT_GAIN_DOWN, AEDIT_GAIN_UP, AEDIT_INVERT, AEDIT_MONO, AEDIT_NORM_LUFS, AEDIT_NORMALIZE,
    AEDIT_PASTE, AEDIT_REDO, AEDIT_REVERSE, AEDIT_SILENCE, AEDIT_SPLIT_PLAYHEAD, AEDIT_TOOL_MOVE,
    AEDIT_TOOL_SCALE, AEDIT_TOOL_SELECT, AEDIT_TRIM, AEDIT_UNDO, loop_state, snapshot,
};
use ph2d_a11y::NodeId;
use ph2d_editor_core::widget::{block_cells, grid_height};
use ph2d_editor_core::zones::Rect;
use ph2d_text::TextSystem;
use ph2d_tokens::{Spacing, Theme};
use ph2d_vector::VectorScene;

/// The toolbar at the top of the section: the tools, then the clipboard, then the structure.
///
/// The three tools are a **group**: exactly one is armed, and clicking one arms it rather than
/// toggling it off — a pointer that means nothing is a pointer that does nothing.
#[allow(clippy::too_many_arguments)]
fn paint_toolbar(
    y: f32,
    x: f32,
    w: f32,
    loaded: bool,
    has_sel: bool,
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    hit_index: &mut ClippedHits,
) -> f32 {
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
    // ⭐⭐ **As três fileiras são UM bloco** — encostam na vertical como encostam na horizontal
    //    (a lei do Blender nas duas direcções). O dono, depois de ver só a metade horizontal:
    //    *«na vertical ainda tem muito espaço ainda»*.
    let block = block_cells(Rect::new(x, y, w, 0.0), &[3, 3, 2], ROW_H);
    let seg = &block[0];
    for (i, (label, id, t, enabled)) in tools.into_iter().enumerate() {
        toggle_in_group(
            seg[i].0,
            seg[i].1,
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

    // Row 2 — the clipboard. Paste is the one op that does NOT need a selection: it needs
    // something to paste. A Paste button lit with an empty clipboard is a button that lies.
    let clip_row: [(&str, NodeId, bool); 3] = [
        ("Cut", AEDIT_CUT, has_sel),
        ("Copy", AEDIT_COPY, has_sel),
        ("Paste", AEDIT_PASTE, snapshot::has_clipboard()),
    ];
    let seg = &block[1];
    for (i, (label, id, enabled)) in clip_row.into_iter().enumerate() {
        button_in_group(
            seg[i].0,
            label,
            enabled,
            id,
            seg[i].1,
            scene,
            text_system,
            theme,
            hit_index,
        );
    }

    // Row 3 — structure. Split cuts at the playhead; Clear Cuts heals every seam (the audio stays
    // wherever you dragged it to — the pieces were never separate buffers).
    let seg = &block[2];
    button_in_group(
        seg[0].0,
        "Split",
        loaded,
        AEDIT_SPLIT_PLAYHEAD,
        seg[0].1,
        scene,
        text_system,
        theme,
        hit_index,
    );
    button_in_group(
        seg[1].0,
        "Clear Cuts",
        tool_state::has_cuts(),
        AEDIT_CUTS_CLEAR,
        seg[1].1,
        scene,
        text_system,
        theme,
        hit_index,
    );
    y + grid_height(3, ROW_H) + Spacing::Md.px()
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
    // ⭐⭐ **As CINCO fileiras são um corpo só** — as quatro daqui mais a `Invert | Force Mono`
    //    logo abaixo: elas fazem a mesma coisa (agir sobre o clipe inteiro), logo encostam.
    let block = block_cells(Rect::new(x, y, w, 0.0), &[2, 2, 2, 2, 2], ROW_H);
    for (r, row) in rows.into_iter().enumerate() {
        let seg = &block[r];
        for (i, (label, id, enabled)) in row.into_iter().enumerate() {
            button_in_group(
                seg[i].0,
                label,
                enabled,
                id,
                seg[i].1,
                scene,
                text_system,
                theme,
                hit_index,
            );
        }
    }
    // Invert | Force Mono (downmix the whole clip for 3D positional audio).
    let seg = &block[4];
    button_in_group(
        seg[0].0,
        "Invert",
        loaded,
        AEDIT_INVERT,
        seg[0].1,
        scene,
        text_system,
        theme,
        hit_index,
    );
    toggle_in_group(
        seg[1].0,
        seg[1].1,
        "Force Mono",
        loop_state::mono_on(),
        loaded,
        AEDIT_MONO,
        scene,
        text_system,
        theme,
        hit_index,
    );
    y += grid_height(5, ROW_H) + Spacing::Md.px();

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
    let block = block_cells(Rect::new(x, y, w, 0.0), &[2, 2], ROW_H);
    for (r, row) in range_rows.into_iter().enumerate() {
        let seg = &block[r];
        for (i, (label, id, enabled)) in row.into_iter().enumerate() {
            button_in_group(
                seg[i].0,
                label,
                enabled,
                id,
                seg[i].1,
                scene,
                text_system,
                theme,
                hit_index,
            );
        }
    }
    y += grid_height(2, ROW_H);
    // The effects rack used to be painted from here. It is its own SECTION now
    // (`paint_sections`), so delegating to it as well drew the whole rack twice — and
    // since `HitIndex::hit` walks back-to-front, the copy up here was a ghost: painted,
    // and unclickable. Enio spotted the duplicate on sight (2026-07-12);
    // `no_control_is_painted_twice` keeps it spotted.
    y + Spacing::Md.px()
}
