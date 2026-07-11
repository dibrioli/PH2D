//! `value.instance_field` — the value-domain SOURCE OF PER-ELEMENT VARIATION
//! (Motion Nodes M2, the value domain — doc 12/13/14). It is the **only** node
//! that mints a genuinely length-N value field out of instance *identity* — every
//! other value so far is either a length-1 global (an LFO, a count) broadcast to
//! all, or a field whose per-element variation was smuggled in through a
//! behaviour's own `phase_stagger`. This is the sanctioned, first-class way that
//! per-element variation is BORN: Houdini's `@ptnum`/`@id`, Cavalry's Index /
//! Falloff, vvvv's spread index, TouchDesigner's Pattern CHOP ramp.
//!
//! **Modes** (`mode`):
//! - **Index** — `0, 1, 2, …, N−1` (the raw ordinal; feed a `value.map_range` to
//!   normalize).
//! - **Ramp** — `i / (N−1)` in `[0,1]` (the normalized gradient; `N=1 → 0`).
//! - **Random** — a stateless hash of `(seed, index) → [0,1)` (Jarzynski/Olano;
//!   transcendental-free, HR-5; see `hash.rs`). A per-instance jitter that
//!   reproduces bit-for-bit under scrub.
//!
//! **The value type** is the continuous per-instance field `(Instances, Scalar,
//! Frame)` on the `v` column (doc 12). Cardinality follows the geometry: the
//! optional `in` port is read for its **count only** (like `value.lfo`) —
//! unconnected → a length-1 field (a degenerate single value). Nothing from the
//! input stream is passed through; this mints a fresh value.
//!
//! `Effect::Pure` — no clock, no state; a pure function of `(N, mode, seed)`.
//! Emits the column `v`. Transcendental-free (HR-5): only integer hash + one
//! division for Ramp.

#![forbid(unsafe_code)]

use ph2d_node_registry::{NodeRegistry, RegistryError};
use ph2d_nodegraph::attr::{Column, Stream};
use ph2d_nodegraph::cook::EvalCtx;
use ph2d_nodegraph::effect::Effect;
use ph2d_nodegraph::node::{LoweringKind, NodeManifest, NodeOp, NodeTypeId, ParamSpec, PortSpec};
use ph2d_nodegraph::port::{Clock, Dim, Domain, PortType};

mod hash;
use hash::rand01;

/// The instance stream type — read for its count only (the optional `in` port).
const INST_VEC2: PortType = PortType::new(Domain::Instances, Dim::Vec2, Clock::Frame);
/// The value type — the continuous per-instance scalar field on the `v` column
/// (mirror of `ph2d_node_pulse_counter::VALUE`; kept local so this stays a leaf
/// drop-crate — the shared vocabulary is the port, not a shared symbol).
pub const VALUE: PortType = PortType::new(Domain::Instances, Dim::Scalar, Clock::Frame);

/// The value output column (the canonical `value`-domain column).
const VALUE_COL: &str = "v";

/// The static contract of this node type (ADR-0031).
pub const MANIFEST: NodeManifest = NodeManifest {
    id: NodeTypeId::of("value.instance_field"),
    name: "value.instance_field",
    // Optional: connected → count N; unconnected → a single degenerate value.
    // Read for its count only; never passed through.
    inputs: &[PortSpec {
        name: "in",
        ty: INST_VEC2,
    }],
    outputs: &[PortSpec {
        name: "out",
        ty: VALUE,
    }],
    // Pure: a fresh field per cook, a pure function of N + params. No clock/state.
    effect: Effect::Pure,
    clock: Clock::Frame,
    params: &[
        // 0 Index (0..N-1) · 1 Ramp (0..1) · 2 Random (hash → [0,1)).
        ParamSpec {
            name: "mode",
            default: 1.0,
        },
        // Random seed (Random mode only). Integer-valued; rounded at eval.
        ParamSpec {
            name: "seed",
            default: 0.0,
        },
    ],
    lowerings: &[LoweringKind::Cpu],
};

/// How the per-instance value is minted from the instance's ordinal.
#[derive(Copy, Clone, PartialEq, Eq)]
enum FieldMode {
    /// `0, 1, 2, … N−1` — the raw ordinal.
    Index,
    /// `i / (N−1)` in `[0,1]` — the normalized gradient.
    Ramp,
    /// A stateless hash of `(seed, index)` → `[0,1)`.
    Random,
}

impl FieldMode {
    fn from_param(v: f32) -> Self {
        match v.round() as i32 {
            0 => FieldMode::Index,
            2 => FieldMode::Random,
            _ => FieldMode::Ramp,
        }
    }
}

/// Mint the length-`n` value field for `mode`/`seed`.
fn field(n: usize, mode: FieldMode, seed: u32) -> Vec<f32> {
    (0..n)
        .map(|i| match mode {
            FieldMode::Index => i as f32,
            // `N == 1` has no span → the low end (0.0), never a divide by zero.
            FieldMode::Ramp => {
                if n > 1 {
                    i as f32 / (n - 1) as f32
                } else {
                    0.0
                }
            }
            FieldMode::Random => rand01(seed, i as u32),
        })
        .collect()
}

struct ValueInstanceField;

impl NodeOp for ValueInstanceField {
    fn manifest(&self) -> &'static NodeManifest {
        &MANIFEST
    }

    fn eval(&self, ctx: &mut EvalCtx<'_>) {
        let mode = FieldMode::from_param(ctx.param("mode"));
        let seed = ctx.param("seed").round().max(0.0) as u32;
        // Cardinality follows the geometry; unconnected → one degenerate value.
        let n = ctx.input(0).count().max(1);
        let v = field(n, mode, seed);
        ctx.emit(Stream::new(n).with(VALUE_COL, Column::Scalar(v)));
    }
}

/// Register this node with the runtime registry. Called (via codegen) from
/// `ph2d-node-registry-init::register_all_nodes`.
pub fn register(reg: &mut NodeRegistry) -> Result<(), RegistryError> {
    reg.register(Box::new(ValueInstanceField))?;
    reg.register_ui(
        MANIFEST.id,
        ph2d_node_registry::NodeUiManifest {
            display_name: "Instance Field",
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
        param: "mode",
        label: "Mode",
        min: 0.0,
        max: 2.0,
        step: 1.0,
        widget: ParamWidget::Enum {
            labels: &["Index", "Ramp", "Random"],
        },
    },
    ParamUiHint {
        param: "seed",
        label: "Seed",
        min: 0.0,
        max: 9999.0,
        step: 1.0,
        widget: ParamWidget::Slider,
    },
];

#[cfg(test)]
mod tests {
    use super::*;
    use ph2d_nodegraph::cook::{Cook, OpResolver};
    use ph2d_nodegraph::graph::{Edge, Graph};

    // A grid source: `n` instances at the origin, so the field can read a count.
    static GRID_MAN: NodeManifest = NodeManifest {
        id: NodeTypeId::of("value.instance_field.test.grid"),
        name: "value.instance_field.test.grid",
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
    struct Grid(usize);
    impl NodeOp for Grid {
        fn manifest(&self) -> &'static NodeManifest {
            &GRID_MAN
        }
        fn eval(&self, ctx: &mut EvalCtx<'_>) {
            ctx.emit(Stream::new(self.0).with("P", Column::Vec2(vec![[0.0, 0.0]; self.0])));
        }
    }

    // Direct unit tests of the core (no cook needed for the field math).
    #[test]
    fn index_mode_is_the_raw_ordinal() {
        assert_eq!(field(4, FieldMode::Index, 0), vec![0.0, 1.0, 2.0, 3.0]);
    }

    #[test]
    fn ramp_mode_is_the_normalized_gradient() {
        assert_eq!(
            field(5, FieldMode::Ramp, 0),
            vec![0.0, 0.25, 0.5, 0.75, 1.0]
        );
        // N == 1 has no span → 0, never a divide by zero.
        assert_eq!(field(1, FieldMode::Ramp, 0), vec![0.0]);
    }

    /// Random is a per-instance field in `[0,1)` — NOT all-equal (the whole point
    /// vs a broadcast constant) and reproducible for a fixed seed.
    #[test]
    fn random_mode_varies_per_instance_and_reproduces() {
        let a = field(8, FieldMode::Random, 42);
        let b = field(8, FieldMode::Random, 42);
        assert_eq!(a, b, "a fixed seed reproduces the field bit-for-bit");
        assert!(a.iter().all(|&v| (0.0..1.0).contains(&v)), "in [0,1)");
        assert!(
            a.windows(2).any(|w| w[0] != w[1]),
            "the field varies per instance (not a broadcast constant)"
        );
        // A different seed decorrelates.
        assert_ne!(a, field(8, FieldMode::Random, 43), "seed changes the field");
    }

    /// End-to-end through the cook: the field's length follows the connected
    /// geometry, and Ramp really spans 0..1 across the instances.
    #[test]
    fn mints_a_field_sized_to_the_connected_geometry() {
        struct Ops;
        impl OpResolver for Ops {
            fn resolve(&self, ty: NodeTypeId) -> Option<&dyn NodeOp> {
                match ty {
                    t if t == GRID_MAN.id => Some(Box::leak(Box::new(Grid(4))) as &dyn NodeOp),
                    t if t == MANIFEST.id => Some(&ValueInstanceField),
                    _ => None,
                }
            }
        }
        let mut g = Graph::new();
        let grid = g.add_node("value.instance_field.test.grid");
        let f = g.add_node("value.instance_field");
        g.connect(Edge {
            from: (grid, 0),
            to: (f, 0),
            delayed: false,
        })
        .unwrap();
        g.set_param(f, "mode", 1.0); // Ramp
        let mut cook = Cook::new();
        let out = cook.cook(&g, &Ops, f, 0.0).unwrap();
        match out[0].as_stream().get(VALUE_COL).unwrap() {
            Column::Scalar(v) => assert_eq!(v, &vec![0.0, 1.0 / 3.0, 2.0 / 3.0, 1.0]),
            _ => panic!("v"),
        }
    }

    /// UNCONNECTED input → a single degenerate value (length-1) through the cook,
    /// never a panic on an empty input.
    #[test]
    fn an_unconnected_field_is_a_single_value() {
        struct Ops;
        impl OpResolver for Ops {
            fn resolve(&self, ty: NodeTypeId) -> Option<&dyn NodeOp> {
                match ty {
                    t if t == MANIFEST.id => Some(&ValueInstanceField),
                    _ => None,
                }
            }
        }
        let mut g = Graph::new();
        let f = g.add_node("value.instance_field");
        let mut cook = Cook::new();
        let out = cook.cook(&g, &Ops, f, 0.0).unwrap();
        match out[0].as_stream().get(VALUE_COL).unwrap() {
            Column::Scalar(v) => assert_eq!(v.len(), 1, "unconnected → one degenerate value"),
            _ => panic!("v"),
        }
    }

    #[test]
    fn registers_and_resolves() {
        let mut reg = NodeRegistry::new();
        register(&mut reg).unwrap();
        assert!(reg.resolve(MANIFEST.id).is_some());
    }
}
