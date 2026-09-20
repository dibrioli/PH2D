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
//!
//! ⭐⭐ **It also carries the two things the engine needs to honour the owner's
//! laws of 2026-09-20** (see the [`ph2d_nodegraph::layout`] module docs): the
//! **PORT** each wire lands on, so the producer of an upper slot is drawn above
//! the producer of a lower one, and the **EXTENT** each card draws, so two long
//! names side by side and a tall card over a short one all clear each other.
//!
//! ⚠️⚠️ **The extent has to come from the CALLER, and that is not an accident of
//! plumbing.** A card's drawn width follows its NAME (measured text) and its
//! height follows its ROW COUNT plus its preview frame — two facts this crate
//! cannot reach: it depends on `ph2d-nodegraph` and nothing else, on purpose, and
//! the name table, the registry and the text measurer all live above it. So
//! [`arrange`] asks, once per card, through [`Medida`].

use crate::MotionDoc;
use crate::subgraph::{Holder, holder_at};
use ph2d_nodegraph::graph::NodeId;
use ph2d_nodegraph::layout::{Extent, Item, Wire};

/// A collapsed group card is keyed with this bit set — above any node id — so
/// nodes and cards never collide in one layout's key space.
const CARD_BIT: u64 = 1 << 40;

/// **What the caller is being asked to measure.** One variant per kind of card a
/// canvas can hold, which is exactly the two the fold produces.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Carta {
    /// A real node of the graph.
    No(NodeId),
    /// A collapsed group card, by subgraph id.
    Grupo(u32),
}

/// How big each card draws. See [`Extent`] for what the numbers mean, and ⚠️ for
/// the rule that a caller drawing two zoom regimes declares the **UNION** of them.
pub trait Medida {
    fn extensao(&self, carta: Carta) -> Extent;
}

impl<F: Fn(Carta) -> Extent> Medida for F {
    fn extensao(&self, carta: Carta) -> Extent {
        self(carta)
    }
}

/// **The door for a caller with no measurer** — every card the historical
/// `190 × 220`. ⚠️ It is honest only where nothing is wider than that: with it,
/// two long names overlap exactly as they did before the extents existed. It is
/// here for the gates of this crate and for a caller that has no registry.
pub struct CartaoDeFabrica;

impl Medida for CartaoDeFabrica {
    fn extensao(&self, _: Carta) -> Extent {
        Extent::default()
    }
}

/// Arrange the whole document: the root canvas and every subgraph's interior,
/// each collapsed group treated as a single card and written to its stored
/// position. The members of a group are laid out on the group's own canvas.
pub fn arrange(doc: &mut MotionDoc, medida: &impl Medida) {
    // Every canvas: the root (None) and each subgraph's interior. Collected up
    // front so no borrow of `doc.subgraphs` is held across the mutation.
    let levels: Vec<Option<u32>> = std::iter::once(None)
        .chain(doc.subgraphs.iter().map(|s| Some(s.id)))
        .collect();
    for level in levels {
        arrange_level(doc, level, medida);
    }
}

/// Lay out one canvas: `None` = the root, `Some(id)` = inside that group.
fn arrange_level(doc: &mut MotionDoc, level: Option<u32>, medida: &impl Medida) {
    // The items DIRECTLY on this canvas: the nodes whose owner is this level,
    // plus one card per child subgraph.
    let mut items: Vec<Item<u64>> = Vec::new();
    for n in doc.graph.nodes() {
        if doc.members.get(&n.id).copied() == level {
            items.push(Item {
                key: u64::from(n.id.0),
                extent: medida.extensao(Carta::No(n.id)),
            });
        }
    }
    for s in &doc.subgraphs {
        if s.parent == level {
            items.push(Item {
                key: CARD_BIT | u64::from(s.id),
                extent: medida.extensao(Carta::Grupo(s.id)),
            });
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
            Holder::Direct => Some(u64::from(node.0)),
            Holder::Card(sid) => Some(CARD_BIT | u64::from(sid)),
            Holder::Outside => None,
        }
    };
    // ⚠️ **A card's slot is not the member's slot.** A collapsed group draws
    // sockets DERIVED from the edges that cross its boundary, and their order is
    // resolved by the shell (`fold::card_ports`), which this crate cannot see. So
    // a folded endpoint reports `0` — the neutral value, which is what every
    // caller without ports gave the engine before. ⭐ The case the owner reported
    // is unaffected: there the slot that decides is on a REAL node (the
    // duplicator's `shape`/`points`), and a real endpoint reports its real slot.
    let slot = |node: NodeId, port: u16| -> u16 {
        match holder_at(&doc.subgraphs, &doc.members, node, level) {
            Holder::Direct => port,
            _ => 0,
        }
    };
    let mut wires: Vec<Wire<u64>> = Vec::new();
    for e in doc.graph.edges() {
        if let (Some(a), Some(b)) = (item_of(e.from.0), item_of(e.to.0))
            && a != b
        {
            wires.push(Wire {
                from: a,
                out_port: slot(e.from.0, e.from.1),
                to: b,
                in_port: slot(e.to.0, e.to.1),
                delayed: e.delayed,
            });
        }
    }

    // Write each placed item where it is drawn: a node into the flat graph's
    // layout, a card into its subgraph's collapsed x/y.
    for (key, pos) in ph2d_nodegraph::layout::plan_edges(&items, &wires) {
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

        arrange(&mut doc, &CartaoDeFabrica);

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

        arrange(&mut doc, &CartaoDeFabrica);

        // Inside the group, m1 feeds m2, so m1 is left of m2. The boundary edges
        // (n0→m1, m2→n3) are ghosts here and must not order the interior.
        assert!(
            doc.graph.pos(m1).expect("m1").x < doc.graph.pos(m2).expect("m2").x,
            "the members are laid out in chain order on the group canvas"
        );
    }

    /// **The measurer reaches the geometry.** A card declared twice as wide
    /// pushes the next column out by exactly that much — the gate against the
    /// extent being collected and then dropped on the floor, which is the way a
    /// plumbing change like this fails silently. FALSIFIED by passing
    /// `Extent::default()` regardless of what `medida` answers.
    #[test]
    fn a_larger_card_pushes_its_neighbour_further_out() {
        let mut doc = MotionDoc::new();
        let a = doc.graph.add_node("motion.grid");
        let b = doc.graph.add_node("motion.output");
        wire(&mut doc.graph, a, b);

        arrange(&mut doc, &CartaoDeFabrica);
        let estreito = doc.graph.pos(b).expect("b").x - doc.graph.pos(a).expect("a").x;

        let largo = |_: Carta| Extent {
            left: 0.0,
            right: 2.0 * (Extent::default().right),
            top: 0.0,
            bottom: Extent::default().bottom,
        };
        arrange(&mut doc, &largo);
        let folgado = doc.graph.pos(b).expect("b").x - doc.graph.pos(a).expect("a").x;

        assert!(
            folgado > estreito + 100.0,
            "the measured width reaches the step (narrow {estreito}, wide {folgado})"
        );
    }
}
