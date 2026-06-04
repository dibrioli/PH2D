//! The hatch engine: fill each region with parallel lines at `angle`, spaced
//! `spacing` apart, clipped to the region interior. The output is a network of
//! **open line segments** (the hatch); render them with a stroke style (or feed
//! `vector.outline-stroke`).
//!
//! ## Method (v1)
//!
//! Rotate the region polygon by `−angle` so hatch lines are horizontal, run a
//! scanline every `spacing` over the rotated bbox, intersect each scanline with
//! the polygon edges, pair the crossings even-odd into interior spans, and
//! rotate the span endpoints back by `+angle`. Region boundaries are treated as
//! polygons (straight edges through their vertices) — curved edges are
//! polygonised, a documented follow-up.
//!
//! Determinism: scanlines are anchored at the rotated bbox min stepping by
//! `spacing` (reproducible); output snapped to Q16.16 when the input is
//! `deterministic`.

use std::collections::BTreeMap;

use glam::{DVec2, Vec2};
use ph2d_vector_doc::deterministic::snap;
use ph2d_vector_doc::{Region, Segment, SegmentId, VectorNetwork, Vertex, VertexId};

/// Allocation guard: max scanlines per region (bounds tiny `spacing` against
/// an OOM line count).
const MAX_LINES: usize = 1 << 14;

/// Hatch every region of `input` with parallel lines.
#[must_use]
pub fn hatch(input: &VectorNetwork, angle_deg: f32, spacing: f32) -> VectorNetwork {
    if spacing <= 0.0 || !spacing.is_finite() || input.regions.is_empty() {
        let mut empty = VectorNetwork::empty();
        empty.deterministic = input.deterministic;
        return empty;
    }
    let verts: BTreeMap<VertexId, Vec2> = input.vertices.iter().map(|v| (v.id, v.pos)).collect();
    let (sin, cos) = f64::from(angle_deg).to_radians().sin_cos();
    let mut out = VectorNetwork::empty();
    let mut next_v: VertexId = 0;
    let mut next_s: SegmentId = 0;

    for region in &input.regions {
        let Some(poly) = region_loop(region, &input.segments, &verts) else {
            continue;
        };
        if poly.len() < 3 {
            continue;
        }
        // Rotate into the hatch frame (lines horizontal).
        let rot: Vec<DVec2> = poly.iter().map(|&p| rotate(p, sin, cos)).collect();
        let (min_y, max_y) = rot
            .iter()
            .fold((f64::INFINITY, f64::NEG_INFINITY), |(lo, hi), p| {
                (lo.min(p.y), hi.max(p.y))
            });
        let space = f64::from(spacing);
        let n_lines = (((max_y - min_y) / space).floor() as usize).min(MAX_LINES);
        for k in 1..=n_lines {
            let y = min_y + k as f64 * space;
            let mut xs = scanline_crossings(&rot, y);
            xs.sort_by(|a, b| a.partial_cmp(b).unwrap());
            // Pair crossings even-odd into interior spans.
            for pair in xs.chunks_exact(2) {
                let a = unrotate(DVec2::new(pair[0], y), sin, cos);
                let b = unrotate(DVec2::new(pair[1], y), sin, cos);
                let va = next_v;
                let vb = next_v + 1;
                next_v += 2;
                out.vertices.push(Vertex::auto(va, a));
                out.vertices.push(Vertex::auto(vb, b));
                out.segments.push(Segment::straight(next_s, va, vb));
                next_s += 1;
            }
        }
    }

    if input.deterministic {
        for v in out.vertices.iter_mut() {
            v.pos = Vec2::new(snap(v.pos.x), snap(v.pos.y));
        }
    }
    out.deterministic = input.deterministic;
    out
}

/// The ordered loop of vertex positions of a region. `None` on a dangling ref.
fn region_loop(
    region: &Region,
    segments: &[Segment],
    verts: &BTreeMap<VertexId, Vec2>,
) -> Option<Vec<Vec2>> {
    let by_id: BTreeMap<SegmentId, &Segment> = segments.iter().map(|s| (s.id, s)).collect();
    let mut pts = Vec::with_capacity(region.segments.len());
    for &(sid, fwd) in &region.segments {
        let seg = by_id.get(&sid)?;
        let start_v = if fwd { seg.start } else { seg.end };
        pts.push(*verts.get(&start_v)?);
    }
    Some(pts)
}

/// X coordinates where the horizontal line `y` crosses the polygon edges.
fn scanline_crossings(poly: &[DVec2], y: f64) -> Vec<f64> {
    let m = poly.len();
    let mut xs = Vec::new();
    for i in 0..m {
        let a = poly[i];
        let b = poly[(i + 1) % m];
        // Half-open edge test [min, max) avoids double-counting shared vertices.
        let (lo, hi) = if a.y <= b.y { (a, b) } else { (b, a) };
        if y >= lo.y && y < hi.y {
            let t = (y - lo.y) / (hi.y - lo.y);
            xs.push(lo.x + t * (hi.x - lo.x));
        }
    }
    xs
}

fn rotate(p: Vec2, sin: f64, cos: f64) -> DVec2 {
    let x = f64::from(p.x);
    let y = f64::from(p.y);
    DVec2::new(cos * x + sin * y, -sin * x + cos * y)
}

fn unrotate(p: DVec2, sin: f64, cos: f64) -> Vec2 {
    Vec2::new(
        (cos * p.x - sin * p.y) as f32,
        (sin * p.x + cos * p.y) as f32,
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use ph2d_vector_doc::primitives;

    fn square() -> VectorNetwork {
        let mut net = primitives::rect(Vec2::new(0.0, 0.0), Vec2::new(100.0, 100.0));
        net.deterministic = true;
        net
    }

    #[test]
    fn horizontal_hatch_fills_the_square() {
        // 100px tall, spacing 10 → 9 interior scanlines, each one span (2 verts).
        let out = hatch(&square(), 0.0, 10.0);
        assert!(out.validate().is_ok());
        assert_eq!(out.segments.len(), 9, "9 interior hatch lines");
        assert_eq!(out.vertices.len(), 18);
        assert!(
            out.regions.is_empty(),
            "hatch emits open lines, not regions"
        );
    }

    #[test]
    fn hatch_lines_span_the_interior() {
        // Each horizontal line should run from x≈0 to x≈100.
        let out = hatch(&square(), 0.0, 25.0);
        for s in &out.segments {
            let a = out.vertices.iter().find(|v| v.id == s.start).unwrap().pos;
            let b = out.vertices.iter().find(|v| v.id == s.end).unwrap().pos;
            let span = (a.x - b.x).abs();
            assert!(
                span > 95.0,
                "interior span should ~cover the width, got {span}"
            );
        }
    }

    #[test]
    fn angled_hatch_is_valid() {
        let out = hatch(&square(), 45.0, 15.0);
        assert!(out.validate().is_ok());
        assert!(!out.segments.is_empty());
    }

    #[test]
    fn zero_spacing_is_empty() {
        let out = hatch(&square(), 0.0, 0.0);
        assert!(out.validate().is_ok());
        assert!(out.segments.is_empty());
    }

    #[test]
    fn deterministic_and_reproducible() {
        let out = hatch(&square(), 30.0, 12.0);
        assert!(out.deterministic);
        assert_eq!(out, hatch(&square(), 30.0, 12.0), "byte-stable");
    }

    #[test]
    fn region_less_input_is_empty() {
        let out = hatch(&VectorNetwork::empty(), 0.0, 10.0);
        assert!(out.validate().is_ok());
        assert!(out.segments.is_empty());
    }
}
