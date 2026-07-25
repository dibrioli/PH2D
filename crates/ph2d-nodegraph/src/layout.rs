//! **Auto-layout: a layered (Sugiyama-style) arrangement of a node graph.**
//!
//! A hand-placed graph tangles as it grows: cards overlap, wires cross, and a
//! branchy sub-graph laid out as parallel rows collides with a neighbour that
//! was placed by hand at the same coordinates. [`arrange`] replaces every
//! node's position with a clean layout:
//!
//! - each **weakly-connected component** is laid out on its own,
//! - nodes are placed in **columns by longest-path depth**, so every edge flows
//!   left → right,
//! - within a column they are **ordered to reduce crossings** (a few barycenter
//!   sweeps, the standard heuristic),
//! - and the components are **stacked into non-overlapping horizontal bands**.
//!
//! It moves POSITIONS only — it never touches nodes, edges, params, labels, or
//! the frozen node contract (ADR-0039): it is a sibling module riding the same
//! `layout` map the graph already stores, exactly like [`crate::gpu`] rides the
//! registry side-channel. A `pre` (delayed) edge is feedback: it is **ignored
//! for column ordering** (it would otherwise force a cycle) but still binds its
//! two endpoints into the same band.
//!
//! Spacing is in editor units, which are panel pixels at zoom 1. The panel's
//! `geom` sizes a card as `CARD_W = 190` wide by `HEADER_H (26) + rows·ROW_H
//! (22) + preview` tall — the tallest producer with a preview is ~250 px. The
//! steps below clear those so no two cards ever overlap; the panel pans and
//! zooms, so generous spacing costs nothing.

use crate::graph::{Graph, NodeId, Pos};
use std::collections::{BTreeMap, BTreeSet};

/// Horizontal step between columns. `> CARD_W` (190), so two adjacent columns
/// never touch.
const DX: f32 = 220.0;
/// Vertical step between two nodes sharing a column. `>` the tallest card, so
/// stacked cards never touch. (A straight chain has one node per column, so
/// this never stretches it — it only spreads siblings that share a column.)
const DY: f32 = 260.0;
/// Vertical gap between two components' bands. `>` a small card's height, so a
/// one-node band never overlaps the band above it.
const BAND_GAP: f32 = 200.0;
/// The top-left origin of the whole arrangement.
const X0: f32 = 60.0;
const Y0: f32 = 60.0;
/// Barycenter crossing-reduction passes. Four down/up sweeps converge on the
/// small graphs this lays out; more buys nothing measurable.
const SWEEPS: usize = 4;

/// Lay every node in `g` out in non-overlapping layered bands (see the module
/// docs). Positions only — nothing else in the graph is touched.
pub fn arrange(g: &mut Graph) {
    for (n, p) in plan(g) {
        g.set_pos(n, p);
    }
}

/// The layout as a list of `(node, position)`. Pure, so it is unit-testable
/// without a mutable graph; [`arrange`] just applies it.
pub fn plan(g: &Graph) -> Vec<(NodeId, Pos)> {
    let nodes: Vec<NodeId> = g.nodes().iter().map(|n| n.id).collect();
    if nodes.is_empty() {
        return Vec::new();
    }
    // Forward/reverse adjacency for COLUMNS (delayed excluded: a `pre` edge is
    // feedback and must not order the layout); undirected adjacency for
    // CONNECTIVITY (delayed included: it still joins the two cards into one
    // visual component).
    let mut fwd: BTreeMap<NodeId, Vec<NodeId>> = BTreeMap::new();
    let mut rev: BTreeMap<NodeId, Vec<NodeId>> = BTreeMap::new();
    let mut undirected: BTreeMap<NodeId, Vec<NodeId>> = BTreeMap::new();
    for e in g.edges() {
        let (u, _) = e.from;
        let (v, _) = e.to;
        undirected.entry(u).or_default().push(v);
        undirected.entry(v).or_default().push(u);
        if !e.delayed {
            fwd.entry(u).or_default().push(v);
            rev.entry(v).or_default().push(u);
        }
    }

    let mut out = Vec::with_capacity(nodes.len());
    let mut band_top = Y0;
    for comp in components(&nodes, &undirected) {
        let (placed, height) = layout_component(&comp, &fwd, &rev);
        for (n, mut p) in placed {
            p.y += band_top;
            out.push((n, p));
        }
        band_top += height + BAND_GAP;
    }
    out
}

/// Weakly-connected components, each in ascending-id order, the components
/// themselves ordered by their smallest node id (so the layout is stable and
/// the first-built subgraph lands in the top band).
fn components(nodes: &[NodeId], undirected: &BTreeMap<NodeId, Vec<NodeId>>) -> Vec<Vec<NodeId>> {
    let all: BTreeSet<NodeId> = nodes.iter().copied().collect();
    let mut seen: BTreeSet<NodeId> = BTreeSet::new();
    let mut comps: Vec<Vec<NodeId>> = Vec::new();
    // `all` iterates in ascending id order, so components come out min-id first.
    for &start in &all {
        if !seen.insert(start) {
            continue;
        }
        let mut stack = vec![start];
        let mut comp = Vec::new();
        while let Some(n) = stack.pop() {
            comp.push(n);
            for &m in undirected.get(&n).into_iter().flatten() {
                if all.contains(&m) && seen.insert(m) {
                    stack.push(m);
                }
            }
        }
        comp.sort();
        comps.push(comp);
    }
    comps
}

/// Lay one component out: longest-path columns, barycenter ordering within each
/// column, coordinates centred vertically. Returns the placements (band-local y,
/// starting at 0) and the band's height.
fn layout_component(
    comp: &[NodeId],
    fwd: &BTreeMap<NodeId, Vec<NodeId>>,
    rev: &BTreeMap<NodeId, Vec<NodeId>>,
) -> (Vec<(NodeId, Pos)>, f32) {
    let set: BTreeSet<NodeId> = comp.iter().copied().collect();

    // Longest-path layering by Kahn over the component's forward edges. The
    // stored graph is acyclic over non-delayed edges (`Graph::connect` enforces
    // it), so every node is reached; a node never reached (defensive) keeps
    // column 0.
    let mut indeg: BTreeMap<NodeId, usize> = comp.iter().map(|&n| (n, 0)).collect();
    for &u in comp {
        for &v in fwd.get(&u).into_iter().flatten() {
            if let Some(d) = indeg.get_mut(&v) {
                *d += 1;
            }
        }
    }
    let mut col: BTreeMap<NodeId, usize> = comp.iter().map(|&n| (n, 0)).collect();
    let mut ready: Vec<NodeId> = comp.iter().copied().filter(|n| indeg[n] == 0).collect();
    let mut i = 0;
    while i < ready.len() {
        let u = ready[i];
        i += 1;
        let cu = col[&u];
        for &v in fwd.get(&u).into_iter().flatten() {
            if !set.contains(&v) {
                continue;
            }
            if cu + 1 > col[&v] {
                col.insert(v, cu + 1);
            }
            if let Some(d) = indeg.get_mut(&v) {
                *d -= 1;
                if *d == 0 {
                    ready.push(v);
                }
            }
        }
    }

    // Bucket into columns, initialised in ascending id order (stable seed).
    let n_cols = col.values().copied().max().unwrap_or(0) + 1;
    let mut columns: Vec<Vec<NodeId>> = vec![Vec::new(); n_cols];
    for &n in comp {
        columns[col[&n]].push(n);
    }

    // Crossing reduction: alternate down (order a column by its predecessors'
    // positions) and up (by its successors') sweeps.
    for _ in 0..SWEEPS {
        sweep(&mut columns, rev, true);
        sweep(&mut columns, fwd, false);
    }

    let tallest = columns.iter().map(Vec::len).max().unwrap_or(1).max(1);
    let height = (tallest as f32 - 1.0) * DY;
    let mut placed = Vec::with_capacity(comp.len());
    for (ci, column) in columns.iter().enumerate() {
        // Centre each column's stack within the band's height.
        let top = (tallest as f32 - column.len() as f32) * DY / 2.0;
        for (k, &n) in column.iter().enumerate() {
            placed.push((
                n,
                Pos {
                    x: X0 + ci as f32 * DX,
                    y: top + k as f32 * DY,
                },
            ));
        }
    }
    (placed, height)
}

/// One barycenter sweep: reorder each column by the median position of its
/// neighbours in the already-fixed adjacent column. `down` orders column `l` by
/// its predecessors in `l-1` (pass `rev`); `!down` by its successors in `l+1`
/// (pass `fwd`).
fn sweep(columns: &mut [Vec<NodeId>], neighbors: &BTreeMap<NodeId, Vec<NodeId>>, down: bool) {
    let n = columns.len();
    if n < 2 {
        return;
    }
    let order: Vec<usize> = if down {
        (1..n).collect()
    } else {
        (0..n - 1).rev().collect()
    };
    for l in order {
        let adj = if down { l - 1 } else { l + 1 };
        // The adjacent column is fixed this sweep; snapshot its positions.
        let pos: BTreeMap<NodeId, usize> = columns[adj]
            .iter()
            .enumerate()
            .map(|(i, &x)| (x, i))
            .collect();
        reorder(&mut columns[l], &pos, neighbors);
    }
}

/// Stable-sort `column` by each node's median neighbour position. A node with no
/// neighbours in the adjacent column keeps its current index as key, so it stays
/// put instead of jumping to the front.
fn reorder(
    column: &mut Vec<NodeId>,
    pos_in_adj: &BTreeMap<NodeId, usize>,
    neighbors: &BTreeMap<NodeId, Vec<NodeId>>,
) {
    let cur: BTreeMap<NodeId, usize> = column.iter().enumerate().map(|(i, &n)| (n, i)).collect();
    let mut keyed: Vec<(f32, NodeId)> = column
        .iter()
        .map(|&n| {
            let mut ps: Vec<usize> = neighbors
                .get(&n)
                .into_iter()
                .flatten()
                .filter_map(|m| pos_in_adj.get(m).copied())
                .collect();
            let key = if ps.is_empty() {
                cur[&n] as f32
            } else {
                ps.sort_unstable();
                let mid = ps.len() / 2;
                if ps.len() % 2 == 1 {
                    ps[mid] as f32
                } else {
                    (ps[mid - 1] + ps[mid]) as f32 / 2.0
                }
            };
            (key, n)
        })
        .collect();
    // `sort_by` is stable: nodes with equal keys keep their current relative order.
    keyed.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap_or(std::cmp::Ordering::Equal));
    *column = keyed.into_iter().map(|(_, n)| n).collect();
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::graph::Edge;

    fn wire(g: &mut Graph, a: NodeId, b: NodeId) {
        g.connect(Edge {
            from: (a, 0),
            to: (b, 0),
            delayed: false,
        })
        .expect("edge");
    }

    /// Positions as a lookup, for asserting geometry.
    fn placed(g: &Graph) -> BTreeMap<NodeId, Pos> {
        plan(g).into_iter().collect()
    }

    #[test]
    fn a_chain_lays_out_as_one_straight_horizontal_line() {
        let mut g = Graph::new();
        let a = g.add_node("motion.grid");
        let b = g.add_node("motion.move");
        let c = g.add_node("motion.drive");
        let d = g.add_node("motion.output");
        wire(&mut g, a, b);
        wire(&mut g, b, c);
        wire(&mut g, c, d);
        let p = placed(&g);

        // One node per column ⇒ every card on the SAME y, strictly increasing x.
        let ys: BTreeSet<u32> = [a, b, c, d].iter().map(|n| p[n].y.to_bits()).collect();
        assert_eq!(ys.len(), 1, "a chain is a single straight line");
        assert!(p[&a].x < p[&b].x && p[&b].x < p[&c].x && p[&c].x < p[&d].x);
        // Columns are exactly one DX apart (no overlap; the source is leftmost).
        assert!((p[&b].x - p[&a].x - DX).abs() < 0.5);
    }

    #[test]
    fn a_branch_stacks_siblings_in_one_column_at_distinct_non_overlapping_y() {
        // a → {b, c} → d : the fork and join each own a column; the siblings
        // share the middle column and must be spread so their cards never touch.
        let mut g = Graph::new();
        let a = g.add_node("motion.grid");
        let b = g.add_node("value.lfo");
        let c = g.add_node("value.noise");
        let d = g.add_node("motion.drive");
        // Fork from a's one output; join into d's TWO input ports (one edge per input).
        wire(&mut g, a, b);
        wire(&mut g, a, c);
        for (from, to_port) in [(b, 0u16), (c, 1u16)] {
            g.connect(Edge {
                from: (from, 0),
                to: (d, to_port),
                delayed: false,
            })
            .expect("edge");
        }
        let p = placed(&g);

        // b and c are in the SAME column (both one step from the source)…
        assert!((p[&b].x - p[&c].x).abs() < 0.5, "siblings share a column");
        // …but at least DY apart, so the cards do not overlap.
        assert!(
            (p[&b].y - p[&c].y).abs() >= DY - 0.5,
            "siblings are spread so their cards never overlap"
        );
        // The fork sits left of the siblings, the join to their right.
        assert!(p[&a].x < p[&b].x && p[&b].x < p[&d].x);
    }

    #[test]
    fn two_disconnected_chains_become_two_non_overlapping_bands() {
        let mut g = Graph::new();
        let a1 = g.add_node("motion.grid");
        let a2 = g.add_node("motion.output");
        wire(&mut g, a1, a2);
        let b1 = g.add_node("motion.grid");
        let b2 = g.add_node("motion.output");
        wire(&mut g, b1, b2);
        let p = placed(&g);

        let band_a = p[&a1].y.max(p[&a2].y);
        let band_b = p[&b1].y.min(p[&b2].y);
        // The whole second component sits a clear BAND_GAP below the first —
        // the two rows never share vertical space.
        assert!(
            band_b - band_a >= BAND_GAP - 1.0,
            "components stack into separate bands (gap {})",
            band_b - band_a
        );
    }

    #[test]
    fn a_pre_edge_is_ignored_for_columns_but_keeps_the_band_together() {
        // a → b → c with a feedback c ⇢ a (delayed). Columns must ignore the
        // feedback (c is still rightmost, no cycle, no stretched layout), and
        // all three stay in ONE band.
        let mut g = Graph::new();
        let a = g.add_node("sim.zone");
        let b = g.add_node("force.wind");
        let c = g.add_node("sim.step");
        wire(&mut g, a, b);
        wire(&mut g, b, c);
        g.connect(Edge {
            from: (c, 0),
            to: (a, 0),
            delayed: true,
        })
        .expect("pre edge");
        let p = placed(&g);

        // Forward order preserved despite the back-edge; exactly three columns.
        assert!(p[&a].x < p[&b].x && p[&b].x < p[&c].x);
        assert!((p[&c].x - p[&a].x - 2.0 * DX).abs() < 0.5, "three columns, no more");
        // One weakly-connected component ⇒ one band: no BAND_GAP jump inside it.
        let span = [a, b, c]
            .iter()
            .map(|n| p[n].y)
            .fold(f32::MIN, f32::max)
            - [a, b, c].iter().map(|n| p[n].y).fold(f32::MAX, f32::min);
        assert!(span < BAND_GAP, "the feedback loop stays in one band");
    }

    #[test]
    fn arrange_moves_the_stored_positions() {
        // Positive control: `arrange` actually writes the plan into the graph.
        let mut g = Graph::new();
        let a = g.add_node("motion.grid");
        let b = g.add_node("motion.output");
        wire(&mut g, a, b);
        g.set_pos(a, Pos { x: 999.0, y: 999.0 });
        arrange(&mut g);
        assert!(g.pos(a).expect("pos").x < 100.0, "arrange overwrote the stale position");
        assert!(g.pos(a).unwrap().x < g.pos(b).unwrap().x);
    }
}
