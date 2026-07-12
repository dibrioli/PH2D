#![forbid(unsafe_code)]
//! `motion.luminance` — **read colour back into a value**: the adapter that turns the
//! `tint` column into a per-instance brightness scalar (Motion Nodes M1, adapters —
//! doc 01 §1.7 / doc 31). The inverse of the colour nodes — where `motion.color_ramp`
//! maps a value TO a colour, this maps a colour back TO a value, closing the loop so an
//! instance's appearance can drive its size / position / anything downstream.
//!
//! **Algorithm — the Rec. 709 luma of the linear-RGB tint.** `v = 0.2126·R + 0.7152·G +
//! 0.0722·B` per element (the standard perceptual weights; the `tint` column and these
//! weights are both linear RGB, so this is relative luminance). Reads the input stream's
//! `tint` and emits a **value field** — a `VALUE`-typed output (like `value.instance_
//! field`), so it plugs straight into any value input (a colour-ramp `t`, a math node).
//! An absent `tint` reads as black (`v = 0`). Transcendental-free (HR-5): a weighted sum.
//! `Effect::Pure`.

use ph2d_node_registry::{NodeRegistry, RegistryError};
use ph2d_nodegraph::attr::{Column, Stream};
use ph2d_nodegraph::cook::EvalCtx;
use ph2d_nodegraph::effect::Effect;
use ph2d_nodegraph::node::{LoweringKind, NodeManifest, NodeOp, NodeTypeId, PortSpec};
use ph2d_nodegraph::port::{Clock, Dim, Domain, PortType};

const INST_VEC2: PortType = PortType::new(Domain::Instances, Dim::Vec2, Clock::Frame);
/// The value-field output type (mirror of `value.instance_field`).
const VALUE: PortType = PortType::new(Domain::Instances, Dim::Scalar, Clock::Frame);
const VALUE_COL: &str = "v";

/// Rec. 709 luma weights (linear RGB → relative luminance).
const WR: f32 = 0.2126;
const WG: f32 = 0.7152;
const WB: f32 = 0.0722;

/// The static contract of this node type (ADR-0031).
pub const MANIFEST: NodeManifest = NodeManifest {
    id: NodeTypeId::of("motion.luminance"),
    name: "motion.luminance",
    inputs: &[PortSpec {
        name: "in",
        ty: INST_VEC2,
    }],
    outputs: &[PortSpec {
        name: "out",
        ty: VALUE,
    }],
    effect: Effect::Pure,
    clock: Clock::Frame,
    params: &[],
    lowerings: &[LoweringKind::Cpu],
};

/// The Rec. 709 luma of each tint (absent tint → 0).
fn luminance(tint: &[[f32; 4]], n: usize) -> Vec<f32> {
    (0..n)
        .map(|i| {
            let c = tint.get(i).copied().unwrap_or([0.0; 4]);
            WR * c[0] + WG * c[1] + WB * c[2]
        })
        .collect()
}

struct MotionLuminance;

impl NodeOp for MotionLuminance {
    fn manifest(&self) -> &'static NodeManifest {
        &MANIFEST
    }

    fn eval(&self, ctx: &mut EvalCtx<'_>) {
        let input = ctx.input(0);
        let n = input.count();
        let tint: Vec<[f32; 4]> = match input.get("tint") {
            Some(Column::Vec4(v)) => v.clone(),
            _ => Vec::new(),
        };
        let v = luminance(&tint, n);
        // A pure value-field output (like `value.instance_field`): just the `v` column.
        ctx.emit(Stream::new(n).with(VALUE_COL, Column::Scalar(v)));
    }
}

/// Register this node with the runtime registry. Called (via codegen) from
/// `ph2d-node-registry-init::register_all_nodes`.
pub fn register(reg: &mut NodeRegistry) -> Result<(), RegistryError> {
    reg.register(Box::new(MotionLuminance))?;
    reg.register_ui(
        MANIFEST.id,
        ph2d_node_registry::NodeUiManifest {
            display_name: "Luminance",
            category: ph2d_node_registry::NodeUiCategory::Utility,
            silhouette: ph2d_node_registry::NodeSilhouette::Rect,
        },
    );
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    /// White is full luminance (the weights sum to 1), black is zero, mid-grey is ~0.5.
    #[test]
    fn white_is_one_black_is_zero() {
        let v = luminance(
            &[
                [1.0, 1.0, 1.0, 1.0],
                [0.0, 0.0, 0.0, 1.0],
                [0.5, 0.5, 0.5, 1.0],
            ],
            3,
        );
        assert!((v[0] - 1.0).abs() < 1e-5, "white -> 1: {}", v[0]);
        assert!(v[1].abs() < 1e-5, "black -> 0: {}", v[1]);
        assert!((v[2] - 0.5).abs() < 1e-5, "grey -> 0.5: {}", v[2]);
    }

    /// Green reads brighter than red reads brighter than blue (the Rec. 709 ordering).
    /// FALSIFIED by a flat average (all three equal).
    #[test]
    fn green_is_brightest_blue_is_dimmest() {
        let v = luminance(
            &[
                [1.0, 0.0, 0.0, 1.0],
                [0.0, 1.0, 0.0, 1.0],
                [0.0, 0.0, 1.0, 1.0],
            ],
            3,
        );
        assert!(v[1] > v[0] && v[0] > v[2], "G > R > B: {v:?}");
    }

    /// An absent tint reads as black (0), not a panic.
    #[test]
    fn absent_tint_is_zero() {
        assert_eq!(luminance(&[], 3), vec![0.0, 0.0, 0.0]);
    }

    /// Deterministic + cooks through the registry: writes `v` from `tint` and passes the
    /// geometry through.
    #[test]
    fn registers_and_reads_luma_through_the_cook() {
        use ph2d_nodegraph::cook::{Cook, OpResolver};
        use ph2d_nodegraph::graph::{Edge, Graph};

        static SRC: NodeManifest = NodeManifest {
            id: NodeTypeId::of("motion.luminance.test.src"),
            name: "motion.luminance.test.src",
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
        struct Src;
        impl NodeOp for Src {
            fn manifest(&self) -> &'static NodeManifest {
                &SRC
            }
            fn eval(&self, ctx: &mut EvalCtx<'_>) {
                ctx.emit(
                    Stream::new(2)
                        .with("P", Column::Vec2(vec![[0.0, 0.0], [1.0, 0.0]]))
                        .with(
                            "tint",
                            Column::Vec4(vec![[1.0, 1.0, 1.0, 1.0], [0.0, 0.0, 0.0, 1.0]]),
                        ),
                );
            }
        }
        struct Ops;
        impl OpResolver for Ops {
            fn resolve(&self, ty: NodeTypeId) -> Option<&dyn NodeOp> {
                match ty {
                    t if t == SRC.id => Some(&Src),
                    t if t == MANIFEST.id => Some(&MotionLuminance),
                    _ => None,
                }
            }
        }
        let mut reg = NodeRegistry::new();
        register(&mut reg).unwrap();
        assert!(reg.resolve(MANIFEST.id).is_some());

        let mut g = Graph::new();
        let src = g.add_node("motion.luminance.test.src");
        let lum = g.add_node("motion.luminance");
        g.connect(Edge {
            from: (src, 0),
            to: (lum, 0),
            delayed: false,
        })
        .unwrap();
        let mut cook = Cook::new();
        let out = cook.cook(&g, &Ops, lum, 0.0).unwrap();
        let s = out[0].as_stream();
        // A pure value field: the `v` column, not the geometry.
        match s.get("v").unwrap() {
            Column::Scalar(v) => {
                assert_eq!(v.len(), 2, "one luma per instance");
                assert!(
                    (v[0] - 1.0).abs() < 1e-5 && v[1].abs() < 1e-5,
                    "white->1, black->0"
                );
            }
            _ => panic!("v"),
        }
    }
}
