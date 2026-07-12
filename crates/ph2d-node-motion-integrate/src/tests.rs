//! Tests for `motion.integrate` — split into a sibling module for the workspace LOC cap
//! (the node's own logic is 250 lines; its guards are twice that, and rightly so: an
//! integrator that quietly drifts is a simulation that quietly lies).

use super::*;
use ph2d_nodegraph::cook::{Cook, OpResolver};
use ph2d_nodegraph::graph::{Edge, Graph, NodeId};

// A 2-instance source whose P slides in X with the playhead (playhead
// itself, so upstream liveness is observable), emitting a `falloff` too.
static SRC_MAN: NodeManifest = NodeManifest {
    id: NodeTypeId::of("motion.integrate.test.src"),
    name: "motion.integrate.test.src",
    inputs: &[],
    outputs: &[PortSpec {
        name: "out",
        ty: INST_VEC2,
    }],
    effect: Effect::Temporal,
    clock: Clock::Frame,
    params: &[],
    lowerings: &[LoweringKind::Cpu],
};
struct Src;
impl NodeOp for Src {
    fn manifest(&self) -> &'static NodeManifest {
        &SRC_MAN
    }
    fn eval(&self, ctx: &mut EvalCtx<'_>) {
        let t = ctx.playhead() as f32;
        ctx.emit(
            Stream::new(2)
                .with("P", Column::Vec2(vec![[t, 0.0], [10.0 + t, 0.0]]))
                .with("falloff", Column::Scalar(vec![1.0, 0.5])),
        );
    }
}

// A constant in-loop "force": accel += (1, 0) — stands in for the force
// chain so the integrator is exercised exactly as wired in the app.
static FORCE_MAN: NodeManifest = NodeManifest {
    id: NodeTypeId::of("motion.integrate.test.force"),
    name: "motion.integrate.test.force",
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
    params: &[],
    lowerings: &[LoweringKind::Cpu],
};
struct ConstForce;
impl NodeOp for ConstForce {
    fn manifest(&self) -> &'static NodeManifest {
        &FORCE_MAN
    }
    fn eval(&self, ctx: &mut EvalCtx<'_>) {
        let input = ctx.input(0);
        let n = input.count();
        let mut out = Stream::new(n);
        for (name, col) in input.columns() {
            out.set(name.clone(), col.clone());
        }
        let mut accel = vec2_to_n(input, "accel", n, [0.0, 0.0]);
        for a in &mut accel {
            a[0] += 1.0;
        }
        if n > 0 {
            out.set("accel", Column::Vec2(accel));
        }
        ctx.emit(out);
    }
}

// The same 2-instance source, but element 0 carries `inv_mass = 0` — what
// `motion.pin_constraint` writes upstream. Element 1 stays free.
static PIN_SRC_MAN: NodeManifest = NodeManifest {
    id: NodeTypeId::of("motion.integrate.test.pinned"),
    name: "motion.integrate.test.pinned",
    inputs: &[],
    outputs: &[PortSpec {
        name: "out",
        ty: INST_VEC2,
    }],
    effect: Effect::Temporal,
    clock: Clock::Frame,
    params: &[],
    lowerings: &[LoweringKind::Cpu],
};
struct PinnedSrc;
impl NodeOp for PinnedSrc {
    fn manifest(&self) -> &'static NodeManifest {
        &PIN_SRC_MAN
    }
    fn eval(&self, ctx: &mut EvalCtx<'_>) {
        let t = ctx.playhead() as f32;
        ctx.emit(
            Stream::new(2)
                .with("P", Column::Vec2(vec![[t, 0.0], [10.0 + t, 0.0]]))
                // A muzzle velocity on BOTH: the pin must beat it too (an
                // infinite mass cannot be carried off by its seed velocity).
                .with("vel", Column::Vec2(vec![[5.0, 0.0], [5.0, 0.0]]))
                .with(INV_MASS, Column::Scalar(vec![0.0, 1.0])),
        );
    }
}

struct Ops;
impl OpResolver for Ops {
    fn resolve(&self, ty: NodeTypeId) -> Option<&dyn NodeOp> {
        match ty {
            t if t == SRC_MAN.id => Some(&Src),
            t if t == PIN_SRC_MAN.id => Some(&PinnedSrc),
            t if t == FORCE_MAN.id => Some(&ConstForce),
            t if t == MANIFEST.id => Some(&MotionIntegrate),
            _ => None,
        }
    }
}

/// src → integrate.rest ; integrate.out --pre--> force → integrate.state.
fn loop_graph() -> (Graph, NodeId) {
    let mut g = Graph::new();
    let src = g.add_node("motion.integrate.test.src");
    let int = g.add_node("motion.integrate");
    let force = g.add_node("motion.integrate.test.force");
    g.connect(Edge {
        from: (src, 0),
        to: (int, 0),
        delayed: false,
    })
    .unwrap();
    g.connect(Edge {
        from: (int, 0),
        to: (force, 0),
        delayed: true,
    })
    .unwrap();
    g.connect(Edge {
        from: (force, 0),
        to: (int, 1),
        delayed: false,
    })
    .unwrap();
    (g, int)
}

fn p_of(cook: &mut Cook, g: &Graph, int: NodeId, playhead: f64) -> Vec<[f32; 2]> {
    let out = cook.cook(g, &Ops, int, playhead).unwrap();
    match out[0].as_stream().get("P").unwrap() {
        Column::Vec2(v) => v.clone(),
        _ => panic!("P"),
    }
}

/// Drive `ticks` fixed steps of `dt`, returning each tick's P.
fn run(ticks: usize, dt: f64) -> Vec<Vec<[f32; 2]>> {
    let (g, int) = loop_graph();
    let mut cook = Cook::new();
    let mut frames = Vec::new();
    for k in 0..ticks {
        let ph = k as f64 * dt;
        frames.push(p_of(&mut cook, &g, int, ph));
        cook.advance_tick(&g, &Ops, ph).unwrap();
    }
    frames
}

#[test]
fn seeds_at_tick_zero_then_integrates_semi_implicit_euler() {
    // Closed form of semi-implicit Euler under constant accel a from rest:
    // vel_k = a·dt·k, d_k = a·dt²·(1+2+…+k) = a·dt²·k(k+1)/2. The upstream
    // src also slides +t in X (liveness), so P.x = t + d_k. dt stays at the
    // MAX_DT boundary — the largest legible step the clamp admits.
    let dt = 0.1_f64;
    let frames = run(4, dt);
    // tick 0: seeded — no displacement, P = rest exactly.
    assert_eq!(frames[0][0], [0.0, 0.0]);
    assert_eq!(frames[0][1], [10.0, 0.0]);
    let d = |k: f64| (0.01 * k * (k + 1.0) / 2.0) as f32; // a=1, dt²=0.01
    for (k, frame) in frames.iter().enumerate().skip(1) {
        let t = (k as f64 * dt) as f32;
        let expect = t + d(k as f64);
        assert!(
            (frame[0][0] - expect).abs() < 1e-4,
            "tick {k}: got {}, want {expect} (live rest + integrated d)",
            frame[0][0]
        );
    }
}

#[test]
fn upstream_stays_live_after_seeding() {
    // The falsification of the "frozen seed" design bug: rest.P keeps
    // sliding after tick 0 and the emitted P follows it (plus physics).
    // If integrate froze the seed, P.x at tick 1 would be d_1 alone.
    let dt = 0.05_f64;
    let frames = run(2, dt);
    let t1 = dt as f32;
    let d1 = 0.0025_f32; // a·dt² with a=1, dt=0.05
    assert!(
        (frames[1][1][0] - (10.0 + t1 + d1)).abs() < 1e-4,
        "instance 1 follows the live rest (10 + t) plus its displacement"
    );
}

#[test]
fn replay_is_deterministic() {
    // Two independent runs of the same graph produce bit-identical
    // trajectories (HR-5: arithmetic only, dt from the stream itself).
    assert_eq!(run(5, 1.0 / 60.0), run(5, 1.0 / 60.0));
}

#[test]
fn accel_is_consumed_not_emitted() {
    let (g, int) = loop_graph();
    let mut cook = Cook::new();
    cook.cook(&g, &Ops, int, 0.0).unwrap();
    cook.advance_tick(&g, &Ops, 0.0).unwrap();
    let out = cook.cook(&g, &Ops, int, 1.0 / 60.0).unwrap();
    let s = out[0].as_stream();
    assert!(
        s.get("accel").is_none(),
        "the transient accel must be dropped so forces start from zero"
    );
    assert!(s.get("vel").is_some() && s.get("sim_d").is_some() && s.get("sim_t").is_some());
    // Non-sim columns flow live from rest.
    assert!(s.get("falloff").is_some(), "rest columns pass through");
}

#[test]
fn without_a_wired_state_the_output_holds_the_rest_pose() {
    // rest → integrate with NOTHING on the state port: seeds every tick,
    // never moves (and never panics on the Empty input).
    let mut g = Graph::new();
    let src = g.add_node("motion.integrate.test.src");
    let int = g.add_node("motion.integrate");
    g.connect(Edge {
        from: (src, 0),
        to: (int, 0),
        delayed: false,
    })
    .unwrap();
    let mut cook = Cook::new();
    let p = p_of(&mut cook, &g, int, 0.5);
    assert_eq!(p[0], [0.5, 0.0], "P = live rest, no displacement");
}

#[test]
fn a_backwards_playhead_freezes_for_one_tick_instead_of_exploding() {
    // Loop-wrap: playhead jumps backwards → dt clamps to 0 → the state
    // holds (no negative-dt integration, no reset of accumulated motion).
    let (g, int) = loop_graph();
    let mut cook = Cook::new();
    for k in 0..3 {
        let ph = k as f64 / 60.0;
        cook.cook(&g, &Ops, int, ph).unwrap();
        cook.advance_tick(&g, &Ops, ph).unwrap();
    }
    let before = p_of(&mut cook, &g, int, 3.0 / 60.0);
    cook.advance_tick(&g, &Ops, 3.0 / 60.0).unwrap();
    // Wrap to t=0: the src rest pose returns to t=0, displacement holds.
    let after = p_of(&mut cook, &g, int, 0.0);
    let d_before = before[0][0] - (3.0 / 60.0) as f32;
    let d_after = after[0][0];
    assert!(
        (d_after - d_before).abs() < 1e-4,
        "sim_d held across the wrap: {d_before} vs {d_after}"
    );
}

#[test]
fn count_change_reseeds() {
    // A count-varying source: 2 instances before t=1, 3 after.
    static VAR_MAN: NodeManifest = NodeManifest {
        id: NodeTypeId::of("motion.integrate.test.var"),
        name: "motion.integrate.test.var",
        inputs: &[],
        outputs: &[PortSpec {
            name: "out",
            ty: INST_VEC2,
        }],
        effect: Effect::Temporal,
        clock: Clock::Frame,
        params: &[],
        lowerings: &[LoweringKind::Cpu],
    };
    struct VarSrc;
    impl NodeOp for VarSrc {
        fn manifest(&self) -> &'static NodeManifest {
            &VAR_MAN
        }
        fn eval(&self, ctx: &mut EvalCtx<'_>) {
            let n = if ctx.playhead() < 1.0 { 2 } else { 3 };
            ctx.emit(Stream::new(n).with("P", Column::Vec2(vec![[0.0, 0.0]; n])));
        }
    }
    struct VarOps;
    impl OpResolver for VarOps {
        fn resolve(&self, ty: NodeTypeId) -> Option<&dyn NodeOp> {
            match ty {
                t if t == VAR_MAN.id => Some(&VarSrc),
                t if t == FORCE_MAN.id => Some(&ConstForce),
                t if t == MANIFEST.id => Some(&MotionIntegrate),
                _ => None,
            }
        }
    }
    let mut g = Graph::new();
    let src = g.add_node("motion.integrate.test.var");
    let int = g.add_node("motion.integrate");
    let force = g.add_node("motion.integrate.test.force");
    g.connect(Edge {
        from: (src, 0),
        to: (int, 0),
        delayed: false,
    })
    .unwrap();
    g.connect(Edge {
        from: (int, 0),
        to: (force, 0),
        delayed: true,
    })
    .unwrap();
    g.connect(Edge {
        from: (force, 0),
        to: (int, 1),
        delayed: false,
    })
    .unwrap();
    let mut cook = Cook::new();
    for k in 0..3 {
        let ph = k as f64 * 0.4;
        cook.cook(&g, &VarOps, int, ph).unwrap();
        cook.advance_tick(&g, &VarOps, ph).unwrap();
    }
    // t=1.2: count 2 → 3 → re-seed: displacement resets to zero.
    let out = cook.cook(&g, &VarOps, int, 1.2).unwrap();
    let s = out[0].as_stream();
    assert_eq!(s.count(), 3);
    match s.get("sim_d").unwrap() {
        Column::Vec2(v) => assert!(v.iter().all(|d| *d == [0.0, 0.0]), "re-seeded"),
        _ => panic!("sim_d"),
    }
}

#[test]
fn non_finite_state_recovers_by_reset() {
    // Feed a poisoned state directly through the pure step fn.
    let rest = Stream::new(1).with("P", Column::Vec2(vec![[1.0, 1.0]]));
    let state = Stream::new(1)
        .with("P", Column::Vec2(vec![[0.0, 0.0]]))
        .with("vel", Column::Vec2(vec![[f32::NAN, 0.0]]))
        .with("sim_d", Column::Vec2(vec![[5.0, 5.0]]))
        .with("sim_t", Column::Scalar(vec![0.0]));
    let out = step(&rest, &state, 1.0 / 60.0);
    match out.get("P").unwrap() {
        Column::Vec2(v) => assert_eq!(v[0], [1.0, 1.0], "reset to the live rest pose"),
        _ => panic!("P"),
    }
}

/// A stream with ids: the survivor keeps its state while its neighbours
/// churn around it. Falsify by pairing positionally and instance `7` picks
/// up the dead particle's velocity.
#[test]
fn an_id_survives_the_churn_and_keeps_its_state() {
    let with_ids = |ids: &[f32], p: &[[f32; 2]]| {
        Stream::new(ids.len())
            .with("P", Column::Vec2(p.to_vec()))
            .with("id", Column::Scalar(ids.to_vec()))
    };
    // Tick 0: particles 5 and 7 exist; 7 was launched with vel (2,0).
    let state = with_ids(&[5.0, 7.0], &[[0.0, 0.0], [0.0, 0.0]])
        .with("vel", Column::Vec2(vec![[9.0, 9.0], [2.0, 0.0]]))
        .with("sim_d", Column::Vec2(vec![[9.0, 9.0], [1.0, 0.0]]))
        .with("accel", Column::Vec2(vec![[0.0, 0.0], [0.0, 0.0]]))
        .with("sim_t", Column::Scalar(vec![0.0, 0.0]));
    // Tick 1: particle 5 died, 7 survived (now at row 0), 8 was born.
    let rest = with_ids(&[7.0, 8.0], &[[0.0, 0.0], [0.0, 0.0]])
        .with("vel", Column::Vec2(vec![[0.0, 0.0], [-3.0, 0.0]]));
    let out = step(&rest, &state, 0.1);

    let vel = match out.get("vel").unwrap() {
        Column::Vec2(v) => v.clone(),
        _ => panic!("vel"),
    };
    let d = match out.get("sim_d").unwrap() {
        Column::Vec2(v) => v.clone(),
        _ => panic!("sim_d"),
    };
    // 7 kept ITS velocity (2,0) and integrated from ITS displacement (1,0):
    // no accel, so d = 1 + 2·0.1 = 1.2. Not particle 5's (9,9) garbage.
    assert_eq!(vel[0], [2.0, 0.0], "the survivor keeps its velocity");
    assert!((d[0][0] - 1.2).abs() < 1e-5, "and its displacement");
    // 8 is newborn: seeded from rest's muzzle velocity, zero displacement.
    assert_eq!(
        vel[1],
        [-3.0, 0.0],
        "the newborn launches at its muzzle vel"
    );
    assert_eq!(d[1], [0.0, 0.0]);
    // The dead particle's row is gone (count follows rest).
    assert_eq!(out.count(), 2);
}

#[test]
fn an_id_stream_that_shrinks_does_not_reseed_the_survivors() {
    // The count changed, but identity says who survived — the id path must
    // NOT take the id-less "count changed → re-seed" branch.
    let state = Stream::new(2)
        .with("P", Column::Vec2(vec![[0.0, 0.0]; 2]))
        .with("id", Column::Scalar(vec![1.0, 2.0]))
        .with("vel", Column::Vec2(vec![[5.0, 0.0], [0.0, 0.0]]))
        .with("sim_d", Column::Vec2(vec![[4.0, 0.0], [0.0, 0.0]]))
        .with("sim_t", Column::Scalar(vec![0.0, 0.0]));
    let rest = Stream::new(1)
        .with("P", Column::Vec2(vec![[0.0, 0.0]]))
        .with("id", Column::Scalar(vec![1.0]));
    let out = step(&rest, &state, 1.0 / 60.0);
    match out.get("sim_d").unwrap() {
        Column::Vec2(v) => assert!(v[0][0] > 4.0, "kept + advanced, not reset"),
        _ => panic!("sim_d"),
    }
}

/// **The pin, through the whole loop** (`motion.pin_constraint`'s `inv_mass`
/// column): a pinned element (`w = 0`) is untouched by the force chain AND
/// by its own seed velocity — it sits exactly on its rest animation, which
/// keeps sliding — while its free neighbour, under the same force, runs
/// ahead of its rest pose. FALSIFIED if the integrator ignored `inv_mass`
/// (the pinned element would drift off with the others) or if it froze the
/// rest pose too (the pin would stop following the animation, which is what
/// makes a hanging cloth's corner track its animated hook).
#[test]
fn a_pinned_element_holds_its_rest_pose_while_its_free_neighbour_flies() {
    let mut g = Graph::new();
    let src = g.add_node("motion.integrate.test.pinned");
    let int = g.add_node("motion.integrate");
    let force = g.add_node("motion.integrate.test.force");
    for (from, to, delayed) in [
        ((src, 0), (int, 0), false),
        ((int, 0), (force, 0), true),
        ((force, 0), (int, 1), false),
    ] {
        g.connect(Edge { from, to, delayed }).unwrap();
    }

    let (mut cook, dt) = (Cook::new(), 1.0 / 60.0);
    let (mut last, mut last_ph) = (Vec::new(), 0.0f64);
    for k in 0..8 {
        let ph = k as f64 * dt;
        last = p_of(&mut cook, &g, int, ph);
        cook.advance_tick(&g, &Ops, ph).unwrap();
        last_ph = ph;
    }
    // The rest pose IS the playhead, taken down the same f64 path the source
    // took (`playhead as f32`) — re-deriving it in f32 arithmetic here would
    // differ in the last bit and the oracle, not the node, would be wrong.
    let ph = last_ph as f32;
    assert_eq!(
        last[0],
        [ph, 0.0],
        "the pinned element rides its rest animation EXACTLY (no force, no seed velocity)"
    );
    assert!(
        last[1][0] > 10.0 + ph + 1e-3,
        "the free element ran ahead of its rest pose ({} vs {})",
        last[1][0],
        10.0 + ph
    );
}

/// The pinned element reports zero velocity, not the phantom seed velocity a
/// downstream reader (`motion.look_at`) would otherwise aim at.
#[test]
fn a_pinned_element_reports_no_velocity() {
    let rest = Stream::new(2)
        .with("P", Column::Vec2(vec![[0.0, 0.0], [1.0, 0.0]]))
        .with("vel", Column::Vec2(vec![[5.0, 0.0], [5.0, 0.0]]))
        .with(INV_MASS, Column::Scalar(vec![0.0, 1.0]));
    let out = step(&rest, &Stream::new(0), 0.0);
    match out.get("vel").unwrap() {
        Column::Vec2(v) => assert_eq!(v, &vec![[0.0, 0.0], [5.0, 0.0]]),
        _ => panic!("vel"),
    }
}

#[test]
fn registers_and_resolves() {
    let mut reg = NodeRegistry::new();
    register(&mut reg).unwrap();
    assert!(reg.resolve(MANIFEST.id).is_some());
}
