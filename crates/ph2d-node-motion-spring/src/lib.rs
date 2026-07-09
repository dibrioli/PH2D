#![forbid(unsafe_code)]
//! `motion.spring` — a damped spring that makes one transform channel *chase*
//! its animated upstream target with lag, overshoot and settle (follow-through).
//! The reference's warning is the usage manual: **it only acts on targets that
//! CHANGE** — wire it after an oscillator/stagger/mouse-ish driver, or it is
//! invisible.
//!
//! A **sequential** node: its state (`spring_value`, `spring_vel` per instance)
//! rides its own output as columns through the `pre` self-loop
//! (`out --pre--> state`, auto-wired on add — the input named `state` with the
//! output's type is the sequential-node convention). At tick 0 the state seeds
//! at the target (no snap); each tick it advances one fixed step.
//!
//! Algorithm (MiniCavalryV2 `spring.js`, clean-room — plan §1.7 mandates the
//! literal port): semi-implicit Euler with an **adaptive sub-step** chosen from
//! the stability limit `sub_dt² · tension < 0.05` — stiff springs stay stable
//! by taking more, smaller steps instead of exploding:
//!
//! ```text
//! accel = −friction·vel − tension·(value − target)
//! vel   += accel·sub_dt ; value += vel·sub_dt      (× steps)
//! ```
//!
//! `dt` derives from the state's own `sim_t` column (playhead at last eval),
//! clamped to `[0, MAX_DT]` — no cross-crate timestep constant, deterministic
//! (HR-5: arithmetic + IEEE sqrt only). The multiplicative `falloff` column
//! blends the OUTPUT between the raw target (0) and the full spring (1); the
//! raw state keeps evolving regardless, exactly like the reference.

use ph2d_node_registry::{NodeRegistry, RegistryError};
use ph2d_nodegraph::attr::{Column, Stream};
use ph2d_nodegraph::cook::EvalCtx;
use ph2d_nodegraph::effect::Effect;
use ph2d_nodegraph::node::{LoweringKind, NodeManifest, NodeOp, NodeTypeId, ParamSpec, PortSpec};
use ph2d_nodegraph::port::{Clock, Dim, Domain, PortType};

mod channel;
use channel::{channel_get, channel_set, falloff_at};

const INST_VEC2: PortType = PortType::new(Domain::Instances, Dim::Vec2, Clock::Frame);

/// Ceiling on a single integration step (see `motion.integrate`).
const MAX_DT: f32 = 0.1;
/// Stability bound for the adaptive sub-step: `sub_dt² · tension < STABLE`.
const STABLE: f32 = 0.05;
/// Hard cap on sub-steps per tick — at the UI's max tension (60) and MAX_DT the
/// adaptive count is 4, so 64 only guards absurd hand-authored overrides.
const MAX_STEPS: usize = 64;

/// The static contract of this node type (ADR-0031).
pub const MANIFEST: NodeManifest = NodeManifest {
    id: NodeTypeId::of("motion.spring"),
    name: "motion.spring",
    inputs: &[
        PortSpec {
            name: "in",
            ty: INST_VEC2,
        },
        // The feedback port — auto-wired `out --pre--> state` on add.
        PortSpec {
            name: "state",
            ty: INST_VEC2,
        },
    ],
    outputs: &[PortSpec {
        name: "out",
        ty: INST_VEC2,
    }],
    // Pure: the tick enters the fingerprint via the consumed `pre` edge; dt
    // derives from the state's own `sim_t`.
    effect: Effect::Pure,
    clock: Clock::Frame,
    params: &[
        ParamSpec {
            name: "channel",
            default: 1.0, // Y — the reference's default
        },
        ParamSpec {
            name: "tension",
            default: 8.0,
        },
        ParamSpec {
            name: "friction",
            default: 1.5,
        },
    ],
    lowerings: &[LoweringKind::Cpu],
};

struct MotionSpring;

impl NodeOp for MotionSpring {
    fn manifest(&self) -> &'static NodeManifest {
        &MANIFEST
    }

    fn eval(&self, ctx: &mut EvalCtx<'_>) {
        let channel = ctx.param("channel").round() as i32;
        let tension = ctx.param("tension").max(0.1);
        let friction = ctx.param("friction").max(0.05);
        let playhead = ctx.playhead() as f32;
        let out = {
            let input = ctx.input(0);
            let state = ctx.input(1);
            step(input, state, channel, tension, friction, playhead)
        };
        ctx.emit(out);
    }
}

/// One spring step (or a seed) as a pure function — the whole node.
fn step(
    input: &Stream,
    state: &Stream,
    channel: i32,
    tension: f32,
    friction: f32,
    playhead: f32,
) -> Stream {
    let n = input.count();
    let targets: Vec<f32> = (0..n).map(|i| channel_get(input, channel, i)).collect();

    let seeded = state.get("spring_value").is_none() || state.count() != n;
    let (mut value, mut vel) = if seeded {
        (targets.clone(), vec![0.0f32; n])
    } else {
        let read = |name: &str| -> Vec<f32> {
            let mut v = match state.get(name) {
                Some(Column::Scalar(v)) => v.clone(),
                _ => Vec::new(),
            };
            v.resize(n, 0.0);
            v
        };
        (read("spring_value"), read("spring_vel"))
    };

    if !seeded {
        let t_prev = match state.get("sim_t") {
            Some(Column::Scalar(v)) => v.first().copied().unwrap_or(playhead),
            _ => playhead,
        };
        let dt = (playhead - t_prev).clamp(0.0, MAX_DT);
        // Adaptive sub-step from the stability limit (reference parity).
        let ideal = (STABLE / tension).sqrt();
        let steps = if dt > 0.0 {
            ((dt / ideal).ceil() as usize).clamp(1, MAX_STEPS)
        } else {
            1
        };
        let sub_dt = dt / steps as f32;
        for i in 0..n {
            // NaN/∞ guard (reference parity): a diverged instance recovers.
            if !(value[i].is_finite() && vel[i].is_finite()) {
                value[i] = targets[i];
                vel[i] = 0.0;
            }
            for _ in 0..steps {
                let accel = -friction * vel[i] - tension * (value[i] - targets[i]);
                vel[i] += accel * sub_dt;
                value[i] += vel[i] * sub_dt;
            }
        }
    }

    // Output channel: the spring blended toward the raw target by the falloff
    // field (falloff 0 → the spring is transparent there).
    let blended: Vec<f32> = (0..n)
        .map(|i| {
            let fs = falloff_at(input, i);
            if fs >= 1.0 {
                value[i]
            } else {
                targets[i] + (value[i] - targets[i]) * fs.max(0.0)
            }
        })
        .collect();
    let mut out = channel_set(input, channel, &blended);
    out.set("spring_value", Column::Scalar(value));
    out.set("spring_vel", Column::Scalar(vel));
    out.set("sim_t", Column::Scalar(vec![playhead; n]));
    out
}

/// Register this node with the runtime registry. Called (via codegen) from
/// `ph2d-node-registry-init::register_all_nodes`.
pub fn register(reg: &mut NodeRegistry) -> Result<(), RegistryError> {
    reg.register(Box::new(MotionSpring))?;
    reg.register_ui(
        MANIFEST.id,
        ph2d_node_registry::NodeUiManifest {
            display_name: "Spring",
            category: ph2d_node_registry::NodeUiCategory::Transform,
            silhouette: ph2d_node_registry::NodeSilhouette::Rect,
        },
    );
    reg.register_param_ui(MANIFEST.id, PARAM_HINTS);
    Ok(())
}

use ph2d_node_registry::{ParamUiHint, ParamWidget};

/// Param UI hints (M1.P1) — ranges mirror the reference's sliders.
static PARAM_HINTS: &[ParamUiHint] = &[
    ParamUiHint {
        param: "channel",
        label: "Channel",
        min: 0.0,
        max: 3.0,
        step: 1.0,
        widget: ParamWidget::Enum {
            labels: &["X", "Y", "Rot", "Size"],
        },
    },
    ParamUiHint {
        param: "tension",
        label: "Tension",
        min: 0.5,
        max: 60.0,
        step: 0.1,
        widget: ParamWidget::Slider,
    },
    ParamUiHint {
        param: "friction",
        label: "Friction",
        min: 0.1,
        max: 20.0,
        step: 0.1,
        widget: ParamWidget::Slider,
    },
];

#[cfg(test)]
mod tests {
    use super::*;
    use ph2d_nodegraph::cook::{Cook, OpResolver};
    use ph2d_nodegraph::graph::{Edge, Graph, NodeId};

    // A target that STEPS: y = 0 before t = 0.5, y = 2 after. The spring must
    // lag it, overshoot it, and settle on it.
    static SRC_MAN: NodeManifest = NodeManifest {
        id: NodeTypeId::of("motion.spring.test.src"),
        name: "motion.spring.test.src",
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
            let y = if ctx.playhead() < 0.5 { 0.0 } else { 2.0 };
            ctx.emit(Stream::new(1).with("P", Column::Vec2(vec![[0.0, y]])));
        }
    }
    struct Ops;
    impl OpResolver for Ops {
        fn resolve(&self, ty: NodeTypeId) -> Option<&dyn NodeOp> {
            match ty {
                t if t == SRC_MAN.id => Some(&Src),
                t if t == MANIFEST.id => Some(&MotionSpring),
                _ => None,
            }
        }
    }

    /// src → spring.in, with the pre self-loop the editor template creates.
    fn spring_graph(params: &[(&str, f32)]) -> (Graph, NodeId) {
        let mut g = Graph::new();
        let src = g.add_node("motion.spring.test.src");
        let sp = g.add_node("motion.spring");
        g.connect(Edge {
            from: (src, 0),
            to: (sp, 0),
            delayed: false,
        })
        .unwrap();
        g.connect(Edge {
            from: (sp, 0),
            to: (sp, 1),
            delayed: true,
        })
        .unwrap();
        for (name, v) in params {
            g.set_param(sp, *name, *v);
        }
        (g, sp)
    }

    /// Run `ticks` fixed steps at 60 Hz, returning the emitted Y per tick.
    fn run(params: &[(&str, f32)], ticks: usize) -> Vec<f32> {
        let (g, sp) = spring_graph(params);
        let mut cook = Cook::new();
        let mut ys = Vec::new();
        for k in 0..ticks {
            let ph = k as f64 / 60.0;
            let out = cook.cook(&g, &Ops, sp, ph).unwrap();
            match out[0].as_stream().get("P").unwrap() {
                Column::Vec2(v) => ys.push(v[0][1]),
                _ => panic!("P"),
            }
            cook.advance_tick(&g, &Ops, ph).unwrap();
        }
        ys
    }

    #[test]
    fn seeds_at_the_target_then_lags_overshoots_and_settles() {
        // 3 seconds at 60 Hz; the step lands at t = 0.5 (tick 30).
        let ys = run(&[("tension", 20.0), ("friction", 3.0)], 180);
        assert_eq!(ys[0], 0.0, "seeded at the target, no snap");
        // Right after the step the spring LAGS (well below the new target).
        assert!(ys[32] < 1.0, "lags the step, got {}", ys[32]);
        // It then OVERSHOOTS (crosses above 2) — the follow-through that
        // distinguishes a spring from a lerp; a lerp never crosses its target.
        let peak = ys[30..].iter().cloned().fold(f32::MIN, f32::max);
        assert!(peak > 2.05, "overshoots the target, peak {peak}");
        // And finally SETTLES on it.
        let last = *ys.last().unwrap();
        assert!(
            (last - 2.0).abs() < 0.05,
            "settles at the target, got {last}"
        );
    }

    #[test]
    fn without_the_state_loop_it_follows_the_target_exactly() {
        // No pre self-loop wired → seeds every tick → output == target. The
        // reference's "only acts on targets that change" footnote, inverted.
        let mut g = Graph::new();
        let src = g.add_node("motion.spring.test.src");
        let sp = g.add_node("motion.spring");
        g.connect(Edge {
            from: (src, 0),
            to: (sp, 0),
            delayed: false,
        })
        .unwrap();
        let mut cook = Cook::new();
        let out = cook.cook(&g, &Ops, sp, 0.9).unwrap();
        match out[0].as_stream().get("P").unwrap() {
            Column::Vec2(v) => assert_eq!(v[0][1], 2.0, "seeds at the live target"),
            _ => panic!("P"),
        }
    }

    #[test]
    fn stiff_spring_stays_stable_via_sub_steps() {
        // tension 60 (the UI max) would explode a single 1/60 Euler step
        // (dt²·k = 0.0167²·60 ≈ 0.017 is fine, but MAX_DT-sized steps are not:
        // 0.1²·60 = 0.6 >> 0.05). Drive with dt = MAX_DT ticks: bounded, settles.
        let (g, sp) = spring_graph(&[("tension", 60.0), ("friction", 2.0)]);
        let mut cook = Cook::new();
        let mut last = 0.0f32;
        for k in 0..60 {
            let ph = k as f64 * 0.1;
            let out = cook.cook(&g, &Ops, sp, ph).unwrap();
            match out[0].as_stream().get("P").unwrap() {
                Column::Vec2(v) => last = v[0][1],
                _ => panic!("P"),
            }
            assert!(last.is_finite() && last.abs() < 10.0, "bounded at tick {k}");
            cook.advance_tick(&g, &Ops, ph).unwrap();
        }
        assert!((last - 2.0).abs() < 0.1, "settled, got {last}");
    }

    #[test]
    fn replay_is_deterministic() {
        let a = run(&[("tension", 8.0)], 90);
        let b = run(&[("tension", 8.0)], 90);
        assert_eq!(a, b);
    }

    #[test]
    fn size_channel_springs_around_unit_identity() {
        // A bare P-only stream on the Size channel: the target is the unit
        // identity → the spring holds size 1.0 (never 0 — sprites don't vanish).
        let (g, sp) = spring_graph(&[("channel", 3.0)]);
        let mut cook = Cook::new();
        for k in 0..3 {
            let ph = k as f64 / 60.0;
            let out = cook.cook(&g, &Ops, sp, ph).unwrap();
            match out[0].as_stream().get("size").unwrap() {
                Column::Vec2(v) => assert_eq!(v[0], [1.0, 1.0], "unit identity at tick {k}"),
                _ => panic!("size"),
            }
            cook.advance_tick(&g, &Ops, ph).unwrap();
        }
    }

    #[test]
    fn falloff_zero_makes_the_spring_transparent() {
        // Poke the pure step fn directly: with falloff 0 the OUTPUT is the raw
        // target even while the internal state lags elsewhere.
        let input = Stream::new(1)
            .with("P", Column::Vec2(vec![[0.0, 5.0]]))
            .with("falloff", Column::Scalar(vec![0.0]));
        let state = Stream::new(1)
            .with("P", Column::Vec2(vec![[0.0, 0.0]]))
            .with("spring_value", Column::Scalar(vec![0.0]))
            .with("spring_vel", Column::Scalar(vec![0.0]))
            .with("sim_t", Column::Scalar(vec![0.0]));
        let out = step(&input, &state, 1, 8.0, 1.5, 1.0 / 60.0);
        match out.get("P").unwrap() {
            Column::Vec2(v) => assert_eq!(v[0][1], 5.0, "falloff 0: raw target"),
            _ => panic!("P"),
        }
    }

    #[test]
    fn registers_and_resolves() {
        let mut reg = NodeRegistry::new();
        register(&mut reg).unwrap();
        assert!(reg.resolve(MANIFEST.id).is_some());
    }
}
