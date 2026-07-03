//! The **Selection** tool (ADR-0103) — a document-wide selection mask that gates every paint op to the
//! selected region, integrated into the painter's single interleaved undo/redo queue.
//!
//! The mask is a single-channel coverage buffer (`w*h` bytes, `0` = outside / `255` = inside; Feather
//! softens the edge), held in [`super::PaintState`]. It is the SELECTION analogue of the Mask brush's
//! `mask_scratch` (see [`super::mask`]): captured into / restored from the `ModelSnapshot` in lock-step
//! with the pixels, and applied at stamp time by reverting texels OUTSIDE the selection to their
//! pre-stamp values ([`Self::restore_deselected_region`], mirror of `restore_protected_region`).
//!
//! Wave 1 lays the state + undo + gate + a minimal rectangular seed. The on-canvas creation MODES
//! (Automatic / Freehand / Rectangle / Ellipse) and boolean operators build on this in Wave 2.

use super::{PainterTool, Region};
use ph2d_editor_core::tool::{CanvasPointer, PointerPhase};
use std::sync::Arc;

/// Largest Feather radius (image px) the `0..1` Feather slider maps to.
const FEATHER_MAX_PX: usize = 64;

/// The in-progress selection gesture — the overlay draws its rubber-band; the mask is rasterized on Up
/// (Automatic writes it live). Held in [`super::PaintState`].
#[derive(Clone, Debug)]
pub(crate) enum SelectionDrag {
    /// Rectangle / Ellipse marquee: a rubber-band from `anchor` to the current point. `ellipse` picks the
    /// shape.
    Marquee {
        anchor: [f32; 2],
        cur: [f32; 2],
        ellipse: bool,
    },
    /// Freehand lasso: the captured path (closed on release).
    Lasso { points: Vec<[f32; 2]> },
    /// Automatic flood-select: the seed + the threshold at gesture start (horizontal drag adjusts it live,
    /// Procreate-style). The mask is written on every change.
    Auto { seed: [f32; 2], thresh0: f32 },
}

impl PainterTool {
    /// `true` while a selection is live (even an empty one, which paints nothing).
    #[must_use]
    pub fn selection_active(&self) -> bool {
        self.paint.selection_active
    }

    /// Selection coverage (`0..=255`) at image texel `(x, y)`; `0` when out of bounds / no selection. Read
    /// by the overlay (marching ants / hatching) and tests.
    #[must_use]
    pub fn selection_coverage_at(&self, x: u32, y: u32) -> u8 {
        let w = self.source_size.0;
        if w == 0 {
            return 0;
        }
        self.paint
            .selection_mask
            .get((y * w + x) as usize)
            .copied()
            .unwrap_or(0)
    }

    /// Snapshot the selection state for the undo model — the buffers live in `PaintState` (private to
    /// `tool::paint`), so the general `snapshot_model` reaches them through here. `Arc`-shared, cheap clone.
    pub(crate) fn selection_for_snapshot(&self) -> (Arc<Vec<u8>>, bool, Arc<Vec<u8>>, f32) {
        (
            Arc::clone(&self.paint.selection_mask),
            self.paint.selection_active,
            Arc::clone(&self.paint.selection_crisp),
            self.paint.selection_feather,
        )
    }

    /// Reinstate the selection state from an undo model (structural undo/redo), keeping the selected region
    /// (effective + crisp accumulator + Feather amount) in sync with the restored layers/pixels.
    pub(crate) fn restore_selection(
        &mut self,
        mask: Arc<Vec<u8>>,
        active: bool,
        crisp: Arc<Vec<u8>>,
        feather: f32,
    ) {
        self.paint.selection_mask = mask;
        self.paint.selection_active = active;
        self.paint.selection_crisp = crisp;
        self.paint.selection_feather = feather;
        // The on-canvas overlay (marching ants / hatching, Wave 4) is derived from this, so refresh.
        self.invalidate_composite();
    }

    /// Whether an active selection should RESTRICT painting to its region right now. False when there is
    /// no selection (paint everywhere) or the buffer is unsized.
    pub(super) fn selection_restricts_paint(&self) -> bool {
        self.paint.selection_active && !self.paint.selection_mask.is_empty()
    }

    /// Restore the DESELECTED texels of `region` from `before` after a stamp: blend the pre-stamp pixel
    /// back by `1 - coverage`, so a fully-selected texel (coverage = 1) keeps the fresh paint and an
    /// unselected one (coverage = 0) reverts entirely. Mirror of [`Self::restore_protected_region`], but
    /// keyed on the selection coverage instead of the protection mask.
    pub(super) fn restore_deselected_region(&mut self, region: Region, before: &[u8]) {
        let (w, _h) = self.source_size;
        let mask = Arc::clone(&self.paint.selection_mask);
        let buf = Arc::make_mut(&mut self.canvas_rgba);
        let n = mask.len().min(buf.len() / 4);
        for ry in 0..region.h {
            for rx in 0..region.w {
                let gidx = ((region.y + ry) * w + (region.x + rx)) as usize;
                if gidx >= n {
                    continue;
                }
                let keep = f32::from(mask[gidx]) / 255.0; // 1 = inside (keep paint), 0 = outside (revert)
                if keep >= 1.0 {
                    continue; // fully selected → keep the fresh paint untouched
                }
                let b = gidx * 4;
                let s = ((ry * region.w + rx) * 4) as usize;
                for c in 0..4 {
                    let painted = f32::from(buf[b + c]);
                    let orig = f32::from(before[s + c]);
                    buf[b + c] = (painted * keep + orig * (1.0 - keep))
                        .round()
                        .clamp(0.0, 255.0) as u8;
                }
            }
        }
    }

    /// Ensure the selection buffer is sized to the current canvas (`w*h`, zero-filled = nothing selected).
    /// Re-allocates only when the size changed.
    pub(super) fn ensure_selection_mask(&mut self) {
        let (w, h) = self.source_size;
        if w == 0 || h == 0 {
            return;
        }
        let need = (w as usize) * (h as usize);
        if self.paint.selection_mask.len() != need {
            self.paint.selection_mask = Arc::new(vec![0u8; need]);
        }
    }

    /// **Wave 1 seed** — replace the selection with a filled rectangle in image px (fully inside = 255).
    /// Records ONE structural undo entry so it joins the single interleaved queue exactly like a brush
    /// stroke. Wave 2 layers the real modes + Add/Remove/Invert operators on top of this primitive.
    pub fn set_rect_selection(&mut self, x: u32, y: u32, rw: u32, rh: u32) {
        let (w, h) = self.source_size;
        if w == 0 || h == 0 {
            return;
        }
        let before = self.snapshot_model();
        let mut crisp = vec![0u8; (w as usize) * (h as usize)];
        let x1 = (x + rw).min(w);
        let y1 = (y + rh).min(h);
        for yy in y.min(h)..y1 {
            for xx in x.min(w)..x1 {
                crisp[(yy * w + xx) as usize] = 255;
            }
        }
        self.set_selection_from_crisp(crisp);
        self.commit_structural_edit(before);
    }

    /// **Clear** (deselect) — no active selection, painting unrestricted again. Records one structural undo
    /// entry (no-op when there is nothing selected).
    pub fn clear_selection(&mut self) {
        if !self.paint.selection_active {
            return;
        }
        let before = self.snapshot_model();
        self.paint.selection_active = false;
        self.paint.selection_mask = Arc::new(Vec::new());
        self.paint.selection_crisp = Arc::new(Vec::new());
        self.invalidate_composite();
        self.commit_structural_edit(before);
    }
}

// ── Wave 2: on-canvas creation modes (Automatic / Freehand / Rectangle / Ellipse) + boolean ops ──────────
impl PainterTool {
    /// Set the Selection sub-mode (`0` Automatic · `1` Freehand · `2` Rectangle · `3` Ellipse).
    pub fn set_selection_mode(&mut self, m: u8) {
        self.paint.selection_mode = m.min(3);
    }
    /// The active Selection sub-mode discriminant.
    #[must_use]
    pub fn selection_mode(&self) -> u8 {
        self.paint.selection_mode
    }
    /// Set the boolean operator for the next gesture (`0` New · `1` Add · `2` Remove).
    pub fn set_selection_bool_op(&mut self, op: u8) {
        self.paint.selection_bool_op = op.min(2);
    }
    /// The active boolean operator discriminant.
    #[must_use]
    pub fn selection_bool_op(&self) -> u8 {
        self.paint.selection_bool_op
    }
    /// Set the Automatic-mode threshold (`0..1`); re-floods live if an Automatic gesture is open.
    pub fn set_selection_threshold(&mut self, t: f32) {
        self.paint.selection_threshold = t.clamp(0.0, 1.0);
        let auto_seed = match &self.paint.selection_drag {
            Some(SelectionDrag::Auto { seed, .. }) => Some(*seed),
            _ => None,
        };
        if let Some(seed) = auto_seed {
            self.auto_reflood(seed);
        }
    }
    /// The Automatic threshold (`0..1`).
    #[must_use]
    pub fn selection_threshold(&self) -> f32 {
        self.paint.selection_threshold
    }

    /// **Invert** the whole selection (`255 − coverage`), sizing the mask to the canvas first. One undo entry.
    pub fn invert_selection(&mut self) {
        let (w, h) = self.source_size;
        if w == 0 || h == 0 {
            return;
        }
        let n = (w as usize) * (h as usize);
        let before = self.snapshot_model();
        let mut crisp = if self.paint.selection_crisp.len() == n {
            (*self.paint.selection_crisp).clone()
        } else {
            vec![0u8; n]
        };
        for v in crisp.iter_mut() {
            *v = 255 - *v;
        }
        self.set_selection_from_crisp(crisp);
        self.commit_structural_edit(before);
    }

    /// Route a Selection-mode canvas pointer to the active sub-mode (called from `on_canvas_pointer`).
    pub(super) fn selection_pointer(&mut self, ev: CanvasPointer) -> bool {
        match ev.phase {
            PointerPhase::Down => self.selection_down(ev.pos),
            PointerPhase::Move => self.selection_move(ev.pos),
            PointerPhase::Up => self.selection_up(ev.pos),
            PointerPhase::Hover => false,
        }
    }

    /// Begin a selection gesture: snapshot for undo, capture the base selection this gesture combines
    /// against, and start the sub-mode's drag. Automatic floods immediately.
    fn selection_down(&mut self, pos: [f32; 2]) -> bool {
        self.paint.stroke_undo = Some(self.snapshot_model());
        self.ensure_selection_mask();
        // Add/Remove combine against the CRISP base (not the Feathered mask), so feathered edges never
        // re-seed the accumulator.
        self.paint.selection_base = Arc::clone(&self.paint.selection_crisp);
        match self.paint.selection_mode {
            0 => {
                let thresh0 = self.paint.selection_threshold;
                self.paint.selection_drag = Some(SelectionDrag::Auto { seed: pos, thresh0 });
                self.auto_reflood(pos);
            }
            1 => {
                self.paint.selection_drag = Some(SelectionDrag::Lasso { points: vec![pos] });
            }
            3 => {
                self.paint.selection_drag = Some(SelectionDrag::Marquee {
                    anchor: pos,
                    cur: pos,
                    ellipse: true,
                });
            }
            _ => {
                self.paint.selection_drag = Some(SelectionDrag::Marquee {
                    anchor: pos,
                    cur: pos,
                    ellipse: false,
                });
            }
        }
        true
    }

    /// Extend the gesture: marquee tracks the corner, lasso appends a point, Automatic drags the threshold
    /// (horizontal delta from the seed, Procreate-style). Only geometry is updated per Move (cheap); the
    /// marquee/lasso mask is rasterized once on Up.
    fn selection_move(&mut self, pos: [f32; 2]) -> bool {
        // Extract the Copy data (and drop the borrow) before touching other `self` fields / methods.
        enum Move {
            Marquee([f32; 2], bool),
            Lasso,
            Auto([f32; 2], f32),
        }
        let kind = match &self.paint.selection_drag {
            Some(SelectionDrag::Marquee {
                anchor, ellipse, ..
            }) => Move::Marquee(*anchor, *ellipse),
            Some(SelectionDrag::Lasso { .. }) => Move::Lasso,
            Some(SelectionDrag::Auto { seed, thresh0 }) => Move::Auto(*seed, *thresh0),
            None => return false,
        };
        match kind {
            Move::Marquee(anchor, ellipse) => {
                if let Some(SelectionDrag::Marquee { cur, .. }) = &mut self.paint.selection_drag {
                    *cur = pos;
                }
                // Live preview: rasterize + combine the pending marquee against the gesture base so the
                // overlay tracks the drag in real time (committed to undo only on pen-up).
                let region = self.raster_marquee(anchor, pos, ellipse);
                self.apply_selection_region(&region);
                self.invalidate_composite();
            }
            Move::Lasso => {
                if let Some(SelectionDrag::Lasso { points }) = &mut self.paint.selection_drag {
                    points.push(pos);
                }
                // Live preview: rasterize the in-progress path (auto-closed) each move.
                let pts = match &self.paint.selection_drag {
                    Some(SelectionDrag::Lasso { points }) => points.clone(),
                    _ => Vec::new(),
                };
                let region = self.raster_lasso(&pts);
                self.apply_selection_region(&region);
                self.invalidate_composite();
            }
            Move::Auto(seed, thresh0) => {
                let w = self.source_size.0;
                if w > 0 {
                    let delta = (pos[0] - seed[0]) / w as f32;
                    self.paint.selection_threshold = (thresh0 + delta).clamp(0.0, 1.0);
                }
                self.auto_reflood(seed);
            }
        }
        true
    }

    /// Finish the gesture: rasterize the region, combine into the mask, and commit ONE structural undo
    /// entry so the selection joins the single interleaved queue.
    fn selection_up(&mut self, pos: [f32; 2]) -> bool {
        let region = match self.paint.selection_drag.take() {
            Some(SelectionDrag::Marquee { anchor, ellipse, .. }) => {
                self.raster_marquee(anchor, pos, ellipse)
            }
            Some(SelectionDrag::Lasso { mut points }) => {
                points.push(pos);
                self.raster_lasso(&points)
            }
            Some(SelectionDrag::Auto { seed, .. }) => self.raster_flood(seed),
            None => return false,
        };
        self.apply_selection_region(&region);
        if let Some(before) = self.paint.stroke_undo.take() {
            self.commit_structural_edit(before);
        }
        self.paint.selection_base = Arc::new(Vec::new());
        self.invalidate_composite();
        true
    }

    /// Re-flood the Automatic selection from `seed` at the current threshold, writing the combined mask live.
    fn auto_reflood(&mut self, seed: [f32; 2]) {
        let region = self.raster_flood(seed);
        self.apply_selection_region(&region);
        self.invalidate_composite();
    }

    /// Rasterize a rectangle / ellipse marquee (image px) into a canvas-sized coverage buffer (0 / 255).
    fn raster_marquee(&self, a: [f32; 2], b: [f32; 2], ellipse: bool) -> Vec<u8> {
        let (w, h) = (self.source_size.0 as usize, self.source_size.1 as usize);
        let mut cov = vec![0u8; w * h];
        if w == 0 || h == 0 {
            return cov;
        }
        let x0 = a[0].min(b[0]).floor().clamp(0.0, w as f32) as usize;
        let x1 = a[0].max(b[0]).ceil().clamp(0.0, w as f32) as usize;
        let y0 = a[1].min(b[1]).floor().clamp(0.0, h as f32) as usize;
        let y1 = a[1].max(b[1]).ceil().clamp(0.0, h as f32) as usize;
        if ellipse {
            let cx = (x0 as f32 + x1 as f32) * 0.5;
            let cy = (y0 as f32 + y1 as f32) * 0.5;
            let rx = ((x1.saturating_sub(x0)) as f32 * 0.5).max(0.5);
            let ry = ((y1.saturating_sub(y0)) as f32 * 0.5).max(0.5);
            for yy in y0..y1 {
                for xx in x0..x1 {
                    let dx = (xx as f32 + 0.5 - cx) / rx;
                    let dy = (yy as f32 + 0.5 - cy) / ry;
                    if dx * dx + dy * dy <= 1.0 {
                        cov[yy * w + xx] = 255;
                    }
                }
            }
        } else {
            for yy in y0..y1 {
                for xx in x0..x1 {
                    cov[yy * w + xx] = 255;
                }
            }
        }
        cov
    }

    /// Rasterize a closed lasso polygon (image px) into a canvas-sized coverage buffer via an even-odd
    /// scanline fill. Needs ≥3 points.
    fn raster_lasso(&self, pts: &[[f32; 2]]) -> Vec<u8> {
        let (w, h) = (self.source_size.0 as usize, self.source_size.1 as usize);
        let mut cov = vec![0u8; w * h];
        if w == 0 || h == 0 || pts.len() < 3 {
            return cov;
        }
        for yy in 0..h {
            let yc = yy as f32 + 0.5;
            let mut xs: Vec<f32> = Vec::new();
            for i in 0..pts.len() {
                let p = pts[i];
                let q = pts[(i + 1) % pts.len()];
                let (py, qy) = (p[1], q[1]);
                if (py <= yc && yc < qy) || (qy <= yc && yc < py) {
                    let t = (yc - py) / (qy - py);
                    xs.push(p[0] + t * (q[0] - p[0]));
                }
            }
            xs.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
            let mut i = 0;
            while i + 1 < xs.len() {
                let xa = xs[i].max(0.0).round() as usize;
                let xb = (xs[i + 1].min(w as f32).round() as usize).min(w);
                for xx in xa..xb {
                    cov[yy * w + xx] = 255;
                }
                i += 2;
            }
        }
        cov
    }

    /// Rasterize an Automatic flood-select from `seed` at the current threshold into a coverage buffer.
    fn raster_flood(&self, seed: [f32; 2]) -> Vec<u8> {
        let (w, h) = (self.source_size.0 as usize, self.source_size.1 as usize);
        let mut cov = vec![0u8; w * h];
        if w == 0 || h == 0 || self.canvas_rgba.len() != w * h * 4 {
            return cov;
        }
        let sx = (seed[0].floor() as i32).clamp(0, w as i32 - 1) as usize;
        let sy = (seed[1].floor() as i32).clamp(0, h as i32 - 1) as usize;
        let tol = (self.paint.selection_threshold.clamp(0.0, 1.0) * 255.0).round() as u8;
        flood_coverage(&self.canvas_rgba, w, h, (sx, sy), tol, &mut cov);
        cov
    }

    /// Combine `region` (canvas-sized coverage) into the selection mask using the current boolean op,
    /// against the gesture's base selection. Updates `selection_active`.
    fn apply_selection_region(&mut self, region: &[u8]) {
        let n = (self.source_size.0 as usize) * (self.source_size.1 as usize);
        if n == 0 || region.len() != n {
            return;
        }
        let base = if self.paint.selection_base.len() == n {
            Arc::clone(&self.paint.selection_base)
        } else {
            Arc::new(vec![0u8; n])
        };
        let mut out = vec![0u8; n];
        match self.paint.selection_bool_op {
            // Add: union (max).
            1 => {
                for i in 0..n {
                    out[i] = base[i].max(region[i]);
                }
            }
            // Remove: base ∧ ¬region.
            2 => {
                for i in 0..n {
                    out[i] = ((u16::from(base[i]) * u16::from(255 - region[i])) / 255) as u8;
                }
            }
            // New: replace.
            _ => out.copy_from_slice(region),
        }
        self.set_selection_from_crisp(out);
    }

    /// Install a CRISP selection accumulator and derive the effective (Feathered) `selection_mask` from it.
    /// The single funnel for every op that changes WHICH texels are selected (marquee / lasso / flood /
    /// rect seed / invert) — so Feather always re-derives from the crisp base, never compounding.
    fn set_selection_from_crisp(&mut self, crisp: Vec<u8>) {
        self.paint.selection_active = crisp.iter().any(|&v| v > 0);
        self.paint.selection_crisp = Arc::new(crisp);
        self.derive_effective();
    }

    /// Recompute `selection_mask` = Feather(`selection_crisp`). Feather is a separable box blur (HR-5
    /// transcendental-free) whose radius scales with the Feather amount; `0` copies the crisp mask.
    fn derive_effective(&mut self) {
        let crisp = Arc::clone(&self.paint.selection_crisp);
        let (w, h) = (self.source_size.0 as usize, self.source_size.1 as usize);
        let radius = (self.paint.selection_feather.clamp(0.0, 1.0) * FEATHER_MAX_PX as f32).round() as usize;
        if radius == 0 || w == 0 || h == 0 || crisp.len() != w * h {
            self.paint.selection_mask = crisp;
        } else {
            // Three box-blur passes ≈ a Gaussian (central-limit): a smooth, regular feather ramp instead
            // of the boxy single-pass profile.
            let mut m = box_blur(&crisp, w, h, radius);
            m = box_blur(&m, w, h, radius);
            m = box_blur(&m, w, h, radius);
            self.paint.selection_mask = Arc::new(m);
        }
        self.invalidate_composite();
    }

    /// Set the **Feather** amount (`0..1`) and re-derive the effective mask from the crisp accumulator.
    pub fn set_selection_feather(&mut self, f: f32) {
        self.paint.selection_feather = f.clamp(0.0, 1.0);
        self.derive_effective();
    }
    /// The Feather amount (`0..1`).
    #[must_use]
    pub fn selection_feather(&self) -> f32 {
        self.paint.selection_feather
    }

    /// Enter / leave **Edit Selection** mode (the boundary shown as an editable Shape — Wave EDIT).
    pub fn set_selection_edit_mode(&mut self, on: bool) {
        self.paint.selection_edit_mode = on;
        self.invalidate_composite();
    }
    /// `true` while the selection boundary is in editable-Shape mode.
    #[must_use]
    pub fn selection_edit_mode(&self) -> bool {
        self.paint.selection_edit_mode
    }

    /// Set the selection-overlay hatch opacity (`0..1`, a view preference).
    pub fn set_selection_overlay_opacity(&mut self, o: f32) {
        self.paint.selection_overlay_opacity = o.clamp(0.0, 1.0);
        self.invalidate_composite();
    }
    /// The selection-overlay hatch opacity (`0..1`).
    #[must_use]
    pub fn selection_overlay_opacity(&self) -> f32 {
        self.paint.selection_overlay_opacity
    }

    /// Clip `canvas_rgba` to the active selection: revert each texel toward `orig` by `1 - coverage`, so an
    /// edit that wrote outside the selection is undone (feathered edges blend). No-op without a selection.
    /// Used by paths that bypass the per-dab stamp gate — the Fill flood and (via the scratch) the Mask.
    pub(super) fn clip_canvas_to_selection(&mut self, orig: &[u8]) {
        if !self.selection_restricts_paint() {
            return;
        }
        let mask = Arc::clone(&self.paint.selection_mask);
        let buf = Arc::make_mut(&mut self.canvas_rgba);
        let n = mask.len().min(buf.len() / 4).min(orig.len() / 4);
        for i in 0..n {
            let keep = f32::from(mask[i]) / 255.0;
            if keep >= 1.0 {
                continue;
            }
            let b = i * 4;
            for c in 0..4 {
                let painted = f32::from(buf[b + c]);
                let o = f32::from(orig[b + c]);
                buf[b + c] = (painted * keep + o * (1.0 - keep)).round().clamp(0.0, 255.0) as u8;
            }
        }
    }

    /// Build the on-canvas selection **overlay** (Procreate-style): semi-transparent diagonal **hatching**
    /// over the DESELECTED area + animated **marching ants** on the boundary. Canvas-sized straight RGBA
    /// the shell blits image→screen. `phase` (a shell frame counter) animates the ants (fast) and the
    /// hatch drift (slow). `None` when no selection is live. Interior (selected, non-edge) stays clear.
    #[must_use]
    pub fn selection_overlay_rgba(&self, phase: u32) -> Option<(Vec<u8>, u32, u32)> {
        if !self.paint.selection_active {
            return None;
        }
        let (w, h) = self.source_size;
        let (wu, hu) = (w as usize, h as usize);
        let mask = &self.paint.selection_mask;
        if wu == 0 || hu == 0 || mask.len() != wu * hu {
            return None;
        }
        const STRIPE: usize = 7; // hatch stripe width (image px)
        const DASH: usize = 8; // marching-ants dash period (image px)
        const HATCH_MAX_ALPHA: f32 = 110.0; // full-strength hatch alpha (scaled by the opacity setting)
        let hatch_shift = (phase / 2) as usize; // hatch drifts at half the ant speed
        let ant = phase as usize;
        let opacity = self.paint.selection_overlay_opacity.clamp(0.0, 1.0);
        let mut out = vec![0u8; wu * hu * 4];
        for y in 0..hu {
            for x in 0..wu {
                let idx = y * wu + x;
                let cov = f32::from(mask[idx]) / 255.0; // 1 = fully selected, 0 = fully outside
                let inside = mask[idx] >= 128;
                // Boundary = the 128 (half-coverage) contour flips across the right or down neighbour.
                let edge = (x + 1 < wu && (mask[idx + 1] >= 128) != inside)
                    || (y + 1 < hu && (mask[idx + wu] >= 128) != inside);
                let o = idx * 4;
                if edge {
                    // Alternating white / black dashes marching along x+y with the phase.
                    let c = if (x + y + ant) % DASH < DASH / 2 {
                        [255u8, 255, 255, 255]
                    } else {
                        [0u8, 0, 0, 255]
                    };
                    out[o..o + 4].copy_from_slice(&c);
                } else if ((x + y + hatch_shift) / STRIPE) % 2 == 0 {
                    // Diagonal hatch whose opacity FADES with the selection coverage — a realistic gradient
                    // across a feathered edge (full outside, half at the 50% contour, clear inside).
                    let a = (HATCH_MAX_ALPHA * opacity * (1.0 - cov)).round() as u8;
                    if a > 0 {
                        out[o..o + 4].copy_from_slice(&[30u8, 30, 30, a]);
                    }
                }
            }
        }
        Some((out, w, h))
    }

    /// Route a Selection-panel event to the tool — the mode / boolean-op segments (Click), the Feather /
    /// Automatic-threshold sliders (SetValue), the Edit-Selection toggle, and the Invert / Clear actions.
    /// Mirrors the other `route_*_event` early-dispatch helpers in `handle_panel_event`. `true` when handled.
    pub(crate) fn route_selection_event(&mut self, event: &ph2d_editor_core::tool::PanelEvent) -> bool {
        use ph2d_editor_core::ids as core_ids;
        use ph2d_editor_core::tool::PanelEvent;
        match event {
            PanelEvent::Click(id) if core_ids::PAINTER_SEL_MODE_IDS.contains(id) => {
                let idx = core_ids::PAINTER_SEL_MODE_IDS
                    .iter()
                    .position(|x| x == id)
                    .unwrap_or(0) as u8;
                self.set_selection_mode(idx);
                true
            }
            PanelEvent::Click(id) if core_ids::PAINTER_SEL_OP_IDS.contains(id) => {
                let idx = core_ids::PAINTER_SEL_OP_IDS
                    .iter()
                    .position(|x| x == id)
                    .unwrap_or(0) as u8;
                self.set_selection_bool_op(idx);
                true
            }
            PanelEvent::Click(id) if *id == core_ids::PAINTER_SEL_EDIT => {
                self.toggle_selection_edit();
                true
            }
            PanelEvent::Click(id) if *id == core_ids::PAINTER_SEL_INVERT => {
                self.invert_selection();
                true
            }
            PanelEvent::Click(id) if *id == core_ids::PAINTER_SEL_CLEAR => {
                self.clear_selection();
                true
            }
            PanelEvent::SetValue(id, v) if *id == core_ids::PAINTER_SEL_FEATHER_SLIDER => {
                self.set_selection_feather(*v as f32);
                true
            }
            PanelEvent::SetValue(id, v) if *id == core_ids::PAINTER_SEL_THRESHOLD_SLIDER => {
                self.set_selection_threshold(*v as f32);
                true
            }
            PanelEvent::SetValue(id, v) if *id == core_ids::PAINTER_SEL_OPACITY_SLIDER => {
                self.set_selection_overlay_opacity(*v as f32);
                true
            }
            _ => false,
        }
    }

    // ── Edit mode (ADR-0103 Am.1): the boundary becomes an editable Curve, reusing the Shape editors ──

    /// Enter Selection **Edit** mode: trace the mask contour into an editable closed Curve (Vector anchors,
    /// so it hugs the traced polygon) and install it as the active Curve editor — its handles / gizmos /
    /// Offset now reshape the selection. No-op without a live, traceable selection.
    pub(super) fn enter_selection_edit(&mut self) {
        if !self.paint.selection_active {
            return;
        }
        let (w, h) = (self.source_size.0 as usize, self.source_size.1 as usize);
        let pts = super::selection_trace::trace_selection_contour(&self.paint.selection_mask, w, h);
        if pts.len() < 3 {
            return;
        }
        self.paint.selection_edit_saved_method = Some(self.paint.brush.stroke_method);
        let anchor = pts[0];
        let handles: Vec<[[f32; 2]; 2]> = pts.iter().map(|p| [*p, *p]).collect();
        let kinds = vec![2u8; pts.len()]; // 2 = Vector → straight segments between anchors
        let seed = self.paint.seed;
        self.paint.seed = self.paint.seed.wrapping_add(1);
        let state = crate::undo::ShapeEditState::Curve(crate::undo::CurveState {
            points: pts,
            handles,
            kinds,
            selected: None,
            added_point: false,
            closed: true,
            editing: true,
            freehand: true,
            seed,
            anchor,
            stabilized: anchor,
        });
        self.restore_shape_overlay(Some(Box::new(state)), None);
        self.paint.selection_edit_mode = true;
        self.invalidate_composite();
    }

    /// Leave Selection Edit mode: discard the Curve overlay (no pixel bake — the mask already holds the
    /// edited region) and restore the pre-edit stroke method.
    pub(super) fn exit_selection_edit(&mut self) {
        if !self.paint.selection_edit_mode {
            return;
        }
        self.paint.selection_edit_mode = false;
        self.discard_open_shape();
        if let Some(m) = self.paint.selection_edit_saved_method.take() {
            self.paint.brush.stroke_method = m;
        }
        self.invalidate_composite();
    }

    /// Toggle Edit mode (the panel's **Edit Selection** button).
    pub fn toggle_selection_edit(&mut self) {
        if self.paint.selection_edit_mode {
            self.exit_selection_edit();
        } else {
            self.enter_selection_edit();
        }
    }

    /// Re-derive the selection mask from the live Curve editor — called after every edit in Edit mode:
    /// apply the Offset (grow/shrink), flatten the closed curve to a polyline, fill it, and install it as
    /// the selection. The Shape-editor undo txn snapshots the whole model (incl. the mask), so boundary
    /// edits ride the single interleaved queue.
    pub(super) fn selection_refill_from_curve(&mut self) {
        let Some(ed) = self.paint.curve.as_ref() else {
            return;
        };
        let pts = ed.points.clone();
        let handles = ed.handles.clone();
        let closed = ed.closed;
        let (pts, handles, _) =
            super::curve_offset::offset_curve_refined(&pts, &handles, self.shape_offset_px(), closed);
        let mut spine = Vec::new();
        super::curve_geom::flatten_spine(&pts, &handles, closed, &mut spine);
        if spine.len() < 3 {
            return;
        }
        let cov = self.raster_lasso(&spine);
        self.set_selection_from_crisp(cov);
    }
}

/// Separable box blur of a single-channel `w*h` coverage buffer (Feather). Two 1-D averaging passes (H
/// then V) with a `(2r+1)` window, clamping samples to the border — transcendental-free (HR-5).
fn box_blur(src: &[u8], w: usize, h: usize, r: usize) -> Vec<u8> {
    if r == 0 || w == 0 || h == 0 || src.len() != w * h {
        return src.to_vec();
    }
    let win = (2 * r + 1) as u32;
    // Horizontal pass into `tmp`.
    let mut tmp = vec![0u8; w * h];
    for y in 0..h {
        let row = y * w;
        for x in 0..w {
            let mut acc = 0u32;
            for k in 0..(2 * r + 1) {
                let xx = (x + k).saturating_sub(r).min(w - 1);
                acc += u32::from(src[row + xx]);
            }
            tmp[row + x] = (acc / win) as u8;
        }
    }
    // Vertical pass into `out`.
    let mut out = vec![0u8; w * h];
    for x in 0..w {
        for y in 0..h {
            let mut acc = 0u32;
            for k in 0..(2 * r + 1) {
                let yy = (y + k).saturating_sub(r).min(h - 1);
                acc += u32::from(tmp[yy * w + x]);
            }
            out[y * w + x] = (acc / win) as u8;
        }
    }
    out
}

/// Scanline flood — like [`super::fill::flood_fill`] but WRITES a coverage buffer (255 in-region) instead
/// of painting, reading `px` (RGBA8) read-only. Matches the 4-connected region within `tol` of the seed.
fn flood_coverage(px: &[u8], w: usize, h: usize, seed: (usize, usize), tol: u8, cov: &mut [u8]) {
    if w == 0 || h == 0 || px.len() != w * h * 4 || cov.len() != w * h {
        return;
    }
    let (sx, sy) = seed;
    if sx >= w || sy >= h {
        return;
    }
    let si = (sy * w + sx) * 4;
    let seed_c = [px[si], px[si + 1], px[si + 2], px[si + 3]];
    let matches = |idx: usize| -> bool {
        let o = idx * 4;
        px[o]
            .abs_diff(seed_c[0])
            .max(px[o + 1].abs_diff(seed_c[1]))
            .max(px[o + 2].abs_diff(seed_c[2]))
            .max(px[o + 3].abs_diff(seed_c[3]))
            <= tol
    };
    let mut visited = vec![false; w * h];
    let mut stack: Vec<(usize, usize)> = vec![seed];
    while let Some((x, y)) = stack.pop() {
        if visited[y * w + x] {
            continue;
        }
        let mut lx = x;
        while lx > 0 && !visited[y * w + lx - 1] && matches(y * w + lx - 1) {
            lx -= 1;
        }
        let mut rx = x;
        while rx + 1 < w && !visited[y * w + rx + 1] && matches(y * w + rx + 1) {
            rx += 1;
        }
        for xx in lx..=rx {
            let idx = y * w + xx;
            visited[idx] = true;
            cov[idx] = 255;
        }
        for ny in [y.checked_sub(1), (y + 1 < h).then_some(y + 1)]
            .into_iter()
            .flatten()
        {
            for xx in lx..=rx {
                if !visited[ny * w + xx] && matches(ny * w + xx) {
                    stack.push((xx, ny));
                }
            }
        }
    }
}
