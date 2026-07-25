#![forbid(unsafe_code)]
//! `value.noise` — the value-domain PRODUCER of a coherent **noise** field: a
//! smooth per-instance random value that varies across instances AND evolves over
//! time (Motion Nodes M2, the value domain — doc 12/69). It is the pure *producer*
//! form of `motion.wiggle` (which writes a transform channel), exactly as
//! `value.lfo` is the producer of `motion.oscillator` — it emits the noise as a
//! **value** on its own socket, to be routed by `motion.drive`, reshaped by
//! `value.curve` / `value.map_range`, or gated by a pulse.
//!
//! **Coherent, not white — the distinction from `value.instance_field`'s Random.**
//! Random is a hash per instance: neighbours are UNCORRELATED (a value that jumps,
//! white noise). Noise samples a continuous field: neighbouring instances read
//! nearby lattice points, so they flow TOGETHER (a smooth gradient across the row
//! that drifts over time) — the "give it life" driver of every motion tool:
//! AE's `wiggle()`, C4D MoGraph's Random(Noise), TouchDesigner's Noise CHOP,
//! Blender's Noise Texture, Houdini's `noise`/`turb`.
//!
//! **The two axes** (doc 69): `frequency` scales the INSTANCE axis (spatial detail
//! across the row — low is a smooth swell, high decorrelates neighbours) and
//! `speed` scales the TIME axis (temporal evolution — 0 freezes the field). `seed`
//! offsets the lattice (a different slice of the same field). `octaves` +
//! `roughness` are the fBm knobs (fractal detail, Blender's Detail/Roughness) —
//! octaves 1 is a single layer, the SAME field a `motion.wiggle` sample reads.
//! `amplitude` scales and `offset` shifts the result.
//!
//! `value_i = fbm(t·speed, i·frequency + seed, octaves, roughness) · amplitude +
//! offset`, with `fbm ∈ [-1, 1]` (normalized by the octave-weight sum, so adding
//! detail never grows the range). See [`noise`].
//!
//! **The value type** is the continuous per-instance scalar field `(Instances,
//! Scalar, Frame)` on the `v` column (doc 12). Cardinality follows the geometry:
//! the optional `in` port is read for its **count only** (like `value.lfo`) —
//! unconnected → a length-1 field (one global wiggle, held across every instance
//! by `motion.drive`'s broadcast rule). Reads the playhead, holds no state →
//! `Effect::Temporal` (pull-side, like the LFO). Transcendental-free (HR-5); the
//! GPU kernel is the WGSL port of the same lattice + fade + fBm, so the node is
//! **device-resident** — it cooks on the GPU, no CPU fallback.

use ph2d_node_registry::{NodeRegistry, RegistryError};
use ph2d_nodegraph::attr::{Column, Stream};
use ph2d_nodegraph::cook::EvalCtx;
use ph2d_nodegraph::effect::Effect;
use ph2d_nodegraph::gpu::{ColumnAccess, ColumnBinding, CountLawCtx, GpuKernel, SourceWindow};
use ph2d_nodegraph::node::{LoweringKind, NodeManifest, NodeOp, NodeTypeId, ParamSpec, PortSpec};
use ph2d_nodegraph::port::{Clock, Dim, Domain, PortType};

mod noise;
use noise::fbm_2d;

/// The instance stream type — the optional `in` port, read for its count only.
const INST_VEC2: PortType = PortType::new(Domain::Instances, Dim::Vec2, Clock::Frame);
/// The value type — the continuous per-instance scalar field on the `v` column
/// (mirror of `ph2d_node_value_lfo::VALUE`; kept local so this stays a leaf
/// drop-crate — the shared vocabulary is the port, not a shared symbol).
pub const VALUE: PortType = PortType::new(Domain::Instances, Dim::Scalar, Clock::Frame);

/// The value output column (the canonical `value`-domain column).
const VALUE_COL: &str = "v";

/// The static contract of this node type (ADR-0031). The kernel is side-metadata
/// (ADR-0126); `NodeManifest` stays the frozen 8 fields.
pub const MANIFEST: NodeManifest = NodeManifest {
    id: NodeTypeId::of("value.noise"),
    name: "value.noise",
    inputs: &[PortSpec {
        name: "in",
        ty: INST_VEC2,
    }],
    outputs: &[PortSpec {
        name: "out",
        ty: VALUE,
    }],
    // Reads the playhead → pull-side; the noise is nonetheless deterministic
    // (a pure function of `(i, t, params)`), so scrubbing reproduces it exactly.
    effect: Effect::Temporal,
    clock: Clock::Frame,
    params: &[
        ParamSpec {
            name: "frequency",
            default: 0.2,
        },
        ParamSpec {
            name: "speed",
            default: 0.5,
        },
        ParamSpec {
            name: "octaves",
            default: 1.0,
        },
        ParamSpec {
            name: "roughness",
            default: 0.5,
        },
        ParamSpec {
            name: "amplitude",
            default: 1.0,
        },
        ParamSpec {
            name: "offset",
            default: 0.0,
        },
        ParamSpec {
            name: "seed",
            default: 0.0,
        },
    ],
    lowerings: &[LoweringKind::Cpu],
};

/// The node's knobs, bundled so [`Sample::at`] takes an instance + a time rather
/// than eight arguments. Read once per cook from the [`EvalCtx`].
struct Sample {
    frequency: f32,
    speed: f32,
    octaves: u32,
    roughness: f32,
    amplitude: f32,
    offset: f32,
    seed: f32,
}

impl Sample {
    fn from_ctx(ctx: &mut EvalCtx<'_>) -> Self {
        Self {
            frequency: ctx.param("frequency"),
            speed: ctx.param("speed"),
            // `round().max(1)` mirrors the WGSL `clamp(round(.), 1, 8)`; `fbm_2d`
            // caps at MAX_OCTAVES, so a wild param can never unbound the loop.
            octaves: ctx.param("octaves").round().max(1.0) as u32,
            roughness: ctx.param("roughness"),
            amplitude: ctx.param("amplitude"),
            offset: ctx.param("offset"),
            seed: ctx.param("seed"),
        }
    }

    /// Instance `i`'s value at playhead `t`. `x = t·speed` (the time axis),
    /// `y = i·frequency + seed` (the instance axis), then `fbm·amplitude + offset`.
    fn at(&self, i: u32, t: f32) -> f32 {
        let x = t * self.speed;
        let y = i as f32 * self.frequency + self.seed;
        fbm_2d(x, y, self.octaves, self.roughness) * self.amplitude + self.offset
    }
}

/// GPU compute kernel (ADR-0126) — the WGSL port of [`Sample::at`] + [`noise`],
/// **fully device-resident**. Reads `params.playhead` (the magic uniform every
/// kernel gets, like `value.lfo`). No `applicable` gate — the sequencer never
/// falls back to the CPU for this node (the "maximize GPU" north). The lattice
/// hash + fade are byte-mirrors of `motion.wiggle`'s WGSL (`vn_` ↔ `wg_`), so the
/// two nodes sample the same field; the fBm loop is bounded by a hard `8`
/// (== `noise::MAX_OCTAVES`).
const GPU_KERNEL: GpuKernel = GpuKernel {
    wgsl: "\
        let vn_oct = clamp(i32(vn_round(params.octaves)), 1, 8);\n\
        let vn_x = params.playhead * params.speed;\n\
        let vn_y = f32(i) * params.frequency + params.seed;\n\
        let vn_n = vn_fbm(vn_x, vn_y, vn_oct, params.roughness);\n\
        write_v(i, vn_n * params.amplitude + params.offset);\n",
    wgsl_lib: "\
        fn vn_round(x: f32) -> f32 {\n\
            // Rust f32::round = half away from zero (WGSL round is half-even).\n\
            return select(ceil(x - 0.5), floor(x + 0.5), x >= 0.0);\n\
        }\n\
        fn vn_hash2(ix: i32, iy: i32) -> f32 {\n\
            // Same mix as noise::hash2 — u32 wraps mod 2^32 (== Rust wrapping_*),\n\
            // bitcast<u32> == Rust `as u32` (bit reinterpretation, not a value cast).\n\
            var h: u32 = bitcast<u32>(ix) * 0x27d4eb2du + bitcast<u32>(iy) * 0x165667b1u;\n\
            h = h ^ (h >> 15u);\n\
            h = h * 0x2c1b3c6du;\n\
            h = h ^ (h >> 12u);\n\
            h = h * 0x297175f9u;\n\
            h = h ^ (h >> 15u);\n\
            return (f32(h) / f32(0xffffffffu)) * 2.0 - 1.0;\n\
        }\n\
        fn vn_fade(t: f32) -> f32 {\n\
            return t * t * t * (t * (t * 6.0 - 15.0) + 10.0);\n\
        }\n\
        fn vn_value_noise(x: f32, y: f32) -> f32 {\n\
            let x0 = floor(x);\n\
            let y0 = floor(y);\n\
            let ix = i32(x0);\n\
            let iy = i32(y0);\n\
            let u = vn_fade(x - x0);\n\
            let v = vn_fade(y - y0);\n\
            let n00 = vn_hash2(ix, iy);\n\
            let n10 = vn_hash2(ix + 1, iy);\n\
            let n01 = vn_hash2(ix, iy + 1);\n\
            let n11 = vn_hash2(ix + 1, iy + 1);\n\
            let nx0 = n00 + u * (n10 - n00);\n\
            let nx1 = n01 + u * (n11 - n01);\n\
            return nx0 + v * (nx1 - nx0);\n\
        }\n\
        fn vn_fbm(x: f32, y: f32, oct: i32, rough: f32) -> f32 {\n\
            var sum = 0.0;\n\
            var amp = 1.0;\n\
            var freq = 1.0;\n\
            var norm = 0.0;\n\
            for (var o = 0; o < oct; o = o + 1) {\n\
            \x20   sum = sum + amp * vn_value_noise(x * freq, y * freq);\n\
            \x20   norm = norm + amp;\n\
            \x20   amp = amp * rough;\n\
            \x20   freq = freq * 2.0;\n\
            }\n\
            return sum / max(norm, 1e-6);\n\
        }\n",
    bindings: &[ColumnBinding {
        column: VALUE_COL,
        dim: Dim::Scalar,
        access: ColumnAccess::Write,
        identity: [0.0; 4],
        port: 0,
    }],
    params: &[
        "frequency",
        "speed",
        "octaves",
        "roughness",
        "amplitude",
        "offset",
        "seed",
    ],
    count_law: Some(noise_count),
    variant_by_param: None,
    applicable: None,
};

/// **How wide is the field?** — the same law `value.lfo`/`value.instance_field`
/// use: connected, one value per instance; **unconnected, ONE global value** (held
/// across every instance by `motion.drive`'s broadcast). The engine's default —
/// "as wide as port 0" — sizes an unconnected one at 0 and SKIPS the stage.
fn noise_count(c: &CountLawCtx<'_>) -> SourceWindow {
    SourceWindow::of_count(c.inputs.first().copied().unwrap_or(0).max(1) as usize)
}

struct ValueNoise;

impl NodeOp for ValueNoise {
    fn manifest(&self) -> &'static NodeManifest {
        &MANIFEST
    }

    fn eval(&self, ctx: &mut EvalCtx<'_>) {
        let s = Sample::from_ctx(ctx);
        let t = ctx.playhead() as f32;
        // Cardinality follows the geometry; unconnected → one degenerate value.
        let n = ctx.input(0).count().max(1);
        let v: Vec<f32> = (0..n as u32).map(|i| s.at(i, t)).collect();
        ctx.emit(Stream::new(n).with(VALUE_COL, Column::Scalar(v)));
    }
}

/// Register this node with the runtime registry. Called (via codegen) from
/// `ph2d-node-registry-init::register_all_nodes`.
pub fn register(reg: &mut NodeRegistry) -> Result<(), RegistryError> {
    reg.register(Box::new(ValueNoise))?;
    reg.register_gpu_kernel(MANIFEST.id, GPU_KERNEL);
    reg.register_ui(
        MANIFEST.id,
        ph2d_node_registry::NodeUiManifest {
            display_name: "Noise",
            // Utility grey: a value producer, plumbing (not a transform).
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
        param: "frequency",
        label: "Frequency",
        min: 0.0,
        max: 4.0,
        step: 0.01,
        widget: ParamWidget::Slider,
    },
    ParamUiHint {
        param: "speed",
        label: "Speed",
        min: 0.0,
        max: 8.0,
        step: 0.01,
        widget: ParamWidget::Slider,
    },
    ParamUiHint {
        param: "octaves",
        label: "Octaves",
        min: 1.0,
        max: 8.0,
        step: 1.0,
        widget: ParamWidget::Slider,
    },
    ParamUiHint {
        param: "roughness",
        label: "Roughness",
        min: 0.0,
        max: 1.0,
        step: 0.01,
        widget: ParamWidget::Slider,
    },
    ParamUiHint {
        param: "amplitude",
        label: "Amplitude",
        min: 0.0,
        max: 100.0,
        step: 0.05,
        widget: ParamWidget::Slider,
    },
    ParamUiHint {
        param: "offset",
        label: "Offset",
        min: -100.0,
        max: 100.0,
        step: 0.05,
        widget: ParamWidget::Slider,
    },
    ParamUiHint {
        param: "seed",
        label: "Seed",
        min: 0.0,
        max: 1000.0,
        step: 1.0,
        widget: ParamWidget::Slider,
    },
];

#[cfg(test)]
mod tests {
    use super::*;
    use ph2d_nodegraph::cook::{Cook, OpResolver};
    use ph2d_nodegraph::graph::{Edge, Graph};

    /// A default-ish sampler for the row tests (frequency low = a smooth swell).
    fn smooth() -> Sample {
        Sample {
            frequency: 0.1,
            speed: 0.5,
            octaves: 1,
            roughness: 0.5,
            amplitude: 1.0,
            offset: 0.0,
            seed: 0.0,
        }
    }

    /// THE falsification: the field is COHERENT, not white. At a low frequency
    /// adjacent instances read nearby lattice points, so the mean step between
    /// neighbours is SMALL; raise the frequency past one lattice unit per instance
    /// and neighbours DECORRELATE (a large step). A regression to white noise (a
    /// per-instance hash, like `instance_field` Random) would fail the low-freq
    /// half — its neighbour step is ~2/3 of the full range, always.
    #[test]
    fn the_field_is_coherent_not_white() {
        let n = 24u32;
        let mean_step = |freq: f32| {
            let s = Sample {
                frequency: freq,
                ..smooth()
            };
            let row: Vec<f32> = (0..n).map(|i| s.at(i, 0.0)).collect();
            let total: f32 = row.windows(2).map(|w| (w[1] - w[0]).abs()).sum();
            total / (n - 1) as f32
        };
        let coherent = mean_step(0.1); // 10 instances per feature → smooth
        let decorrelated = mean_step(3.0); // 3 units apart → white-ish
        assert!(
            coherent < 0.15,
            "low frequency must be smooth, got mean step {coherent}"
        );
        assert!(
            coherent > 0.0,
            "but not constant — it is still a varying field"
        );
        assert!(
            decorrelated > 2.0 * coherent,
            "high frequency decorrelates: {decorrelated} vs {coherent}"
        );
    }

    /// The field EVOLVES over time (the `wiggle`/CHOP-translate behaviour): the
    /// same instance reads a different value at a different playhead when speed > 0.
    #[test]
    fn time_evolves_the_field() {
        let s = smooth();
        assert_ne!(s.at(5, 0.0), s.at(5, 2.0), "speed > 0 drifts the field");
    }

    /// Speed 0 FREEZES the field — a static per-instance coherent random,
    /// independent of the playhead (the degenerate case, and a useful one).
    #[test]
    fn speed_zero_freezes_the_field() {
        let s = Sample {
            speed: 0.0,
            ..smooth()
        };
        for i in 0..24 {
            assert_eq!(s.at(i, 0.0), s.at(i, 7.5), "speed 0 is time-invariant");
        }
    }

    /// The output is bounded by `|amplitude| + |offset|` (fBm ∈ [-1,1]): a value
    /// stream downstream never sees a runaway magnitude, whatever the octaves.
    #[test]
    fn the_output_is_bounded_by_amplitude_and_offset() {
        let s = Sample {
            octaves: 8,
            amplitude: 4.0,
            offset: 10.0,
            ..smooth()
        };
        for i in 0..200 {
            let v = s.at(i, i as f32 * 0.3);
            assert!(v.is_finite(), "finite at {i}");
            assert!((6.0..=14.0).contains(&v), "within offset±amplitude: {v} at {i}");
        }
    }

    /// A value source emitting an N-wide instance stream, so `value.noise` can be
    /// driven for its COUNT through a real cook.
    static SRC_MAN: NodeManifest = NodeManifest {
        id: NodeTypeId::of("value.noise.test.src"),
        name: "value.noise.test.src",
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
    struct Src(usize);
    impl NodeOp for Src {
        fn manifest(&self) -> &'static NodeManifest {
            &SRC_MAN
        }
        fn eval(&self, ctx: &mut EvalCtx<'_>) {
            // A vec2 `P` column of length N — the noise reads it for count only.
            ctx.emit(Stream::new(self.0).with("P", Column::Vec2(vec![[0.0, 0.0]; self.0])));
        }
    }

    struct Ops(usize);
    impl OpResolver for Ops {
        fn resolve(&self, ty: NodeTypeId) -> Option<&dyn NodeOp> {
            match ty {
                t if t == SRC_MAN.id => Some(Box::leak(Box::new(Src(self.0))) as &dyn NodeOp),
                t if t == MANIFEST.id => Some(&ValueNoise),
                _ => None,
            }
        }
    }

    /// End-to-end through the cook: connected to a length-8 stream it emits a
    /// length-8 field (cardinality follows the geometry) of finite values, and the
    /// values match `Sample::at` (the eval reaches the same math the tests probe).
    #[test]
    fn emits_a_length_n_field_through_the_cook() {
        let ops = Ops(8);
        let mut g = Graph::new();
        let src = g.add_node("value.noise.test.src");
        let vn = g.add_node("value.noise");
        g.set_param(vn, "frequency", 0.1);
        g.connect(Edge {
            from: (src, 0),
            to: (vn, 0),
            delayed: false,
        })
        .unwrap();
        let mut cook = Cook::new();
        let out = cook.cook(&g, &ops, vn, 3.0).unwrap();
        match out[0].as_stream().get(VALUE_COL).unwrap() {
            Column::Scalar(v) => {
                assert_eq!(v.len(), 8, "cardinality follows the length-8 stream");
                let s = Sample {
                    frequency: 0.1,
                    ..Sample::from_ctx_defaults()
                };
                for (i, &got) in v.iter().enumerate() {
                    assert!(got.is_finite(), "finite at {i}");
                    assert_eq!(got, s.at(i as u32, 3.0), "eval == Sample::at at {i}");
                }
            }
            _ => panic!("v"),
        }
    }

    /// Unconnected, the field is ONE global value (the count law's `max(_, 1)`) —
    /// not the zero-count stage the engine's default would skip.
    #[test]
    fn an_unconnected_noise_is_one_global_value() {
        let ops = Ops(0);
        let mut g = Graph::new();
        let vn = g.add_node("value.noise");
        let mut cook = Cook::new();
        let out = cook.cook(&g, &ops, vn, 0.0).unwrap();
        match out[0].as_stream().get(VALUE_COL).unwrap() {
            Column::Scalar(v) => assert_eq!(v.len(), 1, "one global oscillation"),
            _ => panic!("v"),
        }
    }

    #[test]
    fn registers_and_resolves() {
        let mut reg = NodeRegistry::new();
        register(&mut reg).unwrap();
        assert!(reg.resolve(MANIFEST.id).is_some());
    }

    impl Sample {
        /// The MANIFEST defaults, for tests that assert the eval path matches the
        /// direct sampler (only `frequency` is overridden in the cook test).
        fn from_ctx_defaults() -> Self {
            Self {
                frequency: 0.2,
                speed: 0.5,
                octaves: 1,
                roughness: 0.5,
                amplitude: 1.0,
                offset: 0.0,
                seed: 0.0,
            }
        }
    }
}
