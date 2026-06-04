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
use crate::value::CookValue;
use std::any::Any;
use std::collections::BTreeMap;
use std::sync::Arc;

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
    cache: BTreeMap<NodeId, Cached>,
    prev_outputs: BTreeMap<NodeId, Vec<CookValue>>,
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
        let pre_sources: std::collections::BTreeSet<NodeId> = graph
            .edges()
            .iter()
            .filter(|e| e.delayed)
            .map(|e| e.from.0)
            .collect();
        for &src in &pre_sources {
            self.cook_node(graph, ops, src, playhead)?;
        }
        self.prev_outputs = self
            .cache
            .iter()
            .filter(|(id, _)| pre_sources.contains(id))
            .map(|(id, c)| (*id, c.outputs.clone()))
            .collect();
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
        self.cook_node(graph, ops, target, playhead)?;
        Ok(&self.cache.get(&target).expect("just cooked").outputs)
    }

    /// Returns the node's current revision (bumped iff it recomputed).
    fn cook_node(
        &mut self,
        graph: &Graph,
        ops: &dyn OpResolver,
        node: NodeId,
        playhead: f64,
    ) -> Result<u64, CookError> {
        let inst = graph.node(node).ok_or(CookError::UnknownNode)?;
        let op = ops.resolve(inst.type_id()).ok_or(CookError::UnknownType)?;
        let manifest = op.manifest();

        // 1. Resolve inputs: cook forward edges (recording revisions); read
        //    `pre` edges from the previous-tick snapshot without recursing.
        let mut input_values: Vec<CookValue> = Vec::with_capacity(manifest.inputs.len());
        let mut input_revs: Vec<u64> = Vec::new();
        let mut consumes_pre = false;
        for port in 0..manifest.inputs.len() {
            match graph.input_edge(node, port) {
                Some((src, src_port, false)) => {
                    let rev = self.cook_node(graph, ops, src, playhead)?;
                    input_revs.push(rev);
                    input_values.push(self.cur_output(src, src_port));
                }
                Some((src, src_port, true)) => {
                    consumes_pre = true;
                    input_values.push(self.prev_output(src, src_port));
                }
                None => input_values.push(CookValue::Empty),
            }
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
        if let Some(c) = self.cache.get(&node)
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
            node,
            Cached {
                outputs: ctx.outputs,
                revision,
                fingerprint,
            },
        );
        Ok(revision)
    }

    fn cur_output(&self, node: NodeId, port: usize) -> CookValue {
        self.cache
            .get(&node)
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
mod tests {
    use super::*;
    use crate::attr::Column;
    use crate::graph::Edge;
    use crate::node::{LoweringKind, NodeManifest, PortSpec};
    use crate::port::{Clock, Dim, Domain, PortType};
    use std::sync::atomic::{AtomicU64, Ordering};

    const SCALAR_FRAME: PortType = PortType::new(Domain::Instances, Dim::Scalar, Clock::Frame);
    const fn port(name: &'static str) -> PortSpec {
        PortSpec {
            name,
            ty: SCALAR_FRAME,
        }
    }

    static GEN_MAN: NodeManifest = NodeManifest {
        id: NodeTypeId::of("test.gen"),
        name: "test.gen",
        inputs: &[],
        outputs: &[port("out")],
        effect: Effect::Pure,
        clock: Clock::Frame,
        params: &[],
        lowerings: &[LoweringKind::Cpu],
    };
    static SCALE_MAN: NodeManifest = NodeManifest {
        id: NodeTypeId::of("test.scale"),
        name: "test.scale",
        inputs: &[port("in")],
        outputs: &[port("out")],
        effect: Effect::Pure,
        clock: Clock::Frame,
        params: &[],
        lowerings: &[LoweringKind::Cpu],
    };
    static ACC_MAN: NodeManifest = NodeManifest {
        id: NodeTypeId::of("test.acc"),
        name: "test.acc",
        inputs: &[port("incr"), port("feedback")],
        outputs: &[port("out")],
        effect: Effect::Pure,
        clock: Clock::Frame,
        params: &[],
        lowerings: &[LoweringKind::Cpu],
    };

    fn scalars(s: &Stream) -> Vec<f32> {
        match s.get("v") {
            Some(Column::Scalar(v)) => v.clone(),
            _ => vec![],
        }
    }
    // A cooked output port is a `CookValue`; motion outputs view as a stream.
    fn out_scalars(v: &CookValue) -> Vec<f32> {
        scalars(v.as_stream())
    }

    struct Gen {
        calls: AtomicU64,
    }
    impl NodeOp for Gen {
        fn manifest(&self) -> &'static NodeManifest {
            &GEN_MAN
        }
        fn eval(&self, ctx: &mut EvalCtx<'_>) {
            self.calls.fetch_add(1, Ordering::Relaxed);
            ctx.emit(Stream::new(3).with("v", Column::Scalar(vec![1.0, 2.0, 3.0])));
        }
    }

    struct Scale;
    impl NodeOp for Scale {
        fn manifest(&self) -> &'static NodeManifest {
            &SCALE_MAN
        }
        fn eval(&self, ctx: &mut EvalCtx<'_>) {
            let out: Vec<f32> = scalars(ctx.input(0)).iter().map(|x| x * 2.0).collect();
            ctx.emit(Stream::new(out.len()).with("v", Column::Scalar(out)));
        }
    }

    static BAD_MAN: NodeManifest = NodeManifest {
        id: NodeTypeId::of("test.bad"),
        name: "test.bad",
        inputs: &[],
        outputs: &[port("out")], // declares one output...
        effect: Effect::Pure,
        clock: Clock::Frame,
        params: &[],
        lowerings: &[LoweringKind::Cpu],
    };

    /// out = incr + feedback(pre). Classic accumulator over the clock.
    struct Acc {
        calls: AtomicU64,
    }
    impl NodeOp for Acc {
        fn manifest(&self) -> &'static NodeManifest {
            &ACC_MAN
        }
        fn eval(&self, ctx: &mut EvalCtx<'_>) {
            self.calls.fetch_add(1, Ordering::Relaxed);
            let incr = scalars(ctx.input(0));
            let fb = scalars(ctx.input(1));
            let out: Vec<f32> = incr
                .iter()
                .enumerate()
                .map(|(i, x)| x + fb.get(i).copied().unwrap_or(0.0))
                .collect();
            ctx.emit(Stream::new(out.len()).with("v", Column::Scalar(out)));
        }
    }

    /// ...but emits zero outputs — a node-implementation bug the cook must catch.
    struct Bad;
    impl NodeOp for Bad {
        fn manifest(&self) -> &'static NodeManifest {
            &BAD_MAN
        }
        fn eval(&self, _ctx: &mut EvalCtx<'_>) {}
    }

    static DELAY_MAN: NodeManifest = NodeManifest {
        id: NodeTypeId::of("test.delay"),
        name: "test.delay",
        inputs: &[port("in")],
        outputs: &[port("out")],
        effect: Effect::Pure,
        clock: Clock::Frame,
        params: &[],
        lowerings: &[LoweringKind::Cpu],
    };

    /// Emits its single input verbatim (used with a `pre` edge as a delay line).
    struct Delay;
    impl NodeOp for Delay {
        fn manifest(&self) -> &'static NodeManifest {
            &DELAY_MAN
        }
        fn eval(&self, ctx: &mut EvalCtx<'_>) {
            let passthrough = ctx.input(0).clone();
            ctx.emit(passthrough);
        }
    }

    struct Ops {
        generator: Gen,
        scale: Scale,
        acc: Acc,
        bad: Bad,
        delay: Delay,
    }
    impl OpResolver for Ops {
        fn resolve(&self, ty: NodeTypeId) -> Option<&dyn NodeOp> {
            match ty {
                t if t == GEN_MAN.id => Some(&self.generator),
                t if t == SCALE_MAN.id => Some(&self.scale),
                t if t == ACC_MAN.id => Some(&self.acc),
                t if t == BAD_MAN.id => Some(&self.bad),
                t if t == DELAY_MAN.id => Some(&self.delay),
                _ => None,
            }
        }
    }

    fn ops() -> Ops {
        Ops {
            generator: Gen {
                calls: AtomicU64::new(0),
            },
            scale: Scale,
            acc: Acc {
                calls: AtomicU64::new(0),
            },
            bad: Bad,
            delay: Delay,
        }
    }

    #[test]
    fn cooks_a_chain_end_to_end() {
        let mut g = Graph::new();
        let generator = g.add_node("test.gen");
        let scale = g.add_node("test.scale");
        g.connect(Edge {
            from: (generator, 0),
            to: (scale, 0),
            delayed: false,
        })
        .unwrap();
        let o = ops();
        let mut cook = Cook::new();
        let out = cook.cook(&g, &o, scale, 0.0).unwrap();
        assert_eq!(out_scalars(&out[0]), vec![2.0, 4.0, 6.0]);
    }

    #[test]
    fn memoizes_unchanged_upstream_across_cooks() {
        let mut g = Graph::new();
        let generator = g.add_node("test.gen");
        let scale = g.add_node("test.scale");
        g.connect(Edge {
            from: (generator, 0),
            to: (scale, 0),
            delayed: false,
        })
        .unwrap();
        let o = ops();
        let mut cook = Cook::new();
        cook.cook(&g, &o, scale, 0.0).unwrap();
        cook.advance_tick(&g, &o, 0.0).unwrap();
        cook.cook(&g, &o, scale, 0.0).unwrap();
        // Combinational + unchanged → generator evaluated exactly once.
        assert_eq!(o.generator.calls.load(Ordering::Relaxed), 1);
    }

    #[test]
    fn pre_feedback_accumulates_across_ticks() {
        // acc.incr <- gen ; acc.feedback <- pre(acc).
        let mut g = Graph::new();
        let generator = g.add_node("test.gen");
        let acc = g.add_node("test.acc");
        g.connect(Edge {
            from: (generator, 0),
            to: (acc, 0),
            delayed: false,
        })
        .unwrap();
        g.connect(Edge {
            from: (acc, 0),
            to: (acc, 1),
            delayed: true,
        })
        .unwrap();
        let o = ops();
        let mut cook = Cook::new();

        // tick 0: feedback empty → gen
        assert_eq!(
            out_scalars(&cook.cook(&g, &o, acc, 0.0).unwrap()[0]),
            vec![1.0, 2.0, 3.0]
        );
        cook.advance_tick(&g, &o, 0.0).unwrap();
        // tick 1: gen + prev(=[1,2,3])
        assert_eq!(
            out_scalars(&cook.cook(&g, &o, acc, 0.0).unwrap()[0]),
            vec![2.0, 4.0, 6.0]
        );
        cook.advance_tick(&g, &o, 0.0).unwrap();
        // tick 2: gen + prev(=[2,4,6])
        assert_eq!(
            out_scalars(&cook.cook(&g, &o, acc, 0.0).unwrap()[0]),
            vec![3.0, 6.0, 9.0]
        );

        // gen is combinational/unchanged → evaluated once despite 3 ticks.
        assert_eq!(o.generator.calls.load(Ordering::Relaxed), 1);
    }

    #[test]
    fn unknown_type_errors() {
        let mut g = Graph::new();
        let n = g.add_node("test.nonexistent");
        let o = ops();
        let mut cook = Cook::new();
        assert_eq!(
            cook.cook(&g, &o, n, 0.0).map(|_| ()),
            Err(CookError::UnknownType)
        );
    }

    #[test]
    fn diamond_memoizes_shared_upstream() {
        // gen feeds BOTH inputs of acc (two forward paths). The shared upstream
        // must be cooked exactly once per tick.
        let mut g = Graph::new();
        let generator = g.add_node("test.gen");
        let acc = g.add_node("test.acc");
        g.connect(Edge {
            from: (generator, 0),
            to: (acc, 0),
            delayed: false,
        })
        .unwrap();
        g.connect(Edge {
            from: (generator, 0),
            to: (acc, 1),
            delayed: false,
        })
        .unwrap();
        let o = ops();
        let mut cook = Cook::new();
        let out = cook.cook(&g, &o, acc, 0.0).unwrap();
        assert_eq!(out_scalars(&out[0]), vec![2.0, 4.0, 6.0]); // [1,2,3] + [1,2,3]
        assert_eq!(o.generator.calls.load(Ordering::Relaxed), 1);
    }

    #[test]
    fn output_count_mismatch_errors() {
        // A node that declares 1 output but emits 0 is caught, not leaked as
        // an empty stream downstream.
        let mut g = Graph::new();
        let n = g.add_node("test.bad");
        let o = ops();
        let mut cook = Cook::new();
        assert_eq!(
            cook.cook(&g, &o, n, 0.0).map(|_| ()),
            Err(CookError::OutputCountMismatch {
                node: n,
                expected: 1,
                got: 0
            })
        );
    }

    #[test]
    fn cook_twice_same_tick_is_idempotent_for_sequential_node() {
        // A `pre`-consuming (sequential) node must not recompute on a second
        // cook within the same tick (no advance_tick between).
        let mut g = Graph::new();
        let generator = g.add_node("test.gen");
        let acc = g.add_node("test.acc");
        g.connect(Edge {
            from: (generator, 0),
            to: (acc, 0),
            delayed: false,
        })
        .unwrap();
        g.connect(Edge {
            from: (acc, 0),
            to: (acc, 1),
            delayed: true,
        })
        .unwrap();
        let o = ops();
        let mut cook = Cook::new();
        cook.cook(&g, &o, acc, 0.0).unwrap();
        cook.cook(&g, &o, acc, 0.0).unwrap(); // same tick, no advance_tick
        assert_eq!(o.acc.calls.load(Ordering::Relaxed), 1);
    }

    #[test]
    fn pre_source_without_forward_consumer_is_still_advanced() {
        // Regression for the audit's C1: `s` (a source) feeds `c` ONLY via a
        // `pre` edge and has no forward consumer. `s` must still be cooked each
        // tick (by advance_tick) so its value reaches `c` next tick — otherwise
        // the feedback value is silently lost.
        let mut g = Graph::new();
        let s = g.add_node("test.gen");
        let c = g.add_node("test.delay");
        g.connect(Edge {
            from: (s, 0),
            to: (c, 0),
            delayed: true,
        })
        .unwrap();
        let o = ops();
        let mut cook = Cook::new();

        // tick 0: prev(s) is empty (s not cooked yet) → c emits empty.
        assert!(out_scalars(&cook.cook(&g, &o, c, 0.0).unwrap()[0]).is_empty());
        cook.advance_tick(&g, &o, 0.0).unwrap(); // cooks s, snapshots it
        // tick 1: c reads last tick's s → [1,2,3].
        assert_eq!(
            out_scalars(&cook.cook(&g, &o, c, 0.0).unwrap()[0]),
            vec![1.0, 2.0, 3.0]
        );
    }

    #[test]
    fn cooking_a_missing_node_errors() {
        // Regression for the audit's B2: the UnknownNode path was untested.
        let mut g = Graph::new();
        g.add_node("test.gen");
        let o = ops();
        let mut cook = Cook::new();
        assert_eq!(
            cook.cook(&g, &o, NodeId(999), 0.0).map(|_| ()),
            Err(CookError::UnknownNode)
        );
    }

    // A node that emits its `k` param (default 7) as a 1-element scalar — the
    // probe for per-instance param overrides + their memo invalidation.
    static PARAM_MAN: NodeManifest = NodeManifest {
        id: NodeTypeId::of("test.param_echo"),
        name: "test.param_echo",
        inputs: &[],
        outputs: &[port("out")],
        effect: Effect::Pure,
        clock: Clock::Frame,
        params: &[crate::node::ParamSpec {
            name: "k",
            default: 7.0,
        }],
        lowerings: &[LoweringKind::Cpu],
    };
    struct ParamEcho {
        calls: AtomicU64,
    }
    impl NodeOp for ParamEcho {
        fn manifest(&self) -> &'static NodeManifest {
            &PARAM_MAN
        }
        fn eval(&self, ctx: &mut EvalCtx<'_>) {
            self.calls.fetch_add(1, Ordering::Relaxed);
            ctx.emit(Stream::new(1).with("v", Column::Scalar(vec![ctx.param("k")])));
        }
    }
    struct ParamOps {
        echo: ParamEcho,
    }
    impl OpResolver for ParamOps {
        fn resolve(&self, ty: NodeTypeId) -> Option<&dyn NodeOp> {
            (ty == PARAM_MAN.id).then_some(&self.echo as &dyn NodeOp)
        }
    }

    #[test]
    fn param_reads_default_then_override() {
        let mut g = Graph::new();
        let n = g.add_node("test.param_echo");
        let o = ParamOps {
            echo: ParamEcho {
                calls: AtomicU64::new(0),
            },
        };
        let mut cook = Cook::new();
        // No override → manifest default (7).
        assert_eq!(
            out_scalars(&cook.cook(&g, &o, n, 0.0).unwrap()[0]),
            vec![7.0]
        );
        // Editing the override and re-cooking the SAME Cook must recompute, not
        // return the memoized pre-edit stream (params fold into the fingerprint).
        g.set_param(n, "k", 42.0);
        assert_eq!(
            out_scalars(&cook.cook(&g, &o, n, 0.0).unwrap()[0]),
            vec![42.0]
        );
        assert_eq!(o.echo.calls.load(Ordering::Relaxed), 2); // recomputed
        // A second edit (override → a different override) must also recompute.
        g.set_param(n, "k", 43.0);
        assert_eq!(
            out_scalars(&cook.cook(&g, &o, n, 0.0).unwrap()[0]),
            vec![43.0]
        );
        assert_eq!(o.echo.calls.load(Ordering::Relaxed), 3);
    }

    #[test]
    fn params_fingerprint_is_unambiguous_across_name_value_boundary() {
        // Regression for the audit's framing collision: without length-prefixed
        // names, `{"p": <bytes "emon">, "q": 2.5}` and `{"pemonq": 2.5}` flatten
        // to the same byte stream and collide → memo would return a stale stream.
        let v_p = f32::from_bits(u32::from_le_bytes([b'e', b'm', b'o', b'n']));
        let mut a = BTreeMap::new();
        a.insert("p".to_string(), v_p);
        a.insert("q".to_string(), 2.5_f32);
        let mut b = BTreeMap::new();
        b.insert("pemonq".to_string(), 2.5_f32);
        assert_ne!(params_fingerprint(Some(&a)), params_fingerprint(Some(&b)));
        // None and an empty map both hash to the bare FNV basis (stable "no
        // overrides").
        assert_eq!(
            params_fingerprint(None),
            params_fingerprint(Some(&BTreeMap::new()))
        );
    }

    #[test]
    fn unchanged_param_still_memoizes() {
        let mut g = Graph::new();
        let n = g.add_node("test.param_echo");
        g.set_param(n, "k", 3.0);
        let o = ParamOps {
            echo: ParamEcho {
                calls: AtomicU64::new(0),
            },
        };
        let mut cook = Cook::new();
        cook.cook(&g, &o, n, 0.0).unwrap();
        cook.advance_tick(&g, &o, 0.0).unwrap();
        cook.cook(&g, &o, n, 0.0).unwrap();
        // Param unchanged + combinational → evaluated exactly once.
        assert_eq!(o.echo.calls.load(Ordering::Relaxed), 1);
    }

    #[test]
    #[should_panic(expected = "read undeclared param")]
    fn reading_an_undeclared_param_panics() {
        // A node whose `eval` reads a name not in its manifest is a programmer
        // bug — caught loudly (by its own test), never a silent 0.0.
        static BAD_PARAM_MAN: NodeManifest = NodeManifest {
            id: NodeTypeId::of("test.bad_param"),
            name: "test.bad_param",
            inputs: &[],
            outputs: &[port("out")],
            effect: Effect::Pure,
            clock: Clock::Frame,
            params: &[],
            lowerings: &[LoweringKind::Cpu],
        };
        struct BadParam;
        impl NodeOp for BadParam {
            fn manifest(&self) -> &'static NodeManifest {
                &BAD_PARAM_MAN
            }
            fn eval(&self, ctx: &mut EvalCtx<'_>) {
                let _ = ctx.param("nope");
                ctx.emit(Stream::new(0));
            }
        }
        struct BadOps;
        impl OpResolver for BadOps {
            fn resolve(&self, ty: NodeTypeId) -> Option<&dyn NodeOp> {
                (ty == BAD_PARAM_MAN.id).then_some(&BadParam as &dyn NodeOp)
            }
        }
        let mut g = Graph::new();
        let n = g.add_node("test.bad_param");
        let mut cook = Cook::new();
        let _ = cook.cook(&g, &BadOps, n, 0.0);
    }
}
