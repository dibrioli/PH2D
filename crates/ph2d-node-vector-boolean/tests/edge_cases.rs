//! T3.5 lens A — boolean **robustness** audit (the cases where Clipper-family
//! engines fail: coincident edges, shared vertices, tangent contact, holes,
//! fully-coincident boundaries). Each case asserts the engine stays total (no
//! panic), produces a `validate`-clean network, and — where the result is
//! unambiguous — the right faces. These are the executable gate behind the
//! "Linesweeper survives" claim for the early-beta dependency.
//!
//! Also pins one **golden** result (cross-OS determinism via Q16.16).

use glam::Vec2;
use ph2d_node_vector_boolean::boolean;
use ph2d_vector_doc::{BooleanOp, VectorNetwork, primitives};

fn square(lo_x: f32, lo_y: f32, hi_x: f32, hi_y: f32) -> VectorNetwork {
    let mut net = primitives::rect(Vec2::new(lo_x, lo_y), Vec2::new(hi_x, hi_y));
    net.deterministic = true;
    net
}

fn circle(cx: f32, cy: f32, r: f32) -> VectorNetwork {
    let mut net = primitives::ellipse(Vec2::new(cx, cy), Vec2::new(r, r));
    net.deterministic = true;
    net
}

fn bbox(net: &VectorNetwork) -> (Vec2, Vec2) {
    let mut min = Vec2::splat(f32::INFINITY);
    let mut max = Vec2::splat(f32::NEG_INFINITY);
    for v in &net.vertices {
        min = min.min(v.pos);
        max = max.max(v.pos);
    }
    (min, max)
}

fn approx(a: Vec2, b: Vec2) -> bool {
    (a - b).length() < 0.02
}

// ── Coincident edges (Clipper's classic failure) ──────────────────────────

#[test]
fn coincident_edge_union_dissolves_the_shared_border() {
    // [0,2]×[0,2] ∪ [2,0]×[4,2] share the edge x=2 exactly.
    let a = square(0.0, 0.0, 2.0, 2.0);
    let b = square(2.0, 0.0, 4.0, 2.0);
    let out = boolean(&a, &b, BooleanOp::Union);
    assert!(out.validate().is_ok());
    assert_eq!(out.regions.len(), 1, "coincident-edge union is one rect");
    let (min, max) = bbox(&out);
    assert!(approx(min, Vec2::new(0.0, 0.0)) && approx(max, Vec2::new(4.0, 2.0)));
}

#[test]
fn coincident_edge_intersection_has_no_area() {
    let a = square(0.0, 0.0, 2.0, 2.0);
    let b = square(2.0, 0.0, 4.0, 2.0);
    let out = boolean(&a, &b, BooleanOp::Intersect);
    assert!(out.validate().is_ok());
    assert!(out.regions.is_empty(), "a shared edge encloses no area");
}

// ── Shared vertex (corner-touch) ──────────────────────────────────────────

#[test]
fn shared_vertex_is_handled_without_panic() {
    // Two squares touching only at the point (2,2).
    let a = square(0.0, 0.0, 2.0, 2.0);
    let b = square(2.0, 2.0, 4.0, 4.0);
    for op in [BooleanOp::Union, BooleanOp::Intersect, BooleanOp::Subtract] {
        let out = boolean(&a, &b, op);
        assert!(out.validate().is_ok(), "corner-touch op {op:?} invalid");
    }
    // Intersection of a point-touch has no area.
    assert!(
        boolean(&a, &b, BooleanOp::Intersect).regions.is_empty(),
        "point contact has no area"
    );
}

// ── Tangent contact (curves touching at one point) ────────────────────────

#[test]
fn tangent_circles_do_not_panic() {
    // Unit circles touching at (1,0).
    let a = circle(0.0, 0.0, 1.0);
    let b = circle(2.0, 0.0, 1.0);
    for op in [BooleanOp::Union, BooleanOp::Intersect, BooleanOp::Exclude] {
        let out = boolean(&a, &b, op);
        assert!(out.validate().is_ok(), "tangent op {op:?} invalid");
    }
}

// ── Fully-coincident boundaries (A op A — the hardest degenerate) ──────────

#[test]
fn identical_inputs_self_subtract_to_empty() {
    let a = square(0.0, 0.0, 2.0, 2.0);
    let same = square(0.0, 0.0, 2.0, 2.0);
    let out = boolean(&a, &same, BooleanOp::Subtract);
    assert!(out.validate().is_ok());
    assert!(out.regions.is_empty(), "A ∖ A is empty");
}

#[test]
fn identical_inputs_union_is_the_shape() {
    let a = square(0.0, 0.0, 2.0, 2.0);
    let same = square(0.0, 0.0, 2.0, 2.0);
    let out = boolean(&a, &same, BooleanOp::Union);
    assert!(out.validate().is_ok());
    assert_eq!(out.regions.len(), 1, "A ∪ A is A");
    let (min, max) = bbox(&out);
    assert!(approx(min, Vec2::new(0.0, 0.0)) && approx(max, Vec2::new(2.0, 2.0)));
}

#[test]
fn identical_inputs_intersect_is_the_shape() {
    let a = square(0.0, 0.0, 2.0, 2.0);
    let same = square(0.0, 0.0, 2.0, 2.0);
    let out = boolean(&a, &same, BooleanOp::Intersect);
    assert!(out.validate().is_ok());
    assert_eq!(out.regions.len(), 1, "A ∩ A is A");
}

#[test]
fn identical_inputs_exclude_is_empty() {
    let a = square(0.0, 0.0, 2.0, 2.0);
    let same = square(0.0, 0.0, 2.0, 2.0);
    let out = boolean(&a, &same, BooleanOp::Exclude);
    assert!(out.validate().is_ok());
    assert!(out.regions.is_empty(), "A ⊕ A is empty");
}

// ── Nested (hole-producing) ───────────────────────────────────────────────

#[test]
fn subtract_inner_hole_yields_outer_plus_hole_contours() {
    // A big square minus a fully-interior small square → a frame with a hole.
    // Linesweeper emits the hole as a separate (oppositely-wound) contour, so
    // the carrier carries TWO regions (outer boundary + hole boundary).
    // NB: rendering this AS a hole needs renderer multi-loop/parent support —
    // a documented W3 limitation (the carrier is faithful; the painter fills
    // each region independently today).
    let a = square(0.0, 0.0, 10.0, 10.0);
    let b = square(3.0, 3.0, 7.0, 7.0);
    let out = boolean(&a, &b, BooleanOp::Subtract);
    assert!(out.validate().is_ok());
    assert_eq!(out.regions.len(), 2, "frame + hole = two contours");
}

#[test]
fn intersect_nested_returns_inner() {
    let a = square(0.0, 0.0, 10.0, 10.0);
    let b = square(3.0, 3.0, 7.0, 7.0);
    let out = boolean(&a, &b, BooleanOp::Intersect);
    assert!(out.validate().is_ok());
    assert_eq!(out.regions.len(), 1);
    let (min, max) = bbox(&out);
    assert!(approx(min, Vec2::new(3.0, 3.0)) && approx(max, Vec2::new(7.0, 7.0)));
}

#[test]
fn union_nested_returns_outer() {
    let a = square(0.0, 0.0, 10.0, 10.0);
    let b = square(3.0, 3.0, 7.0, 7.0);
    let out = boolean(&a, &b, BooleanOp::Union);
    assert!(out.validate().is_ok());
    assert_eq!(out.regions.len(), 1);
    let (min, max) = bbox(&out);
    assert!(approx(min, Vec2::new(0.0, 0.0)) && approx(max, Vec2::new(10.0, 10.0)));
}

// ── Golden (cross-OS determinism) ─────────────────────────────────────────

#[test]
fn golden_intersection_corners_are_exact_on_the_grid() {
    // [0,2]² ∩ [1,3]² = exactly the unit square [1,2]². With Q16.16 snapping the
    // four corners are bit-exact integers on every target — the cross-OS golden.
    let out = boolean(&square(0.0, 0.0, 2.0, 2.0), &square(1.0, 1.0, 3.0, 3.0), BooleanOp::Intersect);
    assert!(out.validate().is_ok());
    assert_eq!(out.regions.len(), 1);
    let mut corners: Vec<(f32, f32)> = out.vertices.iter().map(|v| (v.pos.x, v.pos.y)).collect();
    corners.sort_by(|a, b| a.partial_cmp(b).unwrap());
    assert_eq!(
        corners,
        vec![(1.0, 1.0), (1.0, 2.0), (2.0, 1.0), (2.0, 2.0)],
        "intersection corners must be exact grid integers"
    );
    // And byte-stable across a re-run.
    let again = boolean(&square(0.0, 0.0, 2.0, 2.0), &square(1.0, 1.0, 3.0, 3.0), BooleanOp::Intersect);
    assert_eq!(out, again);
}
