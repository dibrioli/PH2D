//! The **binary s-t min-cut** on a general graph — the max-flow the Colorize LazyBrush
//! multiway is built from (`docs/Flip/09 §3`), and the régua of `§7.1`.
//!
//! Clean-room **Boykov–Kolmogorov** (PAMI 2004) — the published max-flow for the grid graphs
//! of vision, and the one whose grow/augment/adopt structure survives the move off the grid.
//! On the unit-ish LazyBrush cut a generic max-flow (Dinic / push-relabel) does Θ(diameter)
//! phases over the whole graph and is pathologically slow — that gap is precisely the number
//! `§7.1` reserved. Single-threaded and deterministic (fixed scan order, no `HashMap`, no
//! parallelism inside a cut — HR-5: two identical clicks give the same drawing). Integer
//! capacities ⇒ the min-cut is exact.
//!
//! # Why the topology is EXPLICIT (it used to be the pixel grid, implicitly)
//!
//! v1 hard-wired the 4-connected pixel grid: neighbours were index arithmetic and an arc was
//! `4·i + d`. That is the cheapest possible representation *of a grid*, and it made the
//! solver unable to express the graph the product actually needs — the **trapped-ball region
//! graph** (`§8`), whose nodes are regions and whose edges are shared borders of wildly
//! varying degree. `§7.1` measured why that matters: the raw pixel cut is **3,3 s at 4096²**,
//! and two scribbles contradicting each other across one line cost **157 s**.
//!
//! So the graph is now stored, and the grid is a *constructor* ([`Flow::grid_4conn`]) rather
//! than the structure. **There is exactly one solver** — the region cut and the pixel cut are
//! the same code on different adjacency, so they cannot answer differently (and the
//! 128-instance BK ≡ Edmonds–Karp gate now pins the engine the product ships).
//!
//! ## The two representation choices that keep it cheap
//!
//! - **Arcs live in PAIRS**: the reverse of arc `a` is `a ^ 1`, with no lookup and no stored
//!   index. (v1 got this by recomputing the neighbour — only a grid can afford that.)
//! - **`parent[n]` is an ARC, not a direction**: the arc whose tail is `n` and whose head is
//!   n's parent. Every walk in `augment`/`adopt`/`try_origin` then reads a residual with one
//!   index and no arithmetic — `parent_arc` for the child→parent direction, `^ 1` for
//!   parent→child.

// Só o ORÁCULO de pixels (`lazybrush_binary`) fala com a `Grid` — o solver é um grafo.
#[cfg(test)]
use ph2d_flip_fill::{BOUNDARY, Grid};

const FREE: u8 = 0;
const S: u8 = 1; // source tree
const T: u8 = 2; // sink tree

/// `parent[i]` holds an **arc index** when the parent is a real node; these two sentinels
/// mark a tree root (attached straight to its terminal) and a free/orphan node.
///
/// They sit at the top of the `u32` range, where a real arc index cannot reach: the arc count
/// is `2 ×` the edge count, and [`Flow::build`] refuses a graph that could collide with them.
const P_TERMINAL: u32 = u32::MAX;
const P_NONE: u32 = u32::MAX - 1;

/// The largest arc index that can never be mistaken for a sentinel.
const MAX_ARCS: usize = (u32::MAX - 2) as usize;

/// One binary s-t cut over an explicit graph (`09 §3.2`).
pub struct Flow {
    n: usize,
    /// `head[a]` = the node arc `a` points at. Arcs live in pairs: `a ^ 1` is the reverse.
    head: Vec<u32>,
    /// `res[a]` = residual capacity of arc `a`.
    res: Vec<i32>,
    /// CSR: the arcs whose **tail** is `p` are `inc[first[p]..first[p + 1]]`.
    first: Vec<u32>,
    inc: Vec<u32>,
    /// Terminal residual, signed: `t[i] > 0` = residual `source → i`; `t[i] < 0` =
    /// residual `i → sink` (magnitude). A node touches at most one terminal.
    t: Vec<i32>,
    tree: Vec<u8>,
    /// The arc from `i` to its parent, or [`P_TERMINAL`] / [`P_NONE`].
    parent: Vec<u32>,
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
    /// Reused walk buffer for [`Flow::try_origin`] — no per-orphan allocation.
    scratch: Vec<u32>,
}

impl Flow {
    /// Build the graph from `n` nodes and a list of **undirected** edges `(p, q, c)`: cutting
    /// the pair costs `c`, in either direction.
    ///
    /// One constructor, so the CSR can never be half-built — a `finish()` you can forget is a
    /// sentinel with no gate on its reader.
    #[must_use]
    pub fn build(n: usize, edges: impl IntoIterator<Item = (u32, u32, i32)>) -> Self {
        let mut head: Vec<u32> = Vec::new();
        let mut res: Vec<i32> = Vec::new();
        let mut deg = vec![0u32; n + 1];
        for (p, q, c) in edges {
            debug_assert!(
                (p as usize) < n && (q as usize) < n,
                "edge leaves the graph"
            );
            // The pair: `2k` is p→q, `2k+1` is q→p, so the reverse is always `a ^ 1`.
            head.push(q);
            res.push(c);
            head.push(p);
            res.push(c);
            deg[p as usize] += 1;
            deg[q as usize] += 1;
        }
        assert!(
            head.len() <= MAX_ARCS,
            "graph too large for the parent sentinels ({} arcs)",
            head.len()
        );

        // CSR by counting sort: prefix-sum the degrees, then place each arc at its tail.
        let mut first = vec![0u32; n + 1];
        let mut acc = 0u32;
        for i in 0..n {
            first[i] = acc;
            acc += deg[i];
        }
        first[n] = acc;
        let mut cursor = first.clone();
        let mut inc = vec![0u32; head.len()];
        for a in 0..head.len() {
            // The tail of `a` is the head of its partner.
            let tail = head[a ^ 1] as usize;
            inc[cursor[tail] as usize] = a as u32;
            cursor[tail] += 1;
        }

        Self {
            n,
            head,
            res,
            first,
            inc,
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

    /// The 4-connected pixel grid, as a graph: every East and South pair, weighted by
    /// `cap(i, q)`.
    ///
    /// ⚠️ **Não é produto — é o ORÁCULO.** Desde a pré-segmentação (`§8`) o `colorize` corta
    /// sobre o grafo de regiões; a instância de pixels sobrevive porque é a referência contra
    /// a qual aquele caminho é conferido, e a `§7.1` a mede. Deixá-la compilando no binário
    /// afirmaria que alguém a chama.
    #[cfg(test)]
    #[must_use]
    pub fn grid_4conn(w: usize, h: usize, cap: impl Fn(usize, usize) -> i32) -> Self {
        let mut edges = Vec::with_capacity(2 * w * h);
        for i in 0..w * h {
            let (x, y) = (i % w, i / w);
            if x + 1 < w {
                edges.push((i as u32, (i + 1) as u32, cap(i, i + 1)));
            }
            if y + 1 < h {
                edges.push((i as u32, (i + w) as u32, cap(i, i + w)));
            }
        }
        Self::build(w * h, edges)
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

    /// The arcs whose tail is `p`.
    #[inline]
    fn arcs(&self, p: usize) -> std::ops::Range<usize> {
        self.first[p] as usize..self.first[p + 1] as usize
    }

    /// Residual, in the growth direction, of arc `a` (tail `p`, head `q`). For an S-node the
    /// flow would run `p → q`; for a T-node it would run `q → p` (toward the sink).
    #[inline]
    fn tree_cap(&self, p: usize, a: usize) -> i32 {
        if self.tree[p] == S {
            self.res[a]
        } else {
            self.res[a ^ 1]
        }
    }

    /// Run the max-flow. Returns the flow value (= min-cut value, exact).
    pub fn max_flow(&mut self) -> i64 {
        // Seed the trees from the terminals.
        for i in 0..self.n {
            if self.t[i] != 0 {
                self.tree[i] = if self.t[i] > 0 { S } else { T };
                self.parent[i] = P_TERMINAL;
                self.set_active(i);
            }
        }

        let mut flow: i64 = 0;
        while let Some((p, q, a)) = self.grow() {
            flow += self.augment(p, q, a);
            self.adopt_all();
        }
        flow
    }

    /// **Grow** phase: pull active nodes and extend their tree until an arc bridges the two
    /// trees. Returns `(p, q, a)` where `a` is an arc `p → q` and `tree[p] != tree[q]`.
    fn grow(&mut self) -> Option<(usize, usize, usize)> {
        while let Some(pf) = self.active.front().copied() {
            let p = pf as usize;
            // A node may have gone free/orphan since it was queued; drop it lazily.
            if self.tree[p] == FREE {
                self.active.pop_front();
                self.in_active[p] = false;
                continue;
            }
            for k in self.arcs(p) {
                let a = self.inc[k] as usize;
                let q = self.head[a] as usize;
                if self.tree_cap(p, a) <= 0 {
                    continue;
                }
                if self.tree[q] == FREE {
                    // Adopt q into p's tree; q's parent is p, i.e. the arc q → p.
                    self.tree[q] = self.tree[p];
                    self.parent[q] = (a ^ 1) as u32;
                    self.set_active(q);
                } else if self.tree[q] != self.tree[p] {
                    // Bridge found: keep p active (it may bridge again after augment).
                    return Some((p, q, a));
                }
            }
            // p is fully grown for now; it stays in its tree but leaves the queue.
            self.active.pop_front();
            self.in_active[p] = false;
        }
        None
    }

    /// The S-tree node, the T-tree node, and the S→T arc for the bridge `(p, q, a)`.
    #[inline]
    fn orient(&self, p: usize, q: usize, a: usize) -> (usize, usize, usize) {
        if self.tree[p] == S {
            (p, q, a)
        } else {
            (q, p, a ^ 1)
        }
    }

    /// **Augment** along the path source → s_node → t_node → sink. Pushes the bottleneck,
    /// saturates arcs, and severs any child whose parent arc hits zero (an orphan).
    fn augment(&mut self, p: usize, q: usize, a: usize) -> i64 {
        let (s_node, t_node, arc) = self.orient(p, q, a);

        // 1) bottleneck: the connecting arc, plus both tree branches to their terminals.
        let mut bottleneck = self.res[arc];
        // S branch: walk s_node → source root; the tree arc carries flow parent → child,
        // which is the REVERSE of the stored child → parent arc.
        {
            let mut n = s_node;
            loop {
                let pa = self.parent[n];
                if pa == P_TERMINAL {
                    bottleneck = bottleneck.min(self.t[n]); // source residual, > 0
                    break;
                }
                debug_assert!(pa != P_NONE, "tree node has no parent");
                let pa = pa as usize;
                bottleneck = bottleneck.min(self.res[pa ^ 1]);
                n = self.head[pa] as usize;
            }
        }
        // T branch: walk t_node → sink root; here the flow runs child → parent, which IS the
        // stored arc.
        {
            let mut n = t_node;
            loop {
                let pa = self.parent[n];
                if pa == P_TERMINAL {
                    bottleneck = bottleneck.min(-self.t[n]); // sink residual, > 0
                    break;
                }
                debug_assert!(pa != P_NONE, "tree node has no parent");
                let pa = pa as usize;
                bottleneck = bottleneck.min(self.res[pa]);
                n = self.head[pa] as usize;
            }
        }

        let b = bottleneck;
        debug_assert!(b > 0, "augmenting path must carry positive flow");

        // 2) push `b`. Reverse residuals go up by the same amount (undirected arcs).
        self.res[arc] -= b;
        self.res[arc ^ 1] += b;

        // S branch: from s_node up to the source root.
        {
            let mut n = s_node;
            loop {
                let pa = self.parent[n];
                if pa == P_TERMINAL {
                    self.t[n] -= b;
                    if self.t[n] == 0 {
                        self.make_orphan(n);
                    }
                    break;
                }
                let pa = pa as usize;
                let fwd = pa ^ 1; // parent → n
                self.res[fwd] -= b;
                self.res[pa] += b;
                if self.res[fwd] == 0 {
                    self.make_orphan(n); // n lost its parent arc
                }
                n = self.head[pa] as usize;
            }
        }
        // T branch: from t_node down to the sink root.
        {
            let mut n = t_node;
            loop {
                let pa = self.parent[n];
                if pa == P_TERMINAL {
                    self.t[n] += b;
                    if self.t[n] == 0 {
                        self.make_orphan(n);
                    }
                    break;
                }
                let pa = pa as usize;
                // n → parent IS the stored arc.
                self.res[pa] -= b;
                self.res[pa ^ 1] += b;
                if self.res[pa] == 0 {
                    self.make_orphan(n);
                }
                n = self.head[pa] as usize;
            }
        }

        i64::from(b)
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
        for k in self.arcs(n) {
            let a = self.inc[k] as usize;
            let q = self.head[a] as usize;
            if self.tree[q] != mytree {
                continue;
            }
            // Residual of the arc q → n (parent → child in the growth direction).
            let cap = if mytree == S {
                self.res[a ^ 1]
            } else {
                self.res[a]
            };
            if cap <= 0 {
                continue;
            }
            if let Some(dq) = self.try_origin(q, n)
                && best.is_none_or(|(_, bd)| dq < bd)
            {
                best = Some((a, dq));
            }
        }
        if let Some((a, dq)) = best {
            self.parent[n] = a as u32; // n's parent is head[a], reached by the arc n → q
            self.dist[n] = dq + 1;
            self.ts[n] = self.time;
            return;
        }
        // No parent: free `n`. Its same-tree neighbours become candidates to re-grow into
        // it, and its children lose their parent.
        for k in self.arcs(n) {
            let a = self.inc[k] as usize;
            let q = self.head[a] as usize;
            if self.tree[q] != mytree {
                continue;
            }
            // A neighbour that could grow back into n → re-activate it.
            let cap = if mytree == S {
                self.res[a ^ 1]
            } else {
                self.res[a]
            };
            if cap > 0 {
                self.set_active(q);
            }
            // A child of n (its parent arc points back at n) becomes an orphan.
            let pq = self.parent[q];
            if pq != P_TERMINAL && pq != P_NONE && self.head[pq as usize] as usize == n {
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
                pa => {
                    self.scratch.push(n as u32);
                    n = self.head[pa as usize] as usize;
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
    /// the **source side** of the min-cut. This is where the labels are read.
    #[must_use]
    pub fn source_side(&self) -> Vec<bool> {
        let mut side = vec![false; self.n];
        let mut stack = Vec::new();
        // `i` seeds both `side` and the `stack` — a range loop is the clear form.
        #[allow(clippy::needless_range_loop)]
        for i in 0..self.n {
            if self.t[i] > 0 {
                side[i] = true;
                stack.push(i);
            }
        }
        while let Some(i) = stack.pop() {
            for k in self.arcs(i) {
                let a = self.inc[k] as usize;
                let q = self.head[a] as usize;
                if self.res[a] > 0 && !side[q] {
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
/// ⚠️ **This is the PIXEL instance** — `§7.1` measured it at 3,3 s on a 4096² grid, so it is
/// the gates' and the bench's graph, not the one a click should build. The product runs the
/// same solver over the trapped-ball region graph (`§8`).
///
/// ⚠️ v1 reads `V_pq` from the `BOUNDARY` **bit**, not a coverage float: `ink.rs` marks a bit
/// today, and the analytic-coverage `V_pq` of `09 §3.1` is a later refinement.
#[cfg(test)]
#[must_use]
pub fn lazybrush_binary(
    grid: &Grid,
    source: &[usize],
    sink: &[usize],
    v_white: i32,
    v_ink: i32,
) -> Flow {
    let w = grid.w;
    let h = grid.h;

    let k = 2 * (w + h) as i32; // scribble weight — dominates the perimeter (`09 §3`)

    let is_ink = |i: usize| grid.flags[i] & BOUNDARY != 0;
    let mut f = Flow::grid_4conn(w, h, |i, q| {
        if is_ink(i) || is_ink(q) {
            v_ink
        } else {
            v_white
        }
    });

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
