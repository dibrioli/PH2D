#![forbid(unsafe_code)]
//! `vector.source` — the geometry **generator** node (ADR-0058 §2.2.1, plan
//! T3.2). One multi-variant crate emits any of the five procedural primitives
//! (rect / ellipse / polygon / star / spiral) as a [`VectorNetwork`] on its
//! single geometry output port. It consolidates the five generators that
//! already live in [`ph2d_vector_doc::primitives`] (W2 Shape tool) — this node
//! is the W3 graph wrapper, so both consumers share one implementation
//! (resolves Antigravity "critique A").
//!
//! ## Carrier (ADR-0058-amendment-1)
//!
//! The output is a `VectorNetwork`, carried on the edge through the substrate's
//! type-erased opaque channel via [`ph2d_vector_graph::VectorEvalExt::emit_network`].
//! The node author never touches `Arc<dyn Any>`.
//!
//! ## Params (all scalar `f32`, the frozen vocabulary)
//!
//! `kind` is a **discriminant** (`0`=Rect, `1`=Ellipse, `2`=Polygon, `3`=Star,
//! `4`=Spiral; any other value → Rect), read via [`param_as_count`] — the
//! existing total/saturating `f32 → count` conversion, so no enum-param
//! contract extension is needed (ADR-0058-amendment-1 §2.4). `width`/`height`
//! are the bounding box; radial shapes (polygon/star/spiral) use `width` as the
//! diameter and ignore `height`. `sides` is the polygon side / star point
//! count; `inner_ratio` the star inner radius as a fraction of the outer;
//! `turns`/`samples_per_turn` the spiral.
//!
//! ## Determinism (T3.2 DoD — bit-identical cross-OS)
//!
//! The angular generators use `f64` trig, whose last ULP can differ across
//! targets (libm / FMA). [`source_network`] snaps every coordinate to the
//! Q16.16 grid ([`ph2d_vector_doc::deterministic::snap`]) and flags the network
//! `deterministic`, erasing that drift — the output is byte-identical on every
//! platform (ADR-0056 §2.7).

use glam::Vec2;
use ph2d_node_registry::{NodeRegistry, RegistryError};
use ph2d_nodegraph::cook::EvalCtx;
use ph2d_nodegraph::effect::Effect;
use ph2d_nodegraph::node::{
    LoweringKind, NodeManifest, NodeOp, NodeTypeId, ParamSpec, PortSpec, param_as_count,
};
use ph2d_nodegraph::port::Clock;
use ph2d_vector_doc::deterministic::snap;
use ph2d_vector_doc::{VectorNetwork, primitives};
use ph2d_vector_graph::{VECTOR_PORT, VectorEvalExt};

// `kind` discriminants. Rect is `0` and the fallback for any out-of-range value.
const KIND_ELLIPSE: usize = 1;
const KIND_POLYGON: usize = 2;
const KIND_STAR: usize = 3;
const KIND_SPIRAL: usize = 4;
/// Highest valid `kind`; `param_as_count` clamps the discriminant to it.
const KIND_MAX: usize = KIND_SPIRAL;

/// Allocation ceilings for the count params. The generators re-clamp to their
/// own frozen geometry ceilings (`MAX_POLYGON_SIDES` etc.); these only bound the
/// untrusted `f32 → count` conversion against an allocation-overflow value.
const MAX_SIDES: usize = 1 << 12;
const MAX_SAMPLES_PER_TURN: usize = 1 << 8;

/// The static contract of this node type (ADR-0031).
pub const MANIFEST: NodeManifest = NodeManifest {
    id: NodeTypeId::of("vector.source"),
    name: "vector.source",
    inputs: &[],
    outputs: &[PortSpec {
        name: "out",
        ty: VECTOR_PORT,
    }],
    effect: Effect::Pure,
    // A source is cooked once and re-cooked only on a param edit (the memo
    // fingerprint folds in param overrides) — the `Static` clock.
    clock: Clock::Static,
    params: &[
        ParamSpec {
            name: "kind",
            default: 0.0,
        },
        ParamSpec {
            name: "width",
            default: 100.0,
        },
        ParamSpec {
            name: "height",
            default: 100.0,
        },
        ParamSpec {
            name: "sides",
            default: 6.0,
        },
        ParamSpec {
            name: "inner_ratio",
            default: 0.4,
        },
        ParamSpec {
            name: "turns",
            default: 3.0,
        },
        ParamSpec {
            name: "samples_per_turn",
            default: 24.0,
        },
        ParamSpec {
            name: "rotation",
            default: 0.0,
        },
    ],
    lowerings: &[LoweringKind::Cpu],
};

#[inline]
fn snap_vec2(v: Vec2) -> Vec2 {
    Vec2::new(snap(v.x), snap(v.y))
}

/// Build the primitive selected by `kind` (Rect for `0`/out-of-range),
/// centered at the origin, then snap every vertex position and segment tangent
/// to the Q16.16 grid so the result is **byte-identical across targets**. The
/// returned network is flagged `deterministic`. Pure and isolated so the
/// variant selection + snapping is unit-tested directly, alongside the
/// end-to-end cook test that drives it through the real graph.
#[must_use]
pub fn source_network(
    kind: usize,
    width: f32,
    height: f32,
    sides: u32,
    inner_ratio: f32,
    turns: f32,
    samples_per_turn: u32,
    rotation: f32,
) -> VectorNetwork {
    let hw = width * 0.5;
    let hh = height * 0.5;
    let mut net = match kind {
        KIND_ELLIPSE => primitives::ellipse(Vec2::ZERO, Vec2::new(hw, hh)),
        KIND_POLYGON => primitives::polygon(Vec2::ZERO, hw, sides, rotation),
        KIND_STAR => primitives::star(Vec2::ZERO, hw, hw * inner_ratio, sides, rotation),
        KIND_SPIRAL => primitives::spiral(Vec2::ZERO, 0.0, hw, turns, samples_per_turn, rotation),
        // Rect (kind 0) and any out-of-range discriminant.
        _ => primitives::rect(Vec2::new(-hw, -hh), Vec2::new(hw, hh)),
    };
    for v in net.vertices.iter_mut() {
        v.pos = snap_vec2(v.pos);
    }
    for s in net.segments.iter_mut() {
        s.out_at_start = snap_vec2(s.out_at_start);
        s.in_at_end = snap_vec2(s.in_at_end);
    }
    net.deterministic = true;
    net
}

struct VectorSource;

impl NodeOp for VectorSource {
    fn manifest(&self) -> &'static NodeManifest {
        &MANIFEST
    }

    fn eval(&self, ctx: &mut EvalCtx<'_>) {
        let net = source_network(
            param_as_count(ctx.param("kind"), KIND_MAX),
            ctx.param("width"),
            ctx.param("height"),
            param_as_count(ctx.param("sides"), MAX_SIDES) as u32,
            ctx.param("inner_ratio"),
            ctx.param("turns"),
            param_as_count(ctx.param("samples_per_turn"), MAX_SAMPLES_PER_TURN) as u32,
            ctx.param("rotation"),
        );
        ctx.emit_network(net);
    }
}

/// Register this node with the runtime registry. Called (via codegen) from
/// `ph2d-node-registry-init::register_all_nodes`.
pub fn register(reg: &mut NodeRegistry) -> Result<(), RegistryError> {
    reg.register(Box::new(VectorSource))
}

#[cfg(test)]
mod tests {
    use super::*;
    use ph2d_nodegraph::cook::{Cook, OpResolver};
    use ph2d_nodegraph::graph::Graph;

    // Default params (matching MANIFEST) for the four kinds the golden tests
    // exercise; width=height=100 → a 100×100 box centered at the origin.
    fn build(kind: usize) -> VectorNetwork {
        source_network(kind, 100.0, 100.0, 6, 0.4, 3.0, 24, 0.0)
    }

    /// Every coordinate of `net` lies exactly on the Q16.16 grid — the property
    /// that makes the output bit-identical across targets (the trig drift is
    /// erased). If this holds, the golden bytes are platform-independent.
    fn all_coords_grid_snapped(net: &VectorNetwork) -> bool {
        net.vertices.iter().all(|v| v.pos == snap_vec2(v.pos))
            && net.segments.iter().all(|s| {
                s.out_at_start == snap_vec2(s.out_at_start) && s.in_at_end == snap_vec2(s.in_at_end)
            })
    }

    #[test]
    fn rect_structure_and_exact_corners() {
        // No trig → exact golden coordinates. 100×100 centered → ±50 corners.
        let net = build(0);
        assert_eq!(net.vertices.len(), 4);
        assert_eq!(net.segments.len(), 4);
        assert_eq!(net.regions.len(), 1);
        assert!(net.validate().is_ok());
        assert_eq!(net.vertices[0].pos, Vec2::new(-50.0, -50.0));
        assert_eq!(net.vertices[2].pos, Vec2::new(50.0, 50.0));
        assert!(net.deterministic);
    }

    #[test]
    fn ellipse_polygon_star_structure() {
        let ell = build(KIND_ELLIPSE);
        assert_eq!(
            (ell.vertices.len(), ell.segments.len(), ell.regions.len()),
            (4, 4, 1)
        );
        let poly = build(KIND_POLYGON); // sides = 6
        assert_eq!(
            (poly.vertices.len(), poly.segments.len(), poly.regions.len()),
            (6, 6, 1)
        );
        let star = build(KIND_STAR); // 2 * 6 points
        assert_eq!(
            (star.vertices.len(), star.segments.len(), star.regions.len()),
            (12, 12, 1)
        );
        for net in [&ell, &poly, &star] {
            assert!(net.validate().is_ok());
        }
    }

    #[test]
    fn spiral_is_open_no_region() {
        let sp = build(KIND_SPIRAL);
        assert!(sp.regions.is_empty(), "spiral is an open path");
        assert_eq!(sp.segments.len(), sp.vertices.len() - 1, "open chain");
        assert!(sp.validate().is_ok());
    }

    #[test]
    fn out_of_range_kind_falls_back_to_rect() {
        // param_as_count clamps to KIND_MAX in eval, but source_network is pub —
        // a stray discriminant must degrade to Rect, never panic.
        let net = source_network(99, 100.0, 100.0, 6, 0.4, 3.0, 24, 0.0);
        assert_eq!(net.vertices.len(), 4);
        assert_eq!(net.regions.len(), 1);
    }

    #[test]
    fn every_variant_is_grid_snapped_and_reproducible() {
        // Cross-OS golden guarantee: every kind's coords are on the Q16.16 grid,
        // and two builds are byte-equal (PartialEq over the whole network).
        for kind in 0..=KIND_MAX {
            let a = build(kind);
            assert!(
                all_coords_grid_snapped(&a),
                "kind {kind} has off-grid coords (would drift cross-OS)"
            );
            assert_eq!(a, build(kind), "kind {kind} not reproducible");
        }
    }

    struct Ops;
    impl OpResolver for Ops {
        fn resolve(&self, ty: NodeTypeId) -> Option<&dyn NodeOp> {
            (ty == MANIFEST.id).then_some(&VectorSource as &dyn NodeOp)
        }
    }

    #[test]
    fn emits_geometry_through_a_real_cook() {
        // The full path (landmine #1: trace it end to end): add the node, cook,
        // and downcast the opaque output back to a VectorNetwork — proving
        // emit_network → CookValue::Opaque → cook cache → downcast all connect.
        let mut g = Graph::new();
        let src = g.add_node("vector.source");
        let mut cook = Cook::new();
        let out = cook.cook(&g, &Ops, src, 0.0).unwrap();
        let net = out[0]
            .as_any()
            .and_then(|a| a.downcast_ref::<VectorNetwork>())
            .expect("port 0 carries a VectorNetwork");
        // Default kind = Rect → 4 corners.
        assert_eq!(net.vertices.len(), 4);
        assert!(net.validate().is_ok());
    }

    #[test]
    fn per_param_override_switches_variant_through_the_cook() {
        // Override kind → Polygon and sides → 5 on the real cook path; the
        // emitted geometry must be a pentagon (5 vertices), proving params reach
        // the generator (not just the manifest defaults).
        let mut g = Graph::new();
        let src = g.add_node("vector.source");
        g.set_param(src, "kind", KIND_POLYGON as f32);
        g.set_param(src, "sides", 5.0);
        let mut cook = Cook::new();
        let out = cook.cook(&g, &Ops, src, 0.0).unwrap();
        let net = out[0]
            .as_any()
            .and_then(|a| a.downcast_ref::<VectorNetwork>())
            .expect("geometry output");
        assert_eq!(net.vertices.len(), 5, "pentagon");
    }
}
