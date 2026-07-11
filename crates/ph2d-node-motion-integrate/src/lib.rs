#![forbid(unsafe_code)]
//! `motion.integrate` — the **one** sequential integrator of the Motion dynamics
//! chain (plan §1.6, M2). Forces are `Pure` nodes that accumulate into the
//! transient `accel` column; this node semi-implicit-Euler-integrates that
//! acceleration into `vel` and an accumulated displacement `sim_d`, and emits
//! `P = rest.P (live) + sim_d` — so upstream animation (oscillator, orbit, a
//! moving emitter) keeps composing with the physics, exactly the reference's
//! `x = in.x + s.dx` behaviour but with the state as visible stream columns
//! instead of hidden per-node maps (replayable, GPU-lowerable ping-pong).
//!
//! ## Topology (the `pre` loop)
//!
//! ```text
//! rest ──fwd──> integrate.rest                 (the live upstream chain)
//! integrate.out ──pre──> force₁ ─…─ forceₙ ──fwd──> integrate.forces
//! ```
//!
//! With no forces the loop is the plain self-loop `out --pre--> forces` (the
//! editor auto-wires it on add: the plumbing recognises a feedback input named
//! `state` OR `forces` with the output's type — `motion_bridge_plumbing.rs`;
//! this node names it `forces` because the force chain is what plugs in). At
//! tick 0 the `pre` reads Empty → the node **seeds** from `rest` (`sim_d = 0`,
//! `vel` = rest's `vel` column or zero) and nothing moves; from tick 1 on it
//! steps.
//!
//! ## Column contract
//!
//! - Reads from `forces`: `vel`, `sim_d`, `sim_t`, and the transient `accel`
//!   the in-loop forces accumulated. `accel` is **consumed** (never emitted) so
//!   every tick starts from zero acceleration.
//! - Emits: every non-sim column copied **live from `rest`** (tint/size/uv/
//!   falloff keep animating), plus `P = rest.P + sim_d`, `vel`, `sim_d`, and
//!   `sim_t` (= this eval's playhead, from which the next eval derives `dt`).
//! - `dt = clamp(playhead − sim_t, 0, MAX_DT)` — derived from the playhead the
//!   state itself carries, so there is no cross-crate timestep constant and a
//!   backwards playhead jump (loop wrap) freezes for one tick instead of
//!   exploding. The shell cooks once per fixed tick, so in steady state `dt`
//!   IS the fixed timestep (deterministic, HR-5: arithmetic only).
//! - A non-finite `vel`/`sim_d` element resets that instance (the reference's
//!   NaN guard: recover automatically instead of freezing a diverged sim).
//!
//! ## Identity: the `id` column vs position
//!
//! A stream carrying an **`id` column** (the particle emitter stamps one) has
//! *element identity*: the set churns every tick as particles are born and die,
//! yet each survivor must keep its velocity and displacement. State is matched
//! by id; an id with no prior state is **seeded** (taking `vel` from `rest`, so
//! the emitter's muzzle velocity is what launches it) and a state row whose id
//! vanished dies with it.
//!
//! A stream with **no `id`** (a grid, a cloner) has only positional identity, so
//! a changed count means the whole set was rebuilt: the state re-seeds wholesale
//! rather than pairing unrelated elements by index.

use ph2d_node_registry::{NodeRegistry, RegistryError};
use ph2d_nodegraph::attr::{Column, Stream};
use ph2d_nodegraph::cook::EvalCtx;
use ph2d_nodegraph::effect::Effect;
use ph2d_nodegraph::node::{LoweringKind, NodeManifest, NodeOp, NodeTypeId, PortSpec};
use ph2d_nodegraph::port::{Clock, Dim, Domain, PortType};

mod columns;
use columns::{ids_of, scalar_to_n, vec2_to_n};
use std::collections::BTreeMap;

const INST_VEC2: PortType = PortType::new(Domain::Instances, Dim::Vec2, Clock::Frame);

/// Ceiling on a single integration step, in seconds. In steady state `dt` is
/// the fixed timestep (~1/60); this only guards a pathological playhead jump
/// (a scrub / loop-wrap) from becoming one giant unstable step.
const MAX_DT: f32 = 0.1;

/// The static contract of this node type (ADR-0031).
pub const MANIFEST: NodeManifest = NodeManifest {
    id: NodeTypeId::of("motion.integrate"),
    name: "motion.integrate",
    inputs: &[
        PortSpec {
            name: "rest",
            ty: INST_VEC2,
        },
        // The feedback port. `forces` receives the force chain's output (last
        // tick's state with `accel` accumulated); the editor owns its `pre`
        // plumbing — self-loop when empty, `out --pre--> chain head` when a
        // chain is wired in (docs/Motion Nodes/03, O1). The user never draws
        // the loop.
        PortSpec {
            name: "forces",
            ty: INST_VEC2,
        },
    ],
    outputs: &[PortSpec {
        name: "out",
        ty: INST_VEC2,
    }],
    // Temporal: `eval` reads `ctx.playhead()` (it stamps `sim_t` and derives
    // `dt = playhead − sim_t`), and only a Temporal manifest folds the playhead
    // into the memo fingerprint. The consumed `pre` edge already forces a
    // re-cook per tick, which masks this during forward playback — but a
    // same-tick re-cook at a moved playhead (checkpoint/restore scrub, M2.N2)
    // would return a stale `sim_t`/trajectory under `Pure`. Convention: reads
    // playhead ⇒ Temporal (`motion.oscillator`, `pulse.beat`).
    effect: Effect::Temporal,
    clock: Clock::Frame,
    params: &[],
    lowerings: &[LoweringKind::Cpu],
};

struct MotionIntegrate;

impl NodeOp for MotionIntegrate {
    fn manifest(&self) -> &'static NodeManifest {
        &MANIFEST
    }

    fn eval(&self, ctx: &mut EvalCtx<'_>) {
        let playhead = ctx.playhead() as f32;
        let out = {
            let rest = ctx.input(0);
            let state = ctx.input(1);
            step(rest, state, playhead)
        };
        ctx.emit(out);
    }
}

/// One integration step (or a seed) as a pure function — the whole node.
fn step(rest: &Stream, state: &Stream, playhead: f32) -> Stream {
    let n = rest.count();
    // Every non-sim column flows LIVE from rest; the transient `accel` is
    // consumed (dropped) and the sim columns are rewritten below. `id` is NOT a
    // sim column — it rides through, so next tick's state carries it.
    let mut out = Stream::new(n);
    for (name, col) in rest.columns() {
        if !matches!(name.as_str(), "accel" | "vel" | "sim_d" | "sim_t") {
            out.set(name.clone(), col.clone());
        }
    }
    let rest_p = vec2_to_n(rest, "P", n, [0.0, 0.0]);

    // Every element starts at its seed (`vel` from rest — the emitter's muzzle
    // velocity — and no displacement); the ones the state knows then step.
    let mut vel = vec2_to_n(rest, "vel", n, [0.0, 0.0]);
    let mut sim_d = vec![[0.0f32; 2]; n];

    // Pair each rest element with its prior state row (`None` = seed it).
    if let Some(prev) = pairing(rest, state, n) {
        let sn = state.count();
        // dt from the state's own clock column — see the module docs.
        let t_prev = scalar_to_n(state, "sim_t", sn, playhead);
        let dt = (playhead - t_prev.first().copied().unwrap_or(playhead)).clamp(0.0, MAX_DT);
        let s_vel = vec2_to_n(state, "vel", sn, [0.0, 0.0]);
        let s_sim_d = vec2_to_n(state, "sim_d", sn, [0.0, 0.0]);
        let accel = vec2_to_n(state, "accel", sn, [0.0, 0.0]);
        for (i, slot) in prev.iter().enumerate() {
            let Some(j) = *slot else { continue }; // a fresh id: stays seeded
            let (mut v, mut d, a) = (s_vel[j], s_sim_d[j], accel[j]);
            // Semi-implicit Euler: velocity first, then position with the NEW
            // velocity (symplectic — stable for oscillatory forces).
            v[0] += a[0] * dt;
            v[1] += a[1] * dt;
            d[0] += v[0] * dt;
            d[1] += v[1] * dt;
            // NaN/∞ guard (reference parity): a diverged instance recovers by
            // resetting instead of freezing the whole sim.
            if v.iter().chain(&d).all(|x| x.is_finite()) {
                vel[i] = v;
                sim_d[i] = d;
            }
        }
    }

    let p: Vec<[f32; 2]> = rest_p
        .iter()
        .zip(&sim_d)
        .map(|(r, d)| [r[0] + d[0], r[1] + d[1]])
        .collect();
    out.set("P", Column::Vec2(p));
    out.set("vel", Column::Vec2(vel));
    out.set("sim_d", Column::Vec2(sim_d));
    out.set("sim_t", Column::Scalar(vec![playhead; n]));
    out
}

/// For each of the `n` rest elements, the row of `state` holding its previous
/// simulation state — `None` for a freshly-born element (it stays at its seed).
/// The whole result is `None` (seed everything) when the state carries no sim
/// columns yet (tick 0, `pre` = Empty) or when a count change on an id-less
/// stream says the set was rebuilt (see the module docs).
fn pairing(rest: &Stream, state: &Stream, n: usize) -> Option<Vec<Option<usize>>> {
    state.get("sim_d")?; // no state yet (tick 0, `pre` = Empty)
    let sn = state.count();
    match (ids_of(rest, n), ids_of(state, sn)) {
        // Element identity: match by id. `BTreeMap` keeps it deterministic and
        // O(n log n) rather than the O(n·sn) scan a naive `position()` would be.
        (Some(rest_ids), Some(state_ids)) => {
            let index: BTreeMap<u32, usize> = state_ids
                .iter()
                .enumerate()
                .map(|(j, id)| (*id, j))
                .collect();
            Some(rest_ids.iter().map(|id| index.get(id).copied()).collect())
        }
        // Positional identity: a stable count pairs row-for-row; a changed count
        // means the set was rebuilt, so re-seed rather than pair by index.
        _ if sn == n => Some((0..n).map(Some).collect()),
        _ => None,
    }
}

/// Register this node with the runtime registry. Called (via codegen) from
/// `ph2d-node-registry-init::register_all_nodes`.
pub fn register(reg: &mut NodeRegistry) -> Result<(), RegistryError> {
    reg.register(Box::new(MotionIntegrate))?;
    reg.register_ui(
        MANIFEST.id,
        ph2d_node_registry::NodeUiManifest {
            display_name: "Integrate",
            category: ph2d_node_registry::NodeUiCategory::Transform,
            silhouette: ph2d_node_registry::NodeSilhouette::Rect,
        },
    );
    // No params → no param hints to register (the panel renders no rows).
    Ok(())
}

#[cfg(test)]
mod tests {
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

    struct Ops;
    impl OpResolver for Ops {
        fn resolve(&self, ty: NodeTypeId) -> Option<&dyn NodeOp> {
            match ty {
                t if t == SRC_MAN.id => Some(&Src),
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

    #[test]
    fn registers_and_resolves() {
        let mut reg = NodeRegistry::new();
        register(&mut reg).unwrap();
        assert!(reg.resolve(MANIFEST.id).is_some());
    }
}
