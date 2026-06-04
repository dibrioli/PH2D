//! The exact-topology boolean engine: [`VectorNetwork`] ⇄ `kurbo::BezPath` ⇄
//! [`linesweeper`] ⇄ [`VectorNetwork`].
//!
//! This is the **reconcile** (canonical) half of the draft+reconcile pipeline
//! (ADR-0059 §2.4 + ADR-0065): exact planar topology via the Linesweeper
//! sweep-line, run on the CPU as the node's [`crate::MANIFEST`] `eval` output.
//! The SDF GPU *draft* (silhouette, ≤ 0.5 ms) is the renderer-side companion
//! (`crates/ph2d-vector/shaders/boolean_sdf.wgsl`); it is **not** topology and
//! never feeds this path (ADR-0065 §2.3).
//!
//! ## Why Linesweeper
//!
//! Boolean ops over **closed regions** (filled faces) need a robust arrangement
//! of the two boundaries — the hard cases are coincident edges, tangent
//! contact, and shared vertices (T3.5 lens A: "where Clipper fails"). The
//! [`linesweeper`] crate is the engine the spec names canonically
//! (`docs/Vector Module/16_referencias.md`): a robust sweep-line over Bézier
//! paths, kurbo-native (the same path type the `ph2d-vector` renderer already
//! builds). We speak to it in `kurbo::BezPath` and translate the result
//! [`Contours`](linesweeper::topology::Contours) back into our `VectorNetwork`
//! carrier (ADR-0058-amendment-1).
//!
//! ## Curve convention (shared with the renderer)
//!
//! A segment's cubic control points are `c1 = start.pos + out_at_start` and
//! `c2 = end.pos + in_at_end` (tangents are **relative** offset vectors) —
//! identical to `ph2d_vector::build_region_path`, so a network round-trips
//! through the engine with the same geometry the renderer would draw.
//!
//! ## Determinism (cross-OS golden — ADR-0065 §2.4)
//!
//! Linesweeper works in `f64`, whose last ULP can differ across targets. When
//! **both** inputs are flagged `deterministic`, the output coordinates are
//! snapped to the Q16.16 grid ([`snap`]) and the result is flagged
//! `deterministic` — erasing that drift so the bytes are platform-independent,
//! exactly as `vector.source` does for its trig generators.

use std::collections::BTreeMap;

use glam::Vec2;
use kurbo::{BezPath, PathEl, Point};
use linesweeper::{BinaryOp, FillRule, binary_op};
use ph2d_vector_doc::deterministic::snap;
use ph2d_vector_doc::{
    BooleanOp, Region, RegionId, Segment, SegmentId, VectorNetwork, Vertex, VertexId, WindingRule,
};

/// Endpoint-coincidence tolerance used when stitching a contour's closing
/// segment back onto its start vertex. Linesweeper closes a contour on the
/// *exact same* `f64` point it opened it on, so this only guards against last-
/// ULP wobble; it is far below the Q16.16 grid step (`~1.5e-5`).
const CLOSE_EPS: f64 = 1e-9;

#[inline]
fn pt(v: Vec2) -> Point {
    Point::new(f64::from(v.x), f64::from(v.y))
}

#[inline]
fn vec(p: Point) -> Vec2 {
    Vec2::new(p.x as f32, p.y as f32)
}

/// Compute `op` over the filled regions of `a` and `b`, returning a fresh
/// network whose regions are the exact result faces.
///
/// Empty inputs (a network with no fillable regions — e.g. an open spiral) are
/// treated as the empty set: `Union` then yields the other operand, `Subtract`
/// yields `a`, `Intersect` yields nothing, etc. — all of which fall out of the
/// engine naturally from an empty `BezPath`.
#[must_use]
pub fn boolean(a: &VectorNetwork, b: &VectorNetwork, op: BooleanOp) -> VectorNetwork {
    let bez_a = network_to_bezpath(a);
    let bez_b = network_to_bezpath(b);
    let fill = fill_rule_of(a);
    let deterministic = a.deterministic && b.deterministic;

    // Map each frozen `BooleanOp` (9 variants) onto Linesweeper's 4 set ops,
    // composing where Illustrator's vocabulary is richer than raw set algebra.
    // Every arm yields a flat list of result contours (closed loops).
    let contours: Vec<BezPath> = match op {
        // Direct set ops.
        BooleanOp::Union => run(&bez_a, &bez_b, fill, BinaryOp::Union),
        BooleanOp::Subtract => run(&bez_a, &bez_b, fill, BinaryOp::Difference),
        BooleanOp::Intersect => run(&bez_a, &bez_b, fill, BinaryOp::Intersection),
        BooleanOp::Exclude => run(&bez_a, &bez_b, fill, BinaryOp::Xor),

        // Single-fill equivalences (exact for the W3 single-style carrier; the
        // Pathfinder distinction is colour-/operand-bias, not geometry):
        //  - Merge ≡ Union of co-incident faces (no operand bias).
        //  - Crop  ≡ clip A to B's area = Intersection.
        BooleanOp::Merge => run(&bez_a, &bez_b, fill, BinaryOp::Union),
        BooleanOp::Crop => run(&bez_a, &bez_b, fill, BinaryOp::Intersection),

        // Divide: cut both shapes along their mutual boundary into every face —
        // {A∖B} ∪ {A∩B} ∪ {B∖A}, each kept as separate regions.
        BooleanOp::Divide => {
            let mut v = run(&bez_a, &bez_b, fill, BinaryOp::Difference);
            v.extend(run(&bez_a, &bez_b, fill, BinaryOp::Intersection));
            v.extend(run(&bez_b, &bez_a, fill, BinaryOp::Difference));
            v
        }

        // Trim: keep parts of A outside B and parts of B outside A, as separate
        // abutting regions (the overlap is removed; unlike Exclude the two
        // pieces stay distinct rather than merging into one even-odd face).
        BooleanOp::Trim => {
            let mut v = run(&bez_a, &bez_b, fill, BinaryOp::Difference);
            v.extend(run(&bez_b, &bez_a, fill, BinaryOp::Difference));
            v
        }

        // Outline: the merged boundary. W3 emits the Union geometry; true
        // width-expansion stroke-outlining is `vector-outline-stroke` (T3.4+).
        BooleanOp::Outline => run(&bez_a, &bez_b, fill, BinaryOp::Union),
    };

    contours_to_network(&contours, deterministic)
}

/// Run one Linesweeper set op and collect the result faces as closed
/// `BezPath`s. A malformed input (NaN/∞/non-closed — none of which a valid
/// network produces) degrades to *no contours* rather than panicking, since the
/// node's `eval` cannot return a `Result` (matches `vector.source`'s total
/// degradation discipline).
fn run(a: &BezPath, b: &BezPath, fill: FillRule, op: BinaryOp) -> Vec<BezPath> {
    match binary_op(a, b, fill, op) {
        Ok(contours) => contours.contours().map(|c| c.path.clone()).collect(),
        Err(_) => Vec::new(),
    }
}

/// The fill rule to interpret a network's boundaries with — its first region's
/// winding, defaulting to SVG-canonical [`WindingRule::NonZero`] for an
/// empty/region-less network.
fn fill_rule_of(net: &VectorNetwork) -> FillRule {
    match net.regions.first().map(|r| r.winding) {
        Some(WindingRule::EvenOdd) => FillRule::EvenOdd,
        _ => FillRule::NonZero,
    }
}

/// Build a `kurbo::BezPath` containing one closed subpath per fillable region of
/// `net`, using the renderer's cubic control-point convention. Region-less
/// networks (open paths) produce an empty path = the empty set.
fn network_to_bezpath(net: &VectorNetwork) -> BezPath {
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
    fn push_segment(&mut self, start: VertexId, end: VertexId, out_at_start: Vec2, in_at_end: Vec2) {
        let id = self.next_s;
        self.next_s += 1;
        let mut s = Segment::straight(id, start, end);
        s.out_at_start = out_at_start;
        s.in_at_end = in_at_end;
        self.net.segments.push(s);
    }
}

/// Translate the result contours into a `VectorNetwork`: each closed contour →
/// one `NonZero` region (Linesweeper orients faces set-on-left, winding 1, so
/// non-zero fill is correct and holes — emitted as separate oppositely-wound
/// contours — render correctly). Regions are left `fill = None`, matching the
/// `vector.source` generator convention (the asset / bridge assigns the fill).
fn contours_to_network(contours: &[BezPath], deterministic: bool) -> VectorNetwork {
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
    let seg_base = b.next_s;
    let mut closed = false;

    // Helper closure can't borrow `b` mutably and capture, so inline the steps.
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
                close_or_extend(b, &mut refs, &mut cur_v, &mut cur_pos, &mut closed, sv, start_pos, p, Vec2::ZERO, Vec2::ZERO);
            }
            PathEl::QuadTo(c, p) => {
                let Some(sv) = start_v else { continue };
                // Elevate the quadratic to a cubic: the two cubic controls sit
                // 2/3 of the way from each endpoint toward the quad control.
                let out = (vec(c) - vec(cur_pos)) * (2.0 / 3.0);
                let inn = (vec(c) - vec(p)) * (2.0 / 3.0);
                close_or_extend(b, &mut refs, &mut cur_v, &mut cur_pos, &mut closed, sv, start_pos, p, out, inn);
            }
            PathEl::CurveTo(c1, c2, p) => {
                let Some(sv) = start_v else { continue };
                let out = vec(c1) - vec(cur_pos);
                let inn = vec(c2) - vec(p);
                close_or_extend(b, &mut refs, &mut cur_v, &mut cur_pos, &mut closed, sv, start_pos, p, out, inn);
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
    // degenerate 0/1-segment contour is dropped rather than producing an
    // invalid region. (`seg_base` would let us roll back vertices too, but
    // orphan vertices are harmless to `validate` and the renderer ignores them.)
    let _ = seg_base;
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
/// grid — the cross-OS determinism guarantee (ADR-0065 §2.4), identical to the
/// snapping `vector.source` applies to its generators.
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

    /// An axis-aligned square `[lo,hi]²` as a deterministic single-region
    /// network (the `vector.source` rect generator + the determinism flag).
    fn square(lo: f32, hi: f32) -> VectorNetwork {
        let mut net = primitives::rect(Vec2::new(lo, lo), Vec2::new(hi, hi));
        net.deterministic = true;
        net
    }

    /// Axis-aligned bounding box of all vertex positions.
    fn bbox(net: &VectorNetwork) -> (Vec2, Vec2) {
        let mut min = Vec2::splat(f32::INFINITY);
        let mut max = Vec2::splat(f32::NEG_INFINITY);
        for v in &net.vertices {
            min = min.min(v.pos);
            max = max.max(v.pos);
        }
        (min, max)
    }

    fn approx(a: Vec2, b: Vec2) -> bool {
        (a - b).length() < 0.01
    }

    /// Every coordinate lies exactly on the Q16.16 grid — the cross-OS golden
    /// property.
    fn grid_snapped(net: &VectorNetwork) -> bool {
        let on = |v: Vec2| v == Vec2::new(snap(v.x), snap(v.y));
        net.vertices.iter().all(|v| on(v.pos))
            && net
                .segments
                .iter()
                .all(|s| on(s.out_at_start) && on(s.in_at_end))
    }

    const ALL_OPS: [BooleanOp; 9] = [
        BooleanOp::Union,
        BooleanOp::Subtract,
        BooleanOp::Intersect,
        BooleanOp::Exclude,
        BooleanOp::Divide,
        BooleanOp::Trim,
        BooleanOp::Merge,
        BooleanOp::Crop,
        BooleanOp::Outline,
    ];

    #[test]
    fn union_of_overlapping_squares_is_one_region() {
        // [0,2]² ∪ [1,3]² → one connected staircase, spanning [0,3]².
        let out = boolean(&square(0.0, 2.0), &square(1.0, 3.0), BooleanOp::Union);
        assert!(out.validate().is_ok());
        assert_eq!(out.regions.len(), 1);
        let (min, max) = bbox(&out);
        assert!(approx(min, Vec2::new(0.0, 0.0)) && approx(max, Vec2::new(3.0, 3.0)));
    }

    #[test]
    fn intersect_of_overlapping_squares_is_the_overlap() {
        // [0,2]² ∩ [1,3]² = exactly [1,2]².
        let out = boolean(&square(0.0, 2.0), &square(1.0, 3.0), BooleanOp::Intersect);
        assert!(out.validate().is_ok());
        assert_eq!(out.regions.len(), 1);
        let (min, max) = bbox(&out);
        assert!(
            approx(min, Vec2::new(1.0, 1.0)) && approx(max, Vec2::new(2.0, 2.0)),
            "intersection bbox was {min:?}..{max:?}, expected [1,1]..[2,2]"
        );
    }

    #[test]
    fn subtract_is_a_minus_b() {
        // [0,2]² ∖ [1,3]² = an L-shape still bounded by [0,2]².
        let out = boolean(&square(0.0, 2.0), &square(1.0, 3.0), BooleanOp::Subtract);
        assert!(out.validate().is_ok());
        assert_eq!(out.regions.len(), 1);
        let (min, max) = bbox(&out);
        assert!(approx(min, Vec2::new(0.0, 0.0)) && approx(max, Vec2::new(2.0, 2.0)));
    }

    #[test]
    fn intersect_of_disjoint_squares_is_empty() {
        let out = boolean(&square(0.0, 1.0), &square(2.0, 3.0), BooleanOp::Intersect);
        assert!(out.validate().is_ok());
        assert!(out.regions.is_empty(), "disjoint intersection is empty");
    }

    #[test]
    fn union_of_disjoint_squares_is_two_regions() {
        let out = boolean(&square(0.0, 1.0), &square(2.0, 3.0), BooleanOp::Union);
        assert!(out.validate().is_ok());
        assert_eq!(out.regions.len(), 2, "disjoint union keeps two regions");
    }

    #[test]
    fn divide_of_overlap_yields_three_faces() {
        // {A∖B} ∪ {A∩B} ∪ {B∖A} — three distinct faces.
        let out = boolean(&square(0.0, 2.0), &square(1.0, 3.0), BooleanOp::Divide);
        assert!(out.validate().is_ok());
        assert_eq!(out.regions.len(), 3, "divide cuts into three faces");
    }

    #[test]
    fn every_op_produces_a_valid_network() {
        // DoD: all 9 variants run through Linesweeper without an invalid network
        // or a panic — both for overlapping and disjoint operands.
        for op in ALL_OPS {
            for (a, b) in [
                (square(0.0, 2.0), square(1.0, 3.0)), // overlapping
                (square(0.0, 1.0), square(2.0, 3.0)), // disjoint
            ] {
                let out = boolean(&a, &b, op);
                assert!(out.validate().is_ok(), "op {op:?} produced an invalid network");
            }
        }
    }

    #[test]
    fn deterministic_inputs_snap_and_reproduce() {
        let a = square(0.0, 2.0);
        let b = square(1.0, 3.0);
        let out = boolean(&a, &b, BooleanOp::Union);
        assert!(out.deterministic, "both inputs deterministic → output too");
        assert!(grid_snapped(&out), "output must be Q16.16-snapped");
        // Byte-identical on re-run (the cross-OS golden guarantee, given snap).
        assert_eq!(out, boolean(&a, &b, BooleanOp::Union));
    }

    #[test]
    fn non_deterministic_input_leaves_output_unflagged() {
        let mut a = square(0.0, 2.0);
        a.deterministic = false;
        let out = boolean(&a, &square(1.0, 3.0), BooleanOp::Union);
        assert!(!out.deterministic);
    }

    #[test]
    fn empty_inputs_do_not_panic() {
        let out = boolean(&VectorNetwork::empty(), &VectorNetwork::empty(), BooleanOp::Union);
        assert!(out.validate().is_ok());
        assert!(out.regions.is_empty());
    }

    #[test]
    fn union_with_empty_returns_the_other_operand() {
        // A ∪ ∅ = A (one region, bounded by A's box).
        let out = boolean(&square(0.0, 2.0), &VectorNetwork::empty(), BooleanOp::Union);
        assert!(out.validate().is_ok());
        assert_eq!(out.regions.len(), 1);
        let (min, max) = bbox(&out);
        assert!(approx(min, Vec2::new(0.0, 0.0)) && approx(max, Vec2::new(2.0, 2.0)));
    }
}
