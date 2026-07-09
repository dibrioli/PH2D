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

use crate::attr::Stream;
use crate::effect::Effect;
use crate::graph::{Graph, NodeId};
use crate::node::{NodeManifest, NodeOp, NodeTypeId};
use crate::time::TimeMap;
use crate::value::CookValue;
use std::any::Any;
use std::collections::BTreeMap;
use std::sync::Arc;

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

/// Per-eval context handed to a node. A node sees **only** this — its typed
/// inputs, the playhead, and its own resolved parameters — never the graph.
/// FBP black box (ADR-0031).
pub struct EvalCtx<'a> {
    inputs: &'a [CookValue],
    playhead: f64,
    manifest: &'static NodeManifest,
    overrides: Option<&'a BTreeMap<String, f32>>,
    outputs: Vec<CookValue>,
}

impl<'a> EvalCtx<'a> {
    /// The cooked **instance stream** on input `port` (empty if unconnected, or
    /// if the upstream emitted a non-stream value; for a `pre` port, the
    /// previous tick's value). The value's domain is guaranteed by `PortType`
    /// checking at connect time, so a motion node reads its columns directly.
    pub fn input(&self, port: usize) -> &Stream {
        self.inputs[port].as_stream()
    }

    /// The cooked **opaque value** on input `port` (e.g. a geometry
    /// `VectorNetwork`), type-erased; the domain layer downcasts it. `None` if
    /// the input is unconnected or carries an instance stream rather than an
    /// opaque value (ADR-0058-amendment-1).
    pub fn input_any(&self, port: usize) -> Option<&(dyn Any + Send + Sync)> {
        self.inputs.get(port).and_then(CookValue::as_any)
    }

    pub fn input_count(&self) -> usize {
        self.inputs.len()
    }

    /// Current clock time; meaningful for `Temporal` nodes.
    pub fn playhead(&self) -> f64 {
        self.playhead
    }

    /// The current value of parameter `name`: the graph's per-instance override
    /// if set ([`crate::graph::Graph::set_param`]), else the node type's
    /// manifest default. Panics if `name` is not a declared param of this node
    /// — a programmer error (the name is a literal of the node's own crate),
    /// caught by its golden test rather than silently reading `0.0`, the same
    /// no-silent-failure discipline as [`NodeManifest::param_default`].
    pub fn param(&self, name: &str) -> f32 {
        self.overrides
            .and_then(|o| o.get(name).copied())
            .or_else(|| self.manifest.param_default(name))
            .unwrap_or_else(|| {
                panic!(
                    "node `{}` read undeclared param `{name}`",
                    self.manifest.name
                )
            })
    }

    /// Emit the next output port's **instance stream**. Call once per output
    /// port, in order.
    pub fn emit(&mut self, stream: Stream) {
        self.outputs.push(CookValue::Instances(stream));
    }

    /// Emit the next output port's **opaque value** — a domain-specific rich
    /// value (e.g. a geometry `VectorNetwork`) carried type-erased behind
    /// `Arc<dyn Any>` (ADR-0058-amendment-1). Call once per output port, in
    /// order, just like [`Self::emit`]. The domain layer
    /// (`ph2d-vector-graph::VectorEvalExt::emit_network`) wraps this.
    pub fn emit_any(&mut self, value: Arc<dyn Any + Send + Sync>) {
        self.outputs.push(CookValue::Opaque(value));
    }
}

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

/// What a node's reuse decision depends on: revisions of its forward inputs,
/// the playhead bits (if `Temporal`), and the tick (if it consumes a `pre`
/// edge, i.e. is sequential and must advance every tick). Playhead is stored
/// as `to_bits` so the key is a stable bitwise compare (no NaN-self-inequality,
/// no `-0.0`/`+0.0` aliasing).
#[derive(Clone, Default, PartialEq)]
struct Fingerprint {
    input_revs: Vec<u64>,
    playhead: Option<u64>,
    tick: Option<u64>,
    /// FNV-1a of the node's per-instance param overrides (name + value bits).
    /// Folds edited params into the reuse decision: an override change must
    /// recompute, or a re-cook with the same `Cook` would return a stale,
    /// pre-edit stream. Manifest defaults are compile-time constant, so only
    /// overrides can change at runtime — hashing them suffices.
    params: u64,
}

/// FNV-1a over a node's overrides, in `BTreeMap` (deterministic) order. `None`
/// or empty → the FNV offset basis (a stable constant for "no overrides").
///
/// Each name is **length-prefixed** so the byte stream is unambiguous: without
/// it, `{"p": x, "q": y}` and `{"pemonq": y}` can flatten to the same bytes and
/// collide, which (since the fingerprint gates memo reuse) would return a stale,
/// pre-edit stream — a silent wrong result. The length prefix makes the
/// encoding injective, so distinct override sets always hash distinctly.
fn params_fingerprint(overrides: Option<&BTreeMap<String, f32>>) -> u64 {
    let mut hash: u64 = 0xcbf2_9ce4_8422_2325;
    let mut mix = |bytes: &[u8]| {
        for b in bytes {
            hash ^= *b as u64;
            hash = hash.wrapping_mul(0x0000_0100_0000_01b3);
        }
    };
    if let Some(map) = overrides {
        for (name, value) in map {
            mix(&(name.len() as u64).to_le_bytes());
            mix(name.as_bytes());
            mix(&value.to_bits().to_le_bytes());
        }
    }
    hash
}

struct Cached {
    outputs: Vec<CookValue>,
    revision: u64,
    fingerprint: Fingerprint,
}

/// Incremental cook engine. Holds the memo cache and the previous-tick snapshot
/// across cooks; reusing the same `Cook` across frames is what makes
/// re-evaluation cheap and `pre` feedback work.
#[derive(Default)]
pub struct Cook {
    /// Keyed by `(node, scope)`: the same node cooked under two different time
    /// scopes in one frame holds two independent memo entries. Without a scope,
    /// every key is `(node, SCOPE_ROOT)`.
    cache: BTreeMap<(NodeId, ScopeKey), Cached>,
    prev_outputs: BTreeMap<NodeId, Vec<CookValue>>,
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
        };
        if let Some(c) = self.cache.get(&(node, key))
            && c.fingerprint == fingerprint
        {
            return Ok(c.revision);
        }

        // 3. Recompute.
        let mut ctx = EvalCtx {
            inputs: &input_values,
            playhead,
            manifest,
            overrides: graph.node_param_overrides(node),
            outputs: Vec::new(),
        };
        op.eval(&mut ctx);
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
                fingerprint,
            },
        );
        Ok(revision)
    }

    fn cur_output(&self, node: NodeId, key: ScopeKey, port: usize) -> CookValue {
        self.cache
            .get(&(node, key))
            .and_then(|c| c.outputs.get(port))
            .cloned()
            .unwrap_or_default()
    }

    fn prev_output(&self, node: NodeId, port: usize) -> CookValue {
        self.prev_outputs
            .get(&node)
            .and_then(|outs| outs.get(port))
            .cloned()
            .unwrap_or_default()
    }
}

#[cfg(test)]
#[path = "cook_tests.rs"]
mod tests;
