//! Source-snapshot ingestion + derived buffers for [`BgRemovalTool`].
//!
//! Owns the host-pushed source RGBA, the letterboxed panel thumbnail,
//! the downscaled on-canvas preview source, the thumbnail preview
//! rerun, nearest-pixel source sampling, and the one-shot drain
//! accessors the host polls each frame.

use super::{
    BgRemovalTool, IslandPayload, PREVIEW_MAX_DIM, THUMB_SIZE, aspect_fit, aspect_fit_within,
};
use crate::algorithm::run_pipeline;

impl BgRemovalTool {
    /// Push a fresh source RGBA snapshot from the host. Called when
    /// the selection changes or the tool becomes active. Rebuilds
    /// the thumbnail and re-renders the preview with the current
    /// params.
    ///
    /// `pixels` must be straight-alpha `SrgbRgba` of length `w * h`.
    /// Internally re-stored as `Vec<u8>` (downstream consumes bytes);
    /// the cast is zero-copy via `bytemuck::cast_vec`.
    pub fn set_source_snapshot(&mut self, pixels: Vec<ph2d_color::SrgbRgba>, w: u32, h: u32) {
        assert_eq!(pixels.len(), (w as usize) * (h as usize));
        // The protection mask is spatial — a genuinely different image
        // invalidates it. Re-feeding the SAME dimensions (e.g. the Apply
        // re-read of the same sprite) preserves it so the bake honours
        // the painted region. The force-remove mask + its seed list
        // are also spatial → same dim-change invalidation.
        if w != self.protect_mask_w || h != self.protect_mask_h {
            self.protect_mask.clear();
            self.protect_mask_w = 0;
            self.protect_mask_h = 0;
        }
        if w != self.force_remove_mask_w || h != self.force_remove_mask_h {
            self.force_remove_mask.clear();
            self.force_remove_mask_w = 0;
            self.force_remove_mask_h = 0;
            self.add_area_seeds.clear();
        }
        self.source_rgba = bytemuck::allocation::cast_vec(pixels);
        self.source_w = w;
        self.source_h = h;
        self.rebuild_thumbnail();
        self.rebuild_canvas_src();
        self.rerun_preview();
        // The cached on-canvas preview the shell holds was computed against
        // the previous selection's source — mark dirty so the next bridge
        // tick rebuilds it. (Shell also drops `bgremoval_preview` when
        // `last_bgremoval_pushed_entity` changes, so this is a belt-and-
        // suspenders for the path where the snapshot push fires for some
        // other reason but the cached preview is now stale.)
        self.params_dirty = true;
        // Wave 10 / Etapa 1.B audit fix [A1]: explicitly invalidate the
        // tool's internal canvas-preview cache so a stale frame from the
        // previous selection can NEVER paint over the new sprite, even
        // in the path where the shell read failed mid-frame and never
        // refilled its own cache. Without this, the next current_preview
        // call would short-circuit if dirty was somehow consumed first.
        self.cached_canvas_preview = None;
        // Source pixels changed → the cached source-resolution silhouette
        // is stale (its outline reflects the previous image's edges).
        // Force a recompute on the next pipeline tick that needs it.
        self.cached_auto_protect_for = None;
    }

    /// Whether the host has pushed a source snapshot at least once.
    pub fn has_source(&self) -> bool {
        !self.source_rgba.is_empty()
    }

    /// Source texture resolution `(w, h)` of the active snapshot, or
    /// `(0, 0)` before any source is pushed. The shell uses this to map
    /// an on-screen protection-brush radius into source pixels (the unit
    /// [`Self::paint_protect_at_uv`] expects) on the very first dab,
    /// before the protection mask itself is sized.
    pub fn source_size(&self) -> (u32, u32) {
        (self.source_w, self.source_h)
    }

    /// Borrow the current thumbnail preview (RGBA8,
    /// `THUMB_SIZE × THUMB_SIZE`). Returns an empty slice when
    /// `has_source()` is false.
    pub fn preview_rgba(&self) -> &[u8] {
        &self.preview_rgba
    }

    /// Drain the pending-apply flag. Returns `true` exactly once
    /// after each Apply trigger. Host calls this in its per-frame
    /// drain loop; on `true` it runs the pipeline at full resolution.
    /// Drain the per-island RGBA payloads produced by the last Apply
    /// when `params.separate_islands` was on. Returns an empty Vec when
    /// the toggle is off, no Apply has run yet, or the host already
    /// drained. The shell typically calls this right after baking the
    /// main result and spawns one new sprite per returned payload
    /// (legacy parity — biggest island stays in the original sprite,
    /// rest get sibling sprites positioned at their bounding-box origins).
    pub fn take_pending_islands(&mut self) -> Vec<IslandPayload> {
        std::mem::take(&mut self.pending_islands)
    }

    pub fn take_pending_apply(&mut self) -> bool {
        let p = self.pending_apply;
        self.pending_apply = false;
        p
    }

    /// Drain the params-dirty flag. Returns `true` exactly once when
    /// any panel-edit / extra-colour / protect-mask mutator has run
    /// since the last call. The shell uses this as the gate for
    /// rerunning the on-canvas live preview (ADR-0040 TG-B replacement
    /// for the old `!bgremoval_ui_edits.is_empty()` check).
    pub fn take_pending_panel_reset(&mut self) -> bool {
        std::mem::take(&mut self.pending_panel_reset)
    }

    pub fn take_params_dirty(&mut self) -> bool {
        std::mem::take(&mut self.params_dirty)
    }

    /// Sample the stored SOURCE snapshot at normalized UV `(u, v)`
    /// (`[0,1]` each, origin top-left), nearest-pixel. Returns the RGB
    /// of that pixel, or `None` when no source is loaded or the UV is
    /// out of range. Samples the SOURCE — never the framebuffer — so the
    /// picked colour is the true sprite colour, not the composited
    /// preview (which carries the in-progress transparency).
    pub fn sample_source_at_uv(&self, u: f32, v: f32) -> Option<[u8; 3]> {
        if !self.has_source() || !(0.0..=1.0).contains(&u) || !(0.0..=1.0).contains(&v) {
            return None;
        }
        // Nearest-pixel: map [0,1] onto [0, dim-1].
        let px = ((u * (self.source_w as f32 - 1.0)).round() as i64)
            .clamp(0, self.source_w as i64 - 1) as usize;
        let py = ((v * (self.source_h as f32 - 1.0)).round() as i64)
            .clamp(0, self.source_h as i64 - 1) as usize;
        let base = (py * self.source_w as usize + px) * 4;
        Some([
            self.source_rgba[base],
            self.source_rgba[base + 1],
            self.source_rgba[base + 2],
        ])
    }

    /// Aspect-fit `source_rgba` into a `THUMB_SIZE × THUMB_SIZE` RGBA8
    /// buffer with transparent letterbox borders. Uses
    /// `image::imageops::resize` with `Triangle` (cheap box-quality,
    /// no ringing — fine for a 160-px preview that gets re-segmented
    /// every panel-event frame).
    ///
    /// No-op when the host hasn't pushed a source snapshot yet.
    ///
    /// Allocations: one `ImageBuffer` for the source view and one for
    /// the resized output (both freed before return). The owned
    /// `self.thumbnail_rgba` is `clear()`-ed and re-extended so its
    /// capacity persists across calls (HR-3 in the steady state where
    /// every Apply sees the same source size).
    pub(crate) fn rebuild_thumbnail(&mut self) {
        if !self.has_source() {
            self.thumbnail_w = 0;
            self.thumbnail_h = 0;
            self.thumbnail_rgba.clear();
            return;
        }
        let target = THUMB_SIZE;
        // Aspect-fit: scale the LONGER side to `target`, the shorter
        // side gets proportional scaling. Degenerate dims fall back
        // to 1 px so the resize call doesn't panic.
        let (sw, sh) = aspect_fit(self.source_w, self.source_h, target);
        let src = image::ImageBuffer::<image::Rgba<u8>, _>::from_raw(
            self.source_w,
            self.source_h,
            self.source_rgba.clone(),
        )
        .expect("source_rgba length matches source_w * source_h * 4");
        let resized: image::ImageBuffer<image::Rgba<u8>, Vec<u8>> =
            if sw == self.source_w && sh == self.source_h {
                src
            } else {
                image::imageops::resize(&src, sw, sh, image::imageops::FilterType::Triangle)
            };
        // Letterbox into target × target with transparent borders.
        let pad_x = (target - sw) / 2;
        let pad_y = (target - sh) / 2;
        let total_bytes = (target as usize) * (target as usize) * 4;
        self.thumbnail_rgba.clear();
        self.thumbnail_rgba.resize(total_bytes, 0);
        for row in 0..sh {
            let dst_y = (pad_y + row) as usize;
            let dst_start = (dst_y * (target as usize) + pad_x as usize) * 4;
            let src_start = (row as usize) * (sw as usize) * 4;
            let row_bytes = (sw as usize) * 4;
            self.thumbnail_rgba[dst_start..dst_start + row_bytes]
                .copy_from_slice(&resized.as_raw()[src_start..src_start + row_bytes]);
        }
        self.thumbnail_w = target;
        self.thumbnail_h = target;
    }

    /// Re-run the segmentation pipeline against the cached thumbnail
    /// with the current `params`. Output lands in `self.preview_rgba`,
    /// always `THUMB_SIZE * THUMB_SIZE * 4` bytes.
    ///
    /// No-op when `rebuild_thumbnail` hasn't produced a buffer yet.
    pub(crate) fn rerun_preview(&mut self) {
        if self.thumbnail_rgba.is_empty() {
            self.preview_rgba.clear();
            return;
        }
        // The 160² thumbnail preview is letterboxed; threading the
        // protection mask through it would need the same letterbox
        // remap. It is not the user-facing preview (the on-canvas
        // overlay is), so it runs without protection or force-remove.
        run_pipeline(
            &self.thumbnail_rgba,
            self.thumbnail_w,
            self.thumbnail_h,
            &self.params,
            None,
            None,
            &mut self.scratch,
        );
        self.preview_rgba.clear();
        self.preview_rgba
            .extend_from_slice(&self.scratch.output_rgba);
    }

    /// Rebuild [`Self::canvas_src_rgba`] — the source downscaled to fit
    /// [`PREVIEW_MAX_DIM`] (aspect preserved, no letterbox). Called once
    /// per source snapshot; the on-canvas preview re-segments this small
    /// buffer on every parameter change. No-op without a source.
    pub(crate) fn rebuild_canvas_src(&mut self) {
        if !self.has_source() {
            self.canvas_src_w = 0;
            self.canvas_src_h = 0;
            self.canvas_src_rgba.clear();
            return;
        }
        let (dw, dh) = aspect_fit_within(self.source_w, self.source_h, PREVIEW_MAX_DIM);
        self.canvas_src_rgba.clear();
        if dw == self.source_w && dh == self.source_h {
            self.canvas_src_rgba.extend_from_slice(&self.source_rgba);
        } else {
            let src = image::ImageBuffer::<image::Rgba<u8>, _>::from_raw(
                self.source_w,
                self.source_h,
                self.source_rgba.clone(),
            )
            .expect("source_rgba length matches source_w * source_h * 4");
            let resized: image::ImageBuffer<image::Rgba<u8>, Vec<u8>> =
                image::imageops::resize(&src, dw, dh, image::imageops::FilterType::Triangle);
            self.canvas_src_rgba.extend_from_slice(resized.as_raw());
        }
        self.canvas_src_w = dw;
        self.canvas_src_h = dh;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn set_source_snapshot_marks_has_source_true() {
        let mut t = BgRemovalTool::default();
        let buf = vec![255u8; 8 * 8 * 4];
        t.set_source_snapshot(bytemuck::allocation::cast_vec(buf), 8, 8);
        assert!(t.has_source());
    }

    #[test]
    fn set_source_snapshot_builds_thumbnail_and_preview() {
        // Push a 32×32 opaque-white source; the thumbnail must
        // letterbox to 160×160 and the preview pipeline must produce
        // a same-size buffer.
        let mut t = BgRemovalTool::default();
        let buf = vec![255u8; 32 * 32 * 4];
        t.set_source_snapshot(bytemuck::allocation::cast_vec(buf), 32, 32);
        assert_eq!(t.thumbnail_w, THUMB_SIZE);
        assert_eq!(t.thumbnail_h, THUMB_SIZE);
        assert_eq!(
            t.thumbnail_rgba.len(),
            (THUMB_SIZE as usize) * (THUMB_SIZE as usize) * 4
        );
        assert_eq!(
            t.preview_rgba().len(),
            (THUMB_SIZE as usize) * (THUMB_SIZE as usize) * 4
        );
    }

    #[test]
    fn sample_source_at_uv_maps_corners() {
        // 2×2 source with 4 distinct colours: TL red, TR green,
        // BL blue, BR white.
        let mut t = BgRemovalTool::default();
        let buf: Vec<u8> = vec![
            255, 0, 0, 255, // (0,0) red
            0, 255, 0, 255, // (1,0) green
            0, 0, 255, 255, // (0,1) blue
            255, 255, 255, 255, // (1,1) white
        ];
        t.set_source_snapshot(bytemuck::allocation::cast_vec(buf), 2, 2);
        assert_eq!(t.sample_source_at_uv(0.0, 0.0), Some([255, 0, 0]));
        assert_eq!(t.sample_source_at_uv(1.0, 0.0), Some([0, 255, 0]));
        assert_eq!(t.sample_source_at_uv(0.0, 1.0), Some([0, 0, 255]));
        assert_eq!(t.sample_source_at_uv(1.0, 1.0), Some([255, 255, 255]));
        // Out of range → None.
        assert_eq!(t.sample_source_at_uv(1.5, 0.0), None);
        assert_eq!(t.sample_source_at_uv(0.0, -0.1), None);
    }

    #[test]
    fn sample_source_at_uv_none_without_source() {
        let t = BgRemovalTool::default();
        assert_eq!(t.sample_source_at_uv(0.5, 0.5), None);
    }
}
