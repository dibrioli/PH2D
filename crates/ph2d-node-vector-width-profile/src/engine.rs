//! The width-profile engine: turn each open path into a **filled band** whose
//! width tapers linearly from `width_start` to `width_end` along its arc length
//! (a variable-width stroke, ADR-0058 §2.2.9). Each open chain → one region;
//! curved segments are flattened to a polyline first.
//!
//! ## v1 scope
//!
//! A **linear** start→end taper. A full editable profile curve (multiple width
//! stops along the path) is a follow-up — the scalar param vocabulary carries
//! two endpoints today. Closed-region inputs (no open chains) pass through to an
//! empty result (width-profile is a stroke operation).
//!
//! Determinism: fixed flatten count; Q16.16-snapped output when the input is
//! `deterministic`.

use std::collections::{BTreeMap, BTreeSet};

use glam::{DVec2, Vec2};
use ph2d_vector_doc::deterministic::snap;
use ph2d_vector_doc::{Region, Segment, SegmentId, VectorNetwork, Vertex, VertexId, WindingRule};

/// Samples per curved segment when flattening (straight segments need none).
const FLATTEN: usize = 8;

/// Stroke each open path of `input` with a linear width taper.
#[must_use]
pub fn width_profile(input: &VectorNetwork, width_start: f32, width_end: f32) -> VectorNetwork {
    let verts: BTreeMap<VertexId, Vec2> = input.vertices.iter().map(|v| (v.id, v.pos)).collect();
    let region_segs = region_segment_ids(input);
    let mut b = Builder::new();
    b.net.deterministic = input.deterministic;

    for chain in open_chains(input, &region_segs) {
        let poly = flatten_chain(&chain, input, &verts);
        if poly.len() < 2 {
            continue;
        }
        b.emit_band(&poly, f64::from(width_start), f64::from(width_end));
    }

    if input.deterministic {
        snap_network(&mut b.net);
    }
    b.net
}

fn region_segment_ids(net: &VectorNetwork) -> BTreeSet<SegmentId> {
    let mut s = BTreeSet::new();
    for r in &net.regions {
        for (sid, _) in &r.segments {
            s.insert(*sid);
        }
    }
    s
}

/// Greedy adjacency stitch of the non-region segments into ordered chains.
fn open_chains(
    net: &VectorNetwork,
    region_segs: &BTreeSet<SegmentId>,
) -> Vec<Vec<(SegmentId, bool)>> {
    let seg_by_id: BTreeMap<SegmentId, &Segment> = net
        .segments
        .iter()
        .filter(|s| !region_segs.contains(&s.id))
        .map(|s| (s.id, s))
        .collect();
    if seg_by_id.is_empty() {
        return Vec::new();
    }
    let mut adj: BTreeMap<VertexId, Vec<SegmentId>> = BTreeMap::new();
    for s in seg_by_id.values() {
        adj.entry(s.start).or_default().push(s.id);
        adj.entry(s.end).or_default().push(s.id);
    }
    let mut visited: BTreeSet<SegmentId> = BTreeSet::new();
    let mut chains = Vec::new();
    let walk = |start: VertexId, visited: &mut BTreeSet<SegmentId>| {
        let mut chain = Vec::new();
        let mut cur = start;
        while let Some(&next) = adj
            .get(&cur)
            .and_then(|v| v.iter().find(|s| !visited.contains(s)))
        {
            visited.insert(next);
            let seg = seg_by_id[&next];
            let fwd = seg.start == cur;
            chain.push((next, fwd));
            cur = if fwd { seg.end } else { seg.start };
        }
        chain
    };
    let endpoints: Vec<VertexId> = adj
        .iter()
        .filter(|(_, v)| v.len() == 1)
        .map(|(v, _)| *v)
        .collect();
    for v in endpoints {
        let c = walk(v, &mut visited);
        if !c.is_empty() {
            chains.push(c);
        }
    }
    for s in seg_by_id.values() {
        if !visited.contains(&s.id) {
            let c = walk(s.start, &mut visited);
            if !c.is_empty() {
                chains.push(c);
            }
        }
    }
    chains
}

/// Flatten an ordered chain into a polyline (sample curved segments).
fn flatten_chain(
    chain: &[(SegmentId, bool)],
    net: &VectorNetwork,
    verts: &BTreeMap<VertexId, Vec2>,
) -> Vec<DVec2> {
    let seg_by_id: BTreeMap<SegmentId, &Segment> = net.segments.iter().map(|s| (s.id, s)).collect();
    let mut pts: Vec<DVec2> = Vec::new();
    for &(sid, fwd) in chain {
        let Some(seg) = seg_by_id.get(&sid) else {
            continue;
        };
        let (s_id, e_id, out, inn) = if fwd {
            (seg.start, seg.end, seg.out_at_start, seg.in_at_end)
        } else {
            (seg.end, seg.start, seg.in_at_end, seg.out_at_start)
        };
        let (Some(&sp), Some(&ep)) = (verts.get(&s_id), verts.get(&e_id)) else {
            continue;
        };
        let p0 = sp.as_dvec2();
        let p3 = ep.as_dvec2();
        let curved = out != Vec2::ZERO || inn != Vec2::ZERO;
        if pts.is_empty() {
            pts.push(p0);
        }
        if curved {
            let c1 = (sp + out).as_dvec2();
            let c2 = (ep + inn).as_dvec2();
            for i in 1..=FLATTEN {
                let t = i as f64 / FLATTEN as f64;
                pts.push(cubic(p0, c1, c2, p3, t));
            }
        } else {
            pts.push(p3);
        }
    }
    pts
}

fn cubic(p0: DVec2, c1: DVec2, c2: DVec2, p3: DVec2, t: f64) -> DVec2 {
    let u = 1.0 - t;
    p0 * (u * u * u) + c1 * (3.0 * u * u * t) + c2 * (3.0 * u * t * t) + p3 * (t * t * t)
}

struct Builder {
    net: VectorNetwork,
    next_v: VertexId,
    next_s: SegmentId,
    next_r: RegionId,
}
type RegionId = u32;

impl Builder {
    fn new() -> Self {
        Self {
            net: VectorNetwork::empty(),
            next_v: 0,
            next_s: 0,
            next_r: 0,
        }
    }

    /// Build a filled band around `poly` tapering `w_start`→`w_end`.
    fn emit_band(&mut self, poly: &[DVec2], w_start: f64, w_end: f64) {
        let n = poly.len();
        // Cumulative arc length → param t per point.
        let mut len = vec![0.0_f64; n];
        for i in 1..n {
            len[i] = len[i - 1] + (poly[i] - poly[i - 1]).length();
        }
        let total = len[n - 1];
        if total <= 1e-9 {
            return;
        }
        let half = |t: f64| (w_start + (w_end - w_start) * t) * 0.5;
        // Left / right offset points.
        let mut left = Vec::with_capacity(n);
        let mut right = Vec::with_capacity(n);
        for i in 0..n {
            let dir = if i == 0 {
                poly[1] - poly[0]
            } else if i == n - 1 {
                poly[n - 1] - poly[n - 2]
            } else {
                poly[i + 1] - poly[i - 1]
            };
            let nrm = {
                let perp = DVec2::new(-dir.y, dir.x);
                let l = perp.length();
                if l > 1e-12 { perp / l } else { DVec2::ZERO }
            };
            let h = half(len[i] / total);
            left.push(poly[i] + nrm * h);
            right.push(poly[i] - nrm * h);
        }
        // Vertices: left[0..n] then right[0..n].
        let lv: Vec<VertexId> = left.iter().map(|&p| self.push_vertex(p)).collect();
        let rv: Vec<VertexId> = right.iter().map(|&p| self.push_vertex(p)).collect();
        // Closed loop: left forward, end cap, right backward, start cap.
        let mut refs: Vec<(SegmentId, bool)> = Vec::with_capacity(2 * n);
        for i in 0..n - 1 {
            refs.push((self.push_straight(lv[i], lv[i + 1]), true));
        }
        refs.push((self.push_straight(lv[n - 1], rv[n - 1]), true));
        for i in (1..n).rev() {
            refs.push((self.push_straight(rv[i], rv[i - 1]), true));
        }
        refs.push((self.push_straight(rv[0], lv[0]), true));
        let rid = self.next_r;
        self.next_r += 1;
        let mut region = Region::new(rid, WindingRule::NonZero);
        region.segments = refs.into_iter().collect();
        self.net.regions.push(region);
    }

    fn push_vertex(&mut self, p: DVec2) -> VertexId {
        let id = self.next_v;
        self.next_v += 1;
        self.net
            .vertices
            .push(Vertex::auto(id, Vec2::new(p.x as f32, p.y as f32)));
        id
    }

    fn push_straight(&mut self, a: VertexId, b: VertexId) -> SegmentId {
        let id = self.next_s;
        self.next_s += 1;
        self.net.segments.push(Segment::straight(id, a, b));
        id
    }
}

fn snap_network(net: &mut VectorNetwork) {
    for v in net.vertices.iter_mut() {
        v.pos = Vec2::new(snap(v.pos.x), snap(v.pos.y));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// An open horizontal polyline (0,0)→(50,0)→(100,0), no region.
    fn open_line() -> VectorNetwork {
        let mut net = VectorNetwork::empty();
        net.deterministic = true;
        net.vertices.push(Vertex::auto(0, Vec2::new(0.0, 0.0)));
        net.vertices.push(Vertex::auto(1, Vec2::new(50.0, 0.0)));
        net.vertices.push(Vertex::auto(2, Vec2::new(100.0, 0.0)));
        net.segments.push(Segment::straight(0, 0, 1));
        net.segments.push(Segment::straight(1, 1, 2));
        net
    }

    #[test]
    fn taper_makes_a_filled_band() {
        let out = width_profile(&open_line(), 2.0, 20.0);
        assert!(out.validate().is_ok());
        assert_eq!(out.regions.len(), 1, "one open path → one band");
        assert!(!out.regions[0].segments.is_empty());
    }

    #[test]
    fn band_is_wider_at_the_end() {
        // Start half-width 1, end half-width 10 → the band's vertical extent
        // grows along x.
        let out = width_profile(&open_line(), 2.0, 20.0);
        // Find the max |y| near the start (x≈0) vs the end (x≈100).
        let near = |x: f32| {
            out.vertices
                .iter()
                .filter(|v| (v.pos.x - x).abs() < 1.0)
                .map(|v| v.pos.y.abs())
                .fold(0.0_f32, f32::max)
        };
        assert!(near(0.0) < 2.0, "thin at the start");
        assert!(near(100.0) > 8.0, "thick at the end");
    }

    #[test]
    fn region_input_yields_empty() {
        use ph2d_vector_doc::primitives;
        let mut net = primitives::rect(Vec2::new(0.0, 0.0), Vec2::new(10.0, 10.0));
        net.deterministic = true;
        let out = width_profile(&net, 2.0, 4.0);
        assert!(out.validate().is_ok());
        assert!(out.regions.is_empty(), "closed regions aren't strokes");
    }

    #[test]
    fn deterministic_and_reproducible() {
        let out = width_profile(&open_line(), 2.0, 20.0);
        assert!(out.deterministic);
        assert_eq!(out, width_profile(&open_line(), 2.0, 20.0), "byte-stable");
    }

    #[test]
    fn empty_input_is_empty() {
        let out = width_profile(&VectorNetwork::empty(), 2.0, 4.0);
        assert!(out.validate().is_ok());
        assert!(out.regions.is_empty());
    }
}
