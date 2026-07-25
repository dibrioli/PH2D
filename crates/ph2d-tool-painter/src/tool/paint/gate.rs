//! The protection/selection GATE as an **epoch projection** — the ceiling that never erodes.
//!
//! The old gate lerped the stamped result against a per-BATCH snapshot
//! (`new = painted·keep + before·(1−keep)`), which makes the protection a per-pass MULTIPLIER: over
//! N batches a K-protected texel retains only `(1−keep)^N` of its original — every partially
//! protected texel eventually saturates to full paint and the soft feather hardens into an aliased
//! cliff at the full-protection contour (Enio, 2026-07-25; measured: keep=9/255 went 218→25 over
//! 15 passes). The industry splits here: *selections* in Photoshop/Krita compound exactly like this
//! (in PS it is even a technique), but everything whose promise is *protection* — layer masks,
//! alpha lock — is a **ceiling**: the shown result never exceeds the coverage, no matter how many
//! strokes. Our mask IS a protection ("these pixels are X% frozen"), and the Selection feather
//! shows the identical hardening artifact, so BOTH gates take the ceiling here — a deliberate,
//! documented divergence from PS selection semantics (doc 25 §13.7).
//!
//! **The model.** While the protection/selection declaration stands, an *epoch* is live, holding
//! two planes: `gate_ref_rgba` — the canvas when the epoch began — and `gate_free_rgba` — what
//! UNRESTRICTED painting would have produced (every gated batch stamps into it). The canvas the
//! artist sees is always the projection
//!
//! ```text
//! canvas = ref·(1−keep) + free·keep        keep = mask_keep × selection_keep
//! ```
//!
//! so the feather is a faithful `keep`-blend of the unrestricted painting FOREVER: N passes
//! converge (the free plane saturates; the blend stops at the ceiling), density still builds in
//! the feather proportionally as the artist deepens the interior, and the feather profile never
//! sharpens. Where `keep == 0` the free plane is re-pinned to `ref` — a fully frozen texel
//! accumulates no hidden paint, so what a Smear later drags out of a frozen zone is what the
//! artist sees there (not a buried stroke history).
//!
//! **Epoch lifecycle.** Seeded lazily by the first gated batch (`ensure_gate_epoch`, two
//! `Arc::clone`s — O(1)). COMMITTED (planes dropped; the display is already the truth) by
//! [`PainterTool::commit_gate_epoch`] whenever the projection's inputs stop describing the world:
//! any edit to the protection scratch or the selection (the keep source — otherwise raising keep
//! would retroactively REVEAL buried stroke history), any foreign canvas writer (fill / inpaint /
//! Deform / selection ops / watercolor / wet — otherwise the next projection would silently revert
//! their edit inside the region), a layer switch, and teardown. Undo carries both planes inside
//! the `ModelSnapshot` (same-commit law), so undo/redo restores the ceiling mid-epoch instead of
//! re-seeding against a canvas that already ate one epoch's worth of feather paint.

use super::{PainterTool, Region};
use std::sync::Arc;

impl PainterTool {
    /// `true` while a gate epoch is live (the planes are canvas-sized).
    pub(super) fn gate_epoch_live(&self) -> bool {
        !self.paint.gate_ref_rgba.is_empty()
    }

    /// Seed the epoch from the current canvas if none is live (or the canvas was resized). Two
    /// `Arc::clone`s — the copies materialise lazily via CoW on first write.
    pub(super) fn ensure_gate_epoch(&mut self) {
        let n = self.canvas_rgba.len();
        if n == 0 {
            return;
        }
        if self.paint.gate_ref_rgba.len() != n || self.paint.gate_free_rgba.len() != n {
            self.paint.gate_ref_rgba = Arc::clone(&self.canvas_rgba);
            self.paint.gate_free_rgba = Arc::clone(&self.canvas_rgba);
        }
    }

    /// End the epoch: drop both planes. The canvas already shows the projected truth, so nothing
    /// is written — the next gated batch simply seeds a fresh epoch from what is on screen. Called
    /// by every keep-source edit and every foreign canvas writer; cheap no-op when no epoch is live.
    pub(crate) fn commit_gate_epoch(&mut self) {
        if !self.paint.gate_ref_rgba.is_empty() {
            self.paint.gate_ref_rgba = Arc::new(Vec::new());
        }
        if !self.paint.gate_free_rgba.is_empty() {
            self.paint.gate_free_rgba = Arc::new(Vec::new());
        }
    }

    /// Copy `rect`'s RGBA out of the epoch's FREE plane — the drag-preview's free twin (`None` when
    /// no epoch is live, so an ungated preview costs nothing).
    pub(super) fn save_free_region(&self, rect: &Region) -> Option<Vec<u8>> {
        if !self.gate_epoch_live() {
            return None;
        }
        let stride = self.source_size.0 as usize * 4;
        let rw = rect.w as usize * 4;
        let free = &self.paint.gate_free_rgba;
        let mut out = Vec::with_capacity(rw * rect.h as usize);
        for row in 0..rect.h {
            let start = (rect.y + row) as usize * stride + rect.x as usize * 4;
            if start + rw <= free.len() {
                out.extend_from_slice(&free[start..start + rw]);
            }
        }
        Some(out)
    }

    /// Write a [`Self::save_free_region`] capture back into the FREE plane (the preview peel's free
    /// twin). No dirty marking — the free plane is never displayed. No-op when the capture is `None`
    /// (no epoch at save time) or the epoch has since been committed.
    pub(super) fn restore_free_region(&mut self, rect: &Region, pixels: Option<&[u8]>) {
        let Some(pixels) = pixels else { return };
        if !self.gate_epoch_live() {
            return;
        }
        let stride = self.source_size.0 as usize * 4;
        let rw = rect.w as usize * 4;
        let free = Arc::make_mut(&mut self.paint.gate_free_rgba);
        for row in 0..rect.h {
            let dst = (rect.y + row) as usize * stride + rect.x as usize * 4;
            let src = row as usize * rw;
            if dst + rw <= free.len() && src + rw <= pixels.len() {
                free[dst..dst + rw].copy_from_slice(&pixels[src..src + rw]);
            }
        }
    }

    /// Project the epoch over `region`: `canvas = ref·(1−keep) + free·keep`, with
    /// `keep = mask_keep × selection_keep` (each factor 1 when its gate is inactive). `keep == 1`
    /// copies `free` bytes verbatim (so an ungated texel is byte-identical to the unrestricted
    /// stamp); `keep == 0` copies `ref` AND re-pins `free` to `ref` (a frozen texel accumulates no
    /// hidden paint). The blend arithmetic is the old restore's, term for term
    /// (`a·k + b·(1−k)`, `.round().clamp(0,255)`), so a single-gate, first-batch stroke is
    /// byte-identical to the retired per-batch door.
    pub(super) fn project_gated_region(&mut self, region: Region) {
        let (w, _h) = self.source_size;
        let mask_on = self.mask_protection_active();
        let sel_on = self.selection_restricts_paint();
        if (!mask_on && !sel_on) || !self.gate_epoch_live() {
            return;
        }
        let scratch = Arc::clone(&self.paint.mask_scratch_rgba);
        let sel = Arc::clone(&self.paint.selection_mask);
        let ref_p = Arc::clone(&self.paint.gate_ref_rgba);
        let mut n = self.canvas_rgba.len() / 4;
        n = n
            .min(ref_p.len() / 4)
            .min(self.paint.gate_free_rgba.len() / 4);
        if mask_on {
            n = n.min(scratch.len() / 4);
        }
        if sel_on {
            n = n.min(sel.len());
        }
        let free = Arc::make_mut(&mut self.paint.gate_free_rgba);
        let canvas = Arc::make_mut(&mut self.canvas_rgba);
        for ry in 0..region.h {
            for rx in 0..region.w {
                let gidx = ((region.y + ry) * w + (region.x + rx)) as usize;
                if gidx >= n {
                    continue;
                }
                let mut keep = 1.0f32;
                if mask_on {
                    keep *= crate::compositor::mask_value(&scratch, gidx);
                }
                if sel_on {
                    keep *= f32::from(sel[gidx]) / 255.0;
                }
                let b = gidx * 4;
                if keep >= 1.0 {
                    canvas[b..b + 4].copy_from_slice(&free[b..b + 4]);
                } else if keep <= 0.0 {
                    canvas[b..b + 4].copy_from_slice(&ref_p[b..b + 4]);
                    free[b..b + 4].copy_from_slice(&ref_p[b..b + 4]);
                } else {
                    for c in 0..4 {
                        let f = f32::from(free[b + c]);
                        let r = f32::from(ref_p[b + c]);
                        canvas[b + c] =
                            (f * keep + r * (1.0 - keep)).round().clamp(0.0, 255.0) as u8;
                    }
                }
            }
        }
    }
}

impl PainterTool {
    /// The epoch planes for the undo model (`ModelSnapshot`) — `Arc`-shared, cheap. Same-commit law:
    /// they are captured/restored together with the canvas + the keep sources they describe.
    pub(crate) fn gate_for_snapshot(&self) -> (Arc<Vec<u8>>, Arc<Vec<u8>>) {
        (
            Arc::clone(&self.paint.gate_ref_rgba),
            Arc::clone(&self.paint.gate_free_rgba),
        )
    }

    /// Reinstate the epoch planes from a restored snapshot (the undo path — never a commit: the
    /// snapshot's planes pair with the snapshot's canvas and keep sources by construction).
    pub(crate) fn restore_gate_epoch(&mut self, gate_ref: Arc<Vec<u8>>, gate_free: Arc<Vec<u8>>) {
        self.paint.gate_ref_rgba = gate_ref;
        self.paint.gate_free_rgba = gate_free;
    }
}
