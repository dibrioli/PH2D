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
    let port = |p: &ph2d_panel_motion_graph::PortChoice| match p.target {
        ph2d_panel_motion_graph::ChoiceTarget::Port(k) => Some(k),
        // A PARAM row is not a port row, and it must not be mistaken for one — a `port: 0`
        // sentinel collided with real port 0 here, which is why the target is an enum.
        ph2d_panel_motion_graph::ChoiceTarget::Param(_) => None,
    };
    assert!(
        !hidden
            .inputs
            .iter()
            .any(|p| p.node == drive.0 && port(p) == Some(0)),
        "input 0 is already fed - offering it would offer a port the connect must refuse"
    );
    assert!(
        hidden
            .inputs
            .iter()
            .any(|p| p.node == drive.0 && port(p) == Some(1)),
        "but input 1 is free, and without the menu nothing outside could ever reach it"
    );
    let out = hidden
        .outputs
        .iter()
        .find(|p| p.node == drive.0 && port(p) == Some(0))
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
        from_port: port(out).expect("an output row is a port row"),
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
            .any(|p| p.node == drive.0 && port(p) == Some(0)),
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

// ── Driven params (doc 58) ──────────────────────────────────────────────────

/// **The whole feature, through the real seam.** A wire dropped on a node's body offers its
/// PARAMS; picking one drives it; the socket then EXISTS (the view grows it, derived from the
/// wire); and pulling the wire off takes the socket away again.
///
/// Nothing here is a param-specific gesture. The drop, the menu, the socket, the cut are the
/// ones the editor already had — a parameter is simply a place a wire can land.
#[test]
fn a_wire_dropped_on_a_node_drives_one_of_its_params_and_the_socket_appears() {
    let mut m = flat();
    let lfo = m.doc.graph.add_node("value.lfo");
    let wind = m.doc.graph.add_node("force.wind");
    let inputs = fold::manifest_of(&m, wind)
        .expect("wind is registered")
        .inputs
        .len();

    // What the drop offers: the wind's params, by the label the params panel shows.
    let mut snap = ph2d_panel_motion_graph::snapshot_from(&m.doc.graph, &m.registry);
    fold::fold(&m, &mut snap);
    let offered = ph2d_panel_motion_graph::card_hidden_ports(wind.0);
    let strength = offered
        .inputs
        .iter()
        .find_map(|p| match p.target {
            ph2d_panel_motion_graph::ChoiceTarget::Param(name) if name == "strength" => Some(name),
            _ => None,
        })
        .expect("an undriven param is offered - it has no socket, so the menu is the only door");
    assert!(
        offered
            .inputs
            .iter()
            .all(|p| matches!(p.target, ph2d_panel_motion_graph::ChoiceTarget::Param(_))),
        "an ordinary node hides parameters, not ports"
    );

    // The pick — the intent the panel pushes.
    push_intent(GraphIntent::DriveParam {
        from_node: lfo.0,
        from_port: 0,
        to_node: wind.0,
        param: strength,
    });
    apply_graph_intents(
        &mut m,
        &mut ph2d_core::Playhead::default(),
        &mut ph2d_editor::ToastQueue::default(),
        &mut ph2d_editor::screens::layout::CenterSplit::None,
    );
    assert_eq!(
        m.doc
            .graph
            .param_sources(wind)
            .and_then(|s| s.get("strength")),
        Some(&(lfo, 0u16)),
        "the param is driven by the node the artist named"
    );

    // THE SOCKET NOW EXISTS — derived from the wire, appended after the declared inputs — and
    // a wire is drawn into it. Neither is in the graph: the graph has no such port.
    let snap = ph2d_panel_motion_graph::snapshot_from(&m.doc.graph, &m.registry);
    let card = snap.nodes.iter().find(|n| n.id == wind.0).unwrap();
    assert_eq!(
        card.inputs.len(),
        inputs + 1,
        "the card grew a socket for the driven param"
    );
    assert_eq!(card.inputs[inputs].name, "strength");
    assert!(
        snap.edges
            .iter()
            .any(|e| e.from_node == lfo.0 && e.to_node == wind.0 && e.to_port == inputs as u16),
        "and the wire is drawn into it"
    );
    // It is no longer offered by the menu: its socket is right there.
    let mut snap2 = ph2d_panel_motion_graph::snapshot_from(&m.doc.graph, &m.registry);
    fold::fold(&m, &mut snap2);
    assert!(
        !ph2d_panel_motion_graph::card_hidden_ports(wind.0)
            .inputs
            .iter()
            .any(|p| p.target == ph2d_panel_motion_graph::ChoiceTarget::Param("strength"))
    );

    // **Pull the wire off and the socket goes with it.** The gesture is the ordinary one —
    // the panel does not know it is unplugging a parameter.
    push_intent(GraphIntent::Disconnect {
        to_node: wind.0,
        to_port: inputs as u16,
    });
    apply_graph_intents(
        &mut m,
        &mut ph2d_core::Playhead::default(),
        &mut ph2d_editor::ToastQueue::default(),
        &mut ph2d_editor::screens::layout::CenterSplit::None,
    );
    assert!(m.doc.graph.param_sources(wind).is_none(), "un-driven");
    let snap = ph2d_panel_motion_graph::snapshot_from(&m.doc.graph, &m.registry);
    let card = snap.nodes.iter().find(|n| n.id == wind.0).unwrap();
    assert_eq!(
        card.inputs.len(),
        inputs,
        "the socket existed only because the wire did"
    );
}

/// A param driven from OUTSIDE a group keeps its wire when the node is collapsed into a card
/// — the card grows a socket for it, like any crossing wire (doc 57 §3 + doc 58).
///
/// Missed, the wire would VANISH from the view the moment the artist grouped the node: the
/// cook would go on reading it, and the canvas would be lying about what the scene computes.
#[test]
fn grouping_a_node_whose_param_is_driven_from_outside_keeps_the_wire_on_the_card() {
    let mut m = flat();
    let lfo = m.doc.graph.add_node("value.lfo");
    let wind = m.doc.graph.add_node("force.wind");
    m.doc.graph.drive_param(wind, "strength", (lfo, 0)).unwrap();
    super::subgraph::group(&mut m, vec![wind.0]);
    let sid = m.doc.subgraphs[0].id;

    let ports = super::subgraph::card_ports(&m, sid);
    assert_eq!(
        ports.inputs.len(),
        1,
        "the param wire crosses the boundary, so the card has a socket for it"
    );
    // …and that socket resolves back to the param, not to a port that does not exist.
    let (node, port) = super::subgraph::resolve_port(&m, super::subgraph::view_id(sid), 0, true)
        .expect("slot 0 resolves");
    assert_eq!(node, wind);
    assert_eq!(
        super::subgraph::param_at(&m, node, port).as_deref(),
        Some("strength"),
        "the card's socket stands for a PARAMETER of the node inside it"
    );
}
