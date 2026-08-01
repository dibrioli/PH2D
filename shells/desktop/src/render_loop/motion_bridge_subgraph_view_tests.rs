//! **A INTERFACE de um card, derivada dos cruzamentos** — filho do
//! `motion_bridge_subgraph_tests` (cap de LOC da shell, como o
//! `..._ports_tests` ao lado).
//!
//! ⚠️ O corte não foi escolhido: o arquivo pai já **nomeava esta seção** num banner
//! (`── The interface, derived from the crossings ──`), e ela é o que separa *o que
//! um card MOSTRA em cada nível* de *o que os verbos FAZEM ao documento*.
//!
//! ⚠️ FILHO e não irmão, de propósito: `flat()` e `cook()` são as fixtures do pai e
//! têm de continuar sendo UMA porta — `use super::*` as alcança, e uma cópia
//! divergiria no dia em que o boot document mudasse.

use super::*;

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
    super::super::subgraph::group(&mut m, aging.clone());
    let sid = m.doc.subgraphs[0].id;
    let ports = super::super::subgraph::card_ports(&m, sid);
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
    let (n, p) =
        super::super::subgraph::resolve_port(&m, super::super::subgraph::view_id(sid), 0, true)
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
    assert_eq!(card.id, super::super::subgraph::view_id(sid));
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

/// **A group card draws muted only when EVERY member is muted** — muting a whole group (the H
/// verb / the right-click Mute) has to be VISIBLE on the card, and the panel reads exactly this
/// to decide mute-vs-unmute (so the toggle can unmute a fully-muted group). FALSIFIED by the old
/// `bypassed: false`: a muted group looked identical to a live one, and the toggle could only
/// ever mute, never unmute.
#[test]
fn a_group_card_is_muted_only_when_all_of_it_is() {
    let mut m = MotionState::new();
    let sid = m.doc.subgraphs[0].id;
    let inside = subgraph::member_nodes_deep(&m.doc.subgraphs, &m.doc.members, sid);
    assert!(inside.len() >= 2, "the demo group has members to mute");

    let card_bypassed = |m: &MotionState| {
        let mut snap = ph2d_panel_motion_graph::snapshot_from(&m.doc.graph, &m.registry);
        fold::fold(m, &mut snap);
        snap.nodes
            .iter()
            .find(|n| n.id == super::super::subgraph::view_id(sid))
            .expect("the card is drawn")
            .bypassed
    };

    assert!(!card_bypassed(&m), "a live group is not muted");
    for n in &inside[..inside.len() - 1] {
        m.doc.graph.set_bypassed(*n, true);
    }
    assert!(
        !card_bypassed(&m),
        "SOME members muted is not the group muted"
    );
    m.doc.graph.set_bypassed(*inside.last().unwrap(), true);
    assert!(
        card_bypassed(&m),
        "EVERY member muted -> the card draws muted"
    );
}
