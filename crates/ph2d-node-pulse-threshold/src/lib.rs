//! `pulse.threshold` — a value channel → a discrete PULSE, with Schmitt
//! hysteresis (Motion Nodes M2, plan §1.1; decision doc `06_pulse_*`).
//!
//! This is the first PRODUCER of the pulse type: `PortType(Instances, Scalar,
//! Event)`. A pulse is Rive's first-class Trigger — *"similar to booleans, but
//! can only become true for a short time"* — not Cavalry's 0/1-by-convention.
//! The substrate already enforces it: an `Event`-clock port cannot connect to a
//! `Frame`-clock port by a plain edge, so `pulse ≠ value` is a compile error,
//! not a convention.
//!
//! **What it emits:** the `pulse` column is `1.0` only on the tick the chosen
//! edge fires, `0.0` otherwise (Rive "true for a short time" / Max rising bang).
//! No payload. The rising-edge detection lives HERE, in the producer, via the
//! `pre` self-loop — the consumer just reads "1.0 this tick? fire."
//!
//! **Schmitt hysteresis (the whole point):** a single threshold fires on every
//! wiggle across it — noise alone produces a burst of spurious pulses
//! (Wikipedia). Two thresholds — `rise` > `fall` — give a bistable memory: once
//! armed, the signal must fall below the separate `fall` level before it can
//! re-arm. That latched `armed` state is a per-instance recurrence over the
//! tick, so it rides the `pre` self-loop exactly like `spring`/`integrate`
//! (the `state` port convention). Names mirror TouchDesigner
//! (`threshup`/`threshdown`) and Pd `threshold~` (trigger/rest).
//!
//! **Direction** (`edge`): Rise (arm crossing, the default), Fall (disarm
//! crossing), or Both — the TD "Trigger On" selector, `edge~`'s two outlets.
//!
//! **Debounce (`debounce`, seconds):** after a pulse fires, the next `debounce`
//! seconds are silent. TouchDesigner's Trigger/Count CHOPs call it *Re-Trigger
//! Delay*; Pd's `threshold~` calls it *debounce ms per edge*. It was deferred
//! once with the argument *"the hysteresis already kills the chatter"*, and that
//! argument is **half true and the half matters**: hysteresis is an **AMPLITUDE**
//! guard — it swallows noise that wiggles across one level — while a debounce is a
//! **TIME** guard, and the thing it swallows is two *legitimate* crossings that
//! arrive too fast (a hand bouncing on a gesture). Neither substitutes for the
//! other: a signal that swings past BOTH levels quickly defeats the band, and a
//! slow drift inside the band defeats any clock.
//!
//! ⚠️ **The debounce silences the OUTPUT, never the state machine.** The Schmitt
//! keeps tracking the signal through the quiet window — only the pulse is
//! withheld. Freezing the latch instead would desync it from the signal: the arm
//! that happened during the window would be lost, the next disarm would look like
//! nothing, and the trigger would go quiet **for good**.
//!
//! The countdown rides the `pre` self-loop as one more sibling column, and its
//! neutral is `0.0` — which is exactly what an absent column reads — so a graph
//! that never fired and a graph whose window expired are the same state, and
//! `debounce = 0` is the world before this param, tick for tick.
//!
//! Positional per-instance (v1): the caller pairs `in`/`state` by row order.
//! The focus-rig demo has a stable count; id-keyed pairing (for particles) is
//! the same follow-up the other sequential nodes carry.

#![forbid(unsafe_code)]

use ph2d_node_registry::{NodeRegistry, RegistryError};
use ph2d_nodegraph::attr::{Column, Stream};
use ph2d_nodegraph::cook::EvalCtx;
use ph2d_nodegraph::effect::Effect;
use ph2d_nodegraph::gpu::{ColumnAccess, ColumnBinding, GpuKernel};
use ph2d_nodegraph::node::{LoweringKind, NodeManifest, NodeOp, NodeTypeId, ParamSpec, PortSpec};
use ph2d_nodegraph::port::{Clock, Dim, Domain, PortType};

mod channel;
use channel::channel_get;

/// The value stream this node reads: a motion instance stream, from which it
/// samples the selected channel. (There is no dedicated `value` port type yet;
/// like every behaviour, this reads a channel of the `INST_VEC2` stream.)
const INST_VEC2: PortType = PortType::new(Domain::Instances, Dim::Vec2, Clock::Frame);

/// The pulse type — the whole reason for this wave. `Event` clock: it will not
/// connect to a `Frame` port by a plain edge (the membrane), so downstream can
/// only be a pulse consumer.
pub const PULSE: PortType = PortType::new(Domain::Instances, Dim::Scalar, Clock::Event);

/// The canonical column of a pulse stream: `1.0` on the tick it fires.
pub const PULSE_COL: &str = "pulse";
/// The latched bistable state carried on the `pre` self-loop (`1.0` = armed).
/// A sibling column of the pulse stream, like `trail_age` on a trail.
const ARMED_COL: &str = "armed";
/// Seconds of silence still owed by the debounce, on the `pre` self-loop.
/// **Zero is the neutral** — never fired and window expired are the same state,
/// and an absent column already reads zero.
const COOL_COL: &str = "thr_cool";

/// The static contract of this node type (ADR-0031).
pub const MANIFEST: NodeManifest = NodeManifest {
    id: NodeTypeId::of("pulse.threshold"),
    name: "pulse.threshold",
    inputs: &[
        PortSpec {
            name: "in",
            ty: INST_VEC2,
        },
        // Feedback: last tick's pulse output carries the latched `armed` state.
        // Named `state` so the editor plumbs its `pre` self-loop on drop.
        PortSpec {
            name: "state",
            ty: PULSE,
        },
    ],
    outputs: &[PortSpec {
        name: "out",
        ty: PULSE,
    }],
    // Pure: the tick enters the fingerprint through the consumed `pre` edge.
    effect: Effect::Pure,
    clock: Clock::Frame,
    params: &[
        // 0 X · 1 Y · 2 Rotation · 3 Size — the shared channel vocabulary.
        ParamSpec {
            name: "channel",
            default: 1.0,
        },
        ParamSpec {
            name: "rise",
            default: 0.5,
        },
        // Below `rise` by default → a real hysteresis band. `fall > rise` is
        // clamped to `rise` at eval (a band cannot be inverted).
        ParamSpec {
            name: "fall",
            default: 0.3,
        },
        // 0 Rise · 1 Fall · 2 Both — which arm transition fires the pulse.
        ParamSpec {
            name: "edge",
            default: 0.0,
        },
        // Seconds of silence after a pulse. APPENDED, and `0` is the world
        // before it: the countdown never gates, tick for tick.
        ParamSpec {
            name: "debounce",
            default: 0.0,
        },
    ],
    lowerings: &[LoweringKind::Cpu],
};

/// ⭐ **O KERNEL** (ADR-0126) — a porta WGSL do [`step`], **inteiramente no dispositivo**.
///
/// ⚠️⚠️ **Ele nasceu a cobrir só o `debounce = 0`, e a recusa dissolveu no mesmo dia.** O
/// [`debounce_one`] conta um relógio para trás (`cool − dt`) e o módulo gerado **não tinha `dt`**;
/// a saída óbvia era um `applicable` a recuar para a CPU acima do neutro. Em vez disso o `dt`
/// passou a ser **uniform** (ciclo 6 W2) — *o bloqueador era o substrato e não a lei, que é
/// exactamente a espécie de limite que o §0.0 manda medir em vez de aceitar.*
///
/// ⚠️ **NaN no `channel` escolhe X, e ±inf escolhe Size** — não é capricho: `f32::NAN as i32` vale
/// `0` em Rust (a conversão float→int é definida e satura), e `±inf` satura nos extremos, que caem
/// no braço `_`. Os dois estão escritos porque um `round` da WGSL (meio-par) sobre um NaN cairia
/// no `_` e narrava o nó para o canal errado, em silêncio.
///
/// ⚠️ **As três colunas lidas são `Consume`**: o [`step`] da CPU emite `pulse` + `armed` +
/// `thr_cool` e larga a geometria. Sem isto um `P` viajava numa corrente de PULSO.
const GPU_KERNEL: GpuKernel = GpuKernel {
    wgsl: "\
        let th_c = thr_channel(params.channel);\n\
        let th_p = read_in_P(i);\n\
        var th_v = read_in_size(i).x;\n\
        if (th_c == 0) { th_v = th_p.x; }\n\
        else if (th_c == 1) { th_v = th_p.y; }\n\
        else if (th_c == 2) { th_v = read_in_rot(i); }\n\
        // A banda nunca inverte -- o `fall.min(rise)` do `step_one`.\n\
        let th_fall = min(params.fall, params.rise);\n\
        let th_was = read_state_armed(i) > 0.5;\n\
        let th_now = select(th_v >= params.rise, th_v > th_fall, th_was);\n\
        let th_rose = th_now && !th_was;\n\
        let th_fell = !th_now && th_was;\n\
        let th_e = thr_edge(params.edge);\n\
        let th_fire = (th_e == 0 && th_rose) || (th_e == 1 && th_fell)\n\
        \x20   || (th_e == 2 && (th_rose || th_fell));\n\
        write_armed(i, select(0.0, 1.0, th_now));\n\
        // O ABRANDADOR, o gemeo do `debounce_one`. `NaN.max(0.0)` vale `0.0` em Rust; o `max`\n\
        // da WGSL nao promete isso, dai o `select`.\n\
        let th_deb = select(0.0, params.debounce, params.debounce >= 0.0);\n\
        let th_cool0 = read_state_thr_cool(i) - params.dt;\n\
        var th_out = 0.0;\n\
        var th_cool = th_cool0;\n\
        if (th_cool0 <= 0.0) {\n\
        \x20   th_out = select(0.0, 1.0, th_fire);\n\
        \x20   th_cool = select(0.0, th_deb, th_out > 0.5);\n\
        }\n\
        write_pulse(i, th_out);\n\
        write_thr_cool(i, th_cool);\n",
    wgsl_lib: "\
        // Rust `f32::round` = meio para LONGE do zero; o `round` da WGSL e' meio-par.\n\
        fn thr_round(x: f32) -> f32 {\n\
        \x20   return select(ceil(x - 0.5), floor(x + 0.5), x >= 0.0);\n\
        }\n\
        // O gemeo de `channel_get`: 0 X . 1 Y . 2 Rotation . o resto Size.\n\
        // `!(x == x)` e' o teste de NaN (toda comparacao com NaN e' falsa) -- e ele vem PRIMEIRO\n\
        // porque `f32::NAN as i32` vale ZERO em Rust, enquanto +-inf satura nos extremos.\n\
        fn thr_channel(x: f32) -> i32 {\n\
        \x20   if (!(x == x)) { return 0; }\n\
        \x20   let r = thr_round(x);\n\
        \x20   if (r == 0.0) { return 0; }\n\
        \x20   if (r == 1.0) { return 1; }\n\
        \x20   if (r == 2.0) { return 2; }\n\
        \x20   return 3;\n\
        }\n\
        // O gemeo de `EdgeDir::from_param`: `1` Fall, `2` Both, o resto Rise.\n\
        fn thr_edge(x: f32) -> i32 {\n\
        \x20   let r = thr_round(x);\n\
        \x20   if (r == 1.0) { return 1; }\n\
        \x20   if (r == 2.0) { return 2; }\n\
        \x20   return 0;\n\
        }\n",
    bindings: &[
        ColumnBinding {
            column: "P",
            dim: Dim::Vec2,
            access: ColumnAccess::Consume,
            identity: [0.0; 4],
            port: 0,
        },
        ColumnBinding {
            column: "rot",
            dim: Dim::Scalar,
            access: ColumnAccess::Consume,
            identity: [0.0; 4],
            port: 0,
        },
        ColumnBinding {
            // ⚠️ A identidade do `size` é a escala UNITÁRIA, não zero — um limiar de tamanho
            // sobre um gerador nu lê `1`, e não um falso «abaixo de todo limiar».
            column: "size",
            dim: Dim::Vec2,
            access: ColumnAccess::Consume,
            identity: [1.0, 1.0, 0.0, 0.0],
            port: 0,
        },
        ColumnBinding {
            column: PULSE_COL,
            dim: Dim::Scalar,
            access: ColumnAccess::Write,
            identity: [0.0; 4],
            port: 0,
        },
        ColumnBinding {
            column: ARMED_COL,
            dim: Dim::Scalar,
            access: ColumnAccess::ReadWrite,
            identity: [0.0; 4],
            port: 1,
        },
        ColumnBinding {
            // ⛔⛔ **Esta binding foi `Write` durante meia wave, e a troca é um CRASH que só aparece
            // no SEGUNDO tique.** Enquanto o kernel só reivindicava `debounce = 0`, o corpo nunca
            // LIA este relógio — e um `var<storage, read>` que nada referencia é **apagado pela
            // naga do layout derivado**, enquanto o sequenciador continua a pôr o buffer no bind
            // group (`7` entradas contra `6`, e a placa recusa). No primeiro tique passava, porque
            // a coluna ainda não existe na corrente de estado e nenhum buffer é ligado.
            // *Um erro de declaração que só acorda quando a coluna nasce é o pior sítio para o
            // pôr* — daí o gate `todo_read_declarado_e_lido` do `generated_wgsl_validates`.
            column: COOL_COL,
            dim: Dim::Scalar,
            access: ColumnAccess::ReadWrite,
            identity: [0.0; 4],
            port: 1,
        },
    ],
    params: &["channel", "rise", "fall", "edge", "debounce"],
    count_law: None,
    variant_by_param: None,
    // ⚠️ **Sem `applicable`: o kernel cobre o espaço de params INTEIRO.** Ele nasceu com um
    // `!(d > 0.0)` — o `debounce` precisa do `dt`, e o módulo gerado não o tinha —, e a wave que
    // pôs o `dt` no uniform dissolveu essa recusa no mesmo dia. *Um `applicable` é uma dívida
    // datada, não uma propriedade do nó.*
    applicable: None,
};

/// Direction selector for [`fire`].
#[derive(Copy, Clone, PartialEq, Eq)]
enum EdgeDir {
    Rise,
    Fall,
    Both,
}

impl EdgeDir {
    fn from_param(v: f32) -> Self {
        match v.round() as i32 {
            1 => EdgeDir::Fall,
            2 => EdgeDir::Both,
            _ => EdgeDir::Rise,
        }
    }
    fn fires(self, rose: bool, fell: bool) -> bool {
        match self {
            EdgeDir::Rise => rose,
            EdgeDir::Fall => fell,
            EdgeDir::Both => rose || fell,
        }
    }
}

/// One tick of the Schmitt trigger, per instance. Returns the `(pulse, armed)`
/// pair for row `i`, given the channel value and last tick's armed state.
///
/// This is the **raw** edge: the debounce has no say here, and that is the whole
/// point — the latch it returns is the truth about the signal whether or not the
/// output is being withheld ([`debounce_one`]).
fn step_one(v: f32, prev_armed: bool, rise: f32, fall: f32, edge: EdgeDir) -> (f32, f32) {
    // Hysteresis: once armed, only `fall` disarms; once disarmed, only `rise`
    // arms. `fall` is clamped ≤ `rise` so the band can never invert.
    let fall = fall.min(rise);
    let armed_now = if prev_armed { v > fall } else { v >= rise };
    let rose = armed_now && !prev_armed;
    let fell = !armed_now && prev_armed;
    let pulse = if edge.fires(rose, fell) { 1.0 } else { 0.0 };
    (pulse, if armed_now { 1.0 } else { 0.0 })
}

/// The debounce, per instance: `(pulse, next_cooldown)` from the raw edge and
/// last tick's countdown. `debounce = 0` makes the countdown identically zero,
/// so it can never gate — the pre-debounce world, arithmetically.
fn debounce_one(raw: f32, prev_cool: f32, debounce: f32, dt: f32) -> (f32, f32) {
    let cool = prev_cool - dt;
    if cool > 0.0 {
        // Still inside the quiet window: withhold, keep counting down.
        return (0.0, cool);
    }
    if raw > 0.5 {
        (raw, debounce)
    } else {
        (raw, 0.0)
    }
}

fn scalar_col(s: &Stream, name: &str) -> Vec<f32> {
    match s.get(name) {
        Some(Column::Scalar(v)) => v.clone(),
        _ => Vec::new(),
    }
}

/// Everything the artist authored, in one place. It exists because the trigger
/// grew a fifth knob and a positional call stopped being readable — a bundle also
/// makes the fixtures below DECLARE their premise (`debounce: 0.0`) instead of
/// leaving it as the sixth number in a row.
#[derive(Copy, Clone)]
struct Trigger {
    rise: f32,
    fall: f32,
    edge: EdgeDir,
    channel: i32,
    /// Seconds of silence after a pulse. `0` = the world before the param.
    debounce: f32,
}

fn step(input: &Stream, state: &Stream, cfg: Trigger, dt: f32) -> Stream {
    let n = input.count();
    let prev_armed = scalar_col(state, ARMED_COL);
    let prev_cool = scalar_col(state, COOL_COL);
    let mut pulses = Vec::with_capacity(n);
    let mut armed = Vec::with_capacity(n);
    let mut cool = Vec::with_capacity(n);
    for i in 0..n {
        let v = channel_get(input, cfg.channel, i);
        let was = prev_armed.get(i).copied().unwrap_or(0.0) > 0.5;
        let (raw, a) = step_one(v, was, cfg.rise, cfg.fall, cfg.edge);
        let prev = prev_cool.get(i).copied().unwrap_or(0.0);
        let (p, c) = debounce_one(raw, prev, cfg.debounce, dt);
        pulses.push(p);
        // The latch is written from the RAW step, always: the debounce owns the
        // output, never the state machine.
        armed.push(a);
        cool.push(c);
    }
    Stream::new(n)
        .with(PULSE_COL, Column::Scalar(pulses))
        .with(ARMED_COL, Column::Scalar(armed))
        .with(COOL_COL, Column::Scalar(cool))
}

struct PulseThreshold;

impl NodeOp for PulseThreshold {
    fn manifest(&self) -> &'static NodeManifest {
        &MANIFEST
    }

    fn eval(&self, ctx: &mut EvalCtx<'_>) {
        let channel = ctx.param("channel").round() as i32;
        let rise = ctx.param("rise");
        let fall = ctx.param("fall");
        let edge = EdgeDir::from_param(ctx.param("edge"));
        let debounce = ctx.param("debounce").max(0.0);
        let dt = ctx.dt() as f32;
        let cfg = Trigger {
            rise,
            fall,
            edge,
            channel,
            debounce,
        };
        let out = step(ctx.input(0), ctx.input(1), cfg, dt);
        ctx.emit(out);
    }
}

/// Register this node with the runtime registry. Called (via codegen) from
/// `ph2d-node-registry-init::register_all_nodes`.
pub fn register(reg: &mut NodeRegistry) -> Result<(), RegistryError> {
    reg.register(Box::new(PulseThreshold))?;
    reg.register_gpu_kernel(MANIFEST.id, GPU_KERNEL);
    reg.register_ui(
        MANIFEST.id,
        ph2d_node_registry::NodeUiManifest {
            display_key: "node.pulse.threshold.name",
            // Utility grey: a value→pulse adapter, not a visible transform.
            category: ph2d_node_registry::NodeUiCategory::Utility,
            silhouette: ph2d_node_registry::NodeSilhouette::Rect,
        },
    );
    reg.register_param_ui(MANIFEST.id, PARAM_HINTS);
    reg.register_param_hard_max(MANIFEST.id, PARAM_HARD_MAX);
    reg.register_param_units(MANIFEST.id, PARAM_UNITS);
    Ok(())
}

use ph2d_node_registry::{ParamHardMax, ParamUiHint, ParamUnit, ParamUnitDecl, ParamWidget};

/// The debounce is the node's one param with a unit of its own. `rise`/`fall`
/// are deliberately left undeclared: they are a level on the SELECTED channel,
/// so their unit is `FromChannel`, and declaring it would move how they display
/// today — a separate decision, not a rider on this one.
static PARAM_UNITS: &[ParamUnitDecl] = &[ParamUnitDecl {
    param: "debounce",
    unit: ParamUnit::Seconds,
}];

/// O teto que a MÁQUINA (ou o bom senso) impõe, alcançável por DIGITAÇÃO — o slider fica
/// onde a MÃO trabalha (soft/hard do Blender; doc 88 §11). O curso de antes é este número:
/// nada ficou inalcançável, só deixou de ser o que o dedo percorre.
static PARAM_HARD_MAX: &[ParamHardMax] = &[
    ParamHardMax {
        param: "rise",
        max: 10.0,
    },
    ParamHardMax {
        param: "fall",
        max: 10.0,
    },
    // MEASURED, and the resource is **precisão de representação**: the countdown
    // lives in `f32` seconds and is decremented by `dt`, so above some magnitude
    // `cool - dt == cool` and the window would never expire — the node would go
    // quiet for good, silently. The probe
    // `the_debounce_ceiling_is_where_the_countdown_stops_expiring` walks the
    // exponents; measured, the cliff is between **2^19 and 2^20 at 60 fps** and
    // one octave lower per doubling of the frame rate, because the decrement
    // shrinks with `dt`. The number below is the last power of two that still
    // drains at **240 fps** — the fastest clock, hence the harshest test — and it
    // is 36 hours, orders of magnitude past "fire once in this scene", which is
    // the longest window anyone authors. ⚠️ It sits ON the cliff by measurement,
    // not near it: one octave further the countdown stands still, and the gate
    // asserts both halves.
    ParamHardMax {
        param: "debounce",
        max: 131072.0,
    },
];

static PARAM_HINTS: &[ParamUiHint] = &[
    ParamUiHint {
        param: "channel",
        label: "node.pulse.threshold.param.channel",
        min: 0.0,
        max: 3.0,
        step: 1.0,
        widget: ParamWidget::Enum {
            labels: &[
                "node.pulse.threshold.param.channel.0",
                "node.pulse.threshold.param.channel.1",
                "node.pulse.threshold.param.channel.2",
                "node.pulse.threshold.param.channel.3",
            ],
        },
    },
    ParamUiHint {
        param: "rise",
        label: "node.pulse.threshold.param.rise",
        min: -10.0,
        max: 5.0,
        step: 0.05,
        widget: ParamWidget::Slider,
    },
    ParamUiHint {
        param: "fall",
        label: "node.pulse.threshold.param.fall",
        min: -10.0,
        max: 3.0,
        step: 0.05,
        widget: ParamWidget::Slider,
    },
    ParamUiHint {
        param: "edge",
        label: "node.pulse.threshold.param.edge",
        min: 0.0,
        max: 2.0,
        step: 1.0,
        widget: ParamWidget::Enum {
            labels: &[
                "node.pulse.threshold.param.edge.0",
                "node.pulse.threshold.param.edge.1",
                "node.pulse.threshold.param.edge.2",
            ],
        },
    },
    ParamUiHint {
        param: "debounce",
        label: "node.pulse.threshold.param.debounce",
        min: 0.0,
        // The HAND's range: a bouncing gesture settles in tens of milliseconds
        // and "at most one pulse per second" is the far end of what anyone drags
        // to. A step of 10 ms is the resolution that band needs; longer windows
        // are typed (`PARAM_HARD_MAX`).
        max: 1.0,
        step: 0.01,
        widget: ParamWidget::Slider,
    },
];

/// Os gates moram num irmão por teto de LOC — FILHO, para `use super::*` seguir
/// alcançando o que eles medem (`step_one`, `debounce_one`, `EdgeDir`).
#[cfg(test)]
#[path = "tests.rs"]
mod tests;
