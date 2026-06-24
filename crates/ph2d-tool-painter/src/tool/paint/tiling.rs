//! Seamless **Tiling** (wrap-around) painting — split from `paint.rs` for the workspace LOC cap.
//! When Tiling is on for an axis, a dab whose footprint crosses that sprite edge is replicated with a
//! copy shifted by ∓span, so a stroke crossing the border also paints the part that wraps onto the
//! opposite edge → the painted texture is seamless when the sprite is repeated as a tile.

use crate::tool::PainterTool;
use ph2d_painter_brush::Dab;

/// Replicate each dab across the wrapped sprite edges for the enabled `tiling` axes. The originals
/// are always included; an edge-crossing dab additionally gets its wrapped copy (up to 4 per dab at a
/// corner). Returns the expanded list. Called only when at least one axis tiles.
#[must_use]
pub(super) fn tiled_dabs(dabs: &[Dab], source_size: (u32, u32), tiling: [bool; 2]) -> Vec<Dab> {
    let (fw, fh) = (source_size.0 as f32, source_size.1 as f32);
    let mut out = Vec::with_capacity(dabs.len() * 2);
    for d in dabs {
        let mut xs = [0.0f32; 3];
        let nx = axis_offsets(d.center[0], d.radius_px, fw, tiling[0], &mut xs);
        let mut ys = [0.0f32; 3];
        let ny = axis_offsets(d.center[1], d.radius_px, fh, tiling[1], &mut ys);
        for &dx in &xs[..nx] {
            for &dy in &ys[..ny] {
                out.push(Dab {
                    center: [d.center[0] + dx, d.center[1] + dy],
                    ..*d
                });
            }
        }
    }
    out
}

/// Wrap offsets for one axis of a tiled dab: always `0`, plus `∓span` when the dab's footprint
/// (`centre ± radius`) crosses an enabled edge — so an edge dab also paints the wrapped copy. Writes
/// up to 3 offsets into `out` and returns the count (`1` when tiling is off or the dab is interior).
fn axis_offsets(centre: f32, radius: f32, span: f32, tiling: bool, out: &mut [f32; 3]) -> usize {
    out[0] = 0.0;
    let mut n = 1;
    if tiling {
        if centre + radius > span {
            out[n] = -span; // crosses the far edge → wrap to the near edge
            n += 1;
        }
        if centre - radius < 0.0 {
            out[n] = span; // crosses the near edge → wrap to the far edge
            n += 1;
        }
    }
    n
}

impl PainterTool {
    /// Toggle seamless **Tiling** (wrap-around painting) for `axis` (`0` = X, `1` = Y). Plain paint
    /// state — no undo / pixel touch (it only affects future stamps).
    pub fn toggle_brush_tiling(&mut self, axis: usize) {
        if axis < 2 {
            self.paint.tiling[axis] = !self.paint.tiling[axis];
        }
    }

    /// The current per-axis Tiling flags `[x, y]` — the panel reads this to show the toggle state.
    #[must_use]
    pub fn brush_tiling(&self) -> [bool; 2] {
        self.paint.tiling
    }
}
