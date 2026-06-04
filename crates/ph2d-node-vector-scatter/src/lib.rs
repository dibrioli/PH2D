#![forbid(unsafe_code)]
//! `vector.scatter` — place N pseudo-random copies of the input in an area (plan
//! §7). Unary geometry node; the engine lives in [`engine`].
//!
//! `Effect::Pure` (renderer-consumed; memory
//! `project_node_effect_pure_for_renderer_consumed`). Params: `count`
//! (copies, capped via [`param_as_count`]), `spread` (area half-extent),
//! `seed` (PRNG → bit-identical cross-OS, never the global `rand`).

mod engine;

pub use engine::{MAX_COUNT, scatter};

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
    id: NodeTypeId::of("vector.scatter"),
    name: "vector.scatter",
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
            name: "count",
            default: 10.0,
        },
        ParamSpec {
            name: "spread",
            default: 100.0,
        },
        ParamSpec {
            name: "seed",
            default: 0.0,
        },
    ],
    lowerings: &[LoweringKind::Cpu],
};

struct VectorScatter;

impl NodeOp for VectorScatter {
    fn manifest(&self) -> &'static NodeManifest {
        &MANIFEST
    }

    fn eval(&self, ctx: &mut EvalCtx<'_>) {
        let count = param_as_count(ctx.param("count"), MAX_COUNT);
        let spread = ctx.param("spread");
        let seed = param_as_count(ctx.param("seed"), SEED_MAX) as u32;
        let empty = VectorNetwork::empty();
        let input = ctx.input_network(0).unwrap_or(&empty);
        let out = engine::scatter(input, count, spread, seed);
        ctx.emit_network(out);
    }
}

/// Register this node with the runtime registry (codegen entry point).
pub fn register(reg: &mut NodeRegistry) -> Result<(), RegistryError> {
    reg.register(Box::new(VectorScatter))
}

#[cfg(test)]
mod tests {
    use super::*;
    use ph2d_nodegraph::cook::{Cook, OpResolver};
    use ph2d_nodegraph::graph::{Edge, Graph};
    use ph2d_vector_doc::primitives;

    fn dot_src() -> VectorNetwork {
        let mut net = primitives::rect(glam::Vec2::new(-1.0, -1.0), glam::Vec2::new(1.0, 1.0));
        net.deterministic = true;
        net
    }

    static SRC_MAN: NodeManifest = NodeManifest {
        id: NodeTypeId::of("vector.test.dot"),
        name: "vector.test.dot",
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
            ctx.emit_network(dot_src());
        }
    }

    struct Ops;
    impl OpResolver for Ops {
        fn resolve(&self, ty: NodeTypeId) -> Option<&dyn NodeOp> {
            match ty {
                t if t == SRC_MAN.id => Some(&Src),
                t if t == MANIFEST.id => Some(&VectorScatter),
                _ => None,
            }
        }
    }

    #[test]
    fn scatter_through_a_real_cook_makes_copies() {
        let mut g = Graph::new();
        let src = g.add_node("vector.test.dot");
        let sc = g.add_node("vector.scatter");
        g.set_param(sc, "count", 8.0);
        g.set_param(sc, "spread", 50.0);
        g.connect(Edge {
            from: (src, 0),
            to: (sc, 0),
            delayed: false,
        })
        .unwrap();
        let mut cook = Cook::new();
        let out = cook.cook(&g, &Ops, sc, 0.0).unwrap();
        let net = out[0]
            .as_any()
            .and_then(|x| x.downcast_ref::<VectorNetwork>())
            .expect("scatter output carries a VectorNetwork");
        assert!(net.validate().is_ok());
        assert_eq!(net.regions.len(), 8);
    }
}
