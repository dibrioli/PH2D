//! Unit tests for the cook engine (split from `cook.rs` for the HR-18 LOC
//! cap; declared there as a `#[path]` sibling, so `super` is `cook`).
//!
//! Holds the shared test harness — the tiny node ops (`Gen`, `Scale`, `Acc`,
//! `Delay`, `ClockNode`) and their `OpResolver` — used by both this module and
//! the time-scope suite in `cook_scope_tests.rs`.

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

/// Emits its single input verbatim (used with a `pre` edge as a delay line,
/// and — being a plain passthrough — as the remapper node in scope tests).
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

static CLOCK_MAN: NodeManifest = NodeManifest {
    id: NodeTypeId::of("test.clock"),
    name: "test.clock",
    inputs: &[],
    outputs: &[port("out")],
    effect: Effect::Temporal,
    clock: Clock::Frame,
    params: &[],
    lowerings: &[LoweringKind::Cpu],
};

/// Emits the playhead it was pulled at — the probe for time scopes, and a
/// call counter so a test can prove a memo hit rather than assume it.
struct ClockNode {
    calls: AtomicU64,
}
impl NodeOp for ClockNode {
    fn manifest(&self) -> &'static NodeManifest {
        &CLOCK_MAN
    }
    fn eval(&self, ctx: &mut EvalCtx<'_>) {
        self.calls.fetch_add(1, Ordering::Relaxed);
        let t = ctx.playhead() as f32;
        ctx.emit(Stream::new(1).with("v", Column::Scalar(vec![t])));
    }
}

struct Ops {
    generator: Gen,
    scale: Scale,
    acc: Acc,
    bad: Bad,
    delay: Delay,
    clock: ClockNode,
}
impl OpResolver for Ops {
    fn resolve(&self, ty: NodeTypeId) -> Option<&dyn NodeOp> {
        match ty {
            t if t == GEN_MAN.id => Some(&self.generator),
            t if t == SCALE_MAN.id => Some(&self.scale),
            t if t == ACC_MAN.id => Some(&self.acc),
            t if t == BAD_MAN.id => Some(&self.bad),
            t if t == DELAY_MAN.id => Some(&self.delay),
            t if t == CLOCK_MAN.id => Some(&self.clock),
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
        clock: ClockNode {
            calls: AtomicU64::new(0),
        },
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
pub(super) static PARAM_MAN: NodeManifest = NodeManifest {
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
pub(super) struct ParamEcho {
    pub(super) calls: AtomicU64,
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
pub(super) struct ParamOps {
    pub(super) echo: ParamEcho,
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

/// Build the accumulator loop `gen → acc.incr ; acc.out --pre--> acc.feedback`.
/// Frame `T` (after `T` advance_ticks) is `[T+1, 2(T+1), 3(T+1)]` — a stateful
/// recurrence over the tick, the smallest stand-in for a spring/integrator.
fn acc_loop() -> (Graph, NodeId) {
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
    (g, acc)
}

/// The M2.N2 core: a checkpoint taken **before** a frame's cook, restored later,
/// reproduces that exact frame — and re-simulating forward from it matches the
/// original forward trajectory bit-for-bit (deterministic re-sim, the whole
/// point of backwards scrub). This is GGPO save/load/advance.
#[test]
fn checkpoint_then_restore_reproduces_a_past_frame_and_resim_is_bit_exact() {
    let (g, acc) = acc_loop();
    let o = ops();
    let mut cook = Cook::new();

    // Play forward, recording each frame AND the checkpoint that produces it
    // (taken right before that frame's cook — the state the prior advance_tick
    // left). This mirrors what the forward pump records into its ring.
    let mut frames = Vec::new();
    let mut cp = std::collections::BTreeMap::new();
    for t in 0..8u64 {
        cp.insert(t, cook.checkpoint());
        frames.push(out_scalars(&cook.cook(&g, &o, acc, 0.0).unwrap()[0]));
        cook.advance_tick(&g, &o, 0.0).unwrap();
    }
    // Sanity: the recurrence really moved (frame 3 ≠ frame 5).
    assert_eq!(frames[3], vec![4.0, 8.0, 12.0]);
    assert_ne!(frames[3], frames[5]);

    // Scrub back to frame 3: restore its checkpoint → the very next cook IS
    // frame 3, not the future state the cook currently holds.
    cook.restore(&cp[&3]);
    assert_eq!(
        out_scalars(&cook.cook(&g, &o, acc, 0.0).unwrap()[0]),
        frames[3],
        "restore reproduces the exact past frame"
    );
    // Re-sim forward from the restore: frames 4 and 5 come out bit-identical to
    // the original forward run (no drift — the sim is deterministic).
    cook.advance_tick(&g, &o, 0.0).unwrap();
    assert_eq!(
        out_scalars(&cook.cook(&g, &o, acc, 0.0).unwrap()[0]),
        frames[4]
    );
    cook.advance_tick(&g, &o, 0.0).unwrap();
    assert_eq!(
        out_scalars(&cook.cook(&g, &o, acc, 0.0).unwrap()[0]),
        frames[5]
    );
}

/// FALSIFICATION of the whole feature: WITHOUT restore, re-cooking at an earlier
/// playhead reads the CURRENT (future) `pre` state — the exact garbage a naive
/// backwards scrub produces. The checkpoint/restore is what turns this future
/// value into the correct past frame.
#[test]
fn without_restore_a_rewound_cook_reads_the_future_not_the_past() {
    let (g, acc) = acc_loop();
    let o = ops();
    let mut cook = Cook::new();
    let mut frames = Vec::new();
    for _ in 0..6u64 {
        frames.push(out_scalars(&cook.cook(&g, &o, acc, 0.0).unwrap()[0]));
        cook.advance_tick(&g, &o, 0.0).unwrap();
    }
    // "Scrub back to frame 3" the naive way — just cook again, no restore. The
    // pre feedback still holds frame 5's output, so this is frame 6's value,
    // matching NO past frame. (frame 6 would be [7,14,21].)
    let naive = out_scalars(&cook.cook(&g, &o, acc, 0.0).unwrap()[0]);
    assert_eq!(naive, vec![7.0, 14.0, 21.0], "reads the marching future");
    assert!(
        !frames.contains(&naive),
        "the naive rewound cook matches no past frame — the bug checkpoint fixes"
    );
}

/// A checkpoint of a fresh cook is the tick-0 seed: restoring it returns the
/// simulation to its start (empty `pre` → the seed frame), the anchor a far
/// scrub re-sims from. Restoring the seed after playing forward rewinds to
/// frame 0 exactly.
#[test]
fn the_initial_checkpoint_is_the_tick_zero_seed() {
    let (g, acc) = acc_loop();
    let o = ops();
    let mut cook = Cook::new();
    let seed = cook.checkpoint();
    let frame0 = out_scalars(&cook.cook(&g, &o, acc, 0.0).unwrap()[0]);
    for _ in 0..5u64 {
        cook.advance_tick(&g, &o, 0.0).unwrap();
        cook.cook(&g, &o, acc, 0.0).unwrap();
    }
    cook.restore(&seed);
    assert_eq!(
        out_scalars(&cook.cook(&g, &o, acc, 0.0).unwrap()[0]),
        frame0,
        "the seed rewinds to the start frame"
    );
}

#[path = "cook_scope_tests.rs"]
mod scope;
