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
    /// Route the Fill "Fill adjust" modal's controls (the threshold slider + Done/Cancel). Returns
    /// `true` when handled — mirrors the other `route_*_event` early-dispatch helpers in
    /// [`crate::tool::PainterTool::handle_panel_event`]. The slider re-fills live from the pre-fill
    /// snapshot; Done keeps the fill (one undo entry), Cancel reverts it.
    pub(crate) fn route_fill_event(&mut self, event: &ph2d_editor_core::tool::PanelEvent) -> bool {
        use ph2d_editor_core::ids as core_ids;
        use ph2d_editor_core::tool::PanelEvent;
        match event {
            PanelEvent::SetValue(id, v) if *id == core_ids::PAINTER_FILL_MODAL_SLIDER => {
                self.set_fill_threshold(*v as f32);
                true
            }
            PanelEvent::Click(id) if *id == core_ids::PAINTER_FILL_MODAL_DONE => {
                self.fill_commit();
                true
            }
            PanelEvent::Click(id) if *id == core_ids::PAINTER_FILL_MODAL_CANCEL => {
                self.fill_cancel();
                true
            }
            _ => false,
        }
    }

    /// Map the Fill threshold track (`0..1`) to a per-channel colour tolerance (`0..255`).
    fn fill_tolerance(&self) -> u8 {
        (self.paint.fill_threshold.clamp(0.0, 1.0) * 255.0).round() as u8
    }

    /// The connected COMPONENT (4-connected, `≥128` inside) of the active selection mask that contains texel
    /// `(sx, sy)` — `255` inside that one region, `0` elsewhere. Lets a ColorDrop fill only the selection
    /// area the colour was dropped on, not every disjoint area (Enio 2026-07-04). All-`0` when the drop is
    /// outside the selection.
    fn selection_component_at(&self, sx: usize, sy: usize, w: usize, h: usize) -> Vec<u8> {
        let mask = &self.paint.selection_mask;
        let mut comp = vec![0u8; w * h];
        if mask.len() != w * h || sx >= w || sy >= h || mask[sy * w + sx] < 128 {
            return comp; // drop outside the selection → fills nothing
        }
        let mut stack = vec![(sx, sy)];
        comp[sy * w + sx] = 255;
        while let Some((x, y)) = stack.pop() {
            for (dx, dy) in [(-1i32, 0), (1, 0), (0, -1), (0, 1)] {
                let (nx, ny) = (x as i32 + dx, y as i32 + dy);
                if nx < 0 || ny < 0 || nx >= w as i32 || ny >= h as i32 {
                    continue;
                }
                let idx = ny as usize * w + nx as usize;
                if comp[idx] == 0 && mask[idx] >= 128 {
                    comp[idx] = 255;
                    stack.push((nx as usize, ny as usize));
                }
            }
        }
        comp
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
        // The ColorDrop clips to the ONE selection region the colour was dropped on — the connected
        // component of the selection mask under the seed — NOT every disjoint selection area (Enio
        // 2026-07-04). Computed before the mutable canvas borrow. `None` when there's no live selection.
        let clip = (self.paint.selection_active && self.paint.selection_mask.len() == w * h)
            .then(|| self.selection_component_at(sx, sy, w, h));
        let buf = crate::tool::paint::plane_fork::fork_canvas(
            &mut self.canvas_rgba,
            &self.undo.write_state,
            self.source_size.0,
        );
        if buf.len() != w * h * 4 {
            return;
        }
        // Restore the pre-fill pixels so the fill always starts from the original layer.
        if self.paint.fill_snapshot.len() == buf.len() {
            buf.copy_from_slice(&self.paint.fill_snapshot);
        }
        let filled = flood_fill(buf, w, h, (sx, sy), color, tol);
        // Clip the fill to the drop's selection region — the flood writes `canvas_rgba` directly, bypassing
        // the per-dab stamp gate, so revert texels outside that component (feathered coverage kept inside it,
        // 0 outside) back to the pre-fill snapshot (ADR-0103).
        if let Some(comp) = &clip {
            let mask = &self.paint.selection_mask;
            let snap = &self.paint.fill_snapshot;
            let n = (w * h).min(snap.len() / 4);
            for i in 0..n {
                // Keep the feathered selection coverage ONLY inside the drop's component; 0 elsewhere.
                let keep = if comp[i] >= 128 {
                    f32::from(mask[i]) / 255.0
                } else {
                    0.0
                };
                if keep >= 1.0 {
                    continue;
                }
                let b = i * 4;
                for c in 0..4 {
                    let painted = f32::from(buf[b + c]);
                    let orig = f32::from(snap[b + c]);
                    buf[b + c] = (painted * keep + orig * (1.0 - keep))
                        .round()
                        .clamp(0.0, 255.0) as u8;
                }
            }
        }
        // The buffer now holds the snapshot everywhere EXCEPT the new fill. Dirty the UNION of the
        // previous fill and the new one so a SHRINKING fill re-uploads the pixels it vacated (restored to
        // the snapshot) — marking only the new, smaller rect would leave the old overflow on screen.
        let dirty = match (self.paint.fill_last_rect, filled) {
            (Some(prev), Some(new)) => super::union_region(prev, new),
            (Some(prev), None) => prev,
            (None, Some(new)) => new,
            (None, None) => return,
        };
        self.paint.fill_last_rect = filled;
        self.mark_dirty(dirty);
    }

    /// Route a Fill-mode canvas pointer — the ColorDrop drop gesture: drag the colour onto the canvas
    /// (Down begins + snapshots, Move tracks the cursor) and release (Up) to flood-fill at the drop
    /// point. The threshold is then tuned in the floating Fill modal (the shell drives
    /// [`Self::set_fill_threshold`] / [`Self::fill_commit`] / [`Self::fill_cancel`]).
    pub(super) fn fill_pointer(&mut self, ev: CanvasPointer) -> bool {
        match ev.phase {
            PointerPhase::Down => {
                self.fill_begin_drop(ev.pos);
                true
            }
            PointerPhase::Move => {
                // The fill lands where the drag RELEASES — track the cursor here (fill runs on Up).
                if self.paint.fill_seed.is_some() {
                    self.paint.fill_seed = Some(ev.pos);
                }
                true
            }
            PointerPhase::Up => {
                if self.paint.fill_seed.is_some() {
                    self.paint.fill_seed = Some(ev.pos);
                    self.refill_from_snapshot();
                }
                true
            }
            PointerPhase::Hover => false,
        }
    }

    /// Start a ColorDrop: snapshot the layer (for undo + live re-fill) and record the seed. The fill
    /// itself runs on release ([`Self::fill_pointer`]).
    fn fill_begin_drop(&mut self, pos: [f32; 2]) {
        // One structural-undo entry for the whole drop + modal adjust (pre-fill → committed fill).
        self.paint.stroke_undo = Some(self.snapshot_model());
        self.paint.fill_snapshot = (*self.canvas_rgba).clone();
        self.paint.fill_seed = Some(pos);
        self.paint.fill_last_rect = None;
    }

    /// Whether a ColorDrop is pending its modal adjust (a drop happened, not yet committed/cancelled).
    #[must_use]
    pub fn has_active_fill(&self) -> bool {
        self.paint.fill_seed.is_some()
    }

    /// The current Fill threshold (`0..1`) — the Fill modal's slider is seeded from this.
    #[must_use]
    pub fn fill_threshold(&self) -> f32 {
        self.paint.fill_threshold
    }

    /// Set the Fill threshold from the modal slider (`0..1`) and re-fill live from the snapshot.
    pub fn set_fill_threshold(&mut self, t: f32) {
        self.paint.fill_threshold = t.clamp(0.0, 1.0);
        self.refill_from_snapshot();
    }

    /// Enter **Fill** for a momentary ColorDrop (the shell's C&F drag) and remember the mode to RESTORE when
    /// the fill finalizes, so the drag returns to the shape/brush the user was using (Enio 2026-07-03). A
    /// deliberate Fill-tool selection uses `set_paint_tool_mode("fill")` instead (no return).
    pub fn begin_colordrop_fill(&mut self, return_mode: &str) {
        self.set_paint_tool_mode("fill");
        self.paint.fill_return_mode = Some(return_mode.to_string());
    }

    /// Finalize the current fill (modal **Done**, or the dwell live-adjust release): push the undo entry,
    /// drop the transient state. `pub` so the shell's dwell gesture can commit without the modal.
    pub fn fill_commit(&mut self) {
        if self.paint.fill_seed.is_none() {
            return;
        }
        self.close_stroke(); // pushes the pre-fill → filled structural-undo entry
        self.paint.fill_seed = None;
        self.paint.fill_snapshot = Vec::new();
        self.paint.fill_last_rect = None;
        self.restore_after_colordrop();
    }

    /// Restore the pre-ColorDrop mode after a momentary C&F fill finalizes (Done / Cancel / dwell release),
    /// or when a C&F drag never reached the canvas (released off-sprite → the picker path). No-op for a
    /// deliberate Fill / a plain click (no return recorded). `fill_seed` is already cleared, so the
    /// `set_paint_tool_mode` re-entrant `fill_commit` is a no-op (no recursion) and `fill_return_mode` is
    /// taken (drives it exactly once). `pub` so the shell's picker path can end a missed drag.
    pub fn restore_after_colordrop(&mut self) {
        if let Some(ret) = self.paint.fill_return_mode.take() {
            self.set_paint_tool_mode(&ret);
        }
    }

    /// Discard the current fill (modal **Cancel**): restore the pre-fill pixels and drop the pending
    /// undo entry, so the layer is exactly as it was before the drop.
    pub fn fill_cancel(&mut self) {
        if self.paint.fill_seed.is_none() {
            return;
        }
        let n = self.paint.fill_snapshot.len();
        if n > 0 && self.canvas_rgba.len() == n {
            let buf = crate::tool::paint::plane_fork::fork_canvas(
                &mut self.canvas_rgba,
                &self.undo.write_state,
                self.source_size.0,
            );
            buf.copy_from_slice(&self.paint.fill_snapshot);
        }
        if let Some(rect) = self.paint.fill_last_rect {
            self.mark_dirty(rect); // repaint what the fill had painted
        }
        self.paint.stroke_undo = None; // no committed change → no undo entry
        self.paint.fill_seed = None;
        self.paint.fill_snapshot = Vec::new();
        self.paint.fill_last_rect = None;
        self.restore_after_colordrop();
    }

    /// Drop a pending Fill ColorDrop WITHOUT touching pixels or dirtying — for a document rebind, where
    /// the canvas is being replaced anyway (so `fill_cancel`'s snapshot-restore would just write into a
    /// buffer about to be discarded, and could stamp the OLD sprite's pixels onto the new one). See
    /// [`PainterTool::reset_transient_edit_state`].
    pub(super) fn abandon_pending_fill(&mut self) {
        self.paint.fill_seed = None;
        self.paint.fill_snapshot = Vec::new();
        self.paint.fill_last_rect = None;
        self.paint.stroke_undo = None;
        self.paint.fill_return_mode = None; // drop the momentary-fill return (the doc is being replaced)
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
