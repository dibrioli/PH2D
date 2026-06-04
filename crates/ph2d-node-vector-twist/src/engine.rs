//! The twist engine: rotate every vertex about the network's bounding-box
//! centre by an angle **proportional to its distance from the centre** — 0 at
//! the centre, the full `angle` at the rim. Topology is preserved (same
//! vertices/segments/regions); only positions and tangents move.
//!
//! ## Determinism
//!
//! The rotation uses `f64` trig (whose last ULP can differ across targets), so
//! when the input is `deterministic` the output is snapped to the Q16.16 grid —
//! the same drift-erasing trick `vector.source` uses for its angular
//! generators.

use std::collections::BTreeMap;

use glam::Vec2;
use ph2d_vector_doc::deterministic::snap;
use ph2d_vector_doc::{VectorNetwork, VertexId};

/// Rotate `input` by `angle_deg` of twist (full at the rim, 0 at the centre).
#[must_use]
pub fn twist(input: &VectorNetwork, angle_deg: f32) -> VectorNetwork {
    if input.vertices.is_empty() || angle_deg == 0.0 || !angle_deg.is_finite() {
        return input.clone();
    }
    let center = bbox_center(input);
    let dmax = input
        .vertices
        .iter()
        .map(|v| v.pos.distance(center))
        .fold(0.0_f32, f32::max);
    if dmax <= f32::EPSILON {
        return input.clone();
    }
    let angle = f64::from(angle_deg).to_radians();
    // Rotation amount at a point, ∝ its distance from the centre.
    let theta = |p: Vec2| angle * f64::from(p.distance(center) / dmax);

    // Original vertex positions, so a segment's tangents rotate by the rotation
    // at their endpoints (computed from the *pre-twist* geometry).
    let pos_of: BTreeMap<VertexId, Vec2> = input.vertices.iter().map(|v| (v.id, v.pos)).collect();

    let mut out = input.clone();
    for v in out.vertices.iter_mut() {
        v.pos = rotate_about(v.pos, center, theta(v.pos));
    }
    for s in out.segments.iter_mut() {
        if let Some(&p) = pos_of.get(&s.start) {
            s.out_at_start = rotate_vec(s.out_at_start, theta(p));
        }
        if let Some(&p) = pos_of.get(&s.end) {
            s.in_at_end = rotate_vec(s.in_at_end, theta(p));
        }
    }
    if out.deterministic {
        snap_network(&mut out);
    }
    out
}

fn bbox_center(net: &VectorNetwork) -> Vec2 {
    let mut min = Vec2::splat(f32::INFINITY);
    let mut max = Vec2::splat(f32::NEG_INFINITY);
    for v in &net.vertices {
        min = min.min(v.pos);
        max = max.max(v.pos);
    }
    (min + max) * 0.5
}

/// Rotate point `p` about `center` by `t` radians (math in `f64`).
fn rotate_about(p: Vec2, center: Vec2, t: f64) -> Vec2 {
    let (sin, cos) = t.sin_cos();
    let dx = f64::from(p.x - center.x);
    let dy = f64::from(p.y - center.y);
    Vec2::new(
        center.x + (cos * dx - sin * dy) as f32,
        center.y + (sin * dx + cos * dy) as f32,
    )
}

/// Rotate a free vector (relative tangent) by `t` radians — no translation.
fn rotate_vec(v: Vec2, t: f64) -> Vec2 {
    let (sin, cos) = t.sin_cos();
    let dx = f64::from(v.x);
    let dy = f64::from(v.y);
    Vec2::new((cos * dx - sin * dy) as f32, (sin * dx + cos * dy) as f32)
}

fn snap_network(net: &mut VectorNetwork) {
    for v in net.vertices.iter_mut() {
        v.pos = Vec2::new(snap(v.pos.x), snap(v.pos.y));
    }
    for s in net.segments.iter_mut() {
        s.out_at_start = Vec2::new(snap(s.out_at_start.x), snap(s.out_at_start.y));
        s.in_at_end = Vec2::new(snap(s.in_at_end.x), snap(s.in_at_end.y));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ph2d_vector_doc::primitives;

    fn square() -> VectorNetwork {
        let mut net = primitives::rect(Vec2::new(-2.0, -2.0), Vec2::new(2.0, 2.0));
        net.deterministic = true;
        net
    }

    #[test]
    fn twist_preserves_topology() {
        let input = square();
        let out = twist(&input, 90.0);
        assert!(out.validate().is_ok());
        assert_eq!(out.vertices.len(), input.vertices.len());
        assert_eq!(out.segments.len(), input.segments.len());
        assert_eq!(out.regions.len(), input.regions.len());
    }

    #[test]
    fn twist_actually_moves_the_rim() {
        let input = square();
        let out = twist(&input, 90.0);
        // At least one corner moved appreciably (the rim rotates by ~90°).
        let moved = input
            .vertices
            .iter()
            .zip(&out.vertices)
            .any(|(a, b)| a.pos.distance(b.pos) > 0.5);
        assert!(moved, "a 90° twist must move the rim");
    }

    #[test]
    fn twist_preserves_distance_from_center() {
        // Rotation is rigid about the centre → every vertex keeps its radius.
        let input = square();
        let center = bbox_center(&input);
        let out = twist(&input, 137.0);
        for (a, b) in input.vertices.iter().zip(&out.vertices) {
            let ra = a.pos.distance(center);
            let rb = b.pos.distance(center);
            assert!((ra - rb).abs() < 0.01, "radius changed: {ra} → {rb}");
        }
    }

    #[test]
    fn zero_angle_is_identity() {
        let input = square();
        assert_eq!(twist(&input, 0.0), input);
    }

    #[test]
    fn deterministic_and_reproducible() {
        let out = twist(&square(), 90.0);
        assert!(out.deterministic);
        assert_eq!(out, twist(&square(), 90.0), "byte-stable");
    }

    #[test]
    fn empty_input_is_identity() {
        let out = twist(&VectorNetwork::empty(), 90.0);
        assert!(out.validate().is_ok());
        assert!(out.vertices.is_empty());
    }
}
