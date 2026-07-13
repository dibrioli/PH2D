//! Subgraph model gates (doc 57). The fold's arithmetic, and the boundary that
//! refuses a corrupt nesting.

use super::*;
use crate::{Backdrop, MotionDoc};

fn sg(id: u32, parent: Option<u32>, title: &str) -> Subgraph {
    Subgraph {
        id,
        parent,
        x: 0.0,
        y: 0.0,
        title: title.into(),
    }
}

/// `a` (root) holds nodes 1,2 and nests `b`, which holds node 3.
fn nest() -> (Vec<Subgraph>, Members) {
    let subs = vec![sg(0, None, "A"), sg(1, Some(0), "B")];
    let members = Members::from([(NodeId(1), 0), (NodeId(2), 0), (NodeId(3), 1)]);
    (subs, members)
}

#[test]
fn holder_answers_the_one_question_the_fold_asks() {
    let (subs, members) = nest();
    // From the ROOT: node 4 is loose (drawn as itself); everything under A — even
    // node 3, two levels down — is drawn on A's card.
    assert_eq!(
        holder_at(&subs, &members, NodeId(4), None),
        Holder::Direct,
        "a loose node draws as itself at the root"
    );
    assert_eq!(holder_at(&subs, &members, NodeId(1), None), Holder::Card(0));
    assert_eq!(
        holder_at(&subs, &members, NodeId(3), None),
        Holder::Card(0),
        "a node nested two deep terminates on the OUTERMOST card, not on B's"
    );

    // Inside A: its own nodes are direct, B's node lands on B's card, and the loose
    // root node is across the boundary.
    assert_eq!(
        holder_at(&subs, &members, NodeId(1), Some(0)),
        Holder::Direct
    );
    assert_eq!(
        holder_at(&subs, &members, NodeId(3), Some(0)),
        Holder::Card(1)
    );
    assert_eq!(
        holder_at(&subs, &members, NodeId(4), Some(0)),
        Holder::Outside,
        "a root node, seen from inside A, is across the boundary (a ghost)"
    );

    // Inside B: A's own nodes are outside it (the boundary cuts both ways).
    assert_eq!(
        holder_at(&subs, &members, NodeId(3), Some(1)),
        Holder::Direct
    );
    assert_eq!(
        holder_at(&subs, &members, NodeId(1), Some(1)),
        Holder::Outside
    );
}

#[test]
fn descendants_and_deep_members_reach_the_whole_nest() {
    let (subs, members) = nest();
    assert_eq!(descendants(&subs, 0), BTreeSet::from([0, 1]));
    assert_eq!(descendants(&subs, 1), BTreeSet::from([1]));
    let mut deep = member_nodes_deep(&subs, &members, 0);
    deep.sort();
    assert_eq!(
        deep,
        vec![NodeId(1), NodeId(2), NodeId(3)],
        "deleting A must take node 3 with it — it is inside B, which is inside A"
    );
    assert_eq!(ancestors(&subs, 1), vec![0]);
    assert!(ancestors(&subs, 0).is_empty());
}

#[test]
fn a_doc_without_subgraphs_writes_no_section() {
    // Back-compat, byte-for-byte: every graph authored before this feature must
    // serialize exactly as it did. The `[subgraph]` header appears only when there
    // is one.
    let mut g = Graph::new();
    g.add_node("motion.grid");
    let doc = MotionDoc {
        graph: g,
        ..MotionDoc::new()
    };
    let text = doc.to_text();
    assert!(!text.contains("[subgraph]"));
    assert_eq!(MotionDoc::from_text(&text).unwrap(), doc);
}

#[test]
fn subgraphs_round_trip_through_the_text_format() {
    let mut g = Graph::new();
    let a = g.add_node("motion.grid");
    let b = g.add_node("motion.clone");
    g.connect(ph2d_nodegraph::graph::Edge {
        from: (a, 0),
        to: (b, 0),
        delayed: false,
    })
    .unwrap();
    let doc = MotionDoc {
        graph: g,
        backdrops: vec![Backdrop {
            id: 3,
            x: 1.0,
            y: 2.0,
            w: 30.0,
            h: 40.0,
            color: 1,
            title: "Wall".into(),
        }],
        base_z: 2,
        subgraphs: vec![sg(0, None, "Forces Of Nature"), sg(1, Some(0), "Inner")],
        members: Members::from([(a, 0), (b, 1)]),
        backdrop_members: BTreeMap::from([(3, 1)]),
    };
    let back = MotionDoc::from_text(&doc.to_text()).unwrap();
    assert_eq!(back, doc, "the whole nesting survives a round trip");
    assert_eq!(
        back.subgraphs[0].title, "Forces Of Nature",
        "a title with spaces round-trips (trailing free-text field)"
    );
    // Deterministic bytes (the same document always writes the same file).
    assert_eq!(doc.to_text(), back.to_text());
}

#[test]
fn a_cyclic_nest_is_rejected_at_the_boundary() {
    // A nest that contains itself has no root canvas to draw. It dies at parse —
    // NOT in a walk somewhere above, where it would hang or lie.
    let text = "v1\n[layout]\n[backdrop]\nz 0\n[subgraph]\ng 0 1 0 0 A\ng 1 0 0 0 B\n";
    assert!(matches!(
        MotionDoc::from_text(text),
        Err(ParseError::BadLine(_))
    ));
    // Self-parenting is the same defect.
    let text = "v1\n[layout]\n[backdrop]\nz 0\n[subgraph]\ng 0 0 0 0 A\n";
    assert!(matches!(
        MotionDoc::from_text(text),
        Err(ParseError::BadLine(_))
    ));
}

#[test]
fn membership_of_a_node_that_does_not_exist_is_rejected() {
    // A member that outlived its node would make a card claim a member that is not
    // there — and the fold would draw a socket for an edge nobody has.
    let text = "v1\n[layout]\n[backdrop]\nz 0\n[subgraph]\ng 0 - 0 0 A\nm 9 0\n";
    assert!(matches!(
        MotionDoc::from_text(text),
        Err(ParseError::BadLine(_))
    ));
    // ...and so is a member pointing at a subgraph that does not exist.
    let text = "v1\nn 0 motion.grid\n[layout]\n[backdrop]\nz 0\n[subgraph]\ng 0 - 0 0 A\nm 0 7\n";
    assert!(matches!(
        MotionDoc::from_text(text),
        Err(ParseError::BadLine(_))
    ));
}

#[test]
fn forget_nodes_drops_their_membership() {
    let (subs, members) = nest();
    let mut doc = MotionDoc {
        subgraphs: subs,
        members,
        ..MotionDoc::new()
    };
    doc.forget_nodes(&[NodeId(3)]);
    assert!(!doc.members.contains_key(&NodeId(3)));
    assert_eq!(doc.members.len(), 2);
}

#[test]
fn next_id_never_reuses() {
    assert_eq!(next_id(&[]), 0);
    assert_eq!(next_id(&[sg(0, None, ""), sg(4, None, "")]), 5);
}
