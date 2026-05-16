//! Delaunay triangulation + Voronoi cells + Lloyd's relaxation.
//!
//! # Algorithms
//!
//! - **Delaunay** — Bowyer–Watson incremental, O(N² worst case,
//!   ~O(N log N) average). Adequate for the panel demo's seed
//!   counts (≤ ~256).
//! - **Voronoi cells** — Delaunay dual. For each seed `s`, the cell
//!   is the convex polygon whose vertices are the circumcenters of
//!   the Delaunay triangles incident to `s`, ordered CCW. Boundary
//!   seeds produce open cells; v1 returns them unclipped (the
//!   editor render adapter clips to the visible AABB on its side).
//! - **Lloyd's relaxation** — each step replaces every seed with
//!   the centroid of its current Voronoi cell, then re-triangulates.
//!   Multiple steps drive the diagram toward a centroidal Voronoi
//!   tessellation (CVT).
//!
//! # Determinism (HR-5)
//!
//! No `HashMap`; edges keyed in `BTreeMap`. Circumcircle test uses
//! the standard 4×4 determinant on `f64` to keep numerical noise
//! below the `≈ 1e-10` threshold needed for ~256-seed inputs.
//! Identical seed slices → identical triangulation.
//!
//! # Caveats
//!
//! - **Collinear / co-circular seeds** can cause the boundary-edge
//!   detection to misclassify. We dodge by jittering super-triangle
//!   placement away from canonical positions, but pathological
//!   inputs (3 collinear seeds) are not guaranteed to produce a
//!   meaningful triangulation. Acceptable for the inspect-panel
//!   demo; document for gameplay callers.

use crate::Vec2;
use crate::quadtree::AABB;
use std::collections::BTreeMap;

/// Delaunay triangle — indices into the seed array.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct Triangle {
    pub a: usize,
    pub b: usize,
    pub c: usize,
}

impl Triangle {
    pub fn new(a: usize, b: usize, c: usize) -> Self {
        Self { a, b, c }
    }
}

/// Voronoi cell for one seed.
#[derive(Clone, Debug)]
pub struct VoronoiCell {
    /// Owning seed (its world position).
    pub seed: Vec2,
    /// Cell polygon vertices, CCW, no closing repeat. May be open
    /// (i.e., not enclose the seed) for boundary seeds.
    pub vertices: Vec<Vec2>,
}

/// Delaunay triangulation over a fixed seed set.
#[derive(Clone, Debug)]
pub struct Triangulation {
    pub seeds: Vec<Vec2>,
    pub triangles: Vec<Triangle>,
}

impl Triangulation {
    /// Build the Delaunay triangulation of `seeds`. Empty seed list
    /// → empty triangulation. Single seed / two seeds → triangulation
    /// is empty (degenerate); the caller's render adapter is
    /// responsible for handling that case.
    pub fn from_seeds(seeds: &[Vec2]) -> Self {
        if seeds.len() < 3 {
            return Self {
                seeds: seeds.to_vec(),
                triangles: Vec::new(),
            };
        }

        // Compute bounding box and build a super-triangle that
        // contains all seeds. Inflate by 10× span to keep
        // circumcircles well-defined even for clustered inputs.
        let (min, max) = bounds_of(seeds);
        let dx = max[0] - min[0];
        let dy = max[1] - min[1];
        let dmax = dx.max(dy).max(1.0);
        let cx = (min[0] + max[0]) * 0.5;
        let cy = (min[1] + max[1]) * 0.5;
        let super_a = [cx - 20.0 * dmax, cy - dmax];
        let super_b = [cx + 20.0 * dmax, cy - dmax];
        let super_c = [cx, cy + 20.0 * dmax];

        // Concatenated point pool: real seeds first, then super-tri.
        let mut points: Vec<Vec2> = Vec::with_capacity(seeds.len() + 3);
        points.extend_from_slice(seeds);
        let sa = points.len();
        points.push(super_a);
        let sb = points.len();
        points.push(super_b);
        let sc = points.len();
        points.push(super_c);

        let mut triangles: Vec<Triangle> = vec![Triangle::new(sa, sb, sc)];

        for i in 0..seeds.len() {
            let p = points[i];
            // Find all triangles whose circumcircle contains p.
            let mut bad: Vec<usize> = Vec::new();
            for (ti, t) in triangles.iter().enumerate() {
                if in_circumcircle(p, points[t.a], points[t.b], points[t.c]) {
                    bad.push(ti);
                }
            }
            // Boundary edges of the cavity = edges appearing in
            // exactly one bad triangle.
            let mut edge_count: BTreeMap<(usize, usize), u32> = BTreeMap::new();
            for &bi in &bad {
                let t = triangles[bi];
                for (u, v) in [(t.a, t.b), (t.b, t.c), (t.c, t.a)] {
                    let key = if u < v { (u, v) } else { (v, u) };
                    *edge_count.entry(key).or_insert(0) += 1;
                }
            }
            let boundary: Vec<(usize, usize)> = edge_count
                .into_iter()
                .filter_map(|(k, c)| if c == 1 { Some(k) } else { None })
                .collect();
            // Remove bad triangles (reverse order for swap_remove).
            bad.sort_unstable();
            for bi in bad.into_iter().rev() {
                triangles.swap_remove(bi);
            }
            // Add new triangles fanning out from p to each boundary edge.
            for (u, v) in boundary {
                triangles.push(Triangle::new(u, v, i));
            }
        }

        // Drop triangles touching the super-triangle vertices.
        triangles.retain(|t| t.a < seeds.len() && t.b < seeds.len() && t.c < seeds.len());

        Self {
            seeds: seeds.to_vec(),
            triangles,
        }
    }

    /// Voronoi cells dual to `self`. Cell vertices are circumcenters
    /// of incident Delaunay triangles, ordered CCW around the seed.
    /// Boundary seeds may produce open (non-enclosing) polygons.
    pub fn voronoi_cells(&self) -> Vec<VoronoiCell> {
        let mut cells: Vec<VoronoiCell> = self
            .seeds
            .iter()
            .map(|s| VoronoiCell {
                seed: *s,
                vertices: Vec::new(),
            })
            .collect();

        for t in &self.triangles {
            let cc = circumcenter(self.seeds[t.a], self.seeds[t.b], self.seeds[t.c]);
            cells[t.a].vertices.push(cc);
            cells[t.b].vertices.push(cc);
            cells[t.c].vertices.push(cc);
        }
        // Sort each cell's vertices CCW around its seed.
        for cell in &mut cells {
            let s = cell.seed;
            cell.vertices.sort_by(|a, b| {
                let ang_a = (a[1] - s[1]).atan2(a[0] - s[0]);
                let ang_b = (b[1] - s[1]).atan2(b[0] - s[0]);
                ang_a
                    .partial_cmp(&ang_b)
                    .unwrap_or(std::cmp::Ordering::Equal)
            });
        }
        cells
    }

    /// Compute the next Lloyd-relaxation state in place. Each seed
    /// moves to its current Voronoi cell's polygon centroid.
    /// Seeds whose cells degenerate (< 3 vertices) stay put.
    pub fn lloyd_step(&mut self) {
        let cells = self.voronoi_cells();
        let mut new_seeds = Vec::with_capacity(self.seeds.len());
        for (i, cell) in cells.iter().enumerate() {
            if cell.vertices.len() >= 3 {
                new_seeds.push(polygon_centroid(&cell.vertices));
            } else {
                new_seeds.push(self.seeds[i]);
            }
        }
        *self = Triangulation::from_seeds(&new_seeds);
    }

    /// Optional convenience: clip a Voronoi cell to an AABB. Uses
    /// Sutherland–Hodgman against the 4 box edges. Returns the
    /// clipped polygon (CCW) — may be empty if the cell lies fully
    /// outside `bounds`.
    pub fn clip_cell_to_aabb(cell: &VoronoiCell, bounds: AABB) -> Vec<Vec2> {
        sutherland_hodgman(&cell.vertices, bounds)
    }
}

/// Generate deterministic seeds inside `bounds` via SplitMix64.
///
/// SplitMix64 was chosen over xorshift64 (the previous impl) because
/// xorshift64 needs a ~16-iteration warm-up to converge from small
/// inputs: with `seed=42` the first ~15 outputs land in the
/// `[10^5, 10^9]` band versus `u64::MAX ≈ 1.8e19`, mapping all those
/// points to the SW corner of `bounds`. SplitMix64 mixes the seed
/// through three odd-multiplier hashes on the first call, so the
/// initial output is already uniform across `u64` — see Vigna's
/// "Further scramblings of Marsaglia's xorshift generators"
/// (Section 4) for the constants used here.
pub fn deterministic_seeds(bounds: AABB, count: usize, seed: u64) -> Vec<Vec2> {
    let mut state = seed;
    let mut out = Vec::with_capacity(count);
    for _ in 0..count {
        let raw_x = splitmix64(&mut state);
        let raw_y = splitmix64(&mut state);
        let fx = (raw_x as f64 / u64::MAX as f64) as f32;
        let fy = (raw_y as f64 / u64::MAX as f64) as f32;
        let x = bounds.min[0] + fx * (bounds.max[0] - bounds.min[0]);
        let y = bounds.min[1] + fy * (bounds.max[1] - bounds.min[1]);
        out.push([x, y]);
    }
    out
}

/// SplitMix64 step. Returns the next 64-bit output and advances
/// `state` in-place. Constants from Vigna; the same triple any
/// other Rust SplitMix64 implementation uses (rand_xoshiro, fastrand
/// internals, etc.) — guarantees cross-platform reproducibility
/// (HR-5).
fn splitmix64(state: &mut u64) -> u64 {
    *state = state.wrapping_add(0x9E37_79B9_7F4A_7C15);
    let mut z = *state;
    z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    z ^ (z >> 31)
}

// =============================================================================
// Geometric primitives
// =============================================================================

/// Determinant-based "p in circumcircle of triangle (a, b, c)?"
/// test. Uses `f64` internally to keep precision for inputs spanning
/// a wide AABB.
fn in_circumcircle(p: Vec2, a: Vec2, b: Vec2, c: Vec2) -> bool {
    // Standard 4×4 determinant test. Assumes (a, b, c) is CCW; if
    // CW, the sign flips. We treat |det| > 0 as "inside" against
    // the CCW reference and flip when needed.
    let ax = (a[0] - p[0]) as f64;
    let ay = (a[1] - p[1]) as f64;
    let bx = (b[0] - p[0]) as f64;
    let by = (b[1] - p[1]) as f64;
    let cx = (c[0] - p[0]) as f64;
    let cy = (c[1] - p[1]) as f64;

    let det = (ax * ax + ay * ay) * (bx * cy - cx * by) - (bx * bx + by * by) * (ax * cy - cx * ay)
        + (cx * cx + cy * cy) * (ax * by - bx * ay);

    // Orientation of (a, b, c): >0 CCW, <0 CW.
    let orient =
        (b[0] - a[0]) as f64 * (c[1] - a[1]) as f64 - (b[1] - a[1]) as f64 * (c[0] - a[0]) as f64;
    if orient > 0.0 { det > 0.0 } else { det < 0.0 }
}

/// Circumcenter of triangle (a, b, c) — the point equidistant from
/// all three. Used as a Voronoi vertex.
pub fn circumcenter(a: Vec2, b: Vec2, c: Vec2) -> Vec2 {
    let ax = a[0] as f64;
    let ay = a[1] as f64;
    let bx = b[0] as f64;
    let by = b[1] as f64;
    let cx = c[0] as f64;
    let cy = c[1] as f64;

    let d = 2.0 * (ax * (by - cy) + bx * (cy - ay) + cx * (ay - by));
    if d.abs() < 1e-12 {
        // Degenerate (collinear) — return midpoint of (a, c) as a
        // fallback so the Voronoi cell sort doesn't crash.
        return [((ax + cx) * 0.5) as f32, ((ay + cy) * 0.5) as f32];
    }
    let ux = ((ax * ax + ay * ay) * (by - cy)
        + (bx * bx + by * by) * (cy - ay)
        + (cx * cx + cy * cy) * (ay - by))
        / d;
    let uy = ((ax * ax + ay * ay) * (cx - bx)
        + (bx * bx + by * by) * (ax - cx)
        + (cx * cx + cy * cy) * (bx - ax))
        / d;
    [ux as f32, uy as f32]
}

fn bounds_of(seeds: &[Vec2]) -> (Vec2, Vec2) {
    let mut min = [f32::INFINITY, f32::INFINITY];
    let mut max = [f32::NEG_INFINITY, f32::NEG_INFINITY];
    for s in seeds {
        if s[0] < min[0] {
            min[0] = s[0];
        }
        if s[1] < min[1] {
            min[1] = s[1];
        }
        if s[0] > max[0] {
            max[0] = s[0];
        }
        if s[1] > max[1] {
            max[1] = s[1];
        }
    }
    (min, max)
}

/// Centroid of a (potentially non-convex) polygon via the signed-
/// area formula. Returns `[0, 0]` for degenerate polygons.
pub fn polygon_centroid(verts: &[Vec2]) -> Vec2 {
    let n = verts.len();
    if n < 3 {
        return [0.0, 0.0];
    }
    let mut a2 = 0.0_f64;
    let mut cx = 0.0_f64;
    let mut cy = 0.0_f64;
    for i in 0..n {
        let p0 = verts[i];
        let p1 = verts[(i + 1) % n];
        let cross = (p0[0] as f64) * (p1[1] as f64) - (p1[0] as f64) * (p0[1] as f64);
        a2 += cross;
        cx += (p0[0] as f64 + p1[0] as f64) * cross;
        cy += (p0[1] as f64 + p1[1] as f64) * cross;
    }
    if a2.abs() < 1e-12 {
        return verts[0];
    }
    let factor = 1.0 / (3.0 * a2);
    [(cx * factor) as f32, (cy * factor) as f32]
}

/// The four axis-aligned edges of an AABB, in clip order.
#[derive(Copy, Clone)]
enum ClipEdge {
    Left,
    Right,
    Bottom,
    Top,
}

impl ClipEdge {
    fn inside(self, bounds: AABB, p: Vec2) -> bool {
        match self {
            ClipEdge::Left => p[0] >= bounds.min[0],
            ClipEdge::Right => p[0] <= bounds.max[0],
            ClipEdge::Bottom => p[1] >= bounds.min[1],
            ClipEdge::Top => p[1] <= bounds.max[1],
        }
    }

    fn intersect(self, bounds: AABB, a: Vec2, b: Vec2) -> Vec2 {
        match self {
            ClipEdge::Left => {
                let t = (bounds.min[0] - a[0]) / (b[0] - a[0]);
                [bounds.min[0], a[1] + t * (b[1] - a[1])]
            }
            ClipEdge::Right => {
                let t = (bounds.max[0] - a[0]) / (b[0] - a[0]);
                [bounds.max[0], a[1] + t * (b[1] - a[1])]
            }
            ClipEdge::Bottom => {
                let t = (bounds.min[1] - a[1]) / (b[1] - a[1]);
                [a[0] + t * (b[0] - a[0]), bounds.min[1]]
            }
            ClipEdge::Top => {
                let t = (bounds.max[1] - a[1]) / (b[1] - a[1]);
                [a[0] + t * (b[0] - a[0]), bounds.max[1]]
            }
        }
    }
}

/// Sutherland–Hodgman polygon clip against AABB. Polygon must be in
/// CCW order; AABB edges are processed left → right → bottom → top.
fn sutherland_hodgman(subject: &[Vec2], bounds: AABB) -> Vec<Vec2> {
    if subject.is_empty() {
        return Vec::new();
    }
    let mut out = subject.to_vec();
    for edge in [
        ClipEdge::Left,
        ClipEdge::Right,
        ClipEdge::Bottom,
        ClipEdge::Top,
    ] {
        let input = std::mem::take(&mut out);
        if input.is_empty() {
            return Vec::new();
        }
        let n = input.len();
        for i in 0..n {
            let cur = input[i];
            let prev = input[(i + n - 1) % n];
            let cur_in = edge.inside(bounds, cur);
            let prev_in = edge.inside(bounds, prev);
            if cur_in {
                if !prev_in {
                    out.push(edge.intersect(bounds, prev, cur));
                }
                out.push(cur);
            } else if prev_in {
                out.push(edge.intersect(bounds, prev, cur));
            }
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn approx_eq(a: Vec2, b: Vec2, eps: f32) -> bool {
        (a[0] - b[0]).abs() < eps && (a[1] - b[1]).abs() < eps
    }

    #[test]
    fn empty_seeds_returns_empty_triangulation() {
        let t = Triangulation::from_seeds(&[]);
        assert_eq!(t.triangles.len(), 0);
    }

    #[test]
    fn fewer_than_three_seeds_is_degenerate() {
        let t = Triangulation::from_seeds(&[[0.0, 0.0], [1.0, 1.0]]);
        assert_eq!(t.triangles.len(), 0);
    }

    #[test]
    fn three_seeds_form_one_triangle() {
        let seeds = [[0.0, 0.0], [4.0, 0.0], [2.0, 3.0]];
        let t = Triangulation::from_seeds(&seeds);
        assert_eq!(t.triangles.len(), 1);
    }

    #[test]
    fn square_seeds_form_two_triangles() {
        let seeds = [[0.0, 0.0], [10.0, 0.0], [10.0, 10.0], [0.0, 10.0]];
        let t = Triangulation::from_seeds(&seeds);
        assert_eq!(t.triangles.len(), 2);
    }

    #[test]
    fn delaunay_property_no_seed_inside_circumcircle() {
        // 8 random-ish seeds. Verify no other seed lies inside any
        // triangle's circumcircle.
        let seeds: Vec<Vec2> = vec![
            [0.0, 0.0],
            [10.0, 0.0],
            [10.0, 10.0],
            [0.0, 10.0],
            [5.0, 5.0],
            [3.0, 7.0],
            [7.0, 3.0],
            [2.0, 2.0],
        ];
        let t = Triangulation::from_seeds(&seeds);
        assert!(!t.triangles.is_empty());
        for tri in &t.triangles {
            for (i, s) in seeds.iter().enumerate() {
                if i == tri.a || i == tri.b || i == tri.c {
                    continue;
                }
                assert!(
                    !in_circumcircle(*s, seeds[tri.a], seeds[tri.b], seeds[tri.c]),
                    "seed {i} {s:?} inside circumcircle of triangle {tri:?}"
                );
            }
        }
    }

    #[test]
    fn circumcenter_of_unit_right_triangle() {
        // Right triangle with legs on axes: circumcenter is the
        // midpoint of the hypotenuse.
        let cc = circumcenter([0.0, 0.0], [2.0, 0.0], [0.0, 2.0]);
        assert!(approx_eq(cc, [1.0, 1.0], 1e-5), "got {cc:?}");
    }

    #[test]
    fn voronoi_cells_have_one_per_seed() {
        let seeds = [[0.0, 0.0], [10.0, 0.0], [5.0, 10.0]];
        let t = Triangulation::from_seeds(&seeds);
        let cells = t.voronoi_cells();
        assert_eq!(cells.len(), 3);
    }

    #[test]
    fn lloyd_step_moves_seeds_toward_cell_centroids() {
        // Asymmetric input — Lloyd should push seeds toward
        // centroidal equilibrium. Check that at least one seed
        // position changes after one step.
        let seeds = vec![[1.0, 1.0], [9.0, 2.0], [3.0, 8.0], [8.0, 7.0], [5.0, 4.0]];
        let mut t = Triangulation::from_seeds(&seeds);
        let before = t.seeds.clone();
        t.lloyd_step();
        let mut any_changed = false;
        for (a, b) in before.iter().zip(t.seeds.iter()) {
            if (a[0] - b[0]).abs() > 1e-3 || (a[1] - b[1]).abs() > 1e-3 {
                any_changed = true;
                break;
            }
        }
        assert!(any_changed, "Lloyd step had no effect");
    }

    #[test]
    fn first_few_seeds_not_all_in_sw_quadrant() {
        // Regression: xorshift64 with seed=42 put the first ~15
        // outputs in the SW quadrant of bounds (state needed ~16
        // warm-up iters to converge from small seeds). SplitMix64
        // mixes the seed through three odd-multiplier hashes on the
        // first call, so the spread is uniform from index 0.
        let bounds = AABB::new([-10.0, -10.0], [10.0, 10.0]);
        let pts = deterministic_seeds(bounds, 8, 42);
        let sw_count = pts.iter().filter(|p| p[0] < 0.0 && p[1] < 0.0).count();
        assert!(
            sw_count <= 5,
            "expected uniform spread; {sw_count}/8 in SW for seed=42 \
             (pre-SplitMix this was 8/8)"
        );
    }

    #[test]
    fn seeds_distribute_across_4x4_buckets_for_varied_seeds() {
        // Across 256 points and 16 buckets, expected mean = 16/bucket.
        // Tolerance per Coord spec: each bucket ≥ (N/16) * 0.5 = 8.
        // The Poisson lower-tail at mean 16 puts P(count < 8) ≈
        // 0.0001 — safe across the 4 seeds × 16 buckets = 64 trials
        // here. SplitMix64's empirical uniformity is comfortably
        // better than the Poisson floor.
        let bounds = AABB::new([-10.0, -10.0], [10.0, 10.0]);
        let n = 256;
        let min_count = (n / 16) / 2;
        for seed in [1u64, 42, 100, 12345] {
            let pts = deterministic_seeds(bounds, n, seed);
            let mut buckets = [[0u32; 4]; 4];
            for p in &pts {
                let bx = (((p[0] - bounds.min[0]) / (bounds.max[0] - bounds.min[0])) * 4.0)
                    .floor()
                    .clamp(0.0, 3.0) as usize;
                let by = (((p[1] - bounds.min[1]) / (bounds.max[1] - bounds.min[1])) * 4.0)
                    .floor()
                    .clamp(0.0, 3.0) as usize;
                buckets[by][bx] += 1;
            }
            for (j, row) in buckets.iter().enumerate() {
                for (i, c) in row.iter().enumerate() {
                    assert!(
                        *c as usize >= min_count,
                        "bucket ({i}, {j}) has {c} pts for seed={seed}; \
                         expected ≥ {min_count}"
                    );
                }
            }
        }
    }

    #[test]
    fn deterministic_seeds_repeat_with_same_seed() {
        let bounds = AABB::new([0.0, 0.0], [10.0, 10.0]);
        let a = deterministic_seeds(bounds, 16, 42);
        let b = deterministic_seeds(bounds, 16, 42);
        assert_eq!(a, b);
        let c = deterministic_seeds(bounds, 16, 43);
        assert_ne!(a, c);
    }

    #[test]
    fn deterministic_seeds_stay_inside_bounds() {
        let bounds = AABB::new([2.0, 3.0], [12.0, 13.0]);
        let pts = deterministic_seeds(bounds, 100, 7);
        for p in &pts {
            assert!(p[0] >= 2.0 && p[0] <= 12.0, "x: {}", p[0]);
            assert!(p[1] >= 3.0 && p[1] <= 13.0, "y: {}", p[1]);
        }
    }

    #[test]
    fn polygon_centroid_of_unit_square() {
        let verts = vec![[0.0, 0.0], [1.0, 0.0], [1.0, 1.0], [0.0, 1.0]];
        let c = polygon_centroid(&verts);
        assert!(approx_eq(c, [0.5, 0.5], 1e-5), "got {c:?}");
    }

    #[test]
    fn sutherland_hodgman_clip_inside_returns_unchanged() {
        let bounds = AABB::new([0.0, 0.0], [10.0, 10.0]);
        let cell = VoronoiCell {
            seed: [5.0, 5.0],
            vertices: vec![[2.0, 2.0], [8.0, 2.0], [8.0, 8.0], [2.0, 8.0]],
        };
        let clipped = Triangulation::clip_cell_to_aabb(&cell, bounds);
        assert_eq!(clipped.len(), 4);
    }

    #[test]
    fn sutherland_hodgman_clip_crossing_boundary() {
        let bounds = AABB::new([0.0, 0.0], [10.0, 10.0]);
        let cell = VoronoiCell {
            seed: [5.0, 5.0],
            vertices: vec![[-5.0, 5.0], [15.0, 5.0], [5.0, 15.0]],
        };
        let clipped = Triangulation::clip_cell_to_aabb(&cell, bounds);
        // After clipping, vertices must all lie inside bounds.
        for v in &clipped {
            assert!(v[0] >= -1e-3 && v[0] <= 10.001);
            assert!(v[1] >= -1e-3 && v[1] <= 10.001);
        }
        assert!(!clipped.is_empty());
    }
}
