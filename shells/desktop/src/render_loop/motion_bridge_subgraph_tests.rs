//! Subgraph gates (Motion Nodes doc 57). Headless, driving the REAL bridge — the
//! intents the panel pushes, applied to the real document, cooked by the real pump.
//!
//! The first one is the whole design in a single assertion.

use super::*;
use crate::motion_state::MotionState;
use ph2d_motion_doc::subgraph;
use ph2d_nodegraph::graph::NodeId;
use ph2d_panel_motion_graph::{GraphIntent, NodeViewKind, drain_intents, push_intent};

/// The fixed timestep the shell cooks at (mirrors `motion_state_tests::run`).
const FIXED_DT: f64 = 1.0 / 60.0;

/// Cook the boot document forward `ticks` fixed steps and return the instance buffer
/// — the actual pixels-to-be, which is the only output that can settle an argument
/// about whether the cook changed.
///
/// The two traps `motion_state_tests::run` documents apply here too, and they are the
/// reason this is a copy of it rather than something cleverer: **the clock is born
/// PAUSED** (without `play()` the tick never leaves 0), and **the shell cooks tick 0
/// BEFORE advancing** (advancing first drops the pump into the scrub path with an
/// empty ring, and nothing comes out).
fn cook(motion: &mut MotionState, ticks: u64) -> Vec<u8> {
    let mut playhead = ph2d_core::Playhead::new(FIXED_DT);
    playhead.play();
    motion.sinks = output_nodes(&motion.doc.graph);
    let scopes = ph2d_node_motion_time_remap::time_scopes(&motion.doc.graph, &motion.registry);
    for step in 0..=ticks {
        if step > 0 {
            playhead.advance();
        }
        let target = motion_tick(&playhead, FIXED_DT);
        for tick in ticks_owed(motion.pump.last_cooked_tick(), target) {
            motion.pump.advance_or_scrub_scoped(
                &motion.doc.graph,
                &motion.registry,
                &motion.sinks,
                tick,
                |t| t as f64 * FIXED_DT,
                motion.default_uv_rect,
                motion.default_size,
                &scopes,
            );
        }
    }
    // The BYTES of the instance buffer: the render instance type is private to the
    // eval crate, and bytes are what "byte-identical" means anyway.
    bytemuck::cast_slice::<_, u8>(&motion.pump.instances).to_vec()
}

/// A `MotionState` with the boot document's group already dissolved — the same graph,
/// flat. Every test that wants to group something starts from here, so that "before"
/// and "after" differ by exactly the grouping.
pub(super) fn flat() -> MotionState {
    let mut m = MotionState::new();
    m.doc.subgraphs.clear();
    m.doc.members.clear();
    m
}

// ── THE gate ────────────────────────────────────────────────────────────────

/// **Grouping is a byte-identical no-op on the cook.**
///
/// This is the entire claim of doc 57's design: the graph stays FLAT and the subgraph
/// is a fold in the VIEW, so folding the snow's age chain into a card cannot change a
/// single flake. If this ever goes red, the feature has stopped being a fold and
/// become a lie about what the document computes — and the lie would be invisible in
/// every other gate, because every other gate is about the editor.
///
/// Falsifiable, and it was falsified on purpose: make `group` drop one crossing edge
/// (or re-point it, or renumber a node) and the buffers diverge on the first tick.
#[test]
fn grouping_never_changes_the_cook() {
    let aging: Vec<u32> = {
        // The six nodes the boot document folds — taken from the document itself, so
        // this cannot drift from what ships.
        let m = MotionState::new();
        m.doc.members.keys().map(|n| n.0).collect()
    };
    assert_eq!(
        aging.len(),
        6,
        "the boot document ships one card of six nodes"
    );

    let mut before = flat();
    let flat_out = cook(&mut before, 40);
    assert!(
        !flat_out.is_empty(),
        "the snow must actually be falling, or this gate proves nothing at all"
    );

    // Group them through the REAL intent path (not by poking the doc), then cook the
    // same 40 ticks from a fresh pump.
    let mut after = flat();
    push_intent(GraphIntent::GroupSelection { nodes: aging });
    apply_graph_intents(
        &mut after,
        &mut ph2d_core::Playhead::default(),
        &mut ph2d_editor::ToastQueue::default(),
        &mut ph2d_editor::screens::layout::CenterSplit::None,
    );
    assert_eq!(after.doc.subgraphs.len(), 1, "the group was created");
    let grouped_out = cook(&mut after, 40);

    assert_eq!(
        flat_out.len(),
        grouped_out.len(),
        "the same number of flakes"
    );
    assert_eq!(
        flat_out, grouped_out,
        "grouping changed what the graph COOKS - the fold is not a fold"
    );
}

/// Grouping does not even DIRTY the cook: it is document state (undoable) but the
/// graph is untouched, exactly like a backdrop. A `mark_dirty` here would re-cook a
/// heavy graph on a gesture that changed nothing about it.
#[test]
fn grouping_never_re_cooks() {
    let mut m = flat();
    cook(&mut m, 1);
    assert!(!m.pump.is_dirty(), "a settled pump is clean");
    let ids: Vec<u32> = m.doc.graph.nodes().iter().take(3).map(|n| n.id.0).collect();
    subgraph::next_id(&m.doc.subgraphs); // (no-op; keeps the import honest)
    super::subgraph::group(&mut m, ids);
    assert!(
        !m.pump.is_dirty(),
        "grouping re-cooked the graph - it cannot depend on a fold"
    );
}

// ── The interface, derived from the crossings ───────────────────────────────

/// **A card's ports ARE the edges that cross it** — and one source port feeding two
/// outside targets is ONE socket, not two (a port is a port; drawing it twice would
/// say the group had two of them).
#[test]
fn a_card_exposes_exactly_the_crossing_edges() {
    let mut m = flat();
    // The boot document's age chain: one edge in (collide -> lifetime), one out
    // (drive -> falloff), and `value.attribute` feeding THREE consumers, all inside.
    let aging: Vec<u32> = MotionState::new().doc.members.keys().map(|n| n.0).collect();
    super::subgraph::group(&mut m, aging.clone());
    let sid = m.doc.subgraphs[0].id;
    let ports = super::subgraph::card_ports(&m, sid);
    assert_eq!(
        ports.inputs.len(),
        1,
        "one wire crosses in, so the card has one input"
    );
    assert_eq!(
        ports.outputs.len(),
        1,
        "one wire crosses out, so the card has one output"
    );
    // ...and the slot resolves back to the REAL port it stands for.
    let (n, p) = super::subgraph::resolve_port(&m, super::subgraph::view_id(sid), 0, true)
        .expect("input slot 0 resolves");
    assert!(
        aging.contains(&n.0),
        "the card's input slot names a node INSIDE it"
    );
    assert_eq!(p, 0);
}

/// The fold shows: the level's own nodes, one card per child group, and the outsiders
/// that touch the boundary as GHOSTS — and nothing else. An unrelated node from
/// another part of the graph must not leak into the room.
#[test]
fn entering_a_group_shows_its_members_and_the_boundary_ghosts() {
    let mut m = MotionState::new(); // the boot document, group and all
    let sid = m.doc.subgraphs[0].id;
    m.level = Some(sid);

    let mut snap = ph2d_panel_motion_graph::snapshot_from(&m.doc.graph, &m.registry);
    fold::fold(&m, &mut snap);

    let members: Vec<u32> = m
        .doc
        .members
        .iter()
        .filter(|(_, s)| **s == sid)
        .map(|(n, _)| n.0)
        .collect();
    for id in &members {
        assert!(
            snap.nodes
                .iter()
                .any(|n| n.id == *id && n.kind == NodeViewKind::Node),
            "member {id} is drawn as itself inside its own group"
        );
    }
    let ghosts: Vec<&_> = snap
        .nodes
        .iter()
        .filter(|n| n.kind == NodeViewKind::Ghost)
        .collect();
    assert_eq!(
        ghosts.len(),
        2,
        "exactly the two nodes across the boundary that the wires reach: the collide \
         that feeds the chain and the falloff it feeds"
    );
    // The room holds ONLY its members and those two ghosts — the other eleven nodes of
    // the snow are somewhere else entirely.
    assert_eq!(snap.nodes.len(), members.len() + 2);
    assert_eq!(
        snap.breadcrumb.len(),
        2,
        "Root / Age & Fade - two crumbs, and the first one is the way out"
    );
    assert_eq!(snap.breadcrumb[1].title, "Age & Fade");
}

/// At the ROOT the card stands in for its contents: the members are gone from the
/// view, one card is there instead, and the wires that used to reach them now land on
/// its sockets.
#[test]
fn at_the_root_the_card_stands_in_for_its_contents() {
    let m = MotionState::new();
    let sid = m.doc.subgraphs[0].id;
    let mut snap = ph2d_panel_motion_graph::snapshot_from(&m.doc.graph, &m.registry);
    fold::fold(&m, &mut snap);

    for member in m.doc.members.keys() {
        assert!(
            !snap.nodes.iter().any(|n| n.id == member.0),
            "a folded member is not drawn at the parent level"
        );
    }
    let card = snap
        .nodes
        .iter()
        .find(|n| n.kind == NodeViewKind::Subgraph)
        .expect("the card is drawn");
    assert_eq!(card.id, super::subgraph::view_id(sid));
    assert_eq!(card.display_name, "Age & Fade");
    assert_eq!(card.readout.as_deref(), Some("6 nodes"));
    assert_eq!((card.inputs.len(), card.outputs.len()), (1, 1));
    // Every wire that touches the card touches it on a slot that exists.
    for e in &snap.edges {
        if e.to_node == card.id {
            assert!((e.to_port as usize) < card.inputs.len());
        }
        if e.from_node == card.id {
            assert!((e.from_port as usize) < card.outputs.len());
        }
    }
    // Nothing inside the group is wired to anything, as far as this level knows.
    for member in m.doc.members.keys() {
        assert!(
            !snap
                .edges
                .iter()
                .any(|e| e.from_node == member.0 || e.to_node == member.0),
            "a wire to a folded member survived the fold"
        );
    }
}

// ── The verbs ──────────────────────────────────────────────────────────────

/// Ungroup puts everything back, exactly — the document round-trips.
#[test]
fn ungroup_restores_the_document_it_grouped() {
    let mut m = flat();
    let before = m.doc.clone();
    let ids: Vec<u32> = m.doc.graph.nodes().iter().take(4).map(|n| n.id.0).collect();
    super::subgraph::group(&mut m, ids);
    assert_eq!(m.doc.subgraphs.len(), 1);
    let sid = m.doc.subgraphs[0].id;
    super::subgraph::ungroup(&mut m, sid);
    assert_eq!(
        m.doc, before,
        "group then ungroup is the identity on the document"
    );
}

/// **Ungroup hands the freed contents back as the selection** (Enio, smoke 2026-07-13).
///
/// The artist had a cluster in their hand — the card — and dissolving it must not empty
/// their hand: what they were holding is now the members, and the drag or the Ctrl+G that
/// naturally follows has to find them there. (Blender's Ungroup leaves the freed nodes
/// selected for the same reason.) The freed set is read BEFORE the dissolve and handed
/// back AFTER it, because dissolving the room you stand in changes level, and a level
/// change clears the selection.
#[test]
fn ungroup_hands_the_freed_contents_back_as_the_selection() {
    let mut m = flat();
    let n0 = m.doc.graph.nodes()[0].id;
    let n1 = m.doc.graph.nodes()[1].id;
    // A nest: `inner` holds n0; `outer` holds n1 AND inner. Dissolving `outer` spills a
    // node and a CARD onto the root — both are things the artist can now grab, so both
    // come back, and the card comes back in the id space the view speaks (tagged).
    super::subgraph::group(&mut m, vec![n0.0]);
    let inner = m.doc.subgraphs[0].id;
    super::subgraph::group(&mut m, vec![n1.0, super::subgraph::view_id(inner)]);
    let outer = m.doc.subgraphs.iter().find(|s| s.id != inner).unwrap().id;

    super::subgraph::ungroup(&mut m, outer);
    let mut got = ph2d_panel_motion_graph::pending_graph_selection()
        .expect("ungroup must hand a selection back - an empty hand is the bug");
    got.sort_unstable();
    let mut want = vec![n1.0, super::subgraph::view_id(inner)];
    want.sort_unstable();
    assert_eq!(
        got, want,
        "the freed member AND the freed card, and nothing else - n0 is still folded \
         inside `inner`, so it is not something the artist can grab"
    );

    // …and from INSIDE: dissolving the room you are standing in walks you out and STILL
    // leaves the contents in your hand (the level change clears the selection first).
    super::subgraph::set_level(&mut m, Some(inner));
    super::subgraph::ungroup(&mut m, inner);
    assert_eq!(
        m.level, None,
        "the room is gone, so you are back at the root"
    );
    assert_eq!(
        ph2d_panel_motion_graph::pending_graph_selection(),
        Some(vec![n0.0]),
        "the node that was in the dissolved room is selected, not nothing"
    );
}

/// **Deleting a card deletes what is inside it** — at every depth (Nuke: "the original
/// nodes are replaced with the Group node", so the card IS them).
#[test]
fn deleting_a_card_deletes_its_members_at_every_depth() {
    let mut m = flat();
    let n0 = m.doc.graph.nodes()[0].id;
    let n1 = m.doc.graph.nodes()[1].id;
    let before = m.doc.graph.nodes().len();
    // Nest: inner holds n0; outer holds n1 AND inner.
    super::subgraph::group(&mut m, vec![n0.0]);
    let inner = m.doc.subgraphs[0].id;
    super::subgraph::group(&mut m, vec![n1.0, super::subgraph::view_id(inner)]);
    let outer = m.doc.subgraphs.iter().find(|s| s.id != inner).unwrap().id;
    assert_eq!(
        subgraph::find(&m.doc.subgraphs, inner).unwrap().parent,
        Some(outer),
        "grouping a card re-parents it - that is how a nest gets a second storey"
    );

    apply_delete_selection(
        &mut m,
        vec![super::subgraph::view_id(outer)],
        &mut ph2d_editor::ToastQueue::default(),
    );
    assert!(m.doc.subgraphs.is_empty(), "both groups are gone");
    assert!(m.doc.members.is_empty(), "no membership outlived its group");
    assert!(m.doc.graph.node(n0).is_none(), "the nested node died too");
    assert!(m.doc.graph.node(n1).is_none());
    assert_eq!(m.doc.graph.nodes().len(), before - 2);
}

/// A node minted while the artist is INSIDE a group belongs to that group. Without
/// this the node would land at the root and **vanish the instant it was created** —
/// the artist adds a node and nothing appears.
#[test]
fn a_node_added_inside_a_group_is_a_member_of_it() {
    let mut m = flat();
    let ids: Vec<u32> = m.doc.graph.nodes().iter().take(2).map(|n| n.id.0).collect();
    super::subgraph::group(&mut m, ids);
    let sid = m.doc.subgraphs[0].id;
    super::subgraph::set_level(&mut m, Some(sid));

    let before: Vec<NodeId> = m.doc.graph.nodes().iter().map(|n| n.id).collect();
    push_intent(GraphIntent::AddNode {
        type_name: "motion.grid",
        x: 0.0,
        y: 0.0,
    });
    apply_graph_intents(
        &mut m,
        &mut ph2d_core::Playhead::default(),
        &mut ph2d_editor::ToastQueue::default(),
        &mut ph2d_editor::screens::layout::CenterSplit::None,
    );
    let fresh = m
        .doc
        .graph
        .nodes()
        .iter()
        .map(|n| n.id)
        .find(|id| !before.contains(id))
        .expect("the node was added");
    assert_eq!(
        m.doc.members.get(&fresh),
        Some(&sid),
        "a node added inside a group must be IN it, or it is invisible where it was born"
    );

    // ...and it is drawn in the room it was born in.
    let mut snap = ph2d_panel_motion_graph::snapshot_from(&m.doc.graph, &m.registry);
    fold::fold(&m, &mut snap);
    assert!(snap.nodes.iter().any(|n| n.id == fresh.0));
}

/// The level can stop existing under the artist's feet (an undo that unmakes the
/// group). It falls back to the root rather than showing a canvas nothing can leave.
#[test]
fn a_vanished_level_falls_back_to_the_root() {
    let mut m = flat();
    let ids: Vec<u32> = m.doc.graph.nodes().iter().take(2).map(|n| n.id.0).collect();
    super::subgraph::group(&mut m, ids);
    let sid = m.doc.subgraphs[0].id;
    super::subgraph::set_level(&mut m, Some(sid));
    assert_eq!(m.level, Some(sid));

    // Undo the grouping (the same call Ctrl+Z makes).
    if let Some(prev) = m.history.undo(&m.doc) {
        m.doc = prev;
    }
    super::subgraph::clamp_level(&mut m);
    assert_eq!(m.level, None, "the room is gone; stand at the root");
    let _ = drain_intents(); // (the level change queued a selection request, not an intent)
}

/// **Probing a collapsed card reads what the group EMITS, and the HUD hangs off the
/// CARD.** The probe resolves to a node inside the group — a node this level does not
/// draw — so without the fold re-pointing it, the artist would arm P, click the card,
/// and the editor would show them nothing at all: a silent no-op, and the worst kind
/// (the gesture LOOKED like it worked).
#[test]
fn probing_a_card_reads_what_it_emits_and_the_hud_hangs_off_the_card() {
    let mut m = MotionState::new();
    let sid = m.doc.subgraphs[0].id;
    let card = super::subgraph::view_id(sid);

    // The card's first output source — a real node, inside the group.
    let emitter = super::subgraph::probe_target(&m, card).expect("the group emits something");
    assert!(
        m.doc.members.contains_key(&emitter),
        "what a card emits comes from a node INSIDE it"
    );

    push_intent(GraphIntent::SetProbe { node: Some(card) });
    apply_graph_intents(
        &mut m,
        &mut ph2d_core::Playhead::default(),
        &mut ph2d_editor::ToastQueue::default(),
        &mut ph2d_editor::screens::layout::CenterSplit::None,
    );
    assert_eq!(
        m.probe,
        Some(emitter),
        "the probe points at the real node, because that is what the cook can read"
    );

    // ...and the published view puts the reading back ON the card, which is the only
    // thing at this level the HUD could possibly hang off.
    cook(&mut m, 2);
    let mut snap = ph2d_panel_motion_graph::snapshot_from(&m.doc.graph, &m.registry);
    snap.probe = super::edit::sample_probe(&mut m, 0.0, None);
    fold::fold(&m, &mut snap);
    let probe = snap.probe.expect("the probe published a reading");
    assert_eq!(probe.node, card, "the HUD is drawn beside the CARD");
    assert!(
        snap.nodes.iter().any(|n| n.id == probe.node),
        "and the card it points at is actually in the view - a HUD anchored to \
         something that is not drawn is a HUD that is not drawn"
    );
}

/// **A ghost can be MOVED but not DELETED** (Enio, smoke 2026-07-13). It is a real
/// node with one position, so tidying it from inside the group is legitimate — but
/// deleting it would remove a node from a canvas the artist is not looking at, and
/// that is refused, out loud (a toast), never in silence.
#[test]
fn a_ghost_moves_but_cannot_be_deleted_from_inside_the_group() {
    let mut m = MotionState::new();
    let sid = m.doc.subgraphs[0].id;
    m.level = Some(sid);

    // The two nodes across the boundary (the collide that feeds the chain, the falloff
    // it feeds) — the ones the artist sees as "the first and last node of the group".
    let mut snap = ph2d_panel_motion_graph::snapshot_from(&m.doc.graph, &m.registry);
    fold::fold(&m, &mut snap);
    let ghost = snap
        .nodes
        .iter()
        .find(|n| n.kind == NodeViewKind::Ghost)
        .expect("the boundary is drawn")
        .id;
    let before = m.doc.graph.pos(NodeId(ghost)).unwrap();

    // MOVE: it tracks the cursor like any other card.
    push_intent(GraphIntent::MoveNodes {
        nodes: vec![ghost],
        dx: 12.0,
        dy: 7.0,
    });
    // DELETE: refused, and the node is still there.
    push_intent(GraphIntent::DeleteSelection { nodes: vec![ghost] });
    let mut toasts = ph2d_editor::ToastQueue::default();
    apply_graph_intents(
        &mut m,
        &mut ph2d_core::Playhead::default(),
        &mut toasts,
        &mut ph2d_editor::screens::layout::CenterSplit::None,
    );

    assert_eq!(
        m.doc.graph.pos(NodeId(ghost)).unwrap(),
        ph2d_nodegraph::graph::Pos {
            x: before.x + 12.0,
            y: before.y + 7.0
        },
        "a ghost is a node, and a node can be tidied"
    );
    assert!(
        m.doc.graph.node(NodeId(ghost)).is_some(),
        "but it cannot be deleted from a room it does not live in"
    );
}

/// Moving a card carries its members — otherwise entering it would land the artist on
/// empty canvas, a screen away from the card they just dragged.
#[test]
fn moving_a_card_carries_everything_inside_it() {
    let mut m = flat();
    let n0 = m.doc.graph.nodes()[0].id;
    let n1 = m.doc.graph.nodes()[1].id;
    super::subgraph::group(&mut m, vec![n0.0, n1.0]);
    let sid = m.doc.subgraphs[0].id;
    let (before0, before1) = (m.doc.graph.pos(n0).unwrap(), m.doc.graph.pos(n1).unwrap());
    let card = m.doc.subgraphs[0].clone();

    push_intent(GraphIntent::MoveNodes {
        nodes: vec![super::subgraph::view_id(sid)],
        dx: 25.0,
        dy: -10.0,
    });
    apply_graph_intents(
        &mut m,
        &mut ph2d_core::Playhead::default(),
        &mut ph2d_editor::ToastQueue::default(),
        &mut ph2d_editor::screens::layout::CenterSplit::None,
    );

    let s = &m.doc.subgraphs[0];
    assert_eq!((s.x, s.y), (card.x + 25.0, card.y - 10.0), "the card moved");
    assert_eq!(
        m.doc.graph.pos(n0).unwrap(),
        ph2d_nodegraph::graph::Pos {
            x: before0.x + 25.0,
            y: before0.y - 10.0
        },
        "and so did what is inside it"
    );
    assert_eq!(
        m.doc.graph.pos(n1).unwrap(),
        ph2d_nodegraph::graph::Pos {
            x: before1.x + 25.0,
            y: before1.y - 10.0
        }
    );
}
