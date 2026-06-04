#![forbid(unsafe_code)]
//! `vector.warp` — sine-wave envelope deformation (plan §7). Unary geometry
//! node; the engine lives in [`engine`].
//!
//! `Effect::Pure` (renderer-consumed; memory
//! `project_node_effect_pure_for_renderer_consumed`). Params `amplitude` (px)
//! and `frequency` (waves across width), read directly via [`EvalCtx::param`].

mod engine;

pub use engine::warp;

use ph2d_node_registry::{NodeRegistry, RegistryError};
use ph2d_nodegraph::cook::EvalCtx;
use ph2d_nodegraph::effect::Effect;
use ph2d_nodegraph::node::{LoweringKind, NodeManifest, NodeOp, NodeTypeId, ParamSpec, PortSpec};
use ph2d_nodegraph::port::Clock;
use ph2d_vector_doc::VectorNetwork;
use ph2d_vector_graph::{VECTOR_PORT, VectorEvalExt};

/// The static contract of this node type (ADR-0031).
pub const MANIFEST: NodeManifest = NodeManifest {
    id: NodeTypeId::of("vector.warp"),
    name: "vector.warp",
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
    params: &[
        ParamSpec {
            name: "amplitude",
            default: 20.0,
        },
        ParamSpec {
            name: "frequency",
            default: 2.0,
        },
    ],
    lowerings: &[LoweringKind::Cpu],
};

struct VectorWarp;

impl NodeOp for VectorWarp {
    fn manifest(&self) -> &'static NodeManifest {
        &MANIFEST
    }

    fn eval(&self, ctx: &mut EvalCtx<'_>) {
        let amplitude = ctx.param("amplitude");
        let frequency = ctx.param("frequency");
        let empty = VectorNetwork::empty();
        let input = ctx.input_network(0).unwrap_or(&empty);
        let out = engine::warp(input, amplitude, frequency);
        ctx.emit_network(out);
    }
}

/// Register this node with the runtime registry (codegen entry point).
pub fn register(reg: &mut NodeRegistry) -> Result<(), RegistryError> {
    reg.register(Box::new(VectorWarp))
}

#[cfg(test)]
mod tests {
    use super::*;
    use ph2d_nodegraph::cook::{Cook, OpResolver};
    use ph2d_nodegraph::graph::{Edge, Graph};
    use ph2d_vector_doc::primitives;

    fn rect_src() -> VectorNetwork {
        let mut net = primitives::rect(glam::Vec2::new(0.0, -5.0), glam::Vec2::new(100.0, 5.0));
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
                t if t == MANIFEST.id => Some(&VectorWarp),
                _ => None,
            }
        }
    }

    #[test]
    fn warp_through_a_real_cook_preserves_topology() {
        let mut g = Graph::new();
        let src = g.add_node("vector.test.rect");
        let w = g.add_node("vector.warp");
        g.set_param(w, "amplitude", 15.0);
        g.set_param(w, "frequency", 2.0);
        g.connect(Edge {
            from: (src, 0),
            to: (w, 0),
            delayed: false,
        })
        .unwrap();
        let mut cook = Cook::new();
        let out = cook.cook(&g, &Ops, w, 0.0).unwrap();
        let net = out[0]
            .as_any()
            .and_then(|x| x.downcast_ref::<VectorNetwork>())
            .expect("warp output carries a VectorNetwork");
        assert!(net.validate().is_ok());
        assert_eq!(net.vertices.len(), 4);
    }
}
