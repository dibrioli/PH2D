//! `value.lfo` — the value-domain PRODUCER of a continuous oscillation (Motion
//! Nodes M2, the value domain — doc 12). It is the pure *producer* form of
//! `motion.oscillator`: where the oscillator bundles the wave AND writes a
//! transform channel, this emits the wave as a **value** on its own socket, to be
//! routed by `motion.drive` (or reshaped by `value.map_range`, gated by
//! `pulse.sample_hold`, …). This is the TouchDesigner **LFO CHOP** / the Cavalry
//! oscillator behaviour, made a first-class value source (doc 12 §5).
//!
//! **The value type** is the continuous per-instance scalar field
//! `(Instances, Scalar, Frame)` on the `v` column — the continuous dual of the
//! pulse (doc 12). Cardinality follows the geometry: the optional `in` port is
//! read for its **count only** (like the oscillator reads N from its stream) —
//! connected → a length-N field with per-instance `phase_stagger` (a travelling
//! wave across the grid); **unconnected → a length-1 field** (one global
//! oscillation, held across every instance by `motion.drive`'s broadcast rule).
//! Nothing from the input stream is passed through — this mints a fresh value.
//!
//! Reads the playhead, holds no state → `Effect::Temporal` (pull-side, like the
//! oscillator). The waveform math is transcendental-free (HR-5); see `wave.rs`.
//!
//! `value_i = waveform(wave, t / period + phase + i · phase_stagger) · amplitude
//! + offset`, with `period` clamped to `MIN_PERIOD` (never divides by zero).

#![forbid(unsafe_code)]

use ph2d_node_registry::{NodeRegistry, RegistryError};
use ph2d_nodegraph::attr::{Column, Stream};
use ph2d_nodegraph::cook::EvalCtx;
use ph2d_nodegraph::effect::Effect;
use ph2d_nodegraph::node::{LoweringKind, NodeManifest, NodeOp, NodeTypeId, ParamSpec, PortSpec};
use ph2d_nodegraph::port::{Clock, Dim, Domain, PortType};

mod wave;
use wave::waveform;

/// The instance stream type — read for its count only (the optional `in` port).
const INST_VEC2: PortType = PortType::new(Domain::Instances, Dim::Vec2, Clock::Frame);
/// The value type — the continuous per-instance scalar field on the `v` column
/// (mirror of `ph2d_node_pulse_counter::VALUE`; kept local so this stays a leaf
/// drop-crate — the shared vocabulary is the port, not a shared symbol).
pub const VALUE: PortType = PortType::new(Domain::Instances, Dim::Scalar, Clock::Frame);

/// The value output column (the canonical `value`-domain column).
const VALUE_COL: &str = "v";
/// The smallest cycle length, so `t / period` never divides by zero (mirror of
/// `pulse.beat`'s guard).
const MIN_PERIOD: f32 = 1e-3;

/// The static contract of this node type (ADR-0031).
pub const MANIFEST: NodeManifest = NodeManifest {
    id: NodeTypeId::of("value.lfo"),
    name: "value.lfo",
    // Optional: connected → count N + per-instance stagger; unconnected → one
    // global value. Read for its count only; never passed through.
    inputs: &[PortSpec {
        name: "in",
        ty: INST_VEC2,
    }],
    outputs: &[PortSpec {
        name: "out",
        ty: VALUE,
    }],
    // Reads the playhead → pull-side; HR-5-exempt for the clock (the waveform
    // math is nonetheless transcendental-free for cross-platform stability).
    effect: Effect::Temporal,
    clock: Clock::Frame,
    params: &[
        // Shape — 0 Sine (parabolic) · 1 Tri · 2 Square · 3 Saw · 4 Spike.
        ParamSpec {
            name: "wave",
            default: 0.0,
        },
        // Seconds per cycle (clamped ≥ MIN_PERIOD). The pulse-family vocabulary
        // (matches `pulse.beat`'s `period`) rather than the oscillator's frequency.
        ParamSpec {
            name: "period",
            default: 1.0,
        },
        // Peak of the oscillation (value-native units).
        ParamSpec {
            name: "amplitude",
            default: 1.0,
        },
        // A DC shift of the oscillation centre.
        ParamSpec {
            name: "offset",
            default: 0.0,
        },
        // A global phase offset (cycles) — where in the cycle it starts.
        ParamSpec {
            name: "phase",
            default: 0.0,
        },
        // Per-instance phase offset (cycles) → a travelling wave across the field
        // (needs a connected `in` for N > 1; 0 → every instance in lock-step).
        ParamSpec {
            name: "phase_stagger",
            default: 0.0,
        },
    ],
    lowerings: &[LoweringKind::Cpu],
};

struct ValueLfo;

impl NodeOp for ValueLfo {
    fn manifest(&self) -> &'static NodeManifest {
        &MANIFEST
    }

    fn eval(&self, ctx: &mut EvalCtx<'_>) {
        let wave = ctx.param("wave").round() as i32;
        let period = ctx.param("period").max(MIN_PERIOD);
        let amplitude = ctx.param("amplitude");
        let offset = ctx.param("offset");
        let phase0 = ctx.param("phase");
        let stagger = ctx.param("phase_stagger");
        let t = ctx.playhead() as f32;
        // Cardinality follows the geometry: N from the (optional) input, else the
        // length-1 global oscillation (broadcast by `motion.drive`).
        let n = ctx.input(0).count().max(1);
        let value: Vec<f32> = (0..n)
            .map(|i| {
                let phase = t / period + phase0 + i as f32 * stagger;
                waveform(wave, phase) * amplitude + offset
            })
            .collect();
        ctx.emit(Stream::new(n).with(VALUE_COL, Column::Scalar(value)));
    }
}

/// Register this node with the runtime registry. Called (via codegen) from
/// `ph2d-node-registry-init::register_all_nodes`.
pub fn register(reg: &mut NodeRegistry) -> Result<(), RegistryError> {
    reg.register(Box::new(ValueLfo))?;
    reg.register_ui(
        MANIFEST.id,
        ph2d_node_registry::NodeUiManifest {
            display_name: "LFO",
            // Utility grey: a value SOURCE, plumbing (not a visible transform).
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
        param: "wave",
        label: "Wave",
        min: 0.0,
        max: 4.0,
        step: 1.0,
        widget: ParamWidget::Enum {
            labels: &["Sine", "Tri", "Square", "Saw", "Spike"],
        },
    },
    ParamUiHint {
        param: "period",
        label: "Period",
        min: 0.05,
        max: 8.0,
        step: 0.05,
        widget: ParamWidget::Slider,
    },
    ParamUiHint {
        param: "amplitude",
        label: "Amplitude",
        min: 0.0,
        max: 10.0,
        step: 0.05,
        widget: ParamWidget::Slider,
    },
    ParamUiHint {
        param: "offset",
        label: "Offset",
        min: -10.0,
        max: 10.0,
        step: 0.05,
        widget: ParamWidget::Slider,
    },
    ParamUiHint {
        param: "phase",
        label: "Phase",
        min: 0.0,
        max: 1.0,
        step: 0.01,
        widget: ParamWidget::Slider,
    },
    ParamUiHint {
        param: "phase_stagger",
        label: "Stagger",
        min: 0.0,
        max: 2.0,
        step: 0.02,
        widget: ParamWidget::Slider,
    },
];

#[cfg(test)]
mod tests {
    use super::*;
    use ph2d_nodegraph::cook::{Cook, OpResolver};
    use ph2d_nodegraph::graph::{Edge, Graph, NodeId};

    // A grid source: `n` instances at the origin, so the LFO can read a count.
    static GRID_MAN: NodeManifest = NodeManifest {
        id: NodeTypeId::of("value.lfo.test.grid"),
        name: "value.lfo.test.grid",
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
    struct Grid;
    impl NodeOp for Grid {
        fn manifest(&self) -> &'static NodeManifest {
            &GRID_MAN
        }
        fn eval(&self, ctx: &mut EvalCtx<'_>) {
            ctx.emit(Stream::new(3).with("P", Column::Vec2(vec![[0.0, 0.0]; 3])));
        }
    }
    struct Ops;
    impl OpResolver for Ops {
        fn resolve(&self, ty: NodeTypeId) -> Option<&dyn NodeOp> {
            match ty {
                t if t == GRID_MAN.id => Some(&Grid),
                t if t == MANIFEST.id => Some(&ValueLfo),
                _ => None,
            }
        }
    }

    fn vals(s: &Stream) -> Vec<f32> {
        match s.get(VALUE_COL).unwrap() {
            Column::Scalar(v) => v.clone(),
            _ => panic!("v"),
        }
    }

    /// Cook the LFO at `playhead`; `connect_grid` decides whether it reads a
    /// count from a source (length-N) or stands alone (length-1 global).
    fn lfo_at(
        playhead: f64,
        connect_grid: bool,
        setup: impl FnOnce(&mut Graph, NodeId),
    ) -> Vec<f32> {
        let mut g = Graph::new();
        let lfo = g.add_node("value.lfo");
        if connect_grid {
            let grid = g.add_node("value.lfo.test.grid");
            g.connect(Edge {
                from: (grid, 0),
                to: (lfo, 0),
                delayed: false,
            })
            .unwrap();
        }
        setup(&mut g, lfo);
        let mut cook = Cook::new();
        let out = cook.cook(&g, &Ops, lfo, playhead).unwrap();
        vals(out[0].as_stream())
    }

    /// UNCONNECTED input → one global value (length-1). This is the field that
    /// `motion.drive` broadcasts across every instance (the doc-12 rule).
    #[test]
    fn an_unconnected_lfo_emits_a_single_global_value() {
        // Default Sine at t=0 is 0 (phase 0). amplitude 5 → still 0 at the zero.
        let v = lfo_at(0.0, false, |g, lfo| {
            g.set_param(lfo, "amplitude", 5.0);
        });
        assert_eq!(v, vec![0.0], "one global value");
        // A quarter period reaches the parabolic peak → +amplitude.
        let v = lfo_at(0.5, false, |g, lfo| {
            g.set_param(lfo, "period", 2.0); // t=0.5 → phase ¼
            g.set_param(lfo, "amplitude", 3.0);
        });
        assert_eq!(v, vec![3.0], "quarter period → peak amplitude");
    }

    /// CONNECTED input → the value field's length follows the geometry (N=3),
    /// and `phase_stagger` sends a travelling wave across it: at t=0 with
    /// stagger ¼, instance 0 sits at phase 0 (→0) and instance 1 at the peak.
    #[test]
    fn a_connected_lfo_emits_a_field_with_a_travelling_wave() {
        let v = lfo_at(0.0, true, |g, lfo| {
            g.set_param(lfo, "amplitude", 2.0);
            g.set_param(lfo, "phase_stagger", 0.25);
        });
        assert_eq!(v.len(), 3, "length follows the connected geometry");
        assert_eq!(v[0], 0.0, "instance 0 at phase 0");
        assert_eq!(v[1], 2.0, "instance 1 staggered to the peak");
    }

    /// FALSIFICATION of the clamp/DC path: `offset` shifts the centre and
    /// `phase` advances the cycle without touching the playhead.
    #[test]
    fn offset_shifts_the_centre_and_phase_advances_the_cycle() {
        let v = lfo_at(0.0, false, |g, lfo| g.set_param(lfo, "offset", 2.0));
        assert_eq!(v, vec![2.0], "DC offset with no oscillation");
        // phase ¼ at t=0 starts the cycle at the peak.
        let v = lfo_at(0.0, false, |g, lfo| {
            g.set_param(lfo, "amplitude", 3.0);
            g.set_param(lfo, "phase", 0.25);
        });
        assert_eq!(v, vec![3.0], "phase advances to the peak");
    }

    /// A tiny/zero `period` must not divide by zero — it clamps to `MIN_PERIOD`
    /// and stays finite.
    #[test]
    fn a_zero_period_never_divides_by_zero() {
        let v = lfo_at(1.0, false, |g, lfo| g.set_param(lfo, "period", 0.0));
        assert!(v[0].is_finite(), "clamped period keeps the value finite");
    }

    #[test]
    fn registers_and_resolves() {
        let mut reg = NodeRegistry::new();
        register(&mut reg).unwrap();
        assert!(reg.resolve(MANIFEST.id).is_some());
    }
}
