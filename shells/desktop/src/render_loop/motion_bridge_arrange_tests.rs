//! Auto-arrange seam test (Motion Nodes — the Hierarchy chip). Declared by the
//! parent as a `#[path]` sibling, so `super` is `render_loop::motion_bridge`.
//!
//! What it pins down: the `ArrangeLayout` intent lays the document out in the
//! layered flow (positions only), as exactly ONE undo step, and — because positions
//! are UI-only — it never re-cooks. The same UI-only-but-undoable contract as the
//! backdrops, driven through the REAL intent funnel (`apply_graph_intents`), not by
//! poking the doc.

use super::apply_graph_intents;
use crate::motion_state::MotionState;
use ph2d_nodegraph::graph::{Edge, NodeId, Pos};
use ph2d_panel_motion_graph::{GraphIntent, drain_intents, push_intent};

/// A two-node chain (`grid -> output`) with the SOURCE dumped at a stale, far-off
/// position. Arrange has to lay it into a clean left-to-right layout. Returns the
/// state and the stale node's id.
fn stale_chain() -> (MotionState, NodeId) {
    let mut m = MotionState::new();
    m.doc = ph2d_motion_doc::MotionDoc::new();
    let a = m.doc.graph.add_node("motion.grid");
    let b = m.doc.graph.add_node("motion.output");
    m.doc
        .graph
        .connect(Edge {
            from: (a, 0),
            to: (b, 0),
            delayed: false,
        })
        .expect("edge");
    m.doc.graph.set_pos(
        a,
        Pos {
            x: 9999.0,
            y: 9999.0,
        },
    );
    (m, a)
}

/// Push one `ArrangeLayout` and run it through the real funnel.
fn arrange(m: &mut MotionState) {
    let _ = drain_intents();
    push_intent(GraphIntent::ArrangeLayout);
    apply_graph_intents(
        m,
        &mut ph2d_core::Playhead::default(),
        &mut ph2d_editor::ToastQueue::default(),
        &mut ph2d_editor::screens::layout::CenterSplit::None,
    );
}

/// **Arranging lays the graph out in ONE undo step.** The stale source is moved into
/// the layout (column 0 — near the origin band, left of its target), and a single
/// Ctrl+Z brings the mess back. FALSIFIED by not calling `layout::arrange` (the node
/// stays at 9999) or by not bracketing the undo (Ctrl+Z would not restore it).
#[test]
fn arranging_lays_the_graph_out_in_one_undo_step() {
    let (mut m, a) = stale_chain();
    assert!(!m.history.can_undo(), "a fresh state has nothing to undo");

    arrange(&mut m);

    let pa = m.doc.graph.pos(a).expect("arranged position");
    assert!(
        pa.x < 100.0,
        "the stale source was laid into the layout (x={})",
        pa.x
    );

    assert!(m.history.can_undo(), "arranging is exactly one undo step");
    let back = m.history.undo(&m.doc).expect("one undo step");
    assert_eq!(
        back.graph.pos(a).expect("restored position").x,
        9999.0,
        "Ctrl+Z brings the stale layout back"
    );
}

/// **Arranging an ALREADY-tidy graph is not an edit.** It pushes no undo step — the
/// same guard the backdrop-colour intent uses, so that pressing a no-op button never
/// fills the undo queue with steps the artist did not make. The fixture is tidied
/// DIRECTLY (no history), and `layout::arrange` is idempotent, so the intent-arrange
/// changes nothing. FALSIFIED by dropping the `if doc != pre` guard (the no-op would
/// push a spurious step).
#[test]
fn arranging_an_already_tidy_graph_pushes_no_undo_step() {
    let (mut m, _a) = stale_chain();
    ph2d_motion_doc::layout::arrange(&mut m.doc); // tidy it directly — no history
    assert!(
        !m.history.can_undo(),
        "the fixture starts with a clean history"
    );

    arrange(&mut m);

    assert!(
        !m.history.can_undo(),
        "arranging an already-tidy graph is not an edit — no undo step"
    );
}
