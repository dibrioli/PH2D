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
    reg.register_ui(
        MANIFEST.id,
        ph2d_node_registry::NodeUiManifest {
            display_name: "Threshold",
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
        label: "Channel",
        min: 0.0,
        max: 3.0,
        step: 1.0,
        widget: ParamWidget::Enum {
            labels: &["X", "Y", "Rotation", "Size"],
        },
    },
    ParamUiHint {
        param: "rise",
        label: "Rise",
        min: -10.0,
        max: 5.0,
        step: 0.05,
        widget: ParamWidget::Slider,
    },
    ParamUiHint {
        param: "fall",
        label: "Fall",
        min: -10.0,
        max: 3.0,
        step: 0.05,
        widget: ParamWidget::Slider,
    },
    ParamUiHint {
        param: "edge",
        label: "Edge",
        min: 0.0,
        max: 2.0,
        step: 1.0,
        widget: ParamWidget::Enum {
            labels: &["Rise", "Fall", "Both"],
        },
    },
    ParamUiHint {
        param: "debounce",
        label: "Debounce",
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
