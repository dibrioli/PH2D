//! `pulse.on_change` — fire a PULSE the tick a **value** field changes (Motion
//! Nodes M2, the value domain — doc 12/17). This is Max/Pd's **`change`** object,
//! TouchDesigner's Trigger-on-change: the value→pulse trigger that watches the
//! value's *derivative* rather than its *level*. It is the complement of
//! `pulse.compare` (which fires on a threshold CROSSING): `on_change` fires
//! whenever the value STEPS to something new — exactly the clock you want off a
//! `pulse.counter`, a `pulse.sample_hold`, or a `value.switch` flip.
//!
//! **Sequential — it holds the previous value.** Last tick's value rides the
//! `pre` self-loop on the `state` port (like `pulse.sample_hold`/`pulse.counter`),
//! and this tick fires iff `|v − prev| > epsilon`. The `epsilon` guard ignores
//! float dither so a steady value never chatters (Max `change` is exact-equality;
//! a small tolerance is the honest floating-point version). On the FIRST tick it
//! primes (records the value, does NOT fire) so a fresh graph doesn't emit a
//! spurious pulse — the same first-tick discipline as the rest of the family.
//!
//! **Unary over the field:** the pulse mirrors the value field's length `N` (each
//! instance watches its own value), so a length-N field fires a length-N pulse and
//! the downstream `motion.strobe`/`pulse.counter` reacts per element.
//! `Effect::Pure` — the tick enters the fingerprint through the consumed `pre`
//! edge. Transcendental-free (HR-5): subtract / abs / compare only.

#![forbid(unsafe_code)]

use ph2d_node_registry::{NodeRegistry, RegistryError};
use ph2d_nodegraph::attr::{Column, Stream};
use ph2d_nodegraph::cook::EvalCtx;
use ph2d_nodegraph::effect::Effect;
use ph2d_nodegraph::node::{LoweringKind, NodeManifest, NodeOp, NodeTypeId, ParamSpec, PortSpec};
use ph2d_nodegraph::port::{Clock, Dim, Domain, PortType};

/// The value type this node watches — the continuous per-instance scalar field on
/// the `v` column (mirror of `ph2d_node_pulse_counter::VALUE`; kept local so this
/// stays a leaf drop-crate — the shared vocabulary is the port, not a symbol).
pub const VALUE: PortType = PortType::new(Domain::Instances, Dim::Scalar, Clock::Frame);
/// The pulse type it emits — a discrete per-instance event. `Event` clock keeps it
/// off any `Frame` port (the membrane), so downstream can only be a pulse consumer.
pub const PULSE: PortType = PortType::new(Domain::Instances, Dim::Scalar, Clock::Event);

/// The value stream's column (the canonical `value`-domain column).
const VALUE_COL: &str = "v";
/// The canonical column of a pulse stream: `1.0` on the tick it fires.
const PULSE_COL: &str = "pulse";
/// Last tick's value, carried on the `pre` self-loop for the change comparison.
const PREV_COL: &str = "oc_prev";
/// `1.0` once the first value has been recorded (the priming flag), on the `pre`
/// self-loop — so the first tick never fires.
const PRIMED_COL: &str = "oc_primed";

/// The static contract of this node type (ADR-0031).
pub const MANIFEST: NodeManifest = NodeManifest {
    id: NodeTypeId::of("pulse.on_change"),
    name: "pulse.on_change",
    inputs: &[
        PortSpec {
            name: "value",
            ty: VALUE,
        },
        // Feedback: last tick's pulse output carries `oc_prev` + `oc_primed`.
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
        // The smallest change that counts as a change — ignores float dither so a
        // steady value never chatters. Exact-equality is `epsilon = 0`.
        ParamSpec {
            name: "epsilon",
            default: 0.001,
        },
    ],
    lowerings: &[LoweringKind::Cpu],
};

fn scalar_col(s: &Stream, name: &str, n: usize) -> Vec<f32> {
    let mut v = match s.get(name) {
        Some(Column::Scalar(v)) => v.clone(),
        _ => Vec::new(),
    };
    v.resize(n, 0.0);
    v
}

/// One tick of change detection. `N` mirrors the `value` field: each instance
/// fires iff its value moved more than `epsilon` since last tick; the first tick
/// primes (records, never fires).
fn step(value: &Stream, state: &Stream, epsilon: f32) -> Stream {
    let n = value.count();
    let vals = scalar_col(value, VALUE_COL, n);
    let prev = scalar_col(state, PREV_COL, n);
    let primed = scalar_col(state, PRIMED_COL, n);

    let mut pulses = Vec::with_capacity(n);
    for i in 0..n {
        // Prime on the first tick (nothing recorded yet) → never fire then.
        let fired = primed[i] > 0.5 && (vals[i] - prev[i]).abs() > epsilon;
        pulses.push(if fired { 1.0 } else { 0.0 });
    }
    Stream::new(n)
        .with(PULSE_COL, Column::Scalar(pulses))
        // This tick's value becomes next tick's `prev` (the change memory).
        .with(PREV_COL, Column::Scalar(vals))
        .with(PRIMED_COL, Column::Scalar(vec![1.0; n]))
}

struct PulseOnChange;

impl NodeOp for PulseOnChange {
    fn manifest(&self) -> &'static NodeManifest {
        &MANIFEST
    }

    fn eval(&self, ctx: &mut EvalCtx<'_>) {
        let epsilon = ctx.param("epsilon").max(0.0);
        let out = step(ctx.input(0), ctx.input(1), epsilon);
        ctx.emit(out);
    }
}

/// Register this node with the runtime registry. Called (via codegen) from
/// `ph2d-node-registry-init::register_all_nodes`.
pub fn register(reg: &mut NodeRegistry) -> Result<(), RegistryError> {
    reg.register(Box::new(PulseOnChange))?;
    reg.register_ui(
        MANIFEST.id,
        ph2d_node_registry::NodeUiManifest {
            display_name: "On Change",
            // Utility grey: a value→pulse adapter, not a visible transform.
            category: ph2d_node_registry::NodeUiCategory::Utility,
            silhouette: ph2d_node_registry::NodeSilhouette::Rect,
        },
    );
    reg.register_param_ui(MANIFEST.id, PARAM_HINTS);
    Ok(())
}

use ph2d_node_registry::{ParamUiHint, ParamWidget};

static PARAM_HINTS: &[ParamUiHint] = &[ParamUiHint {
    param: "epsilon",
    label: "Epsilon",
    min: 0.0,
    max: 1.0,
    step: 0.001,
    widget: ParamWidget::Slider,
}];

#[cfg(test)]
mod tests {
    use super::*;
    use ph2d_nodegraph::cook::OpResolver;

    fn value(v: f32) -> Stream {
        Stream::new(1).with(VALUE_COL, Column::Scalar(vec![v]))
    }
    fn fired(s: &Stream) -> f32 {
        match s.get(PULSE_COL).unwrap() {
            Column::Scalar(v) => v[0],
            _ => panic!(),
        }
    }

    /// The core: a pulse fires the tick the value STEPS, and only then — a held
    /// value stays quiet. A staircase input fires once per step.
    #[test]
    fn it_fires_on_a_step_and_stays_quiet_while_held() {
        let mut state = Stream::new(1);
        // Prime on tick 0 (records 5, never fires).
        state = step(&value(5.0), &state, 0.001);
        assert_eq!(fired(&state), 0.0, "the first tick primes, never fires");
        // Held at 5 → no change → quiet.
        state = step(&value(5.0), &state, 0.001);
        assert_eq!(fired(&state), 0.0, "a held value is quiet");
        // Steps to 7 → fires once.
        state = step(&value(7.0), &state, 0.001);
        assert_eq!(fired(&state), 1.0, "the step fires");
        // Holds at 7 → quiet again (fired ONLY on the step, not while high).
        state = step(&value(7.0), &state, 0.001);
        assert_eq!(fired(&state), 0.0, "quiet once the new value is held");
    }

    /// FALSIFICATION with a staircase: a 3-step ramp fires exactly 3 times (once
    /// per step), not once per tick. A level detector would behave differently.
    #[test]
    fn a_staircase_fires_once_per_step() {
        // value 0,0,0,1,1,1,2,2,2 — steps at indices 3 and 6 (index 0 primes).
        let seq = [0.0, 0.0, 0.0, 1.0, 1.0, 1.0, 2.0, 2.0, 2.0];
        let mut state = Stream::new(1);
        let mut out = Vec::new();
        for &v in &seq {
            state = step(&value(v), &state, 0.001);
            out.push(fired(&state));
        }
        assert_eq!(out, vec![0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 1.0, 0.0, 0.0]);
    }

    /// The `epsilon` guard swallows dither: a wobble smaller than epsilon is NOT a
    /// change; a step larger than it IS. Exact-equality noise never chatters.
    #[test]
    fn epsilon_ignores_dither_but_not_a_real_step() {
        let mut state = step(&value(1.0), &Stream::new(1), 0.05); // prime → 1.0
        // A 0.02 wobble (< 0.05 epsilon) → no fire.
        state = step(&value(1.02), &state, 0.05);
        assert_eq!(fired(&state), 0.0, "sub-epsilon dither is not a change");
        // A 0.2 step (> epsilon) → fires. (prev is 1.02 now.)
        state = step(&value(1.3), &state, 0.05);
        assert_eq!(fired(&state), 1.0, "a real step fires");
    }

    /// Unary over the FIELD: each instance watches its own value. Two dots, only
    /// one of which steps → only that one's pulse fires.
    #[test]
    fn it_watches_each_instance_of_the_field_independently() {
        let two = |a: f32, b: f32| Stream::new(2).with(VALUE_COL, Column::Scalar(vec![a, b]));
        let mut state = step(&two(1.0, 2.0), &Stream::new(2), 0.001); // prime
        state = step(&two(1.0, 9.0), &state, 0.001); // only dot 1 steps
        match state.get(PULSE_COL).unwrap() {
            Column::Scalar(v) => assert_eq!(v, &vec![0.0, 1.0], "only the changed dot fires"),
            _ => panic!(),
        }
    }

    #[test]
    fn registers_and_resolves() {
        let mut reg = NodeRegistry::new();
        register(&mut reg).unwrap();
        assert!(reg.resolve(MANIFEST.id).is_some());
    }
}
