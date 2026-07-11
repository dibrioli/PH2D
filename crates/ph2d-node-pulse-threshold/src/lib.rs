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

fn step(
    input: &Stream,
    state: &Stream,
    rise: f32,
    fall: f32,
    edge: EdgeDir,
    channel: i32,
) -> Stream {
    let n = input.count();
    let prev_armed = match state.get(ARMED_COL) {
        Some(Column::Scalar(v)) => v.clone(),
        _ => Vec::new(),
    };
    let mut pulses = Vec::with_capacity(n);
    let mut armed = Vec::with_capacity(n);
    for i in 0..n {
        let v = channel_get(input, channel, i);
        let was = prev_armed.get(i).copied().unwrap_or(0.0) > 0.5;
        let (p, a) = step_one(v, was, rise, fall, edge);
        pulses.push(p);
        armed.push(a);
    }
    Stream::new(n)
        .with(PULSE_COL, Column::Scalar(pulses))
        .with(ARMED_COL, Column::Scalar(armed))
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
        let out = step(ctx.input(0), ctx.input(1), rise, fall, edge, channel);
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
    Ok(())
}

use ph2d_node_registry::{ParamUiHint, ParamWidget};

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
        max: 10.0,
        step: 0.05,
        widget: ParamWidget::Slider,
    },
    ParamUiHint {
        param: "fall",
        label: "Fall",
        min: -10.0,
        max: 10.0,
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
];

#[cfg(test)]
mod tests {
    use super::*;

    /// Feed a ramp up then down through the Schmitt trigger, carrying the output
    /// back as `state` (what the `pre` self-loop does). It must fire ONCE on the
    /// way up (crossing `rise`) and, with hysteresis, only re-arm after the
    /// signal drops below the separate, lower `fall`.
    #[test]
    fn it_fires_once_on_the_rising_edge_and_holds_through_the_hysteresis_band() {
        let (rise, fall) = (0.6, 0.3);
        // A signal that climbs past `rise`, dips into the band (0.3..0.6) WITHOUT
        // falling below `fall`, climbs again, then finally drops below `fall`.
        let signal = [0.0, 0.5, 0.7, 0.9, 0.4, 0.8, 0.2, 0.7];
        let mut state = Stream::new(1);
        let mut fired = Vec::new();
        for &v in &signal {
            let input = Stream::new(1).with("P", Column::Vec2(vec![[0.0, v]]));
            state = step(&input, &state, rise, fall, EdgeDir::Rise, 1);
            let p = match state.get(PULSE_COL).unwrap() {
                Column::Scalar(s) => s[0],
                _ => panic!(),
            };
            fired.push(p);
        }
        // Rises at index 2 (0.5→0.7 crosses 0.6). Index 4 (0.4) stays in the band
        // → still armed → NO refire at index 5. Index 6 (0.2 < fall) disarms →
        // index 7 (0.7) re-fires. Exactly two pulses; the band swallowed the dip.
        assert_eq!(fired, vec![0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0]);
    }

    /// FALSIFICATION of the hysteresis: with a SINGLE threshold (fall == rise),
    /// the same in-band dip re-arms and the signal chatters — the extra pulse
    /// the two-threshold design exists to suppress.
    #[test]
    fn a_single_threshold_chatters_where_the_schmitt_stays_quiet() {
        let signal = [0.0, 0.7, 0.4, 0.8]; // dip to 0.4 is below a single 0.6...
        let run = |rise: f32, fall: f32| {
            let mut state = Stream::new(1);
            let mut count = 0;
            for &v in &signal {
                let input = Stream::new(1).with("P", Column::Vec2(vec![[0.0, v]]));
                state = step(&input, &state, rise, fall, EdgeDir::Rise, 1);
                if let Some(Column::Scalar(s)) = state.get(PULSE_COL) {
                    count += (s[0] > 0.5) as i32;
                }
            }
            count
        };
        // Single threshold (fall==rise): 0.4 disarms, 0.8 re-fires → 2 pulses.
        assert_eq!(run(0.6, 0.6), 2, "single threshold chatters");
        // Schmitt (fall 0.3 < rise 0.6): 0.4 stays armed → 1 pulse.
        assert_eq!(run(0.6, 0.3), 1, "hysteresis suppresses the chatter");
    }

    #[test]
    fn the_fall_direction_fires_on_the_disarm_crossing() {
        let signal = [0.0, 0.8, 0.1]; // arm at 0.8, disarm at 0.1.
        let mut state = Stream::new(1);
        let mut fired = Vec::new();
        for &v in &signal {
            let input = Stream::new(1).with("P", Column::Vec2(vec![[0.0, v]]));
            state = step(&input, &state, 0.5, 0.3, EdgeDir::Fall, 1);
            if let Some(Column::Scalar(s)) = state.get(PULSE_COL) {
                fired.push(s[0]);
            }
        }
        // Rise edge is ignored; the pulse is on the fall (index 2).
        assert_eq!(fired, vec![0.0, 0.0, 1.0]);
    }

    /// The `pulse` output is 1.0 only for ONE tick, even while the signal stays
    /// high — the Rive "true for a short time". A sustained level is NOT a train
    /// of pulses.
    #[test]
    fn a_sustained_high_signal_fires_exactly_one_pulse() {
        let mut state = Stream::new(1);
        let mut total = 0.0;
        for _ in 0..10 {
            let input = Stream::new(1).with("P", Column::Vec2(vec![[0.0, 1.0]]));
            state = step(&input, &state, 0.5, 0.3, EdgeDir::Rise, 1);
            if let Some(Column::Scalar(s)) = state.get(PULSE_COL) {
                total += s[0];
            }
        }
        assert_eq!(total, 1.0, "one edge, not one-per-tick-held-high");
    }

    #[test]
    fn an_inverted_band_is_clamped_not_honoured() {
        // fall > rise would be a nonsense inverted band; clamp fall down to rise
        // (degenerates to a single threshold, never a negative-width band).
        let (p, a) = step_one(0.7, false, 0.5, 0.9, EdgeDir::Rise);
        assert_eq!(
            (p, a),
            (1.0, 1.0),
            "still arms at rise; the bad fall is ignored"
        );
    }

    /// `Both` fires on the arm AND the disarm crossing — one pulse each way
    /// (audit 2026-07-10: the third `EdgeDir` was untested). Rise-only and
    /// Fall-only each see exactly one of the two.
    #[test]
    fn the_both_direction_fires_on_arm_and_disarm() {
        let signal = [0.0, 0.8, 0.1]; // arm at 0.8, disarm at 0.1.
        let run = |edge: EdgeDir| {
            let mut state = Stream::new(1);
            let mut fired = Vec::new();
            for &v in &signal {
                let input = Stream::new(1).with("P", Column::Vec2(vec![[0.0, v]]));
                state = step(&input, &state, 0.5, 0.3, edge, 1);
                if let Some(Column::Scalar(s)) = state.get(PULSE_COL) {
                    fired.push(s[0]);
                }
            }
            fired
        };
        assert_eq!(run(EdgeDir::Both), vec![0.0, 1.0, 1.0], "both crossings");
        assert_eq!(run(EdgeDir::Rise), vec![0.0, 1.0, 0.0], "arm only");
        assert_eq!(run(EdgeDir::Fall), vec![0.0, 0.0, 1.0], "disarm only");
    }
}
