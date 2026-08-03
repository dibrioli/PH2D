//! ADR-0154 gates for the shell half of `source.shape`. The load-bearing one is
//! the CROSS-SIDE single door: the shell publishes a shape's geometry under the
//! content key it computes with [`read_params`], and the node's `eval` reads an
//! external under the key IT computes with `ctx.param` — if those two reads
//! diverged, the node would clone the empty external and emit nothing. The gate
//! drives BOTH sides for real (publish + a live cook) so the two keys are computed
//! independently and shown to agree.

use super::{build_shape_path, encode, read_params, VecPathStore};
use crate::motion_state::MotionState;
use ph2d_node_motion_shape::{shape_key, ShapeKind, ShapeParams};
use ph2d_nodegraph::attr::{Column, Stream};

/// **The single door.** The shell interns a shape and publishes it under
/// `read_params`'s key; the node, cooked, reads the external under `ctx.param`'s
/// key and emits the geometry handle — and the store holds the `VecPath` for that
/// handle. FALSIFIED by a `read_params` that computes different param values than
/// the node's `ctx.param` (the keys would diverge → the node reads empty → count 0).
#[test]
fn publish_then_cook_the_node_reads_its_own_shape() {
    let mut state = MotionState::new();
    let n = state.doc.graph.add_node("source.shape");
    state.doc.graph.set_param(n, "kind", 5.0); // Star
    state.doc.graph.set_param(n, "size", 0.8);
    state.doc.graph.set_param(n, "sides", 6.0);

    // The real publish: intern the geometry + set the external on the pump's cook.
    super::publish(&mut state);

    let out = state
        .pump
        .cook
        .cook(&state.doc.graph, &state.registry, n, 0.0)
        .expect("cook");
    let stream = out[0].as_stream();
    assert_eq!(stream.count(), 1, "the node read the shell's published shape");
    let Some(Column::Scalar(ids)) = stream.get("geometry_id") else {
        panic!("geometry_id column");
    };
    let handle = ids[0] as u32;
    assert!(handle >= 1, "a live geometry handle");
    assert!(
        state.shape_store.get(handle).is_some(),
        "the store holds the shape's VecPath"
    );
}

/// **The negative control** — proves the round-trip gate is not vacuously green.
/// Publish a shape under the key of a DIFFERENT descriptor than the node authored;
/// the node's `ctx.param` key won't match, so it reads the empty external and
/// emits nothing.
#[test]
fn a_mismatched_key_decouples_the_node_from_the_shape() {
    let state = MotionState::new();
    let mut g = ph2d_nodegraph::graph::Graph::new();
    let n = g.add_node("source.shape");
    g.set_param(n, "kind", 5.0); // the node authors a Star
    // Publish under a Circle's key — a mismatch.
    let wrong = shape_key(&ShapeParams {
        kind: ShapeKind::Circle,
        ..read_params(g.node_param_overrides(n))
    });
    let mut cook = ph2d_nodegraph::cook::Cook::new();
    cook.set_external(wrong, Stream::new(1).with("geometry_id", Column::Scalar(vec![7.0])));
    let out = cook.cook(&g, &state.registry, n, 0.0).expect("cook");
    assert_eq!(
        out[0].as_stream().count(),
        0,
        "a key mismatch emits nothing — the round-trip gate has teeth"
    );
}

/// Each shape family builds a DISTINCT `VecPath` from the same descriptor — a
/// circle is not a square, a star is not a gear. FALSIFIED by a `build_shape_path`
/// arm that collapses two kinds to the same geometry.
#[test]
fn every_kind_builds_distinct_geometry() {
    let base = read_params(None); // the manifest defaults
    let kinds = [
        ShapeKind::Circle,
        ShapeKind::Square,
        ShapeKind::Ellipse,
        ShapeKind::Rectangle,
        ShapeKind::Polygon,
        ShapeKind::Star,
        ShapeKind::Heart,
        ShapeKind::Gear,
    ];
    // aspect ≠ 1 so Ellipse ≠ Circle and Rectangle ≠ Square even at the same size.
    let geoms: Vec<_> = kinds
        .iter()
        .map(|&kind| {
            build_shape_path(&ShapeParams {
                kind,
                aspect: 1.5,
                ..base
            })
            .verts
        })
        .collect();
    for (i, a) in geoms.iter().enumerate() {
        for (j, b) in geoms.iter().enumerate().skip(i + 1) {
            assert_ne!(a, b, "kind {i} and kind {j} build distinct geometry");
        }
    }
}

/// A handle with no stored geometry (an empty store, or a forward cook) resolves
/// to `None`, and encoding an instance that carries it draws nothing rather than
/// panicking. FALSIFIED by a `get` that indexes out of bounds or an `encode` that
/// unwraps.
#[test]
fn an_unpublished_handle_is_none_and_encodes_without_panic() {
    let store = VecPathStore::default();
    assert!(store.get(0).is_none(), "handle 0 is 'no geometry'");
    assert!(store.get(99).is_none(), "an unpublished handle resolves to None");

    let inst = ph2d_eval_motion::VectorInstance {
        geometry_id: 99,
        world_pos: [0.0, 0.0],
        size: [1.0, 1.0],
        basis: [1.0, 0.0, 0.0, 1.0],
        tint: [1.0, 1.0, 1.0, 1.0],
    };
    let mut scene = ph2d_vector::VectorScene::new();
    encode(&[inst], &store, ph2d_vector::Affine::IDENTITY, &mut scene);
    // Reaching here without a panic IS the assertion.
}

/// **The whole path, end to end** — the smoke's number, gated. A `source.shape`
/// (Star) is crossed with a `motion.grid` (4×4) through a `motion.duplicator`; the
/// cooked sink lowers to 16 `VectorInstance`s, every copy carrying the ONE handle
/// the shape interned (a duplicator that dropped `geometry_id` would give sprites,
/// not shapes). FALSIFIED by the duplicator not preserving the convention column.
#[test]
fn a_shape_stamped_on_a_grid_lowers_to_sixteen_vectors() {
    use ph2d_eval_motion::lower_to_vector_instances_onto;
    use ph2d_nodegraph::graph::{Edge, NodeId};

    let mut state = MotionState::new();
    let (src, out) = {
        let g = &mut state.doc.graph;
        let src = g.add_node("source.shape");
        let grid = g.add_node("motion.grid");
        let dup = g.add_node("motion.duplicator");
        let out = g.add_node("motion.output");
        g.set_param(src, "kind", 5.0); // Star
        g.set_param(src, "size", 0.4);
        g.set_param(grid, "rows", 4.0);
        g.set_param(grid, "cols", 4.0);
        let mut wire = |a: NodeId, ap: u16, b: NodeId, bp: u16| {
            g.connect(Edge {
                from: (a, ap),
                to: (b, bp),
                delayed: false,
            })
            .expect("connect");
        };
        wire(src, 0, dup, 0); // shape → duplicator.shape
        wire(grid, 0, dup, 1); // grid → duplicator.points
        wire(dup, 0, out, 0); // duplicator → output
        (src, out)
    };
    let _ = src;

    super::publish(&mut state);
    let cooked = state
        .pump
        .cook
        .cook(&state.doc.graph, &state.registry, out, 0.0)
        .expect("cook");
    let mut vecs = Vec::new();
    lower_to_vector_instances_onto(cooked[0].as_stream(), &mut vecs);
    assert_eq!(vecs.len(), 16, "a Star on a 4x4 grid = 16 crisp vector copies");

    let handle = vecs[0].geometry_id;
    assert!(handle >= 1, "a live geometry handle");
    assert!(
        vecs.iter().all(|v| v.geometry_id == handle),
        "one shape, one interned handle across all copies"
    );
    assert!(
        state.shape_store.get(handle).is_some(),
        "the store holds the stamped shape's VecPath"
    );
}
