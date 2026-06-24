//! Seamless **Tiling** (wrap-around) painting — split from `paint.rs` for the workspace LOC cap.
//! When Tiling is on for an axis, a dab whose footprint crosses that sprite edge is replicated with a
//! copy shifted by ∓span, so a stroke crossing the border also paints the part that wraps onto the
//! opposite edge → the painted texture is seamless when the sprite is repeated as a tile.

use crate::tool::PainterTool;
use ph2d_painter_brush::Dab;

/// Repeat-Image **Aspect Ratio** slider range (neighbour spacing as a multiple of the sprite size).
/// `1.0` (the default) abuts the tiles at the sprite edges; the panel maps its `0..1` track onto this.
pub const TILE_ASPECT_MIN: f32 = 0.25;
/// See [`TILE_ASPECT_MIN`].
pub const TILE_ASPECT_MAX: f32 = 4.0;

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

    /// Toggle **Repeat Image** — the on-canvas 3×3 tile preview (the shell draws the sprite repeated
    /// in all 8 neighbour directions). Plain state; the shell reads [`Self::repeat_image`] each frame.
    pub fn toggle_repeat_image(&mut self) {
        self.paint.repeat_image = !self.paint.repeat_image;
    }

    /// Whether the Repeat-Image tile preview is on.
    #[must_use]
    pub fn repeat_image(&self) -> bool {
        self.paint.repeat_image
    }

    /// Set the Repeat-Image **Aspect Ratio** for `axis` (`0` = X, `1` = Y) from the slider's `0..1`
    /// track, mapped onto `[TILE_ASPECT_MIN, TILE_ASPECT_MAX]` (neighbour spacing ×sprite-size).
    pub fn set_tile_aspect(&mut self, axis: usize, t: f32) {
        if axis < 2 {
            let t = t.clamp(0.0, 1.0);
            self.paint.tile_aspect[axis] =
                TILE_ASPECT_MIN + t * (TILE_ASPECT_MAX - TILE_ASPECT_MIN);
        }
    }

    /// The current Repeat-Image aspect `[x, y]` (sprite-size multipliers) — the shell render reads this
    /// for the neighbour offsets; the panel maps it back to the slider track.
    #[must_use]
    pub fn tile_aspect(&self) -> [f32; 2] {
        self.paint.tile_aspect
    }
}
