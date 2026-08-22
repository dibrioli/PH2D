#![forbid(unsafe_code)]
//! `motion.stagger` — a Motion **behaviour**: writes a per-index ramp `min→max`
//! (shaped by an easing curve) into a chosen channel of the stream, **added** to
//! the existing value and scaled per-instance by the multiplicative `falloff`
//! column (§1.2; absent → `1.0`). The ramp uses the instance's position in the
//! stream (`i / (n-1)`), so the first instance takes ≈`min·falloff` and the last
//! ≈`max·falloff` (`reverse` flips the direction). Index-based, not time-based →
//! `Pure`. Every other column passes through unchanged (count preserved).
//!
//! Params (read via `ctx.param`):
//! - `channel` (1): target channel — `0` X, `1` Y, `2` Rotation, `3` Size.
//!   Position is world units, Rotation is **degrees** (the `rot` column's unit),
//!   Size a scale delta on the unit identity `[1,1]`.
//! - `min` (-1), `max` (1): the ramp endpoints (channel-native units).
//! - `ease_curve` (0): curve family — `0` Linear, `1` Quad, `2` Cubic, `3` Quart,
//!   `4` Quint, `5` Circ, `6` Back, `7` Bounce. All polynomial except Circ,
//!   which uses IEEE `sqrt` — correctly rounded, so still deterministic (HR-5;
//!   no transcendentals anywhere).
//! - `ease_dir` (0): the family's direction — `0` In, `1` Out, `2` In-Out.
//! - `reverse` (0/1): mirror the ramp (last instance takes `min`).
//!
//! `delta_i = (min + ease(raw_i)·(max−min)) · falloff_i`, added to the channel.

use ph2d_node_registry::{
    NodeRegistry, ParamChannelRange, ParamUnit, ParamUnitDecl, RegistryError,
};
use ph2d_nodegraph::cook::EvalCtx;
use ph2d_nodegraph::effect::Effect;
use ph2d_nodegraph::node::{LoweringKind, NodeManifest, NodeOp, NodeTypeId, ParamSpec, PortSpec};
use ph2d_nodegraph::port::{Clock, Dim, Domain, PortType};

mod channel;
mod ease;
mod kernel;
use channel::{apply_channel_delta, falloff_at};
use ease::ease;

const INST_VEC2: PortType = PortType::new(Domain::Instances, Dim::Vec2, Clock::Frame);

/// The static contract of this node type (ADR-0031).
pub const MANIFEST: NodeManifest = NodeManifest {
    id: NodeTypeId::of("motion.stagger"),
    name: "motion.stagger",
    inputs: &[PortSpec {
        name: "in",
        ty: INST_VEC2,
    }],
    outputs: &[PortSpec {
        name: "out",
        ty: INST_VEC2,
    }],
    // Index-based (no playhead, no state) → freely cacheable.
    effect: Effect::Pure,
    clock: Clock::Frame,
    params: &[
        ParamSpec {
            name: "channel",
            default: 1.0,
        },
        ParamSpec {
            name: "min",
            default: -1.0,
        },
        ParamSpec {
            name: "max",
            default: 1.0,
        },
        // Easing = a curve family shaped by a direction (see `ease`).
        ParamSpec {
            name: "ease_curve",
            default: 0.0,
        },
        ParamSpec {
            name: "ease_dir",
            default: 0.0,
        },
        // O *Offset* da Cavalry: desliza a rampa ao longo do índice, ciclicamente.
        ParamSpec {
            name: "offset",
            default: 0.0,
        },
        ParamSpec {
            name: "reverse",
            default: 0.0,
        },
    ],
    lowerings: &[LoweringKind::Cpu],
};

/// The per-instance stagger delta before it's added to the channel: the eased
/// ramp `min→max` at position `raw ∈ [0,1]`, scaled by `falloff`. Easing is the
/// `curve` family shaped by `dir` (In / Out / In-Out) — see [`ease`].
fn stagger_delta(min: f32, max: f32, curve: i32, dir: i32, raw: f32, falloff: f32) -> f32 {
    (min + ease(curve, dir, raw) * (max - min)) * falloff
}

struct MotionStagger;

impl NodeOp for MotionStagger {
    fn manifest(&self) -> &'static NodeManifest {
        &MANIFEST
    }

    fn eval(&self, ctx: &mut EvalCtx<'_>) {
        let channel = ctx.param("channel").round() as i32;
        let min = ctx.param("min");
        let max = ctx.param("max");
        let curve = ctx.param("ease_curve").round() as i32;
        let dir = ctx.param("ease_dir").round() as i32;
        let reverse = ctx.param("reverse") >= 0.5;
        let offset = ctx.param("offset");
        let out = {
            let input = ctx.input(0);
            let n = input.count();
            let deltas: Vec<f32> = (0..n)
                .map(|i| {
                    // Position in the stream, `0..1` (a single instance → 0).
                    let raw = if n <= 1 {
                        0.0
                    } else {
                        i as f32 / (n as f32 - 1.0)
                    };
                    let raw = if reverse { 1.0 - raw } else { raw };
                    // ⚠️ **O `frac` só corre com o knob armado, e a razão é a
                    // PONTA da rampa:** o último elemento senta exactamente em
                    // `1.0`, e `frac(1.0)` é `0.0` — aplicado sempre, o neutro
                    // mandaria a última peça para o começo da rampa.
                    let raw = if offset != 0.0 {
                        let s = raw + offset;
                        s - s.floor()
                    } else {
                        raw
                    };
                    stagger_delta(min, max, curve, dir, raw, falloff_at(input, i))
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
    reg.register(Box::new(MotionStagger))?;
    reg.register_gpu_kernel(MANIFEST.id, kernel::GPU_KERNEL);
    // M1.R1 — UI metadata. Behaviours modify transform channels → Transform
    // (blue) for now; a dedicated Behaviour category (cyan) is a follow-up.
    reg.register_ui(
        MANIFEST.id,
        ph2d_node_registry::NodeUiManifest {
            display_name: "Stagger",
            category: ph2d_node_registry::NodeUiCategory::Transform,
            silhouette: ph2d_node_registry::NodeSilhouette::Rect,
        },
    );
    reg.register_param_ui(MANIFEST.id, PARAM_HINTS);
    reg.register_param_gates(MANIFEST.id, PARAM_GATES);
    reg.register_param_channel_range(MANIFEST.id, PARAM_CHANNEL_RANGE);
    reg.register_param_units(MANIFEST.id, PARAM_UNITS);
    Ok(())
}

use ph2d_node_registry::{ParamGate, ParamUiHint, ParamWidget};

/// **A direção do easing não existe numa reta** (doc 90 §2, caça aos knobs mortos).
///
/// ⚠️ `ease_curve` nasce em `Linear`, e o `Linear` devolve `t` **antes** de olhar para a
/// direção — In, Out e In-Out dão a mesma saída, ao bit. Era o pior dos dezanove pela posição:
/// um Stagger recém-largado, o artista gira o seletor de direção à procura do que ele promete,
/// e nada se move. *É o primeiro gesto que qualquer pessoa faz neste nó.*
///
/// `0 = Linear` · `1..7 = Quad · Cubic · Quart · Quint · Circ · Back · Bounce`.
static PARAM_GATES: &[ParamGate] = &[ParamGate {
    param: "ease_dir",
    when: "ease_curve",
    values: &[1, 2, 3, 4, 5, 6, 7],
}];

/// Param UI hints (M1.P1). `channel` / `ease_curve` / `ease_dir` are **named**
/// selectors (segmented buttons), `reverse` a checkbox — never number sliders.
/// Easing is a curve family × direction (the Penner set minus the transcendental
/// ones); the enum option index IS the param value.
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
        param: "min",
        label: "Min",
        min: -10.0,
        max: 10.0,
        step: 0.05,
        widget: ParamWidget::Slider,
    },
    ParamUiHint {
        param: "max",
        label: "Max",
        min: -10.0,
        max: 10.0,
        step: 0.05,
        widget: ParamWidget::Slider,
    },
    ParamUiHint {
        param: "ease_curve",
        label: "Ease",
        min: 0.0,
        max: 7.0,
        step: 1.0,
        widget: ParamWidget::Enum {
            labels: &[
                "Linear", "Quad", "Cubic", "Quart", "Quint", "Circ", "Back", "Bounce",
            ],
        },
    },
    ParamUiHint {
        param: "ease_dir",
        label: "Direction",
        min: 0.0,
        max: 2.0,
        step: 1.0,
        widget: ParamWidget::Enum {
            labels: &["In", "Out", "In-Out"],
        },
    },
    // ⚠️ **A faixa é um ciclo INTEIRO** (`0..1`): a rampa fecha em si mesma, e um
    // deslize de `1` é o mesmo que nenhum. Um teto maior seria uma volta a mais
    // que o artista não distingue.
    ParamUiHint {
        param: "offset",
        label: "Offset",
        min: 0.0,
        max: 1.0,
        step: 0.01,
        widget: ParamWidget::Slider,
    },
    ParamUiHint {
        param: "reverse",
        label: "Reverse",
        min: 0.0,
        max: 1.0,
        step: 1.0,
        widget: ParamWidget::Toggle,
    },
];

/// **What each of this node's numbers IS** (doc 88, Wave A). This node's magnitude
/// is `FromChannel`: it means metres on Position, DEGREES on Rotation and a bare
/// scale factor on Size, so the panel resolves the unit per-channel. Declaring a
/// fixed `Length` here would scale degrees by `pixels_per_meter` — the failure
/// that turns a `±90` preset into a `±9000`.
static PARAM_UNITS: &[ParamUnitDecl] = &[
    ParamUnitDecl {
        param: "min",
        unit: ParamUnit::FromChannel,
    },
    ParamUnitDecl {
        param: "max",
        unit: ParamUnit::FromChannel,
    },
];

/// **A faixa que estas magnitudes querem quando o canal é ANGULAR** — graus, não
/// unidades de mundo. Uma volta para cada lado, discada em graus inteiros.
///
/// ⚠️ Ela mora AQUI e não numa tabela do shell porque a tabela apodreceu: medida,
/// ela cobria três dos seis nós que precisavam dela, e cada um dos três ausentes
/// esperava o próprio report do artista.
const TURN: f32 = 360.0;
static PARAM_CHANNEL_RANGE: &[ParamChannelRange] = &[
    ParamChannelRange {
        param: "min",
        min: -TURN,
        max: TURN,
        step: 1.0,
    },
    ParamChannelRange {
        param: "max",
        min: -TURN,
        max: TURN,
        step: 1.0,
    },
];

#[cfg(test)]
mod tests {
    use super::*;
    use ph2d_nodegraph::attr::{Column, Stream};
    use ph2d_nodegraph::cook::{Cook, OpResolver};
    use ph2d_nodegraph::graph::{Edge, Graph};

    // Source: 3 instances on a line (x = 0,1,2) with an existing rot [0,0,0].
    static SRC_MAN: NodeManifest = NodeManifest {
        id: NodeTypeId::of("motion.stagger.test.src"),
        name: "motion.stagger.test.src",
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
            ctx.emit(
                Stream::new(3).with("P", Column::Vec2(vec![[0.0, 0.0], [1.0, 0.0], [2.0, 0.0]])),
            );
        }
    }
    struct Ops;
    impl OpResolver for Ops {
        fn resolve(&self, ty: NodeTypeId) -> Option<&dyn NodeOp> {
            match ty {
                t if t == SRC_MAN.id => Some(&Src),
                t if t == MANIFEST.id => Some(&MotionStagger),
                _ => None,
            }
        }
    }

    fn staggered_y(setup: impl FnOnce(&mut Graph, ph2d_nodegraph::graph::NodeId)) -> Vec<[f32; 2]> {
        let mut g = Graph::new();
        let src = g.add_node("motion.stagger.test.src");
        let st = g.add_node("motion.stagger");
        g.connect(Edge {
            from: (src, 0),
            to: (st, 0),
            delayed: false,
        })
        .unwrap();
        g.set_param(st, "channel", 1.0); // Y
        setup(&mut g, st);
        let mut cook = Cook::new();
        let out = cook.cook(&g, &Ops, st, 0.0).unwrap();
        match out[0].as_stream().get("P").unwrap() {
            Column::Vec2(v) => v.clone(),
            _ => panic!("P"),
        }
    }

    #[test]
    fn linear_ramp_adds_min_to_max_across_the_stream() {
        // channel=Y, min=0, max=2, linear: raw = 0, 0.5, 1 → Δy = 0, 1, 2 added.
        let p = staggered_y(|g, st| {
            g.set_param(st, "min", 0.0);
            g.set_param(st, "max", 2.0);
        });
        assert_eq!(p, vec![[0.0, 0.0], [1.0, 1.0], [2.0, 2.0]]);
    }

    #[test]
    fn reverse_flips_the_ramp_endpoints() {
        // reverse: raw = 1, 0.5, 0 → Δy = 2, 1, 0. X passes through unchanged.
        let p = staggered_y(|g, st| {
            g.set_param(st, "min", 0.0);
            g.set_param(st, "max", 2.0);
            g.set_param(st, "reverse", 1.0);
        });
        assert_eq!(p, vec![[0.0, 2.0], [1.0, 1.0], [2.0, 0.0]]);
    }

    #[test]
    fn easing_applies_a_non_linear_curve_to_the_ramp() {
        // Quad-In (curve 1, dir 0) bends the ramp below the diagonal at the
        // midpoint: min=0,max=4 over 3 instances → mid Δy = 4·ease(1,0,0.5) = 1
        // (vs 2 for linear). Proves ease_curve/ease_dir flow into the delta.
        let p = staggered_y(|g, st| {
            g.set_param(st, "min", 0.0);
            g.set_param(st, "max", 4.0);
            g.set_param(st, "ease_curve", 1.0); // Quad
            g.set_param(st, "ease_dir", 0.0); // In
        });
        assert_eq!(p[0][1], 0.0); // endpoint
        assert_eq!(p[1][1], 1.0); // eased midpoint (below linear's 2)
        assert_eq!(p[2][1], 4.0); // endpoint
    }

    #[test]
    fn single_instance_takes_min() {
        // n == 1 → raw = 0 → delta = min (no divide-by-zero on `n-1`).
        assert_eq!(stagger_delta(3.0, 9.0, 0, 0, 0.0, 1.0), 3.0);
    }

    /// The focus field gates the ramp (audit 2026-07-10: untested until now):
    /// a source that carries `falloff` [1, 0.5, 0] sees the linear 0→2 ramp
    /// scaled per instance — the masked tail doesn't move at all.
    #[test]
    fn falloff_scales_the_ramp_per_instance() {
        static FSRC_MAN: NodeManifest = NodeManifest {
            id: NodeTypeId::of("motion.stagger.test.fsrc"),
            name: "motion.stagger.test.fsrc",
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
                    Stream::new(3)
                        .with("P", Column::Vec2(vec![[0.0, 0.0]; 3]))
                        .with("falloff", Column::Scalar(vec![1.0, 0.5, 0.0])),
                );
            }
        }
        struct FOps;
        impl OpResolver for FOps {
            fn resolve(&self, ty: NodeTypeId) -> Option<&dyn NodeOp> {
                match ty {
                    t if t == FSRC_MAN.id => Some(&FSrc),
                    t if t == MANIFEST.id => Some(&MotionStagger),
                    _ => None,
                }
            }
        }
        let mut g = Graph::new();
        let src = g.add_node("motion.stagger.test.fsrc");
        let st = g.add_node("motion.stagger");
        g.connect(Edge {
            from: (src, 0),
            to: (st, 0),
            delayed: false,
        })
        .unwrap();
        g.set_param(st, "channel", 1.0); // Y
        g.set_param(st, "min", 0.0);
        g.set_param(st, "max", 2.0);
        let mut cook = Cook::new();
        let out = cook.cook(&g, &FOps, st, 0.0).unwrap();
        match out[0].as_stream().get("P").unwrap() {
            // Linear raw = 0, 0.5, 1 → Δy = 0, 1, 2; × falloff [1, 0.5, 0]
            // → 0, 0.5, 0. The masked LAST instance stays put even though the
            // raw ramp peaks there.
            Column::Vec2(v) => assert_eq!(v, &vec![[0.0, 0.0], [0.0, 0.5], [0.0, 0.0]]),
            _ => panic!("P"),
        }
    }

    #[test]
    fn registers_and_resolves() {
        let mut reg = NodeRegistry::new();
        register(&mut reg).unwrap();
        assert!(reg.resolve(MANIFEST.id).is_some());
    }

    /// **TROCAR AS PONTAS JÁ INVERTE A RAMPA — e o `reverse` É OUTRA COISA.**
    ///
    /// ⚠️ **Este gate nasceu de uma célula REFUTADA** (folha 06 linha 31,
    /// *"Min>Max auto-invertido"*, Cavalry, marcada P2 como *omissão de
    /// ergonomia*). Medido em 2026-08-22: `min + ease(t)·(max − min)` **já** dá a
    /// rampa descendente quando o artista troca os dois números — não há gesto
    /// extra, não há nó a mais, e o *default inteligente* que a referência
    /// documenta é o que este nó sempre fez.
    ///
    /// ⚠️ **E o que ficaria por construir seria uma REDEFINIÇÃO, não uma adição.**
    /// Com uma ease não-linear há DUAS rampas descendentes possíveis: a que sai de
    /// `min` devagar (a nossa — a ease descreve *como o valor deixa a primeira
    /// ponta e chega à segunda*, e trocar os números troca as pontas, não a lei) e
    /// a que é a rampa ascendente LIDA AO CONTRÁRIO. A segunda já tem um param
    /// dedicado: `reverse`. Construir um "auto-invert" seria ou um no-op na curva
    /// linear, ou o `reverse` a mudar de significado em silêncio.
    ///
    /// Este gate pina as três metades: a troca inverte · o `reverse` espelha · os
    /// dois **diferem** numa ease não-linear (senão o par seria redundante e a
    /// refutação estaria a defender uma distinção que não existe).
    #[test]
    fn swapping_the_endpoints_inverts_and_reverse_is_a_different_thing() {
        let ramp = |lo: f32, hi: f32, curve: i32| -> Vec<f32> {
            (0..5)
                .map(|i| stagger_delta(lo, hi, curve, 0, i as f32 / 4.0, 1.0))
                .collect()
        };
        for curve in 0..=7 {
            let up = ramp(0.0, 1.0, curve);
            let down = ramp(1.0, 0.0, curve);
            // (1) A troca REFLETE a rampa no valor: `down = 1 − up`, exacto.
            //
            // ⚠️ **E não «é monótona a descer»** — a primeira versão deste gate
            // pediu isso e reprovou nas curvas **Back** e **Bounce**, que
            // ultrapassam de propósito. Uma barra que código correto não consegue
            // satisfazer não é rigor; a lei verdadeira é a reflexão, e ela vale
            // para as oito porque sai da própria fórmula
            // (`min + e·(max−min)` com `[1,0]` é `1 − e`).
            assert!((down[0] - 1.0).abs() < 1e-6, "curva {curve}: comeca em min");
            assert!((down[4] - 0.0).abs() < 1e-6, "curva {curve}: acaba em max");
            for (d, u) in down.iter().zip(&up) {
                assert!(
                    (d - (1.0 - u)).abs() < 1e-6,
                    "curva {curve}: {d} vs {}",
                    1.0 - u
                );
            }
            // (2) O `reverse` ESPELHA a rampa ascendente.
            let mirrored: Vec<f32> = (0..5)
                .map(|i| stagger_delta(0.0, 1.0, curve, 0, 1.0 - i as f32 / 4.0, 1.0))
                .collect();
            let up_rev: Vec<f32> = up.iter().rev().copied().collect();
            for (a, b) in mirrored.iter().zip(&up_rev) {
                assert!((a - b).abs() < 1e-6, "curva {curve}: {a} vs {b}");
            }
        }
        // (3) ⚠️ E as duas leituras DIFEREM numa ease não-linear — é o que torna a
        // refutação uma distinção real e não uma desculpa. Na linear coincidem, de
        // propósito: lá não há nada a distinguir.
        let linear_gap = ramp(1.0, 0.0, 0)
            .iter()
            .zip(ramp(0.0, 1.0, 0).iter().rev())
            .map(|(a, b)| (a - b).abs())
            .fold(0.0f32, f32::max);
        assert!(
            linear_gap < 1e-6,
            "na linear as duas coincidem: {linear_gap}"
        );
        let eased_gap = ramp(1.0, 0.0, 1)
            .iter()
            .zip(ramp(0.0, 1.0, 1).iter().rev())
            .map(|(a, b)| (a - b).abs())
            .fold(0.0f32, f32::max);
        assert!(
            eased_gap > 0.3,
            "numa ease as duas tem de divergir, senao o `reverse` seria redundante: {eased_gap}"
        );
    }

    /// **SONDA: as duas leituras da rampa invertida, lado a lado** — o instrumento
    /// que produziu a refutação acima.
    ///
    /// `cargo test -p ph2d-node-motion-stagger measure_reversed_endpoints -- --ignored --nocapture`
    #[test]
    #[ignore = "sonda, não um gate — `-- --ignored --nocapture`"]
    fn measure_reversed_endpoints() {
        for curve in 0..=7 {
            let up: Vec<f32> = (0..5)
                .map(|i| stagger_delta(0.0, 1.0, curve, 0, i as f32 / 4.0, 1.0))
                .collect();
            let down: Vec<f32> = (0..5)
                .map(|i| stagger_delta(1.0, 0.0, curve, 0, i as f32 / 4.0, 1.0))
                .collect();
            // O que um "swap + reverse" daria: a mesma rampa lida ao contrário.
            let mirror: Vec<f32> = up.iter().rev().copied().collect();
            println!("curva {curve}:");
            println!("  min<max : {up:.3?}");
            println!("  min>max : {down:.3?}");
            println!("  espelho : {mirror:.3?}");
        }
    }
}

#[cfg(test)]
#[path = "offset_tests.rs"]
mod offset_tests;
