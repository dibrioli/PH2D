#![forbid(unsafe_code)]
//! `vector.offset` — the parallel/contour **offset** node (ADR-0058 §2.2.3, plan
//! T3.4). One geometry input and three params (`offset` distance, `join` style,
//! `miter_limit`) produce the inset/outset of the input's filled regions. The
//! engine lives in [`engine`]; this module is the node contract + registration.
//!
//! ## Carrier + Effect
//!
//! Input/output are `VectorNetwork`s on the opaque channel
//! ([`ph2d_vector_graph::VectorEvalExt`]). `Effect::Pure` for the same reason as
//! `vector.boolean`: an offset is a referentially-transparent pull-side function
//! and must stay visible to the presentation `Cook` (a `Stateful` node is never
//! cooked by the renderer — memory `project_node_effect_pure_for_renderer_consumed`).
//!
//! ## Params
//!
//! - `offset` — distance in network-local px (negative = inset). No discriminant.
//! - `join` — corner join, an `f32` discriminant matching the spec order
//!   (`0`=Round, `1`=Bevel, `2`=Miter), read via [`param_as_count`].
//! - `miter_limit` — miter cutoff (clamped to ≥ 1 by the engine).

mod engine;

pub use engine::offset;

use ph2d_node_registry::{NodeRegistry, RegistryError};
use ph2d_nodegraph::cook::EvalCtx;
use ph2d_nodegraph::effect::Effect;
use ph2d_nodegraph::node::{
    LoweringKind, NodeManifest, NodeOp, NodeTypeId, ParamSpec, PortSpec, param_as_count,
};
use ph2d_nodegraph::port::Clock;
use ph2d_vector_doc::VectorNetwork;
use ph2d_vector_graph::{VECTOR_PORT, VectorEvalExt};
use ph2d_vector_kurbo::Join;

/// Highest valid `join` discriminant (`Miter`); [`param_as_count`] clamps to it.
const JOIN_MAX: usize = 2;

/// The static contract of this node type (ADR-0031).
pub const MANIFEST: NodeManifest = NodeManifest {
    id: NodeTypeId::of("vector.offset"),
    name: "vector.offset",
    inputs: &[PortSpec {
        name: "input",
        ty: VECTOR_PORT,
    }],
    outputs: &[PortSpec {
        name: "out",
        ty: VECTOR_PORT,
    }],
    // Pure (pull side) — see the module note + `vector.boolean`.
    effect: Effect::Pure,
    clock: Clock::Static,
    params: &[
        ParamSpec {
            name: "offset",
            default: 10.0,
        },
        ParamSpec {
            name: "join",
            default: 0.0,
        },
        ParamSpec {
            name: "miter_limit",
            default: 4.0,
        },
    ],
    lowerings: &[LoweringKind::Cpu],
};

/// Map the `join` discriminant index to a kurbo [`Join`]. Mirrors the spec
/// order; `0` and any out-of-range index degrade to `Round`.
#[must_use]
const fn join_from_index(i: usize) -> Join {
    match i {
        1 => Join::Bevel,
        2 => Join::Miter,
        // 0 = Round, and any stray discriminant.
        _ => Join::Round,
    }
}

struct VectorOffset;

impl NodeOp for VectorOffset {
    fn manifest(&self) -> &'static NodeManifest {
        &MANIFEST
    }

    fn eval(&self, ctx: &mut EvalCtx<'_>) {
        let distance = ctx.param("offset");
        let join = join_from_index(param_as_count(ctx.param("join"), JOIN_MAX));
        let miter_limit = ctx.param("miter_limit");
        // An unconnected input is the empty set. The immutable input borrow ends
        // before the `emit_network` mutable borrow (NLL).
        let empty = VectorNetwork::empty();
        let input = ctx.input_network(0).unwrap_or(&empty);
        let out = engine::offset(input, distance, join, miter_limit);
        ctx.emit_network(out);
    }
}

/// Register this node with the runtime registry. Called (via codegen) from
/// `ph2d-node-registry-init::register_all_nodes`.
pub fn register(reg: &mut NodeRegistry) -> Result<(), RegistryError> {
    reg.register(Box::new(VectorOffset))
}

#[cfg(test)]
mod tests {
    use super::*;
    use ph2d_nodegraph::cook::{Cook, OpResolver};
    use ph2d_nodegraph::graph::{Edge, Graph};
    use ph2d_vector_doc::primitives;

    fn square_src() -> VectorNetwork {
        let mut net = primitives::rect(glam::Vec2::new(0.0, 0.0), glam::Vec2::new(2.0, 2.0));
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
                t if t == MANIFEST.id => Some(&VectorOffset),
                _ => None,
            }
        }
    }

    #[test]
    fn join_discriminant_maps_every_index() {
        assert!(matches!(join_from_index(0), Join::Round));
        assert!(matches!(join_from_index(1), Join::Bevel));
        assert!(matches!(join_from_index(2), Join::Miter));
        assert!(matches!(join_from_index(99), Join::Round));
    }

    #[test]
    fn offset_through_a_real_cook_grows_the_square() {
        // sq(0..2) → offset(+1) → one region bounded by [-1,3]², proving the
        // cook path (emit_network → opaque → input_network → offset) connects.
        let mut g = Graph::new();
        let src = g.add_node("vector.test.sq");
        let off = g.add_node("vector.offset");
        g.set_param(off, "offset", 1.0);
        g.connect(Edge {
            from: (src, 0),
            to: (off, 0),
            delayed: false,
        })
        .unwrap();
        let mut cook = Cook::new();
        let out = cook.cook(&g, &Ops, off, 0.0).unwrap();
        let net = out[0]
            .as_any()
            .and_then(|x| x.downcast_ref::<VectorNetwork>())
            .expect("offset output carries a VectorNetwork");
        assert!(net.validate().is_ok());
        assert_eq!(net.regions.len(), 1);
        assert!(net.deterministic);
    }
}
