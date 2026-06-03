//! Pipeline drivers for [`BgRemovalTool`].
//!
//! Owns the two `algorithm::run_pipeline` callers (full-res Apply +
//! capped on-canvas preview), the combined-protect preparation
//! (folding the painted mask + edge-aware silhouette + "Add area"
//! un-protect), the source-resolution silhouette cache, and the
//! nearest-resamplers that bring the source-resolution masks down to
//! the canvas-preview dims.

use super::BgRemovalTool;
use crate::algorithm::{islands, run_pipeline, silhouette};

impl BgRemovalTool {
    /// Run the full-resolution pipeline on the cached `source_rgba`
    /// (called from the host's drain handler) and write the result
    /// into `out`. `out` is grown to `source_w * source_h * 4` if
    /// needed.
    ///
    /// Returns the `(w, h)` of the output.
    pub fn run_full_resolution(&mut self, out: &mut Vec<u8>) -> (u32, u32) {
        assert!(self.has_source(), "set_source_snapshot must run first");
        // The protection mask is stored at source resolution, so it
        // aligns 1:1 with the full-res pipeline input. The combined
        // mask folds in `auto_protect_subject` when the toggle is on
        // (edge-aware silhouette upgrade — Enio 2026-05-26).
        let has_protect = self.prepare_combined_protect_full();
        let n = (self.source_w as usize) * (self.source_h as usize);
        let protect: Option<&[u8]> = if has_protect {
            Some(&self.combined_protect[..n])
        } else {
            None
        };
        let force_remove: Option<&[u8]> = if self.force_remove_mask.len() == n {
            Some(&self.force_remove_mask[..n])
        } else {
            None
        };
        run_pipeline(
            &self.source_rgba,
            self.source_w,
            self.source_h,
            &self.params,
            protect,
            force_remove,
            &mut self.scratch,
        );
        out.clear();
        out.extend_from_slice(&self.scratch.output_rgba);

        // Legacy parity: when "Separate Islands" is on, run CCL on the
        // freshly composed RGBA and stash one payload per surviving
        // component (filtered by `min_island_pixels`). The host drains
        // via `take_pending_islands` and spawns the rest as sibling
        // sprites — keeping the biggest one in the original. When the
        // toggle is off, ensure the slot is empty so a stale post-Apply
        // queue from a previous run doesn't leak.
        //
        // We read from `out` (just copied above) rather than
        // `self.scratch.output_rgba` so `&mut self.scratch` (for the
        // CCL label + queue buffers) doesn't clash with the source
        // borrow inside the same call.
        if self.params.separate_islands {
            islands::extract(
                out,
                self.source_w,
                self.source_h,
                self.params.min_island_pixels.max(1),
                &mut self.scratch,
                &mut self.pending_islands,
            );
        } else {
            self.pending_islands.clear();
        }

        (self.source_w, self.source_h)
    }

    /// Run the pipeline for the live on-canvas preview at a capped
    /// resolution (see [`super::PREVIEW_MAX_DIM`]) and write the result
    /// into `out`. The shell draws this scaled to the sprite footprint,
    /// so a slider drag re-segments a small image — keeping the drag
    /// smooth. Returns the `(w, h)` of the output (the capped preview
    /// dims, NOT the source dims).
    ///
    /// No-op (returns `(0, 0)`, clears `out`) when no source is loaded.
    pub fn run_canvas_preview(&mut self, out: &mut Vec<u8>) -> (u32, u32) {
        if self.canvas_src_rgba.is_empty() {
            out.clear();
            return (0, 0);
        }
        let (cw, ch) = (self.canvas_src_w, self.canvas_src_h);
        let n = (cw as usize) * (ch as usize);
        // Resample the source-resolution force-remove mask into the
        // canvas dims FIRST so `prepare_combined_protect_canvas`
        // below can subtract it from the protect buffer (the
        // "Add area" pixels must NOT be force-kept by the silhouette
        // / protect-brush mask, otherwise the chroma injection's
        // `mask=0` would be overridden by `force_keep_protected` in
        // compose — Enio 2026-05-27).
        let has_force_remove = !self.force_remove_mask.is_empty();
        if has_force_remove {
            self.resize_force_remove_into(cw, ch);
        }
        let has_protect = self.prepare_combined_protect_canvas(cw, ch);
        let protect: Option<&[u8]> = if has_protect {
            Some(&self.combined_protect[..n])
        } else {
            None
        };
        let force_remove: Option<&[u8]> = if has_force_remove && self.canvas_remove.len() == n {
            Some(&self.canvas_remove[..n])
        } else {
            None
        };
        run_pipeline(
            &self.canvas_src_rgba,
            cw,
            ch,
            &self.params,
            protect,
            force_remove,
            &mut self.scratch,
        );
        out.clear();
        out.extend_from_slice(&self.scratch.output_rgba);
        (cw, ch)
    }

    /// Fold the user-painted protect mask (source-resolution, byte
    /// per pixel) and — when `auto_protect_subject` is on — the
    /// edge-aware silhouette mask into [`Self::combined_protect`].
    /// Returns `true` iff at least one source contributed any non-zero
    /// pixel (so the caller can hand `None` to the pipeline when no
    /// protect mask is in play).
    ///
    /// Runs at SOURCE resolution. The auto-protect step calls
    /// [`silhouette::detect_subject_interior`] each Apply —
    /// not cached because Apply is rare and Reset/source-change
    /// invalidations were noisier than the recompute (the silhouette
    /// is ~30ms at 2048², well under the Apply time budget).
    fn prepare_combined_protect_full(&mut self) -> bool {
        if !self.has_source() {
            return false;
        }
        let n = (self.source_w as usize) * (self.source_h as usize);
        let has_user = self.protect_mask.len() == n && !self.protect_mask.is_empty();
        let has_auto = self.params.auto_protect_subject;
        if !has_user && !has_auto {
            return false;
        }
        if has_auto {
            self.ensure_auto_protect_cached_at_source();
        }
        self.combined_protect.clear();
        self.combined_protect.resize(n, 0);
        if has_auto && self.cached_auto_protect_source.len() == n {
            for i in 0..n {
                if self.cached_auto_protect_source[i] > self.combined_protect[i] {
                    self.combined_protect[i] = self.cached_auto_protect_source[i];
                }
            }
        }
        if has_user {
            for i in 0..n {
                if self.protect_mask[i] > self.combined_protect[i] {
                    self.combined_protect[i] = self.protect_mask[i];
                }
            }
        }
        // "Add area" un-protects (Enio 2026-05-27): pixels the user
        // explicitly flagged for removal can't be force-kept by the
        // auto silhouette or the protect brush — otherwise compose's
        // `force_keep_protected` would raise their alpha back up after
        // the pipeline injected `mask=0` for them.
        if self.force_remove_mask.len() == n {
            for i in 0..n {
                if self.force_remove_mask[i] > 0 {
                    self.combined_protect[i] = 0;
                }
            }
        }
        // Even when neither auto-protect nor user-paint set anything,
        // the unprotect pass may have left a non-empty combined buffer
        // that the caller still wants to pass through — return true
        // whenever ANY of the three sources contributed.
        true
    }

    /// Recompute the source-resolution silhouette into
    /// `cached_auto_protect_source` if it's missing or stale (dim
    /// mismatch). No-op once cached.
    fn ensure_auto_protect_cached_at_source(&mut self) {
        if !self.has_source() {
            self.cached_auto_protect_source.clear();
            self.cached_auto_protect_for = None;
            return;
        }
        let target = (self.source_w, self.source_h);
        let n = (target.0 as usize) * (target.1 as usize);
        if self.cached_auto_protect_for == Some(target)
            && self.cached_auto_protect_source.len() == n
        {
            return;
        }
        // Size scratch + cache to source dims.
        self.scratch
            .ensure(target.0, target.1, self.params.refinement.color_guide);
        self.cached_auto_protect_source.clear();
        self.cached_auto_protect_source.resize(n, 0);
        silhouette::detect_subject_interior(
            &self.source_rgba,
            target.0,
            target.1,
            &mut self.scratch.luma,
            &mut self.scratch.sobel_mag,
            &mut self.scratch.edge_a,
            &mut self.scratch.edge_b,
            &mut self.scratch.silhouette_visited,
            &mut self.scratch.silhouette_queue,
            &mut self.cached_auto_protect_source,
        );
        self.cached_auto_protect_for = Some(target);
    }

    /// Canvas-preview variant: resamples the user protect mask down to
    /// `(dw, dh)` and runs the silhouette directly on `canvas_src_rgba`
    /// (cheap at preview resolution — ~3ms at 256² on M-series).
    /// Returns `true` iff `combined_protect` is non-empty.
    fn prepare_combined_protect_canvas(&mut self, dw: u32, dh: u32) -> bool {
        let n = (dw as usize) * (dh as usize);
        if n == 0 {
            return false;
        }
        let has_user = !self.protect_mask.is_empty();
        let has_auto = self.params.auto_protect_subject && self.has_source();
        if !has_user && !has_auto {
            return false;
        }
        // Compute the silhouette at SOURCE resolution + cache. This
        // is what gives the preview the SAME outline + soft-falloff
        // band geometry as Apply: at canvas resolution
        // `DISTANCE_TO_FULL_LOCK = 8` would cover ~3% of the image
        // (visibly soft edges) while at source resolution it covers
        // ~0.4% (sharp edges, matching Apply). The cache means we
        // only pay the source-res silhouette cost ONCE per source
        // change, not every preview tick.
        if has_auto {
            self.ensure_auto_protect_cached_at_source();
        }
        // Size scratch + combined_protect to the canvas dims for the
        // pipeline run that follows.
        self.scratch
            .ensure(dw, dh, self.params.refinement.color_guide);
        self.combined_protect.clear();
        self.combined_protect.resize(n, 0);
        if has_auto && !self.cached_auto_protect_source.is_empty() {
            // Nearest-resample source-resolution mask down to canvas
            // dims (same pattern as `resize_protect_into`). The
            // source band is `DISTANCE_TO_FULL_LOCK = 8` pixels;
            // downsample collapses that to ~1 preview pixel, so the
            // preview's silhouette ramp is as tight as Apply's.
            let sw = self.source_w as u64;
            let sh = self.source_h as u64;
            let dw_u = dw as u64;
            let dh_u = dh as u64;
            for y in 0..dh as usize {
                let sy = (((y as u64) * sh) / dh_u).min(sh - 1) as usize;
                for x in 0..dw as usize {
                    let sx = (((x as u64) * sw) / dw_u).min(sw - 1) as usize;
                    let v = self.cached_auto_protect_source[sy * (sw as usize) + sx];
                    let dst = y * (dw as usize) + x;
                    if v > self.combined_protect[dst] {
                        self.combined_protect[dst] = v;
                    }
                }
            }
        }
        if has_user {
            self.resize_protect_into(dw, dh);
            for i in 0..n {
                if self.canvas_protect[i] > self.combined_protect[i] {
                    self.combined_protect[i] = self.canvas_protect[i];
                }
            }
        }
        // "Add area" un-protects at canvas resolution — mirror of the
        // full-res branch. `canvas_remove` is resampled by the caller
        // (`run_canvas_preview`) BEFORE this method runs, so the
        // canvas-dim buffer is ready here.
        if self.canvas_remove.len() == n {
            for i in 0..n {
                if self.canvas_remove[i] > 0 {
                    self.combined_protect[i] = 0;
                }
            }
        }
        true
    }

    /// Nearest-resample the source-resolution protection mask into
    /// `self.canvas_protect` at `(dw, dh)`. Reuses the allocation.
    fn resize_protect_into(&mut self, dw: u32, dh: u32) {
        let n = (dw as usize) * (dh as usize);
        self.canvas_protect.clear();
        self.canvas_protect.resize(n, 0);
        let (sw, sh) = (self.protect_mask_w, self.protect_mask_h);
        if self.protect_mask.is_empty() || sw == 0 || sh == 0 || dw == 0 || dh == 0 {
            return;
        }
        let (sw_u, sh_u) = (sw as u64, sh as u64);
        let (dw_u, dh_u) = (dw as u64, dh as u64);
        for y in 0..dh as usize {
            let sy = (((y as u64) * sh_u) / dh_u).min(sh_u - 1) as usize;
            for x in 0..dw as usize {
                let sx = (((x as u64) * sw_u) / dw_u).min(sw_u - 1) as usize;
                self.canvas_protect[y * dw as usize + x] = self.protect_mask[sy * sw as usize + sx];
            }
        }
    }

    /// Mirror of [`Self::resize_protect_into`] for the force-remove
    /// mask. Nearest-resample the source-resolution force-remove mask
    /// into `self.canvas_remove` at `(dw, dh)`.
    fn resize_force_remove_into(&mut self, dw: u32, dh: u32) {
        let n = (dw as usize) * (dh as usize);
        self.canvas_remove.clear();
        self.canvas_remove.resize(n, 0);
        let (sw, sh) = (self.force_remove_mask_w, self.force_remove_mask_h);
        if self.force_remove_mask.is_empty() || sw == 0 || sh == 0 || dw == 0 || dh == 0 {
            return;
        }
        let (sw_u, sh_u) = (sw as u64, sh as u64);
        let (dw_u, dh_u) = (dw as u64, dh as u64);
        for y in 0..dh as usize {
            let sy = (((y as u64) * sh_u) / dh_u).min(sh_u - 1) as usize;
            for x in 0..dw as usize {
                let sx = (((x as u64) * sw_u) / dw_u).min(sw_u - 1) as usize;
                self.canvas_remove[y * dw as usize + x] =
                    self.force_remove_mask[sy * sw as usize + sx];
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::scratch::BgRemovalScratch;

    #[test]
    fn canvas_preview_runs_at_source_resolution() {
        // PREVIEW_MAX_DIM was lifted to u32::MAX 2026-05-26 so the
        // live preview pipeline is byte-identical to Apply (no
        // downscale-induced anti-alias / silhouette divergence).
        // The preview now passes the source through at native res.
        let mut t = BgRemovalTool::default();
        let buf = vec![255u8; 1024 * 512 * 4];
        t.set_source_snapshot(bytemuck::allocation::cast_vec(buf), 1024, 512);
        let mut out = Vec::new();
        let (cw, ch) = t.run_canvas_preview(&mut out);
        assert_eq!((cw, ch), (1024, 512), "preview now runs at source dims");
        assert_eq!(out.len(), (cw as usize) * (ch as usize) * 4);
    }

    #[test]
    fn canvas_preview_small_source_passes_through() {
        let mut t = BgRemovalTool::default();
        let buf = vec![255u8; 64 * 48 * 4];
        t.set_source_snapshot(bytemuck::allocation::cast_vec(buf), 64, 48);
        let mut out = Vec::new();
        let (cw, ch) = t.run_canvas_preview(&mut out);
        assert_eq!((cw, ch), (64, 48), "sub-cap source is not upscaled");
    }

    #[test]
    fn canvas_preview_no_source_is_noop() {
        let mut t = BgRemovalTool::default();
        let mut out = vec![1u8, 2, 3];
        let (cw, ch) = t.run_canvas_preview(&mut out);
        assert_eq!((cw, ch), (0, 0));
        assert!(out.is_empty());
    }

    #[test]
    fn pending_islands_stays_empty_when_toggle_off() {
        let mut t = BgRemovalTool::default();
        t.set_source_snapshot(
            bytemuck::allocation::cast_vec(vec![255u8; 16 * 16 * 4]),
            16,
            16,
        );
        // Toggle is off by default.
        let mut out = Vec::new();
        let _ = t.run_full_resolution(&mut out);
        assert!(t.take_pending_islands().is_empty());
    }

    #[test]
    fn take_pending_islands_is_one_shot() {
        // Seed pending_islands by hand via the extraction algorithm —
        // we can't reach the field through public API except via take,
        // and an end-to-end test that exercises the pipeline depends
        // on what `chroma::segment` picks as background for a contrived
        // input. This test isolates the take semantics.
        let rgba: Vec<u8> = (0..16 * 16).flat_map(|_| [255u8, 255, 255, 255]).collect();
        let mut scratch = BgRemovalScratch::default();
        let mut islands_out = Vec::new();
        islands::extract(&rgba, 16, 16, 1, &mut scratch, &mut islands_out);
        // Sanity: single opaque block ⇒ exactly one island.
        assert_eq!(islands_out.len(), 1);

        // Splice the pre-computed islands in as if `run_full_resolution`
        // had populated them. (Test-only field access; the production
        // path is the `if self.params.separate_islands` branch in
        // `run_full_resolution`.)
        let mut t = BgRemovalTool {
            pending_islands: islands_out,
            ..BgRemovalTool::default()
        };
        let drained = t.take_pending_islands();
        assert_eq!(drained.len(), 1, "first drain returns the queue");
        assert!(
            t.take_pending_islands().is_empty(),
            "second drain is empty (one-shot)"
        );
    }

    #[test]
    fn run_full_resolution_works_after_per_sprite_source_swap() {
        // Mirrors the shell's multi-Apply drain pattern: one
        // BgRemovalTool instance bakes N sprites in sequence via
        // set_source_snapshot → run_full_resolution per entity. Each
        // bake must reflect the CURRENT snapshot, not leak state
        // (source_w/h, scratch buffer dims) from the prior sprite.
        // Regression cover (§12.6 / §12.9 UI_Bugs + Agent D gap).
        let mut t = BgRemovalTool::default();

        // Sprite 1: 8×8 red.
        let mut buf1: Vec<u8> = Vec::with_capacity(8 * 8 * 4);
        for _ in 0..(8 * 8) {
            buf1.extend_from_slice(&[200u8, 30, 30, 255]);
        }
        t.set_source_snapshot(bytemuck::allocation::cast_vec(buf1), 8, 8);
        let mut out1 = Vec::new();
        let (w1, h1) = t.run_full_resolution(&mut out1);
        assert_eq!((w1, h1), (8, 8));
        assert_eq!(out1.len(), 8 * 8 * 4);

        // Sprite 2: different dims + colour. Must re-bake against the
        // fresh snapshot, not reuse out1's dims.
        let mut buf2: Vec<u8> = Vec::with_capacity(12 * 5 * 4);
        for _ in 0..(12 * 5) {
            buf2.extend_from_slice(&[30u8, 200, 50, 255]);
        }
        t.set_source_snapshot(bytemuck::allocation::cast_vec(buf2), 12, 5);
        let mut out2 = Vec::new();
        let (w2, h2) = t.run_full_resolution(&mut out2);
        assert_eq!((w2, h2), (12, 5), "per-sprite source swap leaked dims");
        assert_eq!(out2.len(), 12 * 5 * 4);
    }
}
