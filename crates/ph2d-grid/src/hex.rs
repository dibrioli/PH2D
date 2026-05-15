//! Hexagonal grids — pointy-top + flat-top, with offset/axial/cube
//! coordinate systems exposed simultaneously.
//!
//! # Canonical reference
//!
//! Math follows Red Blob Games' "Hexagonal Grids" article exactly
//! (same axial direction order, same cube-coord invariant
//! `x + y + z = 0`, same `cube_round` algorithm). Deviating would
//! invite subtle bugs that already have textbook fixes there.
//!
//! # Internal representation
//!
//! Cells are stored as **axial** `(q, r)`. Cube derived as
//! `(q, -q - r, r)`. Offset variants computed on demand for display
//! / array-storage layouts. Storing axial keeps neighbor / distance
//! / line math orientation-agnostic — `HexOrientation` only affects
//! the world-projection step.
//!
//! # Conventions
//!
//! - World Y-up (matches editor / `ph2d-vector`).
//! - `cell_size` is the **radius from center to corner**. For pointy-
//!   top, hex width = √3·size and height = 2·size; for flat-top,
//!   width = 2·size and height = √3·size.

use crate::{GridMath, Vec2};

/// Orientation of the hex grid. Determines world-projection only;
/// axial neighbor/distance math is identical for both.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum HexOrientation {
    /// Pointy-top: hexagons have a vertex at the top. Common in
    /// strategy games (Civilization).
    Pointy,
    /// Flat-top: hexagons have an edge at the top. Common in
    /// 4X / wargames (Hexcells, Battle for Wesnoth).
    Flat,
}

/// Offset-coord variant for array-friendly storage. Each combination
/// of orientation × parity defines how the rows or columns shift
/// relative to the previous one.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum HexOffset {
    /// Pointy-top, odd rows shifted right. Stores in `[row][col]`.
    OddR,
    /// Pointy-top, even rows shifted right.
    EvenR,
    /// Flat-top, odd columns shifted down. Stores in `[col][row]`.
    OddQ,
    /// Flat-top, even columns shifted down.
    EvenQ,
}

/// Hex grid. Cells stored in axial; pixel projection picks the
/// orientation. `offset_default` is the recommended variant for this
/// orientation, used only for the `offset_of()` helper.
#[derive(Copy, Clone, Debug)]
pub struct HexGrid {
    /// Radius from hex center to corner, in world meters. Must be > 0.
    pub cell_size: f32,
    /// Whether hexes are pointy-top or flat-top.
    pub orientation: HexOrientation,
    /// Default offset variant for [`HexGrid::offset_of`] / [`HexGrid::from_offset`].
    /// Convention: `OddR` for pointy, `OddQ` for flat (Red Blob's
    /// canonical choice).
    pub offset_default: HexOffset,
}

impl HexGrid {
    /// Pointy-top grid with `OddR` offset.
    pub fn pointy(cell_size: f32) -> Self {
        debug_assert!(cell_size > 0.0);
        Self {
            cell_size,
            orientation: HexOrientation::Pointy,
            offset_default: HexOffset::OddR,
        }
    }

    /// Flat-top grid with `OddQ` offset.
    pub fn flat(cell_size: f32) -> Self {
        debug_assert!(cell_size > 0.0);
        Self {
            cell_size,
            orientation: HexOrientation::Flat,
            offset_default: HexOffset::OddQ,
        }
    }
}

/// Axial hex cell coordinates. Two-integer representation; the third
/// cube coord is implicit (`s = -q - r`).
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct HexCell {
    pub q: i32,
    pub r: i32,
}

impl HexCell {
    pub const fn new(q: i32, r: i32) -> Self {
        Self { q, r }
    }
}

/// Cube hex coordinates with the invariant `x + y + z == 0`. Useful
/// for distance / line algorithms; equivalent in information to
/// axial but symmetric across the three axes.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct CubeCoord {
    pub x: i32,
    pub y: i32,
    pub z: i32,
}

impl CubeCoord {
    /// Build from raw components; debug-asserts the invariant.
    pub fn new(x: i32, y: i32, z: i32) -> Self {
        debug_assert_eq!(x + y + z, 0, "cube invariant x+y+z=0 violated");
        Self { x, y, z }
    }
}

/// Offset-coord cell `(col, row)` for array layout. Interpretation
/// depends on the [`HexOffset`] variant passed alongside.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct OffsetCell {
    pub col: i32,
    pub row: i32,
}

impl OffsetCell {
    pub const fn new(col: i32, row: i32) -> Self {
        Self { col, row }
    }
}

// =============================================================================
// Coordinate system conversions
// =============================================================================

/// Axial → cube. The cube invariant `x + y + z = 0` reconstructs the
/// missing axis from the two stored ones.
pub fn axial_to_cube(cell: HexCell) -> CubeCoord {
    let x = cell.q;
    let z = cell.r;
    let y = -x - z;
    CubeCoord { x, y, z }
}

/// Cube → axial. Drops `y` (recoverable from `x + z`).
pub fn cube_to_axial(c: CubeCoord) -> HexCell {
    HexCell { q: c.x, r: c.z }
}

/// Axial → offset under the given variant.
pub fn axial_to_offset(cell: HexCell, variant: HexOffset) -> OffsetCell {
    // `rem_euclid(2)` instead of `& 1` so the parity is always 0/1
    // even for negative `q`/`r` (i32::rem_euclid is the floor-mod).
    let q = cell.q;
    let r = cell.r;
    match variant {
        HexOffset::OddR => OffsetCell {
            col: q + (r - r.rem_euclid(2)) / 2,
            row: r,
        },
        HexOffset::EvenR => OffsetCell {
            col: q + (r + r.rem_euclid(2)) / 2,
            row: r,
        },
        HexOffset::OddQ => OffsetCell {
            col: q,
            row: r + (q - q.rem_euclid(2)) / 2,
        },
        HexOffset::EvenQ => OffsetCell {
            col: q,
            row: r + (q + q.rem_euclid(2)) / 2,
        },
    }
}

/// Offset → axial under the given variant.
pub fn offset_to_axial(cell: OffsetCell, variant: HexOffset) -> HexCell {
    let col = cell.col;
    let row = cell.row;
    match variant {
        HexOffset::OddR => HexCell {
            q: col - (row - row.rem_euclid(2)) / 2,
            r: row,
        },
        HexOffset::EvenR => HexCell {
            q: col - (row + row.rem_euclid(2)) / 2,
            r: row,
        },
        HexOffset::OddQ => HexCell {
            q: col,
            r: row - (col - col.rem_euclid(2)) / 2,
        },
        HexOffset::EvenQ => HexCell {
            q: col,
            r: row - (col + col.rem_euclid(2)) / 2,
        },
    }
}

// =============================================================================
// Axial neighbor / distance / line
// =============================================================================

/// Six axial neighbor direction vectors, deterministic order:
/// E, NE, NW, W, SW, SE (pointy-top mnemonic; same axial offsets
/// apply to flat-top — only world-projection rotates).
pub const HEX_DIRECTIONS: [HexCell; 6] = [
    HexCell { q: 1, r: 0 },
    HexCell { q: 1, r: -1 },
    HexCell { q: 0, r: -1 },
    HexCell { q: -1, r: 0 },
    HexCell { q: -1, r: 1 },
    HexCell { q: 0, r: 1 },
];

/// Add two axial cells componentwise.
pub fn hex_add(a: HexCell, b: HexCell) -> HexCell {
    HexCell {
        q: a.q + b.q,
        r: a.r + b.r,
    }
}

/// Subtract `b` from `a` componentwise.
pub fn hex_sub(a: HexCell, b: HexCell) -> HexCell {
    HexCell {
        q: a.q - b.q,
        r: a.r - b.r,
    }
}

/// Multiply axial cell by an integer scalar.
pub fn hex_scale(a: HexCell, k: i32) -> HexCell {
    HexCell {
        q: a.q * k,
        r: a.r * k,
    }
}

/// Hex distance via cube coords: `(|dx| + |dy| + |dz|) / 2 ==
/// max(|dx|, |dy|, |dz|)`. The max formulation avoids the divide.
pub fn hex_distance(a: HexCell, b: HexCell) -> u32 {
    let d = hex_sub(a, b);
    let dx = d.q;
    let dz = d.r;
    let dy = -dx - dz;
    let mx = dx.unsigned_abs();
    let my = dy.unsigned_abs();
    let mz = dz.unsigned_abs();
    mx.max(my).max(mz)
}

/// Cube → cube with each axis rounded to the nearest integer while
/// preserving the `x + y + z = 0` invariant. The standard algorithm:
/// round all three independently, then discard the axis with the
/// largest rounding error and reconstruct it from the other two.
pub fn cube_round(fx: f32, fy: f32, fz: f32) -> CubeCoord {
    let mut rx = fx.round();
    let mut ry = fy.round();
    let mut rz = fz.round();
    let dx = (rx - fx).abs();
    let dy = (ry - fy).abs();
    let dz = (rz - fz).abs();
    if dx > dy && dx > dz {
        rx = -ry - rz;
    } else if dy > dz {
        ry = -rx - rz;
    } else {
        rz = -rx - ry;
    }
    CubeCoord {
        x: rx as i32,
        y: ry as i32,
        z: rz as i32,
    }
}

/// Hex line from `a` to `b`, inclusive. Uses cube lerp + `cube_round`
/// (Red Blob §"Line drawing"). Output order: a … b.
pub fn hex_line(a: HexCell, b: HexCell, out: &mut Vec<HexCell>) {
    out.clear();
    let n = hex_distance(a, b);
    if n == 0 {
        out.push(a);
        return;
    }
    let ca = axial_to_cube(a);
    let cb = axial_to_cube(b);
    let step = 1.0 / n as f32;
    // Epsilon nudge (Red Blob §"Line drawing", "Edge cases") so the
    // line never lands exactly on a cell boundary — picks one side
    // deterministically. 1e-6 in each cube axis, summing to ~0 to
    // keep the invariant.
    let ax = ca.x as f32 + 1e-6;
    let ay = ca.y as f32 + 1e-6;
    let az = ca.z as f32 - 2e-6;
    let bx = cb.x as f32 + 1e-6;
    let by = cb.y as f32 + 1e-6;
    let bz = cb.z as f32 - 2e-6;
    for i in 0..=n {
        let t = i as f32 * step;
        let fx = ax + (bx - ax) * t;
        let fy = ay + (by - ay) * t;
        let fz = az + (bz - az) * t;
        out.push(cube_to_axial(cube_round(fx, fy, fz)));
    }
}

/// All cells within `radius` of `center`, inclusive of `center` and
/// of cells at exactly `radius`. Output order: spiral (center first,
/// then ring-1, then ring-2, …).
pub fn hex_range(center: HexCell, radius: u32, out: &mut Vec<HexCell>) {
    out.clear();
    out.push(center);
    for k in 1..=radius {
        hex_ring_append(center, k, out);
    }
}

/// Cells at *exactly* distance `radius` from `center` (the boundary
/// ring). Cleared; pushed CCW starting from the +x axial direction.
pub fn hex_ring(center: HexCell, radius: u32, out: &mut Vec<HexCell>) {
    out.clear();
    if radius == 0 {
        out.push(center);
        return;
    }
    hex_ring_append(center, radius, out);
}

/// Internal: append a ring at radius `radius` (no clear). Used by
/// both `hex_ring` and `hex_range` (spiral).
fn hex_ring_append(center: HexCell, radius: u32, out: &mut Vec<HexCell>) {
    // Start at center + radius * direction[4] (SW). Walk 6 sides,
    // each `radius` steps, advancing along direction[(side+2) % 6].
    let mut cell = hex_add(center, hex_scale(HEX_DIRECTIONS[4], radius as i32));
    for dir in &HEX_DIRECTIONS {
        for _ in 0..radius {
            out.push(cell);
            cell = hex_add(cell, *dir);
        }
    }
}

// =============================================================================
// World ↔ axial projection
// =============================================================================

const SQRT_3: f32 = 1.732_050_8;

/// Convert axial cell center to world coords for `grid.orientation`.
pub fn axial_to_world(grid: &HexGrid, cell: HexCell) -> Vec2 {
    let s = grid.cell_size;
    let q = cell.q as f32;
    let r = cell.r as f32;
    match grid.orientation {
        HexOrientation::Pointy => {
            let x = s * (SQRT_3 * q + (SQRT_3 / 2.0) * r);
            let y = s * (3.0 / 2.0) * r;
            [x, y]
        }
        HexOrientation::Flat => {
            let x = s * (3.0 / 2.0) * q;
            let y = s * ((SQRT_3 / 2.0) * q + SQRT_3 * r);
            [x, y]
        }
    }
}

/// Convert world coords to the containing axial cell.
pub fn world_to_axial(grid: &HexGrid, world: Vec2) -> HexCell {
    let s = grid.cell_size;
    let (fq, fr) = match grid.orientation {
        HexOrientation::Pointy => {
            let fq = ((SQRT_3 / 3.0) * world[0] - (1.0 / 3.0) * world[1]) / s;
            let fr = (2.0 / 3.0) * world[1] / s;
            (fq, fr)
        }
        HexOrientation::Flat => {
            let fq = (2.0 / 3.0) * world[0] / s;
            let fr = (-(1.0 / 3.0) * world[0] + (SQRT_3 / 3.0) * world[1]) / s;
            (fq, fr)
        }
    };
    // Round in cube space.
    let fx = fq;
    let fz = fr;
    let fy = -fx - fz;
    cube_to_axial(cube_round(fx, fy, fz))
}

/// Six corners of `cell` in world coords, CCW order starting at the
/// +x corner (pointy) or +x corner (flat — same numerical start).
pub fn axial_to_world_vertices(grid: &HexGrid, cell: HexCell, out: &mut Vec<Vec2>) {
    out.clear();
    let center = axial_to_world(grid, cell);
    let s = grid.cell_size;
    // Pointy: corners at 30°, 90°, 150°, 210°, 270°, 330°.
    // Flat:   corners at 0°, 60°, 120°, 180°, 240°, 300°.
    let angle_offset_deg: f32 = match grid.orientation {
        HexOrientation::Pointy => 30.0,
        HexOrientation::Flat => 0.0,
    };
    for i in 0..6 {
        let angle_rad = (angle_offset_deg + 60.0 * i as f32).to_radians();
        out.push([
            center[0] + s * angle_rad.cos(),
            center[1] + s * angle_rad.sin(),
        ]);
    }
}

// =============================================================================
// GridMath impl
// =============================================================================

impl GridMath for HexGrid {
    type Cell = HexCell;

    fn world_to_cell(&self, world: Vec2) -> HexCell {
        world_to_axial(self, world)
    }

    fn cell_to_world_center(&self, cell: HexCell) -> Vec2 {
        axial_to_world(self, cell)
    }

    fn cell_to_world_vertices(&self, cell: HexCell, out: &mut Vec<Vec2>) {
        axial_to_world_vertices(self, cell, out);
    }

    fn neighbors(&self, cell: HexCell, out: &mut Vec<HexCell>) {
        out.clear();
        for d in HEX_DIRECTIONS.iter() {
            out.push(hex_add(cell, *d));
        }
    }

    fn distance(&self, a: HexCell, b: HexCell) -> u32 {
        hex_distance(a, b)
    }

    fn line(&self, a: HexCell, b: HexCell, out: &mut Vec<HexCell>) {
        hex_line(a, b, out);
    }

    fn range(&self, center: HexCell, radius: u32, out: &mut Vec<HexCell>) {
        hex_range(center, radius, out);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cube_invariant_holds_for_axial_conversion() {
        for q in -10..=10 {
            for r in -10..=10 {
                let c = axial_to_cube(HexCell::new(q, r));
                assert_eq!(c.x + c.y + c.z, 0);
            }
        }
    }

    #[test]
    fn cube_axial_round_trip() {
        for q in -5..=5 {
            for r in -5..=5 {
                let h = HexCell::new(q, r);
                assert_eq!(cube_to_axial(axial_to_cube(h)), h);
            }
        }
    }

    #[test]
    fn offset_round_trip_all_variants() {
        for variant in [
            HexOffset::OddR,
            HexOffset::EvenR,
            HexOffset::OddQ,
            HexOffset::EvenQ,
        ] {
            for q in -5..=5 {
                for r in -5..=5 {
                    let h = HexCell::new(q, r);
                    let off = axial_to_offset(h, variant);
                    let back = offset_to_axial(off, variant);
                    assert_eq!(back, h, "round-trip {variant:?} at {h:?}");
                }
            }
        }
    }

    #[test]
    fn neighbors_have_distance_one() {
        let g = HexGrid::pointy(1.0);
        let center = HexCell::new(3, -2);
        let mut out = Vec::new();
        g.neighbors(center, &mut out);
        assert_eq!(out.len(), 6);
        for n in &out {
            assert_eq!(hex_distance(center, *n), 1, "neighbor {n:?}");
        }
    }

    #[test]
    fn neighbors_distinct() {
        let g = HexGrid::flat(1.0);
        let mut out = Vec::new();
        g.neighbors(HexCell::new(0, 0), &mut out);
        for i in 0..out.len() {
            for j in (i + 1)..out.len() {
                assert_ne!(out[i], out[j], "duplicate neighbor");
            }
        }
    }

    #[test]
    fn distance_self_is_zero() {
        assert_eq!(hex_distance(HexCell::new(7, -3), HexCell::new(7, -3)), 0);
    }

    #[test]
    fn distance_matches_red_blob_examples() {
        // From Red Blob's table: distance from (0,0) to (3,-2) = 3.
        assert_eq!(hex_distance(HexCell::new(0, 0), HexCell::new(3, -2)), 3);
        // (0,0) → (-2,-2) → cube diff (-2, 4, -2) → max |.| = 4.
        assert_eq!(hex_distance(HexCell::new(0, 0), HexCell::new(-2, -2)), 4);
    }

    #[test]
    fn cube_round_preserves_invariant() {
        // Arbitrary float triple that doesn't already sum to zero —
        // round must reconstruct one axis to restore the invariant.
        let c = cube_round(1.4, -2.6, 1.1);
        assert_eq!(c.x + c.y + c.z, 0);
    }

    #[test]
    fn hex_line_endpoints() {
        let mut out = Vec::new();
        hex_line(HexCell::new(0, 0), HexCell::new(3, -2), &mut out);
        assert_eq!(out.first(), Some(&HexCell::new(0, 0)));
        assert_eq!(out.last(), Some(&HexCell::new(3, -2)));
    }

    #[test]
    fn hex_line_length_matches_distance_plus_one() {
        for (a, b) in [
            (HexCell::new(0, 0), HexCell::new(5, 0)),
            (HexCell::new(0, 0), HexCell::new(-3, 4)),
            (HexCell::new(2, -1), HexCell::new(2, -1)),
        ] {
            let mut out = Vec::new();
            hex_line(a, b, &mut out);
            assert_eq!(out.len() as u32, hex_distance(a, b) + 1);
        }
    }

    #[test]
    fn hex_line_steps_are_neighbors() {
        let mut out = Vec::new();
        hex_line(HexCell::new(-2, 1), HexCell::new(4, -3), &mut out);
        for w in out.windows(2) {
            assert_eq!(hex_distance(w[0], w[1]), 1, "{:?} → {:?}", w[0], w[1]);
        }
    }

    #[test]
    fn ring_count_equals_six_times_radius() {
        for radius in 1..=5 {
            let mut out = Vec::new();
            hex_ring(HexCell::new(0, 0), radius, &mut out);
            assert_eq!(out.len() as u32, 6 * radius, "radius {radius}");
            for cell in &out {
                assert_eq!(hex_distance(HexCell::new(0, 0), *cell), radius);
            }
        }
    }

    #[test]
    fn ring_radius_zero_is_center_only() {
        let mut out = Vec::new();
        hex_ring(HexCell::new(2, -1), 0, &mut out);
        assert_eq!(out, vec![HexCell::new(2, -1)]);
    }

    #[test]
    fn range_count_matches_centered_hexagon_formula() {
        // |cells within radius N of center| = 1 + 3N(N+1) (Red Blob).
        for n in 0..=5 {
            let mut out = Vec::new();
            hex_range(HexCell::new(0, 0), n, &mut out);
            let expected = 1 + 3 * n * (n + 1);
            assert_eq!(out.len() as u32, expected, "radius {n}");
        }
    }

    #[test]
    fn world_round_trip_pointy() {
        let g = HexGrid::pointy(2.5);
        for q in -5..=5 {
            for r in -5..=5 {
                let h = HexCell::new(q, r);
                let center = axial_to_world(&g, h);
                let back = world_to_axial(&g, center);
                assert_eq!(back, h, "round-trip pointy at {h:?}");
            }
        }
    }

    #[test]
    fn world_round_trip_flat() {
        let g = HexGrid::flat(1.7);
        for q in -5..=5 {
            for r in -5..=5 {
                let h = HexCell::new(q, r);
                let center = axial_to_world(&g, h);
                let back = world_to_axial(&g, center);
                assert_eq!(back, h, "round-trip flat at {h:?}");
            }
        }
    }

    #[test]
    fn vertices_six_corners_at_radius() {
        let g = HexGrid::pointy(3.0);
        let mut v = Vec::new();
        axial_to_world_vertices(&g, HexCell::new(0, 0), &mut v);
        assert_eq!(v.len(), 6);
        for p in &v {
            let dist = (p[0] * p[0] + p[1] * p[1]).sqrt();
            assert!(
                (dist - 3.0).abs() < 1e-4,
                "corner not at radius 3: {:?} dist {}",
                p,
                dist
            );
        }
    }

    #[test]
    fn pointy_vs_flat_swap_width_height() {
        // Pointy width = sqrt(3) * size; height = 2 * size.
        // Flat width = 2 * size; height = sqrt(3) * size.
        let s = 4.0_f32;
        let gp = HexGrid::pointy(s);
        let gf = HexGrid::flat(s);
        // Neighbor at q=1: world distance equals hex width (pointy)
        // or 3/2*size (flat — column step is 1.5*size in x).
        let dp = axial_to_world(&gp, HexCell::new(1, 0));
        assert!((dp[0] - s * SQRT_3).abs() < 1e-4, "pointy +q step x");
        assert!(dp[1].abs() < 1e-4, "pointy +q step y");
        let df = axial_to_world(&gf, HexCell::new(1, 0));
        assert!((df[0] - s * 1.5).abs() < 1e-4, "flat +q step x");
    }

    #[test]
    fn offset_oddr_matches_red_blob_table() {
        // Red Blob's "Offset coordinates" diagram: in odd-r layout,
        // axial (0,1) maps to offset col=0,row=1; axial (1,1) → (1,1);
        // axial (-1,1) → offset col=-1, row=1.
        assert_eq!(
            axial_to_offset(HexCell::new(0, 1), HexOffset::OddR),
            OffsetCell::new(0, 1)
        );
        assert_eq!(
            axial_to_offset(HexCell::new(1, 1), HexOffset::OddR),
            OffsetCell::new(1, 1)
        );
        // Axial (0, 2) under odd-r: r=2 (even), col = q + (2 - 0)/2
        // = 0 + 1 = 1. So (0, 2) → (1, 2).
        assert_eq!(
            axial_to_offset(HexCell::new(0, 2), HexOffset::OddR),
            OffsetCell::new(1, 2)
        );
    }

    #[test]
    fn world_to_axial_inside_known_cell() {
        let g = HexGrid::pointy(2.0);
        // Sample near the center of (3, -1) — should snap to it.
        let center = axial_to_world(&g, HexCell::new(3, -1));
        let nudged = [center[0] + 0.1, center[1] - 0.1];
        assert_eq!(world_to_axial(&g, nudged), HexCell::new(3, -1));
    }
}
