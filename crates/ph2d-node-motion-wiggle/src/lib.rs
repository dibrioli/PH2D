#![forbid(unsafe_code)]
//! `motion.wiggle` — a Motion **behaviour**: adds a smooth pseudo-random offset
//! to a chosen channel, **added** to the existing value and scaled per-instance
//! by the multiplicative `falloff` column (§1.2; absent → `1.0`). The AE-style
//! *wiggle* — each instance samples its own row of a 2D value-noise field scrolled
//! by the playhead, so they jitter organically and independently. Reads the
//! playhead but holds no state → `Effect::Temporal` (pull-side). Every other
//! column passes through unchanged (count preserved).
//!
//! The noise is deterministic / transcendental-free (HR-5): an integer-hash
//! lattice + smootherstep fade + bilinear lerp (see [`noise`]).
//!
//! Params (read via `ctx.param`):
//! - `channel` (1): target — `0` X, `1` Y, `2` Rotation, `3` Size.
//! - `amplitude` (1): peak of the jitter (channel-native units).
//! - `frequency` (0.5): how fast the noise scrolls in playhead-seconds.
//! - `seed` (0): shifts the noise field so several Wiggles differ.
//!
//! `delta_i = value_noise(t·frequency, i + seed) · amplitude · falloff_i`.

use ph2d_node_registry::{
    NodeRegistry, ParamChannelRange, ParamUnit, ParamUnitDecl, RegistryError,
};
use ph2d_nodegraph::cook::EvalCtx;
use ph2d_nodegraph::effect::Effect;
use ph2d_nodegraph::node::{LoweringKind, NodeManifest, NodeOp, NodeTypeId, ParamSpec, PortSpec};
use ph2d_nodegraph::port::{Clock, Dim, Domain, PortType};

mod channel;
mod gpu;
mod noise;
use channel::{apply_channel_delta, clock_at, falloff_at, scalar_values};
use gpu::GPU_KERNEL;
use noise::fbm;

const INST_VEC2: PortType = PortType::new(Domain::Instances, Dim::Vec2, Clock::Frame);
/// O tipo da porta `time` — espelho local do `VALUE` do `motion.drive`. Esta é uma
/// crate-folha: o vocabulário partilhado é a **porta**, nunca um símbolo importado.
const VALUE: PortType = PortType::new(Domain::Instances, Dim::Scalar, Clock::Frame);
/// A coluna que um stream de valor carrega (o que o `value.time` emite).
const VALUE_COL: &str = "v";

/// The static contract of this node type (ADR-0031).
pub const MANIFEST: NodeManifest = NodeManifest {
    id: NodeTypeId::of("motion.wiggle"),
    name: "motion.wiggle",
    inputs: &[
        PortSpec {
            name: "in",
            ty: INST_VEC2,
        },
        // ⚠️ **A PORTA DE TEMPO, e ela é per-ELEMENTO** (folha 06, `SUPERAR 1`).
        // Desligada ⇒ `ctx.playhead()`, **byte-idêntico**; APENDADA, nunca inserida
        // (as arestas de um doc salvo guardam o ÍNDICE da porta).
        //
        // ⚠️ Este nó é o de ÍNDICE (`fbm(t·frequency, i + seed)`, cerca 5), então a
        // porta **não** o transforma no irmão de campo: o que ela dá é o eixo do
        // TEMPO por elemento, e a linha do ruído continua a ser o índice.
        PortSpec {
            name: "time",
            ty: VALUE,
        },
    ],
    outputs: &[PortSpec {
        name: "out",
        ty: INST_VEC2,
    }],
    // Reads the playhead → pull-side; the noise is nonetheless deterministic.
    effect: Effect::Temporal,
    clock: Clock::Frame,
    params: &[
        ParamSpec {
            name: "channel",
            default: 1.0,
        },
        ParamSpec {
            name: "amplitude",
            default: 1.0,
        },
        ParamSpec {
            name: "frequency",
            default: 0.5,
        },
        ParamSpec {
            name: "seed",
            default: 0.0,
        },
        // ⚠️ **A assinatura da própria referência.** O `wiggle()` do After Effects
        // é `wiggle(freq, amp, octaves = 1, amp_mult = 0.5)` — as duas últimas
        // metades faltavam aqui, e é por elas que um wiggle de uma oitava lê como
        // uma onda lenta em vez de um tremor.
        ParamSpec {
            name: "octaves",
            default: 1.0,
        },
        ParamSpec {
            name: "amp_mult",
            default: 0.5,
        },
        // O laço no tempo (Cavalry *Looping + Loop Length*; AE *Fractal Noise ▸
        // Cycle*). `0` = sem laço, e aí a segunda amostra nem é avaliada.
        ParamSpec {
            name: "loop_len",
            default: 0.0,
        },
    ],
    lowerings: &[LoweringKind::Cpu],
};

/// O teto de oitavas — o mesmo do `motion.noise`, e pelo mesmo motivo: o `eval`
/// da folha faz `octaves.max(1)` e **não tem cap**, então quem o dá é o nó (um
/// `f32` não confiável não pode dirigir um laço).
const MAX_OCTAVES: u32 = 8;

struct MotionWiggle;

impl NodeOp for MotionWiggle {
    fn manifest(&self) -> &'static NodeManifest {
        &MANIFEST
    }

    fn eval(&self, ctx: &mut EvalCtx<'_>) {
        let channel = ctx.param("channel").round() as i32;
        let amplitude = ctx.param("amplitude");
        let frequency = ctx.param("frequency");
        let seed = ctx.param("seed");
        let spec = ph2d_fbm::Spec {
            octaves: (ctx.param("octaves").round().max(1.0) as u32).min(MAX_OCTAVES),
            // ⚠️ **A lacunaridade fica em 2 e NÃO vira param aqui.** O `wiggle()`
            // da referência não a tem, e o irmão `motion.noise` já a expõe para
            // quem quer o campo fractal inteiro — dois nós com o mesmo knob é o
            // que faz um artista perguntar qual dos dois manda.
            lacunarity: 2.0,
            roughness: ctx.param("amp_mult"),
            ty: ph2d_fbm::NoiseType::Fbm,
        };
        // ⚠️ **O tempo WRAPA antes de virar coordenada** — a costura mora na folha
        // (`loop_times`), com o porquê de o peso ser smoothstep e não linear.
        // ⚠️ Chamada DENTRO do laço agora: o relógio pode ser um campo, e aí cada
        // elemento fecha o próprio ciclo. Sem porta, os `n` cálculos partem do mesmo
        // número — byte-idêntico ao que se calculava uma vez.
        let playhead = ctx.playhead() as f32;
        let loop_len = ctx.param("loop_len");
        let times = scalar_values(ctx.input(1), VALUE_COL);
        let out = {
            let input = ctx.input(0);
            let n = input.count();
            debug_assert!(
                matches!(times.len(), 0 | 1) || times.len() == n,
                "a porta `time` tem {} valores para {n} instancias",
                times.len()
            );
            let deltas: Vec<f32> = (0..n)
                .map(|i| {
                    let (t_a, t_b, w) =
                        ph2d_fbm::loop_times(clock_at(&times, i, playhead), loop_len);
                    // Each instance = a distinct noise row (`i + seed`), scrolled
                    // by time on the x-axis → independent organic wiggle.
                    let ny = i as f32 + seed;
                    let sample = |tt: f32| fbm(tt * frequency, ny, spec);
                    // `w == 0` é o caminho de sempre: a 2ª amostra nem é avaliada.
                    let s = if w == 0.0 {
                        sample(t_a)
                    } else {
                        let a = sample(t_a);
                        a + (sample(t_b) - a) * w
                    };
                    s * amplitude * falloff_at(input, i)
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
    reg.register(Box::new(MotionWiggle))?;
    // M1.R1 — UI metadata. Behaviours modify transform channels → Transform
    // (blue) for now; a dedicated Behaviour category (cyan) is a follow-up.
    reg.register_ui(
        MANIFEST.id,
        ph2d_node_registry::NodeUiManifest {
            display_name: "Wiggle",
            category: ph2d_node_registry::NodeUiCategory::Transform,
            silhouette: ph2d_node_registry::NodeSilhouette::Rect,
        },
    );
    reg.register_param_ui(MANIFEST.id, PARAM_HINTS);
    reg.register_param_channel_range(MANIFEST.id, PARAM_CHANNEL_RANGE);
    reg.register_param_units(MANIFEST.id, PARAM_UNITS);
    // GPU/M5 Fase 2 (ADR-0126): the WGSL lowering, registered on the side.
    reg.register_gpu_kernel(MANIFEST.id, GPU_KERNEL);
    Ok(())
}

use ph2d_node_registry::{ParamUiHint, ParamWidget};

/// Param UI hints (M1.P1). `channel` is a named selector; `seed` a seed widget.
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
        param: "seed",
        label: "Seed",
        min: 0.0,
        max: 100.0,
        step: 1.0,
        widget: ParamWidget::Seed,
    },
    ParamUiHint {
        param: "octaves",
        label: "Octaves",
        min: 1.0,
        max: MAX_OCTAVES as f32,
        step: 1.0,
        widget: ParamWidget::IntSlider,
    },
    // ⚠️ **O RÓTULO é o da referência, não o do irmão.** O `motion.noise` chama
    // este número de *Roughness* e o AE, cuja assinatura este nó copia, chama-o
    // de `amp_mult` — quem procura o wiggle procura a palavra do AE.
    ParamUiHint {
        param: "amp_mult",
        label: "Amp Multiplier",
        min: 0.0,
        max: 1.0,
        step: 0.01,
        widget: ParamWidget::Slider,
    },
    ParamUiHint {
        param: "loop_len",
        label: "Loop Length",
        min: 0.0,
        max: 30.0,
        step: 0.1,
        widget: ParamWidget::Slider,
    },
];

/// **What each of this node's numbers IS** (doc 88, Wave A). This node's magnitude
/// is `FromChannel`: it means metres on Position, DEGREES on Rotation and a bare
/// scale factor on Size, so the panel resolves the unit per-channel. Declaring a
/// fixed `Length` here would scale degrees by `pixels_per_meter` — the failure
/// that turns a `±90` preset into a `±9000`.
static PARAM_UNITS: &[ParamUnitDecl] = &[
    ParamUnitDecl {
        param: "amplitude",
        unit: ParamUnit::FromChannel,
    },
    // ⚠️ **Uma DURAÇÃO não muda de unidade com o canal** — o precedente é o
    // `ticks` do `motion.delay`: dois segundos são dois segundos quer o laço
    // feche uma posição, um ângulo ou um tamanho.
    ParamUnitDecl {
        param: "loop_len",
        unit: ParamUnit::Seconds,
    },
];

/// **A faixa que estas magnitudes querem quando o canal é ANGULAR** — graus, não
/// unidades de mundo. Uma volta para cada lado, discada em graus inteiros.
///
/// ⚠️ Ela mora AQUI e não numa tabela do shell porque a tabela apodreceu: medida,
/// ela cobria três dos seis nós que precisavam dela, e cada um dos três ausentes
/// esperava o próprio report do artista.
const TURN: f32 = 360.0;
static PARAM_CHANNEL_RANGE: &[ParamChannelRange] = &[ParamChannelRange {
    param: "amplitude",
    min: 0.0,
    max: TURN,
    step: 1.0,
}];

#[cfg(test)]
#[path = "octave_tests.rs"]
mod octave_tests;

#[cfg(test)]
mod tests {
    use super::*;
    use ph2d_nodegraph::attr::{Column, Stream};
    use ph2d_nodegraph::cook::{Cook, OpResolver};
    use ph2d_nodegraph::graph::{Edge, Graph};

    // Source: 4 instances at the origin (so the wiggle IS the output).
    static SRC_MAN: NodeManifest = NodeManifest {
        id: NodeTypeId::of("motion.wiggle.test.src"),
        name: "motion.wiggle.test.src",
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
            ctx.emit(Stream::new(4).with("P", Column::Vec2(vec![[0.0, 0.0]; 4])));
        }
    }
    struct Ops;
    impl OpResolver for Ops {
        fn resolve(&self, ty: NodeTypeId) -> Option<&dyn NodeOp> {
            match ty {
                t if t == SRC_MAN.id => Some(&Src),
                t if t == MANIFEST.id => Some(&MotionWiggle),
                _ => None,
            }
        }
    }

    fn wiggle_y_at(playhead: f64) -> Vec<[f32; 2]> {
        let mut g = Graph::new();
        let src = g.add_node("motion.wiggle.test.src");
        let w = g.add_node("motion.wiggle");
        g.connect(Edge {
            from: (src, 0),
            to: (w, 0),
            delayed: false,
        })
        .unwrap();
        g.set_param(w, "channel", 1.0); // Y
        g.set_param(w, "amplitude", 2.0);
        let mut cook = Cook::new();
        let out = cook.cook(&g, &Ops, w, playhead).unwrap();
        match out[0].as_stream().get("P").unwrap() {
            Column::Vec2(v) => v.clone(),
            _ => panic!("P"),
        }
    }

    #[test]
    fn instances_wiggle_independently_within_amplitude() {
        // At a non-zero time the four instances have different Y offsets (each its
        // own noise row), all bounded by ±amplitude, X untouched.
        let p = wiggle_y_at(0.7);
        for q in &p {
            assert_eq!(q[0], 0.0, "X untouched");
            assert!(q[1].abs() <= 2.0, "within ±amplitude");
        }
        assert!(
            p[0][1] != p[1][1] || p[1][1] != p[2][1],
            "instances jitter independently"
        );
    }

    #[test]
    fn the_wiggle_changes_over_time() {
        // Same instance, two different playheads → different offset (it scrolls).
        let a = wiggle_y_at(0.0);
        let b = wiggle_y_at(1.5);
        assert!((a[2][1] - b[2][1]).abs() > 1e-5, "wiggle scrolls with time");
    }

    #[test]
    fn is_deterministic_for_replay() {
        // The whole point of the transcendental-free noise: identical playhead →
        // identical output, every time (HR-5 replay).
        assert_eq!(wiggle_y_at(0.9), wiggle_y_at(0.9));
    }

    /// The focus field gates the jitter (audit 2026-07-10: untested until now):
    /// with `falloff` [1, 0] the second instance sits perfectly still while the
    /// first wiggles — a wiggle ignoring the mask would shake both.
    #[test]
    fn falloff_zero_pins_the_masked_instance() {
        static FSRC_MAN: NodeManifest = NodeManifest {
            id: NodeTypeId::of("motion.wiggle.test.fsrc"),
            name: "motion.wiggle.test.fsrc",
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
        struct FSrc;
        impl NodeOp for FSrc {
            fn manifest(&self) -> &'static NodeManifest {
                &FSRC_MAN
            }
            fn eval(&self, ctx: &mut EvalCtx<'_>) {
                ctx.emit(
                    Stream::new(2)
                        .with("P", Column::Vec2(vec![[0.0, 0.0]; 2]))
                        .with("falloff", Column::Scalar(vec![1.0, 0.0])),
                );
            }
        }
        struct FOps;
        impl OpResolver for FOps {
            fn resolve(&self, ty: NodeTypeId) -> Option<&dyn NodeOp> {
                match ty {
                    t if t == FSRC_MAN.id => Some(&FSrc),
                    t if t == MANIFEST.id => Some(&MotionWiggle),
                    _ => None,
                }
            }
        }
        let mut g = Graph::new();
        let src = g.add_node("motion.wiggle.test.fsrc");
        let w = g.add_node("motion.wiggle");
        g.connect(Edge {
            from: (src, 0),
            to: (w, 0),
            delayed: false,
        })
        .unwrap();
        g.set_param(w, "channel", 1.0); // Y
        g.set_param(w, "amplitude", 2.0);
        // A playhead where the focused row's noise is visibly non-zero.
        let mut cook = Cook::new();
        let out = cook.cook(&g, &FOps, w, 0.7).unwrap();
        match out[0].as_stream().get("P").unwrap() {
            Column::Vec2(v) => {
                assert!(v[0][1].abs() > 1e-4, "focused instance wiggles: {:?}", v[0]);
                assert_eq!(v[1], [0.0, 0.0], "masked instance is pinned");
            }
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
