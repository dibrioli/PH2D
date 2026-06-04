#![forbid(unsafe_code)]
//! `vector.bend-path` — bend geometry along a circular arc by x-position (plan
//! §7). Unary geometry node; the engine lives in [`engine`].
//!
//! `Effect::Pure` (renderer-consumed; memory
//! `project_node_effect_pure_for_renderer_consumed`). Param `angle` is the arc
//! subtended over the shape width (degrees), read directly via [`EvalCtx::param`].

mod engine;

pub use engine::bend;

use ph2d_node_registry::{NodeRegistry, RegistryError};
use ph2d_nodegraph::cook::EvalCtx;
use ph2d_nodegraph::effect::Effect;
use ph2d_nodegraph::node::{LoweringKind, NodeManifest, NodeOp, NodeTypeId, ParamSpec, PortSpec};
use ph2d_nodegraph::port::Clock;
use ph2d_vector_doc::VectorNetwork;
use ph2d_vector_graph::{VECTOR_PORT, VectorEvalExt};

/// The static contract of this node type (ADR-0031).
pub const MANIFEST: NodeManifest = NodeManifest {
    id: NodeTypeId::of("vector.bend-path"),
    name: "vector.bend-path",
    inputs: &[PortSpec {
        name: "input",
        ty: VECTOR_PORT,
    }],
    outputs: &[PortSpec {
        name: "out",
        ty: VECTOR_PORT,
    }],
    effect: Effect::Pure,
    clock: Clock::Static,
    params: &[ParamSpec {
        name: "angle",
        default: 45.0,
    }],
    lowerings: &[LoweringKind::Cpu],
};

struct VectorBendPath;

impl NodeOp for VectorBendPath {
    fn manifest(&self) -> &'static NodeManifest {
        &MANIFEST
    }

    fn eval(&self, ctx: &mut EvalCtx<'_>) {
        let angle = ctx.param("angle");
        let empty = VectorNetwork::empty();
        let input = ctx.input_network(0).unwrap_or(&empty);
        let out = engine::bend(input, angle);
        ctx.emit_network(out);
    }
}

/// Register this node with the runtime registry (codegen entry point).
pub fn register(reg: &mut NodeRegistry) -> Result<(), RegistryError> {
    reg.register(Box::new(VectorBendPath))
}

#[cfg(test)]
mod tests {
    use super::*;
    use ph2d_nodegraph::cook::{Cook, OpResolver};
    use ph2d_nodegraph::graph::{Edge, Graph};
    use ph2d_vector_doc::primitives;

    fn rect_src() -> VectorNetwork {
        let mut net =
            primitives::rect(glam::Vec2::new(-100.0, -10.0), glam::Vec2::new(100.0, 10.0));
        net.deterministic = true;
        net
    }

    static SRC_MAN: NodeManifest = NodeManifest {
        id: NodeTypeId::of("vector.test.rect"),
        name: "vector.test.rect",
        inputs: &[],
        outputs: &[PortSpec {
            name: "out",
            ty: VECTOR_PORT,
        }],
        effect: Effect::Pure,
        clock: Clock::Static,
        params: &[],
        lowerings: &[LoweringKind::Cpu],
    };
    struct Src;
    impl NodeOp for Src {
        fn manifest(&self) -> &'static NodeManifest {
            &SRC_MAN
        }
        fn eval(&self, ctx: &mut EvalCtx<'_>) {
            ctx.emit_network(rect_src());
        }
    }

    struct Ops;
    impl OpResolver for Ops {
        fn resolve(&self, ty: NodeTypeId) -> Option<&dyn NodeOp> {
            match ty {
                t if t == SRC_MAN.id => Some(&Src),
                t if t == MANIFEST.id => Some(&VectorBendPath),
                _ => None,
            }
        }
    }

    #[test]
    fn bend_through_a_real_cook_preserves_topology() {
        let mut g = Graph::new();
        let src = g.add_node("vector.test.rect");
        let bp = g.add_node("vector.bend-path");
        g.set_param(bp, "angle", 60.0);
        g.connect(Edge {
            from: (src, 0),
            to: (bp, 0),
            delayed: false,
        })
        .unwrap();
        let mut cook = Cook::new();
        let out = cook.cook(&g, &Ops, bp, 0.0).unwrap();
        let net = out[0]
            .as_any()
            .and_then(|x| x.downcast_ref::<VectorNetwork>())
            .expect("bend output carries a VectorNetwork");
        assert!(net.validate().is_ok());
        assert_eq!(net.vertices.len(), 4);
    }
}
