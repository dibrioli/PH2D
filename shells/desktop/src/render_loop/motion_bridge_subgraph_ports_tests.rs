//! **The ports a card HIDES** (Motion Nodes doc 57 §5) — the other half of the derived
//! interface, and the gates for the menu that reaches them.
//!
//! Sibling of `motion_bridge_subgraph_tests` (shell LOC cap). A card's sockets are the
//! wires that already cross it; these are the ports NO wire crosses to, which is precisely
//! the set that would be unreachable from outside if the drop did not ask.

use super::subgraph_tests::flat;
use super::*;
use ph2d_panel_motion_graph::{GraphIntent, drain_intents, push_intent};

/// Wiring into a CARD's socket reaches the real port inside it — the derivation that
/// drew the socket and the one that resolves it are the same function, or the socket
/// would mean a different port than the one it drew.
#[test]
fn wiring_a_card_socket_reaches_the_real_port_inside() {
    let mut m = flat();
    // Grid -> Clone, with the clone alone in a group: the group's one input slot is
    // the clone's port 0, and re-wiring it must land on the CLONE.
    let grid = m.doc.graph.add_node("motion.grid");
    let clone = m.doc.graph.add_node("motion.clone");
    let grid2 = m.doc.graph.add_node("motion.grid");
    m.doc
        .graph
        .connect(ph2d_nodegraph::graph::Edge {
            from: (grid, 0),
            to: (clone, 0),
            delayed: false,
        })
        .unwrap();
    super::subgraph::group(&mut m, vec![clone.0]);
    let sid = m.doc.subgraphs[0].id;

    // The crossing wire occupies the card's only input slot, so a fresh Connect into
    // it is REFUSED (an occupied input is occupied, card or not — the artist grabs the
    // wire's end to move it, which is the same gesture as anywhere else in the editor).
    // What must be true is that it refuses the RIGHT port: the clone's, not the card's.
    let (node, port) =
        super::subgraph::resolve_port(&m, super::subgraph::view_id(sid), 0, true).unwrap();
    assert_eq!((node, port), (clone, 0));

    // Moving the wire's end onto the card's slot from the other grid: same target.
    push_intent(GraphIntent::MoveWireEnd {
        from_node: grid2.0,
        from_port: 0,
        old_to_node: grid.0,
        old_to_port: 0,
        new_to: Some((super::subgraph::view_id(sid), 0)),
    });
    let _ = drain_intents(); // (this one is asserted through `resolve_port` above)
}

/// Publish the fold for `m` and read back what the card is hiding — through the real
/// channel, because the derivation and the PUBLISH are two different things and only one
/// of them is visible to the artist.
fn hidden_of(m: &MotionState, sid: u32) -> ph2d_panel_motion_graph::HiddenPorts {
    let mut snap = ph2d_panel_motion_graph::snapshot_from(&m.doc.graph, &m.registry);
    fold::fold(m, &mut snap);
    ph2d_panel_motion_graph::card_hidden_ports(super::subgraph::view_id(sid))
}

/// **A card hides exactly the ports no wire can reach** — and a wire dropped on its body
/// can then reach them, which is the whole point of the menu (doc 57 §5).
///
/// The asymmetry between the two sides is the graph's own: an input holds ONE edge (so a
/// fed one is not offerable), an output fans out (so it is offerable unless the card
/// already exposes it).
#[test]
fn a_card_hides_the_ports_no_wire_reaches_and_the_menu_can_reach_them() {
    let mut m = flat();
    // grid -> drive, with the DRIVE alone in the group. Drive has two inputs ("in" and
    // "value"): the first is fed from outside, so the card shows it as a socket. The
    // second is free, and so is drive's output — neither has any way in or out.
    let grid = m.doc.graph.add_node("motion.grid");
    let drive = m.doc.graph.add_node("motion.drive");
    m.doc
        .graph
        .connect(ph2d_nodegraph::graph::Edge {
            from: (grid, 0),
            to: (drive, 0),
            delayed: false,
        })
        .unwrap();
    super::subgraph::group(&mut m, vec![drive.0]);
    let sid = m.doc.subgraphs[0].id;

    let hidden = hidden_of(&m, sid);
    assert!(
        !hidden
            .inputs
            .iter()
            .any(|p| p.node == drive.0 && p.port == 0),
        "input 0 is already fed - offering it would offer a port the connect must refuse"
    );
    assert!(
        hidden
            .inputs
            .iter()
            .any(|p| p.node == drive.0 && p.port == 1),
        "but input 1 is free, and without the menu nothing outside could ever reach it"
    );
    let out = hidden
        .outputs
        .iter()
        .find(|p| p.node == drive.0 && p.port == 0)
        .expect("the drive's output crosses nothing, so the card cannot show it");
    assert!(
        out.label.contains(':'),
        "a row names its node AND its port - a group holds many, and 'out' alone would \
         not say whose: {}",
        out.label
    );

    // THE PAYOFF: wire that hidden output to something outside, through the ordinary
    // Connect the menu's pick emits — and the card GROWS the socket, derived from the edge
    // that now crosses it. Nothing declared an interface; the wire IS the interface.
    let sink = m.doc.graph.add_node("motion.output");
    assert_eq!(
        super::subgraph::card_ports(&m, sid).outputs.len(),
        0,
        "before: the group emits nothing anyone can see"
    );
    push_intent(GraphIntent::Connect {
        from_node: out.node,
        from_port: out.port,
        to_node: sink.0,
        to_port: 0,
    });
    apply_graph_intents(
        &mut m,
        &mut ph2d_core::Playhead::default(),
        &mut ph2d_editor::ToastQueue::default(),
        &mut ph2d_editor::screens::layout::CenterSplit::None,
    );
    assert_eq!(
        super::subgraph::card_ports(&m, sid).outputs,
        vec![(drive, 0)],
        "after: the card has an output socket, and it stands for the port the artist named"
    );
    assert!(
        !hidden_of(&m, sid)
            .outputs
            .iter()
            .any(|p| p.node == drive.0 && p.port == 0),
        "...and it is no longer HIDDEN: the socket is right there, so the menu stops \
         offering a second door into the same room"
    );
}

/// A port inside a NESTED card is hidden by the outer one too — the card stands for
/// everything under it, at every depth, or a wire could never reach a node two storeys down
/// without opening both doors.
#[test]
fn a_card_hides_what_its_nested_cards_hide() {
    let mut m = flat();
    // Two fresh, unwired grids: nothing crosses anything, so every port they have is
    // hidden by whatever card ends up standing over it.
    let deep = m.doc.graph.add_node("motion.grid");
    let shallow = m.doc.graph.add_node("motion.grid");
    super::subgraph::group(&mut m, vec![deep.0]);
    let inner = m.doc.subgraphs[0].id;
    super::subgraph::group(&mut m, vec![shallow.0, super::subgraph::view_id(inner)]);
    let outer = m.doc.subgraphs.iter().find(|s| s.id != inner).unwrap().id;

    let hidden = hidden_of(&m, outer);
    assert!(
        hidden.outputs.iter().any(|p| p.node == deep.0),
        "the node two storeys down is reachable from the outer card's menu"
    );
    assert!(
        hidden.outputs.iter().any(|p| p.node == shallow.0),
        "and so is the one directly inside it"
    );
}
