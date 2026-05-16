//! Snap target selection.
//!
//! The two trait methods [`GridMath::snap_to_center`] and
//! [`GridMath::snap_to_nearest_vertex`] cover the per-grid math. The
//! editor `grid_snap/state.rs` owns the enum of active grid kinds
//! and dispatches between the two based on the user's [`SnapTarget`]
//! preference. This module provides the policy enum + the dispatch
//! helper.
//!
//! ## Sprite-aware corner snap
//!
//! [`SnapTarget::Corner`] (and the composite modes that include it)
//! aligns one of the SPRITE's four corners to the nearest grid
//! intersection (vertex). This needs the sprite's half-extent in
//! world meters — the caller passes it via `sprite_half_size`. When
//! the sprite size isn't known (e.g. drag-drop before the asset
//! lands) callers pass `[0.0, 0.0]` and the Corner mode degenerates
//! to point-Intersection snap (the four "corners" collapse to the
//! center).
//!
//! Composite modes (`CenterAndIntersection`,
//! `CenterIntersectionAndCorners`) pick the candidate closest to
//! `world` from the enabled atomic modes — same UX convention as
//! Blender's Snap "Combination" mode.

use crate::{GridMath, Vec2};

/// Which point(s) snap aligns to when the user moves a sprite.
///
/// Atomic modes (Center / Intersection / Corner) each compute one
/// candidate target. Composite modes evaluate multiple atomics and
/// pick the candidate closest to the input — the "magnetic" behavior
/// Blender / Aseprite / Tiled call Combination snap.
///
/// Cycle order in the panel: Center → Intersection → Corner →
/// CenterAndIntersection → CenterIntersectionAndCorners → wrap.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum SnapTarget {
    /// Snap to the center of the containing cell.
    Center,
    /// Snap to the nearest vertex (intersection) of the containing
    /// cell.
    Intersection,
    /// Snap one of the sprite's four corners to the nearest grid
    /// vertex. Requires non-zero `sprite_half_size`; with
    /// `[0.0, 0.0]` this degenerates to point-Intersection snap.
    Corner,
    /// Composite: pick whichever of `Center` / `Intersection` is
    /// closer to `world`.
    CenterAndIntersection,
    /// Composite: pick whichever of `Center` / `Intersection` /
    /// `Corner` is closer to `world`. Corner skipped when
    /// `sprite_half_size == [0.0, 0.0]`.
    CenterIntersectionAndCorners,
}

impl SnapTarget {
    /// Cycle to the next mode (UI affordance for the panel's Target
    /// row cycling button).
    pub fn cycle(self) -> Self {
        match self {
            Self::Center => Self::Intersection,
            Self::Intersection => Self::Corner,
            Self::Corner => Self::CenterAndIntersection,
            Self::CenterAndIntersection => Self::CenterIntersectionAndCorners,
            Self::CenterIntersectionAndCorners => Self::Center,
        }
    }

    /// Human-readable label for the panel chip.
    pub fn label(self) -> &'static str {
        match self {
            Self::Center => "Center",
            Self::Intersection => "Intersection",
            Self::Corner => "Corner",
            Self::CenterAndIntersection => "Center + Intersection",
            Self::CenterIntersectionAndCorners => "Center + Intersection + Corners",
        }
    }
}

/// Dispatch a snap against one concrete grid implementation.
///
/// `sprite_half_size` is the sprite's half-extent in world meters
/// (i.e. `Sprite::size * 0.5`). Used by Corner mode to align the
/// nearest sprite corner to a grid vertex. Pass `[0.0, 0.0]` when
/// the caller is snapping a bare world point (drag-drop, paste
/// before the sprite size is known).
pub fn snap_world<G: GridMath>(
    grid: &G,
    world: Vec2,
    sprite_half_size: Vec2,
    target: SnapTarget,
    scratch: &mut Vec<Vec2>,
) -> Vec2 {
    match target {
        SnapTarget::Center => grid.snap_to_center(world),
        SnapTarget::Intersection => grid.snap_to_nearest_vertex(world, scratch),
        SnapTarget::Corner => snap_sprite_corner(grid, world, sprite_half_size, scratch),
        SnapTarget::CenterAndIntersection => {
            let c = grid.snap_to_center(world);
            let i = grid.snap_to_nearest_vertex(world, scratch);
            nearest_to(world, &[c, i])
        }
        SnapTarget::CenterIntersectionAndCorners => {
            let c = grid.snap_to_center(world);
            let i = grid.snap_to_nearest_vertex(world, scratch);
            let k = snap_sprite_corner(grid, world, sprite_half_size, scratch);
            nearest_to(world, &[c, i, k])
        }
    }
}

/// Magnetism distance used by [`snap_distance_for_target`] to measure
/// how far the snap candidate sits from `world`. For Center /
/// Intersection / composite modes it's the Euclidean distance between
/// `world` and the snapped point. For Corner mode it's the SHIFT
/// magnitude — the distance the sprite would actually move — because
/// the sprite center may already sit far from any grid vertex even
/// when a sprite corner is touching one.
///
/// Returning the shift instead of `dist(world, snapped)` matches the
/// user's perception: "is a corner near a snap point?" not "is my
/// cursor near a grid vertex?". Without this distinction the
/// magnetism radius would never engage for Corner mode on sprites
/// larger than the cell.
pub fn snap_distance_for_target<G: GridMath>(
    grid: &G,
    world: Vec2,
    sprite_half_size: Vec2,
    target: SnapTarget,
    scratch: &mut Vec<Vec2>,
) -> (Vec2, f32) {
    match target {
        SnapTarget::Center => {
            let s = grid.snap_to_center(world);
            (s, dist(world, s))
        }
        SnapTarget::Intersection => {
            let s = grid.snap_to_nearest_vertex(world, scratch);
            (s, dist(world, s))
        }
        SnapTarget::Corner => {
            let s = snap_sprite_corner(grid, world, sprite_half_size, scratch);
            // Shift magnitude == |s - world| because snap_sprite_corner
            // just adds the best shift to `world`.
            (s, dist(world, s))
        }
        SnapTarget::CenterAndIntersection => {
            let c = grid.snap_to_center(world);
            let i = grid.snap_to_nearest_vertex(world, scratch);
            let s = nearest_to(world, &[c, i]);
            (s, dist(world, s))
        }
        SnapTarget::CenterIntersectionAndCorners => {
            let c = grid.snap_to_center(world);
            let i = grid.snap_to_nearest_vertex(world, scratch);
            let k = snap_sprite_corner(grid, world, sprite_half_size, scratch);
            let s = nearest_to(world, &[c, i, k]);
            (s, dist(world, s))
        }
    }
}

#[inline]
fn dist(a: Vec2, b: Vec2) -> f32 {
    sq_dist(a, b).sqrt()
}

/// Compute the world position the sprite center must move to so that
/// the sprite corner closest to a grid vertex lands exactly on that
/// vertex.
///
/// Strategy: enumerate the four sprite corners (sprite center ±
/// `(hw, hh)` and ± `(hw, -hh)`), snap each to its nearest grid
/// vertex, and select the (corner, vertex) pair with the smallest
/// shift. Apply that shift to the sprite center.
///
/// When `sprite_half_size == [0.0, 0.0]` the four corners collapse
/// to the sprite center, so the function returns the point-
/// Intersection result (back-compat with callers that don't know
/// the sprite size).
fn snap_sprite_corner<G: GridMath>(
    grid: &G,
    world: Vec2,
    sprite_half_size: Vec2,
    scratch: &mut Vec<Vec2>,
) -> Vec2 {
    let hw = sprite_half_size[0];
    let hh = sprite_half_size[1];
    if hw == 0.0 && hh == 0.0 {
        // Degenerate: no sprite extent → corner snap == point snap.
        return grid.snap_to_nearest_vertex(world, scratch);
    }
    let corners: [Vec2; 4] = [
        [world[0] - hw, world[1] - hh],
        [world[0] + hw, world[1] - hh],
        [world[0] - hw, world[1] + hh],
        [world[0] + hw, world[1] + hh],
    ];
    let mut best_shift: Vec2 = [0.0, 0.0];
    let mut best_d2 = f32::INFINITY;
    for c in corners {
        let v = grid.snap_to_nearest_vertex(c, scratch);
        let dx = v[0] - c[0];
        let dy = v[1] - c[1];
        let d2 = dx * dx + dy * dy;
        if d2 < best_d2 {
            best_d2 = d2;
            best_shift = [dx, dy];
        }
    }
    [world[0] + best_shift[0], world[1] + best_shift[1]]
}

/// Pick the candidate in `candidates` closest (squared-distance) to
/// `world`. Fixed evaluation order — first candidate wins on tie, so
/// HR-5 deterministic across builds.
fn nearest_to(world: Vec2, candidates: &[Vec2]) -> Vec2 {
    let mut best = candidates[0];
    let mut best_d2 = sq_dist(world, best);
    for &c in &candidates[1..] {
        let d2 = sq_dist(world, c);
        if d2 < best_d2 {
            best_d2 = d2;
            best = c;
        }
    }
    best
}

#[inline]
fn sq_dist(a: Vec2, b: Vec2) -> f32 {
    let dx = a[0] - b[0];
    let dy = a[1] - b[1];
    dx * dx + dy * dy
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::square::{SquareGrid, SquareNeighborhood};

    #[test]
    fn snap_center_pulls_to_cell_center() {
        let g = SquareGrid::new(2.0, SquareNeighborhood::Von4);
        let mut buf = Vec::new();
        // World (0.1, 0.1) is in cell (0, 0). Center is (1.0, 1.0).
        let snapped = snap_world(&g, [0.1, 0.1], [0.0, 0.0], SnapTarget::Center, &mut buf);
        assert_eq!(snapped, [1.0, 1.0]);
    }

    #[test]
    fn snap_intersection_picks_corner_of_cell() {
        let g = SquareGrid::new(2.0, SquareNeighborhood::Von4);
        let mut buf = Vec::new();
        // World (0.1, 0.1) is in cell (0, 0). Corners at (0,0), (2,0), (2,2), (0,2).
        // Closest is (0, 0).
        let snapped = snap_world(
            &g,
            [0.1, 0.1],
            [0.0, 0.0],
            SnapTarget::Intersection,
            &mut buf,
        );
        assert_eq!(snapped, [0.0, 0.0]);
        // World (1.9, 1.9) → closest corner is (2.0, 2.0).
        let snapped = snap_world(
            &g,
            [1.9, 1.9],
            [0.0, 0.0],
            SnapTarget::Intersection,
            &mut buf,
        );
        assert_eq!(snapped, [2.0, 2.0]);
    }

    #[test]
    fn snap_corner_aligns_nearest_sprite_corner_to_grid_vertex() {
        // Sprite 2×2 in world (half = 1×1) centered at (0.1, 0.1).
        // Sprite corners at (-0.9,-0.9), (1.1,-0.9), (-0.9,1.1), (1.1,1.1).
        // Cell size 2 → grid vertices at multiples of 2.
        // (-0.9,-0.9) → nearest vertex (0, 0) → shift = (0.9, 0.9).
        // (1.1,-0.9) → nearest vertex (2, 0)  → shift = (0.9, 0.9).
        // (-0.9, 1.1) → nearest vertex (0, 2) → shift = (0.9, 0.9).
        // (1.1, 1.1)  → nearest vertex (2, 2) → shift = (0.9, 0.9).
        // All four candidates yield the same shift magnitude here;
        // first-wins picks (-0.9, -0.9) → shift (0.9, 0.9).
        // New center = (0.1 + 0.9, 0.1 + 0.9) = (1.0, 1.0).
        let g = SquareGrid::new(2.0, SquareNeighborhood::Von4);
        let mut buf = Vec::new();
        let snapped = snap_world(&g, [0.1, 0.1], [1.0, 1.0], SnapTarget::Corner, &mut buf);
        assert_eq!(snapped, [1.0, 1.0]);
    }

    #[test]
    fn snap_corner_degenerates_to_intersection_when_half_size_is_zero() {
        let g = SquareGrid::new(2.0, SquareNeighborhood::Von4);
        let mut buf = Vec::new();
        // No sprite extent → corner snap == intersection snap.
        let snapped = snap_world(&g, [0.1, 0.1], [0.0, 0.0], SnapTarget::Corner, &mut buf);
        let intersection = snap_world(
            &g,
            [0.1, 0.1],
            [0.0, 0.0],
            SnapTarget::Intersection,
            &mut buf,
        );
        assert_eq!(snapped, intersection);
    }

    #[test]
    fn snap_corner_picks_nearest_corner_when_one_is_closer() {
        // Asymmetric sprite + asymmetric position so different
        // corners produce different shifts. Sprite half = (0.5, 0.5)
        // at center (0.45, 0.45). Corners:
        //   (-0.05, -0.05), (0.95, -0.05), (-0.05, 0.95), (0.95, 0.95)
        // Grid cell_size 1 → vertices at integers.
        // (-0.05, -0.05) → (0, 0) shift (0.05, 0.05). dist² = 0.005.
        // (0.95, -0.05)  → (1, 0) shift (0.05, 0.05). dist² = 0.005.
        // (-0.05, 0.95)  → (0, 1) shift (0.05, 0.05). dist² = 0.005.
        // (0.95, 0.95)   → (1, 1) shift (0.05, 0.05). dist² = 0.005.
        // All tied (symmetric position) — first wins. New center
        // = (0.5, 0.5).
        let g = SquareGrid::new(1.0, SquareNeighborhood::Von4);
        let mut buf = Vec::new();
        let snapped = snap_world(&g, [0.45, 0.45], [0.5, 0.5], SnapTarget::Corner, &mut buf);
        assert_eq!(snapped, [0.5, 0.5]);
    }

    #[test]
    fn composite_center_plus_intersection_picks_closer_candidate() {
        let g = SquareGrid::new(2.0, SquareNeighborhood::Von4);
        let mut buf = Vec::new();
        // World (0.1, 0.1) is in cell (0, 0). Center = (1, 1) dist²
        // ≈ 1.62. Intersection (0, 0) dist² ≈ 0.02. Intersection
        // wins.
        let snapped = snap_world(
            &g,
            [0.1, 0.1],
            [0.0, 0.0],
            SnapTarget::CenterAndIntersection,
            &mut buf,
        );
        assert_eq!(snapped, [0.0, 0.0]);
        // World (0.9, 0.9) is in cell (0, 0). Center = (1, 1) dist²
        // ≈ 0.02. Intersection (0, 0) dist² ≈ 1.62. Center wins.
        let snapped = snap_world(
            &g,
            [0.9, 0.9],
            [0.0, 0.0],
            SnapTarget::CenterAndIntersection,
            &mut buf,
        );
        assert_eq!(snapped, [1.0, 1.0]);
    }

    #[test]
    fn composite_three_picks_closest_of_all_three() {
        let g = SquareGrid::new(2.0, SquareNeighborhood::Von4);
        let mut buf = Vec::new();
        // Sprite 1×1 at (0.85, 0.85). Cell size 2.
        // Center = (1, 1) dist² ≈ 0.045.
        // Intersection (0,0) dist² ≈ 1.445.
        // Corner: sprite corners at (0.35, 0.35), (1.35, 0.35),
        //   (0.35, 1.35), (1.35, 1.35).
        //   Each → nearest vertex; best shift is from (1.35, 1.35)
        //   to (2, 2) = (0.65, 0.65), but the (0.35, 0.35) → (0, 0)
        //   shift is (-0.35, -0.35) which is SMALLER (d² = 0.245
        //   vs 0.845). New center via Corner = (0.85 - 0.35, 0.85 -
        //   0.35) = (0.5, 0.5). dist² from world (0.85, 0.85) = 0.245.
        // Center (dist² 0.045) is closest → wins.
        let snapped = snap_world(
            &g,
            [0.85, 0.85],
            [0.5, 0.5],
            SnapTarget::CenterIntersectionAndCorners,
            &mut buf,
        );
        assert_eq!(snapped, [1.0, 1.0]);
    }

    #[test]
    fn snap_is_zero_alloc_after_first_call() {
        let g = SquareGrid::new(1.0, SquareNeighborhood::Von4);
        let mut buf = Vec::with_capacity(4);
        let cap = buf.capacity();
        for _ in 0..100 {
            snap_world(
                &g,
                [0.5, 0.5],
                [0.25, 0.25],
                SnapTarget::CenterIntersectionAndCorners,
                &mut buf,
            );
        }
        assert_eq!(buf.capacity(), cap, "scratch grew");
    }

    #[test]
    fn cycle_walks_five_modes_and_wraps() {
        let modes = [
            SnapTarget::Center,
            SnapTarget::Intersection,
            SnapTarget::Corner,
            SnapTarget::CenterAndIntersection,
            SnapTarget::CenterIntersectionAndCorners,
        ];
        let mut t = SnapTarget::Center;
        for expected in modes.iter().skip(1).chain(std::iter::once(&modes[0])) {
            t = t.cycle();
            assert_eq!(t, *expected);
        }
    }

    #[test]
    fn label_is_non_empty_for_every_mode() {
        for t in [
            SnapTarget::Center,
            SnapTarget::Intersection,
            SnapTarget::Corner,
            SnapTarget::CenterAndIntersection,
            SnapTarget::CenterIntersectionAndCorners,
        ] {
            assert!(!t.label().is_empty(), "label empty for {t:?}");
        }
    }
}
