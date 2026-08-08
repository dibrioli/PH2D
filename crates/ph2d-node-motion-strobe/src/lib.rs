//! `motion.strobe` — a PULSE fires a decaying flash on the stream it passes
//! through (Motion Nodes M2; decision doc `06_pulse_*`, §4).
//!
//! This is the first CONSUMER of the pulse type, and the wave's visible payoff:
//! every pulse "lights up" the element (a size boost + a colour flash) that then
//! **decays geometrically** over the next ticks — attack-instant, decay-exp, the
//! minimal ADSR. It is the TouchDesigner Trigger CHOP's *"audio-style ADSR
//! envelope"* and Unreal's Notify-State *begin/tick/end*, reduced to the shape a
//! motion element actually needs.
//!
//! **The envelope IS the value of the `pre` self-loop.** A per-instance `glow`
//! (0..1) rides the `state` port: a pulse sets it to 1.0, and each tick with no
//! pulse multiplies it by a rate DERIVED from the authored `Flash Length` (see
//! `decay_per_tick`). Applied ONCE per tick to the carried glow (so
//! a glow `n` ticks old has decayed exactly `n` times), never recomputed from an
//! age — the same geometric discipline as `motion.trail`. The lit look is
//! applied to the FRESH upstream `in` each tick (size/tint), so the boost never
//! compounds into the geometry; only `glow` persists.
//!
//! The multiplicative `falloff` column masks the APPLIED look (size boost +
//! flash), like every behaviour — a focus field chooses which dots flash. The
//! glow memory itself is unmasked, so an animated field fades a live flash
//! without retriggering it.
//!
//! Positional per-instance (v1), matching `pulse.threshold`: `in`/`pulse`/`state`
//! pair by row order. The focus rig has a stable count.

#![forbid(unsafe_code)]

use ph2d_node_registry::{NodeRegistry, RegistryError};
use ph2d_nodegraph::attr::{Column, Stream};
use ph2d_nodegraph::cook::EvalCtx;
use ph2d_nodegraph::effect::Effect;
use ph2d_nodegraph::node::{LoweringKind, NodeManifest, NodeOp, NodeTypeId, ParamSpec, PortSpec};
use ph2d_nodegraph::port::{Clock, Dim, Domain, PortType};

const INST_VEC2: PortType = PortType::new(Domain::Instances, Dim::Vec2, Clock::Frame);
/// The pulse type (mirror of `ph2d_node_pulse_threshold::PULSE`; kept local so
/// this crate stays a leaf drop-crate — the shared vocabulary is the port
/// `(Instances, Scalar, Event)`, not a shared symbol).
const PULSE: PortType = PortType::new(Domain::Instances, Dim::Scalar, Clock::Event);

/// The pulse stream's fire column (`1.0` on a fired tick).
const PULSE_COL: &str = "pulse";
/// The per-instance envelope carried on the `pre` self-loop.
const GLOW_COL: &str = "glow";

/// The static contract of this node type (ADR-0031).
pub const MANIFEST: NodeManifest = NodeManifest {
    id: NodeTypeId::of("motion.strobe"),
    name: "motion.strobe",
    inputs: &[
        PortSpec {
            name: "in",
            ty: INST_VEC2,
        },
        PortSpec {
            name: "pulse",
            ty: PULSE,
        },
        // The envelope feedback; `state` → editor-plumbed `pre` self-loop.
        PortSpec {
            name: "state",
            ty: INST_VEC2,
        },
    ],
    outputs: &[PortSpec {
        name: "out",
        ty: INST_VEC2,
    }],
    // Pure: the tick enters the fingerprint through the consumed `pre` edge.
    effect: Effect::Pure,
    clock: Clock::Frame,
    params: &[
        // ⚠️ A DURAÇÃO do flash em TICKS, não a taxa por tick — ver `decay_per_tick`.
        // O default NÃO foi escolhido: `34` ticks é o que a taxa `0.85` que já shipava
        // produzia, então o flash no default é o mesmo de antes.
        //
        // ⚠️ O param mantém o nome de fio `decay` de propósito: renomeá-lo faria o
        // `validate` RECUSAR todo grafo salvo que o sobrescreve (params que o manifesto
        // não declara mais derrubam o documento inteiro — a cicatriz do `motion.color_ramp`
        // na integração de 2026-07-30). Quem o artista lê é o RÓTULO.
        ParamSpec {
            name: "decay",
            default: 34.0,
        },
        // Peak size multiplier at full glow: size *= 1 + size_boost·glow.
        ParamSpec {
            name: "size_boost",
            default: 0.8,
        },
        // Flash colour + how much of it to mix at full glow (0 = size-only).
        ParamSpec {
            name: "flash_r",
            default: 1.0,
        },
        ParamSpec {
            name: "flash_g",
            default: 1.0,
        },
        ParamSpec {
            name: "flash_b",
            default: 1.0,
        },
        ParamSpec {
            name: "flash_amount",
            default: 0.9,
        },
    ],
    lowerings: &[LoweringKind::Cpu],
};

struct Params {
    decay: f32,
    size_boost: f32,
    flash: [f32; 3],
    flash_amount: f32,
}

/// **O PISO do brilho** — um nível de 8 bits. Abaixo dele o `glow` não levanta um pixel
/// de tamanho nem mistura uma cor visível, então é ali que o flash ACABOU. Mesmo número
/// que o `motion.trail` usa para a ponta da cauda: uma lei, dois nós.
const GLOW_FLOOR: f32 = 1.0 / 255.0;

/// **A taxa por tick que apaga o brilho em `ticks` ticks.**
///
/// ⚠️ O knob é a DURAÇÃO do flash, nunca a taxa — o report de 2026-08-08 mediu o preço de
/// expor a taxa: no curso `0..0.99` do controle antigo os primeiros **86%** cobriam
/// 5..34 ticks e os últimos **14%** cobriam 34..551. *Catorze por cento do slider
/// carregava noventa e quatro por cento da faixa.* Agora ele é linear no que se vê.
///
/// ⚠️ **TICKS e não segundos, e a escolha é sobre RISCO.** Segundos exigiriam `ctx.dt()`,
/// que é `0.0` dentro de um time scope e cuja unidade depende do relógio que o chamador
/// passa ao cook — e um `dt` de zero faria a taxa virar `1.0`, isto é **o flash nunca
/// apagaria**. Uma regressão dessas é muito pior que um slider mal escalado, e o
/// vocabulário do módulo já é o tick: o `motion.trail` mede a cauda dele em ticks
/// (`length`, `spacing`) pela mesma razão.
fn decay_per_tick(ticks: f32) -> f32 {
    if !ticks.is_finite() || ticks <= 0.0 {
        // O degenerado que o artista quer dizer com zero: um flash de UM tick.
        return 0.0;
    }
    libm::powf(GLOW_FLOOR, 1.0 / ticks)
}

/// The envelope value for one instance this tick: a pulse re-arms it to 1.0,
/// otherwise it decays geometrically from last tick's value.
fn glow_of(pulse: f32, prev_glow: f32, decay: f32) -> f32 {
    if pulse > 0.5 { 1.0 } else { prev_glow * decay }
}

fn step(input: &Stream, pulse: &Stream, state: &Stream, p: &Params) -> Stream {
    let n = input.count();
    let pulses = scalar_col(pulse, PULSE_COL, n, 0.0);
    let prev_glow = scalar_col(state, GLOW_COL, n, 0.0);
    // The focus field masks the APPLICATION (like every behaviour): a dot at
    // falloff 0 never visibly flashes. The envelope itself stays unmasked so a
    // falloff animated over a live glow fades the look, not the memory.
    let falloff = scalar_col(input, "falloff", n, 1.0);

    let glow: Vec<f32> = (0..n)
        .map(|i| glow_of(pulses[i], prev_glow[i], p.decay).clamp(0.0, 1.0))
        .collect();

    // Apply the lit look to the FRESH upstream geometry (never the state), so the
    // boost cannot compound; copy every other column through untouched.
    let mut size = vec2_col(input, "size", n, [1.0, 1.0]);
    let mut tint = vec4_col(input, "tint", n, [1.0, 1.0, 1.0, 1.0]);
    for i in 0..n {
        let g = glow[i] * falloff[i].clamp(0.0, 1.0);
        let k = 1.0 + p.size_boost * g;
        size[i] = [size[i][0] * k, size[i][1] * k];
        // Lerp RGB toward the flash colour by amount·glow; alpha untouched (a
        // flash brightens, it does not change opacity).
        let a = p.flash_amount * g;
        for (channel, &target) in tint[i].iter_mut().zip(p.flash.iter()) {
            *channel += (target - *channel) * a;
        }
    }

    let mut out = Stream::new(n);
    for (name, col) in input.columns() {
        if name != "size" && name != "tint" {
            out.set(name.clone(), col.clone());
        }
    }
    out.set("size", Column::Vec2(size));
    out.set("tint", Column::Vec4(tint));
    out.set(GLOW_COL, Column::Scalar(glow));
    out
}

fn scalar_col(s: &Stream, name: &str, n: usize, id: f32) -> Vec<f32> {
    let mut v = match s.get(name) {
        Some(Column::Scalar(v)) => v.clone(),
        _ => Vec::new(),
    };
    v.resize(n, id);
    v
}
fn vec2_col(s: &Stream, name: &str, n: usize, id: [f32; 2]) -> Vec<[f32; 2]> {
    let mut v = match s.get(name) {
        Some(Column::Vec2(v)) => v.clone(),
        _ => Vec::new(),
    };
    v.resize(n, id);
    v
}
fn vec4_col(s: &Stream, name: &str, n: usize, id: [f32; 4]) -> Vec<[f32; 4]> {
    let mut v = match s.get(name) {
        Some(Column::Vec4(v)) => v.clone(),
        _ => Vec::new(),
    };
    v.resize(n, id);
    v
}

struct MotionStrobe;

impl NodeOp for MotionStrobe {
    fn manifest(&self) -> &'static NodeManifest {
        &MANIFEST
    }

    fn eval(&self, ctx: &mut EvalCtx<'_>) {
        let p = Params {
            decay: decay_per_tick(ctx.param("decay")),
            size_boost: ctx.param("size_boost"),
            flash: [
                ctx.param("flash_r"),
                ctx.param("flash_g"),
                ctx.param("flash_b"),
            ],
            flash_amount: ctx.param("flash_amount"),
        };
        let out = step(ctx.input(0), ctx.input(1), ctx.input(2), &p);
        ctx.emit(out);
    }
}

/// Register this node with the runtime registry. Called (via codegen) from
/// `ph2d-node-registry-init::register_all_nodes`.
pub fn register(reg: &mut NodeRegistry) -> Result<(), RegistryError> {
    reg.register(Box::new(MotionStrobe))?;
    reg.register_ui(
        MANIFEST.id,
        ph2d_node_registry::NodeUiManifest {
            display_name: "Strobe",
            // FX magenta: a stylistic flash effect.
            category: ph2d_node_registry::NodeUiCategory::Fx,
            silhouette: ph2d_node_registry::NodeSilhouette::Rect,
        },
    );
    reg.register_param_ui(MANIFEST.id, PARAM_HINTS);
    reg.register_param_units(MANIFEST.id, PARAM_UNITS);
    Ok(())
}

use ph2d_node_registry::{ParamUiHint, ParamUnit, ParamUnitDecl, ParamWidget};

/// O flash é uma contagem de TICKS — a unidade que o painel declara é a que o número É,
/// e é a mesma do `length`/`spacing` do `motion.trail`.
static PARAM_UNITS: &[ParamUnitDecl] = &[ParamUnitDecl {
    param: "decay",
    unit: ParamUnit::Count,
}];

static PARAM_HINTS: &[ParamUiHint] = &[
    ParamUiHint {
        param: "decay",
        label: "Flash Length",
        // Ticks. `0` é o flash de um tick; a 120 (dois segundos a 60 Hz) a coisa deixou
        // de ser um strobe e virou um fade — não há recurso a limitar aqui, o que se
        // fecha é a PALAVRA. Um documento pode carregar mais, e a derivação o honra.
        min: 0.0,
        max: 120.0,
        step: 1.0,
        widget: ParamWidget::Slider,
    },
    ParamUiHint {
        param: "size_boost",
        label: "Size Boost",
        min: 0.0,
        max: 4.0,
        step: 0.05,
        widget: ParamWidget::Slider,
    },
    // The flash colour authored as one swatch → OKLCH picker (the canonical
    // colour UI, like `motion.tint`), driving the three linear channels with
    // `flash_amount` riding the picker's 4th (alpha) slot as the intensity.
    // No standalone `flash_amount` row: the params bridge SUPPRESSES the row of
    // any param folded into a Color group (`motion_bridge_params.rs`,
    // `consumed`), so a dedicated Slider hint here would be dead code.
    ParamUiHint {
        param: "flash_r",
        label: "Flash",
        min: 0.0,
        max: 1.0,
        step: 0.01,
        widget: ParamWidget::Color {
            channels: ["flash_r", "flash_g", "flash_b", "flash_amount"],
        },
    },
];

#[cfg(test)]
mod tests {
    use super::*;

    fn dot() -> Stream {
        Stream::new(1)
            .with("P", Column::Vec2(vec![[0.0, 0.0]]))
            .with("size", Column::Vec2(vec![[1.0, 1.0]]))
            .with("tint", Column::Vec4(vec![[0.2, 0.2, 0.2, 1.0]]))
    }
    fn fire(v: f32) -> Stream {
        Stream::new(1).with(PULSE_COL, Column::Scalar(vec![v]))
    }
    fn params() -> Params {
        Params {
            decay: 0.5,
            size_boost: 1.0,
            flash: [1.0, 1.0, 1.0],
            flash_amount: 1.0,
        }
    }
    fn glow(s: &Stream) -> f32 {
        match s.get(GLOW_COL).unwrap() {
            Column::Scalar(v) => v[0],
            _ => panic!(),
        }
    }
    fn size_x(s: &Stream) -> f32 {
        match s.get("size").unwrap() {
            Column::Vec2(v) => v[0][0],
            _ => panic!(),
        }
    }

    /// A pulse lights the element to full glow, then the envelope decays
    /// geometrically (×decay per tick) — size and flash follow it down. The
    /// upstream geometry is fresh each tick, so the boost never compounds.
    #[test]
    fn a_pulse_lights_then_the_envelope_decays_geometrically() {
        let p = params();
        // Tick 0: fire → glow 1.0, size ×(1+1·1)=2.0.
        let s = step(&dot(), &fire(1.0), &Stream::new(1), &p);
        assert_eq!(glow(&s), 1.0);
        assert_eq!(size_x(&s), 2.0);
        // Tick 1: no fire → glow ×0.5 = 0.5, size ×1.5 (from the FRESH unit size).
        let s = step(&dot(), &fire(0.0), &s, &p);
        assert_eq!(glow(&s), 0.5);
        assert_eq!(size_x(&s), 1.5);
        // Tick 2: glow 0.25, size ×1.25.
        let s = step(&dot(), &fire(0.0), &s, &p);
        assert_eq!(glow(&s), 0.25);
        assert_eq!(size_x(&s), 1.25);
    }

    /// FALSIFICATION of the "apply to fresh upstream, not to state" rule: the
    /// size boost must not COMPOUND. After a pulse and one decay tick, size is
    /// 1.5 (unit × 1.5), NOT 2.0 × 1.5 = 3.0 (which is what re-boosting the
    /// already-boosted state would give).
    #[test]
    fn the_size_boost_does_not_compound_across_ticks() {
        let p = params();
        let s = step(&dot(), &fire(1.0), &Stream::new(1), &p); // size 2.0
        let s = step(&dot(), &fire(0.0), &s, &p);
        assert_eq!(
            size_x(&s),
            1.5,
            "boost applies to fresh geometry, not to 2.0"
        );
    }

    /// At full glow the tint reaches the flash colour (amount 1.0); with no glow
    /// it is the untouched upstream tint. Alpha is never touched.
    #[test]
    fn the_flash_lerps_rgb_toward_the_flash_colour_leaving_alpha_alone() {
        let p = params();
        let lit = step(&dot(), &fire(1.0), &Stream::new(1), &p);
        match lit.get("tint").unwrap() {
            Column::Vec4(v) => {
                assert_eq!(v[0], [1.0, 1.0, 1.0, 1.0], "full flash = white, alpha kept")
            }
            _ => panic!(),
        }
        // Idle (no pulse, glow 0) → the upstream tint, verbatim.
        let dark = step(&dot(), &fire(0.0), &Stream::new(1), &p);
        match dark.get("tint").unwrap() {
            Column::Vec4(v) => assert_eq!(v[0], [0.2, 0.2, 0.2, 1.0]),
            _ => panic!(),
        }
    }

    /// A re-pulse mid-decay re-arms to full glow (the envelope retriggers, like a
    /// bang restarting a `line~`). Not additive — it resets, it does not stack.
    #[test]
    fn a_re_pulse_retriggers_the_envelope_to_full() {
        let p = params();
        let s = step(&dot(), &fire(1.0), &Stream::new(1), &p);
        let s = step(&dot(), &fire(0.0), &s, &p); // glow 0.5
        let s = step(&dot(), &fire(1.0), &s, &p); // re-fire
        assert_eq!(glow(&s), 1.0, "retrigger resets to full, not 0.5+something");
    }

    /// The focus field gates the flash: two dots pulse together, but the one at
    /// falloff 0 keeps its plain look while its neighbour lights up. The glow
    /// MEMORY is per-instance and unmasked (the envelope keeps decaying), only
    /// the applied look is masked — so animating the field over a live glow
    /// fades the flash in and out without restarting it.
    #[test]
    fn falloff_zero_gates_the_flash_without_touching_the_envelope() {
        let p = params();
        let two = Stream::new(2)
            .with("P", Column::Vec2(vec![[0.0, 0.0], [1.0, 0.0]]))
            .with("size", Column::Vec2(vec![[1.0, 1.0]; 2]))
            .with("tint", Column::Vec4(vec![[0.2, 0.2, 0.2, 1.0]; 2]))
            .with("falloff", Column::Scalar(vec![1.0, 0.0]));
        let fire2 = Stream::new(2).with(PULSE_COL, Column::Scalar(vec![1.0, 1.0]));
        let s = step(&two, &fire2, &Stream::new(2), &p);
        match s.get("size").unwrap() {
            Column::Vec2(v) => {
                assert_eq!(v[0], [2.0, 2.0], "focused dot flashes");
                assert_eq!(v[1], [1.0, 1.0], "masked dot stays plain");
            }
            _ => panic!(),
        }
        match s.get("tint").unwrap() {
            Column::Vec4(v) => {
                assert_eq!(v[0], [1.0, 1.0, 1.0, 1.0], "focused dot lights up");
                assert_eq!(v[1], [0.2, 0.2, 0.2, 1.0], "masked dot untouched");
            }
            _ => panic!(),
        }
        // The envelope itself is NOT masked: both instances carry full glow.
        match s.get(GLOW_COL).unwrap() {
            Column::Scalar(v) => assert_eq!(v, &vec![1.0, 1.0], "memory unmasked"),
            _ => panic!(),
        }
    }

    /// A bare positional stream (no size/tint) still gets those columns created
    /// at their identities and modulated — the strobe is not a no-op on a
    /// generator that only emits P.
    #[test]
    fn a_bare_stream_gains_size_and_tint_at_their_identities() {
        let p = params();
        let bare = Stream::new(1).with("P", Column::Vec2(vec![[5.0, 6.0]]));
        let lit = step(&bare, &fire(1.0), &Stream::new(1), &p);
        assert_eq!(size_x(&lit), 2.0, "unit identity ×2");
        match lit.get("P").unwrap() {
            Column::Vec2(v) => assert_eq!(v[0], [5.0, 6.0], "P passes through"),
            _ => panic!(),
        }
    }

    /// **O DEFAULT REPRODUZ O FLASH QUE JÁ SHIPAVA**, e não foi escolhido.
    ///
    /// `34` ticks derivam exatamente a taxa `0.85` que o manifesto trazia — o strobe no
    /// default não muda um pixel, e o que muda é que o número passou a ser linear no que
    /// se vê.
    #[test]
    fn the_default_reproduces_the_shipped_rate() {
        let d = MANIFEST
            .params
            .iter()
            .find(|p| p.name == "decay")
            .expect("param")
            .default;
        assert_eq!(d, 34.0);
        let rate = decay_per_tick(d);
        assert!((rate - 0.85).abs() < 1e-3, "taxa derivada: {rate}");
    }

    /// **O FLASH DURA O QUE O SLIDER DIZ** — o gate do report de 2026-08-08.
    ///
    /// ⚠️ Nasceu VERMELHO: o knob ERA a taxa, então `0.5` durava 8 ticks e `0.99` durava
    /// 551 — a mesma unidade de curso do slider comprando dois tempos que diferem por
    /// setenta vezes. O oráculo é o RELÓGIO: quantos ticks até o brilho chegar ao piso.
    #[test]
    fn the_flash_lasts_as_long_as_the_slider_says() {
        for want in [5u32, 15, 34, 60, 120] {
            let rate = decay_per_tick(want as f32);
            let (mut glow, mut ticks) = (1.0f32, 0u32);
            while glow > GLOW_FLOOR && ticks < 10_000 {
                glow = glow_of(0.0, glow, rate);
                ticks += 1;
            }
            assert!(
                ticks.abs_diff(want) <= 1,
                "{want} ticks pedidos, {ticks} medidos"
            );
        }
    }

    /// **O SLIDER É LINEAR NO QUE SE VÊ** — dobrar o número dobra o flash.
    ///
    /// ⚠️ É a propriedade que a lei antiga não tinha: lá `0.5` durava 8 ticks e `0.99`
    /// durava 551, então uma mesma fração do curso comprava tempos que diferiam por
    /// setenta vezes.
    #[test]
    fn doubling_the_number_doubles_the_flash() {
        let life = |ticks: f32| {
            let rate = decay_per_tick(ticks);
            let (mut glow, mut n) = (1.0f32, 0u32);
            while glow > GLOW_FLOOR && n < 10_000 {
                glow = glow_of(0.0, glow, rate);
                n += 1;
            }
            n as f32
        };
        for base in [10.0f32, 20.0, 40.0] {
            let r = life(base * 2.0) / life(base);
            assert!((r - 2.0).abs() < 0.1, "base {base}: a razao deu {r}");
        }
    }

    /// O degenerado: duração zero (ou lixo de documento) é um flash de UM tick.
    #[test]
    fn the_degenerate_lengths_mean_what_the_artist_means() {
        assert_eq!(decay_per_tick(0.0), 0.0, "zero = um tick");
        assert_eq!(decay_per_tick(-3.0), 0.0);
        assert_eq!(decay_per_tick(f32::NAN), 0.0);
        assert_eq!(decay_per_tick(f32::INFINITY), 0.0);
    }

    /// **A DURAÇÃO AUTORADA ATRAVESSA O COOK** — a porta, não a lei.
    ///
    /// ⚠️ Os seis gates que este arquivo já tinha constroem `Params` com a TAXA à mão, e
    /// são portanto cegos a um `decay_per_tick` que ninguém tivesse ligado no `eval`: a
    /// capacidade passaria em todos eles nascendo morta. Este dirige o grafo REAL, com o
    /// `pre` self-loop, e mede o brilho depois de um pulso.
    #[test]
    fn the_authored_length_reaches_the_node_through_the_cook() {
        use ph2d_nodegraph::cook::{Cook, OpResolver};
        use ph2d_nodegraph::graph::{Edge, Graph};

        // ⚠️ DUAS fontes, como no produto: a geometria fala `Clock::Frame` e o pulso
        // fala `Clock::Event`. A 1ª versao deste gate cravou os dois na MESMA fonte de
        // Frame — o cook serviu o pulso do cache e ele ficou aceso para sempre, com o
        // brilho pregado em 1 nos dois bracos e o gate reprovando produto correto.
        static SRC: NodeManifest = NodeManifest {
            id: NodeTypeId::of("motion.strobe.test.src"),
            name: "motion.strobe.test.src",
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
        static PSRC: NodeManifest = NodeManifest {
            id: NodeTypeId::of("motion.strobe.test.pulse"),
            name: "motion.strobe.test.pulse",
            inputs: &[],
            outputs: &[PortSpec {
                name: "out",
                ty: PULSE,
            }],
            // ⚠️ `Temporal`, nao `Pure`: o fingerprint do cook so inclui o playhead para
            // um no TEMPORAL (`cook.rs`, `playhead: (effect == Temporal).then_some(..)`).
            // Com `Pure` esta fonte era servida do CACHE do tick 0 para sempre, o pulso
            // ficava aceso em todo tick, o brilho pregava em 1 nos dois bracos e o gate
            // reprovava produto correto. A fixture tem de dizer a verdade sobre si mesma.
            effect: Effect::Temporal,
            clock: Clock::Event,
            params: &[],
            lowerings: &[LoweringKind::Cpu],
        };
        struct Src;
        impl NodeOp for Src {
            fn manifest(&self) -> &'static NodeManifest {
                &SRC
            }
            fn eval(&self, ctx: &mut EvalCtx<'_>) {
                ctx.emit(
                    Stream::new(1)
                        .with("P", Column::Vec2(vec![[0.0, 0.0]]))
                        .with("size", Column::Vec2(vec![[1.0, 1.0]])),
                );
            }
        }
        struct PulseSrc;
        impl NodeOp for PulseSrc {
            fn manifest(&self) -> &'static NodeManifest {
                &PSRC
            }
            fn eval(&self, ctx: &mut EvalCtx<'_>) {
                // Um pulso no tick 0 e mais nada: o brilho decai a partir dali.
                let fire = f32::from(u8::from(ctx.playhead() < 0.5));
                ctx.emit(Stream::new(1).with(PULSE_COL, Column::Scalar(vec![fire])));
            }
        }
        struct Ops;
        impl OpResolver for Ops {
            fn resolve(&self, ty: NodeTypeId) -> Option<&dyn NodeOp> {
                match ty {
                    t if t == SRC.id => Some(&Src),
                    t if t == PSRC.id => Some(&PulseSrc),
                    t if t == MANIFEST.id => Some(&MotionStrobe),
                    _ => None,
                }
            }
        }

        // Duas cenas identicas, so a DURACAO difere: a longa tem de brilhar mais no
        // mesmo tick. Um `eval` que ignorasse a derivacao daria o mesmo nos dois.
        let glow_at = |seconds: f32, at: u32| {
            let mut g = Graph::new();
            let src = g.add_node("motion.strobe.test.src");
            let psrc = g.add_node("motion.strobe.test.pulse");
            let st = g.add_node("motion.strobe");
            g.connect(Edge {
                from: (src, 0),
                to: (st, 0),
                delayed: false,
            })
            .unwrap();
            g.connect(Edge {
                from: (psrc, 0),
                to: (st, 1),
                delayed: false,
            })
            .unwrap();
            g.connect(Edge {
                from: (st, 0),
                to: (st, 2),
                delayed: true,
            })
            .unwrap();
            g.set_param(st, "decay", seconds);
            g.set_param(st, "size_boost", 1.0);
            let mut cook = Cook::new();
            let mut last = Stream::new(0);
            for t in 0..=at {
                last = cook.cook(&g, &Ops, st, f64::from(t)).unwrap()[0]
                    .as_stream()
                    .clone();
                cook.advance_tick(&g, &Ops, f64::from(t)).unwrap();
            }
            // `size = 1 + size_boost*glow`, entao o tamanho LE o brilho.
            match last.get("size").unwrap() {
                Column::Vec2(v) => v[0][0] - 1.0,
                _ => panic!("size"),
            }
        };
        let (short, long) = (glow_at(3.0, 8), glow_at(120.0, 8));
        assert!(
            long > short * 2.0,
            "a duracao autorada nao chegou ao no: curta {short}, longa {long}"
        );
    }
}
