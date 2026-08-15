//! The cook engine — demand-driven incremental evaluation (pull side).
//!
//! Gold-standard pull model, not a naive dirty-bit:
//! - **Memoized**: a node recomputes only when a forward input's revision
//!   actually changed (or, for a `Temporal` node, when the playhead moved).
//! - **Synchronous `pre`**: a `pre` (delayed) edge reads the *previous tick's*
//!   snapshot, never recurses, and marks its consumer **sequential** so that
//!   consumer recomputes once per tick (which drives the feedback loop), while
//!   purely combinational subgraphs stay memoized across ticks.
//! - **Diamonds**: shared upstream is cooked once per tick (memoization).
//! - **No cycle detection** anywhere: forward edges are acyclic by
//!   construction (`graph`), feedback is always a `pre`.
//!
//! Ticks advance explicitly via [`Cook::advance_tick`] (call once per frame,
//! between cooks). Within a tick, `cook` is idempotent.
//!
//! **Assumes a validated graph.** Like the membrane and the port types, a
//! node's param overrides are checked by [`crate::graph::Graph::validate`], not
//! here: an override naming a param the node does not declare is reported as
//! [`crate::graph::Violation::UnknownParam`] by `validate` and is otherwise
//! ignored at cook time (the node only reads its *own* declared names via
//! [`EvalCtx::param`]). Call `validate` after editing/loading, before cooking.
//!
//! Scope: drives the **pull / presentation** side (motion, shader, sound).
//! `Stateful` (gameplay) nodes are driven by the push evaluator
//! (`ph2d-script`), never here — that separation is the membrane (ADR-0030).

#[path = "cook_fingerprint.rs"]
mod fingerprint;
use fingerprint::{Fingerprint, params_fingerprint, text_params_fingerprint};

#[path = "cook_bypass.rs"]
mod cook_bypass;

use crate::attr::Stream;
use crate::effect::Effect;
use crate::graph::{Graph, NodeId};
use crate::node::{NodeManifest, NodeOp, NodeTypeId};
use crate::time::TimeMap;
use crate::value::CookValue;
use std::collections::BTreeMap;

/// Identifies the chain of [`TimeMap`]s a node is being cooked under (plan
/// §1.5). `0` = the outer clock, i.e. no remap — the only key a graph without
/// time scopes ever uses, so its behaviour and memo are exactly as before.
///
/// A node reached through two different scope chains in one frame (a diamond
/// where one arm crosses a remapper) is cooked once **per chain**: the memo is
/// keyed by `(NodeId, ScopeKey)`. Keying by `NodeId` alone would let the second
/// arm read the first arm's stream, silently sampled at the wrong time.
pub type ScopeKey = u64;

/// The `ScopeKey` of the outer clock.
pub const SCOPE_ROOT: ScopeKey = 0;

/// Time scopes to apply while cooking: `node -> map` for each remapper node.
/// The map rewrites the clock of that node's **upstream subtree**, never of the
/// node itself. Built by the domain layer (which knows its node types) — the
/// substrate stays type-agnostic.
pub type TimeScopes = BTreeMap<NodeId, TimeMap>;

/// Push `map` (applied at `node`) onto a scope chain. FNV-1a over the node id
/// and the map's bits, so distinct chains key distinct memo lanes.
fn push_scope(key: ScopeKey, node: NodeId, map: &TimeMap) -> ScopeKey {
    let mut hash = if key == SCOPE_ROOT {
        0xcbf2_9ce4_8422_2325
    } else {
        key
    };
    for b in node.0.to_le_bytes() {
        hash ^= u64::from(b);
        hash = hash.wrapping_mul(0x0000_0100_0000_01b3);
    }
    map.hash_into(&mut hash);
    // Never collide with the root: a scoped lane must not alias the unscoped one.
    if hash == SCOPE_ROOT { 1 } else { hash }
}

/// Resolves a node type id to its operation impl. Implemented by the node
/// registry (W1.T3); kept as a trait so the cook engine is decoupled from it.
pub trait OpResolver {
    fn resolve(&self, ty: NodeTypeId) -> Option<&dyn NodeOp>;
}

#[path = "cook_eval_ctx.rs"]
mod eval_ctx;
pub use eval_ctx::EvalCtx;

#[derive(Debug, PartialEq, Eq)]
pub enum CookError {
    /// Target (or an upstream source) is not a node in this graph.
    UnknownNode,
    /// A node's type is not registered with the resolver.
    UnknownType,
    /// A node emitted a number of outputs that disagrees with its manifest —
    /// a node-implementation bug that would otherwise leak as empty streams.
    OutputCountMismatch {
        node: NodeId,
        expected: usize,
        got: usize,
    },
    /// A sequential node (one consuming a `pre` edge) sits inside a remapped
    /// time scope (plan §1.5, v1 restriction). Its state is a recurrence over
    /// the **outer** tick, so a rewritten clock has no defined meaning: under
    /// `Loop` it would be asked to relive ticks it has already integrated,
    /// under `Freeze` to advance while time stands still. Refused loudly at
    /// cook time rather than silently producing a plausible wrong trajectory.
    /// The editor badges the offending node.
    SequentialInTimeScope { node: NodeId },
}

struct Cached {
    outputs: Vec<CookValue>,
    revision: u64,
    fingerprint: Fingerprint,
    /// The externals this node read when it last cooked (doc 65) — the only way the NEXT reuse
    /// decision can know which published values it depends on.
    read_externals: Vec<String>,
}

/// A snapshot of the **simulation state** carried across ticks — the `pre`-edge
/// feedback ([`Cook::prev_outputs`]) plus the sequential tick counter — captured
/// by [`Cook::checkpoint`] and reinstated by [`Cook::restore`] (plan §1.4,
/// M2.N2). Enough to reproduce any later frame by restoring and re-cooking
/// forward: GGPO's *"buffer sufficient to restore"*, and bit-exact here because
/// the cook is deterministic (no transcendentals, hashed RNG — ADR-0032, HR-5).
///
/// **Why this is all of it:** on the pull side no node keeps hidden per-node
/// state — every sequential node (integrate/spring/step/strobe/trail/threshold/
/// beat) carries its entire recurrence in stream columns on its `pre` self-loop
/// (ADR-0032, verified by the 2026-07-10 node audit). So the `pre` snapshots ARE
/// the simulation; the memo cache and the revision clock are derivable and are
/// deliberately excluded (the cache is stale for a rewound clock; the revision
/// clock stays live so a restore reads as a change and redraws).
///
/// The clone is a deep copy of the state columns (`O(state size)`); an `Arc`/COW
/// column would make it cheap for large particle sets — a measured follow-up
/// (the GGRS sparse-saving path keys off exactly this cost).
#[derive(Clone, Default)]
pub struct CookCheckpoint {
    prev_outputs: BTreeMap<NodeId, Vec<CookValue>>,
    tick: u64,
    /// The clock the last tick closed on — restored with the state, so a replayed tick takes
    /// exactly the `dt` it took the first time.
    prev_playhead: Option<f64>,
}

impl CookCheckpoint {
    /// The checkpoint's stream bytes — what a byte-budgeted ring charges for
    /// holding it (ADR-0137). See [`crate::attr::Column::approx_bytes`] for
    /// what "approx" means and which way it errs.
    pub fn approx_bytes(&self) -> usize {
        self.prev_outputs
            .values()
            .flat_map(|vs| vs.iter())
            .map(CookValue::approx_bytes)
            .sum()
    }
}

/// O sub-tique — [`Cook::substep`] e o que ele tem de saber sobre o relogio. FILHO de
/// proposito: ele mexe nos campos privados do `Cook` (`tick`, `prev_playhead`,
/// `prev_outputs`, `cache`), que um modulo IRMAO nao enxergaria.
#[path = "cook_substep.rs"]
mod substep;
pub use substep::{SUBSTEPS_PARAM, SubstepIsland, graph_substeps, substep_islands, upstream_cone};

/// Incremental cook engine. Holds the memo cache and the previous-tick snapshot
/// across cooks; reusing the same `Cook` across frames is what makes
/// re-evaluation cheap and `pre` feedback work.
#[derive(Default)]
pub struct Cook {
    /// **What the app published** (doc 65). Keyed by name; the revision IS the content, so
    /// republishing the same curve every frame invalidates nothing.
    externals: crate::external::All,
    /// Keyed by `(node, scope)`: the same node cooked under two different time
    /// scopes in one frame holds two independent memo entries. Without a scope,
    /// every key is `(node, SCOPE_ROOT)`.
    cache: BTreeMap<(NodeId, ScopeKey), Cached>,
    prev_outputs: BTreeMap<NodeId, Vec<CookValue>>,
    /// The playhead the last tick closed on (`None` before the first). `EvalCtx::dt` is the
    /// step from it — the engine's own clock delta, which the stateful nodes used to have to
    /// reconstruct from a column of their own.
    prev_playhead: Option<f64>,
    /// Scope lanes touched since the last `advance_tick*`. A remap param edit
    /// (a slider drag!) mints a NEW `ScopeKey` every value it passes through,
    /// and each lane holds full `Stream`s: without pruning, one drag across a
    /// slider strands hundreds of subtree copies for the process's lifetime.
    /// Lanes not visited this frame are dropped when the tick advances.
    live_keys: std::collections::BTreeSet<ScopeKey>,
    tick: u64,
    /// Monotonic revision clock. Bumped only on an actual recompute; a node's
    /// stored revision changes iff it recomputed, so a downstream consumer
    /// detects change by a changed input revision. (Replaces the earlier
    /// max-scan, which could recede once cache eviction lands → false hits.)
    rev_counter: u64,
}

impl Cook {
    /// **Publish a value into the cook** (doc 65) — the door from the app into the graph.
    ///
    /// The revision is the CONTENT (`external::fingerprint`), so a caller cannot get the
    /// bookkeeping wrong: republishing the same curve every frame is free, and editing it
    /// invalidates exactly the nodes that read it.
    pub fn set_external(&mut self, name: impl Into<String>, value: crate::attr::Stream) {
        let rev = crate::external::fingerprint(&value);
        self.externals
            .insert(name.into(), crate::external::External { rev, value });
    }

    /// Forget everything published (the shell republishes what still exists each frame, so this is
    /// how a deleted shape stops being visible to the graph).
    pub fn clear_externals(&mut self) {
        self.externals.clear();
    }

    /// What is published right now (read-only — the tests and the panel's readout).
    pub fn externals(&self) -> &crate::external::All {
        &self.externals
    }

    pub fn new() -> Self {
        Self::default()
    }

    /// Advance one tick: snapshot the outputs of every node that feeds a `pre`
    /// edge, so those edges read them next tick. Each `pre` source is **cooked
    /// here at `playhead`** (memoized if the frame's target already pulled it)
    /// before snapshotting — because a `pre` source is part of the live
    /// sequential circuit and must hold a current value even if this frame's
    /// cook target never pulled it (a forward consumer is not required). Call
    /// once per frame, after the frame's `cook`(s), with the same `playhead`.
    pub fn advance_tick(
        &mut self,
        graph: &Graph,
        ops: &dyn OpResolver,
        playhead: f64,
    ) -> Result<(), CookError> {
        self.advance_tick_scoped(graph, ops, playhead, &TimeScopes::new())
    }

    /// [`Self::advance_tick`] under time scopes: a `pre` source whose upstream
    /// crosses a remapper must snapshot the remapped subtree, not the raw one.
    /// The sources themselves are on the outer clock ([`SCOPE_ROOT`]) — a
    /// sequential node inside a scope is refused by [`Self::cook_scoped`].
    pub fn advance_tick_scoped(
        &mut self,
        graph: &Graph,
        ops: &dyn OpResolver,
        playhead: f64,
        scopes: &TimeScopes,
    ) -> Result<(), CookError> {
        let pre_sources: std::collections::BTreeSet<NodeId> = graph
            .edges()
            .iter()
            .filter(|e| e.delayed)
            .map(|e| e.from.0)
            .collect();
        for &src in &pre_sources {
            self.cook_node(graph, ops, src, playhead, SCOPE_ROOT, scopes)?;
        }
        self.prev_playhead = Some(playhead);
        self.prev_outputs = self
            .cache
            .iter()
            .filter(|((id, key), _)| *key == SCOPE_ROOT && pre_sources.contains(id))
            .map(|((id, _), c)| (*id, c.outputs.clone()))
            .collect();
        // Drop the scope lanes this frame never visited (see `live_keys`). The
        // root lane is never pruned — it is the graph's own memo, and pruning
        // it would defeat the incrementality this engine exists for.
        let live = std::mem::take(&mut self.live_keys);
        self.cache
            .retain(|(_, key), _| *key == SCOPE_ROOT || live.contains(key));
        self.tick += 1;
        Ok(())
    }

    /// Capture the current simulation state — the `pre` feedback + the sequential
    /// tick — for later [`Self::restore`] (plan §1.4, M2.N2). Take it at the
    /// point in the tick loop where cooking `target` would reproduce a specific
    /// frame: i.e. **before that frame's `cook`**, which is exactly the state
    /// left by the previous frame's [`Self::advance_tick`]. Then a scrub is
    /// `restore(nearest checkpoint ≤ target)` followed by `cook; advance_tick`
    /// forward to `target` — bit-exact, because it walks the identical cook path
    /// as forward playback (GGPO save/load/advance).
    pub fn checkpoint(&self) -> CookCheckpoint {
        CookCheckpoint {
            prev_outputs: self.prev_outputs.clone(),
            prev_playhead: self.prev_playhead,
            tick: self.tick,
        }
    }

    /// Reinstate a [`Self::checkpoint`]ed simulation state, so the next `cook`
    /// reproduces the frame that checkpoint was taken before. The memo cache is
    /// **cleared** — its entries are stale for a rewound clock (a sequential
    /// node's fingerprint keys on the tick, a `Temporal` node's on the playhead,
    /// both of which just jumped), so a stale hit would serve a future frame
    /// (GGPO's *"invalidate the forward memo"*). The monotonic revision clock is
    /// **kept** so the recompute reads as a change downstream and the scene
    /// redraws. Scope lanes are dropped with the cache.
    pub fn restore(&mut self, cp: &CookCheckpoint) {
        self.prev_outputs = cp.prev_outputs.clone();
        self.prev_playhead = cp.prev_playhead;
        self.tick = cp.tick;
        self.cache.clear();
        self.live_keys.clear();
    }

    /// **Read `node`'s memoized outputs WITHOUT cooking anything** — `None` if this
    /// frame's cook never pulled it (root time lane).
    ///
    /// This is what makes an editor's inline readouts free: the frame's cook has already
    /// evaluated every node that feeds a sink, and their results are sitting right here.
    /// A reader that called [`Self::cook`] instead would be *correct* and still wrong — it
    /// would evaluate nodes the render never needed, once per card per frame, and turn a
    /// glance at the graph into a second full evaluation of it.
    ///
    /// **`None` is information, not a failure.** A node that no sink consumes was never
    /// cooked and has nothing to show — which is precisely the fact the artist wants to see
    /// (a card sitting there with no reading is a card wired to nothing).
    ///
    /// Root lane only: a node inside a `motion.time_remap` scope cooks on another clock and
    /// holds a reading per scope, so there is no single "the" value to report.
    pub fn peek(&self, node: NodeId) -> Option<&[CookValue]> {
        self.cache
            .get(&(node, SCOPE_ROOT))
            .map(|c| c.outputs.as_slice())
    }

    /// Cook `target`'s outputs at `playhead`, pulling upstream on demand and
    /// reusing memoized results whose inputs are unchanged.
    pub fn cook(
        &mut self,
        graph: &Graph,
        ops: &dyn OpResolver,
        target: NodeId,
        playhead: f64,
    ) -> Result<&[CookValue], CookError> {
        self.cook_scoped(graph, ops, target, playhead, &TimeScopes::new())
    }

    /// [`Self::cook`] under time scopes (plan §1.5, M2.N1). Descending into the
    /// inputs of a node listed in `scopes` rewrites the clock of that node's
    /// whole upstream subtree; the node itself stays on the outer clock. An
    /// empty map is exactly [`Self::cook`] — same traversal, same memo lane.
    ///
    /// The memo carries the win: a `Loop` that returns to a `t'` the subtree
    /// already cooked at this tick hits the cache instead of recomputing it.
    pub fn cook_scoped(
        &mut self,
        graph: &Graph,
        ops: &dyn OpResolver,
        target: NodeId,
        playhead: f64,
        scopes: &TimeScopes,
    ) -> Result<&[CookValue], CookError> {
        self.cook_node(graph, ops, target, playhead, SCOPE_ROOT, scopes)?;
        Ok(&self
            .cache
            .get(&(target, SCOPE_ROOT))
            .expect("just cooked")
            .outputs)
    }

    /// Returns the node's current revision (bumped iff it recomputed).
    ///
    /// `playhead`/`key` are the clock and scope chain **this node** cooks under;
    /// its inputs may cook under a different pair when it is itself a remapper.
    fn cook_node(
        &mut self,
        graph: &Graph,
        ops: &dyn OpResolver,
        node: NodeId,
        playhead: f64,
        key: ScopeKey,
        scopes: &TimeScopes,
    ) -> Result<u64, CookError> {
        let inst = graph.node(node).ok_or(CookError::UnknownNode)?;
        let op = ops.resolve(inst.type_id()).ok_or(CookError::UnknownType)?;
        let manifest = op.manifest();
        if key != SCOPE_ROOT {
            self.live_keys.insert(key);
        }

        // A remapper rewrites the clock of everything it pulls. An identity map
        // is dropped so its subtree shares the unscoped memo lane (a remapper
        // left at its defaults costs nothing).
        let (in_playhead, in_key) = match scopes.get(&node) {
            Some(map) if !map.is_identity() => (map.apply(playhead), push_scope(key, node, map)),
            _ => (playhead, key),
        };

        // 1. Resolve inputs: cook forward edges (recording revisions); read
        //    `pre` edges from the previous-tick snapshot without recursing.
        let mut input_values: Vec<CookValue> = Vec::with_capacity(manifest.inputs.len());
        let mut input_revs: Vec<u64> = Vec::new();
        let mut consumes_pre = false;
        for port in 0..manifest.inputs.len() {
            match graph.input_edge(node, port) {
                Some((src, src_port, false)) => {
                    let rev = self.cook_node(graph, ops, src, in_playhead, in_key, scopes)?;
                    input_revs.push(rev);
                    input_values.push(self.cur_output(src, in_key, src_port));
                }
                Some((src, src_port, true)) => {
                    consumes_pre = true;
                    input_values.push(self.prev_output(src, src_port));
                }
                None => input_values.push(CookValue::Empty),
            }
        }

        // 1b. Resolve DRIVEN PARAMS (doc 58) — the same recursion, the same revisions.
        //     A driven param is a dependency: cook it here, or the driver would be read
        //     from a stale memo lane (or never cooked at all, since nothing else pulls it).
        let sources = graph.param_sources(node);
        let mut driven: BTreeMap<&str, f32> = BTreeMap::new();
        if let Some(sources) = sources {
            for (name, (src, src_port)) in sources {
                let rev = self.cook_node(graph, ops, *src, in_playhead, in_key, scopes)?;
                input_revs.push(rev);
                if let Some(v) = crate::param_source::driven_value(&self.cur_output(
                    *src,
                    in_key,
                    *src_port as usize,
                )) {
                    driven.insert(name.as_str(), v);
                }
                // An empty driver leaves the param FALLING BACK to its override/default
                // rather than to 0.0: a wire that has not produced a number yet has not
                // said the number is zero, and a scene that goes to zero on a frame where
                // an emitter has not spawned yet is a scene that flickers.
            }
        }

        // A recurrence over the outer tick cannot run on a rewritten clock
        // (see `CookError::SequentialInTimeScope`). Refuse before evaluating.
        if consumes_pre && key != SCOPE_ROOT {
            return Err(CookError::SequentialInTimeScope { node });
        }

        // 2. Reuse decision (memoization). A `pre`-consuming (sequential) node
        //    keys on the tick, so it recomputes once per tick and drives its
        //    feedback loop; a purely combinational node stays memoized.
        let fingerprint = Fingerprint {
            input_revs,
            playhead: (manifest.effect == Effect::Temporal).then_some(playhead.to_bits()),
            tick: consumes_pre.then_some(self.tick),
            params: params_fingerprint(graph.node_param_overrides(node)),
            text_params: text_params_fingerprint(graph.node_text_param_overrides(node)),
            // A muted node cooks a passthrough, not its op — flipping it must recompute.
            bypassed: graph.node_bypassed(node),
            param_sources: crate::param_source::fingerprint(sources),
            // The externals this node read LAST time, at their revisions NOW (doc 65). A node that
            // has never cooked has read nothing, so this is the hash of the empty list — and its
            // first cook happens for the ordinary reason (no cache entry).
            externals: crate::external::revs_of(
                &self.externals,
                self.cache
                    .get(&(node, key))
                    .map_or(&[][..], |c| &c.read_externals),
            ),
        };
        if let Some(c) = self.cache.get(&(node, key))
            && c.fingerprint == fingerprint
        {
            return Ok(c.revision);
        }

        // 3. Recompute.
        let mut ctx = EvalCtx {
            inputs: &input_values,
            externals: &self.externals,
            read_externals: Vec::new(),
            playhead,
            manifest,
            overrides: graph.node_param_overrides(node),
            text_overrides: graph.node_text_param_overrides(node),
            driven,
            started: self.prev_outputs.contains_key(&node),
            node_key: node.0,
            // The ROOT clock's step. A rewritten lane has no meaningful delta across ticks —
            // and a node that needs one to hold state is sequential, which a scope refuses.
            dt: if key == SCOPE_ROOT {
                self.prev_playhead.map_or(0.0, |p| playhead - p)
            } else {
                0.0
            },
            outputs: Vec::new(),
        };
        // BYPASS/MUTE (H): a switched-off node's op never runs — it passes port 0 straight through
        // (`cook_bypass`). Otherwise the op computes as usual.
        if graph.node_bypassed(node) {
            ctx.outputs = cook_bypass::bypass_outputs(&input_values, manifest.outputs.len());
        } else {
            op.eval(&mut ctx);
        }
        let n_out = manifest.outputs.len();
        if ctx.outputs.len() != n_out {
            return Err(CookError::OutputCountMismatch {
                node,
                expected: n_out,
                got: ctx.outputs.len(),
            });
        }
        self.rev_counter += 1;
        let revision = self.rev_counter;
        self.cache.insert(
            (node, key),
            Cached {
                outputs: ctx.outputs,
                revision,
                // **The stored fingerprint must be a FIXED POINT** (doc 65). The one that made the
                // reuse decision was built from the externals the node read LAST time — which, on
                // a first cook, is *nothing*. Store that and the very next cook compares
                // "nothing" against "Track" and recomputes for no reason: one spurious eval, every
                // time a node first reads a curve.
                //
                // So the entry remembers the fingerprint of what it ACTUALLY depends on. It is the
                // same discipline the undo snapshot needed
                // ([[feedback_a_snapshot_must_be_a_fixed_point_of_the_systems]]): if the next pass
                // would derive something different from the same inputs, you have not converged.
                fingerprint: Fingerprint {
                    externals: crate::external::revs_of(&self.externals, &ctx.read_externals),
                    ..fingerprint
                },
                read_externals: ctx.read_externals,
            },
        );
        Ok(revision)
    }
}

// The cached-output readers live in a sibling for the LOC cap.
#[path = "cook_read.rs"]
mod cook_read;

#[cfg(test)]
#[path = "cook_tests.rs"]
mod tests;

#[cfg(test)]
#[path = "cook_param_source_tests.rs"]
mod param_source_tests;

#[cfg(test)]
#[path = "cook_external_tests.rs"]
mod external_tests;
