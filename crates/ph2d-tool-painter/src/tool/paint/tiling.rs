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
    tiled_dabs_grouped(dabs, source_size, tiling).0
}

/// [`tiled_dabs`] + the **group map**: `groups[i]` is the index of the ORIGINAL dab that output `i` was
/// replicated from (copies of one dab are emitted consecutively).
///
/// **Why this exists (sweep 2026-07-12):** a dab's wrapped copies are not new dabs — they are the SAME
/// dab seen from the other side of the seam. Anything drawn PER DAB (Shape/Grain Random-Angle, Random
/// Offset, Randomize Color) must therefore be drawn ONCE and shared by the copies. The paint routes used
/// to iterate the already-wrapped list and pull from `tex_rng` per entry, so the two sides of the seam got
/// DIFFERENT random frames and the tile stopped matching — the one thing Tiling exists to guarantee.
/// Smear/Blur/Clone always got this right (they compute the frame once per dab and wrap inside).
/// Feed this map to [`DabRng`], which replays the stream per group.
#[must_use]
pub(super) fn tiled_dabs_grouped(
    dabs: &[Dab],
    source_size: (u32, u32),
    tiling: [bool; 2],
) -> (Vec<Dab>, Vec<u32>) {
    let (fw, fh) = (source_size.0 as f32, source_size.1 as f32);
    let mut out = Vec::with_capacity(dabs.len() * 2);
    let mut groups = Vec::with_capacity(dabs.len() * 2);
    for (gi, d) in dabs.iter().enumerate() {
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
                groups.push(gi as u32);
            }
        }
    }
    (out, groups)
}

/// The per-dab texture RNG, replayed so that a dab's wrapped Tiling copies share ONE random frame.
///
/// Call [`Self::enter`] once per entry of the dab list, in order, BEFORE any draw for that entry. The
/// first copy of a group draws normally; every further copy re-runs the stream from the group's start, so
/// it makes the IDENTICAL draws. Because all copies of a dab draw the same amount, the stream ends up
/// advanced exactly once per ORIGINAL dab — which is also what makes the un-tiled path **byte-identical**
/// (with no group map every entry is its own group, so `enter` never rewinds anything).
pub(super) struct DabRng {
    rng: u64,
    group_start: u64,
    cur: u32,
}

impl DabRng {
    pub(super) fn new(rng: u64) -> Self {
        Self {
            rng,
            group_start: rng,
            cur: u32::MAX,
        }
    }

    /// Position the stream for list entry `i`, given the group map (empty ⇒ every entry its own group).
    pub(super) fn enter(&mut self, groups: &[u32], i: usize) -> &mut u64 {
        let g = groups.get(i).copied().unwrap_or(i as u32);
        if g == self.cur {
            self.rng = self.group_start; // a wrapped copy: replay its original's draws
        } else {
            self.cur = g;
            self.group_start = self.rng; // a new dab: this is where its draws begin
        }
        &mut self.rng
    }

    /// The stream state to write back (advanced once per original dab).
    pub(super) fn finish(self) -> u64 {
        self.rng
    }
}

/// Fill `out` with the wrap offsets (`[dx, dy]`, always including `[0, 0]`) for a dab at
/// `center`/`radius` under the enabled `tiling` axes — the same edge-crossing logic as [`tiled_dabs`],
/// written into a stack buffer (no alloc) so the **Smear** route can apply one offset to BOTH its lift
/// (`from`) and stamp (`to`) positions (a wrapped copy keeps the same displacement). Returns the count
/// (`1` when interior; up to `nx·ny ≤ 9` for a dab wider than the sprite that crosses both edges).
pub(super) fn tiled_offsets_into(
    center: [f32; 2],
    radius: f32,
    source_size: (u32, u32),
    tiling: [bool; 2],
    out: &mut [[f32; 2]; 9],
) -> usize {
    let (fw, fh) = (source_size.0 as f32, source_size.1 as f32);
    let mut xs = [0.0f32; 3];
    let nx = axis_offsets(center[0], radius, fw, tiling[0], &mut xs);
    let mut ys = [0.0f32; 3];
    let ny = axis_offsets(center[1], radius, fh, tiling[1], &mut ys);
    let mut n = 0;
    for &dx in &xs[..nx] {
        for &dy in &ys[..ny] {
            out[n] = [dx, dy];
            n += 1;
        }
    }
    n
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

    /// Reset the **Tiling** section to defaults (no wrap-around tiling, no Repeat-Image preview).
    pub fn reset_brush_tiling(&mut self) {
        self.paint.tiling = [false, false];
        self.paint.repeat_image = false;
    }
}
