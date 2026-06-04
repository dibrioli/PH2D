#![forbid(unsafe_code)]
//! `vector.width-profile` — stroke open paths with a linear width taper into
//! filled bands (plan §7, ADR-0058 §2.2.9). Unary geometry node; the engine
//! lives in [`engine`].
//!
//! `Effect::Pure` (renderer-consumed; memory
//! `project_node_effect_pure_for_renderer_consumed`). Params `width_start` and
//! `width_end` (px at each end of the path), read directly via [`EvalCtx::param`].

mod engine;

pub use engine::width_profile;

use ph2d_node_registry::{NodeRegistry, RegistryError};
use ph2d_nodegraph::cook::EvalCtx;
use ph2d_nodegraph::effect::Effect;
use ph2d_nodegraph::node::{LoweringKind, NodeManifest, NodeOp, NodeTypeId, ParamSpec, PortSpec};
use ph2d_nodegraph::port::Clock;
use ph2d_vector_doc::VectorNetwork;
use ph2d_vector_graph::{VECTOR_PORT, VectorEvalExt};

/// The static contract of this node type (ADR-0031).
pub const MANIFEST: NodeManifest = NodeManifest {
    id: NodeTypeId::of("vector.width-profile"),
    name: "vector.width-profile",
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
            name: "width_start",
            default: 1.0,
        },
        ParamSpec {
            name: "width_end",
            default: 10.0,
        },
    ],
    lowerings: &[LoweringKind::Cpu],
};

struct VectorWidthProfile;

impl NodeOp for VectorWidthProfile {
    fn manifest(&self) -> &'static NodeManifest {
        &MANIFEST
    }

    fn eval(&self, ctx: &mut EvalCtx<'_>) {
        let ws = ctx.param("width_start");
        let we = ctx.param("width_end");
        let empty = VectorNetwork::empty();
        let input = ctx.input_network(0).unwrap_or(&empty);
        let out = engine::width_profile(input, ws, we);
        ctx.emit_network(out);
    }
}

/// Register this node with the runtime registry (codegen entry point).
pub fn register(reg: &mut NodeRegistry) -> Result<(), RegistryError> {
    reg.register(Box::new(VectorWidthProfile))
}

#[cfg(test)]
mod tests {
    use super::*;
    use ph2d_nodegraph::cook::{Cook, OpResolver};
    use ph2d_nodegraph::graph::{Edge, Graph};
    use ph2d_vector_doc::{Segment, Vertex};

    fn line_src() -> VectorNetwork {
        let mut net = VectorNetwork::empty();
        net.deterministic = true;
        net.vertices
            .push(Vertex::auto(0, glam::Vec2::new(0.0, 0.0)));
        net.vertices
            .push(Vertex::auto(1, glam::Vec2::new(100.0, 0.0)));
        net.segments.push(Segment::straight(0, 0, 1));
        net
    }

    static SRC_MAN: NodeManifest = NodeManifest {
        id: NodeTypeId::of("vector.test.line"),
        name: "vector.test.line",
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
            ctx.emit_network(line_src());
        }
    }

    struct Ops;
    impl OpResolver for Ops {
        fn resolve(&self, ty: NodeTypeId) -> Option<&dyn NodeOp> {
            match ty {
                t if t == SRC_MAN.id => Some(&Src),
                t if t == MANIFEST.id => Some(&VectorWidthProfile),
                _ => None,
            }
        }
    }

    #[test]
    fn width_profile_through_a_real_cook_makes_a_band() {
        let mut g = Graph::new();
        let src = g.add_node("vector.test.line");
        let wp = g.add_node("vector.width-profile");
        g.set_param(wp, "width_start", 2.0);
        g.set_param(wp, "width_end", 20.0);
        g.connect(Edge {
            from: (src, 0),
            to: (wp, 0),
            delayed: false,
        })
        .unwrap();
        let mut cook = Cook::new();
        let out = cook.cook(&g, &Ops, wp, 0.0).unwrap();
        let net = out[0]
            .as_any()
            .and_then(|x| x.downcast_ref::<VectorNetwork>())
            .expect("width-profile output carries a VectorNetwork");
        assert!(net.validate().is_ok());
        assert_eq!(net.regions.len(), 1);
    }
}
