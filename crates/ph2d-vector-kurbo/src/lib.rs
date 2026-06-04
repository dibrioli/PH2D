#![forbid(unsafe_code)]
//! Kurbo bridge for the vector geometry-node fan-out: `VectorNetwork` ⇄
//! `kurbo::BezPath`, plus the Linesweeper exact-boolean wrapper.
//!
//! This is the **one** crate that speaks `kurbo` + `linesweeper`, so every
//! geometry node (`vector.boolean`, `vector.offset`, future
//! `vector.outline-stroke` / `vector.roughen` …) shares a single, tested
//! conversion — no divergent re-implementations — and kurbo stays confined to a
//! single crate (the eventual `vello_kurbo_only_in_ph2d_vector` whitelist
//! target, ADR-0059 §2.8).
//!
//! ## Curve convention (shared with the renderer)
//!
//! A segment's cubic control points are `c1 = start.pos + out_at_start` and
//! `c2 = end.pos + in_at_end` (tangents are **relative** offset vectors) —
//! identical to `ph2d_vector::build_region_path`, so a network round-trips with
//! the same geometry the renderer draws.
//!
//! ## Determinism (cross-OS golden — ADR-0065 §2.4)
//!
//! Linesweeper / kurbo work in `f64`, whose last ULP can differ across targets.
//! [`contours_to_network`] snaps every output coordinate to the Q16.16 grid
//! ([`snap`]) and flags the result `deterministic` when asked — erasing that
//! drift so the bytes are platform-independent, exactly as `vector.source` does.

use std::collections::BTreeMap;

use glam::Vec2;
use kurbo::{BezPath, PathEl, Point};
use ph2d_vector_doc::deterministic::snap;
use ph2d_vector_doc::{
    Region, RegionId, Segment, SegmentId, VectorNetwork, Vertex, VertexId, WindingRule,
};

// Re-exports so the node crates depend only on this bridge, not on kurbo /
// linesweeper directly (keeps the kurbo surface in one place).
pub use kurbo::{self, BezPath as KurboBezPath, Cap, Join, Stroke, StrokeOpts, stroke};
pub use linesweeper::{BinaryOp, FillRule};

/// Endpoint-coincidence tolerance used when stitching a contour's closing
/// segment back onto its start vertex. Linesweeper closes a contour on the
/// *exact same* `f64` point it opened it on, so this only guards last-ULP
/// wobble; it is far below the Q16.16 grid step (`~1.5e-5`).
const CLOSE_EPS: f64 = 1e-9;

#[inline]
fn pt(v: Vec2) -> Point {
    Point::new(f64::from(v.x), f64::from(v.y))
}

#[inline]
fn vec(p: Point) -> Vec2 {
    Vec2::new(p.x as f32, p.y as f32)
}

/// The fill rule to interpret a network's boundaries with — its first region's
/// winding, defaulting to SVG-canonical [`WindingRule::NonZero`].
#[must_use]
pub fn fill_rule_of(net: &VectorNetwork) -> FillRule {
    match net.regions.first().map(|r| r.winding) {
        Some(WindingRule::EvenOdd) => FillRule::EvenOdd,
        _ => FillRule::NonZero,
    }
}

/// Build a `kurbo::BezPath` with one closed subpath per fillable region of
/// `net`, using the renderer's cubic control-point convention. A region-less
/// network (open paths only) produces an empty path = the empty set.
#[must_use]
pub fn network_to_bezpath(net: &VectorNetwork) -> BezPath {
    let segs: BTreeMap<SegmentId, &Segment> = net.segments.iter().map(|s| (s.id, s)).collect();
    let verts: BTreeMap<VertexId, &Vertex> = net.vertices.iter().map(|v| (v.id, v)).collect();
    let mut path = BezPath::new();
    for region in &net.regions {
        append_region(&mut path, region, &segs, &verts);
    }
    path
}

/// Append `region` as one closed subpath. **Strict**: a dangling segment/vertex
/// ref aborts this region entirely (mirrors `ph2d_vector::build_region_path`),
/// emitting nothing rather than a torn subpath.
fn append_region(
    path: &mut BezPath,
    region: &Region,
    segs: &BTreeMap<SegmentId, &Segment>,
    verts: &BTreeMap<VertexId, &Vertex>,
) {
    if region.segments.is_empty() {
        return;
    }
    // Stage into a scratch buffer so a dangling ref discovered mid-loop discards
    // the whole region instead of leaving a half-built subpath on `path`.
    let mut els: Vec<PathEl> = Vec::with_capacity(region.segments.len() + 2);
    for (i, &(seg_id, forward)) in region.segments.iter().enumerate() {
        let Some(seg) = segs.get(&seg_id) else {
            return;
        };
        let (start, end, c1, c2) = if forward {
            let (Some(s), Some(e)) = (verts.get(&seg.start), verts.get(&seg.end)) else {
                return;
            };
            (
                s.pos,
                e.pos,
                s.pos + seg.out_at_start,
                e.pos + seg.in_at_end,
            )
        } else {
            let (Some(s), Some(e)) = (verts.get(&seg.end), verts.get(&seg.start)) else {
                return;
            };
            (
                s.pos,
                e.pos,
                s.pos + seg.in_at_end,
                e.pos + seg.out_at_start,
            )
        };
        if i == 0 {
            els.push(PathEl::MoveTo(pt(start)));
        }
        els.push(PathEl::CurveTo(pt(c1), pt(c2), pt(end)));
    }
    els.push(PathEl::ClosePath);
    path.extend(els);
}

/// Run one Linesweeper set op and collect the result faces as closed
/// `BezPath`s. A malformed input (NaN/∞/non-closed — none of which a valid
/// network produces) degrades to *no contours* rather than panicking, so a node
/// `eval` (which cannot return a `Result`) stays total.
#[must_use]
pub fn boolean_paths(a: &BezPath, b: &BezPath, fill: FillRule, op: BinaryOp) -> Vec<BezPath> {
    match linesweeper::binary_op(a, b, fill, op) {
        Ok(contours) => contours.contours().map(|c| c.path.clone()).collect(),
        Err(_) => Vec::new(),
    }
}

/// A vertex/segment/region id allocator shared across all contours of one
/// result network, so the whole network's ids are unique (`validate`-clean).
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

    fn push_vertex(&mut self, p: Point) -> VertexId {
        let id = self.next_v;
        self.next_v += 1;
        self.net.vertices.push(Vertex::auto(id, vec(p)));
        id
    }

    /// Push a segment with explicit cubic tangents (relative offsets). `Segment`
    /// is `#[non_exhaustive]`, so we go through the `straight` constructor and
    /// then set the public tangent fields.
    fn push_segment(
        &mut self,
        start: VertexId,
        end: VertexId,
        out_at_start: Vec2,
        in_at_end: Vec2,
    ) {
        let id = self.next_s;
        self.next_s += 1;
        let mut s = Segment::straight(id, start, end);
        s.out_at_start = out_at_start;
        s.in_at_end = in_at_end;
        self.net.segments.push(s);
    }
}

/// Translate result contours into a `VectorNetwork`: each closed contour → one
/// `NonZero` region (Linesweeper orients faces set-on-left, winding 1, so
/// non-zero fill is correct and holes — emitted as separate oppositely-wound
/// contours — render correctly). Regions are left `fill = None`, matching the
/// generator convention (the asset / bridge assigns the fill). When
/// `deterministic`, every coordinate is snapped to the Q16.16 grid and the
/// network is flagged `deterministic`.
#[must_use]
pub fn contours_to_network(contours: &[BezPath], deterministic: bool) -> VectorNetwork {
    let mut b = Builder::new();
    for path in contours {
        emit_contour(&mut b, path);
    }
    if deterministic {
        snap_network(&mut b.net);
    }
    b.net.deterministic = deterministic;
    b.net
}

/// Walk one closed contour's path elements into vertices + segments + a region.
/// A drawing element whose endpoint coincides with the start vertex closes the
/// loop onto it (no duplicate vertex); a trailing `ClosePath` gap is bridged
/// with a straight segment. Degenerate contours (< 2 segments) are skipped.
fn emit_contour(b: &mut Builder, path: &BezPath) {
    let mut start_v: Option<VertexId> = None;
    let mut start_pos = Point::ZERO;
    let mut cur_v: VertexId = 0;
    let mut cur_pos = Point::ZERO;
    let mut refs: Vec<(SegmentId, bool)> = Vec::new();
    let mut closed = false;

    for el in path.elements() {
        match *el {
            PathEl::MoveTo(p) => {
                let v = b.push_vertex(p);
                start_v = Some(v);
                start_pos = p;
                cur_v = v;
                cur_pos = p;
            }
            PathEl::LineTo(p) => {
                let Some(sv) = start_v else { continue };
                close_or_extend(
                    b,
                    &mut refs,
                    &mut cur_v,
                    &mut cur_pos,
                    &mut closed,
                    sv,
                    start_pos,
                    p,
                    Vec2::ZERO,
                    Vec2::ZERO,
                );
            }
            PathEl::QuadTo(c, p) => {
                let Some(sv) = start_v else { continue };
                // Elevate the quadratic to a cubic: the two cubic controls sit
                // 2/3 of the way from each endpoint toward the quad control.
                let out = (vec(c) - vec(cur_pos)) * (2.0_f32 / 3.0);
                let inn = (vec(c) - vec(p)) * (2.0_f32 / 3.0);
                close_or_extend(
                    b,
                    &mut refs,
                    &mut cur_v,
                    &mut cur_pos,
                    &mut closed,
                    sv,
                    start_pos,
                    p,
                    out,
                    inn,
                );
            }
            PathEl::CurveTo(c1, c2, p) => {
                let Some(sv) = start_v else { continue };
                let out = vec(c1) - vec(cur_pos);
                let inn = vec(c2) - vec(p);
                close_or_extend(
                    b,
                    &mut refs,
                    &mut cur_v,
                    &mut cur_pos,
                    &mut closed,
                    sv,
                    start_pos,
                    p,
                    out,
                    inn,
                );
            }
            PathEl::ClosePath => {
                if let Some(sv) = start_v
                    && !closed
                    && cur_pos.distance(start_pos) > CLOSE_EPS
                {
                    let sid = b.next_s;
                    b.push_segment(cur_v, sv, Vec2::ZERO, Vec2::ZERO);
                    refs.push((sid, true));
                }
                closed = true;
            }
        }
    }

    // Emit the region only if the loop encloses area (≥ 2 segments). A
    // degenerate 0/1-segment contour is dropped (orphan vertices are harmless to
    // `validate` and the renderer ignores them).
    if refs.len() >= 2 {
        let rid = b.next_r;
        b.next_r += 1;
        let mut region = Region::new(rid, WindingRule::NonZero);
        region.segments = refs.into_iter().collect();
        b.net.regions.push(region);
    }
}

/// Shared step for `Line`/`Quad`/`Curve`: if `p` coincides with the contour
/// start, close the loop onto `start_v` (no new vertex); otherwise allocate a
/// new vertex at `p`. Either way records the connecting segment.
#[allow(clippy::too_many_arguments)]
fn close_or_extend(
    b: &mut Builder,
    refs: &mut Vec<(SegmentId, bool)>,
    cur_v: &mut VertexId,
    cur_pos: &mut Point,
    closed: &mut bool,
    start_v: VertexId,
    start_pos: Point,
    p: Point,
    out_at_start: Vec2,
    in_at_end: Vec2,
) {
    if *closed {
        return;
    }
    let sid = b.next_s;
    if p.distance(start_pos) <= CLOSE_EPS {
        b.push_segment(*cur_v, start_v, out_at_start, in_at_end);
        refs.push((sid, true));
        *closed = true;
    } else {
        let v = b.push_vertex(p);
        b.push_segment(*cur_v, v, out_at_start, in_at_end);
        refs.push((sid, true));
        *cur_v = v;
        *cur_pos = p;
    }
}

/// Snap every coordinate (vertex positions + segment tangents) to the Q16.16
/// grid — the cross-OS determinism guarantee (ADR-0065 §2.4).
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

    #[test]
    fn region_round_trips_through_a_bezpath() {
        // A single rect region → one closed subpath (MoveTo + 4 CurveTo +
        // ClosePath = 6 elements).
        let net = primitives::rect(Vec2::new(0.0, 0.0), Vec2::new(2.0, 2.0));
        let path = network_to_bezpath(&net);
        assert_eq!(path.elements().len(), 6);
    }

    #[test]
    fn contours_to_network_builds_a_valid_region() {
        // A hand-built closed square contour → one validated 4-segment region.
        let mut path = BezPath::new();
        path.move_to((0.0, 0.0));
        path.line_to((2.0, 0.0));
        path.line_to((2.0, 2.0));
        path.line_to((0.0, 2.0));
        path.close_path();
        let net = contours_to_network(&[path], true);
        assert!(net.validate().is_ok());
        assert_eq!(net.regions.len(), 1);
        assert_eq!(net.regions[0].segments.len(), 4);
        assert!(net.deterministic);
    }

    #[test]
    fn boolean_paths_union_of_overlapping_squares_is_one_contour() {
        let a = network_to_bezpath(&primitives::rect(Vec2::new(0.0, 0.0), Vec2::new(2.0, 2.0)));
        let b = network_to_bezpath(&primitives::rect(Vec2::new(1.0, 1.0), Vec2::new(3.0, 3.0)));
        let contours = boolean_paths(&a, &b, FillRule::NonZero, BinaryOp::Union);
        assert_eq!(contours.len(), 1);
    }

    #[test]
    fn empty_network_is_an_empty_path() {
        assert!(network_to_bezpath(&VectorNetwork::empty()).is_empty());
    }
}
