#![forbid(unsafe_code)]
//! `value.wave` — the value-domain waveform SHAPER: map a field, read as a PHASE,
//! through a periodic waveform (Motion Nodes M2, the value domain — doc 12/84). It
//! is the pure SHAPER dual of `value.lfo`: where the LFO *produces* an oscillation
//! from the PLAYHEAD (`Temporal`), this *shapes* any input field into a waveform
//! (`Pure`) — the phase comes from the wire, not the clock. Feed a
//! `value.instance_field` Ramp and it draws a spatial STANDING WAVE across the grid
//! (a sine ripple of dots); feed a `value.time` and you have rebuilt the LFO from
//! primitives (`time → wave == lfo`, a device gate proves it). It is the Cavalry
//! Wave / TouchDesigner Pattern CHOP, made a value shaper.
//!
//! **Not `value.wrap`.** The wrap FOLDS a value into a range (an address mode,
//! output in `[min,max]`); this maps a phase to a bipolar OSCILLATOR waveform
//! (output `[-1,1]·amplitude + offset`, centred on `offset`). A wrap-Mirror of a
//! ramp is a unipolar triangle in the range; a wave-Triangle is the classic
//! `±1` oscillator triangle — different shapes, different jobs.
//!
//! **`wave`** picks the shape (Sine · Tri · Square · Saw · Spike, `wave.rs`, the
//! same transcendental-free bank as the LFO). **`frequency`** scales the input
//! phase (a `[0,1]` ramp at frequency 3 → three cycles across the field);
//! **`amplitude`**/**`offset`** scale/shift the output; **`phase`** advances the
//! cycle. `out[i] = waveform(wave, in[i]·frequency + phase)·amplitude + offset`.
//!
//! **The value type** is the continuous per-instance scalar field `(Instances,
//! Scalar, Frame)` on the `v` column (doc 12). `Pure` (no clock — the phase is the
//! input); length preserved. Transcendental-free (HR-5), so **device-resident**
//! (no CPU fallback) with the existing kernel channel.

use ph2d_node_registry::{NodeRegistry, RegistryError};
use ph2d_nodegraph::attr::{Column, Stream};
use ph2d_nodegraph::cook::EvalCtx;
use ph2d_nodegraph::effect::Effect;
use ph2d_nodegraph::gpu::{ColumnAccess, ColumnBinding, GpuKernel};
use ph2d_nodegraph::node::{LoweringKind, NodeManifest, NodeOp, NodeTypeId, ParamSpec, PortSpec};
use ph2d_nodegraph::port::{Clock, Dim, Domain, PortType};

mod wave;
use wave::waveform;

/// The value type — the continuous per-instance scalar field on the `v` column
/// (mirror of the sibling value nodes; kept local so this stays a leaf drop-crate
/// — the shared vocabulary is the port, not a shared symbol).
pub const VALUE: PortType = PortType::new(Domain::Instances, Dim::Scalar, Clock::Frame);

/// The value column, in (phase) and out (waveform) — the canonical value column.
const VALUE_COL: &str = "v";

/// The static contract of this node type (ADR-0031).
pub const MANIFEST: NodeManifest = NodeManifest {
    id: NodeTypeId::of("value.wave"),
    name: "value.wave",
    inputs: &[PortSpec {
        name: "in",
        ty: VALUE,
    }],
    outputs: &[PortSpec {
        name: "out",
        ty: VALUE,
    }],
    effect: Effect::Pure,
    clock: Clock::Frame,
    params: &[
        // Shape — 0 Sine (parabolic) · 1 Tri · 2 Square · 3 Saw · 4 Spike.
        ParamSpec {
            name: "wave",
            default: 0.0,
        },
        // Scales the input phase (cycles per input unit): a [0,1] ramp at
        // frequency N draws N cycles across the field.
        ParamSpec {
            name: "frequency",
            default: 1.0,
        },
        // Peak of the oscillation (value-native units).
        ParamSpec {
            name: "amplitude",
            default: 1.0,
        },
        // A DC shift of the oscillation centre.
        ParamSpec {
            name: "offset",
            default: 0.0,
        },
        // A global phase offset (cycles) — where in the cycle it starts.
        ParamSpec {
            name: "phase",
            default: 0.0,
        },
    ],
    lowerings: &[LoweringKind::Cpu],
};

/// GPU compute kernel (ADR-0126) — the WGSL port of [`waveform`], **fully
/// device-resident**. No `applicable` — no CPU fallback. VALUE in (the phase),
/// VALUE out (the waveform); binding `ReadWrite` (reads the phase off `in_v`,
/// writes a fresh `out_v`). `vw_round` matches Rust's `f32::round` (`wave` picks a
/// BRANCH). `vw_wave` is the byte-for-byte port of `value.lfo`'s `lfo_wave` (the
/// leaf-local copy of the shape); the `time → wave == lfo` device gate straddles
/// the two copies so they cannot drift.
const GPU_KERNEL: GpuKernel = GpuKernel {
    wgsl: "\
        let vw_kind = i32(vw_round(params.wave));\n\
        let vw_phase = read_v(i) * params.frequency + params.phase;\n\
        let vw_o = vw_wave(vw_kind, vw_phase) * params.amplitude + params.offset;\n\
        write_v(i, vw_o);\n",
    wgsl_lib: "\
        fn vw_round(x: f32) -> f32 {\n\
            // Rust f32::round = half away from zero (WGSL round is half-even).\n\
            return select(ceil(x - 0.5), floor(x + 0.5), x >= 0.0);\n\
        }\n\
        fn vw_wave(kind: i32, phase: f32) -> f32 {\n\
            let f = phase - floor(phase);\n\
            if (kind == 1) {\n\
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
            // Parabolic sine + Capens 2nd-order correction (HR-5, no sin).\n\
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
    bindings: &[ColumnBinding {
        column: VALUE_COL,
        dim: Dim::Scalar,
        access: ColumnAccess::ReadWrite,
        identity: [0.0; 4],
        port: 0,
    }],
    params: &["wave", "frequency", "amplitude", "offset", "phase"],
    count_law: None,
    variant_by_param: None,
    applicable: None,
};

struct ValueWave;

impl NodeOp for ValueWave {
    fn manifest(&self) -> &'static NodeManifest {
        &MANIFEST
    }

    fn eval(&self, ctx: &mut EvalCtx<'_>) {
        let kind = ctx.param("wave").round() as i32;
        let frequency = ctx.param("frequency");
        let amplitude = ctx.param("amplitude");
        let offset = ctx.param("offset");
        let phase0 = ctx.param("phase");
        let input: Vec<f32> = match ctx.input(0).get(VALUE_COL) {
            Some(Column::Scalar(v)) => v.clone(),
            _ => Vec::new(),
        };
        let n = input.len();
        // Unary map — the field's length is preserved exactly.
        let out: Vec<f32> = input
            .iter()
            .map(|&v| waveform(kind, v * frequency + phase0) * amplitude + offset)
            .collect();
        ctx.emit(Stream::new(n).with(VALUE_COL, Column::Scalar(out)));
    }
}

/// Register this node with the runtime registry. Called (via codegen) from
/// `ph2d-node-registry-init::register_all_nodes`.
pub fn register(reg: &mut NodeRegistry) -> Result<(), RegistryError> {
    reg.register(Box::new(ValueWave))?;
    reg.register_gpu_kernel(MANIFEST.id, GPU_KERNEL);
    reg.register_ui(
        MANIFEST.id,
        ph2d_node_registry::NodeUiManifest {
            display_name: "Wave",
            // Utility grey: a value->value transformer, plumbing (not a transform).
            category: ph2d_node_registry::NodeUiCategory::Utility,
            silhouette: ph2d_node_registry::NodeSilhouette::Rect,
        },
    );
    reg.register_param_ui(MANIFEST.id, PARAM_HINTS);
    Ok(())
}

use ph2d_node_registry::{ParamUiHint, ParamWidget};

static PARAM_HINTS: &[ParamUiHint] = &[
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
        param: "frequency",
        label: "Frequency",
        min: 0.0,
        max: 16.0,
        step: 0.05,
        widget: ParamWidget::Slider,
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
    use ph2d_nodegraph::cook::{Cook, OpResolver};
    use ph2d_nodegraph::graph::{Edge, Graph};

    /// **A ramp phase draws the waveform** — feeding phases `0, ¼, ½, ¾` through the
    /// Sine shape gives the anchor points `0, +A, 0, −A` (amplitude scales, offset
    /// shifts). The input IS the phase; frequency 1 means one cycle per unit.
    #[test]
    fn a_ramp_phase_draws_the_waveform() {
        // Sine (wave 0), amplitude 2, offset 1: phase 0→1, ¼→3, ½→1, ¾→−1.
        let shape = |v: f32| waveform(0, v * 1.0 + 0.0) * 2.0 + 1.0;
        assert_eq!(shape(0.0), 1.0, "phase 0 -> offset");
        assert_eq!(shape(0.25), 3.0, "phase 1/4 -> offset + amplitude");
        assert!((shape(0.5) - 1.0).abs() < 1e-6, "phase 1/2 -> offset");
        assert_eq!(shape(0.75), -1.0, "phase 3/4 -> offset - amplitude");
    }

    /// **`frequency` sets the spatial period** — a `[0,1]` ramp at frequency 2
    /// completes TWO cycles, so it returns to the cycle start (phase 0's value) at
    /// phase 0.5 (input) as well as at 0 and 1.
    #[test]
    fn frequency_sets_the_number_of_cycles() {
        let at = |v: f32, freq: f32| waveform(3, v * freq + 0.0); // Saw, easy to read
        // Saw at frequency 2: input 0 → phase 0 → −1; input 0.5 → phase 1 → frac 0 → −1.
        assert_eq!(at(0.0, 2.0), -1.0, "input 0 starts the saw");
        assert!((at(0.5, 2.0) - (-1.0)).abs() < 1e-6, "input 1/2 restarts the saw (freq 2)");
        // At frequency 1 the same input ½ is mid-cycle (saw at phase ½ → 0).
        assert!((at(0.5, 1.0) - 0.0).abs() < 1e-6, "input 1/2 is mid-cycle at freq 1");
    }

    /// **The output is the bipolar band `[offset − A, offset + A]`** for any input,
    /// shape and params (Spike is unipolar `[offset, offset + A]`), and always
    /// finite — the oscillator range, distinct from a range-fold.
    #[test]
    fn output_stays_in_the_amplitude_band() {
        for &kind in &[0.0, 1.0, 2.0, 3.0, 4.0] {
            for k in -50..50 {
                let v = k as f32 * 0.13;
                let o = waveform(kind as i32, v * 2.3 + 0.1) * 1.5 + 0.5;
                assert!(o.is_finite(), "finite at v={v} kind={kind}");
                assert!((0.5 - 1.5 - 1e-4..=0.5 + 1.5 + 1e-4).contains(&o), "in band at v={v} kind={kind}");
            }
        }
    }

    /// A value source emitting a fixed field, so `value.wave` can be driven through
    /// a real cook (the whole-chain proof, not just the math).
    static SRC_MAN: NodeManifest = NodeManifest {
        id: NodeTypeId::of("value.wave.test.src"),
        name: "value.wave.test.src",
        inputs: &[],
        outputs: &[PortSpec {
            name: "out",
            ty: VALUE,
        }],
        effect: Effect::Pure,
        clock: Clock::Frame,
        params: &[],
        lowerings: &[LoweringKind::Cpu],
    };
    struct Src(Vec<f32>);
    impl NodeOp for Src {
        fn manifest(&self) -> &'static NodeManifest {
            &SRC_MAN
        }
        fn eval(&self, ctx: &mut EvalCtx<'_>) {
            ctx.emit(Stream::new(self.0.len()).with(VALUE_COL, Column::Scalar(self.0.clone())));
        }
    }

    /// End-to-end through the cook: a `[0, 0.25, 0.5, 0.75]` phase ramp through the
    /// Sine shape becomes the `[0, 1, 0, -1]` waveform, length preserved.
    #[test]
    fn shapes_a_phase_field_through_the_cook() {
        struct Ops(Vec<f32>);
        impl OpResolver for Ops {
            fn resolve(&self, ty: NodeTypeId) -> Option<&dyn NodeOp> {
                match ty {
                    t if t == SRC_MAN.id => {
                        Some(Box::leak(Box::new(Src(self.0.clone()))) as &dyn NodeOp)
                    }
                    t if t == MANIFEST.id => Some(&ValueWave),
                    _ => None,
                }
            }
        }
        let ops = Ops(vec![0.0, 0.25, 0.5, 0.75]);
        let mut g = Graph::new();
        let src = g.add_node("value.wave.test.src");
        let vw = g.add_node("value.wave");
        g.set_param(vw, "wave", 0.0); // Sine
        g.connect(Edge {
            from: (src, 0),
            to: (vw, 0),
            delayed: false,
        })
        .unwrap();
        let mut cook = Cook::new();
        let out = cook.cook(&g, &ops, vw, 0.0).unwrap();
        match out[0].as_stream().get(VALUE_COL).unwrap() {
            Column::Scalar(v) => {
                assert_eq!(v.len(), 4, "length preserved");
                assert!((v[0] - 0.0).abs() < 1e-6, "phase 0 -> 0");
                assert!((v[1] - 1.0).abs() < 1e-6, "phase 1/4 -> +1");
                assert!((v[2] - 0.0).abs() < 1e-6, "phase 1/2 -> 0");
                assert!((v[3] - (-1.0)).abs() < 1e-6, "phase 3/4 -> -1");
            }
            _ => panic!("v"),
        }
    }

    #[test]
    fn registers_and_resolves() {
        let mut reg = NodeRegistry::new();
        register(&mut reg).unwrap();
        assert!(reg.resolve(MANIFEST.id).is_some());
    }
}
