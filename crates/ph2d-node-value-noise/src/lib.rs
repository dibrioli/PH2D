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
use ph2d_nodegraph::node::{LoweringKind, NodeManifest, NodeOp, NodeTypeId, ParamSpec, PortSpec};
use ph2d_nodegraph::port::{Clock, Dim, Domain, PortType};

mod kernel;
mod noise;
use kernel::GPU_KERNEL;
use noise::{CellFeature, Kernel, fbm_2d};

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
        // ⚠️ **Apendado**: `0` = Index (o nó que sempre shipou), `1` = World.
        ParamSpec {
            name: "space",
            default: 0.0,
        },
        ParamSpec {
            name: "seed",
            default: 0.0,
        },
        // ⚠️ **Apendado, e o nome NÃO é `type`**: o `motion.noise` já gastou essa
        // palavra na LEI fractal (fBm/Turbulence/Ridged) e aqui a pergunta é o
        // RUÍDO DE BASE. `0` = Value (o que sempre shipou) · `1` = Perlin ·
        // `2` = Cellular.
        ParamSpec {
            name: "kernel",
            default: 0.0,
        },
        // Só o Cellular os lê — o `ParamGate` esconde-os nos outros dois.
        ParamSpec {
            name: "feature",
            default: 0.0,
        },
        ParamSpec {
            name: "jitter",
            default: 1.0,
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
    kernel: Kernel,
    feature: CellFeature,
    jitter: f32,
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
            kernel: Kernel::from_index(ctx.param("kernel")),
            feature: CellFeature::from_index(ctx.param("feature")),
            jitter: ctx.param("jitter"),
        }
    }

    /// **A porta única do campo** — as duas leituras (fila e espaço) diferem só
    /// no PONTO que amostram, nunca no ruído que somam. Duas cópias divergiriam
    /// no dia em que um kernel novo entrasse, e o modo World é exactamente onde
    /// ninguém olha.
    fn field(&self, x: f32, y: f32) -> f32 {
        let (k, feat, j) = (self.kernel, self.feature, self.jitter);
        fbm_2d(x, y, self.octaves, self.roughness, |px, py| {
            noise::base(k, feat, j, px, py)
        }) * self.amplitude
            + self.offset
    }

    /// Instance `i`'s value at playhead `t`. `x = t·speed` (the time axis),
    /// `y = i·frequency + seed` (the instance axis), then `fbm·amplitude + offset`.
    fn at(&self, i: u32, t: f32) -> f32 {
        let x = t * self.speed;
        let y = i as f32 * self.frequency + self.seed;
        self.field(x, y)
    }

    /// **O valor no PONTO `(px, py)`** — o mesmo campo, amostrado no espaço em vez
    /// de na fila (doc 89 folha 10, o P0 do `field.noise`).
    ///
    /// ⚠️ **Sem isto o campo procedural era inexprimível, e o motivo era ESTE.** A
    /// célula nomeava dois bloqueios: (1) *"o `drive` não tem canal `falloff`"* —
    /// **morreu**, o canal existe e escreve a coluna — e (2) *"o `value.noise` é
    /// indexado, não tem `P`, logo não é um campo ESPACIAL"*. Com o eixo espacial,
    /// `value.noise(space = World) → motion.drive(channel = Falloff)` **é** o campo
    /// de ruído: composição, que é o desenho desta biblioteca, e não um nó novo.
    ///
    /// ⚠️ **O TEMPO continua no mesmo eixo `x`**, somado à posição em vez de a
    /// substituí-la: um campo espacial que congelasse ao ganhar `P` perderia o
    /// *"animável"* que a referência (MOPs Noise Falloff) pede no mesmo fôlego.
    fn at_world(&self, px: f32, py: f32, t: f32) -> f32 {
        let x = px * self.frequency + t * self.speed;
        let y = py * self.frequency + self.seed;
        self.field(x, y)
    }
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
        let world = ctx.param("space").round() as i32 == 1;
        let input = ctx.input(0);
        let n = input.count().max(1);
        let pos = match input.get("P") {
            Some(Column::Vec2(v)) if world => Some(v.clone()),
            _ => None,
        };
        let v: Vec<f32> = (0..n)
            .map(|i| match pos.as_ref().and_then(|p| p.get(i)) {
                // ⚠️ Sem coluna `P` o modo World CAI no índice, e não em zero: um
                // stream sem posição não tem espaço a amostrar, e um campo que
                // colapsasse num valor só leria como "o ruído morreu".
                Some(p) => s.at_world(p[0], p[1], t),
                None => s.at(i as u32, t),
            })
            .collect();
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
    reg.register_param_hard_max(MANIFEST.id, PARAM_HARD_MAX);
    reg.register_param_gates(MANIFEST.id, PARAM_GATES);
    Ok(())
}

use ph2d_node_registry::{ParamGate, ParamHardMax, ParamUiHint, ParamWidget};

/// **Os dois knobs do celular só existem no celular** (`kernel == 2`). Um
/// controlo que não faz nada é pior que um controlo que falta — e aqui a
/// alternativa é literal: `feature` e `jitter` não são lidos por nenhum dos
/// outros dois kernels (`noise::base` ramifica antes deles).
static PARAM_GATES: &[ParamGate] = &[
    ParamGate {
        param: "feature",
        when: "kernel",
        values: &[2],
    },
    ParamGate {
        param: "jitter",
        when: "kernel",
        values: &[2],
    },
];

/// O teto que a MÁQUINA (ou o bom senso) impõe, alcançável por DIGITAÇÃO — o slider fica
/// onde a MÃO trabalha (soft/hard do Blender; doc 88 §11). O curso de antes é este número:
/// nada ficou inalcançável, só deixou de ser o que o dedo percorre.
static PARAM_HARD_MAX: &[ParamHardMax] = &[
    ParamHardMax {
        param: "amplitude",
        max: 100.0,
    },
    ParamHardMax {
        param: "frequency",
        max: 4.0,
    },
];

static PARAM_HINTS: &[ParamUiHint] = &[
    // ⚠️ **Primeiro de todos**, porque é a maior decisão visual do nó: que ruído
    // de base a lei fractal soma. `Pattern` e não `Type` — o `motion.noise` já
    // gasta `Type` na LEI (fBm/Turbulence/Ridged), e a mesma palavra a
    // selecionar coisas diferentes em dois nós irmãos é o que faz um menu mentir.
    ParamUiHint {
        param: "kernel",
        label: "Pattern",
        min: 0.0,
        max: 2.0,
        step: 1.0,
        widget: ParamWidget::Enum {
            labels: &["Value", "Perlin", "Cellular"],
        },
    },
    // Os dois seguintes são gateados no Cellular (ver `PARAM_GATES`).
    ParamUiHint {
        param: "feature",
        label: "Cell",
        min: 0.0,
        max: 1.0,
        step: 1.0,
        widget: ParamWidget::Enum {
            labels: &["Cells", "Cracks"],
        },
    },
    ParamUiHint {
        param: "jitter",
        label: "Jitter",
        min: 0.0,
        max: 1.0,
        step: 0.01,
        widget: ParamWidget::Slider,
    },
    // ⚠️ **Depois**, e é o que decide o que o nó SIGNIFICA: o mesmo ruído lido
    // ao longo da FILA (o de sempre) ou no ESPAÇO. É o modo World que faz
    // `value.noise → motion.drive(Falloff)` ser um campo procedural (doc 89
    // folha 10, o P0 do `field.noise`).
    ParamUiHint {
        param: "space",
        label: "Sample",
        min: 0.0,
        max: 1.0,
        step: 1.0,
        widget: ParamWidget::Enum {
            labels: &["Index", "World"],
        },
    },
    ParamUiHint {
        param: "frequency",
        label: "Frequency",
        min: 0.0,
        max: 2.0,
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
        max: 10.0,
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
        widget: ParamWidget::Seed,
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
            // A premissa desta fixture, DECLARADA: ela mede o kernel de VALOR,
            // o que o no sempre shipou. Herda-la de um default e o que faz um
            // teste inverter de sentido quando o default se move.
            kernel: Kernel::Value,
            feature: CellFeature::Cells,
            jitter: 1.0,
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
            assert!(
                (6.0..=14.0).contains(&v),
                "within offset±amplitude: {v} at {i}"
            );
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
                kernel: Kernel::Value,
                feature: CellFeature::Cells,
                jitter: 1.0,
            }
        }
    }
}

#[cfg(test)]
#[path = "space_tests.rs"]
mod space_tests;
