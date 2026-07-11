//! `pulse.counter` — the PURE reducer: a pulse train → a persistent integer
//! **value** (Motion Nodes M2, the value domain — doc 12). The name doc 09 §4.2
//! deliberately left free: `motion.step` is the visible *behaviour* (it pushes a
//! transform channel per beat); this is the abstract *reducer* the mature tools
//! ship (TouchDesigner **Count CHOP**, Max **counter**) — it emits a **value**
//! on its own socket and **never touches a channel**. Route its value into a
//! channel with `motion.drive`, so `beat → pulse.counter → motion.drive`
//! composes the same thing `motion.step` bundles — plus the value can fan out to
//! *several* drives (one count, many channels), which the bundled node cannot.
//!
//! **The value type** is the continuous per-instance field
//! `(Instances, Scalar, Frame)` carried on the `v` column — the same type the
//! `debug.const`/`debug.wave` chain uses, and the continuous dual of the
//! `(…, Event)` pulse. A value is ALWAYS a per-instance field; a "global" value
//! is just a length-1 field (the reference convergence — TD/Houdini/vvvv/Faust:
//! a constant is the degenerate field, so there is no separate global-scalar
//! type to double the node set). See doc 12.
//!
//! **The count math is `motion.step`'s, verbatim** (it was always the reducer at
//! heart): a monotonic per-instance `count_tick` rides the `pre` self-loop
//! (+1 only on the pulse's rising edge — edge-safe, TD `Off to On` vs `While
//! On`), and the *displayed* count is derived each tick from the tick + limit
//! mode: **Wrap** `tick mod N`, **Clamp** `min(tick, N-1)`, **Zigzag** a triangle
//! of period `2(N-1)`. Euclidean integer modulo (HR-5). What changed vs
//! `motion.step`: no `channel`/`step` params, no channel write — the displayed
//! count rides out as the `v` value column (plus `count_tick`/`count_prev` on the
//! `pre` loop). Positional per-instance (v1), matching the pulse family.

#![forbid(unsafe_code)]

use ph2d_node_registry::{NodeRegistry, RegistryError};
use ph2d_nodegraph::attr::{Column, Stream};
use ph2d_nodegraph::cook::EvalCtx;
use ph2d_nodegraph::effect::Effect;
use ph2d_nodegraph::node::{LoweringKind, NodeManifest, NodeOp, NodeTypeId, ParamSpec, PortSpec};
use ph2d_nodegraph::port::{Clock, Dim, Domain, PortType};

/// The pulse type — a discrete per-instance event `(Instances, Scalar, Event)`
/// (mirror of `ph2d_node_pulse_beat::PULSE`; kept local so this crate is a leaf
/// drop-crate — the shared vocabulary is the port, not a shared symbol).
const PULSE: PortType = PortType::new(Domain::Instances, Dim::Scalar, Clock::Event);
/// The value type — a continuous per-instance scalar field `(Instances, Scalar,
/// Frame)`, carried on the `v` column (the `debug.*` convention). The continuous
/// dual of `PULSE`; its `Frame` clock keeps it from connecting to a pulse port.
pub const VALUE: PortType = PortType::new(Domain::Instances, Dim::Scalar, Clock::Frame);

/// The pulse stream's fire column (`1.0` on a fired tick).
const PULSE_COL: &str = "pulse";
/// The value output column (the canonical `value`-domain column).
const VALUE_COL: &str = "v";
/// The monotonic tick carried on the `pre` self-loop (+1 per rising edge).
const TICK_COL: &str = "count_tick";
/// Last tick's pulse value, carried on the `pre` self-loop for edge detection.
const PREV_COL: &str = "count_prev";

/// The static contract of this node type (ADR-0031).
pub const MANIFEST: NodeManifest = NodeManifest {
    id: NodeTypeId::of("pulse.counter"),
    name: "pulse.counter",
    inputs: &[
        PortSpec {
            name: "pulse",
            ty: PULSE,
        },
        // Feedback: last tick's value output carries `count_tick`+`count_prev`.
        // Named `state` so the editor plumbs its `pre` self-loop on drop.
        PortSpec {
            name: "state",
            ty: VALUE,
        },
    ],
    outputs: &[PortSpec {
        name: "out",
        ty: VALUE,
    }],
    // Pure: the tick enters the fingerprint through the consumed `pre` edge.
    effect: Effect::Pure,
    clock: Clock::Frame,
    params: &[
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

fn scalar_col(s: &Stream, name: &str, n: usize) -> Vec<f32> {
    let mut v = match s.get(name) {
        Some(Column::Scalar(v)) => v.clone(),
        _ => Vec::new(),
    };
    v.resize(n, 0.0);
    v
}

fn step(pulse: &Stream, state: &Stream, count_max: i64, mode: LimitMode) -> Stream {
    let n = pulse.count();
    let pulses = scalar_col(pulse, PULSE_COL, n);
    let prev_tick = scalar_col(state, TICK_COL, n);
    let prev_pulse = scalar_col(state, PREV_COL, n);

    let mut tick = Vec::with_capacity(n);
    let mut value = Vec::with_capacity(n);
    for i in 0..n {
        let t = advance_tick(pulses[i], prev_pulse[i], prev_tick[i]);
        tick.push(t);
        value.push(displayed(t as i64, count_max, mode) as f32);
    }
    Stream::new(n)
        .with(VALUE_COL, Column::Scalar(value))
        .with(TICK_COL, Column::Scalar(tick))
        // This tick's pulse becomes next tick's `prev` (the edge memory).
        .with(PREV_COL, Column::Scalar(pulses))
}

struct PulseCounter;

impl NodeOp for PulseCounter {
    fn manifest(&self) -> &'static NodeManifest {
        &MANIFEST
    }

    fn eval(&self, ctx: &mut EvalCtx<'_>) {
        let count_max = (ctx.param("count_max").round() as i64).max(1);
        let mode = LimitMode::from_param(ctx.param("mode"));
        let out = step(ctx.input(0), ctx.input(1), count_max, mode);
        ctx.emit(out);
    }
}

/// Register this node with the runtime registry. Called (via codegen) from
/// `ph2d-node-registry-init::register_all_nodes`.
pub fn register(reg: &mut NodeRegistry) -> Result<(), RegistryError> {
    reg.register(Box::new(PulseCounter))?;
    reg.register_ui(
        MANIFEST.id,
        ph2d_node_registry::NodeUiManifest {
            display_name: "Counter",
            // Utility grey: a pulse→value reducer, plumbing (not a visible transform).
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
    use ph2d_nodegraph::cook::OpResolver;

    fn fire(v: f32) -> Stream {
        Stream::new(1).with(PULSE_COL, Column::Scalar(vec![v]))
    }
    fn value(s: &Stream) -> f32 {
        match s.get(VALUE_COL).unwrap() {
            Column::Scalar(v) => v[0],
            _ => panic!(),
        }
    }

    /// FALSIFICATION of edge-safety: a pulse HELD high advances the count ONCE
    /// (on the rising edge), not once per tick. Counting `pulse > 0.5` every tick
    /// would reach 5 after a 5-tick hold.
    #[test]
    fn a_held_pulse_counts_once_not_once_per_tick() {
        let mut state = Stream::new(1);
        for _ in 0..5 {
            state = step(&fire(1.0), &state, 16, LimitMode::Wrap);
        }
        assert_eq!(value(&state), 1.0, "one rising edge = one count, not five");
        state = step(&fire(0.0), &state, 16, LimitMode::Wrap);
        state = step(&fire(1.0), &state, 16, LimitMode::Wrap);
        assert_eq!(value(&state), 2.0, "the next rising edge counts once more");
    }

    /// The reducer emits a VALUE and NEVER a transform channel — the whole point
    /// of the pure split. The output stream carries `v` (+ the state columns) and
    /// no `P`/`rot`/`size`.
    #[test]
    fn it_emits_a_value_column_and_no_transform_channel() {
        let s = step(&fire(1.0), &Stream::new(1), 6, LimitMode::Wrap);
        assert!(s.get(VALUE_COL).is_some(), "emits the value column");
        assert!(s.get("P").is_none(), "no position");
        assert!(
            s.get("rot").is_none() && s.get("size").is_none(),
            "no channel"
        );
    }

    /// Wrap / Clamp / Zigzag fold the same monotonic tick three ways (TD Count
    /// CHOP limit modes), as a VALUE sequence.
    #[test]
    fn the_three_limit_modes_fold_the_count() {
        let run = |count_max, mode| {
            let mut state = Stream::new(1);
            let mut seq = Vec::new();
            for _ in 0..8 {
                state = step(&fire(1.0), &state, count_max, mode);
                seq.push(value(&state));
                state = step(&fire(0.0), &state, count_max, mode);
            }
            seq
        };
        assert_eq!(
            run(4, LimitMode::Wrap),
            vec![1.0, 2.0, 3.0, 0.0, 1.0, 2.0, 3.0, 0.0],
            "wrap returns home"
        );
        assert_eq!(
            run(3, LimitMode::Clamp),
            vec![1.0, 2.0, 2.0, 2.0, 2.0, 2.0, 2.0, 2.0],
            "clamp plateaus at N-1"
        );
        assert_eq!(
            run(4, LimitMode::Zigzag),
            vec![1.0, 2.0, 3.0, 2.0, 1.0, 0.0, 1.0, 2.0],
            "zigzag ping-pongs"
        );
    }

    /// `count_max = 1` never divides by zero and stays home (value 0).
    #[test]
    fn count_max_one_stays_home_without_dividing_by_zero() {
        let mut state = Stream::new(1);
        for _ in 0..5 {
            state = step(&fire(1.0), &state, 1, LimitMode::Zigzag);
            state = step(&fire(0.0), &state, 1, LimitMode::Zigzag);
        }
        assert_eq!(value(&state), 0.0);
    }

    #[test]
    fn registers_and_resolves() {
        let mut reg = NodeRegistry::new();
        register(&mut reg).unwrap();
        assert!(reg.resolve(MANIFEST.id).is_some());
    }
}
