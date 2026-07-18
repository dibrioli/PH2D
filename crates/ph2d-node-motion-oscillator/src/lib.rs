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
//! (unit period) and the shapes are piecewise polynomial. The "Sine" wave is a
//! parabolic approximation with a 2nd-order correction (Capens/devmaster) — ~0.09%
//! off a true sine using only multiply + abs — since a real `sin` is
//! non-deterministic (plan §1.7).
//!
//! Params (read via `ctx.param`):
//! - `channel` (1): target — `0` X, `1` Y, `2` Rotation, `3` Size.
//! - `wave` (0): shape — `0` Sine (parabolic), `1` Triangle, `2` Square, `3` Saw,
//!   `4` Spike (a narrow unipolar pulse).
//! - `amplitude` (1): peak of the oscillation (channel-native units).
//! - `frequency` (1): cycles per second of playhead.
//! - `phase_stagger` (0.1): per-instance phase offset (cycles) → the travelling wave.
//! - `offset` (0): a DC shift of the oscillation centre.
//! - `phase` (0): a global phase offset (cycles) — where in the cycle it starts.
//!
//! `delta_i = (wave(t·frequency + i·phase_stagger + phase)·amplitude + offset)·falloff_i`.

use ph2d_node_registry::{NodeRegistry, RegistryError};
use ph2d_nodegraph::cook::EvalCtx;
use ph2d_nodegraph::effect::Effect;
use ph2d_nodegraph::gpu::{ColumnAccess, ColumnBinding, GpuKernel};
use ph2d_nodegraph::node::{LoweringKind, NodeManifest, NodeOp, NodeTypeId, ParamSpec, PortSpec};
use ph2d_nodegraph::port::{Clock, Dim, Domain, PortType};

mod channel;
use channel::{apply_channel_delta, falloff_at};
use ph2d_nodegraph::attr::par_build;

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
            default: 0.1,
        },
        ParamSpec {
            name: "offset",
            default: 0.0,
        },
        ParamSpec {
            name: "phase",
            default: 0.0,
        },
    ],
    lowerings: &[LoweringKind::Cpu],
};

/// The fractional part of `p` in `[0,1)` — IEEE `floor` is correctly-rounded and
/// deterministic (HR-5-safe, unlike `sin`).
fn frac(p: f32) -> f32 {
    p - p.floor()
}

/// A periodic waveform at `phase` (in cycles, period 1) — bipolar `[-1,1]` except
/// **Spike** (a unipolar `[0,1]` pulse). All shapes are piecewise polynomial →
/// transcendental-free (HR-5). Unknown / `0` is the parabolic sine-approximation.
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
        4 => {
            // Spike: a narrow unipolar pulse at the cycle start (a periodic kick).
            const SPIKE_WIDTH: f32 = 0.08;
            if f < SPIKE_WIDTH { 1.0 } else { 0.0 }
        }
        _ => {
            // Parabolic sine-approximation: a +hump over [0,½), a −hump over
            // [½,1), each `±4u(1−u)` — continuous, 0 at 0/½, ±1 at ¼/¾.
            let p = if f < 0.5 {
                let u = f * 2.0;
                4.0 * u * (1.0 - u)
            } else {
                let u = (f - 0.5) * 2.0;
                -4.0 * u * (1.0 - u)
            };
            // 2nd-order correction (Capens/devmaster): the bare parabola is ~5.6%
            // off a true sine (visibly rounder at the crest); `0.225·(p·|p|−p)+p`
            // drops that to ~0.09% using only multiply + abs (transcendental-free,
            // HR-5). Endpoint/range-preserving: 0→0, ±1→±1, stays in [-1,1].
            const Q: f32 = 0.225;
            Q * (p * p.abs() - p) + p
        }
    }
}

/// GPU compute kernel (GPU/M5 Fase 1, ADR-0126). The waveform library is a
/// straight WGSL port of [`waveform`] — same piecewise polynomials, same
/// Capens 2nd-order sine correction (HR-5: WGSL `sin` has no cross-vendor
/// guarantee; the parabola is plain mul/abs arithmetic, so GPU-vs-CPU parity
/// holds within float ULPs). `osc_round` is round-half-AWAY-from-zero to match
/// Rust's `f32::round` (WGSL's builtin `round` is half-to-even, which would
/// pick a different waveform at `wave = 2.5`).
///
/// **Covers the X/Y channels only** (`applicable`): the Rotation/Size channels
/// write a different column, which a static binding set cannot switch on — a
/// graph oscillating those falls back to the CPU `eval` (the explicit
/// boundary), never to a wrong answer. The channel test in the body is
/// `< 0.5`, which agrees with the CPU's `round()` for every value `applicable`
/// admits.
const GPU_KERNEL: GpuKernel = GpuKernel {
    wgsl: "\
        let osc_f = read_falloff(i);\n\
        let osc_phase = params.playhead * params.frequency + f32(i) * params.phase_stagger + params.phase;\n\
        let osc_w = osc_wave(i32(osc_round(params.wave)), osc_phase);\n\
        let osc_d = (osc_w * params.amplitude + params.offset) * osc_f;\n\
        var osc_p = read_P(i);\n\
        if (params.channel < 0.5) {\n\
            osc_p.x = osc_p.x + osc_d;\n\
        } else {\n\
            osc_p.y = osc_p.y + osc_d;\n\
        }\n\
        write_P(i, osc_p);\n",
    wgsl_lib: "\
        fn osc_round(x: f32) -> f32 {\n\
            // Rust f32::round = half away from zero (WGSL round is half-even).\n\
            return select(ceil(x - 0.5), floor(x + 0.5), x >= 0.0);\n\
        }\n\
        fn osc_wave(kind: i32, phase: f32) -> f32 {\n\
            let f = phase - floor(phase);\n\
            if (kind == 1) {\n\
                // Triangle: 0 at 0, +1 at 1/4, 0 at 1/2, -1 at 3/4.\n\
                if (f < 0.25) { return 4.0 * f; }\n\
                if (f < 0.75) { return 2.0 - 4.0 * f; }\n\
                return 4.0 * f - 4.0;\n\
            }\n\
            if (kind == 2) {\n\
                if (f < 0.5) { return 1.0; }\n\
                return -1.0;\n\
            }\n\
            if (kind == 3) { return 2.0 * f - 1.0; }\n\
            if (kind == 4) {\n\
                if (f < 0.08) { return 1.0; }\n\
                return 0.0;\n\
            }\n\
            // Parabolic sine-approximation + Capens correction (~0.09% off a\n\
            // true sine, transcendental-free) — the port of `waveform`'s default.\n\
            var p: f32;\n\
            if (f < 0.5) {\n\
                let u = f * 2.0;\n\
                p = 4.0 * u * (1.0 - u);\n\
            } else {\n\
                let u = (f - 0.5) * 2.0;\n\
                p = -4.0 * u * (1.0 - u);\n\
            }\n\
            return 0.225 * (p * abs(p) - p) + p;\n\
        }\n",
    bindings: &[
        // The target channel is materialized from its identity when absent —
        // the CPU's `apply_channel_delta` does the same (`base_vec2`).
        ColumnBinding {
            column: "P",
            dim: Dim::Vec2,
            access: ColumnAccess::ReadWrite,
            identity: [0.0; 4],
            port: 0,
        },
        ColumnBinding {
            column: "falloff",
            dim: Dim::Scalar,
            access: ColumnAccess::Read,
            identity: [1.0; 4],
            port: 0,
        },
    ],
    params: &[
        "channel",
        "wave",
        "amplitude",
        "frequency",
        "phase_stagger",
        "offset",
        "phase",
    ],
    source_window: None,
    applicable: Some(|param| {
        // Same rounding the CPU `eval` applies. Only X (0) / Y (1) write `P`;
        // Rotation (2) / Size (3) write other columns → CPU fallback.
        let channel = param("channel").round();
        channel == 0.0 || channel == 1.0
    }),
};

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
        let offset = ctx.param("offset");
        let phase0 = ctx.param("phase");
        let t = ctx.playhead() as f32;
        let out = {
            let input = ctx.input(0);
            let n = input.count();
            // Pure per-instance map → parallel above the threshold (bit-identical,
            // no reduction). GPU/M5 Fase 0.
            let deltas: Vec<f32> = par_build(n, |i| {
                let phase = t * frequency + i as f32 * phase_stagger + phase0;
                // DC `offset` shifts the oscillation centre; the whole
                // contribution is falloff-masked (like every behaviour).
                (waveform(wave, phase) * amplitude + offset) * falloff_at(input, i)
            });
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
    // GPU/M5 Fase 1 (ADR-0126): the WGSL lowering, registered on the side.
    reg.register_gpu_kernel(MANIFEST.id, GPU_KERNEL);
    Ok(())
}

use ph2d_node_registry::{ParamUiHint, ParamWidget};

/// Param UI hints (M1.P1). `channel` / `wave` are **named** selectors (segmented
/// buttons) — never number sliders. The enum option index IS the param value
/// (channel 0..3; wave 0..4 = Sine/Tri/Square/Saw/Spike — "Sine" is the
/// user-facing name for the transcendental-free parabolic approximation,
/// "Spike" a narrow unipolar pulse at the cycle start).
static PARAM_HINTS: &[ParamUiHint] = &[
    ParamUiHint {
        param: "channel",
        label: "Channel",
        min: 0.0,
        max: 3.0,
        step: 1.0,
        widget: ParamWidget::Enum {
            labels: &["X", "Y", "Rotation", "Size"],
        },
    },
    ParamUiHint {
        param: "wave",
        label: "Wave",
        min: 0.0,
        max: 4.0,
        step: 1.0,
        widget: ParamWidget::Enum {
            labels: &["Sine", "Tri", "Square", "Saw", "Spike"],
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
        label: "Stagger",
        min: 0.0,
        max: 2.0,
        step: 0.02,
        widget: ParamWidget::Slider,
    },
    ParamUiHint {
        param: "offset",
        label: "Offset",
        min: -10.0,
        max: 10.0,
        step: 0.05,
        widget: ParamWidget::Slider,
    },
    ParamUiHint {
        param: "phase",
        label: "Phase",
        min: 0.0,
        max: 1.0,
        step: 0.01,
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
        // Every shape is bounded to [-1,1] (Spike to [0,1]) and repeats per cycle.
        for kind in 0..=4 {
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
        // Anchor points of the corrected sine approximation (preserved).
        assert_eq!(waveform(0, 0.0), 0.0);
        assert_eq!(waveform(0, 0.25), 1.0);
        assert_eq!(waveform(0, 0.75), -1.0);
        // Spike: a narrow unipolar pulse — 1 at the cycle start, 0 through most.
        assert_eq!(waveform(4, 0.0), 1.0);
        assert_eq!(waveform(4, 0.5), 0.0);
    }

    #[test]
    fn offset_shifts_the_centre_and_phase_advances_the_cycle() {
        // A DC `offset` moves the oscillation centre: default Sine at t=0 is 0, so
        // with offset 2 the instances sit at +2.
        let p = osc_y_at(0.0, |g, osc| {
            g.set_param(osc, "phase_stagger", 0.0);
            g.set_param(osc, "offset", 2.0);
        });
        assert_eq!(p, vec![[0.0, 2.0], [0.0, 2.0]]);
        // A global `phase` of ¼ starts the cycle at the peak (like advancing t):
        // amplitude 3, phase ¼ → Δy = +3 at t=0.
        let q = osc_y_at(0.0, |g, osc| {
            g.set_param(osc, "phase_stagger", 0.0);
            g.set_param(osc, "amplitude", 3.0);
            g.set_param(osc, "phase", 0.25);
        });
        assert_eq!(q, vec![[0.0, 3.0], [0.0, 3.0]]);
    }

    #[test]
    fn corrected_sine_beats_the_bare_parabola() {
        use std::f32::consts::TAU;
        // Compare against a true sine (std::sin — test-only, not in the cook) at
        // 64 phases. The corrected wave is well under 0.5% error everywhere and
        // strictly closer than the bare parabola (which peaks ~5.6% off).
        let bare = |f: f32| {
            if f < 0.5 {
                let u = f * 2.0;
                4.0 * u * (1.0 - u)
            } else {
                let u = (f - 0.5) * 2.0;
                -4.0 * u * (1.0 - u)
            }
        };
        let mut worst_corrected = 0.0f32;
        let mut worst_bare = 0.0f32;
        for k in 0..64 {
            let f = k as f32 / 64.0;
            let truth = (f * TAU).sin();
            worst_corrected = worst_corrected.max((waveform(0, f) - truth).abs());
            worst_bare = worst_bare.max((bare(f) - truth).abs());
        }
        assert!(
            worst_corrected < 0.005,
            "corrected worst = {worst_corrected}"
        );
        assert!(worst_bare > 0.04, "bare parabola should be visibly worse");
        assert!(worst_corrected < worst_bare);
    }

    #[test]
    fn registers_and_resolves() {
        let mut reg = NodeRegistry::new();
        register(&mut reg).unwrap();
        assert!(reg.resolve(MANIFEST.id).is_some());
    }
}
