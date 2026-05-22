#![forbid(unsafe_code)]
//! `motion.transform` — a Motion **modifier**: scales then offsets the `P`
//! (Vec2) attribute of its input instance stream; passes every other column
//! through unchanged (count is preserved). Pure.
//!
//! Params (manifest defaults): `scale` (1.0), `offset_x` (0.0), `offset_y` (0.0).
//! `P' = P * scale + (offset_x, offset_y)`.

use ph2d_node_registry::{NodeRegistry, RegistryError};
use ph2d_nodegraph::attr::{Column, Stream};
use ph2d_nodegraph::cook::EvalCtx;
use ph2d_nodegraph::effect::Effect;
use ph2d_nodegraph::node::{LoweringKind, NodeManifest, NodeOp, NodeTypeId, ParamSpec, PortSpec};
use ph2d_nodegraph::port::{Clock, Dim, Domain, PortType};

const INST_VEC2: PortType = PortType::new(Domain::Instances, Dim::Vec2, Clock::Frame);

/// The static contract of this node type (ADR-0031).
pub const MANIFEST: NodeManifest = NodeManifest {
    id: NodeTypeId::of("motion.transform"),
    name: "motion.transform",
    inputs: &[PortSpec {
        name: "in",
        ty: INST_VEC2,
    }],
    outputs: &[PortSpec {
        name: "out",
        ty: INST_VEC2,
    }],
    effect: Effect::Pure,
    clock: Clock::Frame,
    params: &[
        ParamSpec {
            name: "scale",
            default: 1.0,
        },
        ParamSpec {
            name: "offset_x",
            default: 0.0,
        },
        ParamSpec {
            name: "offset_y",
            default: 0.0,
        },
    ],
    // CPU-only by design (see handoff §9): `P` is a `Vec2` column and
    // `ph2d-expr` is scalar-valued, so this Vec2 affine map is not an
    // `eval_column` lowering today, and no Instances-domain WGSL runtime exists.
    // Declaring `Wgsl` would claim parity nothing can run.
    lowerings: &[LoweringKind::Cpu],
};

/// A transform param read; `.expect` documents that the name is a literal of
/// this crate's own `MANIFEST` (a typo fails the golden test, never silent
/// `0.0`).
fn param(name: &str) -> f32 {
    MANIFEST
        .param_default(name)
        .expect("transform reads only its own declared params")
}

/// The per-element affine map `p' = p * scale + (ox, oy)`. Pure and isolated so
/// the arithmetic is unit-tested with non-identity values (the cook only ever
/// feeds the manifest defaults — identity — until per-instance overrides land).
fn apply_xform(p: [f32; 2], scale: f32, ox: f32, oy: f32) -> [f32; 2] {
    [p[0] * scale + ox, p[1] * scale + oy]
}

struct MotionTransform;

impl NodeOp for MotionTransform {
    fn manifest(&self) -> &'static NodeManifest {
        &MANIFEST
    }

    fn eval(&self, ctx: &mut EvalCtx<'_>) {
        let scale = param("scale");
        let (ox, oy) = (param("offset_x"), param("offset_y"));
        let out = {
            let input = ctx.input(0);
            // The port type guarantees `P` is `Vec2`; a `P` of any other dim is
            // an upstream node-author bug. Assert it loudly in debug/test rather
            // than silently passing it through untransformed (which would emit
            // geometry the params claim to have moved).
            debug_assert!(
                !matches!(input.get("P"), Some(c) if !matches!(c, Column::Vec2(_))),
                "motion.transform expects `P` to be a Vec2 column (port type guarantees it)"
            );
            let mut out = Stream::new(input.count());
            for (name, col) in input.columns() {
                match (name.as_str(), col) {
                    ("P", Column::Vec2(v)) => {
                        let t: Vec<[f32; 2]> =
                            v.iter().map(|p| apply_xform(*p, scale, ox, oy)).collect();
                        out.set("P", Column::Vec2(t));
                    }
                    // Every other column is per-element data this node does not
                    // touch — passed through unchanged (count is preserved).
                    _ => out.set(name.clone(), col.clone()),
                }
            }
            out
        };
        ctx.emit(out);
    }
}

/// Register this node with the runtime registry. Called (via codegen) from
/// `ph2d-node-registry-init::register_all_nodes`.
pub fn register(reg: &mut NodeRegistry) -> Result<(), RegistryError> {
    reg.register(Box::new(MotionTransform))
}

#[cfg(test)]
mod tests {
    use super::*;
    use ph2d_nodegraph::cook::{Cook, EvalCtx, OpResolver};
    use ph2d_nodegraph::graph::{Edge, Graph};

    // Source: 2 instances at (1,1),(2,2) with a size column to verify passthrough.
    static SRC_MAN: NodeManifest = NodeManifest {
        id: NodeTypeId::of("motion.transform.test.src"),
        name: "motion.transform.test.src",
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
            &SRC_MAN
        }
        fn eval(&self, ctx: &mut EvalCtx<'_>) {
            ctx.emit(
                Stream::new(2)
                    .with("P", Column::Vec2(vec![[1.0, 1.0], [2.0, 2.0]]))
                    .with("size", Column::Vec2(vec![[5.0, 5.0], [5.0, 5.0]])),
            );
        }
    }
    struct Ops;
    impl OpResolver for Ops {
        fn resolve(&self, ty: NodeTypeId) -> Option<&dyn NodeOp> {
            match ty {
                t if t == SRC_MAN.id => Some(&Src),
                t if t == MANIFEST.id => Some(&MotionTransform),
                _ => None,
            }
        }
    }

    #[test]
    fn scales_and_offsets_p_passes_size_through() {
        // default scale=1, offset=0 → P unchanged; assert passthrough + identity.
        let mut g = Graph::new();
        let src = g.add_node("motion.transform.test.src");
        let xf = g.add_node("motion.transform");
        g.connect(Edge {
            from: (src, 0),
            to: (xf, 0),
            delayed: false,
        })
        .unwrap();
        let mut cook = Cook::new();
        let out = cook.cook(&g, &Ops, xf, 0.0).unwrap();
        assert_eq!(out[0].count(), 2);
        match out[0].get("P").unwrap() {
            Column::Vec2(v) => assert_eq!(v, &vec![[1.0, 1.0], [2.0, 2.0]]),
            _ => panic!("P"),
        }
        // size carried through unchanged
        match out[0].get("size").unwrap() {
            Column::Vec2(v) => assert_eq!(v, &vec![[5.0, 5.0], [5.0, 5.0]]),
            _ => panic!("size"),
        }
    }

    #[test]
    fn apply_xform_scales_then_offsets() {
        // The cook can only feed identity (manifest default scale=1, offset=0),
        // so the actual `p*scale + offset` arithmetic is proven here directly.
        assert_eq!(apply_xform([2.0, 3.0], 2.0, 1.0, -1.0), [5.0, 5.0]);
        assert_eq!(apply_xform([0.0, 0.0], 10.0, 4.0, 7.0), [4.0, 7.0]);
        assert_eq!(apply_xform([1.0, 1.0], 0.0, 0.0, 0.0), [0.0, 0.0]); // collapse
        assert_eq!(apply_xform([-2.0, 5.0], 1.0, 0.0, 0.0), [-2.0, 5.0]); // identity
    }
}
