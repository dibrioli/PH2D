//! Procedural shape primitives — rect / ellipse / polygon / star / spiral.
//!
//! Per plan **T2.2** (`docs/Vector Module/17_plano_de_implementacao.md` §5):
//! the Shape tool (`ph2d-tool-vector-shape`) drives these five generators
//! with a drag (bounding box / radius). Per **T3.2** the same generators
//! are consolidated into the `vector-source` graph node (W3) — so the
//! geometry lives here, in the data-model crate, and both consumers share
//! one implementation (no W3 rewrite).
//!
//! ## Output convention
//!
//! Each generator returns a self-contained [`VectorNetwork`]:
//! - **Closed shapes** (rect / ellipse / polygon / star) emit one
//!   [`Region`] (NonZero winding) wrapping every segment forward, with
//!   `fill = None` — the caller (tool / node) assigns the fill ref.
//! - **Spiral** is an **open** path: vertices + segments, **no region**.
//!   The caller assigns a stroke `style_ref` (it renders via the
//!   `draw_vector_network` stroke-pass).
//!
//! Vertex / segment / region ids start at 0 and are dense, so the network
//! validates and the caller can append an [`crate::EditLog`] by replaying
//! the geometry if it wants op-level provenance.
//!
//! ## Bounds (frozen)
//!
//! `sides` clamps to `[3, MAX_POLYGON_SIDES]`, star `points` to
//! `[2, MAX_POLYGON_SIDES/2]` (so `2·points ≤ MAX_POLYGON_SIDES`), spiral
//! `turns` to `(0, MAX_SPIRAL_TURNS]` and `samples_per_turn` to `[4, 64]`
//! — the same `architecture_vector_contract_surface` ceilings the LLM-gen
//! path enforces ([`crate::postcard_schema`]).
//!
//! ## Determinism (HR-5)
//!
//! The angular generators use `f64` `cos`/`sin`. Output is deterministic
//! **on a given target**; cross-target bit-identical golden output (the
//! W3 T3.2 DoD) requires snapping vertex coords through
//! [`crate::deterministic::snap`] when the destination network is
//! `deterministic == true` — same caller contract as
//! [`crate::cubic_fit`] / [`crate::hobby`]. The math never panics and
//! never emits NaN for finite input.

use glam::Vec2;

use crate::cubic::Vertex;
use crate::network::VectorNetwork;
use crate::postcard_schema::{MAX_POLYGON_SIDES, MAX_SPIRAL_TURNS};
use crate::region::{Region, SegmentRef, WindingRule};
use crate::style::Segment;

/// Cubic control-arm fraction for a 90° circular/elliptic arc
/// (`4/3·(√2 − 1)`), the standard "kappa" Bézier circle approximation
/// (max radial error ≈ 0.0273 %).
const KAPPA: f64 = 0.552_284_749_830_793_4;

/// Minimum / maximum spiral sampling density (vertices per turn).
const MIN_SPIRAL_SAMPLES_PER_TURN: u32 = 4;
const MAX_SPIRAL_SAMPLES_PER_TURN: u32 = 64;

/// Axis-aligned rectangle from two opposite corners (order-independent).
/// 4 straight segments, one NonZero region.
#[must_use]
pub fn rect(corner_a: Vec2, corner_b: Vec2) -> VectorNetwork {
    let x0 = corner_a.x.min(corner_b.x);
    let x1 = corner_a.x.max(corner_b.x);
    let y0 = corner_a.y.min(corner_b.y);
    let y1 = corner_a.y.max(corner_b.y);
    let corners = [
        Vec2::new(x0, y0),
        Vec2::new(x1, y0),
        Vec2::new(x1, y1),
        Vec2::new(x0, y1),
    ];
    closed_polyline(&corners)
}

/// Axis-aligned rectangle with rounded corners, from two opposite corners
/// (order-independent). Four straight edges joined by four cubic quarter-arc
/// corners (the same kappa approximation as [`ellipse`]); one NonZero region.
/// `radius` clamps to `[0, min(width, height)/2]` so the arcs never cross; a
/// near-zero radius falls back to a sharp [`rect`] (4 corners, no curve).
#[must_use]
pub fn rounded_rect(corner_a: Vec2, corner_b: Vec2, radius: f32) -> VectorNetwork {
    let x0 = corner_a.x.min(corner_b.x);
    let x1 = corner_a.x.max(corner_b.x);
    let y0 = corner_a.y.min(corner_b.y);
    let y1 = corner_a.y.max(corner_b.y);
    // Clamp to half the shorter side; a degenerate radius is a sharp rect.
    let r = radius.max(0.0).min((x1 - x0).min(y1 - y0) * 0.5);
    if r <= f32::EPSILON {
        return rect(corner_a, corner_b);
    }
    let k = (f64::from(r) * KAPPA) as f32; // cubic control-arm length

    // Eight vertices clockwise (y-down screen space), two per corner — each
    // rounded corner begins where the adjacent straight edge ends:
    //   v0 ─top→ v1 ╮         v0=(x0+r,y0)  v1=(x1-r,y0)
    //   ↑TL        ↓TR        v2=(x1,y0+r)  v3=(x1,y1-r)
    //   v7         v2         v4=(x1-r,y1)  v5=(x0+r,y1)
    //   ╰BL ←bot─ ╯BR         v6=(x0,y1-r)  v7=(x0,y0+r)
    let pts = [
        Vec2::new(x0 + r, y0),
        Vec2::new(x1 - r, y0),
        Vec2::new(x1, y0 + r),
        Vec2::new(x1, y1 - r),
        Vec2::new(x1 - r, y1),
        Vec2::new(x0 + r, y1),
        Vec2::new(x0, y1 - r),
        Vec2::new(x0, y0 + r),
    ];
    let mut net = VectorNetwork::empty();
    for (i, &p) in pts.iter().enumerate() {
        net.vertices.push(Vertex::auto(i as u32, p));
    }
    // (out_at_start, in_at_end) per segment: straight edges have zero tangents;
    // each corner arc uses kappa tangents (c1 = start + out, c2 = end + in),
    // turning clockwise — `out` along the travel direction at the arc start,
    // `in` back along it at the arc end (cf. `ellipse`).
    let z = Vec2::ZERO;
    let tangents = [
        (z, z),                                   // s0 v0→v1 top edge
        (Vec2::new(k, 0.0), Vec2::new(0.0, -k)),  // s1 v1→v2 TR arc
        (z, z),                                   // s2 v2→v3 right edge
        (Vec2::new(0.0, k), Vec2::new(k, 0.0)),   // s3 v3→v4 BR arc
        (z, z),                                   // s4 v4→v5 bottom edge
        (Vec2::new(-k, 0.0), Vec2::new(0.0, k)),  // s5 v5→v6 BL arc
        (z, z),                                   // s6 v6→v7 left edge
        (Vec2::new(0.0, -k), Vec2::new(-k, 0.0)), // s7 v7→v0 TL arc
    ];
    let n = pts.len() as u32;
    for (i, (out_at_start, in_at_end)) in tangents.into_iter().enumerate() {
        net.segments.push(Segment {
            id: i as u32,
            start: i as u32,
            end: (i as u32 + 1) % n,
            out_at_start,
            in_at_end,
            style_ref: None,
        });
    }
    push_region_over_all_segments(&mut net);
    net
}

/// Ellipse inscribed in the box `center ± radii`. Four cubic quarter-arcs
/// (kappa approximation), one NonZero region. A near-zero radius collapses
/// to a degenerate-but-finite network.
#[must_use]
pub fn ellipse(center: Vec2, radii: Vec2) -> VectorNetwork {
    let rx = radii.x.abs();
    let ry = radii.y.abs();
    let kx = (f64::from(rx) * KAPPA) as f32;
    let ky = (f64::from(ry) * KAPPA) as f32;

    // Axis-extreme vertices (CW in y-down screen space): right, bottom,
    // left, top.
    let right = Vec2::new(center.x + rx, center.y);
    let bottom = Vec2::new(center.x, center.y + ry);
    let left = Vec2::new(center.x - rx, center.y);
    let top = Vec2::new(center.x, center.y - ry);

    let mut net = VectorNetwork::empty();
    for (i, &p) in [right, bottom, left, top].iter().enumerate() {
        net.vertices.push(Vertex::auto(i as u32, p));
    }
    // Quarter-arc tangents (offset vectors; c1 = start + out, c2 = end + in).
    let tangents = [
        (Vec2::new(0.0, ky), Vec2::new(kx, 0.0)),   // right → bottom
        (Vec2::new(-kx, 0.0), Vec2::new(0.0, ky)),  // bottom → left
        (Vec2::new(0.0, -ky), Vec2::new(-kx, 0.0)), // left → top
        (Vec2::new(kx, 0.0), Vec2::new(0.0, -ky)),  // top → right
    ];
    for (i, (out_at_start, in_at_end)) in tangents.into_iter().enumerate() {
        let end = ((i + 1) % 4) as u32;
        net.segments.push(Segment {
            id: i as u32,
            start: i as u32,
            end,
            out_at_start,
            in_at_end,
            style_ref: None,
        });
    }
    push_region_over_all_segments(&mut net);
    net
}

/// Regular convex polygon: `sides` vertices on a circle of `radius`
/// around `center`, the first at angle `rotation` (radians). `sides`
/// clamps to `[3, MAX_POLYGON_SIDES]`.
#[must_use]
pub fn polygon(center: Vec2, radius: f32, sides: u32, rotation: f32) -> VectorNetwork {
    let n = sides.clamp(3, MAX_POLYGON_SIDES);
    let pts: Vec<Vec2> = (0..n)
        .map(|i| {
            let a = f64::from(rotation) + std::f64::consts::TAU * f64::from(i) / f64::from(n);
            point_on_circle(center, radius, a)
        })
        .collect();
    closed_polyline(&pts)
}

/// Star: `points` outer tips on `outer_radius` interleaved with `points`
/// inner vertices on `inner_radius`, around `center`, first tip at angle
/// `rotation`. `points` clamps to `[2, MAX_POLYGON_SIDES/2]` (so the
/// `2·points` vertices stay within the polygon-sides ceiling).
#[must_use]
pub fn star(
    center: Vec2,
    outer_radius: f32,
    inner_radius: f32,
    points: u32,
    rotation: f32,
) -> VectorNetwork {
    let p = points.clamp(2, MAX_POLYGON_SIDES / 2);
    let n = p * 2;
    let pts: Vec<Vec2> = (0..n)
        .map(|i| {
            let a = f64::from(rotation) + std::f64::consts::TAU * f64::from(i) / f64::from(n);
            let r = if i % 2 == 0 {
                outer_radius
            } else {
                inner_radius
            };
            point_on_circle(center, r, a)
        })
        .collect();
    closed_polyline(&pts)
}

/// Archimedean spiral: radius grows linearly from `inner_radius` to
/// `outer_radius` over `turns` full revolutions, starting at angle
/// `rotation`. **Open** path (no region) — the caller strokes it.
/// `turns` clamps to `(0, MAX_SPIRAL_TURNS]`, `samples_per_turn` to
/// `[4, 64]`.
#[must_use]
pub fn spiral(
    center: Vec2,
    inner_radius: f32,
    outer_radius: f32,
    turns: f32,
    samples_per_turn: u32,
    rotation: f32,
) -> VectorNetwork {
    let turns = turns.clamp(f32::MIN_POSITIVE, MAX_SPIRAL_TURNS as f32);
    let spt = samples_per_turn.clamp(MIN_SPIRAL_SAMPLES_PER_TURN, MAX_SPIRAL_SAMPLES_PER_TURN);
    let total = ((f64::from(turns) * f64::from(spt)).ceil() as u32).max(1) + 1;

    let mut net = VectorNetwork::empty();
    for i in 0..total {
        let t = f64::from(i) / f64::from(total - 1); // 0..=1 (total ≥ 2)
        let angle = f64::from(rotation) + std::f64::consts::TAU * f64::from(turns) * t;
        let r = f64::from(inner_radius) + (f64::from(outer_radius) - f64::from(inner_radius)) * t;
        let p = Vec2::new(
            center.x + (r * angle.cos()) as f32,
            center.y + (r * angle.sin()) as f32,
        );
        net.vertices.push(Vertex::auto(i, p));
    }
    // Straight open chain (no closing segment, no region).
    for i in 0..total - 1 {
        net.segments.push(Segment::straight(i, i, i + 1));
    }
    net
}

/// Build a closed network from an ordered ring of points: one vertex per
/// point, straight segments `i → (i+1) mod n` (including the closing
/// edge), and one NonZero region wrapping all segments forward.
fn closed_polyline(pts: &[Vec2]) -> VectorNetwork {
    let mut net = VectorNetwork::empty();
    let n = pts.len();
    if n < 2 {
        return net;
    }
    for (i, &p) in pts.iter().enumerate() {
        net.vertices.push(Vertex::auto(i as u32, p));
    }
    for i in 0..n {
        let end = ((i + 1) % n) as u32;
        net.segments
            .push(Segment::straight(i as u32, i as u32, end));
    }
    push_region_over_all_segments(&mut net);
    net
}

/// Append one NonZero region wrapping every segment in insertion order
/// (all forward). Id 0.
fn push_region_over_all_segments(net: &mut VectorNetwork) {
    let mut region = Region::new(0, WindingRule::NonZero);
    region.segments = net
        .segments
        .iter()
        .map(|s| (s.id, true) as SegmentRef)
        .collect();
    net.regions.push(region);
}

#[inline]
fn point_on_circle(center: Vec2, radius: f32, angle: f64) -> Vec2 {
    Vec2::new(
        center.x + (f64::from(radius) * angle.cos()) as f32,
        center.y + (f64::from(radius) * angle.sin()) as f32,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn eval_cubic_seg(net: &VectorNetwork, seg_idx: usize, t: f32) -> Vec2 {
        let s = &net.segments[seg_idx];
        let p0 = net.vertices.iter().find(|v| v.id == s.start).unwrap().pos;
        let p3 = net.vertices.iter().find(|v| v.id == s.end).unwrap().pos;
        let c1 = p0 + s.out_at_start;
        let c2 = p3 + s.in_at_end;
        let mt = 1.0 - t;
        p0 * (mt * mt * mt) + c1 * (3.0 * mt * mt * t) + c2 * (3.0 * mt * t * t) + p3 * (t * t * t)
    }

    #[test]
    fn rect_is_four_corners_one_region() {
        let net = rect(Vec2::new(10.0, 20.0), Vec2::new(110.0, 70.0));
        assert_eq!(net.vertices.len(), 4);
        assert_eq!(net.segments.len(), 4);
        assert_eq!(net.regions.len(), 1);
        assert!(net.validate().is_ok());
        // Corners in TL, TR, BR, BL order.
        assert_eq!(net.vertices[0].pos, Vec2::new(10.0, 20.0));
        assert_eq!(net.vertices[2].pos, Vec2::new(110.0, 70.0));
    }

    #[test]
    fn rect_normalizes_swapped_corners() {
        let a = rect(Vec2::new(110.0, 70.0), Vec2::new(10.0, 20.0));
        let b = rect(Vec2::new(10.0, 20.0), Vec2::new(110.0, 70.0));
        assert_eq!(a.vertices[0].pos, b.vertices[0].pos);
        assert_eq!(a.vertices[2].pos, b.vertices[2].pos);
    }

    #[test]
    fn rounded_rect_has_eight_verts_and_four_cubic_corners() {
        let net = rounded_rect(Vec2::new(0.0, 0.0), Vec2::new(100.0, 60.0), 20.0);
        assert_eq!(net.vertices.len(), 8);
        assert_eq!(net.segments.len(), 8, "4 edges + 4 corner arcs");
        assert_eq!(net.regions.len(), 1);
        assert!(net.validate().is_ok());
        // Odd segments are the curved corners; even segments are straight edges.
        for i in [1usize, 3, 5, 7] {
            assert!(
                net.segments[i].out_at_start.length() > 1e-3,
                "corner {i} should be curved"
            );
        }
        for i in [0usize, 2, 4, 6] {
            assert_eq!(net.segments[i].out_at_start, Vec2::ZERO, "edge {i} curved");
            assert_eq!(net.segments[i].in_at_end, Vec2::ZERO, "edge {i} curved");
        }
        // The TR arc (segment 1) midpoint lies ~r from the corner centre.
        let centre = Vec2::new(100.0 - 20.0, 20.0);
        let m = eval_cubic_seg(&net, 1, 0.5);
        assert!(
            ((m - centre).length() - 20.0).abs() < 0.2,
            "TR arc midpoint off the radius"
        );
    }

    #[test]
    fn rounded_rect_clamps_radius_and_degenerates_to_sharp() {
        // A radius past half the short side clamps so the arcs never cross.
        let big = rounded_rect(Vec2::ZERO, Vec2::new(100.0, 40.0), 999.0);
        assert!(big.validate().is_ok());
        // Zero radius is exactly the sharp rect: four corners, no curve.
        let sharp = rounded_rect(Vec2::ZERO, Vec2::splat(50.0), 0.0);
        assert_eq!(sharp.vertices.len(), 4);
        assert!(sharp.segments.iter().all(|s| s.out_at_start == Vec2::ZERO));
    }

    #[test]
    fn ellipse_has_four_cubic_arcs_on_the_curve() {
        let c = Vec2::new(50.0, 40.0);
        let r = Vec2::new(30.0, 20.0);
        let net = ellipse(c, r);
        assert_eq!(net.vertices.len(), 4);
        assert_eq!(net.segments.len(), 4);
        assert_eq!(net.regions.len(), 1);
        assert!(net.validate().is_ok());
        // Cubic arcs (non-zero tangents).
        assert!(net.segments[0].out_at_start.length() > 1e-3);
        // Midpoint of each arc lies on the ellipse to < 0.5 px (kappa error).
        for i in 0..4 {
            let m = eval_cubic_seg(&net, i, 0.5);
            let nx = f64::from(m.x - c.x) / f64::from(r.x);
            let ny = f64::from(m.y - c.y) / f64::from(r.y);
            let on = nx * nx + ny * ny;
            assert!(
                (on - 1.0).abs() < 0.01,
                "arc {i} midpoint off ellipse: {on}"
            );
        }
    }

    #[test]
    fn polygon_vertex_count_and_clamp() {
        assert_eq!(polygon(Vec2::ZERO, 50.0, 5, 0.0).vertices.len(), 5);
        assert_eq!(
            polygon(Vec2::ZERO, 50.0, 2, 0.0).vertices.len(),
            3,
            "clamp up to 3"
        );
        assert_eq!(
            polygon(Vec2::ZERO, 50.0, 999, 0.0).vertices.len(),
            MAX_POLYGON_SIDES as usize,
            "clamp down to MAX_POLYGON_SIDES"
        );
        assert!(polygon(Vec2::ZERO, 50.0, 6, 0.0).validate().is_ok());
    }

    #[test]
    fn polygon_vertices_lie_on_the_circle() {
        let net = polygon(Vec2::new(5.0, 5.0), 40.0, 7, 0.3);
        for v in &net.vertices {
            let d = (v.pos - Vec2::new(5.0, 5.0)).length();
            assert!((d - 40.0).abs() < 1e-2, "vertex off circle: r={d}");
        }
    }

    #[test]
    fn star_alternates_radii_and_clamps() {
        let c = Vec2::ZERO;
        let net = star(c, 50.0, 20.0, 5, 0.0);
        assert_eq!(net.vertices.len(), 10, "2 * points");
        assert_eq!(net.regions.len(), 1);
        assert!(net.validate().is_ok());
        // Even indices = outer radius, odd = inner.
        assert!((net.vertices[0].pos.length() - 50.0).abs() < 1e-2);
        assert!((net.vertices[1].pos.length() - 20.0).abs() < 1e-2);
        // points clamps so 2*points <= MAX_POLYGON_SIDES.
        let big = star(c, 50.0, 20.0, 999, 0.0);
        assert!(big.vertices.len() <= MAX_POLYGON_SIDES as usize);
    }

    #[test]
    fn spiral_is_open_with_growing_radius() {
        let c = Vec2::new(100.0, 100.0);
        let net = spiral(c, 5.0, 60.0, 3.0, 24, 0.0);
        assert!(net.regions.is_empty(), "spiral is an open path — no region");
        assert_eq!(net.segments.len(), net.vertices.len() - 1, "open chain");
        assert!(net.validate().is_ok());
        // Radius is monotonically non-decreasing from inner to outer.
        let r_first = (net.vertices[0].pos - c).length();
        let r_last = (net.vertices[net.vertices.len() - 1].pos - c).length();
        assert!(r_first < r_last, "spiral should grow outward");
        assert!((r_first - 5.0).abs() < 1.0 && (r_last - 60.0).abs() < 1.0);
    }

    #[test]
    fn spiral_clamps_turns_and_samples() {
        // Huge turns clamps to MAX_SPIRAL_TURNS; the vertex count stays
        // bounded and the network validates.
        let net = spiral(Vec2::ZERO, 1.0, 100.0, 9999.0, 9999, 0.0);
        assert!(net.validate().is_ok());
        // turns ≤ 64, spt ≤ 64 → ≤ 64*64+1 vertices.
        assert!(
            net.vertices.len() <= (MAX_SPIRAL_TURNS * MAX_SPIRAL_SAMPLES_PER_TURN + 1) as usize
        );
    }

    #[test]
    fn all_closed_shapes_have_no_fill_assigned() {
        // Generators leave fill = None — the tool / node assigns it.
        for net in [
            rect(Vec2::ZERO, Vec2::splat(10.0)),
            ellipse(Vec2::ZERO, Vec2::splat(10.0)),
            polygon(Vec2::ZERO, 10.0, 6, 0.0),
            star(Vec2::ZERO, 10.0, 5.0, 5, 0.0),
        ] {
            assert_eq!(net.regions[0].fill, None);
            for s in &net.segments {
                assert_eq!(s.style_ref, None);
            }
        }
    }

    #[test]
    fn deterministic_across_calls() {
        let a = polygon(Vec2::new(3.0, 7.0), 25.0, 9, 0.7);
        let b = polygon(Vec2::new(3.0, 7.0), 25.0, 9, 0.7);
        assert_eq!(a, b);
        let s1 = spiral(Vec2::ZERO, 2.0, 40.0, 4.0, 16, 0.1);
        let s2 = spiral(Vec2::ZERO, 2.0, 40.0, 4.0, 16, 0.1);
        assert_eq!(s1, s2);
    }
}
