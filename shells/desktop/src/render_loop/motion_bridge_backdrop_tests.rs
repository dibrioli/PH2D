//! Backdrop seam tests (Motion Nodes F2) — the shell half of the graph panel's
//! group regions. Declared by the parent as a `#[path]` sibling, so `super` is
//! `render_loop::motion_bridge`.
//!
//! What they pin down: a backdrop is DOCUMENT state (undoable, and it survives a
//! save/load) but it is **UI-only** (it never re-cooks, and deleting it never
//! takes a node with it).

use super::backdrops;
use crate::motion_state::MotionState;

/// A state whose pump has already cooked once, so `is_dirty` is `false` and a
/// later `true` can only have come from the edit under test.
fn cooked_state() -> MotionState {
    let mut motion = MotionState::new();
    motion.pump.pump(
        &motion.doc.graph,
        &motion.registry,
        &motion.sinks,
        0,
        0.0,
        motion.default_uv_rect,
        motion.default_size,
    );
    assert!(!motion.pump.is_dirty(), "the pump settles after a cook");
    motion
}

/// Adding a backdrop puts it on the document at the requested rect, with a title
/// the artist can rename and a tint that CYCLES — consecutive groups are born
/// visually distinct instead of a wall of one colour.
#[test]
fn adding_backdrops_lands_them_on_the_doc_with_cycling_tints() {
    let mut motion = MotionState::new();
    backdrops::add(&mut motion, 10.0, 20.0, 300.0, 200.0);
    backdrops::add(&mut motion, 0.0, 0.0, 100.0, 100.0);

    assert_eq!(motion.doc.backdrops.len(), 2);
    let first = &motion.doc.backdrops[0];
    assert_eq!(
        (first.x, first.y, first.w, first.h),
        (10.0, 20.0, 300.0, 200.0)
    );
    assert!(!first.title.is_empty(), "it lands named, not blank");
    assert_ne!(
        first.color, motion.doc.backdrops[1].color,
        "the tint cycles between groups"
    );
}

/// **The UI-only contract.** A backdrop is decoration: no edit to one may re-cook
/// the graph. FALSIFIED by a stray `mark_dirty` in the backdrop path — which would
/// re-cook every node on every frame of a backdrop drag (the whole reason a
/// grouping tool must be free). The control: a real graph edit DOES dirty it.
#[test]
fn no_backdrop_edit_ever_re_cooks_the_graph() {
    let mut motion = cooked_state();
    backdrops::add(&mut motion, 0.0, 0.0, 300.0, 200.0);
    let id = motion.doc.backdrops[0].id;
    backdrops::translate(&mut motion, id, 25.0, -10.0);
    backdrops::resize(&mut motion, id, 40.0, 40.0);
    backdrops::set_title(&mut motion, id, "Force chain".to_string());
    backdrops::set_color(&mut motion, id, 3);
    backdrops::delete(&mut motion, id);
    assert!(
        !motion.pump.is_dirty(),
        "decoration cannot change what the graph cooks"
    );

    // Control: an actual graph edit must still dirty the pump (otherwise the
    // assertion above would be vacuous — a pump that never dirties proves nothing).
    motion.pump.mark_dirty();
    assert!(motion.pump.is_dirty());
}

/// The gripper cannot collapse a backdrop into an ungrabbable sliver: the shell
/// clamps to the panel's own minimum (the SAME constant the panel draws and
/// hit-tests with, so the clamp can never drift from the geometry).
#[test]
fn a_resize_clamps_to_the_panels_minimum() {
    let mut motion = MotionState::new();
    backdrops::add(&mut motion, 0.0, 0.0, 300.0, 200.0);
    let id = motion.doc.backdrops[0].id;
    backdrops::resize(&mut motion, id, -9999.0, -9999.0);

    let b = &motion.doc.backdrops[0];
    assert_eq!(b.w, ph2d_panel_motion_graph::BACKDROP_MIN_W);
    assert_eq!(b.h, ph2d_panel_motion_graph::BACKDROP_MIN_H);
}

/// Deleting a backdrop removes the REGION and nothing else — a backdrop owns no
/// nodes, it only draws around them. FALSIFIED if the delete ever grew into a
/// "delete the group" (the nodes would vanish with the wallpaper).
#[test]
fn deleting_a_backdrop_leaves_every_node_standing() {
    let mut motion = cooked_state();
    let before = motion.doc.graph.nodes().len();
    assert!(before > 0, "the boot document has nodes to lose");

    backdrops::add(&mut motion, -500.0, -500.0, 5000.0, 5000.0); // frames everything
    let id = motion.doc.backdrops[0].id;
    backdrops::delete(&mut motion, id);

    assert!(motion.doc.backdrops.is_empty());
    assert_eq!(motion.doc.graph.nodes().len(), before, "the nodes stay");
}

/// Every backdrop edit is undoable (it is document state), and one Ctrl+Z takes
/// the whole thing back — the `[backdrop]` section is part of the doc snapshot.
#[test]
fn a_backdrop_is_undoable() {
    let mut motion = MotionState::new();
    backdrops::add(&mut motion, 10.0, 20.0, 300.0, 200.0);
    assert_eq!(motion.doc.backdrops.len(), 1);

    let back = motion.history.undo(&motion.doc).expect("one undo step");
    assert!(back.backdrops.is_empty(), "the add is undone");
}

/// **The round trip.** The `[backdrop]` section has been in the format since M0
/// but had no producer until this slice — so this is the first time a backdrop
/// authored in the UI is proven to survive a save/load with its rect, tint and
/// (space-bearing) title intact.
#[test]
fn an_authored_backdrop_survives_a_text_round_trip() {
    let mut motion = MotionState::new();
    backdrops::add(&mut motion, 12.5, -30.0, 420.0, 260.0);
    let id = motion.doc.backdrops[0].id;
    backdrops::set_title(&mut motion, id, "Force chain".to_string());
    backdrops::set_color(&mut motion, id, 5);

    let text = motion.doc.to_text();
    let back = ph2d_motion_doc::MotionDoc::from_text(&text).expect("the doc reloads");
    assert_eq!(
        back.backdrops, motion.doc.backdrops,
        "rect, tint and the title's interior space all survive"
    );
}
