#![forbid(unsafe_code)]
//! `vector.roughen` — subdivide the input's edges and jitter them along the
//! normal by deterministic seeded noise (plan §7). Unary geometry node; the
//! engine lives in [`engine`].
//!
//! `Effect::Pure` (renderer-consumed; memory
//! `project_node_effect_pure_for_renderer_consumed`). Params: `amplitude`
//! (displacement, read directly), `detail` (subdivisions, capped via
//! [`param_as_count`]), `seed` (PRNG seed → bit-identical cross-OS, never the
//! global `rand`).

mod engine;

pub use engine::{MAX_DETAIL, roughen};

use ph2d_node_registry::{NodeRegistry, RegistryError};
use ph2d_nodegraph::cook::EvalCtx;
use ph2d_nodegraph::effect::Effect;
use ph2d_nodegraph::node::{
    LoweringKind, NodeManifest, NodeOp, NodeTypeId, ParamSpec, PortSpec, param_as_count,
};
use ph2d_nodegraph::port::Clock;
use ph2d_vector_doc::VectorNetwork;
use ph2d_vector_graph::{VECTOR_PORT, VectorEvalExt};

/// Allocation ceiling for the `seed` conversion (any non-negative `u32`).
const SEED_MAX: usize = u32::MAX as usize;

/// The static contract of this node type (ADR-0031).
pub const MANIFEST: NodeManifest = NodeManifest {
    id: NodeTypeId::of("vector.roughen"),
    name: "vector.roughen",
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
            default: 5.0,
        },
        ParamSpec {
            name: "detail",
            default: 8.0,
        },
        ParamSpec {
            name: "seed",
            default: 0.0,
        },
    ],
    lowerings: &[LoweringKind::Cpu],
};

struct VectorRoughen;

impl NodeOp for VectorRoughen {
    fn manifest(&self) -> &'static NodeManifest {
        &MANIFEST
    }

    fn eval(&self, ctx: &mut EvalCtx<'_>) {
        let amplitude = ctx.param("amplitude");
        let detail = param_as_count(ctx.param("detail"), MAX_DETAIL);
        let seed = param_as_count(ctx.param("seed"), SEED_MAX) as u32;
        let empty = VectorNetwork::empty();
        let input = ctx.input_network(0).unwrap_or(&empty);
        let out = engine::roughen(input, amplitude, detail, seed);
        ctx.emit_network(out);
    }
}

/// Register this node with the runtime registry (codegen entry point).
pub fn register(reg: &mut NodeRegistry) -> Result<(), RegistryError> {
    reg.register(Box::new(VectorRoughen))
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
                t if t == MANIFEST.id => Some(&VectorRoughen),
                _ => None,
            }
        }
    }

    #[test]
    fn roughen_through_a_real_cook_subdivides() {
        let mut g = Graph::new();
        let src = g.add_node("vector.test.sq");
        let rgh = g.add_node("vector.roughen");
        g.set_param(rgh, "detail", 8.0);
        g.set_param(rgh, "amplitude", 5.0);
        g.connect(Edge {
            from: (src, 0),
            to: (rgh, 0),
            delayed: false,
        })
        .unwrap();
        let mut cook = Cook::new();
        let out = cook.cook(&g, &Ops, rgh, 0.0).unwrap();
        let net = out[0]
            .as_any()
            .and_then(|x| x.downcast_ref::<VectorNetwork>())
            .expect("roughen output carries a VectorNetwork");
        assert!(net.validate().is_ok());
        assert_eq!(net.segments.len(), 32, "4 edges × 8 spans");
    }
}
