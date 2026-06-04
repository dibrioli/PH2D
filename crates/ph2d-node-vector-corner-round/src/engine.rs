//! The corner-round engine: replace each sharp corner of a region with a fillet
//! arc of the given `radius` (approximated by one cubic Bézier per corner).
//!
//! ## v1 scope
//!
//! Each region boundary is treated as the **polygon** through its vertices in
//! loop order (straight edges) — the sharp-cornered case this node targets
//! (rect / polygon / star / boolean results). Curve-preserving fillets (a
//! De Casteljau split of curved input edges) are a follow-up; a curved input is
//! polygonised at its vertices here. Open paths (region-less networks) pass
//! through unchanged.
//!
//! ## Determinism
//!
//! The fillet math uses `f64` trig; the result is snapped to the Q16.16 grid
//! when the input is `deterministic`.

use std::collections::BTreeMap;

use glam::{DVec2, Vec2};
use ph2d_vector_doc::deterministic::snap;
use ph2d_vector_doc::{Region, RegionId, Segment, SegmentId, VectorNetwork, Vertex, VertexId};

/// Cubic handle length, as a fraction of the tangent setback, for the fillet
/// arc. `0.5523` is the exact constant for a 90° arc and a good approximation
/// across the corner range typical shapes present.
const KAPPA: f64 = 0.5522847498307933;

/// Round every region corner of `input` with fillet `radius`.
#[must_use]
pub fn corner_round(input: &VectorNetwork, radius: f32) -> VectorNetwork {
    if radius <= 0.0 || !radius.is_finite() || input.regions.is_empty() {
        return input.clone();
    }
    let verts: BTreeMap<VertexId, Vec2> = input.vertices.iter().map(|v| (v.id, v.pos)).collect();
    let mut b = Builder::new();

    for region in &input.regions {
        let Some(loop_pts) = region_loop(region, &input.segments, &verts) else {
            // Dangling ref → keep the region's vertices but skip filleting it.
            continue;
        };
        if loop_pts.len() < 3 {
            continue;
        }
        b.fillet_region(
            &loop_pts,
            f64::from(radius),
            region.winding,
            region.fill,
            region.z,
        );
    }

    if input.deterministic {
        snap_network(&mut b.net);
    }
    b.net.deterministic = input.deterministic;
    b.net
}

/// The ordered loop of vertex positions of a region (edge `i` = `pts[i]`→
/// `pts[i+1]`). `None` if a segment/vertex ref dangles.
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

struct Builder {
    net: VectorNetwork,
    next_v: VertexId,
    next_s: SegmentId,
    next_r: RegionId,
}

impl Builder {
    fn new() -> Self {
        Self {
            net: VectorNetwork::empty(),
            next_v: 0,
            next_s: 0,
            next_r: 0,
        }
    }

    fn push_vertex(&mut self, p: Vec2) -> VertexId {
        let id = self.next_v;
        self.next_v += 1;
        self.net.vertices.push(Vertex::auto(id, p));
        id
    }

    fn push_segment(
        &mut self,
        start: VertexId,
        end: VertexId,
        out_at: Vec2,
        in_at: Vec2,
    ) -> SegmentId {
        let id = self.next_s;
        self.next_s += 1;
        let mut s = Segment::straight(id, start, end);
        s.out_at_start = out_at;
        s.in_at_end = in_at;
        self.net.segments.push(s);
        id
    }

    /// Fillet every corner of the polygon `pts` and emit the resulting region:
    /// per corner an arc `P1→P2`, then a straight edge to the next corner's P1.
    fn fillet_region(
        &mut self,
        pts: &[Vec2],
        radius: f64,
        winding: ph2d_vector_doc::WindingRule,
        fill: Option<u32>,
        z: i32,
    ) {
        let m = pts.len();
        // Per corner: the two tangent points (P1 in, P2 out) + the arc handles.
        struct Corner {
            p1: Vec2,
            p2: Vec2,
            c1: Vec2,
            c2: Vec2,
        }
        let mut corners: Vec<Corner> = Vec::with_capacity(m);
        for i in 0..m {
            let prev = pts[(i + m - 1) % m].as_dvec2();
            let cur = pts[i].as_dvec2();
            let next = pts[(i + 1) % m].as_dvec2();
            let in_vec = cur - prev;
            let out_vec = next - cur;
            let in_len = in_vec.length();
            let out_len = out_vec.length();
            if in_len < 1e-9 || out_len < 1e-9 {
                // Degenerate edge → no fillet; tangent points collapse onto the
                // corner (a zero-radius "fillet").
                corners.push(Corner {
                    p1: pts[i],
                    p2: pts[i],
                    c1: pts[i],
                    c2: pts[i],
                });
                continue;
            }
            let in_dir = in_vec / in_len;
            let out_dir = out_vec / out_len;
            // Interior angle φ between (−in_dir) and out_dir.
            let cos_phi = (-in_dir).dot(out_dir).clamp(-1.0, 1.0);
            let phi = cos_phi.acos();
            // Setback t = r / tan(φ/2), clamped so neighbouring fillets can't
            // overlap (≤ ~half each adjacent edge).
            let half = (phi / 2.0).tan().max(1e-6);
            let mut t = radius / half;
            t = t.min(0.49 * in_len).min(0.49 * out_len);
            let p1 = cur - in_dir * t;
            let p2 = cur + out_dir * t;
            let h = t * KAPPA;
            let c1 = p1 + in_dir * h;
            let c2 = p2 - out_dir * h;
            corners.push(Corner {
                p1: dvec_to_vec2(p1),
                p2: dvec_to_vec2(p2),
                c1: dvec_to_vec2(c1),
                c2: dvec_to_vec2(c2),
            });
        }

        // Emit vertices P1_i, P2_i; arc segment P1_i→P2_i; edge P2_i→P1_{i+1}.
        let v1: Vec<VertexId> = corners.iter().map(|c| self.push_vertex(c.p1)).collect();
        let v2: Vec<VertexId> = corners.iter().map(|c| self.push_vertex(c.p2)).collect();
        let mut refs: Vec<(SegmentId, bool)> = Vec::with_capacity(2 * m);
        for i in 0..m {
            let c = &corners[i];
            // Arc P1_i → P2_i (cubic; tangents relative to endpoints).
            let arc = self.push_segment(v1[i], v2[i], c.c1 - c.p1, c.c2 - c.p2);
            refs.push((arc, true));
            // Straight edge P2_i → P1_{i+1}.
            let edge = self.push_segment(v2[i], v1[(i + 1) % m], Vec2::ZERO, Vec2::ZERO);
            refs.push((edge, true));
        }
        let rid = self.next_r;
        self.next_r += 1;
        let mut r = Region::new(rid, winding);
        r.segments = refs.into_iter().collect();
        r.fill = fill;
        r.z = z;
        self.net.regions.push(r);
    }
}

fn dvec_to_vec2(d: DVec2) -> Vec2 {
    Vec2::new(d.x as f32, d.y as f32)
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
        let mut net = primitives::rect(Vec2::new(0.0, 0.0), Vec2::new(100.0, 100.0));
        net.deterministic = true;
        net
    }

    #[test]
    fn fillets_each_corner_into_arc_plus_edge() {
        // 4 corners → 4 arcs + 4 edges = 8 segments, 8 vertices (P1,P2 per corner).
        let out = corner_round(&square(), 10.0);
        assert!(out.validate().is_ok());
        assert_eq!(out.regions.len(), 1);
        assert_eq!(out.segments.len(), 8);
        assert_eq!(out.vertices.len(), 8);
    }

    #[test]
    fn rounded_corner_pulls_in_from_the_sharp_corner() {
        // No vertex should sit exactly on the original (0,0) corner — it's been
        // replaced by two tangent points set back along the edges.
        let out = corner_round(&square(), 10.0);
        assert!(
            !out.vertices.iter().any(|v| v.pos == Vec2::new(0.0, 0.0)),
            "the sharp corner must be replaced by the fillet"
        );
        // The tangent points sit 10px in from the corner along each edge.
        assert!(out.vertices.iter().any(|v| v.pos == Vec2::new(10.0, 0.0)));
        assert!(out.vertices.iter().any(|v| v.pos == Vec2::new(0.0, 10.0)));
    }

    #[test]
    fn radius_is_clamped_so_fillets_dont_overlap() {
        // A huge radius on a 100px square clamps to ~half the edge → still valid.
        let out = corner_round(&square(), 10_000.0);
        assert!(out.validate().is_ok());
        assert_eq!(out.regions.len(), 1);
    }

    #[test]
    fn zero_radius_is_identity() {
        assert_eq!(corner_round(&square(), 0.0), square());
    }

    #[test]
    fn deterministic_and_reproducible() {
        let out = corner_round(&square(), 12.0);
        assert!(out.deterministic);
        assert_eq!(out, corner_round(&square(), 12.0), "byte-stable");
    }

    #[test]
    fn region_less_input_is_identity() {
        let out = corner_round(&VectorNetwork::empty(), 10.0);
        assert!(out.validate().is_ok());
        assert!(out.regions.is_empty());
    }
}
