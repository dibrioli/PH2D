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

/// **Muting a GROUP bypasses it AS A UNIT** — the H verb / the right-click Mute sets the GROUP's
/// own bypass (input[0] → output[0], the interior skipped; the cook rewires a throwaway clone),
/// NOT a node-bypass on each member. This is the Blender/Nuke idiom (Enio's choice): muting a
/// group is not muting each member (enter the group and mute nodes for that). The members are
/// LEFT ALONE. One undo step; re-cooks (semantic). FALSIFIED by muting the members instead of the
/// group, or by not setting the group flag at all.
#[test]
fn muting_a_group_bypasses_it_as_a_unit() {
    use ph2d_motion_doc::subgraph::Subgraph;
    let (mut m, _a) = cooked_with_a_node();
    let b = m.doc.graph.add_node("motion.grid");
    let c = m.doc.graph.add_node("motion.grid");
    m.doc.subgraphs.push(Subgraph {
        id: 1,
        parent: None,
        x: 0.0,
        y: 0.0,
        title: "Rig".into(),
    });
    m.doc.members.insert(b, 1);
    m.doc.members.insert(c, 1);

    // Mute the CARD (its tagged view id).
    let card = super::subgraph::view_id(1);
    set_bypass(&mut m, vec![card], true);
    assert!(
        m.doc.subgraph_bypassed(1),
        "the group is bypassed as a unit"
    );
    assert!(
        !m.doc.graph.node_bypassed(b) && !m.doc.graph.node_bypassed(c),
        "the MEMBERS are left alone -- muting a group is not muting each member"
    );
    assert!(
        m.pump.is_dirty(),
        "a group bypass is semantic -- it re-cooks"
    );
    assert!(m.history.can_undo(), "and it is one undo step");

    // Un-mute clears the group flag (no phantom `yg` left to serialize).
    set_bypass(&mut m, vec![card], false);
    assert!(
        !m.doc.subgraph_bypassed(1),
        "un-muting clears the group flag"
    );
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

    // A tagged CARD id whose group does not exist is inert too (the `find` guard) — a phantom
    // must never reach `bypassed_subgraphs`, or it would emit a `yg` the loader rejects.
    let phantom_card = super::subgraph::view_id(4242);
    set_bypass(&mut m, vec![phantom_card], true);
    assert!(
        !m.history.can_undo() && m.doc.bypassed_subgraphs.is_empty(),
        "a phantom card bypass is inert -- no `yg` for a group that does not exist"
    );
}

/// **Dissolving or deleting a bypassed group forgets its bypass** — a dangling id left in
/// `bypassed_subgraphs` would emit a `yg` the loader rejects. Both removal sites clean up, next to
/// where they already drop membership. FALSIFIED by dropping the cleanup at either site.
#[test]
fn dissolving_a_bypassed_group_forgets_its_bypass() {
    use ph2d_motion_doc::subgraph::Subgraph;
    let sg = |m: &mut MotionState| {
        let b = m.doc.graph.add_node("motion.grid");
        m.doc.subgraphs.push(Subgraph {
            id: 1,
            parent: None,
            x: 0.0,
            y: 0.0,
            title: "Rig".into(),
        });
        m.doc.members.insert(b, 1);
        m.doc.set_subgraph_bypassed(1, true);
    };

    let (mut m, _a) = cooked_with_a_node();
    sg(&mut m);
    super::subgraph::ungroup(&mut m, 1);
    assert!(
        m.doc.bypassed_subgraphs.is_empty(),
        "ungroup drops the group's bypass"
    );

    let (mut m, _a) = cooked_with_a_node();
    sg(&mut m);
    super::subgraph::delete_deep(&mut m, 1);
    assert!(
        m.doc.bypassed_subgraphs.is_empty(),
        "delete drops the group's bypass"
    );
}
