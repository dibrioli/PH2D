//! Drag-time **snapping** for the Line editor — the Shift 15° direction snap (transcendental-free, via a
//! precomputed unit-direction table) and the automatic point-to-point alignment (snap a dragged point to
//! another point's row / column). Sibling of [`super::line`] (LOC cap); a second `impl PainterTool` block
//! over the same private `PaintState` fields (via `super::*`).

use super::*;

/// A dragged point auto-snaps to another point's row / column when within this fraction of the grab
/// tolerance (small, per Enio) — the automatic horizontal/vertical alignment (no Shift).
const POINT_SNAP_FRAC: f32 = 0.6;

impl PainterTool {
    /// The snapped target for placing / dragging a Line point at index `g` — ALL active snaps compose
    /// (Enio 2026-07-02, "os dois devem atuar juntos"):
    ///   1. **Grid** — start from the shell's grid-snapped position when forwarded (base quantisation).
    ///   2. **Angle** — with Shift, constrain the incoming segment's direction to 15° from `points[g-1]`.
    ///   3. **Point** — align to another point's row / column (or land on it) when within the threshold.
    ///
    /// Point wins last (an exact join beats the grid / angle), then angle, then grid — so none inhibits
    /// another. `g` may be `points.len()` for a not-yet-pushed new point (its anchor is the current last).
    pub(super) fn line_drag_target(&self, g: usize, pos: [f32; 2]) -> [f32; 2] {
        let base = self.paint.grid_snap_pos.unwrap_or(pos);
        let angled = self.line_snapped_grab(g, base); // self-gates on Shift
        self.line_point_snap(g, angled)
    }

    /// Apply the 15° snap to a grabbed point at index `g`: when armed (Shift) and a previous point anchors
    /// the segment, quantise the direction from `points[g-1]` to the nearest 15°; otherwise `pos` unchanged.
    pub(super) fn line_snapped_grab(&self, g: usize, pos: [f32; 2]) -> [f32; 2] {
        if !self.paint.line_snap || g == 0 {
            return pos;
        }
        match self
            .paint
            .line
            .as_ref()
            .and_then(|ed| ed.points().get(g - 1))
        {
            Some(&anchor) => snap_to_15(anchor, pos),
            None => pos,
        }
    }

    /// Auto-align a dragged point to the OTHER points: snap its X to the nearest other point's column and
    /// its Y to the nearest other point's row, each only when within a small threshold. Both snapping to the
    /// same point drops it right on top (the explicit "drag one point onto another" case). No Shift.
    pub(super) fn line_point_snap(&self, g: usize, pos: [f32; 2]) -> [f32; 2] {
        let Some(ed) = self.paint.line.as_ref() else {
            return pos;
        };
        let thresh = self.paint.shape_grab_tol_px * POINT_SNAP_FRAC;
        let (mut best_x, mut bx) = (pos[0], thresh);
        let (mut best_y, mut by) = (pos[1], thresh);
        for (j, p) in ed.points().iter().enumerate() {
            if j == g {
                continue;
            }
            let dx = (pos[0] - p[0]).abs();
            if dx <= bx {
                bx = dx;
                best_x = p[0];
            }
            let dy = (pos[1] - p[1]).abs();
            if dy <= by {
                by = dy;
                best_y = p[1];
            }
        }
        [best_x, best_y]
    }
}

/// `1/√2` = cos/sin of 45° — the named constant (clippy `approx_constant` rejects the 0.7071 literal).
const D45: f32 = std::f32::consts::FRAC_1_SQRT_2;

/// The 24 unit directions at 15° increments (`cos`, `sin` of `k·15°`, `k = 0..24`) — compile-time
/// constants so the snap is transcendental-free at runtime (HR-5). Used by [`snap_to_15`].
#[rustfmt::skip]
const DIRS_15: [[f32; 2]; 24] = [
    [1.0, 0.0],            [0.965_925_8, 0.258_819_04],   [0.866_025_4, 0.5],
    [D45, D45],            [0.5, 0.866_025_4],            [0.258_819_04, 0.965_925_8],
    [0.0, 1.0],           [-0.258_819_04, 0.965_925_8],  [-0.5, 0.866_025_4],
    [-D45, D45],          [-0.866_025_4, 0.5],           [-0.965_925_8, 0.258_819_04],
    [-1.0, 0.0],          [-0.965_925_8, -0.258_819_04], [-0.866_025_4, -0.5],
    [-D45, -D45],         [-0.5, -0.866_025_4],          [-0.258_819_04, -0.965_925_8],
    [0.0, -1.0],           [0.258_819_04, -0.965_925_8],  [0.5, -0.866_025_4],
    [D45, -D45],           [0.866_025_4, -0.5],           [0.965_925_8, -0.258_819_04],
];

/// Snap `cursor` onto the 15°-graduated ray from `anchor`: keep the distance, quantise the direction to
/// the nearest of the 24 [`DIRS_15`] by maximum dot product (no `atan2` — HR-5). A ~zero move is unchanged.
pub(super) fn snap_to_15(anchor: [f32; 2], cursor: [f32; 2]) -> [f32; 2] {
    let v = [cursor[0] - anchor[0], cursor[1] - anchor[1]];
    let len = (v[0] * v[0] + v[1] * v[1]).sqrt();
    if len < 1e-4 {
        return cursor;
    }
    let mut best = DIRS_15[0];
    let mut best_dot = f32::MIN;
    for d in DIRS_15 {
        let dot = v[0] * d[0] + v[1] * d[1];
        if dot > best_dot {
            best_dot = dot;
            best = d;
        }
    }
    [anchor[0] + best[0] * len, anchor[1] + best[1] * len]
}
