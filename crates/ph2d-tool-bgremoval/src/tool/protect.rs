//! Protection-brush state + dab kernel for [`BgRemovalTool`].
//!
//! The protection brush paints a source-resolution force-keep mask
//! threaded into `algorithm::run_pipeline` as the compose force-keep
//! argument. Arming, paint/erase, the interpolated stroke kernel, the
//! show-mask toggle, and the clear button all live here.
//!
//! SCAFFOLD note (Coordinator): the public arm/sample signatures are
//! the contract the panel + shell compile against — do NOT change them
//! without reporting (the shell `input_dispatch` + overlay call them).

use super::{BgRemovalTool, stamp_disc_into};
use crate::params::BrushFalloff;

impl BgRemovalTool {
    /// Whether the panel eyedropper is armed (shell samples canvas
    /// click-drags into extra colours while `true`).
    pub fn is_eyedropper_armed(&self) -> bool {
        self.eyedropper_armed
    }

    /// Set the eyedropper armed state (shell mirror of the panel toggle).
    /// Arming the eyedropper disarms the protect brush + add-area
    /// selector so the three canvas modes never fight over the same
    /// click.
    pub fn set_eyedropper_armed(&mut self, armed: bool) {
        self.eyedropper_armed = armed;
        if armed {
            self.protect_brush_armed = false;
            self.add_area_armed = false;
        }
    }

    // ── Protection brush (SCAFFOLD — Coordinator) ──────────────────────
    // Contract surface the panel + shell compile against. The Implementer
    // fills the dab/threading bodies + tests; do NOT change these public
    // signatures without reporting (the shell `input_dispatch` + overlay
    // call them). Mirrors the eyedropper arm/sample pattern.

    /// Whether the protection brush is armed (shell paints canvas
    /// click-drags into the protection mask while `true`).
    pub fn is_protect_armed(&self) -> bool {
        self.protect_brush_armed
    }

    /// Set the protection-brush armed state (shell mirror of the panel
    /// toggle). Arming the brush disarms the eyedropper AND the add-area
    /// selector so the three canvas modes never fight over the same click.
    pub fn set_protect_armed(&mut self, armed: bool) {
        self.protect_brush_armed = armed;
        if armed {
            self.eyedropper_armed = false;
            self.add_area_armed = false;
        }
    }

    /// Whether the protection mask currently holds any painted pixels.
    pub fn has_protect_mask(&self) -> bool {
        self.protect_mask.iter().any(|&v| v != 0)
    }

    /// Whether a protection-brush dab-drag is currently in progress
    /// (shell paints on every cursor-move while `true`).
    pub fn is_protect_painting(&self) -> bool {
        self.protect_painting
    }

    /// Set the protection-brush dab-drag state (shell sets `true` on
    /// pointer-down over the sprite, `false` on pointer-up). Clearing
    /// the flag also drops `last_protect_uv` so the next stroke
    /// doesn't draw an interpolated line from the previous stroke's
    /// final dab to the new starting position.
    pub fn set_protect_painting(&mut self, painting: bool) {
        if !painting {
            self.last_protect_uv = None;
        }
        self.protect_painting = painting;
    }

    /// Borrow the source-resolution protection mask for the shell's
    /// on-canvas overlay: `(mask, w, h)`, one byte/pixel (`255` =
    /// protected). Empty slice + `(0, 0)` when nothing is painted.
    pub fn protect_mask_source(&self) -> (&[u8], u32, u32) {
        (&self.protect_mask, self.protect_mask_w, self.protect_mask_h)
    }

    /// Paint a brush dab into the protection mask at normalized UV
    /// `(u, v)` (`[0,1]` each, origin top-left) with `radius_px` measured
    /// at SOURCE resolution. Called by the shell on canvas click-drag
    /// while the brush is armed (mirrors `add_extra_color` /
    /// `sample_source_at_uv`).
    ///
    /// Lazy-sizes `protect_mask` to the source dims and stamps a brush dab
    /// at UV `(u, v)` with `radius_px` (SOURCE px). The dab strength
    /// follows the active [`BrushFalloff`] over the normalized distance
    /// `d = dist/radius`, accumulating with `max` so overlapping dabs
    /// build up to full protection.
    ///
    /// Does NOT re-run the pipeline — painting only mutates the mask. The
    /// on-canvas tint overlay reads the mask live each frame (cheap); the
    /// matte re-segments once the stroke ends (the shell drops its cached
    /// preview on pointer-up) — so painting stays cheap (no per-dab
    /// re-segmentation).
    pub fn paint_protect_at_uv(&mut self, u: f32, v: f32, radius_px: f32) {
        self.stamp_protect(u, v, radius_px, false);
    }

    /// Erase from the protection mask at UV `(u, v)` — the inverse of
    /// [`Self::paint_protect_at_uv`]: subtracts the falloff strength
    /// (`saturating_sub`) so the centre erases fully and the rim only
    /// nibbles. No pipeline re-run (same rationale as paint).
    pub fn erase_protect_at_uv(&mut self, u: f32, v: f32, radius_px: f32) {
        self.stamp_protect(u, v, radius_px, true);
    }

    /// Shared dab kernel for paint (`erase = false`) / erase (`erase =
    /// true`). Lazy-sizes the mask, then walks the segment from the
    /// previous stroke anchor to `(u, v)` placing intermediate dabs
    /// every `STAMP_SPACING_FRAC * radius` SOURCE pixels. Without the
    /// interpolation a fast drag draws discrete discs along the path
    /// (Enio 2026-05-26: "espaço entre os pontos de pintura fossem
    /// muito grandes"); 4 dabs per radius (0.25 spacing) gives a
    /// continuous painterly trail with cheap accumulation.
    fn stamp_protect(&mut self, u: f32, v: f32, radius_px: f32, erase: bool) {
        if !self.has_source() {
            return;
        }
        let (w, h) = (self.source_w, self.source_h);
        let n = (w as usize) * (h as usize);
        // Erase on an unsized mask is a no-op (nothing to remove).
        if erase && self.protect_mask.len() != n {
            return;
        }
        // Lazy-size to the source resolution on first paint dab.
        if self.protect_mask.len() != n {
            self.protect_mask.clear();
            self.protect_mask.resize(n, 0);
            self.protect_mask_w = w;
            self.protect_mask_h = h;
        }
        let u = u.clamp(0.0, 1.0);
        let v = v.clamp(0.0, 1.0);
        let r = radius_px.max(0.5);

        // Interpolate intermediate dabs between the previous (u, v)
        // and this one so a fast drag draws a continuous trail
        // instead of spaced discs. STAMP_SPACING_FRAC = 0.05 → 20
        // dabs per radius of cursor motion. Tight overlap is required
        // to mask the Smooth-falloff per-disc ripple — at 0.25 the
        // mask still showed visible bumps along the stroke (Enio
        // 2026-05-26 round 2: "a pintura da máscara ainda não é
        // perfeitamente regular"). 0.05 trades a 4× higher dab count
        // for a visually uniform trail (cheap: each dab is just the
        // disc's bbox scan and `max`-accumulates).
        const STAMP_SPACING_FRAC: f32 = 0.05;
        let spacing_px = (r * STAMP_SPACING_FRAC).max(0.5);
        if let Some((lu, lv)) = self.last_protect_uv {
            let du_px = (u - lu) * (w as f32 - 1.0);
            let dv_px = (v - lv) * (h as f32 - 1.0);
            let dist_px = (du_px * du_px + dv_px * dv_px).sqrt();
            let n_steps = (dist_px / spacing_px).ceil().max(1.0) as u32;
            for i in 1..=n_steps {
                let t = i as f32 / n_steps as f32;
                let iu = lu + (u - lu) * t;
                let iv = lv + (v - lv) * t;
                self.stamp_single(iu, iv, r, erase);
            }
        } else {
            self.stamp_single(u, v, r, erase);
        }
        self.last_protect_uv = Some((u, v));

        // Protect dab mutates the mask (force-keep region for compose).
        // The matte itself only re-segments on pointer-up (shell drops
        // its cached preview there); but the on-canvas tint overlay
        // reads the mask each frame and the canvas preview gate uses
        // this flag — mark dirty so a follow-up render-loop tick sees
        // the new mask without waiting for an unrelated edit.
        self.params_dirty = true;
    }

    /// Stamp a single brush disc into the protection mask at UV
    /// `(u, v)` with `r` SOURCE-px radius. Thin wrapper over the
    /// generic [`stamp_disc_into`] free function — Protect and the
    /// symmetric force-remove brush ("Acrescentar Área") share the
    /// same kernel, only the mask buffer differs.
    fn stamp_single(&mut self, u: f32, v: f32, r: f32, erase: bool) {
        stamp_disc_into(
            &mut self.protect_mask,
            self.source_w,
            self.source_h,
            self.falloff,
            u,
            v,
            r,
            erase,
        );
    }

    /// Protection-brush radius in SOURCE pixels (the unit the shell passes
    /// to [`Self::paint_protect_at_uv`] and converts to a screen-space
    /// ring). Always ≥ a usable minimum.
    pub fn brush_radius_px(&self) -> f32 {
        self.brush_radius.max(0.5)
    }

    /// Active protection-brush falloff profile.
    pub fn falloff(&self) -> BrushFalloff {
        self.falloff
    }

    /// Whether the painted protection mask should be shown as an on-canvas
    /// tint overlay (shell gates its overlay on this).
    pub fn show_mask(&self) -> bool {
        self.show_mask
    }

    /// Whether the in-progress protection drag is an erase drag.
    pub fn is_protect_erasing(&self) -> bool {
        self.protect_erase_mode
    }

    /// Set the erase-drag flag (shell sets `true` on a secondary-button
    /// protection drag, `false` on a primary paint drag / drag-end).
    pub fn set_protect_erasing(&mut self, erasing: bool) {
        self.protect_erase_mode = erasing;
    }

    /// Wipe the painted protection mask. Reruns the preview when a source
    /// is loaded so the matte drops the forced-keep region immediately.
    pub fn clear_protect_mask(&mut self) {
        self.protect_mask.clear();
        self.protect_mask_w = 0;
        self.protect_mask_h = 0;
        if self.has_source() {
            self.rerun_preview();
        }
        // Canvas-preview cache must rebuild (the matte just dropped the
        // forced-keep region).
        self.params_dirty = true;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::params::{BgRemovalUiEdit, DEFAULT_BRUSH_SIZE01};

    #[test]
    fn paint_protect_stamps_disc_and_lazy_sizes_mask() {
        let mut t = BgRemovalTool::default();
        // No source → no-op (no panic, no mask).
        t.paint_protect_at_uv(0.5, 0.5, 4.0);
        assert!(!t.has_protect_mask());

        let buf = vec![255u8; 32 * 32 * 4];
        t.set_source_snapshot(bytemuck::allocation::cast_vec(buf), 32, 32);
        // Hard (Constant) falloff so the whole disc is full strength.
        t.apply_ui_edit(BgRemovalUiEdit::SetFalloff(BrushFalloff::Constant));
        t.paint_protect_at_uv(0.5, 0.5, 6.0);
        assert!(t.has_protect_mask());
        let (mask, w, h) = t.protect_mask_source();
        assert_eq!((w, h), (32, 32));
        // Centre pixel fully protected under Constant falloff.
        let c = 16 * 32 + 16;
        assert_eq!(mask[c], 255, "disc centre must be fully protected (hard)");
        // A far corner is untouched.
        assert_eq!(mask[0], 0, "corner outside the disc stays unprotected");
    }

    #[test]
    fn paint_protect_falloff_is_monotonic_center_to_rim() {
        // Smooth falloff: strength must be max at the centre and decay to
        // ~0 at the rim along a row through the dab.
        let mut t = BgRemovalTool::default();
        t.set_source_snapshot(
            bytemuck::allocation::cast_vec(vec![255u8; 64 * 64 * 4]),
            64,
            64,
        );
        t.apply_ui_edit(BgRemovalUiEdit::SetFalloff(BrushFalloff::Smooth));
        let r = 20.0;
        t.paint_protect_at_uv(0.5, 0.5, r);
        let (mask, _, _) = t.protect_mask_source();
        let cx = (0.5_f32 * 63.0).round() as usize;
        let row = cx * 64;
        let centre = mask[row + cx] as i32;
        let mid = mask[row + cx + 10] as i32; // ~half radius out
        let rim = mask[row + cx + 19] as i32; // near the rim
        assert!(centre >= mid && mid >= rim, "{centre} >= {mid} >= {rim}");
        assert!(centre > 200, "centre near-full, got {centre}");
        assert!(rim < 64, "rim near-zero, got {rim}");
    }

    #[test]
    fn erase_protect_subtracts_strength() {
        let mut t = BgRemovalTool::default();
        t.set_source_snapshot(
            bytemuck::allocation::cast_vec(vec![255u8; 32 * 32 * 4]),
            32,
            32,
        );
        t.apply_ui_edit(BgRemovalUiEdit::SetFalloff(BrushFalloff::Constant));
        // Paint a hard disc, then erase the centre with a hard dab.
        t.paint_protect_at_uv(0.5, 0.5, 8.0);
        let c = 16 * 32 + 16;
        assert_eq!(t.protect_mask_source().0[c], 255);
        t.erase_protect_at_uv(0.5, 0.5, 4.0);
        assert_eq!(
            t.protect_mask_source().0[c],
            0,
            "hard erase clears the painted centre"
        );
    }

    #[test]
    fn erase_on_empty_mask_is_noop() {
        let mut t = BgRemovalTool::default();
        t.set_source_snapshot(
            bytemuck::allocation::cast_vec(vec![255u8; 16 * 16 * 4]),
            16,
            16,
        );
        t.erase_protect_at_uv(0.5, 0.5, 4.0);
        assert!(
            !t.has_protect_mask(),
            "erase without a painted mask is inert"
        );
    }

    #[test]
    fn stroke_interpolation_fills_gap_between_distant_dabs() {
        // Two dabs spaced 30 px apart within one stroke with a small
        // radius (4 px) — before stroke interpolation, the midpoint
        // was untouched ("bolinhas" visible along the path). After
        // the fix, the segment is continuously covered.
        let mut t = BgRemovalTool::default();
        t.set_source_snapshot(
            bytemuck::allocation::cast_vec(vec![255u8; 64 * 64 * 4]),
            64,
            64,
        );
        t.apply_ui_edit(BgRemovalUiEdit::SetFalloff(BrushFalloff::Constant));
        let r = 4.0;
        // Stroke begins at (10, 32), ends at (40, 32) — 30 px horizontal.
        let w = 63.0_f32;
        let h = 63.0_f32;
        t.paint_protect_at_uv(10.0 / w, 32.0 / h, r);
        t.paint_protect_at_uv(40.0 / w, 32.0 / h, r);
        let (mask, _, _) = t.protect_mask_source();
        // Walk the midline: every pixel from (10..=40, 32) must be
        // touched. A single gap = the bug Enio reported.
        for x in 10..=40 {
            let i = 32 * 64 + x;
            assert!(
                mask[i] >= 200,
                "px {x} on the stroke path must be protected (got {})",
                mask[i]
            );
        }
        // Outside the stroke band stays untouched.
        let outside = 50 * 64 + 50;
        assert_eq!(mask[outside], 0);
    }

    #[test]
    fn stroke_anchor_resets_between_strokes() {
        // Stroke 1: paint at (10, 10). Pointer-up. Stroke 2: paint at
        // (50, 50). The line connecting them must NOT be filled — the
        // anchor reset on pointer-up prevents cross-stroke interpolation.
        let mut t = BgRemovalTool::default();
        t.set_source_snapshot(
            bytemuck::allocation::cast_vec(vec![255u8; 64 * 64 * 4]),
            64,
            64,
        );
        t.apply_ui_edit(BgRemovalUiEdit::SetFalloff(BrushFalloff::Constant));
        let r = 3.0;
        let scale = 63.0_f32;
        // First stroke.
        t.set_protect_painting(true);
        t.paint_protect_at_uv(10.0 / scale, 10.0 / scale, r);
        t.set_protect_painting(false); // pointer-up — resets anchor.
        // Second stroke, far away.
        t.set_protect_painting(true);
        t.paint_protect_at_uv(50.0 / scale, 50.0 / scale, r);
        t.set_protect_painting(false);
        let (mask, _, _) = t.protect_mask_source();
        // The mid-segment between the two strokes must remain untouched.
        let mid = 30 * 64 + 30;
        assert_eq!(
            mask[mid], 0,
            "no interpolation across pointer-up boundary (mid-segment must be clean)"
        );
        // Both stroke endpoints ARE painted.
        assert!(mask[10 * 64 + 10] > 200);
        assert!(mask[50 * 64 + 50] > 200);
    }

    #[test]
    fn brush_size_edit_maps_and_round_trips() {
        use crate::params::BRUSH_SIZE_FULL_SCALE;
        let mut t = BgRemovalTool::default();
        // Default snapshot reflects DEFAULT_BRUSH_SIZE01.
        assert!((t.ui_snapshot().brush_size01 - DEFAULT_BRUSH_SIZE01).abs() < 1e-5);
        t.apply_ui_edit(BgRemovalUiEdit::BrushSize(0.5));
        assert!((t.ui_snapshot().brush_size01 - 0.5).abs() < 1e-5);
        assert!((t.brush_radius_px() - 0.5 * BRUSH_SIZE_FULL_SCALE).abs() < 1e-3);
    }

    #[test]
    fn show_mask_and_falloff_edits() {
        let mut t = BgRemovalTool::default();
        assert!(t.show_mask(), "show-mask defaults on");
        t.apply_ui_edit(BgRemovalUiEdit::ToggleShowMask);
        assert!(!t.show_mask());
        assert_eq!(t.falloff(), BrushFalloff::Smooth);
        t.apply_ui_edit(BgRemovalUiEdit::SetFalloff(BrushFalloff::Sharp));
        assert_eq!(t.falloff(), BrushFalloff::Sharp);
        assert_eq!(t.ui_snapshot().falloff, BrushFalloff::Sharp);
    }

    #[test]
    fn clear_protect_mask_wipes_it() {
        let mut t = BgRemovalTool::default();
        let buf = vec![255u8; 16 * 16 * 4];
        t.set_source_snapshot(bytemuck::allocation::cast_vec(buf), 16, 16);
        t.paint_protect_at_uv(0.5, 0.5, 3.0);
        assert!(t.has_protect_mask());
        t.clear_protect_mask();
        assert!(!t.has_protect_mask());
        let (mask, w, h) = t.protect_mask_source();
        assert!(mask.is_empty());
        assert_eq!((w, h), (0, 0));
    }

    #[test]
    fn new_image_dims_clear_stale_protect_mask() {
        let mut t = BgRemovalTool::default();
        t.set_source_snapshot(
            bytemuck::allocation::cast_vec(vec![255u8; 16 * 16 * 4]),
            16,
            16,
        );
        t.paint_protect_at_uv(0.5, 0.5, 3.0);
        assert!(t.has_protect_mask());
        // Same dims → preserved (Apply re-feed case).
        t.set_source_snapshot(
            bytemuck::allocation::cast_vec(vec![128u8; 16 * 16 * 4]),
            16,
            16,
        );
        assert!(t.has_protect_mask(), "same-dims re-feed keeps the mask");
        // Different dims → cleared.
        t.set_source_snapshot(bytemuck::allocation::cast_vec(vec![255u8; 8 * 8 * 4]), 8, 8);
        assert!(!t.has_protect_mask(), "new dimensions drop the stale mask");
    }
}
