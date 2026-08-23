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

use ph2d_curve::Curve;
use ph2d_node_registry::{NodeRegistry, ParamChannelRange, RegistryError};
use ph2d_nodegraph::cook::EvalCtx;
use ph2d_nodegraph::effect::Effect;
use ph2d_nodegraph::node::{LoweringKind, NodeManifest, NodeOp, NodeTypeId, ParamSpec, PortSpec};
use ph2d_nodegraph::port::{Clock, Dim, Domain, PortType};

mod gpu;
use gpu::GPU_KERNEL;
mod params_ui;
use params_ui::{PARAM_GATES, PARAM_GROUPS, PARAM_HARD_MAX, PARAM_HINTS, PARAM_UNITS};
mod channel;
use channel::{apply_channel_delta, clock_at, falloff_at, scalar_values};
use ph2d_nodegraph::attr::par_build;

const INST_VEC2: PortType = PortType::new(Domain::Instances, Dim::Vec2, Clock::Frame);
/// O tipo da porta `time` — espelho local do `VALUE` do `motion.drive`. Esta é uma
/// crate-folha: o vocabulário partilhado é a **porta**, nunca um símbolo importado.
const VALUE: PortType = PortType::new(Domain::Instances, Dim::Scalar, Clock::Frame);
/// A coluna que um stream de valor carrega (o que o `value.time` emite).
const VALUE_COL: &str = "v";

/// The static contract of this node type (ADR-0031).
pub const MANIFEST: NodeManifest = NodeManifest {
    id: NodeTypeId::of("motion.oscillator"),
    name: "motion.oscillator",
    inputs: &[
        PortSpec {
            name: "in",
            ty: INST_VEC2,
        },
        // ⚠️ **A PORTA DE TEMPO, e ela é per-ELEMENTO** (folha 06, `SUPERAR 1`).
        // Desligada ⇒ `ctx.playhead()`, **byte-idêntico** — o precedente exacto é a
        // porta `drive` do `motion.wave` e a `offset` do `motion.path`.
        //
        // ⚠️ **APENDADA, nunca inserida.** As arestas de um documento salvo guardam o
        // ÍNDICE da porta; a porta 0 continua a 0, e um doc de ontem abre igual. É a
        // mesma lei do enum de canal do `motion.drive`.
        //
        // Ela entrega de uma vez o *Time* / *Time Offset* / *Time Scale* do Cavalry
        // **sem um knob novo** — o `value.time` já tem `rate`/`offset`/`stagger` —, e
        // faz o LOOP ser fechado **por construção** (`value.time → value.wrap → time`:
        // `t` e `t+L` passam a ser o mesmo número, em vez de um cross-fade que
        // aproxima). E revoga a cerca 2 da folha: *sem uma porta de tempo externa,
        // `sin(2π(s·t)f) ≡ sin(2π·t·(s·f))`* — com ela, escalar o CAMPO a montante
        // deixa de ser identidade algébrica com `frequency`.
        //
        // ⚠️ **Não é o escopo de tempo do `motion.time_remap`** (cerca 6): aquele
        // recozinha uma sub-árvore e por isso **recusa** um nó sequencial a montante
        // (`CookError::SequentialInTimeScope`). Isto é uma COLUNA — o nó lê um número
        // por elemento, não há segundo cozimento, e nada é recusado.
        PortSpec {
            name: "time",
            ty: VALUE,
        },
    ],
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
        ParamSpec {
            name: "time_mode",
            default: 0.0,
        },
        // ⚠️ **UM knob para o que a referência dá em DOIS.** O TouchDesigner tem
        // *Pulse Width* (a fração do período em que a Square fica em cima) e
        // *Bias* (onde o pico da Triangle/Saw se senta) — e eles são **o mesmo
        // número**: a fatia do ciclo gasta na primeira metade. Dois nomes para um
        // número é como um artista aprende que são coisas diferentes.
        ParamSpec {
            name: "pulse_width",
            default: 0.5,
        },
        ParamSpec {
            name: "bpm",
            default: 120.0,
        },
        // ⚠️ **Apendados**: a FAIXA, a régua alternativa de `amplitude`+`offset`
        // (Cavalry *Minimum/Maximum*). `0` = a régua que sempre shipou.
        // Ver [`natural_range`] — e a armadilha que ela cura.
        ParamSpec {
            name: "range_mode",
            default: 0.0,
        },
        ParamSpec {
            name: "min",
            default: -1.0,
        },
        ParamSpec {
            name: "max",
            default: 1.0,
        },
    ],
    lowerings: &[LoweringKind::Cpu],
};

/// **A FAIXA NATURAL de cada forma** — o que ela produz antes de `amplitude` e
/// `offset`, e o número que um controle de `Min`/`Max` precisa de saber.
///
/// ⚠️ **É por causa dele que a aritmética óbvia do artista está ERRADA**, e a
/// folha 06 registou isto como *"armadilha real"*: a conta que toda a gente faz —
/// `amplitude = (max−min)/2`, `offset = (min+max)/2` — assume que a onda é
/// bipolar, e o **Spike não é**: ele é um pulso unipolar `[0, 1]`. Com essa conta,
/// trocar `Sine` por `Spike` **METADE a excursão e levanta o piso ao centro da
/// faixa**, em silêncio, sem que nenhum número do painel tenha mudado.
///
/// O irmão `motion.noise` tem exactamente a mesma assimetria (`Turbulence` e
/// `Ridged` são retificados) e a lei que a resolve mora em
/// [`ph2d_fbm::gain_offset_for_range`]. Aqui ela é ESPELHADA e não importada: este
/// nó é uma drop-crate folha e não depende do ruído — o mesmo motivo pelo qual
/// `VALUE` e `field_at` são espelhados por toda a família.
fn natural_range(wave: i32) -> (f32, f32) {
    // 4 = Spike, o pulso unipolar; 5 = Custom, que o editor autora no QUADRADO
    // UNITÁRIO (`Curve::eval` grampeia o domínio a `[0,1]` e a curva da casa nasce
    // na diagonal). As outras quatro são `[-1, 1]`.
    //
    // ⚠️ **A `Custom` tinha de responder aqui, e a resposta não é «igual às outras».**
    // A folha 06 registou a armadilha do `Spike` — a conta bipolar `amp=(max−min)/2`
    // entrega metade da faixa com o piso ao centro, em silêncio — e uma forma NOVA
    // que não declarasse a polaridade dela reabriria exactamente esse defeito. Ver
    // [`waveform`].
    if wave == 4 || wave == WAVE_CUSTOM {
        (0.0, 1.0)
    } else {
        (-1.0, 1.0)
    }
}

/// `amplitude`/`offset` derivados da faixa pedida — o gêmeo de
/// [`ph2d_fbm::gain_offset_for_range`], espelhado (ver [`natural_range`]).
fn gain_offset_for_range(natural: (f32, f32), min: f32, max: f32) -> (f32, f32) {
    let (lo, hi) = natural;
    let span = hi - lo;
    if span == 0.0 {
        return (0.0, min);
    }
    let gain = (max - min) / span;
    (gain, min - lo * gain)
}

/// The fractional part of `p` in `[0,1)` — IEEE `floor` is correctly-rounded and
/// deterministic (HR-5-safe, unlike `sin`).
fn frac(p: f32) -> f32 {
    p - p.floor()
}

/// A periodic waveform at `phase` (in cycles, period 1) — bipolar `[-1,1]` except
/// **Spike** (a unipolar `[0,1]` pulse). All shapes are piecewise polynomial →
/// transcendental-free (HR-5). Unknown / `0` is the parabolic sine-approximation.
/// O piso/teto do `pulse_width`.
///
/// ⚠️ **Ele NÃO é uma guarda contra divisão por zero, e a mutação me corrigiu:**
/// eu escrevera que *"a lei divide por `p` e por `1 − p`, então os extremos são
/// uma divisão por zero"* — a estrutura dos ramos já protege (com `p = 0` o
/// ramo que divide por `p` é inalcançável, e com `p = 1` o outro também), e
/// apagar o clamp deixava tudo FINITO e os quatro gates VERDES.
///
/// O que ele compra é a onda continuar a ser uma onda: este param é `f32`, logo
/// **dirigível por fio** (doc 58), e um `pw = 7` sem clamp comprime o ciclo
/// inteiro nos primeiros 7% do domínio da forma — a excursão colapsa e a saída
/// fica quase plana. A 5% de um ciclo a onda já é uma agulha, e é aí que o
/// disfuncional começa.
const PW_MIN: f32 = 0.05;

/// **O WARP DE FASE** — o que faz *Pulse Width* e *Bias* serem um knob só.
///
/// Ele estica a primeira fatia do ciclo (`[0, pw]`) sobre a primeira metade
/// (`[0, ½]`) e comprime o resto na segunda. A forma que vem depois não sabe que
/// isto aconteceu: a Square ganha o ciclo de trabalho, a Triangle e a Saw ganham
/// o *bias* (o pico anda), e a senoide ganha a versão enviesada dela — **cinco
/// formas de uma lei**.
///
/// ⚠️ **`pw = 0.5` é a IDENTIDADE, e por ARITMÉTICA:** `0.5 / 0.5` é exactamente
/// `1.0`, então o primeiro ramo é `f * 1.0 = f`; e no segundo `f − 0.5` é exacto
/// (Sterbenz, os dois estão dentro de um fator de dois) e `0.5 + (f − 0.5)`
/// reconstrói `f` ao bit. É isto que faz o default não mover um pixel.
fn skew(f: f32, pulse_width: f32) -> f32 {
    let p = pulse_width.clamp(PW_MIN, 1.0 - PW_MIN); // CLAMP-OK: ver `PW_MIN`
    if f < p {
        f * (0.5 / p)
    } else {
        0.5 + (f - p) * (0.5 / (1.0 - p))
    }
}

/// **A SEXTA FORMA: a que o artista DESENHA** (doc 89, folha 06 · Cavalry *Wave
/// Style ▸ Custom (Graph)*).
///
/// Ela é o índice `5` do `wave`, e a forma vive num **text param** ([`CURVE_KEY`]) —
/// uma curva não é um número, e é o mesmo canal do `value.curve` / `field.remap` /
/// `motion.strobe`. ⚠️ **Curva não-setada = a IDENTIDADE**, ou seja `y = f`: uma serra
/// unipolar `0 → 1`. Isso é uma onda de facto (não um controle morto), e é a lei que o
/// `value.curve` já pratica.
///
/// ⚠️ **O `pulse_width` continua a valer**, porque ele é um warp da FASE e não da forma:
/// a `Custom` entra como a sexta consumidora de [`skew`] sem uma linha de código a mais,
/// e o artista ganha *bias* sobre o desenho dele de graça.
pub const WAVE_CUSTOM: i32 = 5;

/// A chave do text param que carrega a forma da onda [`WAVE_CUSTOM`] (uma string
/// `ph2d-curve`, autorada pelo editor `ParamWidget::Curve`). **NÃO** é um `ParamSpec` —
/// uma curva não é um número.
pub const CURVE_KEY: &str = "curve";

fn waveform(kind: i32, phase: f32, pulse_width: f32, curve: Option<&Curve>) -> f32 {
    let f = skew(frac(phase), pulse_width);
    match kind {
        // A forma DESENHADA — ver [`WAVE_CUSTOM`]. Sem curva, `eval` de uma `Curve`
        // vazia devolve o próprio `f`, então este ramo é a serra unipolar e nunca
        // um valor morto.
        WAVE_CUSTOM => curve.map_or(f, |c| c.eval(f)),
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

/// **Ciclos por segundo, na régua que o artista escolheu** (`time_mode`: `0` segundos,
/// `1` BPM).
///
/// ⚠️ Isto NÃO é um segundo multiplicador de frequência — é a UNIDADE do mesmo número, a
/// mesma família do px/m da Wave A. A distinção importa porque o Cavalry também traz um
/// *Time Scale*, e esse **não foi construído de propósito**: sem uma porta de tempo externa,
/// `sin(2π·(s·t)·f) ≡ sin(2π·t·(s·f))`, ou seja Time Scale É Frequency por identidade
/// algébrica — um knob que não pode mudar nada que o outro não mude.
fn cycles_per_second(mode: f32, frequency: f32, bpm: f32) -> f32 {
    if mode >= 0.5 { bpm / 60.0 } else { frequency }
}

// ⛔ **O `fade` foi construído e REMOVIDO no mesmo dia** (smoke do Enio, doc 88 B3) — não
// reconstrua sem ler isto. Era o *Strength Fade to Zero* do Cavalry portado como uma rampa em
// segundos a partir do zero ABSOLUTO do playhead, e tinha três defeitos, o primeiro MEDIDO:
//
// 1. **Expirava.** O slider ia até 10 s, então a partir de ~10 s de relógio TODO valor da
//    faixa entregava amplitude zero — o estado permanente do controle era *expirado*, e na
//    tela isso lê como *"o oscilador travou tudo"*, que foi o report.
// 2. **A régua era invisível.** No Cavalry o fade desvanece ao longo da DURAÇÃO DA
//    COMPOSIÇÃO — uma janela com começo e fim na régua. Aqui a janela começava num zero que
//    o artista não vê e terminava num instante que ele não vê; a duração da composição **não
//    existe neste nível** (`EvalCtx` oferece `playhead`/`dt`/`started`, e nada mais).
// 3. **Era uma SEGUNDA PORTA.** `ctx.param` resolve **wire > override > default**, então
//    `value.time → value.map_range → amplitude` já É um fade — com a régua VISÍVEL no grafo,
//    o começo e o fim escolhidos pelo artista, e o painel marcando o param como *driven*.
//
// ⚠️ O `time_mode`/`bpm` FICA, e a distinção é o que separa os dois: uma UNIDADE não expira.
// BPM é a mesma frequência noutra régua e vale igual no segundo 0 e no segundo 600.
//
// O gate `no_control_of_this_oscillator_expires_with_the_clock` guarda a CLASSE, não este
// campo: qualquer knob futuro cuja unidade seja *"segundos desde um zero que ninguém vê"*
// nasce vermelho nele.

struct MotionOscillator;

impl NodeOp for MotionOscillator {
    fn manifest(&self) -> &'static NodeManifest {
        &MANIFEST
    }

    fn eval(&self, ctx: &mut EvalCtx<'_>) {
        let channel = ctx.param("channel").round() as i32;
        let wave = ctx.param("wave").round() as i32;
        let phase_stagger = ctx.param("phase_stagger");
        let offset = ctx.param("offset");
        let phase0 = ctx.param("phase");
        let t = ctx.playhead() as f32;
        let cps = cycles_per_second(
            ctx.param("time_mode"),
            ctx.param("frequency"),
            ctx.param("bpm"),
        );
        // A FAIXA: a régua alternativa. Desligada, os dois números são os de sempre.
        let (amplitude, offset) = if ctx.param("range_mode") >= 0.5 {
            gain_offset_for_range(natural_range(wave), ctx.param("min"), ctx.param("max"))
        } else {
            (ctx.param("amplitude"), offset)
        };
        // ⚠️ Lido AQUI e não dentro do laço: ele é uniforme no dispatch inteiro.
        let pulse_width = ctx.param("pulse_width");
        // A forma DESENHADA (`wave = Custom`). Parseada UMA vez, fora do laço — ela é
        // uniforme no dispatch, como o `pulse_width`, e um parse por elemento seria o
        // custo de uma string por instância.
        let curve = ctx.text_param(CURVE_KEY).and_then(ph2d_curve::parse);
        // A porta de TEMPO (opcional): vazia ⇒ `t` para toda instância.
        let times = scalar_values(ctx.input(1), VALUE_COL);
        let out = {
            let input = ctx.input(0);
            let n = input.count();
            debug_assert!(
                matches!(times.len(), 0 | 1) || times.len() == n,
                "a porta `time` tem {} valores para {n} instancias",
                times.len()
            );
            // Pure per-instance map → parallel above the threshold (bit-identical,
            // no reduction). GPU/M5 Fase 0.
            let deltas: Vec<f32> = par_build(n, |i| {
                let phase = clock_at(&times, i, t) * cps + i as f32 * phase_stagger + phase0;
                // DC `offset` shifts the oscillation centre; the whole
                // contribution is falloff-masked (like every behaviour).
                (waveform(wave, phase, pulse_width, curve.as_ref()) * amplitude + offset)
                    * falloff_at(input, i)
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
    reg.register_param_channel_range(MANIFEST.id, PARAM_CHANNEL_RANGE);
    reg.register_param_hard_max(MANIFEST.id, PARAM_HARD_MAX);
    reg.register_param_gates(MANIFEST.id, PARAM_GATES);
    reg.register_param_groups(MANIFEST.id, PARAM_GROUPS);
    reg.register_param_units(MANIFEST.id, PARAM_UNITS);
    // GPU/M5 Fase 1 (ADR-0126): the WGSL lowering, registered on the side.
    reg.register_gpu_kernel(MANIFEST.id, GPU_KERNEL);
    // A1-gpu: a tabela da onda `Custom`, para o `osc_wave` a ler no device em vez de
    // o nó inteiro cair para a CPU (o precedente é o contorno Curve do `field.remap`).
    reg.register_luts(MANIFEST.id, gpu::LUTS);
    Ok(())
}

/// **A faixa que estas magnitudes querem quando o canal é ANGULAR** — graus, não
/// unidades de mundo. Uma volta para cada lado, discada em graus inteiros.
///
/// ⚠️ Ela mora AQUI e não numa tabela do shell porque a tabela apodreceu: medida,
/// ela cobria três dos seis nós que precisavam dela, e cada um dos três ausentes
/// esperava o próprio report do artista.
const TURN: f32 = 360.0;
static PARAM_CHANNEL_RANGE: &[ParamChannelRange] = &[
    ParamChannelRange {
        param: "offset",
        min: -TURN,
        max: TURN,
        step: 1.0,
    },
    ParamChannelRange {
        param: "amplitude",
        min: 0.0,
        max: TURN,
        step: 1.0,
    },
];

#[cfg(test)]
#[path = "lib_tests.rs"]
mod tests;

#[cfg(test)]
#[path = "skew_tests.rs"]
mod skew_tests;
