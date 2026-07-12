//! Headless seam tests do **plumbing** do grafo (split do `motion_bridge_tests.rs`
//! pelo teto HR-18 de LOC). Espelha o par `motion_bridge_params.rs` /
//! `motion_bridge_param_tests.rs`: aqui provamos o que o módulo irmão
//! `motion_bridge_plumbing.rs` faz — aresta `pre` no ciclo, self-loop de estado,
//! re-plumbing da entrada de forças, recusa do time-remap sobre nó sequencial e
//! a cura do plumbing no delete.

use super::*;
use crate::motion_state::MotionState;

/// M2-dynamics: a wire that would close a cycle is retried as a `pre`
/// (delayed) edge — the one legal meaning of a cycle in this substrate — so a
/// user CAN hand-wire a feedback loop (spring state, force loop). Reverting
/// the retry makes the connect refuse and this goes red.
#[test]
fn a_cycle_closing_connect_becomes_a_pre_edge() {
    use ph2d_editor::ToastQueue;
    use ph2d_nodegraph::graph::{Edge, Graph};

    let mut motion = MotionState::new();
    let mut g = Graph::new();
    let grid = g.add_node("motion.grid");
    let spring = g.add_node("motion.spring");
    g.connect(Edge {
        from: (grid, 0),
        to: (spring, 0),
        delayed: false,
    })
    .unwrap();
    motion.doc.graph = g;
    let mut toasts = ToastQueue::new();

    // spring.out -> spring.state closes a (self-)cycle: the shell converts it.
    super::connect::apply_connect(&mut motion, &mut toasts, spring.0, 0, spring.0, 1);
    let edge = motion
        .doc
        .graph
        .edges()
        .iter()
        .find(|e| e.to == (spring, 1))
        .expect("the feedback wire was created, not refused");
    assert!(edge.delayed, "cycle-closing wire became a pre edge");

    // A plain forward wire stays plain (no over-eager delaying).
    let plain = motion
        .doc
        .graph
        .edges()
        .iter()
        .find(|e| e.to == (spring, 0))
        .unwrap();
    assert!(!plain.delayed);
}

/// Sequential-node template via the plumbing reconcile (docs/Motion Nodes/03):
/// a feedback host (`forces` on integrate, `state` on spring) lands with its
/// `pre` self-loop already plumbed; a plain modifier is untouched.
#[test]
fn add_node_template_wires_the_state_self_loop() {
    use ph2d_nodegraph::graph::Graph;

    let motion = MotionState::new();
    let mut g = Graph::new();
    let before = g.clone();
    let integrate = g.add_node("motion.integrate");
    let spring = g.add_node("motion.spring");
    let _plain = g.add_node("motion.move");
    plumbing::reconcile_after(&mut g, &motion.registry, &before);
    assert!(
        g.edges()
            .iter()
            .any(|e| e.from == (integrate, 0) && e.to == (integrate, 1) && e.delayed),
        "integrate got its pre self-loop on the forces port"
    );
    assert!(
        g.edges()
            .iter()
            .any(|e| e.from == (spring, 0) && e.to == (spring, 1) && e.delayed),
        "spring got its pre self-loop"
    );
    assert_eq!(
        g.edges().len(),
        2,
        "a node with no feedback port is untouched by the template"
    );
    // Idempotent: a second pass changes nothing.
    let snapshot = g.clone();
    plumbing::reconcile_after(&mut g, &motion.registry, &snapshot);
    assert_eq!(g.edges().len(), 2);
}

/// The O1 authoring flow (docs/Motion Nodes/03): the user wires a force chain
/// into integrate's `forces` port and NEVER draws the state loop — the bridge
/// clears the self-loop, lands the forward wire, and re-plumbs the `pre` into
/// the chain's dangling head. Pulling the chain off restores the self-loop and
/// sweeps the branch plumbing. Falsified by construction: skip `apply_connect`
/// (raw `Graph::connect`) and the forces port is occupied by the self-loop, so
/// the wire is refused — the plumbing IS what makes the gesture land.
#[test]
fn wiring_a_chain_into_forces_replumbs_the_state_entry() {
    use ph2d_editor::ToastQueue;
    use ph2d_nodegraph::graph::{Edge, EdgeError, Graph};

    let mut motion = MotionState::new();
    let mut g = Graph::new();
    let before = g.clone();
    let integrate = g.add_node("motion.integrate");
    let wind = g.add_node("force.wind");
    let curl = g.add_node("force.curl");
    let drag = g.add_node("force.drag");
    plumbing::reconcile_after(&mut g, &motion.registry, &before);
    for (from, to) in [(wind, curl), (curl, drag)] {
        g.connect(Edge {
            from: (from, 0),
            to: (to, 0),
            delayed: false,
        })
        .unwrap();
    }

    // Control: WITHOUT the bridge, the self-looped forces port refuses the wire.
    let mut raw = g.clone();
    assert_eq!(
        raw.connect(Edge {
            from: (drag, 0),
            to: (integrate, 1),
            delayed: false,
        }),
        Err(EdgeError::InputAlreadyConnected),
        "the self-loop occupies forces; only the bridge gesture can land there"
    );

    motion.doc.graph = g;
    let mut toasts = ToastQueue::new();
    super::connect::apply_connect(&mut motion, &mut toasts, drag.0, 0, integrate.0, 1);

    let edges = motion.doc.graph.edges();
    assert!(
        edges
            .iter()
            .any(|e| e.from == (drag, 0) && e.to == (integrate, 1) && !e.delayed),
        "the chain landed on forces as a plain forward wire"
    );
    assert!(
        !edges
            .iter()
            .any(|e| e.from == (integrate, 0) && e.to == (integrate, 1)),
        "the self-loop is gone once a chain feeds forces"
    );
    assert!(
        edges
            .iter()
            .any(|e| e.from == (integrate, 0) && e.to == (wind, 0) && e.delayed),
        "the state entry re-plumbed to the chain's dangling head"
    );

    // Pull the chain off `forces`: plumbing sweeps the branch and restores the
    // self-loop (the host stays alive, ballistic).
    apply_disconnect(&mut motion, &mut toasts, integrate.0, 1);
    let edges = motion.doc.graph.edges();
    assert!(
        !edges.iter().any(|e| e.to == (wind, 0)),
        "the branch's state entry was swept with the chain"
    );
    assert!(
        edges
            .iter()
            .any(|e| e.from == (integrate, 0) && e.to == (integrate, 1) && e.delayed),
        "the forces port self-loops again"
    );
}

/// M2.N1: a Time Remap cannot rewrite the clock of a **sequential** node (one
/// on a `pre` edge). The cook refuses it (`SequentialInTimeScope`) and the sink
/// would silently stop drawing — so the editor refuses the WIRE, with the
/// reason. A remap fed by ordinary nodes still connects.
///
/// Falsified against the cook: the graph the guard rejects is exactly the graph
/// `cook_scoped` errors on, and the graph it accepts cooks clean.
#[test]
fn a_wire_dragging_a_sequential_node_under_a_time_remap_is_refused() {
    use ph2d_editor::ToastQueue;
    use ph2d_nodegraph::cook::{Cook, CookError};
    use ph2d_nodegraph::graph::{Edge, Graph};

    let mut motion = MotionState::new();
    let mut g = Graph::new();
    let before = g.clone();
    let grid = g.add_node("motion.grid");
    let spring = g.add_node("motion.spring");
    let remap = g.add_node("motion.time_remap");
    plumbing::reconcile_after(&mut g, &motion.registry, &before); // spring's pre self-loop
    g.connect(Edge {
        from: (grid, 0),
        to: (spring, 0),
        delayed: false,
    })
    .unwrap();
    motion.doc.graph = g;
    motion.doc.graph.set_param(remap, "mode", 3.0); // Freeze → a real (non-identity) scope
    let mut toasts = ToastQueue::new();

    // spring -> remap.in would put the spring inside the remapped scope.
    super::connect::apply_connect(&mut motion, &mut toasts, spring.0, 0, remap.0, 0);
    assert!(
        !motion.doc.graph.edges().iter().any(|e| e.to == (remap, 0)),
        "the wire that would scope a sequential node must be refused"
    );

    // Falsify the guard against the cook: force the edge in — the guard sees it,
    // and the cook errors on exactly that graph.
    let mut forced = motion.doc.graph.clone();
    forced
        .connect(Edge {
            from: (spring, 0),
            to: (remap, 0),
            delayed: false,
        })
        .unwrap();
    assert!(
        super::connect::scopes_a_sequential_node(&forced),
        "the guard must be what rejected the wire (not some other refusal)"
    );
    let scopes = ph2d_node_motion_time_remap::time_scopes(&forced, &motion.registry);
    assert_eq!(
        Cook::new()
            .cook_scoped(&forced, &motion.registry, remap, 1.0, &scopes)
            .err(),
        Some(CookError::SequentialInTimeScope { node: spring }),
        "the guard refuses exactly what the cook refuses"
    );

    // The grid alone (no `pre` anywhere upstream) connects and cooks fine.
    super::connect::apply_connect(&mut motion, &mut toasts, grid.0, 0, remap.0, 0);
    assert!(
        motion
            .doc
            .graph
            .edges()
            .iter()
            .any(|e| e.from == (grid, 0) && e.to == (remap, 0)),
        "an ordinary subtree is welcome under a Time Remap"
    );
    assert!(!super::connect::scopes_a_sequential_node(&motion.doc.graph));
    let scopes = ph2d_node_motion_time_remap::time_scopes(&motion.doc.graph, &motion.registry);
    assert!(
        Cook::new()
            .cook_scoped(&motion.doc.graph, &motion.registry, remap, 1.0, &scopes)
            .is_ok()
    );
}

/// Deleting a mid-chain force re-heals the plumbing: the orphaned upstream
/// stub loses its state entry, the surviving chain's new dangling head gains
/// it. And the managed badge itself is not hand-deletable (the disconnect
/// gesture on it is refused with a toast) while an EXPERT `pre` elsewhere
/// still is.
#[test]
fn plumbing_reheals_on_delete_and_managed_pre_is_not_hand_deletable() {
    use ph2d_editor::ToastQueue;
    use ph2d_nodegraph::graph::{Edge, Graph};

    let mut motion = MotionState::new();
    let mut g = Graph::new();
    let before = g.clone();
    let integrate = g.add_node("motion.integrate");
    let wind = g.add_node("force.wind");
    let curl = g.add_node("force.curl");
    let drag = g.add_node("force.drag");
    plumbing::reconcile_after(&mut g, &motion.registry, &before);
    for (from, to) in [(wind, curl), (curl, drag)] {
        g.connect(Edge {
            from: (from, 0),
            to: (to, 0),
            delayed: false,
        })
        .unwrap();
    }
    motion.doc.graph = g;
    let mut toasts = ToastQueue::new();
    super::connect::apply_connect(&mut motion, &mut toasts, drag.0, 0, integrate.0, 1);

    // The managed badge refuses the delete gesture...
    let before_len = motion.doc.graph.edges().len();
    apply_disconnect(&mut motion, &mut toasts, wind.0, 0);
    assert_eq!(
        motion.doc.graph.edges().len(),
        before_len,
        "the managed state entry is not hand-deletable"
    );

    // ...while deleting the mid-chain node re-heals: wind's stale entry is
    // swept, drag (the new dangling head) gains one.
    apply_delete_selection(&mut motion, vec![curl.0]);
    let edges = motion.doc.graph.edges();
    assert!(
        !edges.iter().any(|e| e.to.0 == curl || e.from.0 == curl),
        "the deleted node's edges died with it"
    );
    assert!(
        !edges.iter().any(|e| e.to == (wind, 0)),
        "the orphaned stub lost its state entry"
    );
    assert!(
        edges
            .iter()
            .any(|e| e.from == (integrate, 0) && e.to == (drag, 0) && e.delayed),
        "the surviving chain's new head gained the state entry"
    );
}
