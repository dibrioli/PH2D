#![forbid(unsafe_code)]
//! `motion.oscillator` — a Motion **behaviour**: oscillates a chosen channel of
//! the stream over the playhead, **added** to the existing value and scaled
//! per-instance by the multiplicative `falloff` column (§1.2; absent → `1.0`).
//! Each instance samples the waveform at `phase = t·frequency + i·phase_stagger`,
//! so a non-zero `phase_stagger` sends a travelling wave across the grid. Reads
//! the playhead but holds no state → `Effect::Temporal` (pull-side). Every other
//! column passes through unchanged (count preserved).
//!
//! Waveforms are **transcendental-free** (HR-5): `phase` is measured in *cycles*
//! (unit period) and the shapes are piecewise polynomial — a parabolic
//! sine-approximation stands in for the true sine (the plan §1.7 marks sine as
//! non-deterministic).
//!
//! Params (read via `ctx.param`):
//! - `channel` (1): target — `0` X, `1` Y, `2` Rotation, `3` Size.
//! - `wave` (0): shape — `0` Parabolic (sine-like), `1` Triangle, `2` Square,
//!   `3` Saw.
//! - `amplitude` (1): peak of the oscillation (channel-native units).
//! - `frequency` (1): cycles per second of playhead.
//! - `phase_stagger` (0.5): per-instance phase offset (cycles) → the wave.
//!
//! `delta_i = wave(t·frequency + i·phase_stagger) · amplitude · falloff_i`.

use ph2d_node_registry::{NodeRegistry, RegistryError};
use ph2d_nodegraph::cook::EvalCtx;
use ph2d_nodegraph::effect::Effect;
use ph2d_nodegraph::node::{LoweringKind, NodeManifest, NodeOp, NodeTypeId, ParamSpec, PortSpec};
use ph2d_nodegraph::port::{Clock, Dim, Domain, PortType};

mod channel;
use channel::{apply_channel_delta, falloff_at};

const INST_VEC2: PortType = PortType::new(Domain::Instances, Dim::Vec2, Clock::Frame);

/// The static contract of this node type (ADR-0031).
pub const MANIFEST: NodeManifest = NodeManifest {
    id: NodeTypeId::of("motion.oscillator"),
    name: "motion.oscillator",
    inputs: &[PortSpec {
        name: "in",
        ty: INST_VEC2,
    }],
    outputs: &[PortSpec {
        name: "out",
        ty: INST_VEC2,
    }],
    // Reads the playhead → pull-side, HR-5-exempt for the clock (the waveform
    // math is nonetheless transcendental-free for cross-platform stability).
    effect: Effect::Temporal,
    clock: Clock::Frame,
    params: &[
        ParamSpec {
            name: "channel",
            default: 1.0,
        },
        ParamSpec {
            name: "wave",
            default: 0.0,
        },
        ParamSpec {
            name: "amplitude",
            default: 1.0,
        },
        ParamSpec {
            name: "frequency",
            default: 1.0,
        },
        ParamSpec {
            name: "phase_stagger",
            default: 0.5,
        },
    ],
    lowerings: &[LoweringKind::Cpu],
};

/// The fractional part of `p` in `[0,1)` — IEEE `floor` is correctly-rounded and
/// deterministic (HR-5-safe, unlike `sin`).
fn frac(p: f32) -> f32 {
    p - p.floor()
}

/// A periodic waveform in `[-1,1]` at `phase` (in cycles, period 1). All shapes
/// are piecewise polynomial → transcendental-free (HR-5). Unknown / `0` is the
/// parabolic sine-approximation.
fn waveform(kind: i32, phase: f32) -> f32 {
    let f = frac(phase);
    match kind {
        1 => {
            // Triangle: 0 at 0, +1 at ¼, 0 at ½, −1 at ¾.
            if f < 0.25 {
                4.0 * f
            } else if f < 0.75 {
                2.0 - 4.0 * f
            } else {
                4.0 * f - 4.0
            }
        }
        2 => {
            // Square: +1 first half, −1 second.
            if f < 0.5 { 1.0 } else { -1.0 }
        }
        3 => 2.0 * f - 1.0, // Saw: −1 → +1 rising.
        _ => {
            // Parabolic sine-approximation: a +hump over [0,½), a −hump over
            // [½,1), each `±4u(1−u)` — continuous, 0 at 0/½, ±1 at ¼/¾.
            if f < 0.5 {
                let u = f * 2.0;
                4.0 * u * (1.0 - u)
            } else {
                let u = (f - 0.5) * 2.0;
                -4.0 * u * (1.0 - u)
            }
        }
    }
}

struct MotionOscillator;

impl NodeOp for MotionOscillator {
    fn manifest(&self) -> &'static NodeManifest {
        &MANIFEST
    }

    fn eval(&self, ctx: &mut EvalCtx<'_>) {
        let channel = ctx.param("channel").round() as i32;
        let wave = ctx.param("wave").round() as i32;
        let amplitude = ctx.param("amplitude");
        let frequency = ctx.param("frequency");
        let phase_stagger = ctx.param("phase_stagger");
        let t = ctx.playhead() as f32;
        let out = {
            let input = ctx.input(0);
            let n = input.count();
            let deltas: Vec<f32> = (0..n)
                .map(|i| {
                    let phase = t * frequency + i as f32 * phase_stagger;
                    waveform(wave, phase) * amplitude * falloff_at(input, i)
                })
                .collect();
            apply_channel_delta(input, channel, &deltas)
        };
        ctx.emit(out);
    }
}

/// Register this node with the runtime registry. Called (via codegen) from
/// `ph2d-node-registry-init::register_all_nodes`.
pub fn register(reg: &mut NodeRegistry) -> Result<(), RegistryError> {
    reg.register(Box::new(MotionOscillator))?;
    // M1.R1 — UI metadata. Behaviours modify transform channels → Transform
    // (blue) for now; a dedicated Behaviour category (cyan) is a follow-up.
    reg.register_ui(
        MANIFEST.id,
        ph2d_node_registry::NodeUiManifest {
            display_name: "Oscillator",
            category: ph2d_node_registry::NodeUiCategory::Transform,
            silhouette: ph2d_node_registry::NodeSilhouette::Rect,
        },
    );
    reg.register_param_ui(MANIFEST.id, PARAM_HINTS);
    Ok(())
}

use ph2d_node_registry::{ParamUiHint, ParamWidget};

/// Param UI hints (M1.P1). `channel` / `wave` are **named** selectors (segmented
/// buttons) — never number sliders. The enum option index IS the param value
/// (channel 0..3; wave 0..3 = Parabolic/Triangle/Square/Saw — "Sine" is the
/// user-facing name for the transcendental-free parabolic approximation).
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
        param: "wave",
        label: "Wave",
        min: 0.0,
        max: 3.0,
        step: 1.0,
        widget: ParamWidget::Enum {
            labels: &["Sine", "Tri", "Square", "Saw"],
        },
    },
    ParamUiHint {
        param: "amplitude",
        label: "Amplitude",
        min: 0.0,
        max: 10.0,
        step: 0.05,
        widget: ParamWidget::Slider,
    },
    ParamUiHint {
        param: "frequency",
        label: "Frequency",
        min: 0.0,
        max: 8.0,
        step: 0.05,
        widget: ParamWidget::Slider,
    },
    ParamUiHint {
        param: "phase_stagger",
        label: "Phase",
        min: 0.0,
        max: 2.0,
        step: 0.02,
        widget: ParamWidget::Slider,
    },
];

#[cfg(test)]
mod tests {
    use super::*;
    use ph2d_nodegraph::attr::{Column, Stream};
    use ph2d_nodegraph::cook::{Cook, OpResolver};
    use ph2d_nodegraph::graph::{Edge, Graph, NodeId};

    // Source: 2 instances at the origin (so the oscillation IS the output).
    static SRC_MAN: NodeManifest = NodeManifest {
        id: NodeTypeId::of("motion.oscillator.test.src"),
        name: "motion.oscillator.test.src",
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
    struct Src;
    impl NodeOp for Src {
        fn manifest(&self) -> &'static NodeManifest {
            &SRC_MAN
        }
        fn eval(&self, ctx: &mut EvalCtx<'_>) {
            ctx.emit(Stream::new(2).with("P", Column::Vec2(vec![[0.0, 0.0], [0.0, 0.0]])));
        }
    }
    struct Ops;
    impl OpResolver for Ops {
        fn resolve(&self, ty: NodeTypeId) -> Option<&dyn NodeOp> {
            match ty {
                t if t == SRC_MAN.id => Some(&Src),
                t if t == MANIFEST.id => Some(&MotionOscillator),
                _ => None,
            }
        }
    }

    fn osc_y_at(playhead: f64, setup: impl FnOnce(&mut Graph, NodeId)) -> Vec<[f32; 2]> {
        let mut g = Graph::new();
        let src = g.add_node("motion.oscillator.test.src");
        let osc = g.add_node("motion.oscillator");
        g.connect(Edge {
            from: (src, 0),
            to: (osc, 0),
            delayed: false,
        })
        .unwrap();
        g.set_param(osc, "channel", 1.0); // Y
        setup(&mut g, osc);
        let mut cook = Cook::new();
        let out = cook.cook(&g, &Ops, osc, playhead).unwrap();
        match out[0].as_stream().get("P").unwrap() {
            Column::Vec2(v) => v.clone(),
            _ => panic!("P"),
        }
    }

    #[test]
    fn at_playhead_zero_the_oscillation_is_neutral() {
        // The default parabolic wave is 0 at phase 0; with phase_stagger 0 every
        // instance is at phase 0 → no displacement (deterministic origin).
        let p = osc_y_at(0.0, |g, osc| {
            g.set_param(osc, "amplitude", 5.0);
            g.set_param(osc, "phase_stagger", 0.0);
        });
        assert_eq!(p, vec![[0.0, 0.0], [0.0, 0.0]]);
    }

    #[test]
    fn quarter_cycle_parabolic_reaches_peak_amplitude() {
        // frequency 1, t = 0.25 → phase ¼ → parabolic peak +1 → Δy = +amplitude.
        let p = osc_y_at(0.25, |g, osc| {
            g.set_param(osc, "amplitude", 3.0);
            g.set_param(osc, "phase_stagger", 0.0);
        });
        assert_eq!(p, vec![[0.0, 3.0], [0.0, 3.0]]);
    }

    #[test]
    fn phase_stagger_offsets_later_instances() {
        // t=0, phase_stagger 0.25: instance 0 at phase 0 (→0), instance 1 at
        // phase ¼ (parabolic peak +1) → only the second instance displaces.
        let p = osc_y_at(0.0, |g, osc| {
            g.set_param(osc, "amplitude", 2.0);
            g.set_param(osc, "phase_stagger", 0.25);
        });
        assert_eq!(p, vec![[0.0, 0.0], [0.0, 2.0]]);
    }

    #[test]
    fn waveforms_stay_in_range_and_are_periodic() {
        // Every shape is bounded to [-1,1] and repeats each unit cycle.
        for kind in 0..=3 {
            for step in 0..40 {
                let p = step as f32 * 0.1;
                let v = waveform(kind, p);
                assert!((-1.0..=1.0).contains(&v), "wave {kind} at {p} = {v}");
                assert!(
                    (waveform(kind, p) - waveform(kind, p + 1.0)).abs() < 1e-5,
                    "wave {kind} periodic at {p}"
                );
            }
        }
        // Anchor points of the parabolic sine-approximation.
        assert_eq!(waveform(0, 0.0), 0.0);
        assert_eq!(waveform(0, 0.25), 1.0);
        assert_eq!(waveform(0, 0.75), -1.0);
    }

    #[test]
    fn registers_and_resolves() {
        let mut reg = NodeRegistry::new();
        register(&mut reg).unwrap();
        assert!(reg.resolve(MANIFEST.id).is_some());
    }
}
