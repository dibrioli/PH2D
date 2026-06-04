#![forbid(unsafe_code)]
//! `ph2d-vector-sdf` — the **draft** half of ADR-0065's draft+reconcile vector
//! boolean. A 2-D signed-distance field of a [`VectorNetwork`]'s filled
//! silhouette, plus the trivial `min/max` boolean combine of two SDFs — the
//! real-time preview during a slider / morph drag (Linesweeper via
//! `ph2d-vector-kurbo` stays the exact, canonical reconcile; ADR-0059 §2.4).
//!
//! ## What this is (and isn't)
//!
//! - **Is:** a silhouette. `network_sdf` rasterizes a network to a signed grid
//!   (negative inside, positive outside, units = world pixels), and
//!   `boolean_sdf` combines two co-located grids with the `min/max` kernel
//!   (ADR-0065 §2.1 — `Union`=`min`, `Subtract`=`max(a,-b)`, `Intersect`=`max`,
//!   `Exclude`, `Outline`). The other four ops (`Merge`/`Crop`/`Divide`/`Trim`)
//!   are topology, not silhouette → Linesweeper only.
//! - **Isn't:** topology. The SDF can't tell two coincident silhouettes apart;
//!   the exact engine owns vertices/regions. The draft is for *interaction* —
//!   the reconcile lands the real network when the drag settles.
//!
//! ## Determinism (ADR-0065 §2.4)
//!
//! Fixed [`SUBDIV_PER_SEGMENT`] flattening + a fixed grid + ordered per-pixel
//! reductions (no parallel min-races) → bit-stable cross-platform. This pure-
//! Rust core is BOTH a CPU silhouette fallback AND the parity oracle for the
//! GPU compute port (`min/max` in WGSL, same algorithm) that lands next.

use glam::Vec2;
use ph2d_vector_doc::{Segment, SegmentId, VectorNetwork, VertexId, WindingRule};
use std::collections::BTreeMap;

pub mod gpu;

/// Cubic subdivisions per segment when flattening to the boundary polyline.
/// FIXED (not adaptive) so the silhouette is bit-stable across platforms — the
/// draft trades a little smoothness for determinism (ADR-0065 §2.4).
pub const SUBDIV_PER_SEGMENT: u32 = 16;

/// Boolean ops representable as an SDF `min/max` combine (ADR-0065 §2.1). The
/// other four — `Merge`/`Crop`/`Divide`/`Trim` — are topology-only (Linesweeper).
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum SdfOp {
    /// `min(a, b)` — everything inside either silhouette.
    Union,
    /// `max(a, -b)` — `a` with `b` carved out.
    Subtract,
    /// `max(a, b)` — only the overlap.
    Intersect,
    /// `max(-min(a, b), min(a, -b))` — symmetric difference (in one xor the other).
    Exclude,
    /// `abs(a) - radius` — a band of half-width `radius` around `a`'s silhouette
    /// (unary; `b` is ignored).
    Outline { radius: f32 },
}

/// Axis-aligned sampling window in network-local (world) pixels.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Bounds {
    pub min: Vec2,
    pub max: Vec2,
}

impl Bounds {
    /// Tight AABB of `net`'s vertices, expanded by `pad` on every side so the
    /// silhouette never clips the grid edge. A unit box for an empty network.
    #[must_use]
    pub fn of_network(net: &VectorNetwork, pad: f32) -> Self {
        let mut min = Vec2::splat(f32::INFINITY);
        let mut max = Vec2::splat(f32::NEG_INFINITY);
        for v in &net.vertices {
            min = min.min(v.pos);
            max = max.max(v.pos);
        }
        if !min.is_finite() || !max.is_finite() {
            return Self {
                min: Vec2::ZERO,
                max: Vec2::ONE,
            };
        }
        Self {
            min: min - Vec2::splat(pad),
            max: max + Vec2::splat(pad),
        }
    }
}

/// A signed-distance field sampled on a `res × res` grid over `bounds`.
/// `data[y * res + x]` is the signed distance at the pixel CENTER; **negative =
/// inside** the filled silhouette, positive = outside (world-pixel units).
#[derive(Clone, Debug, PartialEq)]
pub struct SdfGrid {
    pub res: u32,
    pub bounds: Bounds,
    /// `res * res` signed distances, row-major.
    pub data: Vec<f32>,
}

impl SdfGrid {
    /// Signed distance at grid cell `(x, y)`.
    #[must_use]
    pub fn at(&self, x: u32, y: u32) -> f32 {
        self.data[(y * self.res + x) as usize]
    }

    /// World position of the CENTER of grid cell `(x, y)`.
    #[must_use]
    pub fn cell_center(&self, x: u32, y: u32) -> Vec2 {
        let span = self.bounds.max - self.bounds.min;
        let u = (x as f32 + 0.5) / self.res as f32;
        let v = (y as f32 + 0.5) / self.res as f32;
        self.bounds.min + Vec2::new(span.x * u, span.y * v)
    }
}

/// Evaluate a cubic Bézier `(p0, p1, p2, p3)` at `t ∈ [0, 1]`.
fn cubic(p0: Vec2, p1: Vec2, p2: Vec2, p3: Vec2, t: f32) -> Vec2 {
    let u = 1.0 - t;
    p0 * (u * u * u) + p1 * (3.0 * u * u * t) + p2 * (3.0 * u * t * t) + p3 * (t * t * t)
}

/// Flatten one segment to `SUBDIV_PER_SEGMENT + 1` points from its loop-start to
/// loop-end, honoring traversal direction. The renderer's cubic convention:
/// `c1 = start.pos + out_at_start`, `c2 = end.pos + in_at_end` (mirror of
/// `ph2d_vector_kurbo::network_to_bezpath` + `ph2d_vector::vector_network`).
fn flatten_segment(seg: &Segment, vmap: &BTreeMap<VertexId, Vec2>, forward: bool) -> Vec<Vec2> {
    let Some(&s) = vmap.get(&seg.start) else {
        return Vec::new();
    };
    let Some(&e) = vmap.get(&seg.end) else {
        return Vec::new();
    };
    let (p0, p1, p2, p3) = if forward {
        (s, s + seg.out_at_start, e + seg.in_at_end, e)
    } else {
        // Reversed cubic = B(1-t): control points in reverse order.
        (e, e + seg.in_at_end, s + seg.out_at_start, s)
    };
    (0..=SUBDIV_PER_SEGMENT)
        .map(|i| cubic(p0, p1, p2, p3, i as f32 / SUBDIV_PER_SEGMENT as f32))
        .collect()
}

/// Flatten every region of `net` into a closed directed boundary polyline (the
/// last point repeats the first so edge iteration covers the closing edge).
#[must_use]
pub fn region_loops(net: &VectorNetwork) -> Vec<(Vec<Vec2>, WindingRule)> {
    let vmap: BTreeMap<VertexId, Vec2> = net.vertices.iter().map(|v| (v.id, v.pos)).collect();
    let smap: BTreeMap<SegmentId, &Segment> = net.segments.iter().map(|s| (s.id, s)).collect();

    let mut loops = Vec::with_capacity(net.regions.len());
    for region in &net.regions {
        let mut pts: Vec<Vec2> = Vec::new();
        for &(seg_id, forward) in &region.segments {
            let Some(seg) = smap.get(&seg_id) else {
                continue;
            };
            let poly = flatten_segment(seg, &vmap, forward);
            if pts.is_empty() {
                pts.extend(poly);
            } else {
                // Skip the leading point — it repeats the previous segment's end.
                pts.extend(poly.into_iter().skip(1));
            }
        }
        if pts.len() >= 2 {
            // Ensure the loop is closed for edge / winding iteration.
            if pts.first() != pts.last() {
                pts.push(pts[0]);
            }
            loops.push((pts, region.winding));
        }
    }
    loops
}

/// Flatten `net`'s region boundaries into a flat list of directed edges
/// `[a.x, a.y, b.x, b.y]` — the GPU upload form (and a stable boundary
/// enumeration). Closing edges included.
#[must_use]
pub fn network_edges(net: &VectorNetwork) -> Vec<[f32; 4]> {
    let mut edges = Vec::new();
    for (pts, _winding) in region_loops(net) {
        for w in pts.windows(2) {
            edges.push([w[0].x, w[0].y, w[1].x, w[1].y]);
        }
    }
    edges
}

/// Distance from `p` to the line segment `[a, b]`.
fn point_seg_dist(p: Vec2, a: Vec2, b: Vec2) -> f32 {
    let ab = b - a;
    let len2 = ab.length_squared();
    let t = if len2 > 0.0 {
        ((p - a).dot(ab) / len2).clamp(0.0, 1.0)
    } else {
        0.0
    };
    (a + ab * t).distance(p)
}

/// `> 0` iff `p` is left of the directed edge `a → b`.
fn is_left(a: Vec2, b: Vec2, p: Vec2) -> f32 {
    (b.x - a.x) * (p.y - a.y) - (p.x - a.x) * (b.y - a.y)
}

/// Is `p` inside the closed `loop_pts` under `rule`?
fn point_inside(p: Vec2, loop_pts: &[Vec2], rule: WindingRule) -> bool {
    match rule {
        WindingRule::EvenOdd => {
            // Odd number of rightward-ray crossings.
            let mut crossings = 0u32;
            for w in loop_pts.windows(2) {
                let (a, b) = (w[0], w[1]);
                if (a.y > p.y) != (b.y > p.y) {
                    let t = (p.y - a.y) / (b.y - a.y);
                    if a.x + t * (b.x - a.x) > p.x {
                        crossings += 1;
                    }
                }
            }
            crossings % 2 == 1
        }
        WindingRule::NonZero => {
            // Winding number != 0 (Sunday's algorithm).
            let mut wn = 0i32;
            for w in loop_pts.windows(2) {
                let (a, b) = (w[0], w[1]);
                if a.y <= p.y {
                    if b.y > p.y && is_left(a, b, p) > 0.0 {
                        wn += 1;
                    }
                } else if b.y <= p.y && is_left(a, b, p) < 0.0 {
                    wn -= 1;
                }
            }
            wn != 0
        }
    }
}

/// Signed-distance grid of `net`'s filled silhouette over `bounds` at `res`.
/// Negative inside (any region's winding test), positive outside; magnitude =
/// distance to the nearest boundary edge. Exact for a single-region silhouette
/// (the common draft case — each boolean operand is one source); for a
/// multi-region network the inner shared edges are still counted in the
/// distance (a slight under-estimate inside overlaps), acceptable for a draft.
#[must_use]
pub fn network_sdf(net: &VectorNetwork, res: u32, bounds: Bounds) -> SdfGrid {
    let res = res.max(1);
    let loops = region_loops(net);
    let span = bounds.max - bounds.min;
    let mut data = vec![0.0f32; (res * res) as usize];

    for y in 0..res {
        for x in 0..res {
            let u = (x as f32 + 0.5) / res as f32;
            let v = (y as f32 + 0.5) / res as f32;
            let p = bounds.min + Vec2::new(span.x * u, span.y * v);
            let mut dist = f32::INFINITY;
            let mut inside = false;
            for (pts, rule) in &loops {
                for w in pts.windows(2) {
                    dist = dist.min(point_seg_dist(p, w[0], w[1]));
                }
                if point_inside(p, pts, *rule) {
                    inside = true;
                }
            }
            if !dist.is_finite() {
                dist = 0.0; // empty network → flat zero field
            }
            data[(y * res + x) as usize] = if inside { -dist } else { dist };
        }
    }
    SdfGrid { res, bounds, data }
}

/// Combine two co-located SDF grids with the boolean `op` (the draft boolean —
/// ADR-0065 `min/max`). Returns `None` if `res` / `bounds` differ.
#[must_use]
pub fn boolean_sdf(a: &SdfGrid, b: &SdfGrid, op: SdfOp) -> Option<SdfGrid> {
    if a.res != b.res || a.bounds != b.bounds {
        return None;
    }
    let data = a
        .data
        .iter()
        .zip(&b.data)
        .map(|(&da, &db)| match op {
            SdfOp::Union => da.min(db),
            SdfOp::Subtract => da.max(-db),
            SdfOp::Intersect => da.max(db),
            SdfOp::Exclude => (-(da.min(db))).max(da.min(-db)),
            SdfOp::Outline { radius } => da.abs() - radius,
        })
        .collect();
    Some(SdfGrid {
        res: a.res,
        bounds: a.bounds,
        data,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use ph2d_vector_doc::{Region, Segment, Vertex, VertexKind};

    /// An axis-aligned square `[cx-h, cx+h] × [cy-h, cy+h]` as a one-region
    /// NonZero network (CCW: BL → BR → TR → TL).
    fn square(cx: f32, cy: f32, h: f32) -> VectorNetwork {
        let mut net = VectorNetwork::empty();
        let corners = [
            Vec2::new(cx - h, cy - h),
            Vec2::new(cx + h, cy - h),
            Vec2::new(cx + h, cy + h),
            Vec2::new(cx - h, cy + h),
        ];
        for (i, &p) in corners.iter().enumerate() {
            net.vertices
                .push(Vertex::new(i as VertexId, p, VertexKind::Auto));
        }
        let mut region = Region::new(0, WindingRule::NonZero);
        for i in 0..4u32 {
            let start = i;
            let end = (i + 1) % 4;
            net.segments.push(Segment::straight(i, start, end));
            region.segments.push((i, true));
        }
        net.regions.push(region);
        net
    }

    fn nearest_cell(g: &SdfGrid, world: Vec2) -> (u32, u32) {
        let span = g.bounds.max - g.bounds.min;
        let u = ((world.x - g.bounds.min.x) / span.x * g.res as f32).clamp(0.0, g.res as f32 - 1.0);
        let v = ((world.y - g.bounds.min.y) / span.y * g.res as f32).clamp(0.0, g.res as f32 - 1.0);
        (u as u32, v as u32)
    }

    #[test]
    fn square_sdf_signs_and_magnitude() {
        let net = square(50.0, 50.0, 10.0); // 20×20 square, half-extent 10
        let bounds = Bounds::of_network(&net, 10.0);
        let g = network_sdf(&net, 64, bounds);

        // Center is deepest inside: distance to the nearest edge ≈ 10.
        let (cx, cy) = nearest_cell(&g, Vec2::new(50.0, 50.0));
        let center = g.at(cx, cy);
        assert!(
            center < -8.0,
            "center SDF {center} should be ≈ -10 (inside)"
        );

        // A point well outside is positive.
        let (ox, oy) = nearest_cell(&g, Vec2::new(50.0, 70.0));
        assert!(
            g.at(ox, oy) > 0.0,
            "outside SDF {} should be > 0",
            g.at(ox, oy)
        );

        // A point just inside an edge is inside but shallow.
        let (ex, ey) = nearest_cell(&g, Vec2::new(50.0, 58.5));
        let edge = g.at(ex, ey);
        assert!(
            (-2.5..0.0).contains(&edge),
            "near-edge inside SDF {edge} should be small & negative"
        );
    }

    #[test]
    fn union_and_intersect_of_two_overlapping_squares() {
        // A centered at 45, B at 55, both half-extent 10 → overlap [50..55]² band.
        let a = network_sdf(
            &square(45.0, 50.0, 10.0),
            64,
            Bounds {
                min: Vec2::new(20.0, 20.0),
                max: Vec2::new(80.0, 80.0),
            },
        );
        let b = network_sdf(
            &square(55.0, 50.0, 10.0),
            64,
            Bounds {
                min: Vec2::new(20.0, 20.0),
                max: Vec2::new(80.0, 80.0),
            },
        );
        let uni = boolean_sdf(&a, &b, SdfOp::Union).unwrap();
        let int = boolean_sdf(&a, &b, SdfOp::Intersect).unwrap();

        // A point in the overlap (≈ 50, 50): inside both → union<0 AND intersect<0.
        let (ox, oy) = nearest_cell(&uni, Vec2::new(50.0, 50.0));
        assert!(uni.at(ox, oy) < 0.0, "overlap is inside the union");
        assert!(int.at(ox, oy) < 0.0, "overlap is inside the intersection");

        // A point in A only (≈ 38, 50): inside A, outside B → union<0, intersect>0.
        let (ax, ay) = nearest_cell(&uni, Vec2::new(38.0, 50.0));
        assert!(uni.at(ax, ay) < 0.0, "A-only is inside the union");
        assert!(int.at(ax, ay) > 0.0, "A-only is OUTSIDE the intersection");
    }

    #[test]
    fn subtract_carves_b_out_of_a() {
        let a = network_sdf(
            &square(45.0, 50.0, 10.0),
            64,
            Bounds {
                min: Vec2::new(20.0, 20.0),
                max: Vec2::new(80.0, 80.0),
            },
        );
        let b = network_sdf(
            &square(55.0, 50.0, 10.0),
            64,
            Bounds {
                min: Vec2::new(20.0, 20.0),
                max: Vec2::new(80.0, 80.0),
            },
        );
        let sub = boolean_sdf(&a, &b, SdfOp::Subtract).unwrap();
        // A-only stays inside; the overlap (inside B) is carved out → outside.
        let (ax, ay) = nearest_cell(&sub, Vec2::new(38.0, 50.0));
        assert!(sub.at(ax, ay) < 0.0, "A-only survives the subtract");
        let (ox, oy) = nearest_cell(&sub, Vec2::new(52.0, 50.0));
        assert!(sub.at(ox, oy) > 0.0, "the B overlap is carved out");
    }

    #[test]
    fn outline_is_a_band_around_the_silhouette() {
        let g = network_sdf(
            &square(50.0, 50.0, 10.0),
            64,
            Bounds::of_network(&square(50.0, 50.0, 10.0), 10.0),
        );
        let band = boolean_sdf(&g, &g, SdfOp::Outline { radius: 3.0 }).unwrap();
        // On the edge: abs(sdf) ≈ 0 → band ≈ -3 (inside the outline band).
        let (ex, ey) = nearest_cell(&band, Vec2::new(50.0, 60.0));
        assert!(band.at(ex, ey) < 0.0, "the edge is inside the outline band");
        // Deep inside: abs(sdf) ≈ 10 > 3 → outside the band.
        let (cx, cy) = nearest_cell(&band, Vec2::new(50.0, 50.0));
        assert!(
            band.at(cx, cy) > 0.0,
            "the deep interior is outside the outline band"
        );
    }

    #[test]
    fn mismatched_grids_dont_combine() {
        let g1 = network_sdf(
            &square(50.0, 50.0, 10.0),
            16,
            Bounds::of_network(&square(50.0, 50.0, 10.0), 5.0),
        );
        let g2 = network_sdf(
            &square(50.0, 50.0, 10.0),
            32,
            Bounds::of_network(&square(50.0, 50.0, 10.0), 5.0),
        );
        assert!(boolean_sdf(&g1, &g2, SdfOp::Union).is_none());
    }
}
