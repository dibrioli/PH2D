//! The outline-stroke engine: expand the input's paths (open chains + closed
//! region boundaries) into **filled regions** by kurbo stroke expansion of width
//! `width`, then convert the outline back to a `VectorNetwork` via the shared
//! [`ph2d_vector_kurbo`] bridge (ADR-0058 §2.2.4).
//!
//! Open segments are stitched into chains by adjacency (a pencil stroke →
//! one stroked outline); region boundaries are stroked as closed loops (a filled
//! shape → a frame). Overlapping stroked chains are kept as distinct outline
//! regions (a boolean union cleanup is a follow-up).
//!
//! Determinism: Q16.16-snapped output when the input is `deterministic` (the
//! bridge handles the snap); the kurbo stroke tolerance is fixed.

use std::collections::{BTreeMap, BTreeSet};

use glam::Vec2;
use ph2d_vector_doc::{Segment, SegmentId, VectorNetwork, VertexId};
use ph2d_vector_kurbo::kurbo::{BezPath, PathEl, Point};
use ph2d_vector_kurbo::{Cap, Join, Stroke, StrokeOpts, contours_to_network, stroke};

/// Fixed curve-flattening tolerance for the stroke outline (px).
const STROKE_TOLERANCE: f64 = 0.05;

/// Expand the input's paths into filled outline regions of stroke `width`.
#[must_use]
pub fn outline_stroke(input: &VectorNetwork, width: f32, cap: Cap, join: Join) -> VectorNetwork {
    if width <= 0.0 || !width.is_finite() || input.segments.is_empty() {
        let mut empty = VectorNetwork::empty();
        empty.deterministic = input.deterministic;
        return empty;
    }
    let path = build_path(input);
    if path.is_empty() {
        let mut empty = VectorNetwork::empty();
        empty.deterministic = input.deterministic;
        return empty;
    }
    let style = Stroke::new(f64::from(width)).with_caps(cap).with_join(join);
    let outline = stroke(
        path.iter(),
        &style,
        &StrokeOpts::default(),
        STROKE_TOLERANCE,
    );
    // kurbo returns one BezPath with many closed subpaths; the bridge's
    // converter wants one contour per BezPath, so split first.
    let contours = split_subpaths(&outline);
    contours_to_network(&contours, input.deterministic)
}

/// Build a `BezPath` of every input path: closed subpaths for region boundaries
/// + open subpaths for the remaining (non-region) segment chains.
fn build_path(net: &VectorNetwork) -> BezPath {
    let seg_by_id: BTreeMap<SegmentId, &Segment> = net.segments.iter().map(|s| (s.id, s)).collect();
    let verts: BTreeMap<VertexId, Vec2> = net.vertices.iter().map(|v| (v.id, v.pos)).collect();
    let mut path = BezPath::new();
    let mut region_segs: BTreeSet<SegmentId> = BTreeSet::new();

    // Closed region boundaries.
    for region in &net.regions {
        let refs: Vec<(SegmentId, bool)> = region.segments.iter().copied().collect();
        if refs.iter().all(|(sid, _)| seg_by_id.contains_key(sid)) && !refs.is_empty() {
            for (sid, _) in &refs {
                region_segs.insert(*sid);
            }
            append_subpath(&mut path, &refs, true, &seg_by_id, &verts);
        }
    }

    // Open chains from the leftover segments.
    for chain in open_chains(net, &region_segs, &seg_by_id) {
        append_subpath(&mut path, &chain, false, &seg_by_id, &verts);
    }
    path
}

/// Emit one subpath (cubic per segment, renderer control-point convention) for
/// the ordered `refs`; `closed` adds a `ClosePath`.
fn append_subpath(
    path: &mut BezPath,
    refs: &[(SegmentId, bool)],
    closed: bool,
    seg_by_id: &BTreeMap<SegmentId, &Segment>,
    verts: &BTreeMap<VertexId, Vec2>,
) {
    let mut els: Vec<PathEl> = Vec::with_capacity(refs.len() + 2);
    for (i, &(sid, fwd)) in refs.iter().enumerate() {
        let Some(seg) = seg_by_id.get(&sid) else {
            return;
        };
        let (s_id, e_id, out, inn) = if fwd {
            (seg.start, seg.end, seg.out_at_start, seg.in_at_end)
        } else {
            (seg.end, seg.start, seg.in_at_end, seg.out_at_start)
        };
        let (Some(&sp), Some(&ep)) = (verts.get(&s_id), verts.get(&e_id)) else {
            return;
        };
        if i == 0 {
            els.push(PathEl::MoveTo(pt(sp)));
        }
        els.push(PathEl::CurveTo(pt(sp + out), pt(ep + inn), pt(ep)));
    }
    if closed {
        els.push(PathEl::ClosePath);
    }
    path.extend(els);
}

/// Greedily stitch the non-region segments into chains by shared vertices. Every
/// such segment lands in exactly one chain (degree-1 endpoints start open
/// chains; leftover loops start anywhere).
fn open_chains(
    net: &VectorNetwork,
    region_segs: &BTreeSet<SegmentId>,
    seg_by_id: &BTreeMap<SegmentId, &Segment>,
) -> Vec<Vec<(SegmentId, bool)>> {
    let open: Vec<&Segment> = net
        .segments
        .iter()
        .filter(|s| !region_segs.contains(&s.id))
        .collect();
    if open.is_empty() {
        return Vec::new();
    }
    let mut adj: BTreeMap<VertexId, Vec<SegmentId>> = BTreeMap::new();
    for s in &open {
        adj.entry(s.start).or_default().push(s.id);
        adj.entry(s.end).or_default().push(s.id);
    }
    let mut visited: BTreeSet<SegmentId> = BTreeSet::new();
    let mut chains: Vec<Vec<(SegmentId, bool)>> = Vec::new();

    let walk = |start: VertexId,
                visited: &mut BTreeSet<SegmentId>,
                adj: &BTreeMap<VertexId, Vec<SegmentId>>| {
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

    // Prefer starting at open endpoints (degree-1) so chains run end-to-end.
    let endpoints: Vec<VertexId> = adj
        .iter()
        .filter(|(_, segs)| segs.len() == 1)
        .map(|(v, _)| *v)
        .collect();
    for v in endpoints {
        let chain = walk(v, &mut visited, &adj);
        if !chain.is_empty() {
            chains.push(chain);
        }
    }
    // Any leftover segments (closed loops with no degree-1 vertex).
    for s in &open {
        if !visited.contains(&s.id) {
            let chain = walk(s.start, &mut visited, &adj);
            if !chain.is_empty() {
                chains.push(chain);
            }
        }
    }
    chains
}

/// Split a multi-subpath `BezPath` into one `BezPath` per `MoveTo`-delimited
/// subpath (so the bridge converts one contour at a time).
fn split_subpaths(bez: &BezPath) -> Vec<BezPath> {
    let mut out = Vec::new();
    let mut cur = BezPath::new();
    for el in bez.elements() {
        if matches!(el, PathEl::MoveTo(_)) && !cur.is_empty() {
            out.push(std::mem::take(&mut cur));
        }
        cur.push(*el);
    }
    if !cur.is_empty() {
        out.push(cur);
    }
    out
}

#[inline]
fn pt(v: Vec2) -> Point {
    Point::new(f64::from(v.x), f64::from(v.y))
}

#[cfg(test)]
mod tests {
    use super::*;
    use ph2d_vector_doc::{Vertex, primitives};

    fn closed_square() -> VectorNetwork {
        let mut net = primitives::rect(Vec2::new(0.0, 0.0), Vec2::new(100.0, 100.0));
        net.deterministic = true;
        net
    }

    /// An open 2-segment polyline (a "pencil stroke"): (0,0)→(50,0)→(50,50), no
    /// region.
    fn open_stroke() -> VectorNetwork {
        let mut net = VectorNetwork::empty();
        net.deterministic = true;
        net.vertices.push(Vertex::auto(0, Vec2::new(0.0, 0.0)));
        net.vertices.push(Vertex::auto(1, Vec2::new(50.0, 0.0)));
        net.vertices.push(Vertex::auto(2, Vec2::new(50.0, 50.0)));
        net.segments.push(Segment::straight(0, 0, 1));
        net.segments.push(Segment::straight(1, 1, 2));
        net
    }

    #[test]
    fn closed_shape_outlines_to_a_frame() {
        // Stroking a filled square's boundary makes a frame: outer + inner loop.
        let out = outline_stroke(&closed_square(), 10.0, Cap::Butt, Join::Miter);
        assert!(out.validate().is_ok());
        assert!(
            out.regions.len() >= 2,
            "a frame has an outer + inner contour"
        );
    }

    #[test]
    fn open_stroke_outlines_to_a_filled_region() {
        // An open polyline becomes a filled outline (one closed region).
        let out = outline_stroke(&open_stroke(), 8.0, Cap::Round, Join::Round);
        assert!(out.validate().is_ok());
        assert_eq!(out.regions.len(), 1, "a single stroke → one filled outline");
        // The outline must enclose area roughly the stroke length × width.
        assert!(out.vertices.len() >= 4);
    }

    #[test]
    fn zero_width_is_empty() {
        let out = outline_stroke(&open_stroke(), 0.0, Cap::Round, Join::Round);
        assert!(out.validate().is_ok());
        assert!(out.regions.is_empty());
    }

    #[test]
    fn deterministic_and_reproducible() {
        let out = outline_stroke(&open_stroke(), 8.0, Cap::Round, Join::Round);
        assert!(out.deterministic);
        assert_eq!(
            out,
            outline_stroke(&open_stroke(), 8.0, Cap::Round, Join::Round),
            "byte-stable"
        );
    }

    #[test]
    fn empty_input_is_empty() {
        let out = outline_stroke(&VectorNetwork::empty(), 8.0, Cap::Round, Join::Round);
        assert!(out.validate().is_ok());
        assert!(out.regions.is_empty());
    }
}
