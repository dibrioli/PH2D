//! Snap target selection.
//!
//! The two trait methods [`GridMath::snap_to_center`] and
//! [`GridMath::snap_to_nearest_vertex`] cover the per-grid math. The
//! editor `grid_snap/state.rs` owns the enum of active grid kinds
//! and dispatches between the two based on the user's `SnapTarget`
//! preference. This module just provides the policy enum + a free
//! helper for callers holding a `&dyn` of a single grid kind.

use crate::{GridMath, Vec2};

/// Whether snapping picks cell centers or cell vertices
/// (intersections).
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum SnapTarget {
    /// Snap to the center of the containing cell.
    Center,
    /// Snap to the nearest vertex of the containing cell.
    Intersection,
}

/// Dispatch a snap against one concrete grid implementation.
pub fn snap_world<G: GridMath>(
    grid: &G,
    world: Vec2,
    target: SnapTarget,
    scratch: &mut Vec<Vec2>,
) -> Vec2 {
    match target {
        SnapTarget::Center => grid.snap_to_center(world),
        SnapTarget::Intersection => grid.snap_to_nearest_vertex(world, scratch),
    }
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
        let snapped = snap_world(&g, [0.1, 0.1], SnapTarget::Center, &mut buf);
        assert_eq!(snapped, [1.0, 1.0]);
    }

    #[test]
    fn snap_intersection_picks_corner() {
        let g = SquareGrid::new(2.0, SquareNeighborhood::Von4);
        let mut buf = Vec::new();
        // World (0.1, 0.1) is in cell (0, 0). Corners at (0,0), (2,0), (2,2), (0,2).
        // Closest is (0, 0).
        let snapped = snap_world(&g, [0.1, 0.1], SnapTarget::Intersection, &mut buf);
        assert_eq!(snapped, [0.0, 0.0]);
        // World (1.9, 1.9) → closest corner is (2.0, 2.0).
        let snapped = snap_world(&g, [1.9, 1.9], SnapTarget::Intersection, &mut buf);
        assert_eq!(snapped, [2.0, 2.0]);
    }

    #[test]
    fn snap_is_zero_alloc_after_first_call() {
        let g = SquareGrid::new(1.0, SquareNeighborhood::Von4);
        let mut buf = Vec::with_capacity(4);
        let cap = buf.capacity();
        for _ in 0..100 {
            snap_world(&g, [0.5, 0.5], SnapTarget::Intersection, &mut buf);
        }
        assert_eq!(buf.capacity(), cap, "scratch grew");
    }
}
