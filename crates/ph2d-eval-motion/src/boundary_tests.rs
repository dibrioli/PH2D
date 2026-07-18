//! **The plural CPU→GPU boundary** (GPU/M5 slice B) — one tick march, N streams.
//!
//! A plan whose staged region has two uncovered inputs on two different ports
//! leaves TWO boundaries (measured in `ph2d-gpu-cook`'s `boundary_arity`, item
//! (d), the day `motion.look_at` got a kernel). The pump used to take exactly
//! one, so the shell forfeited the GPU for the whole frame.
//!
//! The trap this file exists to disprove is the reason the singular shape was
//! chosen in the first place: *"marching twice would advance the clock twice"*.
//! That describes the CALLER, not the engine — the march and the `pre` feedback
//! are per CALL, and only the consume step differs per target. So the fix is one
//! march with N consumes, and the gate that matters **COUNTS EVALUATIONS** rather
//! than timing anything: the claim is that two boundaries sharing a prefix hit
//! the memo and do not re-simulate it.

use super::*;
use ph2d_nodegraph::attr::Column;
use ph2d_nodegraph::cook::EvalCtx;
use ph2d_nodegraph::effect::Effect;
use ph2d_nodegraph::node::{LoweringKind, NodeManifest, NodeOp, NodeTypeId, ParamSpec, PortSpec};
use ph2d_nodegraph::port::{Clock, Dim, Domain, PortType};
use std::cell::Cell;

const INST_VEC2: PortType = PortType::new(Domain::Instances, Dim::Vec2, Clock::Frame);

/// The SHARED prefix. Counts its own evaluations, which is the whole point: if
/// cooking two boundaries re-cooks this, the count is 2 and the memo claim is
/// false. Its output carries the PLAYHEAD, so a stream also says which tick it
/// was cooked at.
const SRC_MAN: NodeManifest = NodeManifest {
    id: NodeTypeId::of("motion.test.bsrc"),
    name: "motion.test.bsrc",
    inputs: &[],
    outputs: &[PortSpec {
        name: "out",
        ty: INST_VEC2,
    }],
    effect: Effect::Pure,
    clock: Clock::Frame,
    params: &[],
    lowerings: &[LoweringKind::Cpu],
};

/// A branch off the shared prefix: copies `P` through and stamps a `mark` column
/// with its own param, so the two boundary streams are TELLABLE APART. Without
/// the mark, handing the same stream back twice would pass.
const TAP_MAN: NodeManifest = NodeManifest {
    id: NodeTypeId::of("motion.test.btap"),
    name: "motion.test.btap",
    inputs: &[PortSpec {
        name: "in",
        ty: INST_VEC2,
    }],
    outputs: &[PortSpec {
        name: "out",
        ty: INST_VEC2,
    }],
    effect: Effect::Pure,
    clock: Clock::Frame,
    params: &[ParamSpec {
        name: "mark",
        default: 0.0,
    }],
    lowerings: &[LoweringKind::Cpu],
};

thread_local! {
    static SRC_EVALS: Cell<u32> = const { Cell::new(0) };
}

struct Src;
impl NodeOp for Src {
    fn manifest(&self) -> &'static NodeManifest {
        &SRC_MAN
    }
    fn eval(&self, ctx: &mut EvalCtx<'_>) {
        SRC_EVALS.with(|c| c.set(c.get() + 1));
        // `P.x` IS the playhead, so a stream says which tick it came from. Without
        // that, a scrub gate can only check that streams are present — which a
        // scrub returning the marching-future state would also satisfy.
        let t = ctx.playhead() as f32;
        ctx.emit(Stream::new(2).with("P", Column::Vec2(vec![[t, 0.0], [t, 1.0]])));
    }
}

struct Tap;
impl NodeOp for Tap {
    fn manifest(&self) -> &'static NodeManifest {
        &TAP_MAN
    }
    fn eval(&self, ctx: &mut EvalCtx<'_>) {
        let mark = ctx.param("mark");
        let input = ctx.input(0);
        let n = input.count();
        let p = match input.get("P") {
            Some(Column::Vec2(v)) => v.clone(),
            _ => vec![[0.0, 0.0]; n],
        };
        ctx.emit(
            Stream::new(n)
                .with("P", Column::Vec2(p))
                .with("mark", Column::Scalar(vec![mark; n])),
        );
    }
}

struct Ops;
impl OpResolver for Ops {
    fn resolve(&self, ty: NodeTypeId) -> Option<&dyn NodeOp> {
        match ty {
            t if t == SRC_MAN.id => Some(&Src as &dyn NodeOp),
            t if t == TAP_MAN.id => Some(&Tap as &dyn NodeOp),
            _ => None,
        }
    }
}

/// `src → tapA` and `src → tapB`: one shared prefix, two boundaries.
fn two_boundary_graph() -> (Graph, NodeId, NodeId) {
    let mut g = Graph::new();
    let src = g.add_node("motion.test.bsrc");
    let a = g.add_node("motion.test.btap");
    let b = g.add_node("motion.test.btap");
    g.set_param(a, "mark", 1.0);
    g.set_param(b, "mark", 2.0);
    for t in [a, b] {
        g.connect(ph2d_nodegraph::graph::Edge {
            from: (src, 0),
            to: (t, 0),
            delayed: false,
        })
        .unwrap();
    }
    (g, a, b)
}

fn mark_of(s: &Stream) -> f32 {
    match s.get("mark") {
        Some(Column::Scalar(v)) => v[0],
        _ => panic!("the tap emits a mark"),
    }
}

/// **One march, two streams, each the right one.**
#[test]
fn the_pump_hands_over_every_boundary_in_one_march() {
    let (g, a, b) = two_boundary_graph();
    let mut pump = MotionCookPump::new();
    SRC_EVALS.with(|c| c.set(0));

    assert!(pump.advance_or_scrub_to_nodes_scoped(
        &g,
        &Ops,
        &[a, b],
        0,
        |t| t as f64 * 0.016,
        &TimeScopes::default(),
    ));

    let got = pump.boundary_streams();
    assert_eq!(got.len(), 2, "both boundaries handed over");
    // Tellable apart: handing the same stream back twice would pass a test that
    // only counted them.
    let marks: Vec<f32> = got.iter().map(|(_, s)| mark_of(s)).collect();
    assert_eq!(marks, vec![1.0, 2.0], "each boundary got ITS own stream");
    assert_eq!(
        got.iter().map(|(n, _)| *n).collect::<Vec<_>>(),
        vec![a, b],
        "and each stream is labelled with the node it came from"
    );
}

/// **The shared prefix is cooked ONCE.** The claim that made a plural pump
/// cheap — two `cook_scoped` calls at the same playhead hit the same memo
/// fingerprint — was answered by READING `cook.rs` and never measured, because
/// no two-boundary graph existed to measure. Now one does.
///
/// This counts evaluations rather than timing: a timing bar would pass on a fast
/// machine with the prefix re-simulated.
#[test]
fn two_boundaries_sharing_a_prefix_do_not_re_cook_it() {
    let (g, a, b) = two_boundary_graph();
    let mut pump = MotionCookPump::new();
    SRC_EVALS.with(|c| c.set(0));

    pump.advance_or_scrub_to_nodes_scoped(
        &g,
        &Ops,
        &[a, b],
        0,
        |t| t as f64 * 0.016,
        &TimeScopes::default(),
    );

    assert_eq!(
        SRC_EVALS.with(Cell::get),
        1,
        "the shared prefix must hit the memo on the second boundary — if this is \
         2, every extra boundary re-runs the whole upstream chain, and for a \
         sequential prefix that is a second simulation of one state"
    );
}

/// **A node consumed on two ports appears twice in `plan.boundaries`** (the plan
/// pushes per port), and the pump must not hand it over twice: the GPU side keys
/// its uploads by node, so a duplicate is at best a wasted upload and at worst a
/// `want`/`got` mismatch that fails the whole cook.
#[test]
fn a_boundary_named_twice_is_handed_over_once() {
    let (g, a, _) = two_boundary_graph();
    let mut pump = MotionCookPump::new();

    pump.advance_or_scrub_to_nodes_scoped(
        &g,
        &Ops,
        &[a, a],
        0,
        |t| t as f64 * 0.016,
        &TimeScopes::default(),
    );

    assert_eq!(
        pump.boundary_streams().len(),
        1,
        "the same node named twice is ONE hand-off"
    );
}

/// **The clock advances once per march, not once per boundary.** This is the
/// exact fear that kept the pump singular; it is a property of the CALL, and a
/// gate is cheaper than trusting the reading.
#[test]
fn n_boundaries_advance_the_clock_once() {
    let (g, a, b) = two_boundary_graph();
    let mut pump = MotionCookPump::new();

    for tick in 0..3u64 {
        pump.advance_or_scrub_to_nodes_scoped(
            &g,
            &Ops,
            &[a, b],
            tick,
            |t| t as f64 * 0.016,
            &TimeScopes::default(),
        );
    }
    assert_eq!(
        pump.last_cooked_tick(),
        Some(2),
        "three marches over two boundaries land on tick 2, not tick 5"
    );
}

/// **A failing boundary does not silence the others.** The route falls back
/// cleanly only if the streams it DID get are still there; dropping all of them
/// because one node failed would turn a mid-edit type error into a black frame.
#[test]
fn one_failing_boundary_leaves_the_others_intact() {
    let (mut g, a, _) = two_boundary_graph();
    // A node whose type nothing resolves — the mid-edit shape.
    let bad = g.add_node("motion.test.nonexistent");
    let mut pump = MotionCookPump::new();

    pump.advance_or_scrub_to_nodes_scoped(
        &g,
        &Ops,
        &[a, bad],
        0,
        |t| t as f64 * 0.016,
        &TimeScopes::default(),
    );

    let got = pump.boundary_streams();
    assert_eq!(got.len(), 1, "the healthy boundary still handed over");
    assert_eq!(got[0].0, a);
    assert!(pump.last_error().is_some(), "and the error is kept");
}

/// **(b) Disjoint prefixes** — the case the seam map flagged as unverified.
///
/// Two boundaries that share NOTHING must each get their own prefix cooked, once
/// each. The memo makes the shared case cheap; it must not make the disjoint case
/// WRONG by handing the second boundary the first one's answer, and it must not
/// make it expensive by cooking either twice.
#[test]
fn two_boundaries_with_disjoint_prefixes_each_cook_once() {
    let mut g = Graph::new();
    // Two independent `src → tap` chains: no shared node at all.
    let mut ends = Vec::new();
    for (i, mark) in [1.0f32, 2.0].iter().enumerate() {
        let src = g.add_node("motion.test.bsrc");
        let tap = g.add_node("motion.test.btap");
        g.set_param(tap, "mark", *mark);
        g.connect(ph2d_nodegraph::graph::Edge {
            from: (src, 0),
            to: (tap, 0),
            delayed: false,
        })
        .unwrap();
        assert_eq!(i, ends.len());
        ends.push(tap);
    }
    let mut pump = MotionCookPump::new();
    SRC_EVALS.with(|c| c.set(0));

    pump.advance_or_scrub_to_nodes_scoped(
        &g,
        &Ops,
        &ends,
        0,
        |t| t as f64 * 0.016,
        &TimeScopes::default(),
    );

    let got = pump.boundary_streams();
    assert_eq!(got.len(), 2);
    assert_eq!(
        got.iter().map(|(_, s)| mark_of(s)).collect::<Vec<_>>(),
        vec![1.0, 2.0],
        "each boundary carries ITS OWN chain's answer — the memo must key on the \
         node, not hand the second boundary the first one's stream"
    );
    assert_eq!(
        SRC_EVALS.with(Cell::get),
        2,
        "two disjoint sources, one evaluation each — not 1 (a collapsed memo key) \
         and not 4 (the prefix re-cooked per boundary)"
    );
}

/// **(c) A backwards scrub with N boundaries** — the other flagged unknown. The
/// ring belongs to the PUMP, so restoring it has to serve every boundary of the
/// march, not the last one asked for.
///
/// The source stamps the playhead into `P.x`, so the stream says which tick it
/// came from and a scrub that quietly returned the marching-future state is
/// visible — checking only that two streams came back would not see it.
#[test]
fn a_backwards_scrub_rewinds_every_boundary() {
    let (g, a, b) = two_boundary_graph();
    let mut pump = MotionCookPump::new();
    let clock = |t: u64| t as f64 * 0.016;

    for tick in 0..6u64 {
        pump.advance_or_scrub_to_nodes_scoped(
            &g,
            &Ops,
            &[a, b],
            tick,
            clock,
            &TimeScopes::default(),
        );
    }
    assert_eq!(pump.last_cooked_tick(), Some(5));

    // Back to tick 2 — behind the playhead.
    pump.advance_or_scrub_to_nodes_scoped(&g, &Ops, &[a, b], 2, clock, &TimeScopes::default());
    let got = pump.boundary_streams();
    assert_eq!(got.len(), 2, "a scrub hands over every boundary too");
    assert_eq!(
        got.iter().map(|(_, s)| mark_of(s)).collect::<Vec<_>>(),
        vec![1.0, 2.0],
        "and each is still its own"
    );
    assert_eq!(
        pump.last_cooked_tick(),
        Some(2),
        "the pump's clock landed on the scrubbed tick"
    );
    // THE assertion: the VALUES are tick 2's, not tick 5's.
    for (node, stream) in got {
        let p = match stream.get("P") {
            Some(Column::Vec2(v)) => v[0][0],
            _ => panic!("the source emits P"),
        };
        assert!(
            (p - clock(2) as f32).abs() < 1e-6,
            "{node:?} handed back the state of playhead {p}, not the scrubbed \
             tick 2 ({}) — a scrub that returns the marching future is the \
             classic determinism trap",
            clock(2)
        );
    }
}
