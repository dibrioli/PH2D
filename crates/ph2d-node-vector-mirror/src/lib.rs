#![forbid(unsafe_code)]
//! `vector.mirror` — reflect the input geometry across the X and/or Y axis and
//! combine the copies (plan §7, ADR-0058 geometry fan-out). Unary geometry node;
//! the engine lives in [`engine`].
//!
//! `Effect::Pure` (renderer-consumed nodes must be — a `Stateful` node is never
//! driven by the presentation `Cook`; memory
//! `project_node_effect_pure_for_renderer_consumed`). Param `axis` is an `f32`
//! discriminant (`0`=X, `1`=Y, `2`=Both), read via [`param_as_count`].

mod engine;

pub use engine::{MirrorAxis, mirror};

use ph2d_node_registry::{NodeRegistry, RegistryError};
use ph2d_nodegraph::cook::EvalCtx;
use ph2d_nodegraph::effect::Effect;
use ph2d_nodegraph::node::{
    LoweringKind, NodeManifest, NodeOp, NodeTypeId, ParamSpec, PortSpec, param_as_count,
};
use ph2d_nodegraph::port::Clock;
use ph2d_vector_doc::VectorNetwork;
use ph2d_vector_graph::{VECTOR_PORT, VectorEvalExt};

/// Highest valid `axis` discriminant (`Both`); [`param_as_count`] clamps to it.
const AXIS_MAX: usize = 2;

/// The static contract of this node type (ADR-0031).
pub const MANIFEST: NodeManifest = NodeManifest {
    id: NodeTypeId::of("vector.mirror"),
    name: "vector.mirror",
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
        name: "axis",
        default: 2.0,
    }],
    lowerings: &[LoweringKind::Cpu],
};

/// Map the `axis` discriminant index to a [`MirrorAxis`]; out-of-range → `Both`.
#[must_use]
const fn axis_from_index(i: usize) -> MirrorAxis {
    match i {
        0 => MirrorAxis::X,
        1 => MirrorAxis::Y,
        _ => MirrorAxis::Both,
    }
}

struct VectorMirror;

impl NodeOp for VectorMirror {
    fn manifest(&self) -> &'static NodeManifest {
        &MANIFEST
    }

    fn eval(&self, ctx: &mut EvalCtx<'_>) {
        let axis = axis_from_index(param_as_count(ctx.param("axis"), AXIS_MAX));
        let empty = VectorNetwork::empty();
        let input = ctx.input_network(0).unwrap_or(&empty);
        let out = engine::mirror(input, axis);
        ctx.emit_network(out);
    }
}

/// Register this node with the runtime registry (codegen entry point).
pub fn register(reg: &mut NodeRegistry) -> Result<(), RegistryError> {
    reg.register(Box::new(VectorMirror))
}

#[cfg(test)]
mod tests {
    use super::*;
    use ph2d_nodegraph::cook::{Cook, OpResolver};
    use ph2d_nodegraph::graph::{Edge, Graph};
    use ph2d_vector_doc::primitives;

    fn quad_src() -> VectorNetwork {
        let mut net = primitives::rect(glam::Vec2::new(1.0, 1.0), glam::Vec2::new(3.0, 3.0));
        net.deterministic = true;
        net
    }

    static SRC_MAN: NodeManifest = NodeManifest {
        id: NodeTypeId::of("vector.test.quad"),
        name: "vector.test.quad",
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
            ctx.emit_network(quad_src());
        }
    }

    struct Ops;
    impl OpResolver for Ops {
        fn resolve(&self, ty: NodeTypeId) -> Option<&dyn NodeOp> {
            match ty {
                t if t == SRC_MAN.id => Some(&Src),
                t if t == MANIFEST.id => Some(&VectorMirror),
                _ => None,
            }
        }
    }

    #[test]
    fn axis_discriminant_maps_every_index() {
        assert!(matches!(axis_from_index(0), MirrorAxis::X));
        assert!(matches!(axis_from_index(1), MirrorAxis::Y));
        assert!(matches!(axis_from_index(2), MirrorAxis::Both));
        assert!(matches!(axis_from_index(99), MirrorAxis::Both));
    }

    #[test]
    fn mirror_through_a_real_cook_makes_four_copies() {
        let mut g = Graph::new();
        let src = g.add_node("vector.test.quad");
        let mir = g.add_node("vector.mirror");
        g.connect(Edge {
            from: (src, 0),
            to: (mir, 0),
            delayed: false,
        })
        .unwrap();
        let mut cook = Cook::new();
        let out = cook.cook(&g, &Ops, mir, 0.0).unwrap();
        let net = out[0]
            .as_any()
            .and_then(|x| x.downcast_ref::<VectorNetwork>())
            .expect("mirror output carries a VectorNetwork");
        assert!(net.validate().is_ok());
        assert_eq!(net.regions.len(), 4);
    }
}
