#![forbid(unsafe_code)]
//! `vector.recolor` — reassign region fill refs, geometry intact (plan §7).
//! Unary geometry node; the engine lives in [`engine`].
//!
//! `Effect::Pure` (renderer-consumed; memory
//! `project_node_effect_pure_for_renderer_consumed`). Param `fill` is the target
//! fill-ref index (the colour it resolves to is asset-side — see [`engine`]),
//! read via [`param_as_count`].

mod engine;

pub use engine::recolor;

use ph2d_node_registry::{NodeRegistry, RegistryError};
use ph2d_nodegraph::cook::EvalCtx;
use ph2d_nodegraph::effect::Effect;
use ph2d_nodegraph::node::{
    LoweringKind, NodeManifest, NodeOp, NodeTypeId, ParamSpec, PortSpec, param_as_count,
};
use ph2d_nodegraph::port::Clock;
use ph2d_vector_doc::VectorNetwork;
use ph2d_vector_graph::{VECTOR_PORT, VectorEvalExt};

/// Allocation ceiling for the `fill` ref conversion (any non-negative `u32`).
const FILL_MAX: usize = u32::MAX as usize;

/// The static contract of this node type (ADR-0031).
pub const MANIFEST: NodeManifest = NodeManifest {
    id: NodeTypeId::of("vector.recolor"),
    name: "vector.recolor",
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
        name: "fill",
        default: 0.0,
    }],
    lowerings: &[LoweringKind::Cpu],
};

struct VectorRecolor;

impl NodeOp for VectorRecolor {
    fn manifest(&self) -> &'static NodeManifest {
        &MANIFEST
    }

    fn eval(&self, ctx: &mut EvalCtx<'_>) {
        let fill_ref = param_as_count(ctx.param("fill"), FILL_MAX) as u32;
        let empty = VectorNetwork::empty();
        let input = ctx.input_network(0).unwrap_or(&empty);
        let out = engine::recolor(input, fill_ref);
        ctx.emit_network(out);
    }
}

/// Register this node with the runtime registry (codegen entry point).
pub fn register(reg: &mut NodeRegistry) -> Result<(), RegistryError> {
    reg.register(Box::new(VectorRecolor))
}

#[cfg(test)]
mod tests {
    use super::*;
    use ph2d_nodegraph::cook::{Cook, OpResolver};
    use ph2d_nodegraph::graph::{Edge, Graph};
    use ph2d_vector_doc::primitives;

    fn rect_src() -> VectorNetwork {
        let mut net = primitives::rect(glam::Vec2::new(0.0, 0.0), glam::Vec2::new(10.0, 10.0));
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
                t if t == MANIFEST.id => Some(&VectorRecolor),
                _ => None,
            }
        }
    }

    #[test]
    fn recolor_through_a_real_cook_sets_fill() {
        let mut g = Graph::new();
        let src = g.add_node("vector.test.rect");
        let rc = g.add_node("vector.recolor");
        g.set_param(rc, "fill", 5.0);
        g.connect(Edge {
            from: (src, 0),
            to: (rc, 0),
            delayed: false,
        })
        .unwrap();
        let mut cook = Cook::new();
        let out = cook.cook(&g, &Ops, rc, 0.0).unwrap();
        let net = out[0]
            .as_any()
            .and_then(|x| x.downcast_ref::<VectorNetwork>())
            .expect("recolor output carries a VectorNetwork");
        assert!(net.validate().is_ok());
        assert!(net.regions.iter().all(|r| r.fill == Some(5)));
    }
}
