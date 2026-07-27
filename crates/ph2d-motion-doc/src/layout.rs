//! **Subgraph-aware auto-layout for a `MotionDoc`.**
//!
//! [`ph2d_nodegraph::layout`] lays out the FLAT graph, but a `MotionDoc` folds
//! sets of nodes into collapsed group cards ([`crate::subgraph`]): on the parent
//! canvas those members are hidden behind one card, whose position lives on the
//! [`Subgraph`](crate::Subgraph), not on any node. Arranging the flat graph alone
//! moves the hidden members (and every neighbour) but leaves each group card at
//! its stale spot, floating away from the chain it belongs in.
//!
//! [`arrange`] fixes that: it lays out **each canvas** — the root and every
//! group's interior — treating a group as ONE card. On a canvas, an edge that
//! crosses into a child group terminates on that child's card (the same
//! [`holder_at`](crate::subgraph::holder_at) fold the view uses), so a group
//! sits inline among its neighbours; its members are laid out on the group's own
//! canvas. Positions only — the flat graph the cook sees is untouched.

use crate::MotionDoc;
use crate::subgraph::{Holder, holder_at};
use ph2d_nodegraph::graph::NodeId;

/// A collapsed group card is keyed with this bit set — above any node id — so
/// nodes and cards never collide in one layout's key space.
const CARD_BIT: u64 = 1 << 40;

/// Arrange the whole document: the root canvas and every subgraph's interior,
/// each collapsed group treated as a single card and written to its stored
/// position. The members of a group are laid out on the group's own canvas.
pub fn arrange(doc: &mut MotionDoc) {
    // Every canvas: the root (None) and each subgraph's interior. Collected up
    // front so no borrow of `doc.subgraphs` is held across the mutation.
    let levels: Vec<Option<u32>> = std::iter::once(None)
        .chain(doc.subgraphs.iter().map(|s| Some(s.id)))
        .collect();
    for level in levels {
        arrange_level(doc, level);
    }
}

/// Lay out one canvas: `None` = the root, `Some(id)` = inside that group.
fn arrange_level(doc: &mut MotionDoc, level: Option<u32>) {
    // The items DIRECTLY on this canvas: the nodes whose owner is this level,
    // plus one card per child subgraph.
    let mut items: Vec<u64> = Vec::new();
    for n in doc.graph.nodes() {
        if doc.members.get(&n.id).copied() == level {
            items.push(n.id.0 as u64);
        }
    }
    for s in &doc.subgraphs {
        if s.parent == level {
            items.push(CARD_BIT | s.id as u64);
        }
    }
    if items.is_empty() {
        return;
    }

    // Every graph edge projected onto this canvas: an endpoint draws as itself
    // (Direct), as the child card standing in for it (Card), or is across the
    // boundary (Outside → not on this canvas). An edge internal to one child card
    // (both endpoints map to it) does not order this canvas.
    let item_of = |node: NodeId| -> Option<u64> {
        match holder_at(&doc.subgraphs, &doc.members, node, level) {
            Holder::Direct => Some(node.0 as u64),
            Holder::Card(sid) => Some(CARD_BIT | sid as u64),
            Holder::Outside => None,
        }
    };
    let mut edges: Vec<(u64, u64, bool)> = Vec::new();
    for e in doc.graph.edges() {
        if let (Some(a), Some(b)) = (item_of(e.from.0), item_of(e.to.0))
            && a != b
        {
            edges.push((a, b, e.delayed));
        }
    }

    // Write each placed item where it is drawn: a node into the flat graph's
    // layout, a card into its subgraph's collapsed x/y.
    for (key, pos) in ph2d_nodegraph::layout::plan_edges(&items, &edges) {
        if key & CARD_BIT != 0 {
            let sid = (key & !CARD_BIT) as u32;
            if let Some(s) = doc.subgraphs.iter_mut().find(|s| s.id == sid) {
                s.x = pos.x;
                s.y = pos.y;
            }
        } else {
            doc.graph.set_pos(NodeId(key as u32), pos);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Subgraph;
    use ph2d_nodegraph::graph::{Edge, Graph};

    fn wire(g: &mut Graph, a: NodeId, b: NodeId) {
        g.connect(Edge {
            from: (a, 0),
            to: (b, 0),
            delayed: false,
        })
        .expect("edge");
    }

    /// A chain `n0 → m1 → m2 → n3` with `m1,m2` folded into a group. On the root
    /// canvas the group must sit INLINE between `n0` and `n3` (its members hidden),
    /// with the card written to a fresh position — not left at its stale spot.
    #[test]
    fn a_folded_group_sits_inline_between_its_neighbours() {
        let mut doc = MotionDoc::new();
        let n0 = doc.graph.add_node("motion.grid");
        let m1 = doc.graph.add_node("motion.move");
        let m2 = doc.graph.add_node("motion.drive");
        let n3 = doc.graph.add_node("motion.output");
        wire(&mut doc.graph, n0, m1);
        wire(&mut doc.graph, m1, m2);
        wire(&mut doc.graph, m2, n3);
        doc.subgraphs.push(Subgraph {
            id: 0,
            parent: None,
            x: 999.0,
            y: 999.0,
            title: "Group".into(),
        });
        doc.members.insert(m1, 0);
        doc.members.insert(m2, 0);

        arrange(&mut doc);

        let card = &doc.subgraphs[0];
        let x0 = doc.graph.pos(n0).expect("n0").x;
        let x3 = doc.graph.pos(n3).expect("n3").x;
        // The card is between its neighbours, and its stale 999 was overwritten.
        assert!(
            x0 < card.x && card.x < x3,
            "the group card sits inline in the chain"
        );
        assert!(
            card.x < 500.0,
            "the stale card position was replaced by the layout"
        );
    }

    /// The members of a group are laid out on the group's OWN canvas — in chain
    /// order, and independent of the neighbours across the boundary.
    #[test]
    fn a_groups_members_are_arranged_on_its_own_canvas() {
        let mut doc = MotionDoc::new();
        let n0 = doc.graph.add_node("motion.grid");
        let m1 = doc.graph.add_node("motion.move");
        let m2 = doc.graph.add_node("motion.drive");
        let n3 = doc.graph.add_node("motion.output");
        wire(&mut doc.graph, n0, m1);
        wire(&mut doc.graph, m1, m2);
        wire(&mut doc.graph, m2, n3);
        doc.subgraphs.push(Subgraph {
            id: 0,
            parent: None,
            x: 0.0,
            y: 0.0,
            title: "Group".into(),
        });
        doc.members.insert(m1, 0);
        doc.members.insert(m2, 0);

        arrange(&mut doc);

        // Inside the group, m1 feeds m2, so m1 is left of m2. The boundary edges
        // (n0→m1, m2→n3) are ghosts here and must not order the interior.
        assert!(
            doc.graph.pos(m1).expect("m1").x < doc.graph.pos(m2).expect("m2").x,
            "the members are laid out in chain order on the group canvas"
        );
    }
}
