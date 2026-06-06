//! The pattern-along-path engine: place `count` copies of a **shape** network
//! evenly (by arc length) along a **path** network, each copy translated to its
//! frame and — when `align` — rotated so the shape's local +x follows the path
//! tangent. Merged into one network with fresh ids.
//!
//! ## Following the real curve
//!
//! The path's segments are cubic (`out_at_start`/`in_at_end` handles; the
//! `c1 = start + out`, `c2 = end + in` convention `primitives::ellipse` uses).
//! We walk the segments into one ordered chain ([`ordered_segments`]), flatten
//! each cubic to a dense polyline, and sample frames by **arc length** — so the
//! copies sit *on* the curve, not on its chords.
//!
//! ## Determinism (cross-OS golden)
//!
//! The transform per copy is a rotate+scale+translate built from the unit
//! tangent's components directly (no `atan2`/`sin`/`cos`), so it is exactly
//! reproducible; positions snap to Q16.16 when either input is `deterministic`
//! (mirrors `ph2d-node-vector-scatter`).

use std::collections::BTreeMap;

use glam::Vec2;
use ph2d_vector_doc::deterministic::snap;
use ph2d_vector_doc::{Region, RegionId, Segment, SegmentId, VectorNetwork, Vertex, VertexId};

/// Allocation ceiling for `count` — bounds the untrusted `f32 → count` against
/// an OOM value (each copy duplicates the whole shape network). Matches
/// `ph2d-node-vector-scatter`.
pub const MAX_COUNT: usize = 1 << 14;

/// Cubic-flattening samples per path segment. 24 keeps the arc-length estimate
/// tight for hand-drawn curves while staying cheap (paths inline ≤ 64 segments).
const FLATTEN_STEPS: usize = 24;

/// Place `count` copies of `shape` along `path`. `align` rotates each copy to the
/// local path tangent; `scale` uniformly scales each copy. Empty shape / empty or
/// point-only path → an empty network.
#[must_use]
pub fn pattern_along_path(
    shape: &VectorNetwork,
    path: &VectorNetwork,
    count: usize,
    align: bool,
    scale: f32,
) -> VectorNetwork {
    let n = count.min(MAX_COUNT);
    let deterministic = shape.deterministic || path.deterministic;
    if n == 0 || shape.vertices.is_empty() {
        return empty(deterministic);
    }
    let frames = path_frames(path, n);
    if frames.is_empty() {
        return empty(deterministic);
    }

    let mut b = Builder::new();
    for f in &frames {
        let (cos, sin) = if align { (f.dir.x, f.dir.y) } else { (1.0, 0.0) };
        b.append(
            shape,
            Xform {
                pos: f.pos,
                cos,
                sin,
                scale,
            },
        );
    }
    if deterministic {
        snap_network(&mut b.net);
    }
    b.net.deterministic = deterministic;
    b.net
}

fn empty(deterministic: bool) -> VectorNetwork {
    let mut net = VectorNetwork::empty();
    net.deterministic = deterministic;
    net
}

// ─────────────────────────── path → arc-length frames ───────────────────────

/// A placement frame: a point on the path and the unit tangent there.
struct Frame {
    pos: Vec2,
    dir: Vec2,
}

/// `count` frames spaced evenly by arc length: a single copy sits at the path
/// midpoint; otherwise the ends are inclusive (`0`, …, `L`).
fn path_frames(path: &VectorNetwork, count: usize) -> Vec<Frame> {
    let poly = flatten_path(path);
    if poly.len() < 2 {
        return Vec::new();
    }
    // Cumulative arc length along the dense polyline.
    let mut cum = vec![0.0_f32; poly.len()];
    for i in 1..poly.len() {
        cum[i] = cum[i - 1] + (poly[i] - poly[i - 1]).length();
    }
    let total = cum[poly.len() - 1];
    if total <= f32::EPSILON {
        return Vec::new();
    }

    let mut frames = Vec::with_capacity(count);
    for i in 0..count {
        let s = if count == 1 {
            0.5 * total
        } else {
            total * i as f32 / (count - 1) as f32
        };
        frames.push(sample_arc(&poly, &cum, s));
    }
    frames
}

/// Point + unit tangent at arc length `s` along the polyline.
fn sample_arc(poly: &[Vec2], cum: &[f32], s: f32) -> Frame {
    let s = s.clamp(0.0, cum[cum.len() - 1]);
    let mut j = 0;
    while j + 2 < poly.len() && cum[j + 1] < s {
        j += 1;
    }
    let seg_len = (cum[j + 1] - cum[j]).max(f32::EPSILON);
    let t = ((s - cum[j]) / seg_len).clamp(0.0, 1.0);
    let pos = poly[j].lerp(poly[j + 1], t);
    let mut dir = (poly[j + 1] - poly[j]).normalize_or_zero();
    if dir == Vec2::ZERO {
        dir = Vec2::X;
    }
    Frame { pos, dir }
}

/// Flatten the path's (ordered) cubic segments into one dense polyline.
fn flatten_path(path: &VectorNetwork) -> Vec<Vec2> {
    let ordered = ordered_segments(path);
    if ordered.is_empty() {
        return Vec::new();
    }
    let mut out: Vec<Vec2> = Vec::new();
    for (idx, fwd) in ordered {
        let s = &path.segments[idx];
        let p0 = vertex_pos(path, s.start);
        let p3 = vertex_pos(path, s.end);
        // Convention (primitives::ellipse): c1 = start + out, c2 = end + in.
        let c1 = p0 + s.out_at_start;
        let c2 = p3 + s.in_at_end;
        let (a0, a1, a2, a3) = if fwd {
            (p0, c1, c2, p3)
        } else {
            (p3, c2, c1, p0)
        };
        // Skip the join point (t=0) on every segment after the first.
        let first_k = if out.is_empty() { 0 } else { 1 };
        for k in first_k..=FLATTEN_STEPS {
            let t = k as f32 / FLATTEN_STEPS as f32;
            out.push(cubic_eval(a0, a1, a2, a3, t));
        }
    }
    out
}

/// Order the path's segments into a single connected chain, each tagged whether
/// it is traversed forward (entered at its `start`). Starts at an open end
/// (degree-1 vertex) when there is one, else the smallest vertex id (closed
/// loop). Only the component containing the start is walked (a path is one
/// curve); deterministic via [`BTreeMap`] ordering.
fn ordered_segments(net: &VectorNetwork) -> Vec<(usize, bool)> {
    if net.segments.is_empty() {
        return Vec::new();
    }
    let mut adj: BTreeMap<VertexId, Vec<(usize, bool)>> = BTreeMap::new();
    for (i, s) in net.segments.iter().enumerate() {
        adj.entry(s.start).or_default().push((i, true));
        adj.entry(s.end).or_default().push((i, false));
    }
    let start = adj
        .iter()
        .find(|(_, e)| e.len() == 1)
        .map(|(v, _)| *v)
        .or_else(|| adj.keys().next().copied());
    let Some(mut cur) = start else {
        return Vec::new();
    };

    let mut visited = vec![false; net.segments.len()];
    let mut out = Vec::new();
    loop {
        let next = adj
            .get(&cur)
            .and_then(|edges| edges.iter().find(|(i, _)| !visited[*i]).copied());
        let Some((i, at_start)) = next else {
            break;
        };
        visited[i] = true;
        out.push((i, at_start));
        cur = if at_start {
            net.segments[i].end
        } else {
            net.segments[i].start
        };
    }
    out
}

fn vertex_pos(net: &VectorNetwork, id: VertexId) -> Vec2 {
    net.vertices
        .iter()
        .find(|v| v.id == id)
        .map_or(Vec2::ZERO, |v| v.pos)
}

/// Cubic Bézier point at `t`.
fn cubic_eval(p0: Vec2, p1: Vec2, p2: Vec2, p3: Vec2, t: f32) -> Vec2 {
    let u = 1.0 - t;
    p0 * (u * u * u) + p1 * (3.0 * u * u * t) + p2 * (3.0 * u * t * t) + p3 * (t * t * t)
}

// ────────────────────────────── transformed merge ──────────────────────────

/// A rigid (rotate + uniform-scale + translate) transform. `cos`/`sin` are the
/// unit tangent's components (identity when not aligning), so no transcendental
/// is evaluated and the result is bit-stable.
struct Xform {
    pos: Vec2,
    cos: f32,
    sin: f32,
    scale: f32,
}

impl Xform {
    /// Transform a position (rotate+scale, then translate to the frame).
    fn point(&self, p: Vec2) -> Vec2 {
        self.pos + self.vector(p)
    }

    /// Transform a free vector (a tangent handle): rotate+scale, no translation.
    fn vector(&self, p: Vec2) -> Vec2 {
        let s = p * self.scale;
        Vec2::new(s.x * self.cos - s.y * self.sin, s.x * self.sin + s.y * self.cos)
    }
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

    /// Append one copy of `src` transformed by `xf`. Cubic tangent handles are
    /// rotated+scaled (not translated); regions/fills/styles are preserved.
    /// Fresh ids keep the merged network valid.
    fn append(&mut self, src: &VectorNetwork, xf: Xform) {
        let mut vmap: BTreeMap<VertexId, VertexId> = BTreeMap::new();
        for v in &src.vertices {
            let nid = self.next_v;
            self.next_v += 1;
            vmap.insert(v.id, nid);
            self.net.vertices.push(Vertex::new(nid, xf.point(v.pos), v.kind));
        }
        let mut smap: BTreeMap<SegmentId, SegmentId> = BTreeMap::new();
        for s in &src.segments {
            let (Some(&start), Some(&end)) = (vmap.get(&s.start), vmap.get(&s.end)) else {
                continue;
            };
            let nid = self.next_s;
            self.next_s += 1;
            smap.insert(s.id, nid);
            let mut ns = Segment::straight(nid, start, end);
            ns.out_at_start = xf.vector(s.out_at_start);
            ns.in_at_end = xf.vector(s.in_at_end);
            ns.style_ref = s.style_ref;
            self.net.segments.push(ns);
        }
        for r in &src.regions {
            let nid = self.next_r;
            self.next_r += 1;
            let mut nr = Region::new(nid, r.winding);
            nr.segments = r
                .segments
                .iter()
                .filter_map(|(sid, fwd)| smap.get(sid).map(|&ns| (ns, *fwd)))
                .collect();
            nr.fill = r.fill;
            nr.z = r.z;
            if !nr.segments.is_empty() {
                self.net.regions.push(nr);
            }
        }
    }
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

    /// A unit square shape centered at the origin.
    fn shape() -> VectorNetwork {
        let mut net = primitives::rect(Vec2::new(-1.0, -1.0), Vec2::new(1.0, 1.0));
        net.deterministic = true;
        net
    }

    /// A straight horizontal path from x=0 to x=L on y=0 (2 vertices, 1 segment).
    fn straight_path(len: f32) -> VectorNetwork {
        let mut net = VectorNetwork::empty();
        net.vertices.push(Vertex::auto(0, Vec2::new(0.0, 0.0)));
        net.vertices.push(Vertex::auto(1, Vec2::new(len, 0.0)));
        net.segments.push(Segment::straight(0, 0, 1));
        net.deterministic = true;
        net
    }

    #[test]
    fn places_count_copies_along_path() {
        let out = pattern_along_path(&shape(), &straight_path(100.0), 5, false, 1.0);
        assert!(out.validate().is_ok());
        assert_eq!(out.regions.len(), 5, "one region per copy");
        assert_eq!(out.vertices.len(), 20, "5 × 4 corners");
    }

    #[test]
    fn copies_span_the_path_endpoints() {
        // count=2 → ends inclusive: copies centered at x=0 and x=100.
        let out = pattern_along_path(&shape(), &straight_path(100.0), 2, false, 1.0);
        let xs: Vec<f32> = out.vertices.iter().map(|v| v.pos.x).collect();
        let min = xs.iter().cloned().fold(f32::INFINITY, f32::min);
        let max = xs.iter().cloned().fold(f32::NEG_INFINITY, f32::max);
        // first copy corners around x=0 (∈[-1,1]); last around x=100 (∈[99,101]).
        assert!((-1.1..=-0.9).contains(&min), "min x = {min}");
        assert!((99.9..=101.1).contains(&max), "max x = {max}");
    }

    #[test]
    fn align_rotates_copy_to_tangent() {
        // Vertical path → tangent (0,1); align rotates the shape 90°. A square is
        // symmetric, so use an asymmetric shape: a wide-thin rect (x:±4, y:±1).
        let mut wide = primitives::rect(Vec2::new(-4.0, -1.0), Vec2::new(4.0, 1.0));
        wide.deterministic = true;
        let mut vpath = VectorNetwork::empty();
        vpath.vertices.push(Vertex::auto(0, Vec2::new(0.0, 0.0)));
        vpath.vertices.push(Vertex::auto(1, Vec2::new(0.0, 100.0)));
        vpath.segments.push(Segment::straight(0, 0, 1));
        vpath.deterministic = true;

        let out = pattern_along_path(&wide, &vpath, 1, true, 1.0);
        // The single copy sits at the path midpoint (0,50). After a +90° rotation
        // the ±4 x-extent maps onto the y-axis → wide in y, thin in x.
        let span_x = span(&out, |v| v.pos.x);
        let span_y = span(&out, |v| v.pos.y);
        assert!(span_y > span_x + 4.0, "expected rotation: span_y {span_y} vs span_x {span_x}");
    }

    #[test]
    fn follows_cubic_curve_not_chord() {
        // A quarter-ellipse arc bulges away from its chord; a mid copy must land
        // off the straight line between the arc endpoints.
        let arc = primitives::ellipse(Vec2::new(0.0, 0.0), Vec2::new(100.0, 100.0));
        // ellipse is a closed 4-arc loop; place 8 copies, assert at least one is
        // well off the bounding diagonal (i.e., the curve is being followed).
        let out = pattern_along_path(&shape(), &arc, 8, false, 1.0);
        assert!(out.validate().is_ok());
        let off_axis = out
            .vertices
            .iter()
            .any(|v| v.pos.x.abs() > 50.0 && v.pos.y.abs() > 50.0);
        assert!(off_axis, "copies should follow the curved arc, not a chord");
    }

    #[test]
    fn zero_count_or_empty_inputs_are_empty() {
        assert!(pattern_along_path(&shape(), &straight_path(100.0), 0, false, 1.0)
            .regions
            .is_empty());
        assert!(pattern_along_path(&VectorNetwork::empty(), &straight_path(100.0), 5, false, 1.0)
            .regions
            .is_empty());
        // Path with no segments → no frames → empty.
        assert!(pattern_along_path(&shape(), &VectorNetwork::empty(), 5, false, 1.0)
            .regions
            .is_empty());
    }

    #[test]
    fn deterministic_and_reproducible() {
        let out = pattern_along_path(&shape(), &straight_path(100.0), 7, true, 1.5);
        assert!(out.deterministic);
        assert_eq!(
            out,
            pattern_along_path(&shape(), &straight_path(100.0), 7, true, 1.5),
            "byte-stable"
        );
    }

    #[test]
    fn scale_grows_copies() {
        let small = pattern_along_path(&shape(), &straight_path(100.0), 1, false, 1.0);
        let big = pattern_along_path(&shape(), &straight_path(100.0), 1, false, 3.0);
        assert!(span(&big, |v| v.pos.x) > span(&small, |v| v.pos.x) + 2.0);
    }

    fn span(net: &VectorNetwork, f: impl Fn(&Vertex) -> f32) -> f32 {
        let mut lo = f32::INFINITY;
        let mut hi = f32::NEG_INFINITY;
        for v in &net.vertices {
            let x = f(v);
            lo = lo.min(x);
            hi = hi.max(x);
        }
        hi - lo
    }
}
