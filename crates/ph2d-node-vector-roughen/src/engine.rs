//! The roughen engine: subdivide every segment into `detail` straight spans and
//! displace each interior sample along the curve **normal** by deterministic
//! hash-noise scaled by `amplitude`. Endpoints are never moved (so shared edges
//! and closed loops stay stitched); a segment is seeded by its **id**, so an
//! edge shared by two regions roughens identically on both sides.
//!
//! ## Determinism (cross-OS golden)
//!
//! The noise is a pure integer hash of `(seed, segment_id, sample_index)` mapped
//! to `[-1, 1]` — bit-identical on every target (no `rand`, no global state,
//! ADR-0056 §2.7 / handoff §2). The cubic sampling runs in `f64`; the result is
//! snapped to the Q16.16 grid when the input is `deterministic`, erasing the
//! last-ULP drift.

use std::collections::BTreeMap;

use glam::{DVec2, Vec2};
use ph2d_vector_doc::deterministic::snap;
use ph2d_vector_doc::{
    Region, RegionId, Segment, SegmentId, VectorNetwork, Vertex, VertexId, WindingRule,
};

/// Allocation ceiling for `detail` — bounds the untrusted `f32 → count`
/// subdivision against an OOM value. Geometry stays sane well below this.
pub const MAX_DETAIL: usize = 1 << 10;

/// Roughen `input`: each segment becomes `detail` straight spans with interior
/// samples jittered along the normal by up to `amplitude`, seeded by `seed`.
#[must_use]
pub fn roughen(input: &VectorNetwork, amplitude: f32, detail: usize, seed: u32) -> VectorNetwork {
    // `detail <= 1` (or zero amplitude) means "no interior samples" → identity.
    let n = detail.min(MAX_DETAIL);
    if n < 2 || amplitude == 0.0 || !amplitude.is_finite() {
        return input.clone();
    }
    let verts: BTreeMap<VertexId, Vec2> = input.vertices.iter().map(|v| (v.id, v.pos)).collect();
    let mut b = Builder::new();

    // 1. Carry over original vertices (endpoints stay put).
    let mut vmap: BTreeMap<VertexId, VertexId> = BTreeMap::new();
    for v in &input.vertices {
        vmap.insert(v.id, b.push_vertex(v.pos));
    }

    // 2. Subdivide each segment into a chain of straight spans, displacing the
    //    interior samples. Record old-segment → forward chain of new spans.
    let mut chains: BTreeMap<SegmentId, Vec<SegmentId>> = BTreeMap::new();
    for s in &input.segments {
        let (Some(&p0), Some(&p3)) = (verts.get(&s.start), verts.get(&s.end)) else {
            continue;
        };
        let c1 = p0 + s.out_at_start;
        let c2 = p3 + s.in_at_end;
        let (Some(&nstart), Some(&nend)) = (vmap.get(&s.start), vmap.get(&s.end)) else {
            continue;
        };
        let chain = b.subdivide(
            s.id,
            p0,
            c1,
            c2,
            p3,
            n,
            amplitude,
            seed,
            nstart,
            nend,
            s.style_ref,
        );
        chains.insert(s.id, chain);
    }

    // 3. Rebuild regions: expand each segment ref to its span chain (reversed
    //    when the ref is backward).
    for r in &input.regions {
        let mut segs = Vec::new();
        for &(sid, fwd) in &r.segments {
            let Some(chain) = chains.get(&sid) else {
                continue;
            };
            if fwd {
                segs.extend(chain.iter().map(|&s| (s, true)));
            } else {
                segs.extend(chain.iter().rev().map(|&s| (s, false)));
            }
        }
        if !segs.is_empty() {
            b.push_region(r.winding, segs, r.fill, r.z);
        }
    }

    if input.deterministic {
        snap_network(&mut b.net);
    }
    b.net.deterministic = input.deterministic;
    b.net
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

    fn push_straight(&mut self, start: VertexId, end: VertexId, style: Option<u32>) -> SegmentId {
        let id = self.next_s;
        self.next_s += 1;
        let mut s = Segment::straight(id, start, end);
        s.style_ref = style;
        self.net.segments.push(s);
        id
    }

    fn push_region(
        &mut self,
        winding: WindingRule,
        segs: Vec<(SegmentId, bool)>,
        fill: Option<u32>,
        z: i32,
    ) {
        let id = self.next_r;
        self.next_r += 1;
        let mut r = Region::new(id, winding);
        r.segments = segs.into_iter().collect();
        r.fill = fill;
        r.z = z;
        self.net.regions.push(r);
    }

    /// Subdivide one cubic into `n` straight spans, displacing interior samples
    /// along the normal. Returns the forward chain of span ids.
    #[allow(clippy::too_many_arguments)]
    fn subdivide(
        &mut self,
        seg_id: SegmentId,
        p0: Vec2,
        c1: Vec2,
        c2: Vec2,
        p3: Vec2,
        n: usize,
        amplitude: f32,
        seed: u32,
        nstart: VertexId,
        nend: VertexId,
        style: Option<u32>,
    ) -> Vec<SegmentId> {
        let mut chain = Vec::with_capacity(n);
        let mut prev = nstart;
        for i in 1..n {
            let t = i as f64 / n as f64;
            let pos = cubic_point(p0, c1, c2, p3, t);
            let nrm = cubic_normal(p0, c1, c2, p3, t);
            let amount = f64::from(amplitude) * noise(seed, seg_id, i as u32);
            let displaced = pos + nrm * amount;
            let v = self.push_vertex(Vec2::new(displaced.x as f32, displaced.y as f32));
            chain.push(self.push_straight(prev, v, style));
            prev = v;
        }
        chain.push(self.push_straight(prev, nend, style));
        chain
    }
}

fn cubic_point(p0: Vec2, c1: Vec2, c2: Vec2, p3: Vec2, t: f64) -> DVec2 {
    let (p0, c1, c2, p3) = (p0.as_dvec2(), c1.as_dvec2(), c2.as_dvec2(), p3.as_dvec2());
    let u = 1.0 - t;
    p0 * (u * u * u) + c1 * (3.0 * u * u * t) + c2 * (3.0 * u * t * t) + p3 * (t * t * t)
}

/// Unit normal of the cubic at `t` (perp of the tangent). Degenerate tangents
/// fall back to `(0,0)` so the sample isn't displaced (no NaN).
fn cubic_normal(p0: Vec2, c1: Vec2, c2: Vec2, p3: Vec2, t: f64) -> DVec2 {
    let (p0, c1, c2, p3) = (p0.as_dvec2(), c1.as_dvec2(), c2.as_dvec2(), p3.as_dvec2());
    let u = 1.0 - t;
    let tangent = (c1 - p0) * (3.0 * u * u) + (c2 - c1) * (6.0 * u * t) + (p3 - c2) * (3.0 * t * t);
    let perp = DVec2::new(-tangent.y, tangent.x);
    let len = perp.length();
    if len > 1e-12 { perp / len } else { DVec2::ZERO }
}

/// Deterministic signed noise in `[-1, 1)` from `(seed, segment, sample)`.
/// Integer hash (no float drift) → exact cross-OS.
fn noise(seed: u32, segment: u32, sample: u32) -> f64 {
    let mut h = seed ^ 0x9e37_79b9;
    h = (h.wrapping_mul(0x85eb_ca6b)) ^ segment.rotate_left(15);
    h = (h.wrapping_mul(0xc2b2_ae35)) ^ sample.rotate_left(13);
    h ^= h >> 16;
    h = h.wrapping_mul(0x7feb_352d);
    h ^= h >> 15;
    // u32 → [-1, 1): exact, deterministic.
    (f64::from(h) / f64::from(u32::MAX)) * 2.0 - 1.0
}

fn snap_network(net: &mut VectorNetwork) {
    for v in net.vertices.iter_mut() {
        v.pos = Vec2::new(snap(v.pos.x), snap(v.pos.y));
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
    fn roughen_subdivides_and_stays_valid() {
        let out = roughen(&square(), 5.0, 8, 0);
        assert!(out.validate().is_ok());
        // 4 segments × 8 spans = 32 spans; 4 corners + 4×7 interior = 32 vertices.
        assert_eq!(out.segments.len(), 32);
        assert_eq!(out.vertices.len(), 32);
        assert_eq!(out.regions.len(), 1);
        assert_eq!(out.regions[0].segments.len(), 32);
    }

    #[test]
    fn corners_are_preserved() {
        // Endpoints (the original 4 corners) are never displaced.
        let out = roughen(&square(), 5.0, 8, 0);
        for corner in [
            Vec2::new(0.0, 0.0),
            Vec2::new(100.0, 0.0),
            Vec2::new(100.0, 100.0),
            Vec2::new(0.0, 100.0),
        ] {
            assert!(
                out.vertices.iter().any(|v| v.pos == corner),
                "corner {corner:?} must survive"
            );
        }
    }

    #[test]
    fn amplitude_actually_displaces() {
        let out = roughen(&square(), 5.0, 8, 0);
        // Some interior vertex must be off the original axis-aligned edges.
        let off_edge = out.vertices.iter().any(|v| {
            let on_x = v.pos.x == 0.0 || v.pos.x == 100.0;
            let on_y = v.pos.y == 0.0 || v.pos.y == 100.0;
            !on_x && !on_y
        });
        assert!(off_edge, "roughen must push samples off the straight edges");
    }

    #[test]
    fn seed_changes_the_result() {
        let a = roughen(&square(), 5.0, 8, 1);
        let b = roughen(&square(), 5.0, 8, 2);
        assert_ne!(a, b, "different seeds → different jitter");
    }

    #[test]
    fn deterministic_and_reproducible() {
        let out = roughen(&square(), 5.0, 8, 42);
        assert!(out.deterministic);
        assert_eq!(out, roughen(&square(), 5.0, 8, 42), "byte-stable per seed");
    }

    #[test]
    fn zero_detail_or_amplitude_is_identity() {
        assert_eq!(roughen(&square(), 5.0, 1, 0), square());
        assert_eq!(roughen(&square(), 0.0, 8, 0), square());
    }
}
