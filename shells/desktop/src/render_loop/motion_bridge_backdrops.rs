//! Backdrop intents (Motion Nodes F2) — the shell half of the graph panel's group
//! regions, split out of `motion_bridge` (shell LOC cap).
//!
//! Backdrops are **UI-only decoration**: they live on the `MotionDoc` beside the
//! graph, they serialize with it, and **nothing here ever calls `mark_dirty`** —
//! a cook cannot depend on decoration, so re-cooking on a backdrop edit would be
//! pure waste (and would make dragging a group stutter a 70-node graph).
//!
//! They ARE undoable (they are document state), so every edit pushes an undo the
//! same way a node edit does: the drag/resize gestures arrive already bracketed by
//! `BeginDrag`/`EndDrag` (one step for the whole gesture, shared with the
//! companion `MoveNodes` that carries the framed nodes), and the one-shot edits
//! (add / delete / rename / re-tint) push their own.

use super::MotionState;
use ph2d_motion_doc::Backdrop;

/// The next free backdrop id. Ids are never reused within a session (a fresh
/// `max + 1`), so a stale id held by an in-flight gesture can only miss — never
/// hit the wrong backdrop.
fn next_id(motion: &MotionState) -> u32 {
    motion
        .doc
        .backdrops
        .iter()
        .map(|b| b.id)
        .max()
        .map_or(0, |m| m.saturating_add(1))
}

/// The default title a new backdrop lands with. English (app UI is English) —
/// the artist renames it in the params panel.
const DEFAULT_TITLE: &str = "Group";

/// Add a backdrop over the given graph-space rect (the panel already decided
/// whether that is the wrapped selection or a default block). One undo step.
pub(super) fn add(motion: &mut MotionState, x: f32, y: f32, w: f32, h: f32) {
    let pre = motion.doc.clone();
    let id = next_id(motion);
    // The tint cycles through the 8 tokens, so consecutive groups are born
    // visually distinct instead of a wall of the same colour.
    let color = (id % 8) as u8;
    motion.doc.backdrops.push(Backdrop {
        id,
        x,
        y,
        w,
        h,
        color,
        title: DEFAULT_TITLE.to_string(),
    });
    motion.history.push_undo(pre);
}

/// Move a backdrop (header drag). The nodes it frames are carried by the
/// companion `MoveNodes` the panel pushes in the same frame — this only moves the
/// region. Inside the panel's `BeginDrag`/`EndDrag` bracket, so no undo here.
pub(super) fn translate(motion: &mut MotionState, id: u32, dx: f32, dy: f32) {
    if let Some(b) = find(motion, id) {
        b.x += dx;
        b.y += dy;
    }
}

/// Resize from either bottom gripper, clamped to the panel's minimum so a backdrop
/// can never be collapsed into an ungrabbable sliver (its header would go with it
/// and the region would be unrecoverable). Inside the drag bracket.
///
/// **The edge the artist did NOT grab stays put.** Dragging the bottom-LEFT corner
/// moves `x` and shrinks `w`, holding the right edge — and the minimum-size clamp
/// anchors to that same fixed edge, so a corner pushed past the minimum stops dead
/// instead of dragging the whole region along with it.
pub(super) fn resize(motion: &mut MotionState, id: u32, left: bool, dx: f32, dy: f32) {
    let (min_w, min_h) = (
        ph2d_panel_motion_graph::BACKDROP_MIN_W,
        ph2d_panel_motion_graph::BACKDROP_MIN_H,
    );
    if let Some(b) = find(motion, id) {
        b.h = (b.h + dy).max(min_h);
        if left {
            let right = b.x + b.w; // the anchored edge
            b.w = (b.w - dx).max(min_w);
            b.x = right - b.w;
        } else {
            b.w = (b.w + dx).max(min_w);
        }
    }
}

/// Delete a backdrop. The nodes it framed STAY — a backdrop owns nothing, it
/// draws around things. One undo step.
pub(super) fn delete(motion: &mut MotionState, id: u32) {
    if motion.doc.backdrops.iter().any(|b| b.id == id) {
        let pre = motion.doc.clone();
        motion.doc.backdrops.retain(|b| b.id != id);
        motion.history.push_undo(pre);
    }
}

/// Rename (params panel Title row). **No undo push here** — the params bridge
/// brackets a typing session exactly as it does for a node's formula, so the whole
/// rename is ONE step instead of one per keystroke.
pub(super) fn set_title(motion: &mut MotionState, id: u32, title: String) {
    if let Some(b) = find(motion, id) {
        b.title = title;
    }
}

/// Re-tint (params panel Color row), clamped to the 8 tokens. Undo is the params
/// bridge's discrete bracket (like any enum row).
pub(super) fn set_color(motion: &mut MotionState, id: u32, color: u8) {
    if let Some(b) = find(motion, id) {
        b.color = color.min(7); // CLAMP-OK: 8 `graph-backdrop-*` tokens, 0-based
    }
}

/// The 8 tint names, in token order (`graph-backdrop-1..8`) — the hue ramp the
/// tokens actually walk (20°, 60°, 110°, 150°, 200°, 250°, 300°, and a near-grey),
/// so the label says what the artist will see. App UI is English.
const COLOR_LABELS: &[&str] = &[
    "Red", "Amber", "Olive", "Green", "Teal", "Blue", "Violet", "Grey",
];

/// The params-panel rows for the selected BACKDROP, or `None` when the subject is
/// a node (or nothing). A backdrop has no manifest and no `ParamUiHint`s — it is
/// not a node — so its two properties are hand-built here: the rename box and the
/// 8-tint picker. Without this the artist could create a group and never name it.
pub(super) fn params_snapshot(
    motion: &MotionState,
) -> Option<ph2d_panel_motion_params::ParamsSnapshot> {
    use ph2d_panel_motion_params::{EnumRow, ParamRow, ParamsSnapshot, TextRow};

    let id = ph2d_panel_motion_graph::current_graph_backdrop_selection()?;
    let b = motion.doc.backdrops.iter().find(|b| b.id == id)?;
    Some(ParamsSnapshot {
        node: b.id,
        title: "Backdrop".to_string(),
        rows: vec![
            ParamRow::Text(TextRow {
                name: "title",
                label: "Title".to_string(),
                value: b.title.clone(),
            }),
            ParamRow::Enum(EnumRow {
                name: "color",
                label: "Color".to_string(),
                selected: (b.color as usize).min(COLOR_LABELS.len() - 1),
                labels: COLOR_LABELS,
            }),
        ],
    })
}

/// Route one params-panel edit to the selected backdrop. Undo is the params
/// bridge's session bracket (a rename is one step, not one per keystroke), and
/// nothing here re-cooks.
pub(super) fn apply_param_intent(
    motion: &mut MotionState,
    id: u32,
    intent: ph2d_panel_motion_params::MotionParamIntent,
) {
    use ph2d_panel_motion_params::MotionParamIntent as I;
    match intent {
        I::SetParam {
            param: "color",
            value,
            ..
        } => set_color(motion, id, value.max(0.0) as u8),
        I::SetTextParam {
            param: "title",
            value,
            ..
        } => set_title(motion, id, value),
        _ => {}
    }
}

fn find(motion: &mut MotionState, id: u32) -> Option<&mut Backdrop> {
    motion.doc.backdrops.iter_mut().find(|b| b.id == id)
}
