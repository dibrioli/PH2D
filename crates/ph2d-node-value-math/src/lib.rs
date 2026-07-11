//! `value.math` — the value-domain COMBINER: fold two **value** fields into one
//! with an arithmetic operation (Motion Nodes M2, the value domain — doc 12/14).
//! This is the first node that takes TWO value fields, so it is where the doc-12
//! broadcast rule is first exercised between two *fields* (until now only the
//! consumer `motion.drive` broadcast, and only a value against the transform
//! stream). It is TouchDesigner's **Math CHOP** (Combine), Cavalry's **Math**,
//! Nuke's Merge(math): one node, an operation selector — not a per-op node
//! explosion (the mature node editors all converge on the single multi-op node,
//! and the value domain's whole ethos is *author once*, doc 12).
//!
//! **The one broadcast rule (the load-bearing decision, doc 12):** the output is
//! `max(len_a, len_b)` long; a length-1 field is HELD (broadcast) at every index;
//! two length-N fields combine element-wise; any other pairing is a mismatch
//! (`debug_assert`, then a lenient element-wise read — no silent no-op), exactly
//! the `1→N`-only rule `motion.drive` uses. It is what makes
//! `value.instance_field × value.lfo → …` — a per-element **spatial gradient
//! modulated in time** — a single wire: the length-N gradient times the length-1
//! global oscillation, broadcast. A disconnected input reads as the zero field
//! `{0}` (the additive-identity degenerate field, matching `motion.drive`'s
//! `value_at`).
//!
//! **The value type** is the continuous per-instance field `(Instances, Scalar,
//! Frame)` on the `v` column (doc 12). `Pure` (no clock, no state — a straight
//! combinator). Transcendental-free (HR-5): only `+ − × ÷` and `min`/`max`;
//! division is IEEE-deterministic and guarded against a (near-)zero divisor
//! (collapses to `0.0`) so a downstream field never sees `inf`/`NaN`.

#![forbid(unsafe_code)]

use ph2d_node_registry::{NodeRegistry, RegistryError};
use ph2d_nodegraph::attr::{Column, Stream};
use ph2d_nodegraph::cook::EvalCtx;
use ph2d_nodegraph::effect::Effect;
use ph2d_nodegraph::node::{LoweringKind, NodeManifest, NodeOp, NodeTypeId, ParamSpec, PortSpec};
use ph2d_nodegraph::port::{Clock, Dim, Domain, PortType};

/// The value type — the continuous per-instance scalar field on the `v` column
/// (mirror of `ph2d_node_pulse_counter::VALUE`; kept local so this stays a leaf
/// drop-crate — the shared vocabulary is the port, not a shared symbol).
pub const VALUE: PortType = PortType::new(Domain::Instances, Dim::Scalar, Clock::Frame);

/// The value column, both inputs and the output (the canonical `value`-domain
/// column).
const VALUE_COL: &str = "v";
/// Below this magnitude a divisor is treated as zero (the quotient collapses to
/// `0.0`), so `Divide` never emits `inf`/`NaN` into a downstream field.
const MIN_DIVISOR: f32 = 1e-9;

/// The static contract of this node type (ADR-0031).
pub const MANIFEST: NodeManifest = NodeManifest {
    id: NodeTypeId::of("value.math"),
    name: "value.math",
    inputs: &[
        PortSpec {
            name: "a",
            ty: VALUE,
        },
        PortSpec {
            name: "b",
            ty: VALUE,
        },
    ],
    outputs: &[PortSpec {
        name: "out",
        ty: VALUE,
    }],
    effect: Effect::Pure,
    clock: Clock::Frame,
    params: &[
        // 0 Add · 1 Subtract · 2 Multiply · 3 Divide · 4 Min · 5 Max — the
        // reference-convergent core (TD Math CHOP / Cavalry Math). Power/log are
        // omitted by HR-5.
        ParamSpec {
            name: "op",
            default: 0.0,
        },
    ],
    lowerings: &[LoweringKind::Cpu],
};

/// The arithmetic operation two value fields are folded by (the TD Math CHOP /
/// Cavalry Math combine vocabulary, restricted to the HR-5-safe core).
#[derive(Copy, Clone, PartialEq, Eq)]
enum Op {
    Add,
    Subtract,
    Multiply,
    Divide,
    Min,
    Max,
}

impl Op {
    fn from_param(v: f32) -> Self {
        match v.round() as i32 {
            1 => Op::Subtract,
            2 => Op::Multiply,
            3 => Op::Divide,
            4 => Op::Min,
            5 => Op::Max,
            _ => Op::Add,
        }
    }
    /// Combine one pair of samples. Division by a (near-)zero divisor collapses
    /// to `0.0` rather than producing `inf`/`NaN` — the documented guard.
    fn apply(self, a: f32, b: f32) -> f32 {
        match self {
            Op::Add => a + b,
            Op::Subtract => a - b,
            Op::Multiply => a * b,
            Op::Divide => {
                if b.abs() < MIN_DIVISOR {
                    0.0
                } else {
                    a / b
                }
            }
            Op::Min => a.min(b),
            Op::Max => a.max(b),
        }
    }
}

/// The sample of value field `v` at index `i`, applying the `1→N` broadcast
/// rule: a length-1 field is held at every index; a length-N field is read
/// element-wise; a missing field reads as `0.0` (the zero degenerate field).
fn field_at(v: &[f32], i: usize) -> f32 {
    match v.len() {
        0 => 0.0,
        1 => v[0], // broadcast: one value → every index (the 1→N rule)
        _ => v.get(i).copied().unwrap_or(0.0),
    }
}

fn scalar_col(s: &Stream, name: &str) -> Vec<f32> {
    match s.get(name) {
        Some(Column::Scalar(v)) => v.clone(),
        _ => Vec::new(),
    }
}

/// Combine two value fields by `op` under the broadcast rule. Output length is
/// `max(len_a, len_b)`; a length that is neither 1 nor the other's length is a
/// mismatch — `debug_assert`ed loudly, then read leniently (element-wise, `0.0`
/// past the end) so a release build degrades rather than panics.
fn combine(a: &[f32], b: &[f32], op: Op) -> Vec<f32> {
    debug_assert!(
        a.len() <= 1 || b.len() <= 1 || a.len() == b.len(),
        "value.math: field lengths {} and {} are neither broadcast (1) nor equal",
        a.len(),
        b.len()
    );
    let n = a.len().max(b.len());
    (0..n)
        .map(|i| op.apply(field_at(a, i), field_at(b, i)))
        .collect()
}

struct ValueMath;

impl NodeOp for ValueMath {
    fn manifest(&self) -> &'static NodeManifest {
        &MANIFEST
    }

    fn eval(&self, ctx: &mut EvalCtx<'_>) {
        let op = Op::from_param(ctx.param("op"));
        let a = scalar_col(ctx.input(0), VALUE_COL);
        let b = scalar_col(ctx.input(1), VALUE_COL);
        let out = combine(&a, &b, op);
        ctx.emit(Stream::new(out.len()).with(VALUE_COL, Column::Scalar(out)));
    }
}

/// Register this node with the runtime registry. Called (via codegen) from
/// `ph2d-node-registry-init::register_all_nodes`.
pub fn register(reg: &mut NodeRegistry) -> Result<(), RegistryError> {
    reg.register(Box::new(ValueMath))?;
    reg.register_ui(
        MANIFEST.id,
        ph2d_node_registry::NodeUiManifest {
            display_name: "Math",
            // Utility grey: a value→value combiner, plumbing (not a transform).
            category: ph2d_node_registry::NodeUiCategory::Utility,
            silhouette: ph2d_node_registry::NodeSilhouette::Rect,
        },
    );
    reg.register_param_ui(MANIFEST.id, PARAM_HINTS);
    Ok(())
}

use ph2d_node_registry::{ParamUiHint, ParamWidget};

static PARAM_HINTS: &[ParamUiHint] = &[ParamUiHint {
    param: "op",
    label: "Op",
    min: 0.0,
    max: 5.0,
    step: 1.0,
    widget: ParamWidget::Enum {
        labels: &["Add", "Subtract", "Multiply", "Divide", "Min", "Max"],
    },
}];

#[cfg(test)]
mod tests {
    use super::*;
    use ph2d_nodegraph::cook::{Cook, OpResolver};
    use ph2d_nodegraph::graph::{Edge, Graph};

    // Direct unit tests of the core (no cook needed for the arithmetic).

    /// Each op folds a pair the way its name says — the reference-convergent
    /// combine vocabulary (TD Math CHOP).
    #[test]
    fn every_op_combines_the_pair_as_named() {
        assert_eq!(Op::Add.apply(2.0, 3.0), 5.0);
        assert_eq!(Op::Subtract.apply(2.0, 3.0), -1.0);
        assert_eq!(Op::Multiply.apply(2.0, 3.0), 6.0);
        assert_eq!(Op::Divide.apply(6.0, 3.0), 2.0);
        assert_eq!(Op::Min.apply(2.0, 3.0), 2.0);
        assert_eq!(Op::Max.apply(2.0, 3.0), 3.0);
    }

    /// FALSIFICATION of the divide guard: dividing by a (near-)zero divisor
    /// collapses to `0.0` — a downstream field never sees `inf`/`NaN`. An
    /// unguarded `a / 0.0` would be non-finite and poison the whole graph.
    #[test]
    fn divide_by_zero_collapses_to_zero_not_infinity() {
        let q = Op::Divide.apply(5.0, 0.0);
        assert!(q.is_finite(), "guarded: finite, not inf/NaN");
        assert_eq!(q, 0.0, "collapses to 0");
        // A divisor below the epsilon is treated as zero too.
        assert_eq!(Op::Divide.apply(5.0, 1e-12), 0.0);
        // A real divisor still divides.
        assert_eq!(Op::Divide.apply(5.0, 2.0), 2.5);
    }

    /// The broadcast rule (doc 12): a length-1 field is HELD at every index of a
    /// length-N field — the whole point of the combiner, and what makes
    /// `gradient(N) × global(1)` one wire. Falsifiable: an element-wise-only
    /// implementation would read `b` past its single entry as 0 and multiply the
    /// tail to 0.
    #[test]
    fn a_length_one_field_broadcasts_across_a_length_n_field() {
        // gradient [0, 0.5, 1] × global 2.0 → [0, 1, 2] (b broadcast to all 3).
        let out = combine(&[0.0, 0.5, 1.0], &[2.0], Op::Multiply);
        assert_eq!(out, vec![0.0, 1.0, 2.0], "the single b held at every index");
        // Symmetric: a length-1 `a` broadcasts against a length-N `b`.
        let out = combine(&[10.0], &[1.0, 2.0, 3.0], Op::Add);
        assert_eq!(
            out,
            vec![11.0, 12.0, 13.0],
            "the single a held at every index"
        );
    }

    /// Two equal-length fields combine element-wise, length preserved.
    #[test]
    fn two_length_n_fields_combine_element_wise() {
        let out = combine(&[1.0, 2.0, 3.0], &[10.0, 20.0, 30.0], Op::Add);
        assert_eq!(out, vec![11.0, 22.0, 33.0]);
    }

    /// A disconnected (empty) input reads as the zero field: `a + {} = a`
    /// (additive identity passthrough), while `a × {} = 0` (the documented
    /// consequence of the zero degenerate field). The output still tracks the
    /// connected input's length.
    #[test]
    fn a_disconnected_input_is_the_zero_field() {
        assert_eq!(
            combine(&[1.0, 2.0], &[], Op::Add),
            vec![1.0, 2.0],
            "add: passthrough of the connected field"
        );
        assert_eq!(
            combine(&[1.0, 2.0], &[], Op::Multiply),
            vec![0.0, 0.0],
            "multiply: the zero field collapses it"
        );
        // Both empty → empty (no field at all).
        assert!(combine(&[], &[], Op::Add).is_empty());
    }

    // Two value sources with distinct type ids (the `motion.drive` two-source
    // harness): field `a` is length-3, field `b` is length-1, so the cook sees
    // the broadcast max.
    static SRC_A_MAN: NodeManifest = NodeManifest {
        id: NodeTypeId::of("value.math.test.a"),
        name: "value.math.test.a",
        inputs: &[],
        outputs: &[PortSpec {
            name: "out",
            ty: VALUE,
        }],
        effect: Effect::Pure,
        clock: Clock::Frame,
        params: &[],
        lowerings: &[LoweringKind::Cpu],
    };
    static SRC_B_MAN: NodeManifest = NodeManifest {
        id: NodeTypeId::of("value.math.test.b"),
        name: "value.math.test.b",
        inputs: &[],
        outputs: &[PortSpec {
            name: "out",
            ty: VALUE,
        }],
        effect: Effect::Pure,
        clock: Clock::Frame,
        params: &[],
        lowerings: &[LoweringKind::Cpu],
    };
    struct SrcA;
    impl NodeOp for SrcA {
        fn manifest(&self) -> &'static NodeManifest {
            &SRC_A_MAN
        }
        fn eval(&self, ctx: &mut EvalCtx<'_>) {
            ctx.emit(Stream::new(3).with(VALUE_COL, Column::Scalar(vec![0.0, 0.5, 1.0])));
        }
    }
    struct SrcB;
    impl NodeOp for SrcB {
        fn manifest(&self) -> &'static NodeManifest {
            &SRC_B_MAN
        }
        fn eval(&self, ctx: &mut EvalCtx<'_>) {
            ctx.emit(Stream::new(1).with(VALUE_COL, Column::Scalar(vec![2.0])));
        }
    }
    struct Ops;
    impl OpResolver for Ops {
        fn resolve(&self, ty: NodeTypeId) -> Option<&dyn NodeOp> {
            match ty {
                t if t == SRC_A_MAN.id => Some(&SrcA),
                t if t == SRC_B_MAN.id => Some(&SrcB),
                t if t == MANIFEST.id => Some(&ValueMath),
                _ => None,
            }
        }
    }

    /// End-to-end through the cook: a length-3 gradient `a` and a length-1 global
    /// `b` are multiplied, and the output is the broadcast max (length 3) —
    /// exactly the `instance_field × lfo` shape the boot scene wires.
    #[test]
    fn combines_two_value_sources_through_the_cook() {
        let mut g = Graph::new();
        let a = g.add_node("value.math.test.a");
        let b = g.add_node("value.math.test.b");
        let m = g.add_node("value.math");
        g.connect(Edge {
            from: (a, 0),
            to: (m, 0),
            delayed: false,
        })
        .unwrap();
        g.connect(Edge {
            from: (b, 0),
            to: (m, 1),
            delayed: false,
        })
        .unwrap();
        g.set_param(m, "op", 2.0); // Multiply
        let mut cook = Cook::new();
        let out = cook.cook(&g, &Ops, m, 0.0).unwrap();
        match out[0].as_stream().get(VALUE_COL).unwrap() {
            Column::Scalar(v) => assert_eq!(v, &vec![0.0, 1.0, 2.0], "length-3 × broadcast 2"),
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
