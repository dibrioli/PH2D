//! Bypass/mute (H) seam test — the shell half of the graph editor's switch-off. Declared by the
//! parent as a `#[path]` sibling, so `super` is `render_loop::motion_bridge`. Drives `SetBypass`
//! through the REAL intent funnel (`apply_graph_intents`), not by poking the doc.

use super::apply_graph_intents;
use crate::motion_state::MotionState;
use ph2d_nodegraph::graph::NodeId;
use ph2d_panel_motion_graph::{GraphIntent, drain_intents, push_intent};

/// A state with one real node, cooked once so `is_dirty` is `false` — a later `true` can only have
/// come from the mute under test.
fn cooked_with_a_node() -> (MotionState, NodeId) {
    let mut m = MotionState::new();
    let a = m.doc.graph.add_node("motion.grid");
    m.pump.pump(
        &m.doc.graph,
        &m.registry,
        &m.sinks,
        0,
        0.0,
        m.default_uv_rect,
        m.default_size,
    );
    assert!(!m.pump.is_dirty(), "the pump settles after a cook");
    (m, a)
}

fn set_bypass(m: &mut MotionState, nodes: Vec<u32>, on: bool) {
    let _ = drain_intents();
    push_intent(GraphIntent::SetBypass { nodes, on });
    apply_graph_intents(
        m,
        &mut ph2d_core::Playhead::default(),
        &mut ph2d_editor::ToastQueue::default(),
        &mut ph2d_editor::screens::layout::CenterSplit::None,
    );
}

/// **Muting a node through the funnel switches it off, re-cooks, and is ONE undo step.** Bypass is
/// SEMANTIC (a muted node cooks a passthrough), so — unlike the UI-only arrange/backdrop intents —
/// it marks the cook dirty; and Ctrl+Z un-mutes. FALSIFIED by not calling `set_bypassed` (the node
/// stays on), by dropping `mark_dirty` (no re-cook), or by not bracketing the undo.
#[test]
fn muting_a_node_switches_it_off_recooks_and_is_one_undo_step() {
    let (mut m, a) = cooked_with_a_node();
    assert!(!m.doc.graph.node_bypassed(a));

    set_bypass(&mut m, vec![a.0], true);
    assert!(m.doc.graph.node_bypassed(a), "the node is muted");
    assert!(m.pump.is_dirty(), "muting is semantic -- it re-cooks");
    assert!(m.history.can_undo(), "muting is exactly one undo step");
    let back = m.history.undo(&m.doc).expect("one undo step");
    assert!(!back.graph.node_bypassed(a), "Ctrl+Z un-mutes");
}

/// **A bypass on a non-existent node is inert.** A subgraph CARD's id is tagged, and a stale
/// selection can name a deleted node; muting a phantom must change nothing — and never emit a `y`
/// record the loader would reject. FALSIFIED by dropping the `graph.node(nid).is_some()` filter:
/// the phantom bypass would change the doc and push a spurious undo step.
#[test]
fn muting_a_phantom_id_is_inert() {
    let (mut m, _a) = cooked_with_a_node();
    set_bypass(&mut m, vec![999_999], true);
    assert!(
        !m.history.can_undo(),
        "a bypass on a non-existent node is not an edit"
    );
    assert!(!m.pump.is_dirty(), "and it does not re-cook");
}
