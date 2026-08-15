#![forbid(unsafe_code)]
//! `motion.noise` — a Motion **behaviour**: a coherent Perlin **gradient**-noise
//! FIELD that displaces a chosen channel, added to the existing value and scaled
//! per-instance by the multiplicative `falloff` column (§1.2; absent → `1.0`).
//! Reads the playhead but holds no state → `Effect::Temporal`.
//!
//! **Field, not jitter — the distinction from `motion.wiggle`.** Wiggle samples
//! `noise(time, instance_index)`: each element jitters on its own row, so they
//! move INDEPENDENTLY (nervous jitter). Noise samples `noise(position·scale,
//! time)`: neighbouring elements read nearby points of one continuous field, so
//! they flow TOGETHER — coherent turbulence (smoke, current, drift). And it is
//! **gradient** noise, not the **value** noise wiggle uses: gradient noise is
//! zero at every lattice point, so it has none of value noise's grid artifacts
//! (see [`noise`] and docs/Motion Nodes/07).
//!
//! Gold standard (doc 07): Improved Perlin 2002 (quintic fade, 8 isotropic
//! gradients) + fBm, transcendental-free (HR-5). Param surface is the
//! cross-tool intersection (Cavalry/AE/Houdini/Blender): scale, octaves,
//! roughness, type, speed, seed.
//!
//! `delta_i = fbm(P_i·scale, seed, octaves, roughness, type @ t·speed) ·
//! amplitude · falloff_i`, added to the chosen channel.

use ph2d_node_registry::{
    NodeRegistry, ParamChannelRange, ParamGroup, ParamUnit, ParamUnitDecl, RegistryError,
};
use ph2d_nodegraph::attr::{Column, Stream};
use ph2d_nodegraph::cook::EvalCtx;
use ph2d_nodegraph::effect::Effect;
use ph2d_nodegraph::node::{LoweringKind, NodeManifest, NodeOp, NodeTypeId, ParamSpec, PortSpec};
use ph2d_nodegraph::port::{Clock, Dim, Domain, PortType};

mod channel;
mod kernel;
use kernel::GPU_KERNEL;
mod noise;
use channel::{apply_channel_delta, falloff_at};
use noise::{NoiseType, fbm};

const INST_VEC2: PortType = PortType::new(Domain::Instances, Dim::Vec2, Clock::Frame);

/// Hard ceiling on octaves — an untrusted `f32` param drives the fBm loop count.
/// 8 is past the point of visible return (each octave halves the feature size).
const MAX_OCTAVES: u32 = 8;

/// The static contract of this node type (ADR-0031).
pub const MANIFEST: NodeManifest = NodeManifest {
    id: NodeTypeId::of("motion.noise"),
    name: "motion.noise",
    inputs: &[PortSpec {
        name: "in",
        ty: INST_VEC2,
    }],
    outputs: &[PortSpec {
        name: "out",
        ty: INST_VEC2,
    }],
    // Reads the playhead → pull-side; the noise is nonetheless deterministic.
    effect: Effect::Temporal,
    clock: Clock::Frame,
    params: &[
        // 0 X · 1 Y · 2 Rotation · 3 Size — the shared channel vocabulary.
        ParamSpec {
            name: "channel",
            default: 1.0,
        },
        // Peak of the displacement (channel-native units), before falloff.
        ParamSpec {
            name: "amplitude",
            default: 1.0,
        },
        // Spatial frequency: feature size of the field. World units are metres
        // (~single digits), so a smaller scale than a pixel tool's — 0.4 gives
        // features a couple of metres across.
        ParamSpec {
            name: "scale",
            default: 0.4,
        },
        // fBm octaves (= AE "Complexity" / Blender "Detail").
        ParamSpec {
            name: "octaves",
            default: 3.0,
        },
        // Per-octave amplitude falloff (= Houdini/Blender "Roughness", the
        // gain/persistence). 0.5 is the universal default.
        ParamSpec {
            name: "roughness",
            default: 0.5,
        },
        // 0 fBm · 1 Turbulence · 2 Ridged.
        ParamSpec {
            name: "type",
            default: 0.0,
        },
        // Temporal scroll speed (= AE "Evolution" / Cavalry "Time Scale"): the
        // field drifts through the elements over playhead-seconds.
        ParamSpec {
            name: "speed",
            default: 0.4,
        },
        // O comprimento do LOOP em segundos (`0` = nunca fecha, o mundo de sempre).
        ParamSpec {
            name: "loop_len",
            default: 0.0,
        },
        // Decorrelates several Noise nodes.
        ParamSpec {
            name: "seed",
            default: 0.0,
        },
        // ⚠️ **Apendado**, e o default é o valor que era const: `2.0` reproduz o
        // mundo de antes AO BIT (escalar por potência de dois não arredonda).
        ParamSpec {
            name: "lacunarity",
            default: 2.0,
        },
    ],
    lowerings: &[LoweringKind::Cpu],
};

struct MotionNoise;

impl NodeOp for MotionNoise {
    fn manifest(&self) -> &'static NodeManifest {
        &MANIFEST
    }

    fn eval(&self, ctx: &mut EvalCtx<'_>) {
        let channel = ctx.param("channel").round() as i32;
        let amplitude = ctx.param("amplitude");
        let scale = ctx.param("scale");
        let octaves = (ctx.param("octaves").round().max(1.0) as u32).min(MAX_OCTAVES);
        let roughness = ctx.param("roughness");
        let ty = NoiseType::from_index(ctx.param("type"));
        let speed = ctx.param("speed");
        let seed = ctx.param("seed").round() as i32;
        let spec = ph2d_fbm::Spec {
            octaves,
            lacunarity: ctx.param("lacunarity"),
            roughness,
            ty,
        };
        // ⚠️ A costura do laço no tempo mudou-se para a folha `ph2d_fbm` — ela é a
        // terceira peça com UM dono e dois consumidores futuros (a família de
        // forças herda o `loop_period` no doc 89 folha 02). O raciocínio inteiro
        // (por que o tempo tem de WRAPAR primeiro, e por que o peso é smoothstep
        // e não linear) viajou com ela.
        let (t_a, t_b, w) = ph2d_fbm::loop_times(ctx.playhead() as f32, ctx.param("loop_len"));

        let out = {
            let input = ctx.input(0);
            let n = input.count();
            // Each element's own world position is the sample point, so the field
            // is spatially coherent; the playhead scrolls it along Y (the field
            // "flows" through the elements).
            let pos = positions(input, n);
            let deltas: Vec<f32> = (0..n)
                .map(|i| {
                    let (px, py) = pos[i];
                    let sample = |tt: f32| fbm(px * scale, py * scale + tt * speed, seed, spec);
                    // `w == 0` é o caminho de sempre: a segunda amostra nem é avaliada.
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

/// Each element's `P` (absent → origin), the field's sample points.
fn positions(input: &Stream, n: usize) -> Vec<(f32, f32)> {
    match input.get("P") {
        Some(Column::Vec2(v)) => {
            let mut out: Vec<(f32, f32)> = v.iter().map(|p| (p[0], p[1])).collect();
            out.resize(n, (0.0, 0.0));
            out
        }
        _ => vec![(0.0, 0.0); n],
    }
}

/// Register this node with the runtime registry. Called (via codegen) from
/// `ph2d-node-registry-init::register_all_nodes`.
pub fn register(reg: &mut NodeRegistry) -> Result<(), RegistryError> {
    reg.register(Box::new(MotionNoise))?;
    reg.register_gpu_kernel(MANIFEST.id, GPU_KERNEL);
    reg.register_ui(
        MANIFEST.id,
        ph2d_node_registry::NodeUiManifest {
            display_name: "Noise",
            // Transform blue: a spatial behaviour that moves elements.
            category: ph2d_node_registry::NodeUiCategory::Transform,
            silhouette: ph2d_node_registry::NodeSilhouette::Rect,
        },
    );
    reg.register_param_ui(MANIFEST.id, PARAM_HINTS);
    reg.register_param_channel_range(MANIFEST.id, PARAM_CHANNEL_RANGE);
    reg.register_param_units(MANIFEST.id, PARAM_UNITS);
    reg.register_param_groups(MANIFEST.id, PARAM_GROUPS);
    Ok(())
}

use ph2d_node_registry::{ParamUiHint, ParamWidget};

/// **O que o `loop_len` É** (doc 88, Wave A): uma DURAÇÃO. É a única unidade deste nó — a
/// `amplitude` é `FromChannel` como a do oscilador seria, mas aqui ela não é declarada porque
/// este nó ainda não passou pela varredura de unidades; o `loop_len` entra declarado para não
/// nascer com a dívida.
static PARAM_UNITS: &[ParamUnitDecl] = &[ParamUnitDecl {
    param: "loop_len",
    unit: ParamUnit::Seconds,
}];

/// As SEÇÕES deste nó (doc 88 B3). Nove controles respondem a três perguntas.
///
/// ⚠️ **"Timing" é o MESMO título do `motion.oscillator`, de propósito** — os dois respondem
/// *em que relógio isto anda*, e dois nomes para a mesma pergunta ensinariam que são coisas
/// diferentes (o precedente dos dois nós de curva, que partilham "Curve").
///
/// Ficam SOLTOS `channel`, `amplitude` e `type`: onde o ruído escreve, quanto ele vale, e que
/// ruído ele é.
static PARAM_GROUPS: &[ParamGroup] = &[
    // A FORMA do campo.
    ParamGroup::new("scale", "Field"),
    ParamGroup::new("octaves", "Field"),
    ParamGroup::new("roughness", "Field"),
    ParamGroup::new("seed", "Field"),
    // Em que relógio ele anda.
    ParamGroup::new("speed", "Timing"),
    ParamGroup::new("loop_len", "Timing"),
];

static PARAM_HINTS: &[ParamUiHint] = &[
    // ⚠️ A faixa começa em 1: lacunarity < 1 faz as oitavas ficarem mais GRANDES
    // que a base — o campo perde a leitura fractal e vira um borrão de baixa
    // frequência. `2` é o universal, e `1,5..3` é onde a mão trabalha.
    ParamUiHint {
        param: "lacunarity",
        label: "Lacunarity",
        min: 1.0,
        max: 4.0,
        step: 0.05,
        widget: ParamWidget::Slider,
    },
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
        param: "scale",
        label: "Scale",
        min: 0.02,
        max: 2.0,
        step: 0.02,
        widget: ParamWidget::Slider,
    },
    ParamUiHint {
        param: "octaves",
        label: "Octaves",
        min: 1.0,
        max: MAX_OCTAVES as f32,
        step: 1.0,
        widget: ParamWidget::IntSlider,
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
        param: "type",
        label: "Type",
        min: 0.0,
        max: 2.0,
        step: 1.0,
        widget: ParamWidget::Enum {
            labels: &["fBm", "Turbulence", "Ridged"],
        },
    },
    ParamUiHint {
        param: "speed",
        label: "Speed",
        min: 0.0,
        max: 3.0,
        step: 0.05,
        widget: ParamWidget::Slider,
    },
    // A faixa de um loop é a de um take de motion graphics: 0 (nunca fecha) até 30 s.
    // ⚠️ **CORRIGIDO (doc 89, grupo B):** este comentário dizia que *"a caixa aceita
    // além dele pelo `ParamHardMax`"* e isso é FALSO — este nó nunca chamou
    // `register_param_hard_max`, e o shell resolve `param_hard_max(..).unwrap_or(max)`,
    // logo a caixa PARA nos 30 s. Um ciclo mais longo é hoje inalcançável neste nó.
    // O irmão `value.noise` declara o dele (2²⁴, precisão de representação); subir
    // este muda o que a caixa aceita e é decisão do dono deste nó.
    ParamUiHint {
        param: "loop_len",
        label: "Loop Length",
        min: 0.0,
        max: 30.0,
        step: 0.1,
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
mod tests {
    use super::*;
    use ph2d_nodegraph::cook::{Cook, OpResolver};
    use ph2d_nodegraph::graph::{Graph, NodeId as GNodeId};

    struct Reg;
    impl OpResolver for Reg {
        fn resolve(&self, ty: NodeTypeId) -> Option<&dyn NodeOp> {
            // A tiny grid source + this node.
            static SRC: NodeManifest = NodeManifest {
                id: NodeTypeId::of("test.grid"),
                name: "test.grid",
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
                    &SRC
                }
                fn eval(&self, ctx: &mut EvalCtx<'_>) {
                    // A 4-across row so neighbours are spatially close.
                    let p: Vec<[f32; 2]> = (0..8).map(|i| [i as f32, 0.0]).collect();
                    ctx.emit(Stream::new(8).with("P", Column::Vec2(p)));
                }
            }
            match ty {
                t if t == MANIFEST.id => {
                    static N: MotionNoise = MotionNoise;
                    Some(&N)
                }
                t if t == SRC.id => {
                    static S: Src = Src;
                    Some(&S)
                }
                _ => None,
            }
        }
    }

    fn cook_y(graph: &Graph, node: GNodeId, t: f64) -> Vec<f32> {
        let mut cook = Cook::new();
        let out = cook.cook(graph, &Reg, node, t).unwrap();
        match out[0].as_stream().get("P").unwrap() {
            Column::Vec2(v) => v.iter().map(|p| p[1]).collect(),
            _ => panic!("P"),
        }
    }

    /// Cooked through the substrate: the noise displaces Y, and the field is
    /// COHERENT — neighbouring elements move by similar amounts (a smooth field),
    /// unlike a per-element jitter which would be uncorrelated. This is the whole
    /// point of a spatial noise field vs `motion.wiggle`.
    #[test]
    fn the_field_displaces_y_coherently_across_neighbours() {
        let mut g = Graph::new();
        let src = g.add_node("test.grid");
        let noise = g.add_node("motion.noise");
        g.connect(ph2d_nodegraph::graph::Edge {
            from: (src, 0),
            to: (noise, 0),
            delayed: false,
        })
        .unwrap();
        g.set_param(noise, "amplitude", 1.0);
        g.set_param(noise, "scale", 0.25); // large features → strong neighbour correlation
        g.set_param(noise, "octaves", 1.0);

        let ys = cook_y(&g, noise, 0.5);
        // Something moved (the field is not flat).
        assert!(
            ys.iter().any(|&y| y.abs() > 0.01),
            "the field displaced nothing"
        );
        // Coherence: at a large feature size, adjacent elements differ far less
        // than the amplitude — they belong to the same swell, not independent
        // random draws. Max neighbour step is a fraction of the peak.
        let max_step = ys
            .windows(2)
            .map(|w| (w[1] - w[0]).abs())
            .fold(0.0_f32, f32::max);
        let peak = ys.iter().map(|y| y.abs()).fold(0.0_f32, f32::max);
        assert!(
            max_step < peak * 0.9,
            "neighbours should move coherently: step {max_step} vs peak {peak}"
        );
    }

    /// SONDA: quanto o ciclo deriva ao longo de muitas voltas (precisao de f32).
    #[test]
    #[ignore = "sonda de medição"]
    fn measure_the_loop_drift() {
        let l = 3.0f32;
        println!("\n=== tempo LIDO pelo campo, volta a volta (L = 3.0) ===");
        for volta in [0u32, 1, 2, 10, 100, 1000] {
            let t = 0.125 + f32::from(u16::try_from(volta).unwrap()) * l;
            let (a, _b, w) = ph2d_fbm::loop_times(t, l);
            let mut g = Graph::new();
            let src = g.add_node("test.grid");
            let noise = g.add_node("motion.noise");
            g.connect(ph2d_nodegraph::graph::Edge {
                from: (src, 0),
                to: (noise, 0),
                delayed: false,
            })
            .unwrap();
            g.set_param(noise, "speed", 1.0);
            g.set_param(noise, "loop_len", l);
            let base = cook_y(&g, noise, 0.125);
            let here = cook_y(&g, noise, f64::from(t));
            let dev = base
                .iter()
                .zip(&here)
                .map(|(a, b)| (a - b).abs())
                .fold(0.0f32, f32::max);
            println!(
                "  volta {volta:>5}: t={t:>10.4}  tau={a:>12.9}  w={w:>12.9}  desvio no VALOR={dev:.3e}"
            );
        }
        println!();
    }

    /// SONDA: a inclinação do campo ao longo do ciclo — a costura é quina ou é ruído?
    #[test]
    #[ignore = "sonda de medição"]
    fn measure_the_seam_slope() {
        let mut g = Graph::new();
        let src = g.add_node("test.grid");
        let noise = g.add_node("motion.noise");
        g.connect(ph2d_nodegraph::graph::Edge {
            from: (src, 0),
            to: (noise, 0),
            delayed: false,
        })
        .unwrap();
        g.set_param(noise, "speed", 1.0);
        let l = 3.0f64;
        g.set_param(noise, "loop_len", l as f32);
        for d in [0.05f64, 0.02, 0.005, 0.001] {
            let slope = |t: f64| -> f32 {
                let lo = cook_y(&g, noise, t - d);
                let hi = cook_y(&g, noise, t + d);
                ((hi[0] - lo[0]) as f64 / (2.0 * d)) as f32
            };
            print!("d={d:>6} |");
            for frac in [0.02f64, 0.25, 0.5, 0.75, 0.98] {
                print!("  tau={:>5.2}: {:>7.3}", frac * l, slope(frac * l));
            }
            println!(
                "   salto na costura: {:.4}",
                (slope(l - 2.0 * d) - slope(2.0 * d)).abs()
            );
        }
    }

    /// **O ciclo FECHA: o campo em `t` e em `t + L` é o mesmo.**
    ///
    /// Nasce vermelho sobre o cross-fade ingênuo (misturar `t` com `t − L` sem wrapar o tempo
    /// primeiro), que é o erro natural aqui — ele produz um valor contínuo e um ciclo que NÃO
    /// fecha, errando por O(1); nenhum outro gate desta crate o distingue.
    ///
    /// Amostra o ciclo inteiro, não os endpoints: uma lei que só casasse em `t = 0` passaria
    /// por um oráculo de dois pontos.
    ///
    /// ⚠️ **A tolerância é MEDIDA, não escolhida, e o mecanismo dela é `f32`:** a igualdade é
    /// exata em ℝ, mas `frac(t / L)` perde mantissa conforme `t` cresce, então o tempo que o
    /// campo lê deriva. Medido (sonda `measure_the_loop_drift`, L = 3): **2,1e-7 na 1ª volta ·
    /// 1,4e-5 na centésima · 1,1e-4 na milésima** — 50 minutos de relógio para um desvio de um
    /// décimo de milésimo de unidade de mundo. Fazer o wrap em `f64` cortaria isso, e foi
    /// RECUSADO: o WGSL só tem `f32`, então os dois lados divergiriam e a paridade CPU×GPU —
    /// que é o que prova que o device concorda — passaria a ter um épsilon que ninguém mediu.
    #[test]
    fn the_loop_closes_the_field_repeats_exactly() {
        let mut g = Graph::new();
        let src = g.add_node("test.grid");
        let noise = g.add_node("motion.noise");
        g.connect(ph2d_nodegraph::graph::Edge {
            from: (src, 0),
            to: (noise, 0),
            delayed: false,
        })
        .unwrap();
        g.set_param(noise, "speed", 1.0);
        g.set_param(noise, "loop_len", 3.0);
        // 10× o desvio medido na 2ª volta — aperta o bastante para o cross-fade ingênuo, que
        // erra por O(1), morrer com folga de quatro ordens.
        const TOL: f32 = 5e-6;
        let dev = |a: &[f32], b: &[f32]| {
            a.iter()
                .zip(b)
                .map(|(x, y)| (x - y).abs())
                .fold(0.0f32, f32::max)
        };
        for k in 0..24 {
            let t = f64::from(k) * 0.125;
            let base = cook_y(&g, noise, t);
            let d1 = dev(&base, &cook_y(&g, noise, t + 3.0));
            let d2 = dev(&base, &cook_y(&g, noise, t + 6.0));
            assert!(
                d1 < TOL && d2 < TOL,
                "o campo em t={t} nao volta em t+L: desvio {d1} numa volta, {d2} em duas"
            );
        }
    }

    /// **Loop desligado é o mundo de sempre, AO BIT** — o default não move um número.
    ///
    /// A metade oposta do gate acima: sem ela, "faça o ciclo fechar" tem a resposta trivial de
    /// congelar o campo, que fecha o ciclo perfeitamente e destrói o nó.
    #[test]
    fn no_loop_is_the_old_world_and_the_field_still_evolves() {
        let mut g = Graph::new();
        let src = g.add_node("test.grid");
        let noise = g.add_node("motion.noise");
        g.connect(ph2d_nodegraph::graph::Edge {
            from: (src, 0),
            to: (noise, 0),
            delayed: false,
        })
        .unwrap();
        g.set_param(noise, "speed", 1.0);
        // Sem loop o campo NUNCA se repete no alcance medido.
        assert_ne!(cook_y(&g, noise, 0.0), cook_y(&g, noise, 3.0));
        assert_ne!(cook_y(&g, noise, 0.0), cook_y(&g, noise, 1.0));
        // E com o loop ARMADO ele continua evoluindo DENTRO do ciclo (não congela).
        g.set_param(noise, "loop_len", 3.0);
        assert_ne!(cook_y(&g, noise, 0.0), cook_y(&g, noise, 1.0));
        assert_ne!(cook_y(&g, noise, 1.0), cook_y(&g, noise, 2.0));
    }

    /// **A costura é C¹: o salto de inclinação CONVERGE A ZERO quando a amostragem aperta.**
    ///
    /// O peso smoothstep existe só para isto — com peso LINEAR o valor fecha e a derivada
    /// salta, e um salto de derivada num campo de movimento lê como um tranco a cada volta.
    ///
    /// ⚠️ **O oráculo é a CONVERGÊNCIA, não um número**, e as duas versões anteriores deste
    /// gate erraram de maneiras opostas — vale mais que a lei:
    ///
    /// 1. A primeira media a inclinação de `a + (b − a)·w`, uma mistura de TEMPOS, e ficava
    ///    **VERDE sobre o peso linear**: ali `w = u` colapsa a expressão em `τ − L·(τ/L) = 0`,
    ///    constante. Mas o campo faz `lerp(fbm(a), fbm(b), w)`, e `fbm` não é linear — misturar
    ///    tempos não é misturar campos. Era espelho da aritmética, não do fenômeno.
    /// 2. A segunda amostrou o campo COZIDO (certo) com uma diferença central de `d = 0,02` e
    ///    **REPROVOU a lei correta**, acusando um salto de 0,60. Medido (sonda
    ///    `measure_the_seam_slope`), o salto é **0,9653 · 0,2609 · 0,0206 · 0,0009** para
    ///    `d = 0,05 · 0,02 · 0,005 · 0,001`: ele converge a zero, que é a assinatura de uma
    ///    derivada que EXISTE. Uma quina de verdade daria salto constante.
    ///
    /// Por isso o gate compara o salto em duas resoluções: se a derivada existe ele encolhe com
    /// `d`; se há quina, ele fica onde está.
    #[test]
    fn the_seam_of_the_loop_is_smooth_not_a_kink() {
        let mut g = Graph::new();
        let src = g.add_node("test.grid");
        let noise = g.add_node("motion.noise");
        g.connect(ph2d_nodegraph::graph::Edge {
            from: (src, 0),
            to: (noise, 0),
            delayed: false,
        })
        .unwrap();
        g.set_param(noise, "speed", 1.0);
        let l = 3.0f64;
        g.set_param(noise, "loop_len", l as f32);

        // O salto de inclinação através da costura, medido com passo `d`.
        let jump = |d: f64| -> f32 {
            let slope = |t: f64| -> Vec<f32> {
                let lo = cook_y(&g, noise, t - d);
                let hi = cook_y(&g, noise, t + d);
                hi.iter()
                    .zip(&lo)
                    .map(|(a, b)| ((a - b) as f64 / (2.0 * d)) as f32)
                    .collect()
            };
            let before = slope(l - 2.0 * d);
            let after = slope(2.0 * d);
            before
                .iter()
                .zip(&after)
                .map(|(a, b)| (a - b).abs())
                .fold(0.0f32, f32::max)
        };

        let coarse = jump(0.02);
        let fine = jump(0.001);
        assert!(
            fine < coarse * 0.25,
            "o salto de inclinacao NAO converge ({coarse} em d=0.02 contra {fine} em d=0.001) \
             -- a costura tem QUINA, e nao erro de amostragem"
        );
    }

    /// The playhead scrolls the field (Temporal): the same elements read a
    /// different slice of the field at a later time.
    #[test]
    fn the_field_evolves_with_the_playhead() {
        let mut g = Graph::new();
        let src = g.add_node("test.grid");
        let noise = g.add_node("motion.noise");
        g.connect(ph2d_nodegraph::graph::Edge {
            from: (src, 0),
            to: (noise, 0),
            delayed: false,
        })
        .unwrap();
        g.set_param(noise, "speed", 1.0);
        assert_ne!(cook_y(&g, noise, 0.0), cook_y(&g, noise, 1.0));
    }
}
