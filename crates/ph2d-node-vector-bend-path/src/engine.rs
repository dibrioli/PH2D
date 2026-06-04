//! The bend-path engine: bend geometry along a circular arc by its x-position
//! (Illustrator "Arc" warp). The shape's horizontal centreline maps to an arc
//! subtending `angle` over the bounding-box width; a point keeps its height as a
//! radial offset. Topology is preserved (positions + tangents move only).
//!
//! ## Determinism
//!
//! `f64` trig; the result is snapped to the Q16.16 grid when the input is
//! `deterministic`.

use std::collections::BTreeMap;

use glam::Vec2;
use ph2d_vector_doc::deterministic::snap;
use ph2d_vector_doc::{VectorNetwork, VertexId};

/// Bend `input` by `angle_deg` (total arc subtended over the shape's width;
/// sign flips the bend direction).
#[must_use]
pub fn bend(input: &VectorNetwork, angle_deg: f32) -> VectorNetwork {
    if input.vertices.is_empty() || angle_deg == 0.0 || !angle_deg.is_finite() {
        return input.clone();
    }
    let (min, max) = bbox(input);
    let center = (min + max) * 0.5;
    let width = max.x - min.x;
    if width <= f32::EPSILON {
        return input.clone();
    }
    let theta = f64::from(angle_deg).to_radians();
    let radius = f64::from(width) / theta;
    let cx = f64::from(center.x);
    let cy = f64::from(center.y);
    // φ(x) = arc angle at x: s / R, with s = x − cx.
    let phi = |x: f32| (f64::from(x) - cx) * theta / f64::from(width);

    // Map a point onto the arc: O = (cx, cy − R); the centreline sits at radius
    // R, the point's height `h = y − cy` is a radial offset.
    let bend_point = |p: Vec2| -> Vec2 {
        let a = phi(p.x);
        let (sin, cos) = a.sin_cos();
        let h = f64::from(p.y) - cy;
        let r = radius + h;
        Vec2::new((cx + r * sin) as f32, (cy - radius + r * cos) as f32)
    };

    let pos_of: BTreeMap<VertexId, Vec2> = input.vertices.iter().map(|v| (v.id, v.pos)).collect();
    let mut out = input.clone();
    for v in out.vertices.iter_mut() {
        v.pos = bend_point(v.pos);
    }
    // Tangents rotate by the local arc angle at their endpoint (approximation,
    // like twist): a relative vector rotates by φ at that vertex's x.
    for s in out.segments.iter_mut() {
        if let Some(&p) = pos_of.get(&s.start) {
            s.out_at_start = rotate_vec(s.out_at_start, phi(p.x));
        }
        if let Some(&p) = pos_of.get(&s.end) {
            s.in_at_end = rotate_vec(s.in_at_end, phi(p.x));
        }
    }
    if out.deterministic {
        snap_network(&mut out);
    }
    out
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

    fn wide_rect() -> VectorNetwork {
        let mut net = primitives::rect(Vec2::new(-100.0, -10.0), Vec2::new(100.0, 10.0));
        net.deterministic = true;
        net
    }

    #[test]
    fn bend_preserves_topology() {
        let input = wide_rect();
        let out = bend(&input, 90.0);
        assert!(out.validate().is_ok());
        assert_eq!(out.vertices.len(), input.vertices.len());
        assert_eq!(out.segments.len(), input.segments.len());
        assert_eq!(out.regions.len(), input.regions.len());
    }

    #[test]
    fn bend_rotates_the_far_corners() {
        // The far corners (x = ±100) ride up the arc by ±angle/2, so each is
        // displaced substantially from its original axis-aligned position.
        let input = wide_rect();
        let out = bend(&input, 120.0);
        let max_move = input
            .vertices
            .iter()
            .zip(&out.vertices)
            .map(|(a, b)| a.pos.distance(b.pos))
            .fold(0.0_f32, f32::max);
        assert!(
            max_move > 20.0,
            "a 120° bend must move the far corners appreciably (max move {max_move})"
        );
    }

    #[test]
    fn zero_angle_is_identity() {
        assert_eq!(bend(&wide_rect(), 0.0), wide_rect());
    }

    #[test]
    fn deterministic_and_reproducible() {
        let out = bend(&wide_rect(), 75.0);
        assert!(out.deterministic);
        assert_eq!(out, bend(&wide_rect(), 75.0), "byte-stable");
    }

    #[test]
    fn empty_input_is_identity() {
        let out = bend(&VectorNetwork::empty(), 90.0);
        assert!(out.validate().is_ok());
        assert!(out.vertices.is_empty());
    }
}
