//! **The graph shows what is alive** (Motion Nodes F3) — which cards the cook actually pulls,
//! which wires have data moving through them right now, and how much of it.
//!
//! Three readings, all of them things the artist can otherwise only get by aiming a probe at
//! one node at a time:
//!
//! ## 1. Live vs inert — `live_set`
//!
//! The cook is **pull-based**: it starts at the sinks (`motion.output`) and evaluates only
//! what they reach. Houdini's network is the same shape — *"a geometry network only cooks the
//! nodes necessary to generate the node with the display flag"* — and so a branch that reaches
//! no sink is not "slow", it is **not running at all**. That is the single most useful thing
//! this editor can say, and it is a reachability question, not a cook question: walk BACKWARDS
//! from the sinks.
//!
//! Backwards is the whole point. Walking forwards from the sources marks everything the
//! sources *feed*, which includes every dead end they feed into — the exact opposite of the
//! question. And a wire is live iff **what it feeds** is live (its source is then live by
//! construction): a wire out of a live node into a dead one goes nowhere, and says so.
//!
//! `pre` (delayed) edges count as edges here: the state loop is real consumption, and a node
//! that feeds only a `pre` head is very much cooking.
//!
//! ## 2. Flowing vs still — the marching dash (drawn from `hot`)
//!
//! TouchDesigner's reading, ported: *"when you see the wires between nodes animating (dashed
//! lines animating), it means the upstream node is cooking"*. TD only cooks what is dirty, so
//! an animating wire there means new data. Our cook re-pulls every frame from the playhead, so
//! the faithful port of "cooking" is **the value CHANGED** (the shell's digest, `hot`) — not
//! "was evaluated", which would set the whole canvas on fire and mean nothing.
//!
//! A wired, live, constant branch is drawn STILL. That is not an omission: nothing is flowing.
//!
//! ## 3. How much — the taper
//!
//! Wire width grows with the mass of the stream it carries (`count`), so a scatter that fans
//! 12 points into 5 000 shows it in the wire, and a scalar VALUE wire is visibly a thread. The
//! curve is `sqrt` (not linear): the interesting decades are the low ones — 12 → 200 must be
//! legible, and 4 000 → 8 000 need not be.

use crate::snapshot::GraphViewSnapshot;
use std::collections::BTreeSet;

/// Stream mass at which a wire is drawn at full width. Beyond it the wire stops growing —
/// past a few thousand instances the artist wants "a lot", not a bar chart.
const MASS_REF: f32 = 4096.0; // LITERAL-PX-OK: instance count mapped to the widest wire
const W_THREAD: f32 = 1.8; // LITERAL-PX-OK: wire width carrying a single value
const W_HEAVY: f32 = 5.2; // LITERAL-PX-OK: wire width at MASS_REF instances or more

/// Dash geometry, in graph units (scaled by zoom at the call site).
pub(crate) const DASH_PERIOD: f32 = 15.0; // LITERAL-PX-OK: dash + gap along a marching wire
pub(crate) const DASH_ON: f32 = 6.0; // LITERAL-PX-OK: the lit part of one dash
/// How fast the dashes march, in graph units per second. Slow enough to read as *direction*
/// (source → target), fast enough to read as *alive*.
pub(crate) const DASH_SPEED: f32 = 42.0; // LITERAL-PX-OK: dash march speed (units/second)

/// **The nodes the cook actually pulls** — reachability BACKWARDS from the sinks.
///
/// Everything else is inert: it is not slow, it is not running. Cost is one pass per live node
/// over the edge list (documents are tens of nodes, and this replaces a per-node cook lookup
/// that would have been wrong for scoped lanes anyway).
pub(crate) fn live_set(snap: &GraphViewSnapshot) -> BTreeSet<u32> {
    let mut live: BTreeSet<u32> = snap
        .nodes
        .iter()
        .filter(|n| n.is_sink)
        .map(|n| n.id)
        .collect();
    let mut frontier: Vec<u32> = live.iter().copied().collect();
    while let Some(n) = frontier.pop() {
        for e in snap.edges.iter().filter(|e| e.to_node == n) {
            if live.insert(e.from_node) {
                frontier.push(e.from_node);
            }
        }
    }
    live
}

/// A wire is live iff **what it feeds** is live. (Its source is then live too, by the way
/// `live_set` is built — so this single test is the whole answer.)
pub(crate) fn edge_is_live(live: &BTreeSet<u32>, to_node: u32) -> bool {
    live.contains(&to_node)
}

/// **Everything the selection touches**: what feeds it (its ancestors) and what it changes (its
/// descendants), plus the selection itself. The rest of the canvas recedes.
///
/// This is the question you ask a big graph before you dare edit it — *if I change this, what
/// moves?* — and the one that is hardest to answer by eye, because the wire you must follow is
/// exactly the one that disappears behind three other cards.
///
/// It is a STRUCTURAL walk over the edges, not the per-attribute walk the F3 plan sketched
/// ("BFS por AttrAccess"): which attributes a node reads and writes is not in `NodeManifest`,
/// and `NodeManifest` is a FROZEN contract (§6). A finer influence would buy "this node moves P
/// but not tint" — worth an ADR one day, worth nothing to buy by breaking the freeze today.
///
/// **Two DIRECTED walks, not one undirected one.** Walking "both ways" from every node reached
/// is the connected COMPONENT, not the influence: it climbs to the Output and then walks right
/// back down every other branch that feeds it, and lights up a graph the selection cannot touch
/// (the guard caught exactly this). Ancestors expand only upstream; descendants only
/// downstream; the influence is their union with the selection.
pub(crate) fn influence_set(snap: &GraphViewSnapshot, selected: &BTreeSet<u32>) -> BTreeSet<u32> {
    let mut inf: BTreeSet<u32> = selected.clone();
    for up in [true, false] {
        let mut frontier: Vec<u32> = selected.iter().copied().collect();
        while let Some(n) = frontier.pop() {
            for e in &snap.edges {
                let next = match up {
                    true if e.to_node == n => e.from_node,  // what feeds n
                    false if e.from_node == n => e.to_node, // what n feeds
                    _ => continue,
                };
                if inf.insert(next) {
                    frontier.push(next);
                }
            }
        }
    }
    inf
}

/// A wire belongs to the influence iff **both its ends do**. A wire leaving an ancestor towards
/// something unrelated carries data the selection never sees, and lighting it up would be the
/// answer to a question nobody asked.
pub(crate) fn edge_in_influence(inf: &BTreeSet<u32>, from_node: u32, to_node: u32) -> bool {
    inf.contains(&from_node) && inf.contains(&to_node)
}

/// Wire width from the mass of the stream it carries. `None` (never cooked) draws as a thread:
/// an inert wire carries nothing, and pretending otherwise would give a dead branch presence.
pub(crate) fn wire_width(count: Option<u32>) -> f32 {
    let Some(c) = count else {
        return W_THREAD;
    };
    let t = (c as f32 / MASS_REF).sqrt().clamp(0.0, 1.0); // CLAMP-OK: const bounds, min<max
    W_THREAD + (W_HEAVY - W_THREAD) * t
}

/// The lit stretches of a marching wire: the polyline cut into dashes, in arclength space,
/// offset by `phase`.
///
/// The pattern's origin ADVANCES with `phase`, so a phase that grows with time runs the dashes
/// from the source towards the target — the direction the data goes, on a canvas that reads
/// left to right. (Signs are cheap to get backwards and expensive to notice: the guard
/// measures the direction, it does not eyeball it.) The dashes are cut out of the very
/// polyline the wire is drawn along (and hit-tested along, and knifed along), so they cannot
/// drift off the wire they belong to.
pub(crate) fn dashes(pts: &[(f32, f32)], period: f32, on: f32, phase: f32) -> Vec<Vec<(f32, f32)>> {
    if pts.len() < 2 || period <= 0.0 || on <= 0.0 {
        return Vec::new();
    }
    // Cumulative arclength at each vertex.
    let mut cum = Vec::with_capacity(pts.len());
    let mut total = 0.0f32;
    cum.push(0.0);
    for w in pts.windows(2) {
        total += dist(w[0], w[1]);
        cum.push(total);
    }
    if total <= 0.0 {
        return Vec::new();
    }

    let mut out = Vec::new();
    // Start one period BEFORE the wire, so the dash wrapping in at the source is drawn as the
    // partial it is instead of popping into existence. `rem_euclid` keeps the origin in
    // [-period, 0) for a phase of any sign or size, so the march never jumps.
    let mut s = phase.rem_euclid(period) - period;
    while s < total {
        let (a, b) = (s.max(0.0), (s + on).min(total));
        if b > a {
            out.push(sub_path(pts, &cum, a, b));
        }
        s += period;
    }
    out
}

/// The stretch of the polyline between two arclengths, with the vertices in between kept —
/// a dash across a bend has to bend with it.
fn sub_path(pts: &[(f32, f32)], cum: &[f32], a: f32, b: f32) -> Vec<(f32, f32)> {
    let mut path = vec![point_at(pts, cum, a)];
    for i in 0..pts.len() {
        if cum[i] > a && cum[i] < b {
            path.push(pts[i]);
        }
    }
    path.push(point_at(pts, cum, b));
    path
}

/// The point at arclength `s` along the polyline.
fn point_at(pts: &[(f32, f32)], cum: &[f32], s: f32) -> (f32, f32) {
    for i in 1..pts.len() {
        if cum[i] >= s {
            let span = cum[i] - cum[i - 1];
            let t = if span > 0.0 {
                (s - cum[i - 1]) / span
            } else {
                0.0
            };
            let (p, q) = (pts[i - 1], pts[i]);
            return (p.0 + (q.0 - p.0) * t, p.1 + (q.1 - p.1) * t);
        }
    }
    *pts.last().expect("a polyline has at least one point")
}

fn dist(a: (f32, f32), b: (f32, f32)) -> f32 {
    let (dx, dy) = (b.0 - a.0, b.1 - a.1);
    (dx * dx + dy * dy).sqrt()
}

#[cfg(test)]
#[path = "flow_tests.rs"]
mod tests;
