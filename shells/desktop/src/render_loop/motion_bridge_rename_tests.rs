//! Seam tests for **naming things** (doc 61) — the shell half of F2.
//!
//! The panel proves the box opens, seeds, commits and hands the keyboard back
//! (`f2_actually_renames_the_thing`). This proves the other half: that the name the box committed
//! lands in the right one of three storage places, that it is **undoable**, and that pressing
//! Enter on a box you did not edit is not an edit.
//!
//! Declared by the parent as a `#[path]` sibling, so `super` is `render_loop::motion_bridge`.

use crate::motion_state::MotionState;
use ph2d_motion_doc::{Backdrop, Subgraph};
use ph2d_nodegraph::graph::{Graph, NodeId};
use ph2d_panel_motion_graph::RenameTarget;

/// A document with one of each: a node, a group, and a backdrop — all three carrying the id **1**,
/// which is the ordinary case and the reason a rename cannot travel as a bare `u32`.
fn scene() -> MotionState {
    let mut motion = MotionState::new();
    motion.doc.graph = Graph::new();
    motion.doc.graph.add_node("motion.grid"); // id 0
    motion.doc.graph.add_node("motion.move"); // id 1
    motion.doc.subgraphs = vec![Subgraph {
        id: 1,
        parent: None,
        x: 0.0,
        y: 0.0,
        title: "Age & Fade".into(),
    }];
    motion.doc.backdrops = vec![Backdrop {
        id: 1,
        x: 0.0,
        y: 0.0,
        w: 100.0,
        h: 100.0,
        color: 0,
        title: "The Snow".into(),
    }];
    motion
}

/// **One id, three spaces, three names.** Rename each, and the other two must not move.
#[test]
fn the_name_lands_in_the_space_the_target_names() {
    let mut motion = scene();

    super::intents::rename(&mut motion, RenameTarget::Node(1), "Up To The Sky".into());
    assert_eq!(motion.doc.graph.label(NodeId(1)), Some("Up To The Sky"));
    assert_eq!(
        motion.doc.subgraphs[0].title, "Age & Fade",
        "the group with the same id must not have moved"
    );
    assert_eq!(motion.doc.backdrops[0].title, "The Snow");

    super::intents::rename(&mut motion, RenameTarget::Subgraph(1), "The Melt".into());
    assert_eq!(motion.doc.subgraphs[0].title, "The Melt");
    assert_eq!(
        motion.doc.graph.label(NodeId(1)),
        Some("Up To The Sky"),
        "and the NODE with the same id must not have moved either"
    );

    super::intents::rename(&mut motion, RenameTarget::Backdrop(1), "The Sea".into());
    assert_eq!(motion.doc.backdrops[0].title, "The Sea");
    assert_eq!(motion.doc.subgraphs[0].title, "The Melt");
}

/// A rename is an edit, so **Ctrl+Z takes it back** — one step, not one per keystroke (the box
/// commits once, on Enter).
#[test]
fn a_rename_is_one_undo_step() {
    let mut motion = scene();
    super::intents::rename(&mut motion, RenameTarget::Node(1), "The Sea".into());
    assert_eq!(motion.doc.graph.label(NodeId(1)), Some("The Sea"));

    let doc = motion.doc.clone();
    let pre = motion.history.undo(&doc).expect("one step");
    motion.doc = pre;
    assert_eq!(
        motion.doc.graph.label(NodeId(1)),
        None,
        "undo must give the card its old name back"
    );
}

/// **Enter on a box you did not edit is not an edit.** Without this the undo queue fills with
/// steps that change nothing, and the artist presses Ctrl+Z three times to undo one thing.
#[test]
fn committing_the_same_name_pushes_no_undo_step() {
    let mut motion = scene();
    super::intents::rename(&mut motion, RenameTarget::Backdrop(1), "The Snow".into());
    assert!(
        motion.history.undo(&motion.doc.clone()).is_none(),
        "the name did not change, so there is nothing to undo"
    );
}

/// **An empty name is not a name.** Clearing the box means *call it what it is* — the card goes
/// back to its type, the group back to its default title — not *leave it blank*.
#[test]
fn clearing_the_box_gives_the_thing_its_own_name_back() {
    let mut motion = scene();
    super::intents::rename(&mut motion, RenameTarget::Node(1), "Named".into());
    super::intents::rename(&mut motion, RenameTarget::Node(1), "   ".into());
    assert_eq!(
        motion.doc.graph.label(NodeId(1)),
        None,
        "whitespace is not a name"
    );

    super::intents::rename(&mut motion, RenameTarget::Subgraph(1), String::new());
    assert!(
        motion.doc.subgraphs[0].title.is_empty(),
        "and the group's empty title is what the fold turns back into its default card name"
    );
}

/// The boot document **names its cards** (doc 61) — which is both the ready-to-smoke example and
/// the thing that makes twenty cards, six of which are a `Move` or a `Drive`, readable.
///
/// FALSIFIABLE the way it matters: it does not check that `set_label` was called, it checks that
/// the name comes back out through the **snapshot the panel paints from**. Wire the label into the
/// graph and forget to read it in `snapshot_from` and this is what goes red.
#[test]
fn the_boot_document_names_its_cards() {
    let motion = MotionState::new();
    let snap = ph2d_panel_motion_graph::snapshot_from(&motion.doc.graph, &motion.registry);
    let named: Vec<&str> = snap
        .nodes
        .iter()
        .map(|n| n.display_name.as_str())
        .filter(|n| ["The Sea", "The Snow", "Birth Sites", "The Kill Disc"].contains(n))
        .collect();
    assert_eq!(
        named.len(),
        4,
        "the boot cards should carry the names the demo gave them, got: {:?}",
        snap.nodes
            .iter()
            .map(|n| n.display_name.as_str())
            .collect::<Vec<_>>()
    );
}
