#![forbid(unsafe_code)]
//! `vector.boolean` — the geometry **boolean** node (ADR-0058 §2.2.2, plan
//! T3.3). Two geometry inputs `a`/`b` and one frozen `op` discriminant produce
//! one geometry output: the exact boolean result of the two filled regions,
//! computed by the Linesweeper sweep-line (the reconcile half of the
//! draft+reconcile pipeline, ADR-0059 §2.4 + ADR-0065). The engine lives in
//! [`engine`]; this module is the node contract + registration.
//!
//! ## Carrier (ADR-0058-amendment-1)
//!
//! Both inputs and the output are `VectorNetwork`s on the substrate's opaque
//! channel, read/emitted through [`ph2d_vector_graph::VectorEvalExt`]. The node
//! author never touches `Arc<dyn Any>`.
//!
//! ## Effect — `Pure`, not `Stateful`
//!
//! The spec pseudocode (`02_geometry_graph.md` §2.2.2) writes
//! `Effect::Stateful // cached by hash`, but in *this* substrate
//! [`Effect::Stateful`](ph2d_nodegraph::effect::Effect::Stateful) means "mutates
//! `SimWorld`, push side" and such nodes are **never driven by the presentation
//! `Cook`** (the membrane, ADR-0030). A boolean of two geometry inputs is a
//! referentially-transparent pull-side function — `Effect::Pure`. Making it
//! `Stateful` would make it invisible to the renderer, so the
//! `source → boolean → render` smoke would be dead (memory:
//! `feedback_tool_unit_green_integration_dead`). The `Cook` memoizes a `Pure`
//! node by `(input revisions + param hash)`, which is exactly the ADR's "cache
//! by `(input_a_hash, input_b_hash, op)`". Build against the real substrate, not
//! the aspirational pseudocode (memory: `project_vector_node_opaque_carrier`).
//!
//! ## Param `op` (the frozen 9-variant vocabulary)
//!
//! `op` is an `f32` **discriminant** matching [`BooleanOp`]'s declaration order
//! (`0`=Union … `8`=Outline), read via [`param_as_count`] — the same total
//! `f32 → index` discipline `vector.source` uses for `kind`, so no enum-param
//! contract extension is needed. Out-of-range values degrade to `Union`.

mod engine;

pub use engine::boolean;

use ph2d_node_registry::{NodeRegistry, RegistryError};
use ph2d_nodegraph::cook::EvalCtx;
use ph2d_nodegraph::effect::Effect;
use ph2d_nodegraph::node::{
    LoweringKind, NodeManifest, NodeOp, NodeTypeId, ParamSpec, PortSpec, param_as_count,
};
use ph2d_nodegraph::port::Clock;
use ph2d_vector_doc::{BooleanOp, VectorNetwork};
use ph2d_vector_graph::{VECTOR_PORT, VectorEvalExt};

/// Highest valid `op` discriminant (`Outline`); [`param_as_count`] clamps to it.
const OP_MAX: usize = 8;

/// The static contract of this node type (ADR-0031).
pub const MANIFEST: NodeManifest = NodeManifest {
    id: NodeTypeId::of("vector.boolean"),
    name: "vector.boolean",
    inputs: &[
        PortSpec {
            name: "a",
            ty: VECTOR_PORT,
        },
        PortSpec {
            name: "b",
            ty: VECTOR_PORT,
        },
    ],
    outputs: &[PortSpec {
        name: "out",
        ty: VECTOR_PORT,
    }],
    // Pure (pull side) — see the module-level "Effect" note. The Cook's memo
    // (input revisions + param hash) IS the ADR-0058 §2.2.2 result cache.
    effect: Effect::Pure,
    // Two static geometry inputs → a static result; re-cooked only when an input
    // or the `op` param changes (the `Static` clock, like `vector.source`).
    clock: Clock::Static,
    params: &[ParamSpec {
        name: "op",
        default: 0.0,
    }],
    lowerings: &[LoweringKind::Cpu],
};

/// Map the `op` discriminant index to its [`BooleanOp`]. Mirrors the enum's
/// declaration order; `0` and any out-of-range index degrade to `Union`.
#[must_use]
const fn op_from_index(i: usize) -> BooleanOp {
    match i {
        1 => BooleanOp::Subtract,
        2 => BooleanOp::Intersect,
        3 => BooleanOp::Exclude,
        4 => BooleanOp::Divide,
        5 => BooleanOp::Trim,
        6 => BooleanOp::Merge,
        7 => BooleanOp::Crop,
        8 => BooleanOp::Outline,
        // 0 = Union, and any stray discriminant.
        _ => BooleanOp::Union,
    }
}

struct VectorBoolean;

impl NodeOp for VectorBoolean {
    fn manifest(&self) -> &'static NodeManifest {
        &MANIFEST
    }

    fn eval(&self, ctx: &mut EvalCtx<'_>) {
        let op = op_from_index(param_as_count(ctx.param("op"), OP_MAX));
        // An unconnected geometry input is the empty set. Borrow the inputs
        // (no clone) for the engine; the immutable borrows end before the
        // `emit_network` mutable borrow (NLL).
        let empty = VectorNetwork::empty();
        let a = ctx.input_network(0).unwrap_or(&empty);
        let b = ctx.input_network(1).unwrap_or(&empty);
        let out = engine::boolean(a, b, op);
        ctx.emit_network(out);
    }
}

/// Register this node with the runtime registry. Called (via codegen) from
/// `ph2d-node-registry-init::register_all_nodes`.
pub fn register(reg: &mut NodeRegistry) -> Result<(), RegistryError> {
    reg.register(Box::new(VectorBoolean))
}

#[cfg(test)]
mod tests {
    use super::*;
    use ph2d_nodegraph::cook::{Cook, OpResolver};
    use ph2d_nodegraph::graph::{Edge, Graph};
    use ph2d_vector_doc::primitives;

    // Two in-test source nodes emitting fixed overlapping unit squares, so the
    // cook path (emit_network → opaque → input_network ×2 → boolean) is exercised
    // end-to-end without a dev-dep on the real `vector.source`.
    use ph2d_nodegraph::node::PortSpec as PS;

    fn square(lo: f32, hi: f32) -> VectorNetwork {
        let mut net = primitives::rect(glam::Vec2::new(lo, lo), glam::Vec2::new(hi, hi));
        net.deterministic = true;
        net
    }

    static SRC_A_MAN: NodeManifest = NodeManifest {
        id: NodeTypeId::of("vector.test.sqA"),
        name: "vector.test.sqA",
        inputs: &[],
        outputs: &[PS {
            name: "out",
            ty: VECTOR_PORT,
        }],
        effect: Effect::Pure,
        clock: Clock::Static,
        params: &[],
        lowerings: &[LoweringKind::Cpu],
    };
    static SRC_B_MAN: NodeManifest = NodeManifest {
        id: NodeTypeId::of("vector.test.sqB"),
        name: "vector.test.sqB",
        inputs: &[],
        outputs: &[PS {
            name: "out",
            ty: VECTOR_PORT,
        }],
        effect: Effect::Pure,
        clock: Clock::Static,
        params: &[],
        lowerings: &[LoweringKind::Cpu],
    };

    struct SrcA;
    impl NodeOp for SrcA {
        fn manifest(&self) -> &'static NodeManifest {
            &SRC_A_MAN
        }
        fn eval(&self, ctx: &mut EvalCtx<'_>) {
            ctx.emit_network(square(0.0, 2.0));
        }
    }
    struct SrcB;
    impl NodeOp for SrcB {
        fn manifest(&self) -> &'static NodeManifest {
            &SRC_B_MAN
        }
        fn eval(&self, ctx: &mut EvalCtx<'_>) {
            ctx.emit_network(square(1.0, 3.0));
        }
    }

    struct Ops;
    impl OpResolver for Ops {
        fn resolve(&self, ty: NodeTypeId) -> Option<&dyn NodeOp> {
            match ty {
                t if t == SRC_A_MAN.id => Some(&SrcA),
                t if t == SRC_B_MAN.id => Some(&SrcB),
                t if t == MANIFEST.id => Some(&VectorBoolean),
                _ => None,
            }
        }
    }

    fn cook_boolean(op_discriminant: f32) -> VectorNetwork {
        let mut g = Graph::new();
        let a = g.add_node("vector.test.sqA");
        let b = g.add_node("vector.test.sqB");
        let bool_node = g.add_node("vector.boolean");
        g.set_param(bool_node, "op", op_discriminant);
        g.connect(Edge {
            from: (a, 0),
            to: (bool_node, 0),
            delayed: false,
        })
        .unwrap();
        g.connect(Edge {
            from: (b, 0),
            to: (bool_node, 1),
            delayed: false,
        })
        .unwrap();
        let mut cook = Cook::new();
        let out = cook.cook(&g, &Ops, bool_node, 0.0).unwrap();
        out[0]
            .as_any()
            .and_then(|x| x.downcast_ref::<VectorNetwork>())
            .expect("boolean output carries a VectorNetwork")
            .clone()
    }

    #[test]
    fn op_discriminant_maps_every_index() {
        // The 9 frozen variants in declaration order; out-of-range → Union.
        assert!(matches!(op_from_index(0), BooleanOp::Union));
        assert!(matches!(op_from_index(1), BooleanOp::Subtract));
        assert!(matches!(op_from_index(2), BooleanOp::Intersect));
        assert!(matches!(op_from_index(3), BooleanOp::Exclude));
        assert!(matches!(op_from_index(4), BooleanOp::Divide));
        assert!(matches!(op_from_index(5), BooleanOp::Trim));
        assert!(matches!(op_from_index(6), BooleanOp::Merge));
        assert!(matches!(op_from_index(7), BooleanOp::Crop));
        assert!(matches!(op_from_index(8), BooleanOp::Outline));
        assert!(matches!(op_from_index(99), BooleanOp::Union));
    }

    #[test]
    fn union_through_a_real_cook_is_one_validated_region() {
        // The full smoke path: sqA(0..2) ∪ sqB(1..3) → one connected staircase
        // region, emitted as an opaque VectorNetwork and downcast back.
        let net = cook_boolean(0.0); // Union
        assert!(net.validate().is_ok());
        assert_eq!(net.regions.len(), 1, "overlapping union is one region");
        assert!(net.deterministic, "both inputs deterministic → output too");
    }

    #[test]
    fn intersect_through_a_real_cook_is_the_overlap_square() {
        let net = cook_boolean(2.0); // Intersect
        assert!(net.validate().is_ok());
        assert_eq!(net.regions.len(), 1, "the [1,2]² overlap is one region");
    }

    #[test]
    fn every_op_cooks_to_a_valid_network() {
        // DoD: all 9 variants run through Linesweeper without producing an
        // invalid network or panicking.
        for i in 0..=8u32 {
            let net = cook_boolean(i as f32);
            assert!(
                net.validate().is_ok(),
                "op discriminant {i} produced an invalid network"
            );
        }
    }
}
