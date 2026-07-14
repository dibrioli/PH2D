//! **Bridson 2007** — *Fast Poisson Disk Sampling in Arbitrary Dimensions* (SIGGRAPH
//! sketch): dart-throwing with a background grid, `O(N)`.
//!
//! ```text
//! 1. cell = r/√2 — small enough that a cell can hold AT MOST ONE sample, which is what
//!    turns the "is anything within r?" question into a fixed 5×5 cell scan.
//! 2. Seed one point; it is the first ACTIVE point.
//! 3. While the active list is not empty: pick an active point at random, throw K darts
//!    into the annulus [r, 2r] around it, and keep the first that lands in bounds and no
//!    closer than r to any point already placed. If all K miss, that point is DONE
//!    (retire it from the active list — its neighbourhood is full).
//! ```
//!
//! Every iteration either places a point or retires an active one, and both are bounded
//! by the number of cells, so the loop always terminates.
//!
//! ## Two deliberate departures from the sketch
//!
//! **The dart's direction is rejection-sampled from the unit disc, not polar.** The
//! sketch says "pick a random angle", which means `sin`/`cos` — banned (HR-5). Drawing a
//! point in the square and normalising it would bias the direction toward the diagonals
//! (the corners are farther out), so the square sample is *rejected* until it lands
//! inside the unit disc: uniform direction, arithmetic and `sqrt` only.
//!
//! **The dart's radius is uniform by AREA, not uniform in `[r, 2r]`.** The sketch draws
//! the radius uniformly, which crowds the darts onto the inner ring (a thin annulus at
//! radius ρ has area ∝ ρ, so uniform-in-ρ over-samples small ρ). The area-uniform draw
//! `ρ = √(r² + u·3r²)` is the accepted correction and spends fewer darts on the region
//! that is most likely to be already occupied.

use crate::hash::Draws;

/// Bridson's cell size is `r/√2` — see the module doc.
const SQRT2: f32 = std::f32::consts::SQRT_2;

/// Darts thrown at an active point before it is retired. Bridson's `k = 30`.
const K: u32 = 30;

/// The ceiling on the background grid — and therefore on everything else, because a
/// Bridson cell holds at most one point: **bounding the cells bounds the memory AND the
/// count.** This is what a node with no `count` param needs instead of one; a radius
/// typed as `0` must not ask for an infinite grid.
const MAX_CELLS: usize = 1 << 18;

/// An absolute floor under the radius, for a rectangle so small that the cell budget
/// alone would still allow a near-zero one (a zero radius divides by zero and asks for a
/// grid of `usize::MAX` cells — the saturating `f32 as usize` cast makes that an
/// allocation, not a panic).
const MIN_RADIUS: f32 = 1e-4;

/// Rejection tries for a uniform direction. Acceptance is `π/4 ≈ 79%`, so all eight
/// missing has probability `~5e-6`; that dart then flies along `+X`, which is a valid
/// (merely not uniformly-chosen) direction — the invariant the node promises (no two
/// points closer than `r`) is checked afterwards regardless.
const DIR_TRIES: u32 = 8;

/// The smallest radius this rectangle can afford, given the cell budget.
///
/// The grid is `(w/cell) × (h/cell)` cells with `cell = r/√2`, i.e. `2·w·h/r²` of them,
/// so `r ≥ √(2·w·h / MAX_CELLS)`.
fn clamp_radius(w: f32, h: f32, radius: f32) -> f32 {
    let floor = (2.0 * w * h / MAX_CELLS as f32).sqrt();
    radius.max(floor).max(MIN_RADIUS)
}

/// A dart from `center`: uniform direction × area-uniform radius in the annulus `[r, 2r]`.
fn dart(d: &mut Draws, center: [f32; 2], r: f32) -> [f32; 2] {
    let mut dir = [1.0, 0.0];
    for _ in 0..DIR_TRIES {
        let x = d.next() * 2.0 - 1.0;
        let y = d.next() * 2.0 - 1.0;
        let len_sq = x * x + y * y;
        if len_sq > 1e-8 && len_sq <= 1.0 {
            let len = len_sq.sqrt();
            dir = [x / len, y / len];
            break;
        }
    }
    let rho = (r * r + d.next() * 3.0 * r * r).sqrt();
    [center[0] + dir[0] * rho, center[1] + dir[1] * rho]
}

/// The grid cell a point falls in.
fn cell_of(p: [f32; 2], cell: f32, gw: usize, gh: usize) -> (usize, usize) {
    let cx = ((p[0] / cell) as usize).min(gw - 1);
    let cy = ((p[1] / cell) as usize).min(gh - 1);
    (cx, cy)
}

/// **The invariant, checked**: is `p` at least `r` from every point already placed?
///
/// A Bridson cell is `r/√2` across, so anything within `r` of `p` lies in the 5×5 block
/// of cells around it — a fixed scan, which is where the `O(N)` comes from.
fn far_enough(
    grid: &[u32],
    pts: &[[f32; 2]],
    cell: f32,
    gw: usize,
    gh: usize,
    p: [f32; 2],
    r: f32,
) -> bool {
    let (cx, cy) = cell_of(p, cell, gw, gh);
    let lo_x = cx.saturating_sub(2);
    let lo_y = cy.saturating_sub(2);
    let hi_x = (cx + 2).min(gw - 1);
    let hi_y = (cy + 2).min(gh - 1);
    let r_sq = r * r;
    for y in lo_y..=hi_y {
        for x in lo_x..=hi_x {
            let at = grid[y * gw + x];
            if at == u32::MAX {
                continue;
            }
            let q = pts[at as usize];
            let (dx, dy) = (p[0] - q[0], p[1] - q[1]);
            if dx * dx + dy * dy < r_sq {
                return false;
            }
        }
    }
    true
}

/// Fill the `w × h` rectangle (centred on the origin) with points no two of which are
/// closer than `radius`. The count is **implicit** — it is whatever the spacing allows,
/// which is the whole difference between this node and `motion.scatter`.
pub(crate) fn sample(w: f32, h: f32, radius: f32, seed: u32) -> Vec<[f32; 2]> {
    if !w.is_finite() || !h.is_finite() || !radius.is_finite() || w <= 0.0 || h <= 0.0 {
        return Vec::new();
    }
    let r = clamp_radius(w, h, radius);
    let cell = r / SQRT2;
    let gw = (w / cell).ceil().max(1.0) as usize;
    let gh = (h / cell).ceil().max(1.0) as usize;
    // The clamp above already bounds this; the guard is what makes that a fact rather
    // than an argument (a `ceil` of a huge float saturates the cast instead of wrapping).
    if gw.saturating_mul(gh) > MAX_CELLS.saturating_mul(2) {
        return Vec::new();
    }
    let mut grid = vec![u32::MAX; gw * gh];
    let mut pts: Vec<[f32; 2]> = Vec::new();
    let mut active: Vec<u32> = Vec::new();
    let mut d = Draws { seed, n: 0 };

    let place = |p: [f32; 2], grid: &mut [u32], pts: &mut Vec<[f32; 2]>, active: &mut Vec<u32>| {
        let (cx, cy) = cell_of(p, cell, gw, gh);
        grid[cy * gw + cx] = pts.len() as u32;
        active.push(pts.len() as u32);
        pts.push(p);
    };

    place(
        [d.next() * w, d.next() * h],
        &mut grid,
        &mut pts,
        &mut active,
    );

    while !active.is_empty() && pts.len() < MAX_CELLS {
        // Bridson picks the active point at RANDOM (not FIFO): the front grows in every
        // direction at once instead of sweeping across the rectangle in a wave.
        let a = ((d.next() * active.len() as f32) as usize).min(active.len() - 1);
        let center = pts[active[a] as usize];
        let mut landed = false;
        for _ in 0..K {
            let c = dart(&mut d, center, r);
            if c[0] < 0.0 || c[0] >= w || c[1] < 0.0 || c[1] >= h {
                continue;
            }
            if far_enough(&grid, &pts, cell, gw, gh, c, r) {
                place(c, &mut grid, &mut pts, &mut active);
                landed = true;
                break;
            }
        }
        if !landed {
            active.swap_remove(a); // its neighbourhood is full — retire it
        }
    }

    // Author in [0,w)×[0,h), hand back centred on the origin (the convention every
    // distribution here shares: the rectangle is around where you dropped the node).
    let (hw, hh) = (w * 0.5, h * 0.5);
    pts.iter().map(|p| [p[0] - hw, p[1] - hh]).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The closest pair in the set. `f32::MAX` for fewer than two points.
    fn min_gap(pts: &[[f32; 2]]) -> f32 {
        let mut best = f32::MAX;
        for (i, p) in pts.iter().enumerate() {
            for q in &pts[i + 1..] {
                let (dx, dy) = (p[0] - q[0], p[1] - q[1]);
                best = best.min((dx * dx + dy * dy).sqrt());
            }
        }
        best
    }

    /// **The one promise the node makes.** Everything else is a consequence.
    #[test]
    fn no_two_points_are_closer_than_the_radius() {
        for seed in 0..6u32 {
            let pts = sample(4.0, 3.0, 0.3, seed);
            assert!(pts.len() > 10, "seed {seed} produced almost nothing");
            let gap = min_gap(&pts);
            assert!(
                gap >= 0.3 - 1e-4,
                "seed {seed}: two points {gap} apart, closer than the 0.3 radius"
            );
        }
    }

    /// …and it does not buy that promise by placing three points and giving up: the
    /// disc packs the rectangle. The theoretical maximum for radius `r` is the hexagonal
    /// packing `area / (r²·√3/2)`; Bridson reaches ~65-75% of it, and a broken
    /// neighbourhood check (one that rejects everything) would sit at 1 point.
    #[test]
    fn it_actually_fills_the_rectangle() {
        let (w, h, r) = (4.0f32, 3.0f32, 0.3f32);
        let pts = sample(w, h, r, 1);
        let hex_max = (w * h) / (r * r * 0.866);
        let ratio = pts.len() as f32 / hex_max;
        assert!(
            (0.5..=1.0).contains(&ratio),
            "packed {} points, {ratio:.2} of the hexagonal maximum {hex_max:.0}",
            pts.len()
        );
        // Every point is inside the centred rectangle it was asked for.
        for p in &pts {
            assert!(
                p[0] >= -w * 0.5 && p[0] <= w * 0.5,
                "x out of bounds: {p:?}"
            );
            assert!(
                p[1] >= -h * 0.5 && p[1] <= h * 0.5,
                "y out of bounds: {p:?}"
            );
        }
    }

    /// **The radius is the knob, the count is the answer** — the inverse-square law of
    /// the family, and the thing that makes this node different from `motion.scatter`.
    #[test]
    fn halving_the_radius_roughly_quadruples_the_count() {
        let coarse = sample(4.0, 4.0, 0.4, 1).len();
        let fine = sample(4.0, 4.0, 0.2, 1).len();
        let factor = fine as f32 / coarse as f32;
        assert!(
            (3.0..5.0).contains(&factor),
            "half the radius gave {factor:.2}x the points ({coarse} -> {fine}), not ~4x"
        );
    }

    /// Pure function of the seed: a scrub, a re-cook or another machine redraws the
    /// exact same layout — and another seed redraws a different one.
    #[test]
    fn the_layout_is_a_pure_function_of_the_seed() {
        assert_eq!(sample(4.0, 3.0, 0.3, 5), sample(4.0, 3.0, 0.3, 5));
        assert_ne!(sample(4.0, 3.0, 0.3, 5), sample(4.0, 3.0, 0.3, 6));
    }

    /// **A radius of zero must not hang the app, and must not allocate the world.** A
    /// count-less distribution has no `param_as_count` to hide behind: the *radius* is
    /// the allocation vector, so it is the radius that gets clamped.
    #[test]
    fn a_pathological_radius_is_bounded_not_fatal() {
        for r in [0.0, -1.0, f32::NAN, f32::INFINITY, 1e-30] {
            let pts = sample(4.0, 4.0, r, 1);
            assert!(
                pts.len() <= MAX_CELLS,
                "radius {r} produced {} points",
                pts.len()
            );
        }
        // A degenerate rectangle is empty, not a panic.
        assert!(sample(0.0, 4.0, 0.3, 1).is_empty());
        assert!(sample(f32::NAN, 4.0, 0.3, 1).is_empty());
        // A radius larger than the rectangle fits exactly one point (the seed dart).
        assert_eq!(sample(1.0, 1.0, 10.0, 1).len(), 1);
    }
}
