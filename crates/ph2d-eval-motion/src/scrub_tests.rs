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

/// **The O(1) loop-wrap gate** — the 2026-07-20 audit's measurement, FLIPPED
/// (ADR-0137). The old ring recorded only strictly-forward ticks and evicted
/// the oldest, so every wrap of a loop re-simmed the whole history, forever
/// (measured here: lap 1 = 101 evals, lap 2 = 101 AGAIN). With backfill the
/// first lap's re-sim rebuilds coverage as it goes, and lap 2 anchors on it:
/// the wrap costs the ONE eval of the target frame. This ran `#[ignore]`d as a
/// measurement while the pathology was named-not-fixed; it is a plain gate now.
#[test]
fn a_loop_wrap_anchors_on_the_previous_laps_backfill() {
    use std::sync::atomic::{AtomicUsize, Ordering};
    static EVALS: AtomicUsize = AtomicUsize::new(0);

    static MAN: NodeManifest = NodeManifest {
        id: NodeTypeId::of("motion.test.counting"),
        name: "motion.test.counting",
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
    struct Counting;
    impl NodeOp for Counting {
        fn manifest(&self) -> &'static NodeManifest {
            &MAN
        }
        fn eval(&self, ctx: &mut EvalCtx<'_>) {
            EVALS.fetch_add(1, Ordering::Relaxed);
            let prev = match ctx.input(0).get("P") {
                Some(Column::Vec2(v)) if !v.is_empty() => v[0][0],
                _ => -1.0,
            };
            ctx.emit(Stream::new(1).with("P", Column::Vec2(vec![[prev + 1.0, 0.0]])));
        }
    }
    struct COps;
    impl OpResolver for COps {
        fn resolve(&self, ty: NodeTypeId) -> Option<&dyn NodeOp> {
            (ty == MAN.id).then_some(&Counting as &dyn NodeOp)
        }
    }

    let mut g = Graph::new();
    let c = g.add_node("motion.test.counting");
    g.connect(Edge {
        from: (c, 0),
        to: (c, 0),
        delayed: true,
    })
    .unwrap();
    let scopes = TimeScopes::new();
    let mut pump = MotionCookPump::new();
    let ph = |t: u64| t as f64 * DT;

    // A "loop" of [100, 400] — longer than RECENT_CAPACITY's horizon once the
    // playhead has visited the tail. Play the first pass.
    for t in 0..=400u64 {
        pump.advance_or_scrub_scoped(&g, &COps, &[c], t, ph, UV, SIZE, &scopes);
    }
    let wrap = |pump: &mut MotionCookPump, label: &str| {
        let before = EVALS.load(Ordering::Relaxed);
        pump.advance_or_scrub_scoped(&g, &COps, &[c], 100, ph, UV, SIZE, &scopes);
        let cost = EVALS.load(Ordering::Relaxed) - before;
        assert_eq!(frame_x(pump), 100.0, "{label}: the frame itself is right");
        eprintln!("{label}: wrap to tick 100 cost {cost} evals");
        cost
    };

    let first = wrap(&mut pump, "wrap 1 (the lap that backfills)");
    // The FIRST wrap may legitimately re-sim (the history was never anchored —
    // playing 0..=400 forward recorded it, so with backfill even lap 1 is
    // cheap; the bound below only assumes the ring KEPT tick-100 coverage).
    for t in 101..=400u64 {
        pump.advance_or_scrub_scoped(&g, &COps, &[c], t, ph, UV, SIZE, &scopes);
    }
    let second = wrap(&mut pump, "wrap 2 (anchored by lap 1's coverage)");
    eprintln!("loop-wrap: first {first} evals, second {second} evals");
    // The O(1) claim, on BOTH laps: forward play 0..=400 already recorded every
    // tick (backfill admits them all under the entry backstop), so each wrap
    // anchors AT tick 100 and pays exactly the target frame's eval. `> 90` was
    // the audit's starvation measurement; a regression to eviction-by-oldest or
    // forward-only recording sends this straight back to ~101.
    assert!(
        first <= 2 && second <= 2,
        "a wrap must anchor on recorded coverage, not re-sim the history \
         (first {first}, second {second} evals — the pre-ADR-0137 ring paid 101)"
    );

    // PHASE 2 — the EVICTION regime (the original disease needed it: capacity
    // 300 under a 401-tick history is what starved the old ring, and a fixture
    // whose entries all fit would stay green with backfill deleted). Squeeze
    // the budget so the march MUST evict, re-run the laps, and bound the wrap
    // by the ring's RESOLUTION over the span — never by the loop's position.
    let mut pump = MotionCookPump::new();
    // The counting state is one Vec2 element (~8 B/checkpoint): 200 B ≈ 25
    // anchors over 401 ticks ⇒ eviction bites hard (half protected-recent,
    // half thinned history ⇒ history gap ≈ 33).
    pump.set_ring_budget(200);
    for t in 0..=400u64 {
        pump.advance_or_scrub_scoped(&g, &COps, &[c], t, ph, UV, SIZE, &scopes);
    }
    let lap1 = wrap(&mut pump, "squeezed wrap 1");
    for t in 101..=400u64 {
        pump.advance_or_scrub_scoped(&g, &COps, &[c], t, ph, UV, SIZE, &scopes);
    }
    let lap2 = wrap(&mut pump, "squeezed wrap 2");
    // ~12 anchors thinned over [0, 400] ⇒ gap ≈ 33; generous slack to 60 so
    // the bound is about the SHAPE (resolution vs position), not the constant.
    // The pre-ADR-0137 ring paid the loop's POSITION here: 101 evals, forever.
    assert!(
        lap1 <= 60 && lap2 <= 60,
        "under eviction the wrap is bounded by ring resolution, not loop \
         position (laps {lap1}/{lap2} evals; starved = 101)"
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
