#![forbid(unsafe_code)]
//! `vector.outline-stroke` — expand strokes/paths into filled outline regions
//! (plan §7, ADR-0058 §2.2.4). Unary geometry node; the engine lives in
//! [`engine`].
//!
//! `Effect::Pure` (renderer-consumed; memory
//! `project_node_effect_pure_for_renderer_consumed`). Params: `width` (stroke
//! width, read directly), `cap` (`0`=Butt/`1`=Round/`2`=Square) and `join`
//! (`0`=Miter/`1`=Round/`2`=Bevel) discriminants via [`param_as_count`].

mod engine;

pub use engine::outline_stroke;

use ph2d_node_registry::{NodeRegistry, RegistryError};
use ph2d_nodegraph::cook::EvalCtx;
use ph2d_nodegraph::effect::Effect;
use ph2d_nodegraph::node::{
    LoweringKind, NodeManifest, NodeOp, NodeTypeId, ParamSpec, PortSpec, param_as_count,
};
use ph2d_nodegraph::port::Clock;
use ph2d_vector_doc::VectorNetwork;
use ph2d_vector_graph::{VECTOR_PORT, VectorEvalExt};
use ph2d_vector_kurbo::{Cap, Join};

const CAP_MAX: usize = 2;
const JOIN_MAX: usize = 2;

/// The static contract of this node type (ADR-0031).
pub const MANIFEST: NodeManifest = NodeManifest {
    id: NodeTypeId::of("vector.outline-stroke"),
    name: "vector.outline-stroke",
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
            name: "width",
            default: 4.0,
        },
        ParamSpec {
            name: "cap",
            default: 1.0,
        },
        ParamSpec {
            name: "join",
            default: 1.0,
        },
    ],
    lowerings: &[LoweringKind::Cpu],
};

/// `0`=Butt, `1`=Round, `2`=Square; out-of-range → Round.
#[must_use]
const fn cap_from_index(i: usize) -> Cap {
    match i {
        0 => Cap::Butt,
        2 => Cap::Square,
        _ => Cap::Round,
    }
}

/// `0`=Miter, `1`=Round, `2`=Bevel; out-of-range → Round.
#[must_use]
const fn join_from_index(i: usize) -> Join {
    match i {
        0 => Join::Miter,
        2 => Join::Bevel,
        _ => Join::Round,
    }
}

struct VectorOutlineStroke;

impl NodeOp for VectorOutlineStroke {
    fn manifest(&self) -> &'static NodeManifest {
        &MANIFEST
    }

    fn eval(&self, ctx: &mut EvalCtx<'_>) {
        let width = ctx.param("width");
        let cap = cap_from_index(param_as_count(ctx.param("cap"), CAP_MAX));
        let join = join_from_index(param_as_count(ctx.param("join"), JOIN_MAX));
        let empty = VectorNetwork::empty();
        let input = ctx.input_network(0).unwrap_or(&empty);
        let out = engine::outline_stroke(input, width, cap, join);
        ctx.emit_network(out);
    }
}

/// Register this node with the runtime registry (codegen entry point).
pub fn register(reg: &mut NodeRegistry) -> Result<(), RegistryError> {
    reg.register(Box::new(VectorOutlineStroke))
}

#[cfg(test)]
mod tests {
    use super::*;
    use ph2d_nodegraph::cook::{Cook, OpResolver};
    use ph2d_nodegraph::graph::{Edge, Graph};
    use ph2d_vector_doc::primitives;

    fn square_src() -> VectorNetwork {
        let mut net = primitives::rect(glam::Vec2::new(0.0, 0.0), glam::Vec2::new(100.0, 100.0));
        net.deterministic = true;
        net
    }

    static SRC_MAN: NodeManifest = NodeManifest {
        id: NodeTypeId::of("vector.test.sq"),
        name: "vector.test.sq",
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
            ctx.emit_network(square_src());
        }
    }

    struct Ops;
    impl OpResolver for Ops {
        fn resolve(&self, ty: NodeTypeId) -> Option<&dyn NodeOp> {
            match ty {
                t if t == SRC_MAN.id => Some(&Src),
                t if t == MANIFEST.id => Some(&VectorOutlineStroke),
                _ => None,
            }
        }
    }

    #[test]
    fn cap_join_discriminants_map() {
        assert!(matches!(cap_from_index(0), Cap::Butt));
        assert!(matches!(cap_from_index(1), Cap::Round));
        assert!(matches!(cap_from_index(2), Cap::Square));
        assert!(matches!(join_from_index(0), Join::Miter));
        assert!(matches!(join_from_index(1), Join::Round));
        assert!(matches!(join_from_index(2), Join::Bevel));
    }

    #[test]
    fn outline_stroke_through_a_real_cook_makes_a_frame() {
        let mut g = Graph::new();
        let src = g.add_node("vector.test.sq");
        let os = g.add_node("vector.outline-stroke");
        g.set_param(os, "width", 10.0);
        g.connect(Edge {
            from: (src, 0),
            to: (os, 0),
            delayed: false,
        })
        .unwrap();
        let mut cook = Cook::new();
        let out = cook.cook(&g, &Ops, os, 0.0).unwrap();
        let net = out[0]
            .as_any()
            .and_then(|x| x.downcast_ref::<VectorNetwork>())
            .expect("outline-stroke output carries a VectorNetwork");
        assert!(net.validate().is_ok());
        assert!(net.regions.len() >= 2);
    }
}
