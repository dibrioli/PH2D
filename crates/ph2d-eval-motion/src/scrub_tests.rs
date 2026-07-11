//! Pump-level backwards-scrub tests (M2.N2/N3), split from `lib.rs` for the
//! HR-18 700-LOC cap. Drives the real [`MotionCookPump`] with a sequential
//! counter node whose state marches with the tick, and proves a backwards scrub
//! restores the exact past frame — the gold-standard save/load/advance the
//! cook-level tests prove bit-exact in `ph2d-nodegraph`.

use crate::*;
use ph2d_nodegraph::attr::{Column, Stream};
use ph2d_nodegraph::cook::EvalCtx;
use ph2d_nodegraph::graph::{Edge, Graph};
use ph2d_nodegraph::node::{LoweringKind, NodeManifest, NodeOp, NodeTypeId, PortSpec};
use ph2d_nodegraph::port::{Clock, Dim, Domain, PortType};

const INST: PortType = PortType::new(Domain::Instances, Dim::Vec2, Clock::Frame);

/// A sequential counter: `P.x` starts at 0 and rises by 1 each tick, carried on
/// its own `pre` self-loop (`out --pre--> state`). Frame `T` has `P.x == T` — a
/// minimal stand-in for a spring/integrator whose trajectory a scrub must
/// reproduce. Reads the previous frame off the `state` input (Empty at tick 0 →
/// seed 0).
static COUNT_MAN: NodeManifest = NodeManifest {
    id: NodeTypeId::of("motion.test.counter"),
    name: "motion.test.counter",
    inputs: &[PortSpec {
        name: "state",
        ty: INST,
    }],
    outputs: &[PortSpec {
        name: "out",
        ty: INST,
    }],
    effect: ph2d_nodegraph::effect::Effect::Pure,
    clock: Clock::Frame,
    params: &[],
    lowerings: &[LoweringKind::Cpu],
};
struct Counter;
impl NodeOp for Counter {
    fn manifest(&self) -> &'static NodeManifest {
        &COUNT_MAN
    }
    fn eval(&self, ctx: &mut EvalCtx<'_>) {
        let prev = match ctx.input(0).get("P") {
            Some(Column::Vec2(v)) if !v.is_empty() => v[0][0],
            _ => -1.0, // Empty pre (tick 0) → seed so the first frame is 0.
        };
        ctx.emit(Stream::new(1).with("P", Column::Vec2(vec![[prev + 1.0, 0.0]])));
    }
}
struct Ops;
impl OpResolver for Ops {
    fn resolve(&self, ty: NodeTypeId) -> Option<&dyn NodeOp> {
        (ty == COUNT_MAN.id).then_some(&Counter as &dyn NodeOp)
    }
}

fn counter_graph() -> (Graph, NodeId) {
    let mut g = Graph::new();
    let c = g.add_node("motion.test.counter");
    g.connect(Edge {
        from: (c, 0),
        to: (c, 0),
        delayed: true,
    })
    .unwrap();
    (g, c)
}

const UV: [f32; 4] = [0.0, 0.0, 1.0, 1.0];
const SIZE: [f32; 2] = [1.0, 1.0];
const DT: f64 = 1.0 / 60.0;

fn frame_x(pump: &MotionCookPump) -> f32 {
    pump.instances[0].world_pos[0]
}

/// Play forward, then scrub BACKWARDS: the restored frame is the exact past
/// state (`P.x == target`), not the marching future — the M2 gate ("spring com
/// scrub para trás correto"). And a forward re-sim off the scrub matches
/// playback bit-for-bit.
#[test]
fn a_backwards_scrub_restores_the_exact_past_frame() {
    let (g, c) = counter_graph();
    let mut pump = MotionCookPump::new();
    let scopes = TimeScopes::new();

    // Forward playback to tick 20 — each frame's P.x equals its tick.
    for t in 0..=20u64 {
        pump.pump_scoped(&g, &Ops, &[c], t, t as f64 * DT, UV, SIZE, &scopes);
        assert_eq!(frame_x(&pump), t as f32, "forward frame {t}");
    }

    // Scrub back to tick 5: the counter must read 5, not 20.
    pump.scrub_to_scoped(&g, &Ops, &[c], 5, |t| t as f64 * DT, UV, SIZE, &scopes);
    assert_eq!(frame_x(&pump), 5.0, "restored the exact past frame");

    // Scrub back further, to tick 0 (the seed), then to a mid tick.
    pump.scrub_to_scoped(&g, &Ops, &[c], 0, |t| t as f64 * DT, UV, SIZE, &scopes);
    assert_eq!(frame_x(&pump), 0.0, "seed frame");
    pump.scrub_to_scoped(&g, &Ops, &[c], 12, |t| t as f64 * DT, UV, SIZE, &scopes);
    assert_eq!(frame_x(&pump), 12.0);

    // Resume forward playback from the scrub point: bit-exact continuation.
    pump.pump_scoped(&g, &Ops, &[c], 13, 13.0 * DT, UV, SIZE, &scopes);
    assert_eq!(frame_x(&pump), 13.0, "forward re-sim off the scrub");
}

/// FALSIFICATION: the SAME rewind via a plain forward `pump_scoped` reads the
/// future `pre` state — the bug the scrub path exists to fix. (Cooking tick 5
/// after playing to 20 gives 21, because the pre feedback is still tick 20's.)
#[test]
fn a_plain_pump_at_a_past_tick_reads_the_future() {
    let (g, c) = counter_graph();
    let mut pump = MotionCookPump::new();
    let scopes = TimeScopes::new();
    for t in 0..=20u64 {
        pump.pump_scoped(&g, &Ops, &[c], t, t as f64 * DT, UV, SIZE, &scopes);
    }
    // Naive "scrub" to 5 with the forward pump: the pre state marches on.
    pump.pump_scoped(&g, &Ops, &[c], 5, 5.0 * DT, UV, SIZE, &scopes);
    assert_eq!(frame_x(&pump), 21.0, "plain pump reads the marching future");
    assert_ne!(
        frame_x(&pump),
        5.0,
        "which is exactly the bug scrub_to fixes"
    );
}

/// A graph edit invalidates the scrub cache: after `mark_dirty` the ring is
/// cleared, so a scrub re-sims from the tick-0 seed under the current graph
/// (Blender/Houdini "edit invalidates the cache") — still correct, just not
/// cached.
#[test]
fn an_edit_clears_the_ring_and_the_scrub_resims_from_the_seed() {
    let (g, c) = counter_graph();
    let mut pump = MotionCookPump::new();
    let scopes = TimeScopes::new();
    for t in 0..=10u64 {
        pump.pump_scoped(&g, &Ops, &[c], t, t as f64 * DT, UV, SIZE, &scopes);
    }
    pump.mark_dirty(); // simulate a graph edit
    // The ring is empty now → scrub to 6 re-sims 0..6 from the seed. Same graph,
    // so the answer is still exactly 6 (determinism).
    pump.scrub_to_scoped(&g, &Ops, &[c], 6, |t| t as f64 * DT, UV, SIZE, &scopes);
    assert_eq!(
        frame_x(&pump),
        6.0,
        "re-sim from seed lands the right frame"
    );
}
