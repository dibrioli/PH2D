//! **Auto-layout for the value-domain smoke scenes.** The demos build small but
//! BRANCHY graphs (a grid feeds a `move` and two or three value producers), and
//! hand-placed `set_pos` calls tangled them into overlapping cards. This lays a
//! subgraph out as a **single straight horizontal line** — every node a distinct
//! `x`, one shared `y`, spaced so cards never overlap — ordered so edges flow
//! left→right (a topological sort). Two comparison rows become two clean lines.
//!
//! It also MARKS the one node the smoke wants evaluated, so the reviewer knows
//! where to look without counting cards.

use ph2d_nodegraph::graph::{Graph, NodeId, Pos};
use std::collections::{BTreeMap, BTreeSet};

/// Horizontal step between cards. `> CARD_W` (190 px in the panel) plus a gap, so
/// two adjacent cards never touch.
const DX: f32 = 220.0;
/// The `x` of the leftmost card.
const X0: f32 = 60.0;
/// Vertical gap between two comparison rows — clears the tallest card (a
/// seven-param producer is ~200 px), so the rows never overlap.
pub(crate) const ROW_GAP: f32 = 380.0;
/// The label stamped on the node to evaluate. ASCII only (the tofu gate) and
/// English (the UI-language rule); the brackets make it pop among plain names.
const MARK: &str = ">> EVALUATE <<";

/// Lay `nodes` out in a straight horizontal line at height `y`, left to right in
/// topological order (edges flow forward), then stamp `hero` (when `Some`) with
/// the evaluate marker. `nodes` is the subgraph to arrange; edges to nodes outside
/// the set are ignored for ordering. A reference row passes `None` — only the row
/// under evaluation is marked.
pub(crate) fn lay_horizontal(g: &mut Graph, nodes: &[NodeId], y: f32, hero: Option<NodeId>) {
    let order = topo_order(g, nodes);
    for (i, &n) in order.iter().enumerate() {
        g.set_pos(
            n,
            Pos {
                x: X0 + i as f32 * DX,
                y,
            },
        );
    }
    if let Some(h) = hero {
        g.set_label(h, MARK);
    }
}

/// Kahn topological sort of the subgraph induced by `nodes`, over non-`delayed`
/// edges (a `pre` edge is feedback — it must not order the layout). Ties break by
/// input order, so the demo's creation order (already source→sink) is preserved
/// where the graph allows. Any node left unresolved (a cycle) is appended in
/// input order rather than dropped.
fn topo_order(g: &Graph, nodes: &[NodeId]) -> Vec<NodeId> {
    let set: BTreeSet<NodeId> = nodes.iter().copied().collect();
    let mut indeg: BTreeMap<NodeId, usize> = nodes.iter().map(|&n| (n, 0)).collect();
    let mut succ: BTreeMap<NodeId, Vec<NodeId>> = BTreeMap::new();
    for e in g.edges() {
        if e.delayed {
            continue;
        }
        let (u, _) = e.from;
        let (v, _) = e.to;
        if set.contains(&u) && set.contains(&v) {
            if let Some(d) = indeg.get_mut(&v) {
                *d += 1;
            }
            succ.entry(u).or_default().push(v);
        }
    }
    // Seed with the zero-in-degree nodes in INPUT order (stable, source-first).
    let mut ready: Vec<NodeId> = nodes.iter().copied().filter(|n| indeg[n] == 0).collect();
    let mut out = Vec::with_capacity(nodes.len());
    let mut i = 0;
    while i < ready.len() {
        let n = ready[i];
        i += 1;
        out.push(n);
        if let Some(children) = succ.get(&n) {
            for &v in children {
                if let Some(d) = indeg.get_mut(&v) {
                    *d -= 1;
                    if *d == 0 {
                        ready.push(v);
                    }
                }
            }
        }
    }
    // A cycle would leave nodes unplaced — append them so none is lost.
    for &n in nodes {
        if !out.contains(&n) {
            out.push(n);
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A branchy chain (`grid → {a, b} → sink`) lays out with every card at a
    /// DISTINCT x and the SHARED y — a straight horizontal line, no two cards on
    /// top of each other — and the source sits left of the sink (edges flow
    /// forward).
    #[test]
    fn a_branchy_chain_becomes_a_straight_line_with_no_overlap() {
        let mut g = Graph::new();
        let grid = g.add_node("motion.grid");
        let a = g.add_node("value.lfo");
        let b = g.add_node("value.noise");
        let drive = g.add_node("motion.drive");
        for (from, to, port) in [(grid, a, 0u16), (grid, b, 0), (a, drive, 1), (b, drive, 1)] {
            let _ = g.connect(ph2d_nodegraph::graph::Edge {
                from: (from, 0),
                to: (to, port),
                delayed: false,
            });
        }
        let nodes = [grid, a, b, drive];
        lay_horizontal(&mut g, &nodes, 100.0, Some(drive));

        // Every node on the same y, and every x distinct (no overlap).
        let mut xs: Vec<f32> = Vec::new();
        for &n in &nodes {
            let p = g.layout()[&n];
            assert_eq!(p.y, 100.0, "all cards share the row's y");
            assert!(!xs.iter().any(|&x| (x - p.x).abs() < DX - 1.0), "x overlaps");
            xs.push(p.x);
        }
        // The source is left of the sink (topological order reached x).
        assert!(
            g.layout()[&grid].x < g.layout()[&drive].x,
            "the source sits left of the sink"
        );
        // The hero carries the mark; the others do not.
        assert_eq!(g.label(drive), Some(MARK), "the hero is marked");
        assert_ne!(g.label(grid), Some(MARK), "a non-hero is not marked");
    }
}
