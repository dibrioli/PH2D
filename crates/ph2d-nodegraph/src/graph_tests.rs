//! Tests for [`super`] — the graph's own invariants (ids, edges, cycles, params, labels).
//!
//! A sibling file, not an inline module: `graph.rs` is the hottest foundational file in the
//! repo and it hit the workspace's 700-LOC cap. The cap is answered by SPLITTING, never by an
//! allowlist entry ([[feedback_loc_cap_split_not_allowlist_and_fmt_reexpands]]).
use super::*;

fn edge(from: NodeId, to: NodeId, delayed: bool) -> Edge {
    Edge {
        from: (from, 0),
        to: (to, 0),
        delayed,
    }
}

#[test]
fn plain_back_edge_is_rejected_but_pre_is_allowed() {
    let mut g = Graph::new();
    let a = g.add_node("a");
    let b = g.add_node("b");
    assert_eq!(g.connect(edge(a, b, false)), Ok(()));
    // b -> a as a plain edge would close a cycle: rejected.
    assert_eq!(g.connect(edge(b, a, false)), Err(EdgeError::WouldCycle));
    // b -> a as a `pre` (delayed) edge is the legal way to express feedback.
    assert_eq!(g.connect(edge(b, a, true)), Ok(()));
    assert_eq!(g.edges().len(), 2);
}

#[test]
fn unknown_node_is_rejected() {
    let mut g = Graph::new();
    let a = g.add_node("a");
    assert_eq!(
        g.connect(edge(a, NodeId(999), false)),
        Err(EdgeError::UnknownNode)
    );
}

#[test]
fn self_edge_is_a_cycle() {
    let mut g = Graph::new();
    let a = g.add_node("a");
    assert_eq!(g.connect(edge(a, a, false)), Err(EdgeError::WouldCycle));
}

#[test]
#[should_panic(expected = "whitespace-free")]
fn add_node_rejects_whitespaced_name() {
    // Would corrupt the whitespace-delimited textual format.
    Graph::new().add_node("motion clone");
}

#[test]
fn duplicate_input_edge_is_rejected() {
    let mut g = Graph::new();
    let a = g.add_node("a");
    let b = g.add_node("b");
    let c = g.add_node("c");
    g.connect(Edge {
        from: (a, 0),
        to: (c, 0),
        delayed: false,
    })
    .unwrap();
    // A second edge into the same input port (c, 0) is rejected.
    assert_eq!(
        g.connect(Edge {
            from: (b, 0),
            to: (c, 0),
            delayed: false
        }),
        Err(EdgeError::InputAlreadyConnected)
    );
}

#[test]
fn disconnect_removes_the_edge_into_an_input() {
    let mut g = Graph::new();
    let a = g.add_node("a");
    let b = g.add_node("b");
    g.connect(Edge {
        from: (a, 0),
        to: (b, 1),
        delayed: false,
    })
    .unwrap();
    assert_eq!(g.edges().len(), 1);
    // Wrong port → nothing removed.
    assert_eq!(g.disconnect(b, 0), None);
    assert_eq!(g.edges().len(), 1);
    // Right port → the edge comes back out.
    assert_eq!(
        g.disconnect(b, 1),
        Some(Edge {
            from: (a, 0),
            to: (b, 1),
            delayed: false
        })
    );
    assert!(g.edges().is_empty());
    // The port is free again — a fresh connect is accepted.
    assert_eq!(
        g.connect(Edge {
            from: (a, 0),
            to: (b, 1),
            delayed: false
        }),
        Ok(())
    );
}

#[test]
fn remove_node_drops_node_incident_edges_layout_and_params() {
    let mut g = Graph::new();
    let a = g.add_node("a");
    let b = g.add_node("b");
    let c = g.add_node("c");
    g.connect(edge(a, b, false)).unwrap();
    g.connect(edge(b, c, false)).unwrap();
    g.set_pos(b, Pos { x: 1.0, y: 2.0 });
    g.set_param(b, "k", 3.0);

    assert!(g.remove_node(b));
    // Node gone.
    assert!(g.node(b).is_none());
    assert_eq!(g.nodes().len(), 2);
    // Both edges incident on `b` (a→b and b→c) are gone; none reference `b`.
    assert!(g.edges().is_empty());
    // Layout + param overrides for `b` are gone.
    assert!(g.pos(b).is_none());
    assert!(g.node_param_overrides(b).is_none());
    // Untouched neighbours survive.
    assert!(g.node(a).is_some() && g.node(c).is_some());
    // Removing a non-existent node is a no-op.
    assert!(!g.remove_node(b));
    assert!(!g.remove_node(NodeId(999)));
}

#[test]
fn input_edge_resolves_source() {
    let mut g = Graph::new();
    let a = g.add_node("a");
    let b = g.add_node("b");
    g.connect(Edge {
        from: (a, 0),
        to: (b, 1),
        delayed: false,
    })
    .unwrap();
    assert_eq!(g.input_edge(b, 1), Some((a, 0, false)));
    assert_eq!(g.input_edge(b, 0), None);
}

/// **An empty name is not a name** (doc 61): clearing the rename box means *"call it what
/// it is"*, not *"leave the card blank"*. Without this, a name you typed and then deleted
/// would leave an invisible label behind — and the file would stay at `v4` forever over a
/// rename that no longer exists.
#[test]
fn naming_a_node_and_un_naming_it() {
    let mut g = Graph::new();
    let n = g.add_node("motion.grid");
    assert_eq!(g.label(n), None, "a node is born with no name of its own");

    g.set_label(n, "The Sky");
    assert_eq!(g.label(n), Some("The Sky"));
    g.set_label(n, "   ");
    assert_eq!(g.label(n), None, "whitespace is not a name");
    assert!(g.node_labels().is_empty());

    // The textual format is LINE-oriented, so a newline inside a label would be a second,
    // unparsable record. Refused at the door rather than corrupting the file on save.
    g.set_label(n, "Two\nLines");
    assert_eq!(g.label(n), None);
    g.set_label(n, "  Padded  ");
    assert_eq!(g.label(n), Some("Padded"), "and it is trimmed");
}

/// Deleting a node takes its name with it. A label left behind on a dead id is a phantom
/// that the next node to be minted with that id would inherit — it would be *born named*
/// after the node you just deleted.
#[test]
fn deleting_a_node_takes_its_name_with_it() {
    let mut g = Graph::new();
    let n = g.add_node("motion.grid");
    g.set_label(n, "The Sky");
    g.remove_node(n);
    assert!(g.node_labels().is_empty());
    assert_eq!(g.label(n), None);
}
