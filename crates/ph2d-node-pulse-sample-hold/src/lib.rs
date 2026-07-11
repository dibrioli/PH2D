//! `pulse.sample_hold` — the DISCRETE SAMPLER of a continuous value (Motion Nodes
//! M2, the value domain — doc 12/14). On a pulse's rising edge it **samples** the
//! `value` field and **holds** it until the next pulse; between pulses the output
//! is the last sampled value. This is the modular-synth Sample & Hold / Max
//! `sah~` / TouchDesigner Hold CHOP — Shannon sampling made a node, the bridge
//! from the continuous value domain back into a stepped, event-timed one.
//!
//! It closes the canonical combo the plan has been building toward (doc 09):
//! `value.lfo → pulse.sample_hold ← pulse.beat`, then `→ motion.drive` — a smooth
//! oscillation frozen into discrete steps on every beat (the "random"-looking
//! staircase of a sequencer sampling an LFO).
//!
//! **Sequential — it holds state.** The held value + the previous pulse (for edge
//! detection) ride the `pre` self-loop on the `state` port, exactly like
//! `pulse.counter`/`motion.strobe`. On the FIRST tick it primes (samples
//! immediately) so the output is live from the start rather than a dead 0 — the
//! same first-tick discipline as `pulse.beat`. Edge-safe: a pulse HELD high
//! re-samples once (on the rising edge), not once per tick.
//!
//! **Broadcast rule (doc 12):** the held field mirrors the `value` field's length
//! `N`; the `pulse` is read per-instance with the `1→N` hold (a length-1 global
//! pulse triggers every instance together; a length-N pulse triggers each on its
//! own edge). `Effect::Pure` — the tick enters the fingerprint through the
//! consumed `pre` edge. Transcendental-free (HR-5): comparisons and copies only.

#![forbid(unsafe_code)]

use ph2d_node_registry::{NodeRegistry, RegistryError};
use ph2d_nodegraph::attr::{Column, Stream};
use ph2d_nodegraph::cook::EvalCtx;
use ph2d_nodegraph::effect::Effect;
use ph2d_nodegraph::node::{LoweringKind, NodeManifest, NodeOp, NodeTypeId, PortSpec};
use ph2d_nodegraph::port::{Clock, Dim, Domain, PortType};

/// The pulse type (mirror of `ph2d_node_pulse_beat::PULSE`; kept local so this
/// crate stays a leaf drop-crate — the shared vocabulary is the port, not a
/// shared symbol).
const PULSE: PortType = PortType::new(Domain::Instances, Dim::Scalar, Clock::Event);
/// The value type — the continuous per-instance scalar field on the `v` column
/// (mirror of `ph2d_node_pulse_counter::VALUE`).
pub const VALUE: PortType = PortType::new(Domain::Instances, Dim::Scalar, Clock::Frame);

/// The value column, sampled in and held out (the canonical `value` column).
const VALUE_COL: &str = "v";
/// The pulse stream's fire column.
const PULSE_COL: &str = "pulse";
/// Last tick's pulse, carried on the `pre` self-loop for rising-edge detection.
const PREV_COL: &str = "sh_prev";
/// `1.0` once the first sample has been taken (the priming flag), on the `pre`
/// self-loop.
const PRIMED_COL: &str = "sh_primed";

/// The static contract of this node type (ADR-0031).
pub const MANIFEST: NodeManifest = NodeManifest {
    id: NodeTypeId::of("pulse.sample_hold"),
    name: "pulse.sample_hold",
    inputs: &[
        PortSpec {
            name: "value",
            ty: VALUE,
        },
        PortSpec {
            name: "pulse",
            ty: PULSE,
        },
        // Feedback: last tick's output carries the held value + `sh_prev` +
        // `sh_primed`. Named `state` so the editor plumbs its `pre` loop on drop.
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
    params: &[],
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

/// The pulse driving instance `i`, applying the `1→N` broadcast rule: a length-1
/// pulse fires every instance together; a length-N pulse fires each on its own.
fn pulse_at(pulse: &Stream, i: usize) -> f32 {
    match pulse.get(PULSE_COL) {
        Some(Column::Scalar(v)) => match v.len() {
            0 => 0.0,
            1 => v[0], // broadcast: one global trigger → every instance
            _ => v.get(i).copied().unwrap_or(0.0),
        },
        _ => 0.0,
    }
}

/// One tick of sample-and-hold. `N` mirrors the `value` field: each instance
/// re-samples on its pulse's rising edge (or on the very first tick, to prime),
/// otherwise holds last tick's value.
fn step(value: &Stream, pulse: &Stream, state: &Stream) -> Stream {
    let n = value.count();
    let vals = scalar_col(value, VALUE_COL, n);
    let prev_held = scalar_col(state, VALUE_COL, n);
    let prev_pulse = scalar_col(state, PREV_COL, n);
    let primed = scalar_col(state, PRIMED_COL, n);

    let mut held = Vec::with_capacity(n);
    let mut this_pulse = Vec::with_capacity(n);
    for i in 0..n {
        let p = pulse_at(pulse, i);
        let rising = p > 0.5 && prev_pulse[i] <= 0.5;
        // Prime on the first tick (nothing sampled yet) so the output is live
        // from the start, not a dead 0.
        let sample = primed[i] <= 0.5 || rising;
        held.push(if sample { vals[i] } else { prev_held[i] });
        this_pulse.push(p);
    }
    Stream::new(n)
        .with(VALUE_COL, Column::Scalar(held))
        // This tick's pulse becomes next tick's `prev` (the edge memory).
        .with(PREV_COL, Column::Scalar(this_pulse))
        .with(PRIMED_COL, Column::Scalar(vec![1.0; n]))
}

struct PulseSampleHold;

impl NodeOp for PulseSampleHold {
    fn manifest(&self) -> &'static NodeManifest {
        &MANIFEST
    }

    fn eval(&self, ctx: &mut EvalCtx<'_>) {
        let out = step(ctx.input(0), ctx.input(1), ctx.input(2));
        ctx.emit(out);
    }
}

/// Register this node with the runtime registry. Called (via codegen) from
/// `ph2d-node-registry-init::register_all_nodes`.
pub fn register(reg: &mut NodeRegistry) -> Result<(), RegistryError> {
    reg.register(Box::new(PulseSampleHold))?;
    reg.register_ui(
        MANIFEST.id,
        ph2d_node_registry::NodeUiManifest {
            display_name: "Sample & Hold",
            // Utility grey: a pulse+value → value sampler, plumbing.
            category: ph2d_node_registry::NodeUiCategory::Utility,
            silhouette: ph2d_node_registry::NodeSilhouette::Rect,
        },
    );
    // No params — a pure edge-triggered sampler.
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use ph2d_nodegraph::cook::OpResolver;

    fn value(v: f32) -> Stream {
        Stream::new(1).with(VALUE_COL, Column::Scalar(vec![v]))
    }
    fn fire(p: f32) -> Stream {
        Stream::new(1).with(PULSE_COL, Column::Scalar(vec![p]))
    }
    fn held(s: &Stream) -> f32 {
        match s.get(VALUE_COL).unwrap() {
            Column::Scalar(v) => v[0],
            _ => panic!(),
        }
    }

    /// The core behaviour: the output HOLDS between pulses and only updates to the
    /// input's current value on a pulse's rising edge. A continuous ramp sampled
    /// on sparse pulses becomes a staircase.
    #[test]
    fn it_holds_between_pulses_and_samples_on_the_rising_edge() {
        let mut state = Stream::new(1);
        // Prime on tick 0: samples the input (10) immediately.
        state = step(&value(10.0), &fire(0.0), &state);
        assert_eq!(held(&state), 10.0, "primed to the first input");
        // Input moves to 20 but NO pulse → still holds 10.
        state = step(&value(20.0), &fire(0.0), &state);
        assert_eq!(held(&state), 10.0, "holds through a moving input");
        // A pulse rises → samples the current input (30).
        state = step(&value(30.0), &fire(1.0), &state);
        assert_eq!(held(&state), 30.0, "the rising edge samples");
        // Input moves to 40, pulse still high (no NEW rising edge) → holds 30.
        state = step(&value(40.0), &fire(1.0), &state);
        assert_eq!(held(&state), 30.0, "a held-high pulse does not re-sample");
    }

    /// FALSIFICATION of edge-safety: a pulse HELD high across many ticks samples
    /// ONCE (on the rising edge), not once per tick. If it sampled every high
    /// tick, the output would track the input while high.
    #[test]
    fn a_held_high_pulse_samples_once_not_every_tick() {
        let mut state = step(&value(1.0), &fire(0.0), &Stream::new(1)); // prime → 1
        // Pulse rises at the moving input 5, then stays high while input climbs.
        let inputs = [5.0, 6.0, 7.0, 8.0];
        for (k, &v) in inputs.iter().enumerate() {
            state = step(&value(v), &fire(1.0), &state);
            if k == 0 {
                assert_eq!(held(&state), 5.0, "samples on the rising edge");
            } else {
                assert_eq!(held(&state), 5.0, "holds 5 while the pulse stays high");
            }
        }
    }

    /// A fresh pulse (fall then rise again) re-samples the new input value — the
    /// staircase steps.
    #[test]
    fn a_new_rising_edge_re_samples() {
        let mut state = step(&value(1.0), &fire(1.0), &Stream::new(1)); // prime+fire → 1
        state = step(&value(2.0), &fire(0.0), &state); // hold 1
        assert_eq!(held(&state), 1.0);
        state = step(&value(2.0), &fire(1.0), &state); // new edge → sample 2
        assert_eq!(held(&state), 2.0, "the next edge steps the staircase");
    }

    /// Broadcast: a length-1 (global) pulse triggers a length-N value field — the
    /// doc-12 `1→N` rule. Two instances both sample their own value on one shared
    /// pulse.
    #[test]
    fn a_global_pulse_samples_a_field() {
        let two = Stream::new(2).with(VALUE_COL, Column::Scalar(vec![10.0, 20.0]));
        let global_pulse = Stream::new(1).with(PULSE_COL, Column::Scalar(vec![1.0]));
        let s = step(&two, &global_pulse, &Stream::new(2));
        match s.get(VALUE_COL).unwrap() {
            Column::Scalar(v) => assert_eq!(v, &vec![10.0, 20.0], "each holds its own value"),
            _ => panic!(),
        }
    }

    /// The whole point of the value bridge: a continuous LFO-like ramp, sampled on
    /// a slow pulse, becomes a STAIRCASE — piecewise constant, stepping only on
    /// the pulses. Falsifiable: a pass-through would track the ramp exactly.
    #[test]
    fn a_ramp_sampled_on_sparse_pulses_becomes_a_staircase() {
        // A ramp 0,1,2,…; a pulse every 3rd tick. Output holds between, steps on.
        let mut state = Stream::new(1);
        let mut out = Vec::new();
        for t in 0..9 {
            let p = if t % 3 == 0 { 1.0 } else { 0.0 };
            state = step(&value(t as f32), &fire(p), &state);
            out.push(held(&state));
        }
        // Samples at t=0,3,6 (values 0,3,6), holds in between.
        assert_eq!(out, vec![0.0, 0.0, 0.0, 3.0, 3.0, 3.0, 6.0, 6.0, 6.0]);
    }

    #[test]
    fn registers_and_resolves() {
        let mut reg = NodeRegistry::new();
        register(&mut reg).unwrap();
        assert!(reg.resolve(MANIFEST.id).is_some());
    }
}
