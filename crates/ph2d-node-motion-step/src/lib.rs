//! `motion.step` — a PULSE train steps a channel through a persistent staircase
//! (Motion Nodes M2, pulse family 3/n; decision doc `08_pulse_counter_*`,
//! renamed from `pulse.counter` by handoff doc `09_handoff_pulse_*` §4.2).
//!
//! `pulse.threshold` turns a signal into a pulse (value→event); `motion.strobe`
//! turns a pulse into a *momentary* flash (event→frame, decaying). `motion.step`
//! turns a pulse into a *persistent, accumulated* displacement — the count
//! survives and grows across ticks, the inverse of the threshold and the thing
//! that lets an event drive continuous motion. The counting core is every mature
//! tool's counter (TouchDesigner / Houdini **Count CHOP**, Max **counter**,
//! Cavalry **Timeline Counter**) — toggle is `count & 1`, sequence is `count mod
//! N` — but this node also *applies* `count · step` to a chosen channel, so it
//! is a visible **behaviour** (hence `motion.*`), not the pure pulse→value
//! reducer those tools call "counter". The `pulse.counter` name is deliberately
//! left FREE for that pure reducer, for when a scalar value domain exists to
//! carry its output (doc 09 §4.3).
//!
//! **The state is a monotonic tick, not the folded count.** A per-instance
//! integer `count_tick` (+1 on each pulse rising edge) rides the `state` pre
//! self-loop, alongside `count_prev` (last tick's pulse value, for edge safety).
//! The *displayed* count is derived from the tick + limit mode every tick, so all
//! three modes fall out of one state: **Wrap** `tick mod N` (the staircase zeroes,
//! TD Loop Min/Max), **Clamp** `min(tick, N-1)` (it plateaus, TD Clamp),
//! **Zigzag** a triangle of period `2(N-1)` (it ping-pongs, TD Zigzag). The
//! displacement `count · step` is added to the chosen channel (the shared
//! `apply_channel_delta`, falloff-masked), applied to the FRESH `in` each tick so
//! it never compounds; the reducer output rides out as the `count` column.
//!
//! **Edge-safe by construction (the whole correctness point):** the count only
//! advances on the pulse's rising edge (`pulse > 0.5 && prev <= 0.5`), never
//! "while high" — TD's `Off to On` vs `While On`. `pulse.threshold` already emits
//! single-tick pulses, but the step is robust to any producer (a sustained
//! Cavalry-style 0/1 level counts once, not once-per-tick).
//!
//! ## The `reset` port — and why it is LEVEL where the count is EDGE
//!
//! A monotonic tick with nothing that can zero it is a counter that only ever
//! climbs: lowering `count_max` just refolds the same tick, and no other node in
//! the catalogue can write another node's `pre` state. Every mature counter ships
//! the way back — TD's **Count CHOP** has *Reset* + *Reset Condition* + *Reset
//! Value*, and the minicavalry `counter`/`stateMachine` both list
//! `reset(pulse)`. A fourth input takes one: high ⇒ the tick becomes `reset_to`.
//!
//! ⚠️ **The count needs the edge and the reset does not, and that asymmetry is
//! the design:** *the edge matters where the effect ACCUMULATES*. Counting
//! accumulates, so a held level must count once; resetting is **idempotent**, so
//! a held level simply holds the counter down — which is exactly TD's `While On`,
//! while a single-tick pulse is exactly its `Off to On` button. One law gives
//! both, so there is no *Reset Condition* param to choose between them and no
//! second edge-memory column to carry.
//!
//! ⚠️ **The reset WINS its tick:** a reset arriving alongside a pulse lands on
//! `reset_to`, not `reset_to + 1` — the order is the meaning of the word.
//! It does **not** touch `count_prev`: a counting pulse that was already high
//! stays counted, so no phantom edge is manufactured when the reset lets go.
//!
//! **An unwired `reset` is byte-identical** (the port cooks to an empty stream,
//! the column reads `0.0`, and `reset_to` is never consulted), so the port is
//! append-only in every sense: the three existing port indices do not move and
//! every saved graph keeps meaning what it meant.
//!
//! ## Quem é quem: o estado casa por `id`, o pulso difunde (doc 89 folha 07)
//!
//! ✅ **A limitação que este cabeçalho declarava está CURADA**, e eram duas — o
//! diagnóstico separa-as porque a cura de cada uma é outra (mecanismo e tabela em
//! [`pairing`]):
//!
//! - o **`state`** vem do tique ANTERIOR, e entre os dois tiques o conjunto pode
//!   ter girado (uma stream com `id` — o emitter — nasce e morre peças). Ele
//!   casa agora por `id`, como o `motion.integrate`/`motion.spring` sempre
//!   fizeram: uma peça sobrevivente leva o SEU degrau, uma peça nova semeia em 0,
//!   e uma que morreu leva o degrau dela consigo. Sem `id` a identidade é
//!   posicional; uma contagem estável pareia linha a linha (byte-idêntico ao que
//!   shipava) e uma contagem MUDADA re-semeia, porque a linha 1 de uma grelha
//!   2×3 não é a linha 1 de uma 2×2.
//! - as portas **`pulse`/`reset`** vêm do MESMO tique, então não giram; o que
//!   nelas desalinhava era o COMPRIMENTO. Um batimento global (uma linha só) era
//!   esticado com zeros e chegava **apenas ao elemento 0** — o resto do conjunto
//!   ficava parado. Elas leem agora a escada `0/1/n` da casa: ausente → 0,
//!   comprimento 1 → difundido a todos, comprimento n → um por peça.
//!
//! ⚠️ **`count_prev` sai DIFUNDIDO.** É a memória de beira, e se ela guardasse a
//! coluna crua enquanto o `pulse` de hoje é lido difundido, o tique seguinte veria
//! `prev = 0` contra `pulse = 1` em toda peça a partir da linha 1 — uma beira de
//! subida FABRICADA por quadro, e a escada subiria sem parar.

#![forbid(unsafe_code)]

use ph2d_node_registry::{NodeRegistry, ParamChannelRange, RegistryError};
use ph2d_nodegraph::attr::{Column, Stream};
use ph2d_nodegraph::cook::EvalCtx;
use ph2d_nodegraph::effect::Effect;
use ph2d_nodegraph::node::{LoweringKind, NodeManifest, NodeOp, NodeTypeId, ParamSpec, PortSpec};
use ph2d_nodegraph::port::{Clock, Dim, Domain, PortType};

mod channel;
mod pairing;
use channel::{apply_channel_delta, falloff_at};
use pairing::{beat_at, pairing, raw_col};

const INST_VEC2: PortType = PortType::new(Domain::Instances, Dim::Vec2, Clock::Frame);
/// The pulse type (mirror of `ph2d_node_pulse_threshold::PULSE`; kept local so
/// this crate stays a leaf drop-crate — the shared vocabulary is the port
/// `(Instances, Scalar, Event)`, not a shared symbol).
const PULSE: PortType = PortType::new(Domain::Instances, Dim::Scalar, Clock::Event);

/// The pulse stream's fire column (`1.0` on a fired tick).
const PULSE_COL: &str = "pulse";
/// The reducer output: the displayed count this tick (a drivable index).
const COUNT_COL: &str = "count";
/// The monotonic tick carried on the `pre` self-loop (+1 per rising edge).
const TICK_COL: &str = "count_tick";
/// Last tick's pulse value, carried on the `pre` self-loop for edge detection.
const PREV_COL: &str = "count_prev";

/// The static contract of this node type (ADR-0031).
pub const MANIFEST: NodeManifest = NodeManifest {
    id: NodeTypeId::of("motion.step"),
    name: "motion.step",
    inputs: &[
        PortSpec {
            name: "in",
            ty: INST_VEC2,
        },
        PortSpec {
            name: "pulse",
            ty: PULSE,
        },
        // Feedback: last tick's output carries `count_tick` + `count_prev`.
        // Named `state` so the editor plumbs its `pre` self-loop on drop.
        PortSpec {
            name: "state",
            ty: INST_VEC2,
        },
        // APPENDED (see the header): high ⇒ the tick becomes `reset_to`. Last, so
        // the three indices a saved graph already stores keep pointing where they
        // pointed; unwired it cooks to an empty stream and changes nothing.
        PortSpec {
            name: "reset",
            ty: PULSE,
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
        // 0 X · 1 Y · 2 Rotation · 3 Size — the shared channel vocabulary.
        ParamSpec {
            name: "channel",
            default: 0.0,
        },
        // Units added to the channel per count. Negative steps count "down".
        ParamSpec {
            name: "step",
            default: 0.5,
        },
        // Distinct counts in the cycle (the wrap/zigzag period). Clamped ≥1.
        ParamSpec {
            name: "count_max",
            default: 6.0,
        },
        // 0 Wrap · 1 Clamp · 2 Zigzag — the TD Count CHOP limit-mode vocabulary.
        ParamSpec {
            name: "mode",
            default: 0.0,
        },
        // TD's *Reset Value*: where the staircase restarts. **0 is the neutral**
        // point — with the port unwired it is never read at all.
        ParamSpec {
            name: "reset_to",
            default: 0.0,
        },
    ],
    lowerings: &[LoweringKind::Cpu],
};

/// What happens to the count at the top of its range (TD Count CHOP limit modes).
#[derive(Copy, Clone, PartialEq, Eq)]
enum LimitMode {
    /// `tick mod N` — the staircase returns home (TD Loop Min/Max).
    Wrap,
    /// `min(tick, N-1)` — the staircase plateaus at the top (TD Clamp).
    Clamp,
    /// A triangle of period `2(N-1)` — up then down (TD Zigzag).
    Zigzag,
}

impl LimitMode {
    fn from_param(v: f32) -> Self {
        match v.round() as i32 {
            1 => LimitMode::Clamp,
            2 => LimitMode::Zigzag,
            _ => LimitMode::Wrap,
        }
    }
}

struct Params {
    channel: i32,
    step: f32,
    count_max: i64,
    mode: LimitMode,
    reset_to: f32,
}

/// The monotonic tick after this frame: +1 only on the pulse's rising edge, and
/// `reset_to` whenever the reset is HIGH.
///
/// The reset is read as a **level** while the count reads an **edge** — see the
/// header: counting accumulates (so a held pulse must count once) and resetting
/// is idempotent (so a held reset simply holds). It is applied **last**, so a
/// reset arriving with a pulse lands on `reset_to` rather than one past it.
fn advance_tick(pulse: f32, prev_pulse: f32, prev_tick: f32, reset: f32, reset_to: f32) -> f32 {
    if reset > 0.5 {
        return reset_to;
    }
    let rising = pulse > 0.5 && prev_pulse <= 0.5;
    if rising { prev_tick + 1.0 } else { prev_tick }
}

/// The displayed count for a monotonic `tick`, folded by the limit mode. Integer
/// arithmetic only (Euclidean modulo) — exact and transcendental-free (HR-5).
fn displayed(tick: i64, n: i64, mode: LimitMode) -> i64 {
    let n = n.max(1);
    match mode {
        LimitMode::Wrap => tick.rem_euclid(n),
        LimitMode::Clamp => tick.min(n - 1),
        LimitMode::Zigzag => {
            if n == 1 {
                return 0;
            }
            let period = 2 * (n - 1);
            let m = tick.rem_euclid(period);
            if m < n { m } else { period - m }
        }
    }
}

fn step(input: &Stream, pulse: &Stream, state: &Stream, reset: &Stream, p: &Params) -> Stream {
    let n = input.count();
    // As portas do MESMO tique: a escada `0/1/n` (um batimento global alcança
    // todas as peças, não só a linha 0 — ver `pairing::beat_at`).
    let pulses = raw_col(pulse, PULSE_COL);
    // An unwired `reset` cooks to an empty stream, so this reads all-zero and the
    // node is byte-identical to the one that had no fourth port.
    let resets = raw_col(reset, PULSE_COL);
    // O estado é do tique ANTERIOR: casa por `id` quando a stream tem identidade
    // (`pairing`), e semeia quem não tinha linha lá.
    let rows = pairing(input, state, n);
    let prev_tick = raw_col(state, TICK_COL);
    let prev_pulse = raw_col(state, PREV_COL);

    let mut tick = Vec::with_capacity(n);
    let mut count = Vec::with_capacity(n);
    let mut deltas = Vec::with_capacity(n);
    // A coluna que sai como `count_prev`: o pulso DESTE elemento, já difundido —
    // senão o `prev` de amanhã contradiz o `pulse` de hoje e a beira desaparece.
    let mut pulse_out = Vec::with_capacity(n);
    for i in 0..n {
        let (pt, pp) = match rows.as_ref().and_then(|r| r[i]) {
            Some(j) => (
                prev_tick.get(j).copied().unwrap_or(0.0),
                prev_pulse.get(j).copied().unwrap_or(0.0),
            ),
            // Peça nova (ou o conjunto foi reconstruído): a escada começa em 0.
            None => (0.0, 0.0),
        };
        let this_pulse = beat_at(&pulses, i);
        pulse_out.push(this_pulse);
        let t = advance_tick(this_pulse, pp, pt, beat_at(&resets, i), p.reset_to);
        let disp = displayed(t as i64, p.count_max, p.mode) as f32;
        tick.push(t);
        count.push(disp);
        // The displacement rides the falloff mask like every behaviour; applied to
        // the FRESH `in` (never the state) so the step never compounds.
        deltas.push(disp * p.step * falloff_at(input, i));
    }

    let mut out = apply_channel_delta(input, p.channel, &deltas);
    out.set(COUNT_COL, Column::Scalar(count));
    out.set(TICK_COL, Column::Scalar(tick));
    // This tick's pulse becomes next tick's `prev` (the edge memory).
    out.set(PREV_COL, Column::Scalar(pulse_out));
    out
}

struct MotionStep;

impl NodeOp for MotionStep {
    fn manifest(&self) -> &'static NodeManifest {
        &MANIFEST
    }

    fn eval(&self, ctx: &mut EvalCtx<'_>) {
        let p = Params {
            channel: ctx.param("channel").round() as i32,
            step: ctx.param("step"),
            count_max: (ctx.param("count_max").round() as i64).max(1),
            mode: LimitMode::from_param(ctx.param("mode")),
            // A count is a whole number of steps; the tick it seeds is an integer.
            reset_to: ctx.param("reset_to").round(),
        };
        let out = step(ctx.input(0), ctx.input(1), ctx.input(2), ctx.input(3), &p);
        ctx.emit(out);
    }
}

/// Register this node with the runtime registry. Called (via codegen) from
/// `ph2d-node-registry-init::register_all_nodes`.
pub fn register(reg: &mut NodeRegistry) -> Result<(), RegistryError> {
    reg.register(Box::new(MotionStep))?;
    reg.register_ui(
        MANIFEST.id,
        ph2d_node_registry::NodeUiManifest {
            display_name: "Step",
            // Transform blue: a visible behaviour — it pushes a transform channel
            // per beat (the very reason it is `motion.*`, not `pulse.*`).
            category: ph2d_node_registry::NodeUiCategory::Transform,
            silhouette: ph2d_node_registry::NodeSilhouette::Rect,
        },
    );
    reg.register_param_ui(MANIFEST.id, PARAM_HINTS);
    reg.register_param_channel_range(MANIFEST.id, PARAM_CHANNEL_RANGE);
    reg.register_param_units(MANIFEST.id, PARAM_UNITS);
    // CPU-only: this node reads `falloff` only at eval runtime (no GPU kernel), so the
    // diagnoser cannot derive the role from a `ColumnBinding` — declare it (ADR-0155).
    reg.register_couplings(
        MANIFEST.id,
        &[ph2d_node_registry::Coupling::Consumes("falloff")],
    );
    Ok(())
}

use ph2d_node_registry::{ParamUiHint, ParamUnit, ParamUnitDecl, ParamWidget};

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
        param: "step",
        label: "Step",
        min: -4.0,
        max: 4.0,
        step: 0.05,
        widget: ParamWidget::Slider,
    },
    ParamUiHint {
        param: "count_max",
        label: "Count",
        min: 1.0,
        max: 32.0,
        step: 1.0,
        widget: ParamWidget::Slider,
    },
    ParamUiHint {
        param: "mode",
        label: "Mode",
        min: 0.0,
        max: 2.0,
        step: 1.0,
        widget: ParamWidget::Enum {
            labels: &["Wrap", "Clamp", "Zigzag"],
        },
    },
    ParamUiHint {
        param: "reset_to",
        label: "Reset To",
        // The slider stops at 0 because a staircase restarting BELOW its own
        // floor is a fourth limit-mode nobody asked for; the modes above describe
        // the range `0..count_max` and nothing else.
        min: 0.0,
        max: 32.0,
        step: 1.0,
        widget: ParamWidget::Slider,
    },
];

/// Both numbers are whole counts of steps — the unit the row shows is what the
/// number IS (doc 88). `channel` and `mode` are enums and `step` is a
/// displacement whose unit is the CHANNEL's, which is what
/// [`ParamUnit::FromChannel`] exists to say.
static PARAM_UNITS: &[ParamUnitDecl] = &[
    ParamUnitDecl {
        param: "count_max",
        unit: ParamUnit::Count,
    },
    ParamUnitDecl {
        param: "reset_to",
        unit: ParamUnit::Count,
    },
    ParamUnitDecl {
        param: "step",
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
static PARAM_CHANNEL_RANGE: &[ParamChannelRange] = &[ParamChannelRange {
    param: "step",
    min: -TURN,
    max: TURN,
    step: 1.0,
}];

#[cfg(test)]
#[path = "tests.rs"]
mod tests;
