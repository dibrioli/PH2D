//! The **binary s-t min-cut** on the implicit 4-connected pixel grid — the max-flow the
//! Colorize LazyBrush multiway is built from (`docs/Flip/09 §3`), and the régua of `§7.1`.
//!
//! Clean-room **Boykov–Kolmogorov** (PAMI 2004) — the published max-flow for grid graphs
//! of vision, which is exactly our 4-connected topology. On the unit-ish LazyBrush cut a
//! generic max-flow (Dinic / push-relabel) does Θ(diameter) phases over the whole grid and
//! is pathologically slow — that gap is precisely the number `§7.1` reserved. Single-threaded
//! and deterministic (fixed scan order, no `HashMap`, no parallelism inside a cut — HR-5: two
//! identical clicks give the same drawing). Integer capacities ⇒ the min-cut is exact.
//!
//! ⚠️ **The pixel grid, not yet the region graph.** `§7.1` MEASURED that one raw pixel-grid
//! cut is seconds+ at full res, so production must run the multiway over the trapped-ball
//! *region* graph (dozens of nodes, sub-ms). This grid solver is the correct §3 LazyBrush on
//! small art, the engine's current back-bone, and the §7.1 bench; the region reduction is the
//! documented perf follow-up. [`GridFlow`] would generalise to a region-adjacency graph.

use ph2d_flip_fill::{BOUNDARY, Grid};

const FREE: u8 = 0;
const S: u8 = 1; // source tree
const T: u8 = 2; // sink tree

/// `parent[i]` holds a neighbour direction `0..4` when the parent is a real node; these
/// two sentinels mark a tree root (attached straight to its terminal) and a
/// free/orphan node.
const P_TERMINAL: i8 = -1;
const P_NONE: i8 = -2;

/// The opposite of neighbour direction `d`. Layout: `0`=East, `1`=West, `2`=South,
/// `3`=North ⇒ the pairs flip with a single bit.
#[inline]
const fn opp(d: usize) -> usize {
    d ^ 1
}

/// The **implicit** 4-connected grid graph for one binary s-t cut. The grid is never
/// materialised as an edge list — neighbours are index arithmetic, so the memory is a
/// fixed handful of arrays indexed by pixel (`09 §3.2`).
pub struct GridFlow {
    w: usize,
    h: usize,
    /// `res[4*i + d]` = residual capacity of the arc `i → neighbour(i, d)`. An undirected
    /// n-link of weight `c` sets both `res[4*p+d]` and `res[4*q+opp(d)]` to `c`.
    res: Vec<i32>,
    /// Terminal residual, signed: `t[i] > 0` = residual `source → i`; `t[i] < 0` =
    /// residual `i → sink` (magnitude). A node touches at most one terminal.
    t: Vec<i32>,
    tree: Vec<u8>,
    /// A neighbour direction `0..4`, or [`P_TERMINAL`] / [`P_NONE`].
    parent: Vec<i8>,
    /// FIFO of nodes whose edges still need growing, with a membership flag so a node is
    /// never queued twice (determinism + no unbounded growth).
    active: std::collections::VecDeque<u32>,
    in_active: Vec<bool>,
    orphans: Vec<u32>,
    /// BK distance heuristic: `dist[i]` ≈ tree distance from `i` to its terminal root,
    /// valid iff `ts[i] == time`. Adopt picks the min-`dist` parent and caches the walk, so
    /// the trees stay shallow and the free/re-grow thrashing that makes a naïve BK
    /// super-linear (`09 §7.1`) is bounded. A stale `dist` only picks a worse *valid* parent
    /// — the flow value is unaffected (pinned by the gate).
    dist: Vec<u32>,
    ts: Vec<u64>,
    time: u64,
    /// Reused walk buffer for [`GridFlow::try_origin`] — no per-orphan allocation.
    scratch: Vec<u32>,
}

impl GridFlow {
    #[must_use]
    pub fn new(w: usize, h: usize) -> Self {
        let n = w * h;
        Self {
            w,
            h,
            res: vec![0; 4 * n],
            t: vec![0; n],
            tree: vec![FREE; n],
            parent: vec![P_NONE; n],
            active: std::collections::VecDeque::new(),
            in_active: vec![false; n],
            orphans: Vec::new(),
            dist: vec![0; n],
            ts: vec![0; n],
            time: 0,
            scratch: Vec::new(),
        }
    }

    /// The neighbour of `i` in direction `d`, or `None` at the grid edge.
    #[inline]
    fn neighbour(&self, i: usize, d: usize) -> Option<usize> {
        let x = i % self.w;
        let y = i / self.w;
        match d {
            0 if x + 1 < self.w => Some(i + 1),
            1 if x > 0 => Some(i - 1),
            2 if y + 1 < self.h => Some(i + self.w),
            3 if y > 0 => Some(i - self.w),
            _ => None,
        }
    }

    /// Set an **undirected** n-link between `i` and its neighbour `d` to weight `c` (both
    /// directions). Cutting the pair costs `c`.
    #[inline]
    pub fn set_nlink(&mut self, i: usize, d: usize, c: i32) {
        if let Some(q) = self.neighbour(i, d) {
            self.res[4 * i + d] = c;
            self.res[4 * q + opp(d)] = c;
        }
    }

    /// Attach node `i` to the **source** (`to_src`) and/or **sink** (`to_snk`) terminals.
    /// The net signed residual is what the solver sees; a data term `D_p` maps to one of
    /// these (`09 §3`).
    #[inline]
    pub fn set_tlink(&mut self, i: usize, to_src: i32, to_snk: i32) {
        self.t[i] = to_src - to_snk;
    }

    #[inline]
    fn set_active(&mut self, i: usize) {
        if !self.in_active[i] {
            self.in_active[i] = true;
            self.active.push_back(i as u32);
        }
    }

    /// Residual, in the growth direction, of the arc between `p` (in its own tree) and its
    /// neighbour `q = neighbour(p, d)`. For an S-node the flow would run `p → q`; for a
    /// T-node it would run `q → p` (toward the sink).
    #[inline]
    fn tree_cap(&self, p: usize, d: usize, q: usize) -> i32 {
        if self.tree[p] == S {
            self.res[4 * p + d]
        } else {
            self.res[4 * q + opp(d)]
        }
    }

    /// Run the max-flow. Returns the flow value (= min-cut value, exact).
    pub fn max_flow(&mut self) -> i64 {
        let n = self.w * self.h;
        // Seed the trees from the terminals.
        for i in 0..n {
            if self.t[i] != 0 {
                self.tree[i] = if self.t[i] > 0 { S } else { T };
                self.parent[i] = P_TERMINAL;
                self.set_active(i);
            }
        }

        let mut flow: i64 = 0;
        while let Some((p, q, d)) = self.grow() {
            flow += self.augment(p, q, d);
            self.adopt_all();
        }
        flow
    }

    /// **Grow** phase: pull active nodes and extend their tree until an edge bridges the
    /// two trees. Returns `(p, q, d)` where `q = neighbour(p, d)` and `tree[p] != tree[q]`.
    fn grow(&mut self) -> Option<(usize, usize, usize)> {
        while let Some(pf) = self.active.front().copied() {
            let p = pf as usize;
            // A node may have gone free/orphan since it was queued; drop it lazily.
            if self.tree[p] == FREE {
                self.active.pop_front();
                self.in_active[p] = false;
                continue;
            }
            for d in 0..4 {
                let Some(q) = self.neighbour(p, d) else {
                    continue;
                };
                if self.tree_cap(p, d, q) <= 0 {
                    continue;
                }
                if self.tree[q] == FREE {
                    // Adopt q into p's tree; q's parent is p (direction q→p = opp(d)).
                    self.tree[q] = self.tree[p];
                    self.parent[q] = opp(d) as i8;
                    self.set_active(q);
                } else if self.tree[q] != self.tree[p] {
                    // Bridge found: keep p active (it may bridge again after augment).
                    return Some((p, q, d));
                }
            }
            // p is fully grown for now; it stays in its tree but leaves the queue.
            self.active.pop_front();
            self.in_active[p] = false;
        }
        None
    }

    /// The S-tree node, the T-tree node, and the S→T arc residual for the bridge `(p,q,d)`.
    #[inline]
    fn orient(&self, p: usize, q: usize, d: usize) -> (usize, usize, usize) {
        // Returns (s_node, t_node, arc_index) where arc_index is `4*s_node + dir(s→t)`.
        if self.tree[p] == S {
            (p, q, 4 * p + d)
        } else {
            (q, p, 4 * q + opp(d))
        }
    }

    /// **Augment** along the path source → s_node → t_node → sink. Pushes the bottleneck,
    /// saturates arcs, and severs any child whose parent arc hits zero (an orphan).
    fn augment(&mut self, p: usize, q: usize, d: usize) -> i64 {
        let (s_node, t_node, arc) = self.orient(p, q, d);

        // 1) bottleneck: the connecting arc, plus both tree branches to their terminals.
        let mut bottleneck = self.res[arc];
        // S branch: walk s_node → source root, tree edge is parent→child.
        {
            let mut n = s_node;
            loop {
                let pd = self.parent[n];
                if pd == P_TERMINAL {
                    bottleneck = bottleneck.min(self.t[n]); // source residual, > 0
                    break;
                }
                let pd = pd as usize;
                let par = self.neighbour(n, pd).expect("tree parent exists");
                // arc parent→child = par → n, direction opp(pd) from par.
                bottleneck = bottleneck.min(self.res[4 * par + opp(pd)]);
                n = par;
            }
        }
        // T branch: walk t_node → sink root, tree edge is child→parent (toward sink).
        {
            let mut n = t_node;
            loop {
                let pd = self.parent[n];
                if pd == P_TERMINAL {
                    bottleneck = bottleneck.min(-self.t[n]); // sink residual, > 0
                    break;
                }
                let pd = pd as usize;
                let par = self.neighbour(n, pd).expect("tree parent exists");
                // arc child→parent = n → par, direction pd from n.
                bottleneck = bottleneck.min(self.res[4 * n + pd]);
                n = par;
            }
        }

        let b = bottleneck;
        debug_assert!(b > 0, "augmenting path must carry positive flow");

        // 2) push `b`. Reverse residuals go up by the same amount (undirected arcs).
        self.res[arc] -= b;
        let rev = self.rev_arc(arc);
        self.res[rev] += b;

        // S branch: from s_node up to the source root.
        {
            let mut n = s_node;
            loop {
                let pd = self.parent[n];
                if pd == P_TERMINAL {
                    self.t[n] -= b;
                    if self.t[n] == 0 {
                        self.make_orphan(n);
                    }
                    break;
                }
                let pd = pd as usize;
                let par = self.neighbour(n, pd).expect("tree parent exists");
                let fwd = 4 * par + opp(pd); // par → n
                let rev = self.rev_arc(fwd);
                self.res[fwd] -= b;
                self.res[rev] += b;
                if self.res[fwd] == 0 {
                    self.make_orphan(n); // n lost its parent arc
                }
                n = par;
            }
        }
        // T branch: from t_node down to the sink root.
        {
            let mut n = t_node;
            loop {
                let pd = self.parent[n];
                if pd == P_TERMINAL {
                    self.t[n] += b;
                    if self.t[n] == 0 {
                        self.make_orphan(n);
                    }
                    break;
                }
                let pd = pd as usize;
                let par = self.neighbour(n, pd).expect("tree parent exists");
                let fwd = 4 * n + pd; // n → par
                let rev = self.rev_arc(fwd);
                self.res[fwd] -= b;
                self.res[rev] += b;
                if self.res[fwd] == 0 {
                    self.make_orphan(n);
                }
                n = par;
            }
        }

        i64::from(b)
    }

    /// The reverse of an arc index `4*i + d`: `4*q + opp(d)`.
    #[inline]
    fn rev_arc(&self, arc: usize) -> usize {
        let i = arc / 4;
        let d = arc % 4;
        let q = self.neighbour(i, d).expect("arc has a head");
        4 * q + opp(d)
    }

    #[inline]
    fn make_orphan(&mut self, n: usize) {
        self.parent[n] = P_NONE;
        self.orphans.push(n as u32);
    }

    /// **Adopt** phase: re-attach every orphan to a valid parent in its own tree, or free
    /// it (and cascade to its children).
    fn adopt_all(&mut self) {
        // One cache generation per augment: every origin walk this round shares the stamps.
        self.time += 1;
        while let Some(nf) = self.orphans.pop() {
            self.adopt(nf as usize);
        }
    }

    fn adopt(&mut self, n: usize) {
        let mytree = self.tree[n];
        debug_assert!(mytree != FREE);
        // Find the valid parent CLOSEST to the terminal — a same-tree neighbour whose arc
        // toward `n` still has residual and whose lineage reaches a terminal (not through
        // `n`). Min-`dist` keeps the trees shallow, which is what stops the thrashing
        // (`09 §7.1`).
        let mut best: Option<(usize, u32)> = None;
        for d in 0..4 {
            let Some(q) = self.neighbour(n, d) else {
                continue;
            };
            if self.tree[q] != mytree {
                continue;
            }
            // Residual of the arc q → n (parent → child in growth direction).
            let cap = if mytree == S {
                self.res[4 * q + opp(d)]
            } else {
                self.res[4 * n + d]
            };
            if cap <= 0 {
                continue;
            }
            if let Some(dq) = self.try_origin(q, n)
                && best.is_none_or(|(_, bd)| dq < bd)
            {
                best = Some((d, dq));
            }
        }
        if let Some((d, dq)) = best {
            self.parent[n] = d as i8; // n's parent is q, direction n→q = d
            self.dist[n] = dq + 1;
            self.ts[n] = self.time;
            return;
        }
        // No parent: free `n`. Its same-tree neighbours become candidates to re-grow into
        // it, and its children lose their parent.
        for d in 0..4 {
            let Some(q) = self.neighbour(n, d) else {
                continue;
            };
            if self.tree[q] != mytree {
                continue;
            }
            // A neighbour that could grow back into n → re-activate it.
            let cap = if mytree == S {
                self.res[4 * q + opp(d)]
            } else {
                self.res[4 * n + d]
            };
            if cap > 0 {
                self.set_active(q);
            }
            // A child of n (its parent points back at n) becomes an orphan.
            if self.parent[q] != P_TERMINAL
                && self.parent[q] != P_NONE
                && self.neighbour(q, self.parent[q] as usize) == Some(n)
            {
                self.make_orphan(q);
            }
        }
        self.tree[n] = FREE;
        self.parent[n] = P_NONE;
        // Leave `in_active` as it is: if `n` is still in the queue, `grow()` drops it
        // lazily (it skips FREE fronts); clearing the flag here would let `set_active`
        // enqueue a duplicate.
    }

    /// Follow the parents of `q` to a validated node or a terminal, returning its distance
    /// to the root (for the min-`dist` parent choice) — or `None` if the lineage passes
    /// through the orphan `avoid` or a free node. Validated nodes are cached by `(ts, dist)`
    /// for the round, so the next orphan short-circuits. This is the BK origin check; the
    /// walk is its exact meaning, and it can only ever return a VALID parent (one that
    /// reaches a terminal), so the flow value is unaffected by the `dist` estimate.
    fn try_origin(&mut self, q: usize, avoid: usize) -> Option<u32> {
        self.scratch.clear();
        let mut n = q;
        let root_dist = loop {
            if n == avoid {
                return None; // lineage passes through the orphan we're adopting
            }
            if self.ts[n] == self.time {
                break self.dist[n]; // already validated this round
            }
            match self.parent[n] {
                P_TERMINAL => {
                    self.ts[n] = self.time;
                    self.dist[n] = 0;
                    break 0;
                }
                P_NONE => return None, // reached a free/orphan node → no valid origin
                pd => {
                    self.scratch.push(n as u32);
                    n = self.neighbour(n, pd as usize).expect("tree parent exists");
                }
            }
        };
        // Stamp the walked nodes with their distance to the root (`q` is farthest).
        let len = self.scratch.len() as u32;
        for (k, &idx) in self.scratch.iter().enumerate() {
            let m = idx as usize;
            self.dist[m] = root_dist + len - k as u32;
            self.ts[m] = self.time;
        }
        Some(root_dist + len)
    }

    /// After `max_flow`, the nodes still reachable from the source in the residual graph —
    /// the **source side** of the min-cut. Used by the correctness gate; the shipping
    /// motor would read the labels here.
    #[must_use]
    pub fn source_side(&self) -> Vec<bool> {
        let n = self.w * self.h;
        let mut side = vec![false; n];
        let mut stack = Vec::new();
        // `i` seeds both `side` and the `stack` — a range loop is the clear form.
        #[allow(clippy::needless_range_loop)]
        for i in 0..n {
            if self.t[i] > 0 {
                side[i] = true;
                stack.push(i);
            }
        }
        while let Some(i) = stack.pop() {
            for d in 0..4 {
                if self.res[4 * i + d] > 0
                    && let Some(q) = self.neighbour(i, d)
                    && !side[q]
                {
                    side[q] = true;
                    stack.push(q);
                }
            }
        }
        side
    }
}

/// The LazyBrush **binary** instance on a product [`Grid`]: label A (source) vs label B
/// (sink). `V_pq` (smoothness) is the clarity of paper between adjacent pixels — `v_ink` to
/// cut across the line, `v_white` across paper — so the cut is *attracted* to the ink and a
/// gap need not close (`09 §3`). `D_p` (data) is the scribble pixels, weighted `K = 2(w+h)`
/// so no cut betrays a scribble (any boundary is bounded by the grid perimeter).
///
/// The two weights are the CALLER's: the engine passes `v_ink = 0` (cutting through the line
/// is free ⇒ the flood is confined by it ⇒ a scribble colours exactly its region), while the
/// `§7.1` bench passes `v_ink = 1` to force a non-trivial worst-case flow along the whole
/// boundary. ⚠️ **The scribbles must be REGIONS, not single pixels** — a one-pixel seed lets
/// the cheapest cut "fence off that pixel" (`perimeter · v_white`) instead of the region cut;
/// a scribble whose perimeter exceeds the gap it must beat forces the real region answer.
///
/// ⚠️ v1 reads `V_pq` from the `BOUNDARY` **bit**, not a coverage float: `ink.rs` marks a bit
/// today, and the analytic-coverage `V_pq` of `09 §3.1` is a later refinement.
#[must_use]
pub fn lazybrush_binary(
    grid: &Grid,
    source: &[usize],
    sink: &[usize],
    v_white: i32,
    v_ink: i32,
) -> GridFlow {
    let w = grid.w;
    let h = grid.h;
    let mut f = GridFlow::new(w, h);

    let k = 2 * (w + h) as i32; // scribble weight — dominates the perimeter (`09 §3`)

    let is_ink = |i: usize| grid.flags[i] & BOUNDARY != 0;
    // n-links: East and South only (each undirected pair is set once).
    for i in 0..w * h {
        for d in [0usize, 2] {
            if let Some(q) = f.neighbour(i, d) {
                let c = if is_ink(i) || is_ink(q) {
                    v_ink
                } else {
                    v_white
                };
                f.set_nlink(i, d, c);
            }
        }
    }
    // t-links: the two scribbles. A → source, B → sink.
    for &p in source {
        f.set_tlink(p, k, 0);
    }
    for &p in sink {
        f.set_tlink(p, 0, k);
    }
    f
}

#[cfg(test)]
#[path = "flow_tests.rs"]
mod tests;
