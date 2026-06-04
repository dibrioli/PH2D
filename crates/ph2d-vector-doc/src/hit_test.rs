//! Hit testing helpers for [`crate::VectorNetwork`].
//!
//! Per the W1 plan §2.1 module map + R4 audit Lens-G MED-G5:
//! Pen tool close-path detection ([`VectorNetwork::nearest_vertex`]),
//! Select tool marquee + click ([`VectorNetwork::bounding_box`] +
//! [`VectorNetwork::region_contains_point`]), Direct Select tangent grab
//! ([`VectorNetwork::nearest_tangent_handle`]).
//!
//! W1 shipped only the brute-force `nearest_vertex`; W2 (T2.3 Select /
//! Direct Select) adds the bounding-box, point-in-region, and tangent-
//! handle helpers below. All are O(N) linear scans — fine for typical
//! interactive documents on the editor write path (NOT the render tick).
//! A BVH spatial index for dense networks lands later via amendment when
//! a tool exercises it at scale.

use glam::Vec2;

use crate::cubic::{TangentSide, VertexId};
use crate::network::VectorNetwork;
use crate::region::{Region, WindingRule};
use crate::style::SegmentId;

/// Flatten resolution (chords per cubic segment) used by
/// [`VectorNetwork::region_contains_point`]. 16 chords keep a 90° arc
/// within ~0.1 % of the true curve — ample for click-select hit testing.
const DEFAULT_FLATTEN_STEPS: usize = 16;

/// Below this squared tangent-offset length a cubic handle is treated as
/// coincident with its vertex (a straight segment has a zero tangent) and
/// is **not** a grabbable handle — so [`VectorNetwork::nearest_tangent_handle`]
/// skips it and the vertex wins the grab.
const MIN_HANDLE_LEN_SQ: f32 = 1e-6;

impl VectorNetwork {
    /// Return the [`VertexId`] of the vertex closest to `p` within
    /// `tolerance` pixels (Euclidean distance), or `None` if every
    /// vertex is farther than the tolerance.
    ///
    /// **W1 close-path detection helper** (R4 audit Lens-G MED-G5).
    /// Linear scan O(N_vertices); cache the result across frames if
    /// calling per pointer event.
    ///
    /// Ties broken by lowest [`VertexId`] (deterministic for replay).
    #[must_use]
    pub fn nearest_vertex(&self, p: Vec2, tolerance: f32) -> Option<VertexId> {
        let tol_sq = tolerance * tolerance;
        let mut best: Option<(VertexId, f32)> = None;
        for v in &self.vertices {
            let d_sq = (v.pos - p).length_squared();
            if d_sq > tol_sq {
                continue;
            }
            best = match best {
                None => Some((v.id, d_sq)),
                Some((_, prev_d_sq)) if d_sq < prev_d_sq => Some((v.id, d_sq)),
                // Tie-break: keep the lower id for deterministic replay.
                Some((prev_id, prev_d_sq)) if d_sq == prev_d_sq && v.id < prev_id => {
                    Some((v.id, d_sq))
                }
                other => other,
            };
        }
        best.map(|(id, _)| id)
    }

    /// Axis-aligned bounding box `(min, max)` over every vertex **and**
    /// every cubic control point (so a segment bulging past its endpoints
    /// is still bounded). `None` for an empty network.
    ///
    /// **T2.3 Select** marquee helper: a network is window-selected when
    /// its box is contained in the marquee, crossing-selected when it
    /// intersects. O(V + S).
    #[must_use]
    pub fn bounding_box(&self) -> Option<(Vec2, Vec2)> {
        if self.vertices.is_empty() {
            return None;
        }
        let mut min = Vec2::splat(f32::INFINITY);
        let mut max = Vec2::splat(f32::NEG_INFINITY);
        let mut grow = |p: Vec2| {
            if p.is_finite() {
                min = min.min(p);
                max = max.max(p);
            }
        };
        for v in &self.vertices {
            grow(v.pos);
        }
        // Control points: c1 = start + out_at_start, c2 = end + in_at_end.
        for s in &self.segments {
            let (Some(start), Some(end)) = (self.vertex_pos(s.start), self.vertex_pos(s.end))
            else {
                continue;
            };
            grow(start + s.out_at_start);
            grow(end + s.in_at_end);
        }
        if min.x.is_finite() && max.x.is_finite() {
            Some((min, max))
        } else {
            None
        }
    }

    /// `true` if `p` lies inside `region` under its winding rule. Cubic
    /// segments are flattened to chords for the polygon test.
    ///
    /// **T2.3 Select** click helper (click inside a filled shape selects
    /// its network). Editor write path; O(S × flatten_steps).
    #[must_use]
    pub fn region_contains_point(&self, region: &Region, p: Vec2) -> bool {
        let boundary = self.flatten_region_boundary(region, DEFAULT_FLATTEN_STEPS);
        if boundary.len() < 3 {
            return false;
        }
        match region.winding {
            WindingRule::EvenOdd => point_in_polygon_even_odd(&boundary, p),
            WindingRule::NonZero => point_in_polygon_nonzero(&boundary, p),
        }
    }

    /// Nearest cubic tangent handle to `p` within `tolerance`, as
    /// `(segment id, which side)`. Handles sit at `start + out_at_start`
    /// and `end + in_at_end`.
    ///
    /// **T2.3 Direct Select** tangent-grab helper. Ties broken by lower
    /// `(segment id, OutAtStart < InAtEnd)` for determinism.
    #[must_use]
    pub fn nearest_tangent_handle(
        &self,
        p: Vec2,
        tolerance: f32,
    ) -> Option<(SegmentId, TangentSide)> {
        let tol_sq = tolerance * tolerance;
        let mut best: Option<(SegmentId, TangentSide, f32)> = None;
        for s in &self.segments {
            let (Some(start), Some(end)) = (self.vertex_pos(s.start), self.vertex_pos(s.end))
            else {
                continue;
            };
            for (side, offset, handle) in [
                (
                    TangentSide::OutAtStart,
                    s.out_at_start,
                    start + s.out_at_start,
                ),
                (TangentSide::InAtEnd, s.in_at_end, end + s.in_at_end),
            ] {
                // A (near-)zero tangent has no distinct handle — it sits on
                // the vertex (straight segment). Skip it so the vertex wins
                // the grab instead of a phantom coincident handle.
                if offset.length_squared() < MIN_HANDLE_LEN_SQ {
                    continue;
                }
                let d_sq = (handle - p).length_squared();
                if d_sq > tol_sq {
                    continue;
                }
                let better = match best {
                    None => true,
                    Some((_, _, prev)) => d_sq < prev,
                };
                if better {
                    best = Some((s.id, side, d_sq));
                }
            }
        }
        best.map(|(id, side, _)| (id, side))
    }

    /// Position of vertex `id`, if present. Linear scan (write path).
    fn vertex_pos(&self, id: VertexId) -> Option<Vec2> {
        self.vertices.iter().find(|v| v.id == id).map(|v| v.pos)
    }

    /// Flatten a region's boundary into a closed polyline of points.
    /// Walks `region.segments` in order, honoring per-ref direction, and
    /// samples each cubic into `steps` chords. Dangling refs end the walk
    /// (best-effort open boundary). The returned ring omits the duplicate
    /// closing point.
    fn flatten_region_boundary(&self, region: &Region, steps: usize) -> Vec<Vec2> {
        let steps = steps.max(1);
        let mut pts: Vec<Vec2> = Vec::new();
        for &(seg_id, forward) in &region.segments {
            let Some(seg) = self.segments.iter().find(|s| s.id == seg_id) else {
                break;
            };
            let (a_id, b_id, c1off, c2off) = if forward {
                (seg.start, seg.end, seg.out_at_start, seg.in_at_end)
            } else {
                (seg.end, seg.start, seg.in_at_end, seg.out_at_start)
            };
            let (Some(a), Some(b)) = (self.vertex_pos(a_id), self.vertex_pos(b_id)) else {
                break;
            };
            let c1 = a + c1off;
            let c2 = b + c2off;
            // Sample t in (0, 1]; the segment start equals the previous
            // segment's end, so we skip t=0 to avoid duplicate points.
            for k in 1..=steps {
                let t = k as f32 / steps as f32;
                pts.push(eval_cubic(a, c1, c2, b, t));
            }
        }
        pts
    }

    /// The [`SegmentId`] whose flattened cubic passes closest to `p`, if within
    /// `tolerance` (network-local units). For click-selecting OPEN stroked paths
    /// (Pencil / Spiral / open Pen) that have no fillable region — the region
    /// test alone can never hit them. Flattens each segment with the same
    /// `eval_cubic` + `DEFAULT_FLATTEN_STEPS` as the region boundary; O(S × steps)
    /// write path. Ties broken by lowest distance then insertion order.
    #[must_use]
    pub fn nearest_segment_within(&self, p: Vec2, tolerance: f32) -> Option<SegmentId> {
        let tol_sq = tolerance * tolerance;
        let mut best: Option<(SegmentId, f32)> = None;
        for seg in &self.segments {
            let (Some(a), Some(b)) = (self.vertex_pos(seg.start), self.vertex_pos(seg.end)) else {
                continue;
            };
            let c1 = a + seg.out_at_start;
            let c2 = b + seg.in_at_end;
            let mut prev = a;
            let mut min_sq = f32::INFINITY;
            for k in 1..=DEFAULT_FLATTEN_STEPS {
                let t = k as f32 / DEFAULT_FLATTEN_STEPS as f32;
                let cur = eval_cubic(a, c1, c2, b, t);
                min_sq = min_sq.min(point_to_segment_sq(p, prev, cur));
                prev = cur;
            }
            if min_sq <= tol_sq {
                let better = match best {
                    Some((_, d)) => min_sq < d,
                    None => true,
                };
                if better {
                    best = Some((seg.id, min_sq));
                }
            }
        }
        best.map(|(id, _)| id)
    }
}

/// Evaluate a cubic Bézier at `t`.
#[inline]
fn eval_cubic(p0: Vec2, p1: Vec2, p2: Vec2, p3: Vec2, t: f32) -> Vec2 {
    let mt = 1.0 - t;
    p0 * (mt * mt * mt) + p1 * (3.0 * mt * mt * t) + p2 * (3.0 * mt * t * t) + p3 * (t * t * t)
}

/// Squared distance from `p` to the segment `a`–`b` (clamped projection).
#[inline]
fn point_to_segment_sq(p: Vec2, a: Vec2, b: Vec2) -> f32 {
    let ab = b - a;
    let len_sq = ab.length_squared();
    if len_sq < 1e-12 {
        return (p - a).length_squared();
    }
    let t = ((p - a).dot(ab) / len_sq).clamp(0.0, 1.0);
    (p - (a + ab * t)).length_squared()
}

/// Even-odd (parity) point-in-polygon ray cast (+x ray).
fn point_in_polygon_even_odd(poly: &[Vec2], p: Vec2) -> bool {
    let mut inside = false;
    let n = poly.len();
    let mut j = n - 1;
    for i in 0..n {
        let (a, b) = (poly[i], poly[j]);
        if (a.y > p.y) != (b.y > p.y) {
            let x_cross = a.x + (p.y - a.y) / (b.y - a.y) * (b.x - a.x);
            if p.x < x_cross {
                inside = !inside;
            }
        }
        j = i;
    }
    inside
}

/// Non-zero winding-number point-in-polygon test.
fn point_in_polygon_nonzero(poly: &[Vec2], p: Vec2) -> bool {
    let mut winding: i32 = 0;
    let n = poly.len();
    for i in 0..n {
        let a = poly[i];
        let b = poly[(i + 1) % n];
        if a.y <= p.y {
            if b.y > p.y && is_left(a, b, p) > 0.0 {
                winding += 1;
            }
        } else if b.y <= p.y && is_left(a, b, p) < 0.0 {
            winding -= 1;
        }
    }
    winding != 0
}

/// > 0 if `p` is left of the directed line `a→b`, < 0 if right, 0 on it.
#[inline]
fn is_left(a: Vec2, b: Vec2, p: Vec2) -> f32 {
    (b.x - a.x) * (p.y - a.y) - (p.x - a.x) * (b.y - a.y)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cubic::Vertex;
    use crate::region::{Region, SegmentRef};
    use crate::style::Segment;

    fn make_net_with_3_vertices() -> VectorNetwork {
        let mut net = VectorNetwork::empty();
        net.vertices.push(Vertex::auto(0, Vec2::new(0.0, 0.0)));
        net.vertices.push(Vertex::auto(1, Vec2::new(100.0, 0.0)));
        net.vertices.push(Vertex::auto(2, Vec2::new(50.0, 86.6)));
        net
    }

    #[test]
    fn nearest_segment_within_hits_open_stroke_body() {
        // An open 2-vertex stroke (no region). A click near the segment BODY
        // (not a vertex) is detected within tolerance; a far click is not.
        let mut net = VectorNetwork::empty();
        net.vertices.push(Vertex::auto(0, Vec2::new(0.0, 0.0)));
        net.vertices.push(Vertex::auto(1, Vec2::new(100.0, 0.0)));
        net.segments.push(Segment::straight(0, 0, 1));
        assert_eq!(
            net.nearest_segment_within(Vec2::new(50.0, 2.0), 5.0),
            Some(0)
        );
        assert_eq!(net.nearest_segment_within(Vec2::new(50.0, 50.0), 5.0), None);
    }

    /// Closed triangle network with a NonZero region over its 3 edges.
    fn triangle_region() -> VectorNetwork {
        let mut net = make_net_with_3_vertices();
        net.segments.push(Segment::straight(0, 0, 1));
        net.segments.push(Segment::straight(1, 1, 2));
        net.segments.push(Segment::straight(2, 2, 0));
        let mut r = Region::new(0, WindingRule::NonZero);
        r.segments = [(0u32, true), (1, true), (2, true)]
            .into_iter()
            .map(|x| x as SegmentRef)
            .collect();
        net.regions.push(r);
        net
    }

    #[test]
    fn nearest_vertex_returns_closest_within_tolerance() {
        let net = make_net_with_3_vertices();
        assert_eq!(net.nearest_vertex(Vec2::new(1.0, 1.0), 5.0), Some(0));
        assert_eq!(net.nearest_vertex(Vec2::new(99.0, 1.0), 5.0), Some(1));
    }

    #[test]
    fn nearest_vertex_returns_none_when_all_out_of_tolerance() {
        let net = make_net_with_3_vertices();
        assert_eq!(net.nearest_vertex(Vec2::new(500.0, 500.0), 5.0), None);
    }

    #[test]
    fn nearest_vertex_handles_empty_network() {
        let net = VectorNetwork::empty();
        assert_eq!(net.nearest_vertex(Vec2::ZERO, 100.0), None);
    }

    #[test]
    fn nearest_vertex_tie_break_favors_lower_id() {
        let mut net = VectorNetwork::empty();
        net.vertices.push(Vertex::auto(5, Vec2::ZERO));
        net.vertices.push(Vertex::auto(2, Vec2::ZERO));
        net.vertices.push(Vertex::auto(7, Vec2::ZERO));
        assert_eq!(net.nearest_vertex(Vec2::ZERO, 1.0), Some(2));
    }

    #[test]
    fn bounding_box_covers_vertices() {
        let net = make_net_with_3_vertices();
        let (min, max) = net.bounding_box().expect("bbox");
        assert_eq!(min, Vec2::new(0.0, 0.0));
        assert_eq!(max, Vec2::new(100.0, 86.6));
    }

    #[test]
    fn bounding_box_includes_control_points() {
        // A single segment whose control points bulge well past the
        // endpoints must widen the box.
        let mut net = VectorNetwork::empty();
        net.vertices.push(Vertex::auto(0, Vec2::new(0.0, 0.0)));
        net.vertices.push(Vertex::auto(1, Vec2::new(10.0, 0.0)));
        let mut seg = Segment::straight(0, 0, 1);
        seg.out_at_start = Vec2::new(0.0, 50.0); // bulge up
        seg.in_at_end = Vec2::new(0.0, 50.0); // bulge down (from end)
        net.segments.push(seg);
        let (min, max) = net.bounding_box().expect("bbox");
        assert!(max.y >= 50.0, "control point should widen bbox: {max:?}");
        assert_eq!(min.x, 0.0);
        assert_eq!(max.x, 10.0);
    }

    #[test]
    fn bounding_box_empty_is_none() {
        assert!(VectorNetwork::empty().bounding_box().is_none());
    }

    #[test]
    fn region_contains_point_inside_and_outside() {
        let net = triangle_region();
        let region = &net.regions[0];
        // Centroid is inside.
        assert!(net.region_contains_point(region, Vec2::new(50.0, 30.0)));
        // Well outside.
        assert!(!net.region_contains_point(region, Vec2::new(200.0, 200.0)));
        assert!(!net.region_contains_point(region, Vec2::new(0.0, 80.0)));
    }

    #[test]
    fn region_contains_point_rect_evenodd() {
        // Axis-aligned square via 4 straight segments, EvenOdd.
        let mut net = VectorNetwork::empty();
        for (i, p) in [
            Vec2::new(0.0, 0.0),
            Vec2::new(10.0, 0.0),
            Vec2::new(10.0, 10.0),
            Vec2::new(0.0, 10.0),
        ]
        .into_iter()
        .enumerate()
        {
            net.vertices.push(Vertex::auto(i as u32, p));
        }
        for i in 0..4u32 {
            net.segments.push(Segment::straight(i, i, (i + 1) % 4));
        }
        let mut r = Region::new(0, WindingRule::EvenOdd);
        r.segments = (0..4u32).map(|i| (i, true) as SegmentRef).collect();
        net.regions.push(r);
        let region = &net.regions[0];
        assert!(net.region_contains_point(region, Vec2::new(5.0, 5.0)));
        assert!(!net.region_contains_point(region, Vec2::new(15.0, 5.0)));
    }

    #[test]
    fn nearest_tangent_handle_grabs_the_right_side() {
        let mut net = VectorNetwork::empty();
        net.vertices.push(Vertex::auto(0, Vec2::new(0.0, 0.0)));
        net.vertices.push(Vertex::auto(1, Vec2::new(100.0, 0.0)));
        let mut seg = Segment::straight(7, 0, 1);
        seg.out_at_start = Vec2::new(20.0, 10.0); // handle at (20,10)
        seg.in_at_end = Vec2::new(-20.0, 10.0); // handle at (80,10)
        net.segments.push(seg);
        assert_eq!(
            net.nearest_tangent_handle(Vec2::new(21.0, 11.0), 5.0),
            Some((7, TangentSide::OutAtStart))
        );
        assert_eq!(
            net.nearest_tangent_handle(Vec2::new(79.0, 9.0), 5.0),
            Some((7, TangentSide::InAtEnd))
        );
        assert_eq!(net.nearest_tangent_handle(Vec2::new(50.0, 50.0), 5.0), None);
    }

    #[test]
    fn nearest_tangent_handle_skips_zero_length_handles() {
        // A straight segment (zero tangents) has handles coincident with
        // its vertices — they must NOT be grabbable, so a point right on a
        // vertex finds no handle (the vertex grab wins upstream).
        let mut net = VectorNetwork::empty();
        net.vertices.push(Vertex::auto(0, Vec2::new(0.0, 0.0)));
        net.vertices.push(Vertex::auto(1, Vec2::new(100.0, 0.0)));
        net.segments.push(Segment::straight(0, 0, 1)); // zero tangents
        assert_eq!(net.nearest_tangent_handle(Vec2::new(0.0, 0.0), 5.0), None);
        assert_eq!(net.nearest_tangent_handle(Vec2::new(100.0, 0.0), 5.0), None);
    }
}
