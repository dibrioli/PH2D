//! `value.map_range` — the value-domain GLUE: linearly remap a **value** field
//! from an input range to an output range (Motion Nodes M2, the value domain —
//! doc 12). This is the single most-used node in every mature value graph:
//! Houdini's `fit()`, TouchDesigner's **Math CHOP** Range, Nuke/Cavalry "Remap",
//! Max's `scale`. It is what turns a raw oscillation or a normalized count into
//! the units a channel actually wants (a `[-1,1]` LFO → a `[0,90]°` swing).
//!
//! **Semantics** (the reference convergence): map `[in_lo, in_hi] → [out_lo,
//! out_hi]` linearly. `t = (v − in_lo) / (in_hi − in_lo)`, then
//! `out = out_lo + t · (out_hi − out_lo)`. Clamping is applied to the *normalized*
//! `t ∈ [0,1]` (so inverted output ranges stay honest) — **on by default**,
//! matching Houdini's canonical `fit()` (turn it off for `efit`-style
//! extrapolation). A degenerate input span (`in_hi == in_lo`) can't divide, so
//! the whole input collapses to `out_lo` (the documented guard) instead of
//! producing `NaN`.
//!
//! **The value type** is the continuous per-instance field `(Instances, Scalar,
//! Frame)` on the `v` column (doc 12). This is a **unary** map — it preserves the
//! field's length exactly (no broadcast decision to make; that rule lives in the
//! *consumer*, `motion.drive`). `Pure` (no clock, no state). Transcendental-free:
//! only `+ − × ÷` and `clamp` (HR-5; division is IEEE-deterministic).

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

/// The value column, in and out (the canonical `value`-domain column).
const VALUE_COL: &str = "v";
/// Below this the input span is treated as degenerate (collapses to `out_lo`),
/// so `map_range` never divides by (near-)zero.
const MIN_SPAN: f32 = 1e-9;

/// The static contract of this node type (ADR-0031).
pub const MANIFEST: NodeManifest = NodeManifest {
    id: NodeTypeId::of("value.map_range"),
    name: "value.map_range",
    inputs: &[PortSpec {
        name: "in",
        ty: VALUE,
    }],
    outputs: &[PortSpec {
        name: "out",
        ty: VALUE,
    }],
    effect: Effect::Pure,
    clock: Clock::Frame,
    params: &[
        ParamSpec {
            name: "in_lo",
            default: 0.0,
        },
        ParamSpec {
            name: "in_hi",
            default: 1.0,
        },
        ParamSpec {
            name: "out_lo",
            default: 0.0,
        },
        ParamSpec {
            name: "out_hi",
            default: 1.0,
        },
        // 0 off (extrapolate, `efit`) · 1 on (clamp to the output range, `fit`).
        // On by default — the Houdini `fit()` convention.
        ParamSpec {
            name: "clamp",
            default: 1.0,
        },
    ],
    lowerings: &[LoweringKind::Cpu],
};

/// Remap one value from `[in_lo, in_hi]` to `[out_lo, out_hi]`. Clamping (when
/// `clamp`) is applied to the normalized parameter `t ∈ [0,1]`, so inverted
/// output ranges stay within bounds. A degenerate input span collapses to
/// `out_lo`.
fn map_one(v: f32, in_lo: f32, in_hi: f32, out_lo: f32, out_hi: f32, clamp: bool) -> f32 {
    let span = in_hi - in_lo;
    if span.abs() < MIN_SPAN {
        return out_lo;
    }
    let mut t = (v - in_lo) / span;
    if clamp {
        t = t.clamp(0.0, 1.0);
    }
    out_lo + t * (out_hi - out_lo)
}

struct ValueMapRange;

impl NodeOp for ValueMapRange {
    fn manifest(&self) -> &'static NodeManifest {
        &MANIFEST
    }

    fn eval(&self, ctx: &mut EvalCtx<'_>) {
        let in_lo = ctx.param("in_lo");
        let in_hi = ctx.param("in_hi");
        let out_lo = ctx.param("out_lo");
        let out_hi = ctx.param("out_hi");
        let clamp = ctx.param("clamp") > 0.5;
        let input: Vec<f32> = match ctx.input(0).get(VALUE_COL) {
            Some(Column::Scalar(v)) => v.clone(),
            _ => Vec::new(),
        };
        let n = input.len();
        // Unary map — the field's length is preserved exactly.
        let out: Vec<f32> = input
            .iter()
            .map(|&v| map_one(v, in_lo, in_hi, out_lo, out_hi, clamp))
            .collect();
        ctx.emit(Stream::new(n).with(VALUE_COL, Column::Scalar(out)));
    }
}

/// Register this node with the runtime registry. Called (via codegen) from
/// `ph2d-node-registry-init::register_all_nodes`.
pub fn register(reg: &mut NodeRegistry) -> Result<(), RegistryError> {
    reg.register(Box::new(ValueMapRange))?;
    reg.register_ui(
        MANIFEST.id,
        ph2d_node_registry::NodeUiManifest {
            display_name: "Map Range",
            // Utility grey: a value→value transformer, plumbing (not a transform).
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
        param: "in_lo",
        label: "In Low",
        min: -100.0,
        max: 100.0,
        step: 0.05,
        widget: ParamWidget::Slider,
    },
    ParamUiHint {
        param: "in_hi",
        label: "In High",
        min: -100.0,
        max: 100.0,
        step: 0.05,
        widget: ParamWidget::Slider,
    },
    ParamUiHint {
        param: "out_lo",
        label: "Out Low",
        min: -100.0,
        max: 100.0,
        step: 0.05,
        widget: ParamWidget::Slider,
    },
    ParamUiHint {
        param: "out_hi",
        label: "Out High",
        min: -100.0,
        max: 100.0,
        step: 0.05,
        widget: ParamWidget::Slider,
    },
    ParamUiHint {
        param: "clamp",
        label: "Clamp",
        min: 0.0,
        max: 1.0,
        step: 1.0,
        widget: ParamWidget::Enum {
            labels: &["Off", "On"],
        },
    },
];

#[cfg(test)]
mod tests {
    use super::*;
    use ph2d_nodegraph::cook::{Cook, OpResolver};
    use ph2d_nodegraph::graph::{Edge, Graph, NodeId};

    /// A value source emitting a fixed field, so map_range can be driven through
    /// a real cook.
    static SRC_MAN: NodeManifest = NodeManifest {
        id: NodeTypeId::of("value.map_range.test.src"),
        name: "value.map_range.test.src",
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
    struct Src(Vec<f32>);
    impl NodeOp for Src {
        fn manifest(&self) -> &'static NodeManifest {
            &SRC_MAN
        }
        fn eval(&self, ctx: &mut EvalCtx<'_>) {
            ctx.emit(Stream::new(self.0.len()).with(VALUE_COL, Column::Scalar(self.0.clone())));
        }
    }

    // Direct unit tests of the core (no cook needed for the math).
    #[test]
    fn maps_the_canonical_ranges() {
        // 0..1 → 0..100: the midpoint lands at 50.
        assert_eq!(map_one(0.5, 0.0, 1.0, 0.0, 100.0, true), 50.0);
        // -1..1 → 0..90 (an LFO → a degree swing): 0 → 45.
        assert_eq!(map_one(0.0, -1.0, 1.0, 0.0, 90.0, true), 45.0);
        assert_eq!(map_one(-1.0, -1.0, 1.0, 0.0, 90.0, true), 0.0);
        assert_eq!(map_one(1.0, -1.0, 1.0, 0.0, 90.0, true), 90.0);
    }

    /// FALSIFICATION of the clamp path: with clamp ON an over-range input pins to
    /// the output bound; with clamp OFF it extrapolates past it (`efit`).
    #[test]
    fn clamp_pins_the_output_and_off_extrapolates() {
        // input 2.0 is past in_hi=1 → clamped to out_hi=10.
        assert_eq!(map_one(2.0, 0.0, 1.0, 0.0, 10.0, true), 10.0);
        // same input, clamp off → extrapolates to 20.
        assert_eq!(map_one(2.0, 0.0, 1.0, 0.0, 10.0, false), 20.0);
    }

    /// An INVERTED output range stays within bounds under clamp (the reason to
    /// clamp `t`, not the raw output): 0..1 → 10..0, input 2 pins to 0, not -10.
    #[test]
    fn an_inverted_output_range_stays_in_bounds() {
        assert_eq!(map_one(2.0, 0.0, 1.0, 10.0, 0.0, true), 0.0);
        assert_eq!(map_one(0.0, 0.0, 1.0, 10.0, 0.0, true), 10.0);
    }

    /// A degenerate input span (`in_lo == in_hi`) collapses to `out_lo` instead
    /// of dividing by zero into `NaN`.
    #[test]
    fn a_degenerate_input_span_never_produces_nan() {
        let v = map_one(5.0, 3.0, 3.0, -2.0, 7.0, true);
        assert!(v.is_finite());
        assert_eq!(v, -2.0, "collapses to out_lo");
    }

    /// End-to-end through the cook: a length-2 value field is remapped
    /// element-wise, length preserved (the unary contract).
    #[test]
    fn remaps_a_field_through_the_cook_preserving_length() {
        struct Ops(Vec<f32>);
        impl OpResolver for Ops {
            fn resolve(&self, ty: NodeTypeId) -> Option<&dyn NodeOp> {
                // Leaked to satisfy the `'static` op borrow in this tiny harness.
                match ty {
                    t if t == SRC_MAN.id => {
                        Some(Box::leak(Box::new(Src(self.0.clone()))) as &dyn NodeOp)
                    }
                    t if t == MANIFEST.id => Some(&ValueMapRange),
                    _ => None,
                }
            }
        }
        let ops = Ops(vec![0.0, 1.0]);
        let mut g = Graph::new();
        let src = g.add_node("value.map_range.test.src");
        let mr = g.add_node("value.map_range");
        g.connect(Edge {
            from: (src, 0),
            to: (mr, 0),
            delayed: false,
        })
        .unwrap();
        set_range(&mut g, mr, 0.0, 1.0, 0.0, 10.0);
        let mut cook = Cook::new();
        let out = cook.cook(&g, &ops, mr, 0.0).unwrap();
        match out[0].as_stream().get(VALUE_COL).unwrap() {
            Column::Scalar(v) => assert_eq!(v, &vec![0.0, 10.0], "element-wise, length 2 kept"),
            _ => panic!("v"),
        }
    }

    fn set_range(g: &mut Graph, n: NodeId, il: f32, ih: f32, ol: f32, oh: f32) {
        g.set_param(n, "in_lo", il);
        g.set_param(n, "in_hi", ih);
        g.set_param(n, "out_lo", ol);
        g.set_param(n, "out_hi", oh);
    }

    #[test]
    fn registers_and_resolves() {
        let mut reg = NodeRegistry::new();
        register(&mut reg).unwrap();
        assert!(reg.resolve(MANIFEST.id).is_some());
    }
}
