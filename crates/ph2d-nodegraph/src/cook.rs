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
//! Scope: drives the **pull / presentation** side (motion, shader, sound).
//! `Stateful` (gameplay) nodes are driven by the push evaluator
//! (`ph2d-script`), never here — that separation is the membrane (ADR-0030).

use crate::attr::Stream;
use crate::effect::Effect;
use crate::graph::{Graph, NodeId};
use crate::node::{NodeOp, NodeTypeId};
use std::collections::BTreeMap;

/// Resolves a node type id to its operation impl. Implemented by the node
/// registry (W1.T3); kept as a trait so the cook engine is decoupled from it.
pub trait OpResolver {
    fn resolve(&self, ty: NodeTypeId) -> Option<&dyn NodeOp>;
}

/// Per-eval context handed to a node. A node sees **only** this — its typed
/// inputs and the playhead — never the graph. FBP black box (ADR-0031).
pub struct EvalCtx<'a> {
    inputs: &'a [Stream],
    playhead: f64,
    outputs: Vec<Stream>,
}

impl<'a> EvalCtx<'a> {
    /// The cooked stream on input `port` (empty if unconnected; for a `pre`
    /// port, the previous tick's value).
    pub fn input(&self, port: usize) -> &Stream {
        &self.inputs[port]
    }

    pub fn input_count(&self) -> usize {
        self.inputs.len()
    }

    /// Current clock time; meaningful for `Temporal` nodes.
    pub fn playhead(&self) -> f64 {
        self.playhead
    }

    /// Emit the next output port's stream. Call once per output port, in order.
    pub fn emit(&mut self, stream: Stream) {
        self.outputs.push(stream);
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum CookError {
    /// Target (or an upstream source) is not a node in this graph.
    UnknownNode,
    /// A node's type is not registered with the resolver.
    UnknownType,
}

/// What a node's reuse decision depends on: revisions of its forward inputs,
/// the playhead (if `Temporal`), and the tick (if it consumes a `pre` edge,
/// i.e. is sequential and must advance every tick).
#[derive(Clone, Default, PartialEq)]
struct Fingerprint {
    input_revs: Vec<u64>,
    playhead: Option<f64>,
    tick: Option<u64>,
}

struct Cached {
    outputs: Vec<Stream>,
    revision: u64,
    fingerprint: Fingerprint,
}

/// Incremental cook engine. Holds the memo cache and the previous-tick snapshot
/// across cooks; reusing the same `Cook` across frames is what makes
/// re-evaluation cheap and `pre` feedback work.
#[derive(Default)]
pub struct Cook {
    cache: BTreeMap<NodeId, Cached>,
    prev_outputs: BTreeMap<NodeId, Vec<Stream>>,
    tick: u64,
}

impl Cook {
    pub fn new() -> Self {
        Self::default()
    }

    /// Advance one tick: snapshot this tick's outputs as the previous tick (so
    /// `pre` edges read them next tick). Call once per frame, between cooks.
    pub fn advance_tick(&mut self) {
        self.prev_outputs = self
            .cache
            .iter()
            .map(|(id, c)| (*id, c.outputs.clone()))
            .collect();
        self.tick += 1;
    }

    /// Cook `target`'s outputs at `playhead`, pulling upstream on demand and
    /// reusing memoized results whose inputs are unchanged.
    pub fn cook(
        &mut self,
        graph: &Graph,
        ops: &dyn OpResolver,
        target: NodeId,
        playhead: f64,
    ) -> Result<&[Stream], CookError> {
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
        let mut input_streams: Vec<Stream> = Vec::with_capacity(manifest.inputs.len());
        let mut input_revs: Vec<u64> = Vec::new();
        let mut consumes_pre = false;
        for port in 0..manifest.inputs.len() {
            match graph.input_edge(node, port) {
                Some((src, src_port, false)) => {
                    let rev = self.cook_node(graph, ops, src, playhead)?;
                    input_revs.push(rev);
                    input_streams.push(self.cur_output(src, src_port));
                }
                Some((src, src_port, true)) => {
                    consumes_pre = true;
                    input_streams.push(self.prev_output(src, src_port));
                }
                None => input_streams.push(Stream::default()),
            }
        }

        // 2. Reuse decision (memoization). A `pre`-consuming (sequential) node
        //    keys on the tick, so it recomputes once per tick and drives its
        //    feedback loop; a purely combinational node stays memoized.
        let fingerprint = Fingerprint {
            input_revs,
            playhead: (manifest.effect == Effect::Temporal).then_some(playhead),
            tick: consumes_pre.then_some(self.tick),
        };
        if let Some(c) = self.cache.get(&node)
            && c.fingerprint == fingerprint
        {
            return Ok(c.revision);
        }

        // 3. Recompute.
        let mut ctx = EvalCtx { inputs: &input_streams, playhead, outputs: Vec::new() };
        op.eval(&mut ctx);
        let revision = self.next_revision();
        self.cache.insert(node, Cached { outputs: ctx.outputs, revision, fingerprint });
        Ok(revision)
    }

    fn next_revision(&mut self) -> u64 {
        // Monotonic across the engine's lifetime; downstream nodes detect a
        // changed input by a changed revision.
        let last = self.cache.values().map(|c| c.revision).max().unwrap_or(0);
        last + 1
    }

    fn cur_output(&self, node: NodeId, port: usize) -> Stream {
        self.cache
            .get(&node)
            .and_then(|c| c.outputs.get(port))
            .cloned()
            .unwrap_or_default()
    }

    fn prev_output(&self, node: NodeId, port: usize) -> Stream {
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
    use crate::node::{NodeManifest, PortSpec};
    use crate::port::{Clock, Dim, Domain, PortType};
    use std::sync::atomic::{AtomicU64, Ordering};

    const SCALAR_FRAME: PortType = PortType::new(Domain::Instances, Dim::Scalar, Clock::Frame);
    const fn port(name: &'static str) -> PortSpec {
        PortSpec { name, ty: SCALAR_FRAME }
    }

    static GEN_MAN: NodeManifest = NodeManifest {
        id: NodeTypeId::of("test.gen"),
        name: "test.gen",
        inputs: &[],
        outputs: &[port("out")],
        effect: Effect::Pure,
        clock: Clock::Frame,
    };
    static SCALE_MAN: NodeManifest = NodeManifest {
        id: NodeTypeId::of("test.scale"),
        name: "test.scale",
        inputs: &[port("in")],
        outputs: &[port("out")],
        effect: Effect::Pure,
        clock: Clock::Frame,
    };
    static ACC_MAN: NodeManifest = NodeManifest {
        id: NodeTypeId::of("test.acc"),
        name: "test.acc",
        inputs: &[port("incr"), port("feedback")],
        outputs: &[port("out")],
        effect: Effect::Pure,
        clock: Clock::Frame,
    };

    fn scalars(s: &Stream) -> Vec<f32> {
        match s.get("v") {
            Some(Column::Scalar(v)) => v.clone(),
            _ => vec![],
        }
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

    /// out = incr + feedback(pre). Classic accumulator over the clock.
    struct Acc;
    impl NodeOp for Acc {
        fn manifest(&self) -> &'static NodeManifest {
            &ACC_MAN
        }
        fn eval(&self, ctx: &mut EvalCtx<'_>) {
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

    struct Ops {
        generator: Gen,
        scale: Scale,
        acc: Acc,
    }
    impl OpResolver for Ops {
        fn resolve(&self, ty: NodeTypeId) -> Option<&dyn NodeOp> {
            match ty {
                t if t == GEN_MAN.id => Some(&self.generator),
                t if t == SCALE_MAN.id => Some(&self.scale),
                t if t == ACC_MAN.id => Some(&self.acc),
                _ => None,
            }
        }
    }

    fn ops() -> Ops {
        Ops { generator: Gen { calls: AtomicU64::new(0) }, scale: Scale, acc: Acc }
    }

    #[test]
    fn cooks_a_chain_end_to_end() {
        let mut g = Graph::new();
        let generator = g.add_node("test.gen");
        let scale = g.add_node("test.scale");
        g.connect(Edge { from: (generator, 0), to: (scale, 0), delayed: false }).unwrap();
        let o = ops();
        let mut cook = Cook::new();
        let out = cook.cook(&g, &o, scale, 0.0).unwrap();
        assert_eq!(scalars(&out[0]), vec![2.0, 4.0, 6.0]);
    }

    #[test]
    fn memoizes_unchanged_upstream_across_cooks() {
        let mut g = Graph::new();
        let generator = g.add_node("test.gen");
        let scale = g.add_node("test.scale");
        g.connect(Edge { from: (generator, 0), to: (scale, 0), delayed: false }).unwrap();
        let o = ops();
        let mut cook = Cook::new();
        cook.cook(&g, &o, scale, 0.0).unwrap();
        cook.advance_tick();
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
        g.connect(Edge { from: (generator, 0), to: (acc, 0), delayed: false }).unwrap();
        g.connect(Edge { from: (acc, 0), to: (acc, 1), delayed: true }).unwrap();
        let o = ops();
        let mut cook = Cook::new();

        // tick 0: feedback empty → gen
        assert_eq!(scalars(&cook.cook(&g, &o, acc, 0.0).unwrap()[0]), vec![1.0, 2.0, 3.0]);
        cook.advance_tick();
        // tick 1: gen + prev(=[1,2,3])
        assert_eq!(scalars(&cook.cook(&g, &o, acc, 0.0).unwrap()[0]), vec![2.0, 4.0, 6.0]);
        cook.advance_tick();
        // tick 2: gen + prev(=[2,4,6])
        assert_eq!(scalars(&cook.cook(&g, &o, acc, 0.0).unwrap()[0]), vec![3.0, 6.0, 9.0]);

        // gen is combinational/unchanged → evaluated once despite 3 ticks.
        assert_eq!(o.generator.calls.load(Ordering::Relaxed), 1);
    }

    #[test]
    fn unknown_type_errors() {
        let mut g = Graph::new();
        let n = g.add_node("test.nonexistent");
        let o = ops();
        let mut cook = Cook::new();
        assert_eq!(cook.cook(&g, &o, n, 0.0), Err(CookError::UnknownType));
    }
}
