//! The warp engine: a basic **sine-wave envelope** — displace each vertex in Y
//! by `amplitude · sin(2π · frequency · (x − min_x) / width)`. Topology is
//! preserved (positions move; tangents are left as-is in v1).
//!
//! ## v1 scope
//!
//! This is a single-axis wave envelope. A full lattice / mesh warp (a control
//! grid the user drags, Illustrator "Envelope Distort") needs a grid-of-handles
//! parameter model the scalar param vocabulary doesn't carry yet — a follow-up.
//!
//! Determinism: `f64` trig; Q16.16-snapped output when the input is
//! `deterministic`.

use glam::Vec2;
use ph2d_vector_doc::VectorNetwork;
use ph2d_vector_doc::deterministic::snap;

/// Apply a sine-wave Y-warp of `amplitude` and `frequency` (waves across the
/// bounding-box width) to `input`.
#[must_use]
pub fn warp(input: &VectorNetwork, amplitude: f32, frequency: f32) -> VectorNetwork {
    if input.vertices.is_empty() || amplitude == 0.0 || !amplitude.is_finite() {
        return input.clone();
    }
    let (min, max) = bbox(input);
    let width = max.x - min.x;
    if width <= f32::EPSILON {
        return input.clone();
    }
    let amp = f64::from(amplitude);
    let two_pi_f = std::f64::consts::TAU * f64::from(frequency);
    let min_x = f64::from(min.x);
    let w = f64::from(width);
    let dy = |x: f32| amp * (two_pi_f * (f64::from(x) - min_x) / w).sin();

    let mut out = input.clone();
    for v in out.vertices.iter_mut() {
        v.pos.y += dy(v.pos.x) as f32;
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
    use ph2d_vector_doc::{Vertex, primitives};

    fn wide_rect() -> VectorNetwork {
        let mut net = primitives::rect(Vec2::new(0.0, -5.0), Vec2::new(100.0, 5.0));
        net.deterministic = true;
        net
    }

    #[test]
    fn warp_preserves_topology() {
        let input = wide_rect();
        let out = warp(&input, 20.0, 2.0);
        assert!(out.validate().is_ok());
        assert_eq!(out.vertices.len(), input.vertices.len());
        assert_eq!(out.segments.len(), input.segments.len());
        assert_eq!(out.regions.len(), input.regions.len());
    }

    #[test]
    fn warp_displaces_in_y_only() {
        // A vertex a quarter-wave along (x where sin peaks) lifts; x stays put.
        let mut net = VectorNetwork::empty();
        net.deterministic = true;
        net.vertices.push(Vertex::auto(0, Vec2::new(0.0, 0.0)));
        net.vertices.push(Vertex::auto(1, Vec2::new(25.0, 0.0))); // quarter wave at freq 1, width 100
        net.vertices.push(Vertex::auto(2, Vec2::new(100.0, 0.0)));
        let out = warp(&net, 10.0, 1.0);
        // x unchanged everywhere.
        for (a, b) in net.vertices.iter().zip(&out.vertices) {
            assert_eq!(a.pos.x, b.pos.x, "x must not move");
        }
        // The quarter-wave vertex lifts ~+amplitude.
        let lifted = out.vertices[1].pos.y;
        assert!(
            (lifted - 10.0).abs() < 0.1,
            "quarter-wave peak ~+10, got {lifted}"
        );
    }

    #[test]
    fn zero_amplitude_is_identity() {
        assert_eq!(warp(&wide_rect(), 0.0, 2.0), wide_rect());
    }

    #[test]
    fn deterministic_and_reproducible() {
        let out = warp(&wide_rect(), 15.0, 3.0);
        assert!(out.deterministic);
        assert_eq!(out, warp(&wide_rect(), 15.0, 3.0), "byte-stable");
    }

    #[test]
    fn empty_input_is_identity() {
        let out = warp(&VectorNetwork::empty(), 20.0, 2.0);
        assert!(out.validate().is_ok());
        assert!(out.vertices.is_empty());
    }
}
