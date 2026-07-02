//! The **Fill** (Bucket) tool's flood-fill core + the Procreate-style ColorDrop gesture state.
//!
//! [`flood_fill`] is the pure algorithm: a scanline (span) flood fill of the 4-connected region whose
//! colour is within `tol` of the seed pixel, painted with `fill`. The gesture (drag the colour onto the
//! canvas → drop → drag to adjust the threshold) is driven by [`PainterTool`]'s Fill methods, which snapshot
//! the pre-fill layer so every threshold change re-fills from the original pixels (not the already-filled
//! result), exactly like Procreate's live ColorDrop-Threshold.

use super::Region;
use crate::tool::PainterTool;
use ph2d_editor_core::tool::{CanvasPointer, PointerPhase};
use std::sync::Arc;

/// ColorDrop threshold change per pixel of the adjust drag (≈ full `0..1` over ~330 px).
const FILL_ADJUST_SENS: f32 = 0.003;

/// Max per-channel (RGBA) difference from the seed colour for a pixel to be filled, at a given tolerance.
#[inline]
fn color_match(px: &[u8], i: usize, seed: [u8; 4], tol: u8) -> bool {
    let d = px[i]
        .abs_diff(seed[0])
        .max(px[i + 1].abs_diff(seed[1]))
        .max(px[i + 2].abs_diff(seed[2]))
        .max(px[i + 3].abs_diff(seed[3]));
    d <= tol
}

/// Scanline flood-fill `px` (straight RGBA8, `w*h`) from `seed`, replacing the connected region within
/// `tol` of the seed colour with opaque `fill`. Returns the affected bounding box, or `None` if the seed
/// is out of bounds or nothing changed. A `visited` bitmap makes it correct even when `fill` equals the
/// seed colour (no infinite loop).
pub(super) fn flood_fill(
    px: &mut [u8],
    w: usize,
    h: usize,
    seed: (usize, usize),
    fill: [u8; 3],
    tol: u8,
) -> Option<Region> {
    if w == 0 || h == 0 || px.len() != w * h * 4 {
        return None;
    }
    let (sx, sy) = seed;
    if sx >= w || sy >= h {
        return None;
    }
    let si = (sy * w + sx) * 4;
    let seed_c = [px[si], px[si + 1], px[si + 2], px[si + 3]];
    let fillpx = [fill[0], fill[1], fill[2], 255];
    // No-op fast path: the seed already IS the fill colour (a solid fill would change nothing there).
    // We still run — an anti-aliased edge may differ — but skip if the seed itself is exactly fill.
    let mut visited = vec![false; w * h];
    let mut stack: Vec<(usize, usize)> = vec![seed];
    let (mut minx, mut miny, mut maxx, mut maxy) = (sx, sy, sx, sy);
    let mut changed = false;
    while let Some((x, y)) = stack.pop() {
        if visited[y * w + x] {
            continue;
        }
        // Grow the horizontal span [lx, rx] around x over matching, unvisited pixels.
        let mut lx = x;
        while lx > 0
            && !visited[y * w + lx - 1]
            && color_match(px, (y * w + lx - 1) * 4, seed_c, tol)
        {
            lx -= 1;
        }
        let mut rx = x;
        while rx + 1 < w
            && !visited[y * w + rx + 1]
            && color_match(px, (y * w + rx + 1) * 4, seed_c, tol)
        {
            rx += 1;
        }
        // Fill the span.
        for xx in lx..=rx {
            let idx = y * w + xx;
            visited[idx] = true;
            let o = idx * 4;
            if px[o..o + 4] != fillpx {
                changed = true;
            }
            px[o..o + 4].copy_from_slice(&fillpx);
        }
        minx = minx.min(lx);
        maxx = maxx.max(rx);
        miny = miny.min(y);
        maxy = maxy.max(y);
        // Seed the rows above and below across the span.
        for ny in [y.checked_sub(1), (y + 1 < h).then_some(y + 1)]
            .into_iter()
            .flatten()
        {
            for xx in lx..=rx {
                if !visited[ny * w + xx] && color_match(px, (ny * w + xx) * 4, seed_c, tol) {
                    stack.push((xx, ny));
                }
            }
        }
    }
    changed.then_some(Region {
        x: minx as u32,
        y: miny as u32,
        w: (maxx - minx + 1) as u32,
        h: (maxy - miny + 1) as u32,
    })
}

impl PainterTool {
    /// Map the Fill threshold track (`0..1`) to a per-channel colour tolerance (`0..255`).
    fn fill_tolerance(&self) -> u8 {
        (self.paint.fill_threshold.clamp(0.0, 1.0) * 255.0).round() as u8
    }

    /// Run the flood fill from the ORIGINAL (pre-fill) pixels: restore the snapshot into the canvas, then
    /// fill from `fill_seed` at the current threshold with the current brush colour. Shared by the drop
    /// and every live threshold adjustment. No-op without a seed / snapshot.
    pub(super) fn refill_from_snapshot(&mut self) {
        let (Some(seed), size) = (self.paint.fill_seed, self.source_size) else {
            return;
        };
        let (w, h) = (size.0 as usize, size.1 as usize);
        if w == 0 || h == 0 {
            return;
        }
        let sx = (seed[0].floor() as i32).clamp(0, w as i32 - 1) as usize;
        let sy = (seed[1].floor() as i32).clamp(0, h as i32 - 1) as usize;
        let tol = self.fill_tolerance();
        let color = [
            (self.paint.brush.color[0].clamp(0.0, 1.0) * 255.0 + 0.5) as u8,
            (self.paint.brush.color[1].clamp(0.0, 1.0) * 255.0 + 0.5) as u8,
            (self.paint.brush.color[2].clamp(0.0, 1.0) * 255.0 + 0.5) as u8,
        ];
        let buf = Arc::make_mut(&mut self.canvas_rgba);
        if buf.len() != w * h * 4 {
            return;
        }
        // Restore the pre-fill pixels so the fill always starts from the original layer.
        if self.paint.fill_snapshot.len() == buf.len() {
            buf.copy_from_slice(&self.paint.fill_snapshot);
        }
        let filled = flood_fill(buf, w, h, (sx, sy), color, tol);
        // Dirty the union of the pre-restore and new fill so a shrinking fill also repaints the vacated
        // pixels. Restoring the whole snapshot already covers it; mark the new rect (or the whole layer
        // if the fill vanished so the restore shows).
        let rect = filled.unwrap_or(Region {
            x: 0,
            y: 0,
            w: w as u32,
            h: h as u32,
        });
        self.mark_dirty(rect);
    }

    /// Route a Fill-mode canvas pointer — the Procreate ColorDrop gesture:
    /// 1. **Drop** (Down → drag → Up): the release point seeds a flood fill at the current threshold.
    /// 2. **Adjust** (the next drag): right/up raises the threshold (bigger fill), left/down lowers it,
    ///    re-filling live from the pre-fill snapshot. Its release commits the fill (one undo step).
    pub(super) fn fill_pointer(&mut self, ev: CanvasPointer) -> bool {
        match ev.phase {
            PointerPhase::Down => {
                if self.paint.fill_adjusting {
                    // Begin the adjust drag (anchor + the threshold at this moment).
                    self.paint.fill_adjust_start = Some(ev.pos);
                    self.paint.fill_base_threshold = self.paint.fill_threshold;
                } else {
                    self.fill_begin_drop(ev.pos);
                }
                true
            }
            PointerPhase::Move => {
                if self.paint.fill_adjusting {
                    if let Some(start) = self.paint.fill_adjust_start {
                        let dx = ev.pos[0] - start[0];
                        let dy = ev.pos[1] - start[1]; // image-space +Y is DOWN
                        // Right (dx > 0) and up (dy < 0) both INCREASE the threshold (bigger fill).
                        let t = self.paint.fill_base_threshold + (dx - dy) * FILL_ADJUST_SENS;
                        self.paint.fill_threshold = t.clamp(0.0, 1.0);
                        self.refill_from_snapshot();
                    }
                } else if self.paint.fill_seed.is_some() {
                    // Dropping — the fill lands where the drag RELEASES, so just track the cursor here
                    // (no per-move flood fill; the fill runs on Up).
                    self.paint.fill_seed = Some(ev.pos);
                }
                true
            }
            PointerPhase::Up => {
                if self.paint.fill_adjusting {
                    // The adjust drag (or a confirming tap) ended → commit the fill.
                    self.paint.fill_adjust_start = None;
                    self.fill_commit();
                } else if self.paint.fill_seed.is_some() {
                    // Drop released → fill at the release point, then arm the live threshold adjust.
                    self.paint.fill_seed = Some(ev.pos);
                    self.refill_from_snapshot();
                    self.paint.fill_adjusting = true;
                    self.paint.fill_base_threshold = self.paint.fill_threshold;
                }
                true
            }
            PointerPhase::Hover => false,
        }
    }

    /// Start a ColorDrop: snapshot the layer (for undo + live re-fill) and record the seed. The fill
    /// itself runs on release ([`Self::fill_pointer`]).
    fn fill_begin_drop(&mut self, pos: [f32; 2]) {
        // One structural-undo entry for the whole drop+adjust (pre-fill → committed fill).
        self.paint.stroke_undo = Some(self.snapshot_model());
        self.paint.fill_snapshot = (*self.canvas_rgba).clone();
        self.paint.fill_seed = Some(pos);
        self.paint.fill_adjusting = false;
        self.paint.fill_adjust_start = None;
    }

    /// Finalize the current fill: push the undo entry and drop the transient ColorDrop state.
    pub(crate) fn fill_commit(&mut self) {
        if self.paint.fill_seed.is_none() && !self.paint.fill_adjusting {
            return;
        }
        self.close_stroke(); // pushes the pre-fill → filled structural-undo entry
        self.paint.fill_seed = None;
        self.paint.fill_snapshot = Vec::new();
        self.paint.fill_adjusting = false;
        self.paint.fill_adjust_start = None;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// An `w×h` RGBA8 buffer: left half `a`, right half `b`.
    fn split(w: usize, h: usize, a: [u8; 4], b: [u8; 4]) -> Vec<u8> {
        let mut px = vec![0u8; w * h * 4];
        for y in 0..h {
            for x in 0..w {
                let o = (y * w + x) * 4;
                px[o..o + 4].copy_from_slice(if x < w / 2 { &a } else { &b });
            }
        }
        px
    }

    fn rgb(px: &[u8], w: usize, x: usize, y: usize) -> [u8; 3] {
        let o = (y * w + x) * 4;
        [px[o], px[o + 1], px[o + 2]]
    }

    #[test]
    fn fills_the_connected_region_and_stops_at_a_colour_boundary() {
        let (w, h) = (8, 8);
        let mut px = split(w, h, [200, 0, 0, 255], [0, 0, 200, 255]);
        let rect = flood_fill(&mut px, w, h, (1, 1), [0, 255, 0], 10).expect("filled");
        for y in 0..h {
            for x in 0..w {
                if x < 4 {
                    assert_eq!(
                        rgb(&px, w, x, y),
                        [0, 255, 0],
                        "red half → green at ({x},{y})"
                    );
                } else {
                    assert_eq!(
                        rgb(&px, w, x, y),
                        [0, 0, 200],
                        "blue half untouched at ({x},{y})"
                    );
                }
            }
        }
        assert_eq!((rect.x, rect.y, rect.w, rect.h), (0, 0, 4, 8));
    }

    #[test]
    fn tolerance_bridges_small_differences() {
        // Two near-identical reds (Δ = 8) meeting a far blue. tol 0 fills only the exact seed colour;
        // tol 20 bridges the near-red into one region; neither crosses into blue.
        let (w, h) = (6, 1);
        let mut px = vec![0u8; w * h * 4];
        let cols = [
            [200, 0, 0, 255],
            [200, 0, 0, 255],
            [208, 0, 0, 255],
            [208, 0, 0, 255],
            [0, 0, 200, 255],
            [0, 0, 200, 255],
        ];
        for (x, c) in cols.iter().enumerate() {
            px[x * 4..x * 4 + 4].copy_from_slice(c);
        }
        let mut tight = px.clone();
        flood_fill(&mut tight, w, h, (0, 0), [0, 255, 0], 0);
        assert_eq!(rgb(&tight, w, 0, 0), [0, 255, 0]);
        assert_eq!(
            rgb(&tight, w, 2, 0),
            [208, 0, 0],
            "Δ=8 not bridged at tol 0"
        );
        let mut loose = px.clone();
        flood_fill(&mut loose, w, h, (0, 0), [0, 255, 0], 20);
        assert_eq!(rgb(&loose, w, 2, 0), [0, 255, 0], "Δ=8 bridged at tol 20");
        assert_eq!(rgb(&loose, w, 4, 0), [0, 0, 200], "far blue never crossed");
    }

    #[test]
    fn out_of_bounds_seed_returns_none() {
        let mut px = vec![255u8; 4 * 4 * 4];
        assert!(flood_fill(&mut px, 4, 4, (9, 9), [1, 2, 3], 0).is_none());
    }

    #[test]
    fn no_change_returns_none() {
        // Seed region already the fill colour ⇒ nothing changes ⇒ None.
        let (w, h) = (4, 4);
        let mut px = vec![0u8; w * h * 4];
        for c in px.chunks_exact_mut(4) {
            c.copy_from_slice(&[0, 255, 0, 255]);
        }
        assert!(flood_fill(&mut px, w, h, (1, 1), [0, 255, 0], 0).is_none());
    }
}
