//! **Auto-layout: a layered (Sugiyama-style) arrangement of a node graph.**
//!
//! A hand-placed graph tangles as it grows: cards overlap, wires cross, and a
//! branchy sub-graph laid out as parallel rows collides with a neighbour that
//! was placed by hand at the same coordinates. [`arrange`] replaces every
//! node's position with a clean layout:
//!
//! - each **weakly-connected component** is laid out on its own,
//! - nodes are placed in **columns**: longest path from the sources, then pulled
//!   RIGHT until they sit next to what they feed,
//! - within a column they are **ordered to reduce crossings** (barycenter
//!   sweeps), with the **PORT** breaking every tie,
//! - their `y` is then **pulled to the average of their neighbours'**, so a join
//!   sits centred between the rows it joins and a chain stays straight,
//! - and no two **DRAWN BOXES** touch, in either direction.
//!
//! It moves POSITIONS only — it never touches nodes, edges, params, labels, or
//! the frozen node contract (ADR-0039): it is a sibling module riding the same
//! `layout` map the graph already stores, exactly like [`crate::gpu`] rides the
//! registry side-channel. A `pre` (delayed) edge is feedback: it is **ignored
//! for column ordering** (it would otherwise force a cycle) but still binds its
//! two endpoints into the same band.
//!
//! # The three laws the owner asked for (2026-09-20), and the input each needs
//!
//! ⭐⭐⭐ **1. A card that plugs into an UPPER slot is placed ABOVE — never with a
//! crossed wire.** The barycenter heuristic orders a column by the *median
//! position* of its neighbours in the adjacent column; when two cards feed the
//! SAME consumer those medians TIE, and the tie was broken by the seed order,
//! which is the key order, which means *nothing*. ⇒ every [`Wire`] carries the
//! **ports it lands on**, and the port is the secondary sort key: the card on
//! `in_port 0` sorts above the card on `in_port 1`, and symmetrically two
//! consumers of one producer sort by the `out_port` they hang off.
//!
//! ⭐⭐⭐ **2. A card sits next to what it FEEDS.** Longest-path layering puts
//! every source in column `0`, so a `source.shape` whose only consumer was four
//! columns away got drawn four columns away, with its wire crossing every card
//! in between (measured on the owner's `=120`: shape at `x = 60`, duplicator at
//! `x = 720`). ⇒ after the longest-path pass every non-sink is pulled right to
//! `min(col(successor)) − 1`, the classic as-late-as-possible layering. ⚠️ It can
//! only move a card RIGHT and it takes the `min`, so a source that also feeds a
//! shallow consumer stays where the shallow one needs it.
//!
//! ⭐⭐⭐ **3. «If there are two levels, the levels align at the CENTRE.»** The old
//! code put item `k` of a column at `k · DY` and centred the stack in the band —
//! a grid, not a layout: a card joining an upper row and a lower row landed on
//! whichever grid slot its index happened to name, and the source of a chain
//! could sit a whole `DY/2` off the row it feeds (measured: the rope card at
//! `y = 320` against its own `scale` at `190`). ⇒ once the ordering is fixed,
//! [`place_column`] pulls every card to the **average `y` of its already placed
//! neighbours** and then pushes the column apart just enough to clear the drawn
//! boxes, keeping the order. Passes alternate left→right and right→left and
//! **end on right→left**, so the head of a chain follows its tail: that is what
//! makes `rope → scale → move` come out on ONE straight row with the
//! `duplicator` centred between that row and the `shape` above it.
//!
//! ⛔⛔ **And the spacing needs a SIZE, because a card is not `CARD_W` wide.** Two
//! constants cannot express it: the pill a card draws when the graph is zoomed
//! out follows the **NAME** (2026-09-20) and a full card's height follows its
//! **ROW COUNT** plus its preview frame. A constant step of `220 × 260` is the
//! step for one particular card, and two long names side by side overlap by
//! construction. ⇒ every [`Item`] declares the [`Extent`] it DRAWS, relative to
//! the position this module returns, and the steps come from the widest and
//! tallest boxes actually involved. [`Extent::default`] is the historical card,
//! so a caller that does not measure gets exactly the old layout.
//!
//! The engine is [`plan_edges`], which lays out an **arbitrary item set** over
//! opaque `Ord + Copy` keys. [`plan`]/[`arrange`] are the graph-node wrappers; a
//! caller that folds groups into single cards (a motion subgraph, a collapsed
//! node) builds its own item set — one key per visible card — and applies the
//! result where each card is drawn.
//!
//! Spacing is in editor units, which are panel pixels at zoom 1.

use crate::graph::{Graph, NodeId, Pos};
use std::collections::{BTreeMap, BTreeSet};

/// Horizontal step between two columns of **default-sized** cards. Kept as the
/// name callers assert against; the real step is derived from the [`Extent`]s.
pub const DX: f32 = 220.0;
/// Vertical step between two **default-sized** cards sharing a column. Same
/// deal: derived in general, exactly this number when nobody measures.
pub const DY: f32 = 260.0;
/// Vertical gap between two components' bands.
pub const BAND_GAP: f32 = 200.0;
/// The clear gap left between the DRAWN boxes of two adjacent **columns**. It is
/// `DX` minus the historical card width (190), so the default extents reproduce
/// `DX` to the bit and a wider pill only ever pushes further apart.
pub const GAP_X: f32 = 30.0;
/// The clear gap between the DRAWN boxes of two **rows**. ⚠️ It is NOT `DY` minus
/// the tallest card — the tallest card stopped being a constant the day a card's
/// height started following its row count. It is the gap that makes two stacked
/// cards read as two cards, taken off the arrangement the owner drew by hand
/// (2026-09-20: two pills ~`40` units apart at that zoom).
pub const GAP_Y: f32 = 40.0;
/// The top-left origin of the whole arrangement.
const X0: f32 = 60.0;
const Y0: f32 = 60.0;
/// Barycenter crossing-reduction passes (ordering). Four down/up sweeps converge
/// on the small graphs this lays out; more buys nothing measurable.
const SWEEPS: usize = 4;
/// Coordinate passes (the `y` pull). Each is one left→right plus one right→left;
/// the chain in the owner's reference picture converges after the FIRST, and
/// four leave margin for a wide fan.
const COORD_PASSES: usize = 4;

/// **How big an item DRAWS**, as offsets from the position [`plan_edges`]
/// returns (the card's anchor — its top-left, in the historical sense).
///
/// ⚠️ `left`/`top` are normally `≤ 0`: a zoomed-out pill is **centred** on the
/// card box and pokes out on both sides, and a preview frame parked above the
/// header reaches up past the anchor. A caller that draws two regimes (a full
/// card up close, a name-sized pill far away) declares the **UNION** of the two,
/// because ONE set of positions has to be safe at every zoom.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Extent {
    /// Left edge, relative to the anchor.
    pub left: f32,
    /// Right edge, relative to the anchor.
    pub right: f32,
    /// Top edge, relative to the anchor.
    pub top: f32,
    /// Bottom edge, relative to the anchor.
    pub bottom: f32,
}

impl Default for Extent {
    /// The historical card: `190` wide by `220` tall, anchored at its top-left.
    /// With [`GAP_X`]/[`GAP_Y`] these reproduce [`DX`]/[`DY`] to the bit, which
    /// is what keeps a caller that does not measure on exactly the old layout.
    fn default() -> Self {
        Self {
            left: 0.0,
            right: DX - GAP_X,
            top: 0.0,
            bottom: DY - GAP_Y,
        }
    }
}

/// One card to place: its key and the box it draws.
#[derive(Clone, Copy, Debug)]
pub struct Item<K> {
    pub key: K,
    pub extent: Extent,
}

impl<K> Item<K> {
    /// An item drawn as the historical card — the door for a caller with no measurer.
    pub fn plain(key: K) -> Self {
        Self {
            key,
            extent: Extent::default(),
        }
    }
}

/// One connection, **with the slots it lands on**. The ports are what let the
/// layout honour law 1; a caller with none passes `0` and gets the old
/// tie-breaking (the seed order).
#[derive(Clone, Copy, Debug)]
pub struct Wire<K> {
    pub from: K,
    /// The producer's OUTPUT slot — orders two consumers of one producer.
    pub out_port: u16,
    pub to: K,
    /// The consumer's INPUT slot — orders two producers of one consumer.
    pub in_port: u16,
    /// A `pre` edge: feedback. Ignored for columns, still binds the band.
    pub delayed: bool,
}

/// Lay every node in `g` out in non-overlapping layered bands (see the module
/// docs). Positions only — nothing else in the graph is touched.
pub fn arrange(g: &mut Graph) {
    for (n, p) in plan(g) {
        g.set_pos(n, p);
    }
}

/// The graph's layout as a list of `(node, position)`, every card the default
/// size. Pure, so it is unit-testable without a mutable graph.
pub fn plan(g: &Graph) -> Vec<(NodeId, Pos)> {
    let items: Vec<Item<u64>> = g
        .nodes()
        .iter()
        .map(|n| Item::plain(u64::from(n.id.0)))
        .collect();
    let wires: Vec<Wire<u64>> = g
        .edges()
        .iter()
        .map(|e| Wire {
            from: u64::from(e.from.0.0),
            out_port: e.from.1,
            to: u64::from(e.to.0.0),
            in_port: e.to.1,
            delayed: e.delayed,
        })
        .collect();
    plan_edges(&items, &wires)
        .into_iter()
        .map(|(k, p)| (NodeId(k as u32), p))
        .collect()
}

/// **The layout engine.** Lay `items` out in non-overlapping layered bands given
/// `wires`. Generic over an opaque `Ord + Copy` key so a caller can mix
/// heterogeneous cards — a node and a collapsed group card — in one arrangement.
/// Wires naming a key not in `items` are ignored, so a caller can pass the whole
/// edge list and only the items it wants placed. Returns a position for every item.
pub fn plan_edges<K: Ord + Copy>(items: &[Item<K>], wires: &[Wire<K>]) -> Vec<(K, Pos)> {
    if items.is_empty() {
        return Vec::new();
    }
    let size: BTreeMap<K, Extent> = items.iter().map(|i| (i.key, i.extent)).collect();
    let keys: Vec<K> = items.iter().map(|i| i.key).collect();

    // Forward/reverse adjacency for COLUMNS (delayed excluded: a `pre` edge is
    // feedback and must not order the layout); undirected adjacency for
    // CONNECTIVITY (delayed included: it still joins the two cards into one
    // visual component).
    let mut fwd: BTreeMap<K, Vec<K>> = BTreeMap::new();
    let mut rev: BTreeMap<K, Vec<K>> = BTreeMap::new();
    let mut undirected: BTreeMap<K, Vec<K>> = BTreeMap::new();
    // The slot each neighbour relation lands on, keyed `(item, neighbour)` so the
    // sweeps look it up the same way in both directions. ⚠️ Two wires between the
    // same pair keep the SMALLEST slot: the topmost socket is the one whose
    // position the artist reads.
    let mut port_fwd: BTreeMap<(K, K), u16> = BTreeMap::new();
    let mut port_rev: BTreeMap<(K, K), u16> = BTreeMap::new();
    for w in wires {
        if !size.contains_key(&w.from) || !size.contains_key(&w.to) || w.from == w.to {
            continue;
        }
        undirected.entry(w.from).or_default().push(w.to);
        undirected.entry(w.to).or_default().push(w.from);
        if !w.delayed {
            fwd.entry(w.from).or_default().push(w.to);
            rev.entry(w.to).or_default().push(w.from);
            keep_min(&mut port_fwd, (w.from, w.to), w.in_port);
            keep_min(&mut port_rev, (w.to, w.from), w.out_port);
        }
    }

    let mut out = Vec::with_capacity(items.len());
    let mut band_top = Y0;
    for comp in components(&keys, &undirected) {
        let (placed, height) = layout_component(&comp, &size, &fwd, &rev, &port_fwd, &port_rev);
        for (n, mut p) in placed {
            p.y += band_top;
            out.push((n, p));
        }
        band_top += height + BAND_GAP;
    }
    out
}

fn keep_min<K: Ord + Copy>(m: &mut BTreeMap<(K, K), u16>, k: (K, K), v: u16) {
    m.entry(k).and_modify(|p| *p = (*p).min(v)).or_insert(v);
}

/// Weakly-connected components, each in ascending-key order, the components
/// themselves ordered by their smallest key (so the layout is stable and the
/// first-built subgraph lands in the top band).
fn components<K: Ord + Copy>(items: &[K], undirected: &BTreeMap<K, Vec<K>>) -> Vec<Vec<K>> {
    let all: BTreeSet<K> = items.iter().copied().collect();
    let mut seen: BTreeSet<K> = BTreeSet::new();
    let mut comps: Vec<Vec<K>> = Vec::new();
    // `all` iterates in ascending key order, so components come out min-key first.
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

/// Longest-path layering by Kahn over the component's forward edges, then the
/// **pull right** of law 2. Forward edges are acyclic (feedback is excluded
/// upstream), so every item is reached; an item never reached keeps column 0.
fn columns_of<K: Ord + Copy>(
    comp: &[K],
    fwd: &BTreeMap<K, Vec<K>>,
    set: &BTreeSet<K>,
) -> BTreeMap<K, usize> {
    let mut indeg: BTreeMap<K, usize> = comp.iter().map(|&n| (n, 0)).collect();
    for &u in comp {
        for &v in fwd.get(&u).into_iter().flatten() {
            if let Some(d) = indeg.get_mut(&v) {
                *d += 1;
            }
        }
    }
    let mut col: BTreeMap<K, usize> = comp.iter().map(|&n| (n, 0)).collect();
    let mut ready: Vec<K> = comp.iter().copied().filter(|n| indeg[n] == 0).collect();
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

    // ⭐ **Law 2 — pull right.** Walk in DESCENDING longest-path column: every
    // successor of `u` has a strictly greater one, so it is already final when
    // `u` is read. A sink has no successors and keeps its column. This can only
    // move a card right (its successors are at least one column past it), so it
    // never re-opens the acyclicity the pass above relies on.
    let mut by_col: Vec<K> = comp.to_vec();
    by_col.sort_by_key(|n| std::cmp::Reverse(col[n]));
    for u in by_col {
        let earliest_consumer = fwd
            .get(&u)
            .into_iter()
            .flatten()
            .filter(|v| set.contains(v))
            .map(|v| col[v])
            .min();
        if let Some(c) = earliest_consumer {
            col.insert(u, c - 1);
        }
    }
    col
}

/// Lay one component out. Returns the placements (band-local `y`, starting at 0)
/// and the band's height, which is the height of the DRAWN boxes, not of a grid.
fn layout_component<K: Ord + Copy>(
    comp: &[K],
    size: &BTreeMap<K, Extent>,
    fwd: &BTreeMap<K, Vec<K>>,
    rev: &BTreeMap<K, Vec<K>>,
    port_fwd: &BTreeMap<(K, K), u16>,
    port_rev: &BTreeMap<(K, K), u16>,
) -> (Vec<(K, Pos)>, f32) {
    let set: BTreeSet<K> = comp.iter().copied().collect();
    let col = columns_of(comp, fwd, &set);

    // Bucket into columns, initialised in ascending key order (stable seed).
    let n_cols = col.values().copied().max().unwrap_or(0) + 1;
    let mut columns: Vec<Vec<K>> = vec![Vec::new(); n_cols];
    for &n in comp {
        columns[col[&n]].push(n);
    }

    // Crossing reduction: alternate down (order a column by its predecessors'
    // positions) and up (by its successors'), the PORT breaking every tie.
    for _ in 0..SWEEPS {
        sweep(&mut columns, rev, port_rev, true);
        sweep(&mut columns, fwd, port_fwd, false);
    }

    // Horizontal: each step clears the widest box on the left against the
    // furthest-left box on the right. With default extents this is exactly `DX`.
    let mut xs = vec![X0; n_cols];
    for c in 1..n_cols {
        let right = columns[c - 1]
            .iter()
            .map(|n| size[n].right)
            .fold(f32::MIN, f32::max);
        let left = columns[c]
            .iter()
            .map(|n| size[n].left)
            .fold(f32::MAX, f32::min);
        xs[c] = xs[c - 1] + (right - left) + GAP_X;
    }

    // Vertical: seed each column as a clear stack, then pull everyone to the
    // average of their neighbours (law 3). The LAST half-pass is right→left, so
    // a chain's head follows its tail instead of the other way round.
    let mut y: BTreeMap<K, f32> = BTreeMap::new();
    for column in &columns {
        let mut acc = 0.0f32;
        for (k, n) in column.iter().enumerate() {
            if k > 0 {
                acc += size[&column[k - 1]].bottom + GAP_Y - size[n].top;
            }
            y.insert(*n, acc);
        }
    }
    for _ in 0..COORD_PASSES {
        for column in columns.iter().skip(1) {
            place_column(column, &mut y, size, rev);
        }
        for column in columns.iter().take(n_cols.saturating_sub(1)).rev() {
            place_column(column, &mut y, size, fwd);
        }
    }

    // Normalise the band: the top of the topmost DRAWN box sits at 0, and the
    // height is what the boxes actually span — so the next band clears them.
    let top = comp
        .iter()
        .map(|n| y[n] + size[n].top)
        .fold(f32::MAX, f32::min);
    let bottom = comp
        .iter()
        .map(|n| y[n] + size[n].bottom)
        .fold(f32::MIN, f32::max);
    let placed = comp
        .iter()
        .map(|&n| {
            (
                n,
                Pos {
                    x: xs[col[&n]],
                    y: y[&n] - top,
                },
            )
        })
        .collect();
    (placed, bottom - top)
}

/// One coordinate pass over a column: pull each card to the mean `y` of its
/// already-placed neighbours, then push the column apart so no two DRAWN boxes
/// touch — **keeping the order the sweeps chose**, which is what carries law 1
/// into the geometry.
///
/// ⚠️ The push walks downward from the first card, so the column can only drift
/// DOWN; the rigid shift afterwards puts the block back on the barycentre it
/// asked for. Shifting a whole column is separation-preserving by construction,
/// which is why the two steps can be done in this order at all.
fn place_column<K: Ord + Copy>(
    column: &[K],
    y: &mut BTreeMap<K, f32>,
    size: &BTreeMap<K, Extent>,
    neighbors: &BTreeMap<K, Vec<K>>,
) {
    if column.is_empty() {
        return;
    }
    let desired: Vec<f32> = column
        .iter()
        .map(|n| {
            let mut sum = 0.0f32;
            let mut count = 0.0f32;
            for m in neighbors.get(n).into_iter().flatten() {
                if let Some(v) = y.get(m) {
                    sum += *v;
                    count += 1.0;
                }
            }
            if count == 0.0 { y[n] } else { sum / count }
        })
        .collect();

    let mut placed = desired.clone();
    for k in 1..column.len() {
        let floor = placed[k - 1] + size[&column[k - 1]].bottom + GAP_Y - size[&column[k]].top;
        if placed[k] < floor {
            placed[k] = floor;
        }
    }
    #[expect(
        clippy::cast_precision_loss,
        reason = "a column holds a handful of cards"
    )]
    let n = column.len() as f32;
    let shift = desired.iter().sum::<f32>() / n - placed.iter().sum::<f32>() / n;
    for (k, key) in column.iter().enumerate() {
        y.insert(*key, placed[k] + shift);
    }
}

/// One barycenter sweep: reorder each column by the median position of its
/// neighbours in the already-fixed adjacent column. `down` orders column `l` by
/// its predecessors in `l-1` (pass `rev`); `!down` by its successors in `l+1`
/// (pass `fwd`).
fn sweep<K: Ord + Copy>(
    columns: &mut [Vec<K>],
    neighbors: &BTreeMap<K, Vec<K>>,
    ports: &BTreeMap<(K, K), u16>,
    down: bool,
) {
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
        let pos: BTreeMap<K, usize> = columns[adj]
            .iter()
            .enumerate()
            .map(|(i, &x)| (x, i))
            .collect();
        reorder(&mut columns[l], &pos, neighbors, ports);
    }
}

/// Stable-sort `column` by each item's median neighbour position, **the median
/// PORT breaking the tie** (law 1). An item with no neighbours in the adjacent
/// column keeps its current index as key, so it stays put instead of jumping to
/// the front.
fn reorder<K: Ord + Copy>(
    column: &mut Vec<K>,
    pos_in_adj: &BTreeMap<K, usize>,
    neighbors: &BTreeMap<K, Vec<K>>,
    ports: &BTreeMap<(K, K), u16>,
) {
    let cur: BTreeMap<K, usize> = column.iter().enumerate().map(|(i, &n)| (n, i)).collect();
    let mut keyed: Vec<(f32, f32, K)> = column
        .iter()
        .map(|&n| {
            let mut ps: Vec<usize> = Vec::new();
            let mut slots: Vec<u16> = Vec::new();
            for m in neighbors.get(&n).into_iter().flatten() {
                if let Some(i) = pos_in_adj.get(m) {
                    ps.push(*i);
                    slots.push(ports.get(&(n, *m)).copied().unwrap_or(0));
                }
            }
            if ps.is_empty() {
                #[expect(clippy::cast_precision_loss, reason = "a column index fits an f32")]
                let here = cur[&n] as f32;
                (here, 0.0, n)
            } else {
                (median_usize(&mut ps), median_u16(&mut slots), n)
            }
        })
        .collect();
    // `sort_by` is stable: items with equal keys keep their current relative order.
    keyed.sort_by(|a, b| {
        a.0.partial_cmp(&b.0)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then(a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal))
    });
    *column = keyed.into_iter().map(|(_, _, n)| n).collect();
}

#[expect(clippy::cast_precision_loss, reason = "column indices fit an f32")]
fn median_usize(v: &mut [usize]) -> f32 {
    v.sort_unstable();
    let mid = v.len() / 2;
    if v.len() % 2 == 1 {
        v[mid] as f32
    } else {
        (v[mid - 1] + v[mid]) as f32 / 2.0
    }
}

fn median_u16(v: &mut [u16]) -> f32 {
    v.sort_unstable();
    let mid = v.len() / 2;
    if v.len() % 2 == 1 {
        f32::from(v[mid])
    } else {
        f32::from(v[mid - 1] + v[mid]) / 2.0
    }
}

#[cfg(test)]
#[path = "layout_tests.rs"]
mod tests;
