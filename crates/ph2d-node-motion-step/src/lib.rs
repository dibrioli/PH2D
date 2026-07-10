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
//! Positional per-instance (v1), matching the family: `in`/`pulse`/`state` pair
//! by row order. Under a uniform clock every row shares one count (a global beat);
//! per-instance generalises to per-dot pulses for free.

#![forbid(unsafe_code)]

use ph2d_node_registry::{NodeRegistry, RegistryError};
use ph2d_nodegraph::attr::{Column, Stream};
use ph2d_nodegraph::cook::EvalCtx;
use ph2d_nodegraph::effect::Effect;
use ph2d_nodegraph::node::{LoweringKind, NodeManifest, NodeOp, NodeTypeId, ParamSpec, PortSpec};
use ph2d_nodegraph::port::{Clock, Dim, Domain, PortType};

mod channel;
use channel::{apply_channel_delta, falloff_at};

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
}

/// The monotonic tick after this frame: +1 only on the pulse's rising edge.
fn advance_tick(pulse: f32, prev_pulse: f32, prev_tick: f32) -> f32 {
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

fn scalar_col(s: &Stream, name: &str, n: usize, id: f32) -> Vec<f32> {
    let mut v = match s.get(name) {
        Some(Column::Scalar(v)) => v.clone(),
        _ => Vec::new(),
    };
    v.resize(n, id);
    v
}

fn step(input: &Stream, pulse: &Stream, state: &Stream, p: &Params) -> Stream {
    let n = input.count();
    let pulses = scalar_col(pulse, PULSE_COL, n, 0.0);
    let prev_tick = scalar_col(state, TICK_COL, n, 0.0);
    let prev_pulse = scalar_col(state, PREV_COL, n, 0.0);

    let mut tick = Vec::with_capacity(n);
    let mut count = Vec::with_capacity(n);
    let mut deltas = Vec::with_capacity(n);
    for i in 0..n {
        let t = advance_tick(pulses[i], prev_pulse[i], prev_tick[i]);
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
    out.set(PREV_COL, Column::Scalar(pulses));
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
        };
        let out = step(ctx.input(0), ctx.input(1), ctx.input(2), &p);
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
];

#[cfg(test)]
mod tests {
    use super::*;

    fn dot(x: f32) -> Stream {
        Stream::new(1).with("P", Column::Vec2(vec![[x, 0.0]]))
    }
    fn fire(v: f32) -> Stream {
        Stream::new(1).with(PULSE_COL, Column::Scalar(vec![v]))
    }
    fn params(count_max: i64, mode: LimitMode) -> Params {
        Params {
            channel: 0, // X
            step: 1.0,
            count_max,
            mode,
        }
    }
    fn count(s: &Stream) -> f32 {
        match s.get(COUNT_COL).unwrap() {
            Column::Scalar(v) => v[0],
            _ => panic!(),
        }
    }
    fn x(s: &Stream) -> f32 {
        match s.get("P").unwrap() {
            Column::Vec2(v) => v[0][0],
            _ => panic!(),
        }
    }

    /// FALSIFICATION of edge-safety: a pulse HELD high for several ticks advances
    /// the count exactly ONCE (on the rising edge), not once per tick. Counting
    /// `pulse > 0.5` every tick — the bug — would reach 5 after a 5-tick hold.
    #[test]
    fn a_held_pulse_counts_once_not_once_per_tick() {
        let p = params(16, LimitMode::Wrap);
        let mut state = Stream::new(1);
        // The pulse rises at tick 0 and STAYS high for five ticks.
        for _ in 0..5 {
            state = step(&dot(0.0), &fire(1.0), &state, &p);
        }
        assert_eq!(count(&state), 1.0, "one rising edge = one count, not five");
        // It drops, then rises again → the second edge counts.
        state = step(&dot(0.0), &fire(0.0), &state, &p);
        state = step(&dot(0.0), &fire(1.0), &state, &p);
        assert_eq!(count(&state), 2.0, "the next rising edge counts once more");
    }

    /// Wrap mode: the count climbs 0..N-1 and returns HOME (0) on the Nth pulse —
    /// the displacement is `count · step`, so the grid slides out and snaps back.
    #[test]
    fn wrap_mode_returns_home_after_count_max() {
        let p = params(4, LimitMode::Wrap); // counts 0,1,2,3 then wraps
        let mut state = Stream::new(1);
        let mut seq = Vec::new();
        // Ten single-tick pulses (a 0 between each so every 1.0 is a fresh edge).
        for _ in 0..10 {
            state = step(&dot(0.0), &fire(1.0), &state, &p);
            seq.push(count(&state));
            state = step(&dot(0.0), &fire(0.0), &state, &p);
        }
        // 1,2,3,0,1,2,3,0,1,2 — home at every 4th pulse.
        assert_eq!(seq, vec![1.0, 2.0, 3.0, 0.0, 1.0, 2.0, 3.0, 0.0, 1.0, 2.0]);
    }

    /// Clamp mode: the count plateaus at N-1 and never wraps — the staircase holds
    /// at the top instead of returning home.
    #[test]
    fn clamp_mode_plateaus_at_the_top() {
        let p = params(3, LimitMode::Clamp); // 0,1,2 then holds at 2
        let mut state = Stream::new(1);
        let mut seq = Vec::new();
        for _ in 0..6 {
            state = step(&dot(0.0), &fire(1.0), &state, &p);
            seq.push(count(&state));
            state = step(&dot(0.0), &fire(0.0), &state, &p);
        }
        assert_eq!(seq, vec![1.0, 2.0, 2.0, 2.0, 2.0, 2.0], "holds at N-1");
    }

    /// Zigzag mode: a triangle — up to N-1 then back down to 0 and up again
    /// (period `2(N-1)`). This is the smooth ping-pong sweep the demo uses.
    #[test]
    fn zigzag_mode_pingpongs_up_then_down() {
        let p = params(4, LimitMode::Zigzag); // triangle 0..3..0, period 6
        let mut state = Stream::new(1);
        let mut seq = Vec::new();
        for _ in 0..8 {
            state = step(&dot(0.0), &fire(1.0), &state, &p);
            seq.push(count(&state));
            state = step(&dot(0.0), &fire(0.0), &state, &p);
        }
        // 1,2,3,2,1,0,1,2 — climbs to 3, folds back to 0, climbs again.
        assert_eq!(seq, vec![1.0, 2.0, 3.0, 2.0, 1.0, 0.0, 1.0, 2.0]);
    }

    /// FALSIFICATION of "apply to fresh input, never compound": after three
    /// counts the X displacement is `count · step` off the FRESH base (3·1 = 3),
    /// NOT the sum of every step applied to an ever-growing state (1+2+3 = 6).
    #[test]
    fn the_displacement_never_compounds_across_ticks() {
        let p = params(16, LimitMode::Wrap);
        let mut state = Stream::new(1);
        for _ in 0..3 {
            // Every tick feeds the SAME fresh base X = 10.0.
            state = step(&dot(10.0), &fire(1.0), &state, &p);
            state = step(&dot(10.0), &fire(0.0), &state, &p);
        }
        // count 3, step 1 → X = 10 + 3 = 13, not 10 + (1+2+3) = 16.
        assert_eq!(count(&state), 3.0);
        assert_eq!(
            x(&state),
            13.0,
            "displacement is count·step off the fresh base"
        );
    }

    /// A degenerate `count_max = 1` never divides by zero and stays home (count 0,
    /// zero displacement) no matter how many pulses arrive.
    #[test]
    fn count_max_one_stays_home_without_dividing_by_zero() {
        let p = params(1, LimitMode::Zigzag); // period 2(N-1) would be 0 — guarded
        let mut state = Stream::new(1);
        for _ in 0..5 {
            state = step(&dot(0.0), &fire(1.0), &state, &p);
            state = step(&dot(0.0), &fire(0.0), &state, &p);
        }
        assert_eq!(count(&state), 0.0);
        assert_eq!(x(&state), 0.0, "no displacement, no panic");
    }
}
