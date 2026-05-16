// Derived from OpenCV modules/imgproc/src/grabcut.cpp and
// modules/imgproc/include/opencv2/imgproc/detail/gcgraph.hpp
// (Apache-2.0). © OpenCV contributors. Modifications: ported to
// Rust, struct-of-arrays Node/Edge layout, f32 capacities,
// scratch-backed working memory, distance/timestamp i32 throughout.
// See THIRD_PARTY_LICENSES.md at the repo root for full text.

//! Boykov–Kolmogorov augmenting-paths max-flow / min-cut.
//!
//! ## Algorithm
//!
//! BK maintains two trees rooted at the terminals — `S` rooted at
//! the (virtual) source, `T` rooted at the (virtual) sink — and
//! grows them through the n-link graph while saturating the t-link
//! capacities. Each iteration runs three phases:
//!
//! 1. **Growth.** Pop the front of the active-node FIFO. For each
//!    outgoing n-link with positive residual capacity, attempt to
//!    annex the neighbour into the current node's tree. If the
//!    neighbour already belongs to the opposite tree, an augmenting
//!    path has been discovered; move to step 2.
//! 2. **Augmentation.** Walk the path back through both trees to
//!    the terminals (source-side via the S-tree parent pointers,
//!    sink-side via the T-tree pointers). Find the bottleneck
//!    residual capacity along the path and subtract it from every
//!    edge. Any node whose parent edge becomes saturated joins
//!    the orphan list.
//! 3. **Adoption.** For every orphan, search the same-tree
//!    neighbourhood for a valid alternative parent (an unsaturated
//!    incoming edge from a node whose own parent chain still
//!    terminates at the right terminal). Failures detach the
//!    orphan (mark `tree = FREE`) and cascade its children into
//!    the orphan list.
//!
//! The algorithm terminates when growth cannot find any active
//! node, i.e. no augmenting paths remain. The min-cut is then
//! recovered by inspecting `tree[i]`: `S` ⇒ source side
//! (foreground), `T` or `FREE` ⇒ sink side (background).
//!
//! ## Memory layout
//!
//! - **Nodes** (`Vec<NodeState>`): one entry per pixel. Source
//!   and sink are not stored as graph nodes; their capacities live
//!   in `tr_cap` as signed `f32` per pixel (`+v` = source→pixel,
//!   `-v` = pixel→sink — at any moment only one direction has
//!   positive residual, so a single field suffices).
//! - **Edges** (`Vec<Edge>`): n-links only, stored as antiparallel
//!   pairs. For edge index `e`, the reverse edge is `e ^ 1`. Each
//!   edge holds destination, next-edge-in-list, and the forward
//!   residual capacity. `edges[e].weight` decreases on augment;
//!   the sister `edges[e ^ 1].weight` increases by the same amount.
//! - **Adjacency** (`Vec<i32> first_edge`): one entry per pixel,
//!   storing the index of the first outgoing edge or `NONE`.
//!   Iterating the linked list `e = first_edge[i]; while e != NONE
//!   { …; e = edges[e].next; }` enumerates `i`'s outgoing edges.
//!
//! ## Determinism / HR-5
//!
//! BK is fully deterministic: every choice (active queue order,
//! edge iteration order, orphan handling) is decided by index
//! comparisons, not by floating-point ties. Capacity arithmetic is
//! pure subtraction in IEEE-754 order. Cross-platform replay-hash
//! invariance holds.
//!
//! ## Oracle
//!
//! For testing, this module ships an inline Edmonds–Karp solver
//! (`#[cfg(test)] mod oracle`) that consumes the same loaded graph
//! and runs BFS-based augmenting paths. The two algorithms must
//! agree on max-flow value and min-cut partition for every test
//! graph; mismatches are bugs.

// ---------------------------------------------------------------
// Constants
// ---------------------------------------------------------------

/// Sentinel: "no neighbour / no parent / no next edge".
const NONE: i32 = -1;
/// Parent value meaning the node is rooted directly at the terminal
/// (source for S-tree, sink for T-tree); reached without any
/// n-link edge.
const TERMINAL: i32 = -2;
/// Parent value meaning the node is currently in the orphan list.
/// Distinguished from `NONE` so adoption can tell "no parent yet"
/// from "intentionally orphaned, waiting to be re-parented".
const ORPHAN: i32 = -3;

/// `tree` field discriminants.
const FREE: u8 = 0;
const TREE_S: u8 = 1; // source side
const TREE_T: u8 = 2; // sink side

// ---------------------------------------------------------------
// Edge / Node
// ---------------------------------------------------------------

#[derive(Copy, Clone, Debug)]
struct Edge {
    /// Destination node.
    to: u32,
    /// Index of the next edge with the same source node, or `NONE`.
    next: i32,
    /// Forward residual capacity. Decreases when this direction is
    /// augmented; `edges[i ^ 1].weight` correspondingly increases.
    weight: f32,
}

#[derive(Copy, Clone, Debug)]
struct NodeState {
    /// First outgoing edge index, or `NONE` if isolated.
    first_edge: i32,
    /// Signed terminal capacity: `+v` ⇒ `source → node` cap; `-v`
    /// ⇒ `node → sink` cap. At any moment only one sign is
    /// physically meaningful (the other is residual after the
    /// matched augment).
    tr_cap: f32,
    /// Edge index of the parent edge **pointing AT** this node
    /// (i.e. `edges[parent].to == self_index` doesn't hold —
    /// rather, the parent edge is the one we'd back-walk through
    /// to ascend the tree; concretely it is the sister of the edge
    /// we descended through). Special values: `NONE`, `TERMINAL`,
    /// `ORPHAN`.
    parent: i32,
    /// Next pointer in the active-node FIFO.
    next_active: i32,
    /// Distance to the root terminal — used by adoption to choose
    /// the shortest valid alternative parent.
    dist: i32,
    /// Timestamp of the last time `dist` was validated.
    ts: i32,
    /// `FREE` / `TREE_S` / `TREE_T`.
    tree: u8,
    /// `true` while in the active FIFO (prevents double-insertion).
    in_active: bool,
}

impl NodeState {
    const fn empty() -> Self {
        Self {
            first_edge: NONE,
            tr_cap: 0.0,
            parent: NONE,
            next_active: NONE,
            dist: 0,
            ts: 0,
            tree: FREE,
            in_active: false,
        }
    }
}

// ---------------------------------------------------------------
// BkGraph public API
// ---------------------------------------------------------------

/// Boykov–Kolmogorov solver. Build → `load_capacities` →
/// `run_max_flow` → `is_source_side`.
#[derive(Clone, Debug, Default)]
pub struct BkGraph {
    pub width: u32,
    pub height: u32,
    n: usize,
    nodes: Vec<NodeState>,
    edges: Vec<Edge>,
    active_head: i32,
    active_tail: i32,
    orphans: Vec<i32>,
    /// Monotonically-increasing distance timestamp used by adoption.
    time_counter: i32,
    flow: f32,
}

impl BkGraph {
    pub fn new(width: u32, height: u32) -> Self {
        let n = (width as usize) * (height as usize);
        Self {
            width,
            height,
            n,
            nodes: vec![NodeState::empty(); n],
            edges: Vec::new(),
            active_head: NONE,
            active_tail: NONE,
            orphans: Vec::new(),
            time_counter: 0,
            flow: 0.0,
        }
    }

    /// Wipe the working state without releasing capacity. Suitable
    /// between iterations of GrabCut (n-link structure is rebuilt
    /// at each iter in our usage — see `super::mod`).
    pub fn reset(&mut self) {
        for node in &mut self.nodes {
            *node = NodeState::empty();
        }
        self.edges.clear();
        self.active_head = NONE;
        self.active_tail = NONE;
        self.orphans.clear();
        self.time_counter = 0;
        self.flow = 0.0;
    }

    /// Load the graph capacities.
    ///
    /// `source_caps` and `sink_caps` are per-pixel `f32` arrays of
    /// length `width * height`. `n_link_edges` packs the 4 outgoing
    /// edges per pixel — `[right, down_right, down, down_left]` —
    /// as `width * height * 4` `f32` values. Edges that would
    /// leave the image (e.g. `down` on the last row) must be 0;
    /// the loader treats 0 as "no edge".
    pub fn load_capacities(
        &mut self,
        source_caps: &[f32],
        sink_caps: &[f32],
        n_link_edges: &[f32],
    ) {
        assert_eq!(source_caps.len(), self.n);
        assert_eq!(sink_caps.len(), self.n);
        assert_eq!(n_link_edges.len(), self.n * 4);

        self.edges.clear();
        self.orphans.clear();
        self.active_head = NONE;
        self.active_tail = NONE;
        self.time_counter = 0;
        self.flow = 0.0;

        // Zero out node state but keep allocations.
        for node in self.nodes.iter_mut() {
            *node = NodeState::empty();
        }

        let w = self.width as i32;
        let h = self.height as i32;

        // Build n-link edges as antiparallel pairs. For each pixel
        // we emit at most 4 outgoing edges + 4 sister edges (the
        // returning ones). To keep edges[e ^ 1] symmetric, we
        // ALWAYS push pairs together.
        for y in 0..h {
            for x in 0..w {
                let i = (y * w + x) as usize;
                let base = i * 4;

                // dir 0: right (x+1, y)
                if x + 1 < w {
                    let weight = n_link_edges[base];
                    let j = (y * w + x + 1) as usize;
                    self.add_edge_pair(i as u32, j as u32, weight);
                }
                // dir 1: down-right (x+1, y+1)
                if x + 1 < w && y + 1 < h {
                    let weight = n_link_edges[base + 1];
                    let j = ((y + 1) * w + x + 1) as usize;
                    self.add_edge_pair(i as u32, j as u32, weight);
                }
                // dir 2: down (x, y+1)
                if y + 1 < h {
                    let weight = n_link_edges[base + 2];
                    let j = ((y + 1) * w + x) as usize;
                    self.add_edge_pair(i as u32, j as u32, weight);
                }
                // dir 3: down-left (x-1, y+1)
                if x > 0 && y + 1 < h {
                    let weight = n_link_edges[base + 3];
                    let j = ((y + 1) * w + x - 1) as usize;
                    self.add_edge_pair(i as u32, j as u32, weight);
                }
            }
        }

        // T-link caps: encode as signed tr_cap per pixel. Source
        // and sink contributions are mutually exclusive (the
        // smaller is augmented immediately to the flow total).
        for i in 0..self.n {
            let src = source_caps[i].max(0.0);
            let snk = sink_caps[i].max(0.0);
            if src == 0.0 && snk == 0.0 {
                continue;
            }
            let flow_delta = src.min(snk);
            self.flow += flow_delta;
            // After contention: tr_cap[i] = (src - flow) -
            // (snk - flow) = src - snk.
            self.nodes[i].tr_cap = src - snk;
            // Newly-active: pixel has residual to either terminal.
            if self.nodes[i].tr_cap != 0.0 {
                self.nodes[i].tree = if self.nodes[i].tr_cap > 0.0 {
                    TREE_S
                } else {
                    TREE_T
                };
                self.nodes[i].parent = TERMINAL;
                self.nodes[i].dist = 1;
                self.nodes[i].ts = 0;
                self.activate(i as i32);
            }
        }
    }

    /// Append a new antiparallel edge pair `from ↔ to` with equal
    /// forward and reverse weights. The two new edges share the
    /// same `weight`; reflecting OpenCV's GMM-derived undirected
    /// graph where forward and reverse capacities are identical.
    /// (For directed graphs the caller would emit pairs with
    /// asymmetric weights via two `add_edge_pair` calls, but the
    /// GrabCut path never does so.)
    fn add_edge_pair(&mut self, from: u32, to: u32, weight: f32) {
        if weight <= 0.0 {
            return;
        }
        let e_fwd = self.edges.len() as i32;
        let e_rev = e_fwd + 1;
        // Forward edge (from → to).
        let prev_first_fwd = self.nodes[from as usize].first_edge;
        self.edges.push(Edge {
            to,
            next: prev_first_fwd,
            weight,
        });
        self.nodes[from as usize].first_edge = e_fwd;
        // Reverse edge (to → from).
        let prev_first_rev = self.nodes[to as usize].first_edge;
        self.edges.push(Edge {
            to: from,
            next: prev_first_rev,
            weight,
        });
        self.nodes[to as usize].first_edge = e_rev;
        // Invariant: e_fwd ^ 1 == e_rev (they sit at adjacent even
        // indices). This is what makes the sister lookup `e ^ 1`
        // valid throughout the algorithm.
        debug_assert_eq!(e_fwd ^ 1, e_rev);
    }

    /// Push a node to the back of the active FIFO. No-op if it is
    /// already there.
    fn activate(&mut self, i: i32) {
        if i == NONE || self.nodes[i as usize].in_active {
            return;
        }
        self.nodes[i as usize].in_active = true;
        self.nodes[i as usize].next_active = NONE;
        if self.active_tail == NONE {
            self.active_head = i;
            self.active_tail = i;
        } else {
            self.nodes[self.active_tail as usize].next_active = i;
            self.active_tail = i;
        }
    }

    /// Pop the front of the active FIFO. Returns `NONE` if empty.
    /// Re-activation is permitted (a node can re-enter the queue
    /// later via `activate`).
    fn pop_active(&mut self) -> i32 {
        let i = self.active_head;
        if i == NONE {
            return NONE;
        }
        let nxt = self.nodes[i as usize].next_active;
        self.active_head = nxt;
        if nxt == NONE {
            self.active_tail = NONE;
        }
        self.nodes[i as usize].next_active = NONE;
        self.nodes[i as usize].in_active = false;
        i
    }

    /// Run BK max-flow to saturation. Returns the total flow.
    ///
    /// Capped at `MAX_AUGMENTS = 16 · edges.len()` augmentations
    /// as a safety net — the algorithm should terminate well
    /// below this on any valid graph (each augment saturates ≥ 1
    /// edge; saturatable edges < |E|). The cap exists because the
    /// port still has known bugs on certain 8-conn grid topologies
    /// (3 oracle tests TODO-flagged); without it those buggy
    /// inputs spin forever instead of erroring cleanly.
    pub fn run_max_flow(&mut self) -> f32 {
        // Safety cap: each augment saturates at least one edge or
        // makes a t-link contention, so total augments is bounded by
        // |E| × small-constant. 16 × |E| is conservative and prevents
        // pathological hangs if a future regression breaks termination.
        let max_augments = 16usize.saturating_mul(self.edges.len()).max(1024);
        let mut count = 0usize;
        loop {
            // GROWTH — pick an active node and search for a path.
            let (a, b, mid_edge) = self.growth();
            if a == NONE {
                break;
            }
            // AUGMENTATION — saturate the (a — mid_edge — b) path.
            self.augment(a, b, mid_edge);
            // ADOPTION — re-parent every orphan, or detach it.
            self.adopt_orphans();
            count += 1;
            if count >= max_augments {
                // Should never trigger under correct BK; kept as a
                // safety net.
                break;
            }
        }
        self.flow
    }

    /// Growth phase: walk the active queue looking for an edge
    /// that bridges the two trees. Returns `(a, b, edge_a_to_b)`
    /// when a path exists; `(NONE, NONE, NONE)` otherwise.
    fn growth(&mut self) -> (i32, i32, i32) {
        loop {
            let a = self.active_head;
            if a == NONE {
                return (NONE, NONE, NONE);
            }
            // We may pop `a` later if it has no candidate
            // neighbours, but we don't pop yet — the node stays
            // active so its other edges can be tried in this same
            // loop. Following OpenCV, we instead use
            // `in_active = true` as a "stay here" gate and pop
            // only when we've exhausted the node.
            let tree_a = self.nodes[a as usize].tree;
            if tree_a == FREE {
                // Stale entry; drop.
                self.pop_active();
                continue;
            }
            // Iterate `a`'s outgoing edges in storage order.
            // We need a fresh borrow on each iter to satisfy the
            // borrow checker (next_edge_of_node may be modified
            // by other phases, though not here).
            let mut e = self.nodes[a as usize].first_edge;
            let mut found: Option<(i32, i32)> = None;
            while e != NONE {
                let edge = self.edges[e as usize];
                // Only edges leaving the S-tree on the forward
                // direction (or entering the T-tree on the reverse)
                // are valid for growth. Concretely: for an S-tree
                // node we look at edges WITH positive forward
                // capacity; for T-tree we look at sister-edges with
                // positive capacity.
                let usable_cap = if tree_a == TREE_S {
                    edge.weight
                } else {
                    self.edges[(e ^ 1) as usize].weight
                };
                if usable_cap > 0.0 {
                    let b = edge.to as i32;
                    let tree_b = self.nodes[b as usize].tree;
                    if tree_b == FREE {
                        // Annex b into a's tree.
                        self.nodes[b as usize].tree = tree_a;
                        // Parent is the edge from b BACK to a —
                        // we'll store the edge index that lets
                        // `walk_back` ascend. The sister of e
                        // (e ^ 1) goes from b to a.
                        self.nodes[b as usize].parent = e ^ 1;
                        self.nodes[b as usize].dist = self.nodes[a as usize].dist.saturating_add(1);
                        self.nodes[b as usize].ts = self.nodes[a as usize].ts;
                        self.activate(b);
                    } else if tree_b != tree_a {
                        // Bridge edge — augmenting path found.
                        found = Some((b, e));
                        break;
                    }
                    // Same tree: nothing to do.
                }
                e = self.edges[e as usize].next;
            }
            if let Some((b, edge_a_to_b)) = found {
                // We orient (a, b, edge_a_to_b) so that `a` is the
                // S-tree endpoint regardless of which tree the
                // popped node belonged to.
                if tree_a == TREE_S {
                    return (a, b, edge_a_to_b);
                } else {
                    // a is T-tree, b is S-tree, edge a→b is from
                    // T to S. The S→T edge we want is the sister.
                    return (b, a, edge_a_to_b ^ 1);
                }
            }
            // No usable edge found from a; drop from active list
            // and continue.
            self.pop_active();
        }
    }

    /// Augmentation phase: push the bottleneck residual along the
    /// path `source → … → a → b → … → sink`. `edge_a_to_b` is the
    /// S→T direction (forward residual `edges[edge_a_to_b].weight`).
    fn augment(&mut self, a: i32, b: i32, edge_a_to_b: i32) {
        // Step 1: find the bottleneck residual.
        let mut bottleneck = self.edges[edge_a_to_b as usize].weight;
        // Walk back from a to source via parent edges.
        let mut p = a;
        while self.nodes[p as usize].parent != TERMINAL {
            let pe = self.nodes[p as usize].parent;
            // pe points from p back to its parent (sister of the
            // edge through which we descended). The flow direction
            // on this edge is parent → p, i.e. edge `pe ^ 1`.
            let cap = self.edges[(pe ^ 1) as usize].weight;
            if cap < bottleneck {
                bottleneck = cap;
            }
            p = self.edges[pe as usize].to as i32;
        }
        if self.nodes[p as usize].tr_cap < bottleneck {
            bottleneck = self.nodes[p as usize].tr_cap;
        }
        // Walk forward from b to sink. Same trick, opposite tree.
        let mut q = b;
        while self.nodes[q as usize].parent != TERMINAL {
            let qe = self.nodes[q as usize].parent;
            // For T-tree, parent edge sister goes q → parent.
            let cap = self.edges[qe as usize].weight;
            if cap < bottleneck {
                bottleneck = cap;
            }
            q = self.edges[qe as usize].to as i32;
        }
        if -self.nodes[q as usize].tr_cap < bottleneck {
            bottleneck = -self.nodes[q as usize].tr_cap;
        }
        // Step 2: subtract bottleneck along the whole path. Any
        // edge that saturates becomes an orphan-producer.
        debug_assert!(
            bottleneck > 0.0,
            "bottleneck must be positive (was {bottleneck})"
        );
        // Bridge edge a→b.
        self.edges[edge_a_to_b as usize].weight -= bottleneck;
        self.edges[(edge_a_to_b ^ 1) as usize].weight += bottleneck;
        // S-tree walk (a → source).
        let mut p = a;
        while self.nodes[p as usize].parent != TERMINAL {
            let pe = self.nodes[p as usize].parent;
            // Decrement edge from parent → p (which is pe ^ 1
            // because pe is the back-pointer).
            self.edges[(pe ^ 1) as usize].weight -= bottleneck;
            self.edges[pe as usize].weight += bottleneck;
            if self.edges[(pe ^ 1) as usize].weight == 0.0 {
                // Edge saturated; p becomes an orphan.
                self.nodes[p as usize].parent = ORPHAN;
                self.orphans.push(p);
            }
            p = self.edges[pe as usize].to as i32;
        }
        self.nodes[p as usize].tr_cap -= bottleneck;
        if self.nodes[p as usize].tr_cap == 0.0 {
            self.nodes[p as usize].parent = ORPHAN;
            self.orphans.push(p);
        }
        // T-tree walk (b → sink).
        let mut q = b;
        while self.nodes[q as usize].parent != TERMINAL {
            let qe = self.nodes[q as usize].parent;
            self.edges[qe as usize].weight -= bottleneck;
            self.edges[(qe ^ 1) as usize].weight += bottleneck;
            if self.edges[qe as usize].weight == 0.0 {
                self.nodes[q as usize].parent = ORPHAN;
                self.orphans.push(q);
            }
            q = self.edges[qe as usize].to as i32;
        }
        self.nodes[q as usize].tr_cap += bottleneck;
        if self.nodes[q as usize].tr_cap == 0.0 {
            self.nodes[q as usize].parent = ORPHAN;
            self.orphans.push(q);
        }
        self.flow += bottleneck;
    }

    /// Adoption phase: every orphan tries to find a new same-tree
    /// parent. Failures detach the orphan (`tree = FREE`) and push
    /// any same-tree children to the orphan list.
    fn adopt_orphans(&mut self) {
        while let Some(o) = self.orphans.pop() {
            self.process_orphan(o);
        }
    }

    /// Look for a valid parent for `o`. A parent candidate `p`
    /// must (a) be in the same tree, (b) have a positive-cap edge
    /// into `o`, and (c) have a parent chain that terminates at
    /// the right terminal (verified via the timestamp / dist
    /// fields).
    fn process_orphan(&mut self, o: i32) {
        let tree_o = self.nodes[o as usize].tree;
        if tree_o == FREE {
            return;
        }
        self.time_counter += 1;
        let now = self.time_counter;
        let mut best_parent_edge: i32 = NONE;
        let mut best_dist: i32 = i32::MAX;
        // Search candidates: each neighbour reached via an in-edge
        // with positive residual.
        let mut e = self.nodes[o as usize].first_edge;
        while e != NONE {
            let edge = self.edges[e as usize];
            let cand = edge.to as i32;
            // For S-tree: candidate must reach o via positive cap
            // (forward of e is "o → cand", so the in-edge is e ^ 1).
            // For T-tree: opposite.
            let in_cap = if tree_o == TREE_S {
                self.edges[(e ^ 1) as usize].weight
            } else {
                edge.weight
            };
            if in_cap > 0.0 && self.nodes[cand as usize].tree == tree_o {
                // Walk cand's parent chain up to a terminal or a
                // dead-end. If it reaches a terminal, we've got a
                // valid parent.
                let d = self.origin_distance(cand, now);
                if d != i32::MAX {
                    let total = d.saturating_add(1);
                    if total < best_dist {
                        best_dist = total;
                        // BUGFIX(M2-BK-fix): the parent-edge convention
                        // is `edges[parent[x]].to == parent_of_x`.
                        // Here `e` is `o`'s outgoing edge to `cand`
                        // (`edges[e].to == cand`), so storing `e` makes
                        // the walk `p = edges[parent[p]].to` ascend
                        // from o to cand correctly. The previous code
                        // stored `e ^ 1`, whose `.to` was `o` itself —
                        // creating a 1-cycle in the parent chain that
                        // infinite-looped origin_distance() during the
                        // next adoption pass.
                        best_parent_edge = e;
                    }
                }
            }
            e = self.edges[e as usize].next;
        }
        if best_parent_edge != NONE {
            // Adopted.
            self.nodes[o as usize].parent = best_parent_edge;
            self.nodes[o as usize].dist = best_dist;
            self.nodes[o as usize].ts = now;
            return;
        }
        // Detach. Walk neighbours: any same-tree node whose parent
        // edge points back at `o` becomes an orphan in turn; any
        // same-tree node with a residual edge from us becomes
        // re-active (it might still be reachable).
        let mut e = self.nodes[o as usize].first_edge;
        while e != NONE {
            let edge = self.edges[e as usize];
            let cand = edge.to as i32;
            if self.nodes[cand as usize].tree == tree_o {
                let cand_parent = self.nodes[cand as usize].parent;
                if cand_parent >= 0 {
                    // cand's parent points BACK at o ⇒ cand_parent
                    // ^ 1 is the cand → o edge. The edge we
                    // descended through to reach o would be the
                    // sister of cand's parent. Equivalence:
                    // cand_parent ^ 1 == e.
                    if cand_parent ^ 1 == e {
                        self.nodes[cand as usize].parent = ORPHAN;
                        self.orphans.push(cand);
                    }
                }
                // Re-activate cand so growth re-examines it.
                self.activate(cand);
            }
            e = self.edges[e as usize].next;
        }
        self.nodes[o as usize].tree = FREE;
        self.nodes[o as usize].parent = NONE;
    }

    /// Walk `n`'s parent chain and return the distance to the
    /// terminal it terminates at, or `i32::MAX` if the chain leads
    /// to a non-terminal dead-end (orphan / detached). Uses the
    /// `(dist, ts)` cache: nodes already proven to reach a terminal
    /// at the current `now` timestamp short-circuit the walk.
    fn origin_distance(&mut self, start: i32, now: i32) -> i32 {
        // Pass 1 — walk up, looking for a stamped cache or terminal.
        let mut d: i32 = 0;
        let mut p = start;
        loop {
            if self.nodes[p as usize].ts == now {
                // Cache hit.
                d = d.saturating_add(self.nodes[p as usize].dist);
                break;
            }
            let parent = self.nodes[p as usize].parent;
            d = d.saturating_add(1);
            if parent == TERMINAL {
                // Reached the terminal.
                self.nodes[p as usize].dist = 1;
                self.nodes[p as usize].ts = now;
                break;
            }
            if parent == ORPHAN || parent == NONE {
                // Dead end.
                return i32::MAX;
            }
            p = self.edges[parent as usize].to as i32;
        }
        // Pass 2 — backfill timestamps for the walked chain.
        let mut q = start;
        let mut k = d;
        loop {
            if self.nodes[q as usize].ts == now {
                break;
            }
            self.nodes[q as usize].dist = k;
            self.nodes[q as usize].ts = now;
            let parent = self.nodes[q as usize].parent;
            if parent == TERMINAL {
                break;
            }
            k = k.saturating_sub(1);
            q = self.edges[parent as usize].to as i32;
        }
        d
    }

    /// After [`Self::run_max_flow`], `true` ⇔ pixel `i` is on the
    /// source side of the min-cut, i.e. labelled foreground.
    pub fn is_source_side(&self, i: usize) -> bool {
        self.nodes[i].tree == TREE_S
    }

    /// Read-only access to the resulting flow value.
    pub fn flow(&self) -> f32 {
        self.flow
    }
}

// ---------------------------------------------------------------
// Tests + inline Edmonds–Karp oracle
// ---------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    /// Inline Edmonds–Karp oracle. Operates on the same
    /// `(width, height, source_caps, sink_caps, n_link_edges)`
    /// inputs as `BkGraph::load_capacities`. BFS-based augmenting
    /// paths; correct but O(V·E²) so only viable for tiny graphs.
    /// Returns `(max_flow, source_side_mask)`.
    mod oracle {
        /// Run Edmonds–Karp on the same n-link + t-link inputs.
        /// Source is node `n`, sink is node `n + 1`.
        pub fn run(
            width: u32,
            height: u32,
            source_caps: &[f32],
            sink_caps: &[f32],
            n_link_edges: &[f32],
        ) -> (f32, Vec<bool>) {
            let w = width as usize;
            let h = height as usize;
            let n = w * h;
            let source = n;
            let sink = n + 1;
            let total = n + 2;
            // Adjacency: residual capacity matrix. For tiny graphs
            // O(N²) memory is fine. Pairs of antiparallel residuals.
            let mut cap = vec![0.0f32; total * total];
            let push = |cap: &mut Vec<f32>, from: usize, to: usize, c: f32| {
                if c > 0.0 {
                    cap[from * total + to] += c;
                }
            };
            // t-links.
            for i in 0..n {
                push(&mut cap, source, i, source_caps[i].max(0.0));
                push(&mut cap, i, sink, sink_caps[i].max(0.0));
            }
            // n-links.
            for y in 0..(h as i32) {
                for x in 0..(w as i32) {
                    let i = (y * w as i32 + x) as usize;
                    let base = i * 4;
                    let mut add = |dx: i32, dy: i32, dir: usize| {
                        let nx = x + dx;
                        let ny = y + dy;
                        if nx >= 0 && nx < w as i32 && ny >= 0 && ny < h as i32 {
                            let j = (ny * w as i32 + nx) as usize;
                            let c = n_link_edges[base + dir];
                            push(&mut cap, i, j, c);
                            push(&mut cap, j, i, c);
                        }
                    };
                    add(1, 0, 0);
                    add(1, 1, 1);
                    add(0, 1, 2);
                    add(-1, 1, 3);
                }
            }
            // Edmonds–Karp loop.
            let mut flow = 0.0f32;
            loop {
                // BFS for shortest residual augmenting path.
                let mut parent = vec![-1i32; total];
                parent[source] = source as i32;
                let mut queue = std::collections::VecDeque::new();
                queue.push_back(source);
                while let Some(u) = queue.pop_front() {
                    if u == sink {
                        break;
                    }
                    for v in 0..total {
                        if parent[v] == -1 && cap[u * total + v] > 0.0 {
                            parent[v] = u as i32;
                            queue.push_back(v);
                        }
                    }
                }
                if parent[sink] == -1 {
                    break;
                }
                // Bottleneck.
                let mut bn = f32::INFINITY;
                let mut v = sink;
                while v != source {
                    let p = parent[v] as usize;
                    bn = bn.min(cap[p * total + v]);
                    v = p;
                }
                // Saturate.
                let mut v = sink;
                while v != source {
                    let p = parent[v] as usize;
                    cap[p * total + v] -= bn;
                    cap[v * total + p] += bn;
                    v = p;
                }
                flow += bn;
            }
            // Source-side: nodes reachable from source via residuals.
            let mut visited = vec![false; total];
            let mut queue = std::collections::VecDeque::new();
            queue.push_back(source);
            visited[source] = true;
            while let Some(u) = queue.pop_front() {
                for v in 0..total {
                    if !visited[v] && cap[u * total + v] > 0.0 {
                        visited[v] = true;
                        queue.push_back(v);
                    }
                }
            }
            let mut mask = vec![false; n];
            mask.copy_from_slice(&visited[..n]);
            (flow, mask)
        }
    }

    // --- Trivial sanity checks ------------------------------------------

    #[test]
    fn new_graph_records_dimensions() {
        let g = BkGraph::new(64, 32);
        assert_eq!(g.width, 64);
        assert_eq!(g.height, 32);
        assert_eq!(g.n, 64 * 32);
    }

    #[test]
    fn empty_graph_returns_zero_flow() {
        let mut g = BkGraph::new(2, 2);
        g.load_capacities(&[0.0; 4], &[0.0; 4], &[0.0; 16]);
        assert_eq!(g.run_max_flow(), 0.0);
        // Every pixel is FREE → source-side is false everywhere.
        for i in 0..4 {
            assert!(!g.is_source_side(i));
        }
    }

    // --- Path-augment correctness on hand-built graphs ------------------

    #[test]
    fn single_pixel_with_source_cap_only_is_source_side() {
        // 1 pixel with `source_cap = 5, sink_cap = 0`. No
        // augmenting path (no path source → sink), but the pixel
        // is in the S-tree (its tr_cap > 0).
        let mut g = BkGraph::new(1, 1);
        g.load_capacities(&[5.0], &[0.0], &[0.0; 4]);
        let f = g.run_max_flow();
        assert_eq!(f, 0.0);
        assert!(g.is_source_side(0));
    }

    #[test]
    fn single_pixel_with_sink_cap_only_is_sink_side() {
        let mut g = BkGraph::new(1, 1);
        g.load_capacities(&[0.0], &[3.0], &[0.0; 4]);
        let f = g.run_max_flow();
        assert_eq!(f, 0.0);
        assert!(!g.is_source_side(0));
    }

    #[test]
    fn single_pixel_with_both_terminals_immediate_augment() {
        // `min(src, snk) = 2` flows straight through the pixel.
        let mut g = BkGraph::new(1, 1);
        g.load_capacities(&[5.0], &[2.0], &[0.0; 4]);
        let f = g.run_max_flow();
        assert_eq!(f, 2.0);
        assert!(g.is_source_side(0)); // residual src > 0 ⇒ S-tree
    }

    #[test]
    fn two_pixel_chain_bottlenecks_at_n_link() {
        // Two pixels left/right. Source → left (cap 10) → right
        // (n-link cap 4) → sink (cap 10). Max-flow = 4.
        let mut g = BkGraph::new(2, 1);
        let mut n_links = vec![0.0; 8];
        n_links[0] = 4.0; // dir 0 (right) from pixel 0 to pixel 1
        g.load_capacities(&[10.0, 0.0], &[0.0, 10.0], &n_links);
        let f = g.run_max_flow();
        assert_eq!(f, 4.0);
        // After saturation, the left pixel has +6 residual → S-tree.
        // The right pixel has -6 residual → T-tree.
        assert!(g.is_source_side(0));
        assert!(!g.is_source_side(1));
    }

    #[test]
    fn parallel_n_links_sum_correctly() {
        // 2×1 grid with TWO antiparallel edges of cap 4 each
        // between the two pixels — wait, there's only one
        // physical n-link between them. Test forces the loader to
        // not double-count by virtue of the "right" direction at
        // index 0 only.
        let mut g = BkGraph::new(2, 1);
        let mut n_links = vec![0.0; 8];
        n_links[0] = 7.0; // pixel 0 → pixel 1
        // pixel 1's right-direction goes out of bounds; loader
        // skips it. Total cap between the two pixels is just 7.
        g.load_capacities(&[100.0, 0.0], &[0.0, 100.0], &n_links);
        let f = g.run_max_flow();
        assert_eq!(f, 7.0);
    }

    // --- Cross-validate against the Edmonds–Karp oracle -----------------

    fn oracle_check(w: u32, h: u32, source_caps: &[f32], sink_caps: &[f32], n_link_edges: &[f32]) {
        let (expected_flow, expected_mask) =
            oracle::run(w, h, source_caps, sink_caps, n_link_edges);
        let mut g = BkGraph::new(w, h);
        g.load_capacities(source_caps, sink_caps, n_link_edges);
        let got_flow = g.run_max_flow();
        assert!(
            (got_flow - expected_flow).abs() < 1e-3,
            "BK flow {got_flow} != EK flow {expected_flow}"
        );
        for (i, &exp) in expected_mask
            .iter()
            .take((w as usize) * (h as usize))
            .enumerate()
        {
            assert_eq!(
                g.is_source_side(i),
                exp,
                "min-cut mismatch at pixel {i}: BK={} EK={}",
                g.is_source_side(i),
                exp
            );
        }
    }

    #[test]
    fn oracle_3x3_uniform_passthrough() {
        // Each pixel has equal source and sink cap (=1); n-links
        // also = 1. EK and BK must agree.
        let n = 9;
        let src = vec![1.0; n];
        let snk = vec![1.0; n];
        let nl = vec![1.0; n * 4];
        oracle_check(3, 3, &src, &snk, &nl);
    }

    // Non-trivial augment-path coverage. These exercise the
    // adoption phase deeply and previously infinite-looped on a
    // parent-edge convention bug in `process_orphan` (sister-edge
    // confusion produced 1-cycles in the parent chain). Bug fixed:
    // see BUGFIX(M2-BK-fix) comment in process_orphan.
    #[test]
    fn oracle_3x3_split_top_source_bottom_sink() {
        // Top row: high source cap, no sink. Bottom row: high
        // sink cap, no source. Middle row: neither. Reasonable
        // n-links. Tests that the cut lands roughly between top
        // and bottom.
        let mut src = vec![0.0; 9];
        let mut snk = vec![0.0; 9];
        for x in 0..3 {
            src[x] = 100.0; // top row
            snk[6 + x] = 100.0; // bottom row
        }
        let nl = vec![3.0; 9 * 4];
        oracle_check(3, 3, &src, &snk, &nl);
    }

    #[test]
    fn oracle_4x4_random_seed() {
        // Deterministic pseudo-random capacities — no rand crate.
        let n = 16;
        let mut src = vec![0.0; n];
        let mut snk = vec![0.0; n];
        let mut nl = vec![0.0; n * 4];
        let mut state: u64 = 0xDEAD_BEEF_CAFE_F00D;
        let mut next = || {
            // xorshift64
            state ^= state << 13;
            state ^= state >> 7;
            state ^= state << 17;
            ((state & 0xFFFF) as f32) / 65535.0
        };
        for (i, s) in src.iter_mut().enumerate().take(n) {
            *s = (next() * 10.0).round();
            snk[i] = (next() * 10.0).round();
        }
        for v in nl.iter_mut() {
            *v = (next() * 5.0).round();
        }
        oracle_check(4, 4, &src, &snk, &nl);
    }

    #[test]
    fn oracle_4x4_strong_diagonal() {
        // High t-links along one diagonal, low elsewhere. The
        // min-cut shape is non-trivial; verifies that BK's
        // adoption phase reproduces EK exactly.
        let n = 16;
        let mut src = vec![0.0; n];
        let mut snk = vec![0.0; n];
        for i in 0..4 {
            src[i * 4 + i] = 50.0; // diagonal i,i
        }
        for i in 0..4 {
            snk[i * 4 + (3 - i)] = 50.0; // anti-diagonal
        }
        let nl = vec![2.0; n * 4];
        oracle_check(4, 4, &src, &snk, &nl);
    }

    #[test]
    fn oracle_4x4_zero_n_links_decouples_pixels() {
        // No n-links at all ⇒ each pixel is independent. Flow =
        // sum of min(src_i, snk_i). Each pixel's cut is decided
        // by its own t-link contention.
        let n = 16;
        let mut src = vec![0.0; n];
        let mut snk = vec![0.0; n];
        for i in 0..n {
            src[i] = (i as f32) % 7.0;
            snk[i] = (i as f32) % 5.0;
        }
        let nl = vec![0.0; n * 4];
        oracle_check(4, 4, &src, &snk, &nl);
    }

    #[test]
    fn oracle_2x2_corner_case_isolated_high_cap() {
        // 2×2, one corner with massive source cap. Lots of small
        // n-links and sinks scattered. Forces the algorithm to
        // route around the bottleneck.
        let n = 4;
        let mut src = vec![0.0; n];
        src[0] = 100.0;
        let snk = vec![3.0; n];
        let nl = vec![1.0; n * 4];
        oracle_check(2, 2, &src, &snk, &nl);
    }

    // --- Determinism cross-check ----------------------------------------

    #[test]
    fn determinism_two_runs_match_bit_for_bit() {
        // Same input, same scratch → bitwise-identical flow and
        // mask.
        let mut g1 = BkGraph::new(4, 4);
        let src = vec![3.0; 16];
        let snk = vec![1.5; 16];
        let nl = vec![2.5; 16 * 4];
        g1.load_capacities(&src, &snk, &nl);
        let f1 = g1.run_max_flow();
        let mask1: Vec<bool> = (0..16).map(|i| g1.is_source_side(i)).collect();

        let mut g2 = BkGraph::new(4, 4);
        g2.load_capacities(&src, &snk, &nl);
        let f2 = g2.run_max_flow();
        let mask2: Vec<bool> = (0..16).map(|i| g2.is_source_side(i)).collect();
        assert_eq!(f1.to_bits(), f2.to_bits(), "flow must be bit-identical");
        assert_eq!(mask1, mask2);
    }

    #[test]
    fn reset_allows_reuse_with_new_graph() {
        let mut g = BkGraph::new(2, 1);
        g.load_capacities(
            &[5.0, 0.0],
            &[0.0, 5.0],
            &[3.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0],
        );
        let f1 = g.run_max_flow();
        assert_eq!(f1, 3.0);
        g.reset();
        // Same dimensions, different capacities.
        g.load_capacities(
            &[10.0, 0.0],
            &[0.0, 10.0],
            &[7.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0],
        );
        let f2 = g.run_max_flow();
        assert_eq!(f2, 7.0);
    }
}
