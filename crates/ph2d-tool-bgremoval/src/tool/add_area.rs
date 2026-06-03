//! "Add area" destructive selector + extra-bg colour list for
//! [`BgRemovalTool`].
//!
//! The "Add area" selector (Enio 2026-05-26) is a single-click
//! flood-fill from a source pixel into the force-remove mask —
//! symmetric to the eyedropper: arm → click → algorithm runs. NOT a
//! brush (no drag, no falloff). The extra-bg colour list backs the
//! eyedropper's chroma picks.

use super::BgRemovalTool;
use crate::params::MAX_EXTRA_BG_COLORS;

/// Squared RGB Euclidean distance below which two extra colours are
/// treated as duplicates (skip-on-add). `24²` ≈ a barely-perceptible
/// step; stops a click-drag from appending hundreds of near-identical
/// samples across a smooth gradient. // LITERAL-OK: dedup perceptual budget
const EXTRA_COLOR_DEDUP_DIST_SQ: i32 = 24 * 24;

impl BgRemovalTool {
    // ── "Add area" automatic selector (Enio 2026-05-26) ──
    // Single-click flood-fill from a source pixel into the force-remove
    // mask. Symmetric to the eyedropper: arm → click → algorithm runs.
    // NOT a brush — there is no drag, no falloff, no per-pixel painting
    // surface. Shown in the eyedropper-row slot when
    // `auto_protect_subject` is on (Pick Colors doesn't apply to the
    // silhouette path, so the slot is repurposed for this destructive
    // selector).

    /// Whether the "Add area" selector is armed.
    pub fn is_add_area_armed(&self) -> bool {
        self.add_area_armed
    }

    /// Set the "Add area" armed state. Arming it disarms the eyedropper
    /// AND the protect brush so the three canvas modes never fight over
    /// the same click.
    pub fn set_add_area_armed(&mut self, armed: bool) {
        self.add_area_armed = armed;
        if armed {
            self.eyedropper_armed = false;
            self.protect_brush_armed = false;
        }
    }

    /// Whether the force-remove mask currently holds any filled pixels.
    pub fn has_force_remove_mask(&self) -> bool {
        self.force_remove_mask.iter().any(|&v| v != 0)
    }

    /// Borrow the source-resolution force-remove mask: `(mask, w, h)`,
    /// one byte/pixel (`255` = forced removed). Empty slice + `(0, 0)`
    /// when nothing is filled.
    pub fn force_remove_mask_source(&self) -> (&[u8], u32, u32) {
        (
            &self.force_remove_mask,
            self.force_remove_mask_w,
            self.force_remove_mask_h,
        )
    }

    /// Single-click "Add area" seed. Called by the shell on a Primary
    /// Down inside the sprite footprint while the selector is armed.
    /// UV `(u, v)` is the clicked pixel in normalized `[0,1]` source
    /// coords (origin top-left). Pushes the source-pixel position onto
    /// [`Self::add_area_seeds`] and regenerates the force-remove mask
    /// using the SAME soft-band math as the compose path: ΔE² in
    /// Oklab space, hard removal inside `tolerance²`, lerp to zero
    /// across the `[tolerance, tolerance+feather]` band, stop outside.
    /// Connectivity is enforced by 4-connected BFS, so the destructive
    /// region is bounded to the clicked sprite area (it doesn't bleed
    /// globally the way `extra_bg_colors` does).
    pub fn flood_fill_remove_at_uv(&mut self, u: f32, v: f32) {
        if !self.has_source() {
            return;
        }
        let (w, h) = (self.source_w, self.source_h);
        if w == 0 || h == 0 {
            return;
        }
        let cx = (u.clamp(0.0, 1.0) * (w as f32 - 1.0)).round() as u32;
        let cy = (v.clamp(0.0, 1.0) * (h as f32 - 1.0)).round() as u32;
        self.add_area_seeds.push((cx, cy));
        self.regenerate_force_remove_mask();
    }

    /// Regenerate [`Self::force_remove_mask`] from
    /// [`Self::add_area_seeds`] using the current `chroma.tolerance` /
    /// `chroma.feather` slider values. Called on every click AND every
    /// time the user moves Tolerance or Feather — so the destructive
    /// area tracks the same soft-band math the basal chroma backend
    /// uses for `extra_bg_colors`.
    ///
    /// Algorithm: for each seed, flood-fill 4-connected from the seed
    /// position; per-pixel ΔE² in Oklab space against the seed's
    /// source colour decides (a) whether to expand into the pixel —
    /// reject when `de_sq >= (tol+feather)²` — and (b) the strength
    /// written into the mask: `255` inside `tol²`, lerp `255 → 0`
    /// across the soft band. Multiple seeds accumulate via `max`,
    /// so overlapping areas keep the strongest seed's strength.
    /// No-op when `add_area_seeds` is empty (mask cleared so the
    /// pipeline skips the per-pixel min-clamp pass).
    pub(crate) fn regenerate_force_remove_mask(&mut self) {
        let (w, h) = (self.source_w, self.source_h);
        let n = (w as usize) * (h as usize);
        if n == 0 {
            return;
        }
        self.force_remove_mask.clear();
        self.force_remove_mask.resize(n, 0);
        self.force_remove_mask_w = w;
        self.force_remove_mask_h = h;
        // Sync dirty so the canvas-preview cache rebuilds on the next
        // bridge tick (the matte must reflect the new mask whether the
        // seeds list changed or just the slider values).
        self.params_dirty = true;
        if self.add_area_seeds.is_empty() {
            return;
        }
        let tol = self.params.chroma.tolerance;
        let feat = self.params.chroma.feather.max(1e-6);
        let outer = tol + feat;
        // Connectivity reach (Enio 2026-05-27 "a área nova não é
        // sujeita a ajustes finais com os sliders"): the mask is now
        // a BINARY "is in region" flag (not a per-pixel strength), and
        // the pipeline (`algorithm::run_pipeline`) injects the region
        // into `scratch.mask` + `scratch.delta_e` as hard background
        // BEFORE refine + grow. That makes the destructive area
        // subject to every basal slider — Refine smooths the edge via
        // guided filter, Grow morphs it, Feather widens the connected
        // region's reach (here) — exactly like the auto-detected bg.
        //
        // The flood-fill keeps a small relaxation factor over the
        // basal soft band so thin hatching / ink barriers (ΔE just
        // past `outer`) don't cut off the connectivity prematurely;
        // the binary mask gates pipeline injection, not final alpha,
        // so the relaxation only matters for reach.
        const BRIDGE_RELAX_FACTOR: f32 = 1.5;
        let bridge_outer = outer * BRIDGE_RELAX_FACTOR;
        let bridge_outer_sq = bridge_outer * bridge_outer;
        let mut visited: Vec<bool> = vec![false; n];
        let mut queue: Vec<(u32, u32)> = Vec::new();
        for seed_idx in 0..self.add_area_seeds.len() {
            let (sx, sy) = self.add_area_seeds[seed_idx];
            if sx >= w || sy >= h {
                continue;
            }
            for v in visited.iter_mut() {
                *v = false;
            }
            queue.clear();
            let seed_pixel = (sy as usize) * (w as usize) + (sx as usize);
            let seed_base = seed_pixel * 4;
            let seed_oklab = crate::algorithm::chroma::srgb_to_oklab(
                self.source_rgba[seed_base],
                self.source_rgba[seed_base + 1],
                self.source_rgba[seed_base + 2],
            );
            queue.push((sx, sy));
            while let Some((x, y)) = queue.pop() {
                let i = (y as usize) * (w as usize) + (x as usize);
                if visited[i] {
                    continue;
                }
                visited[i] = true;
                let base = i * 4;
                let p_oklab = crate::algorithm::chroma::srgb_to_oklab(
                    self.source_rgba[base],
                    self.source_rgba[base + 1],
                    self.source_rgba[base + 2],
                );
                let de_sq = crate::algorithm::chroma::oklab_dist_sq(p_oklab, seed_oklab);
                if de_sq >= bridge_outer_sq {
                    continue;
                }
                // Binary mark — the pipeline turns this into hard bg
                // (mask=0, delta_e=0) before Refine + Grow run.
                self.force_remove_mask[i] = 255;
                if x > 0 {
                    queue.push((x - 1, y));
                }
                if x + 1 < w {
                    queue.push((x + 1, y));
                }
                if y > 0 {
                    queue.push((x, y - 1));
                }
                if y + 1 < h {
                    queue.push((x, y + 1));
                }
            }
        }
    }

    /// Wipe the entire force-remove mask AND the seed list. Reruns
    /// the preview when a source is loaded so the matte un-removes
    /// those pixels immediately.
    pub fn clear_force_remove_mask(&mut self) {
        self.force_remove_mask.clear();
        self.force_remove_mask_w = 0;
        self.force_remove_mask_h = 0;
        self.add_area_seeds.clear();
        if self.has_source() {
            self.rerun_preview();
        }
        self.params_dirty = true;
    }

    /// Borrow the current extra background colours (sRGB 8-bit).
    pub fn extra_colors(&self) -> &[[u8; 3]] {
        &self.params.extra_bg_colors
    }

    /// Append a user-picked extra background colour. No-op when the
    /// colour duplicates (exactly or within
    /// [`EXTRA_COLOR_DEDUP_DIST_SQ`]) one already stored, or when the
    /// list is already at [`MAX_EXTRA_BG_COLORS`]. Re-runs the preview
    /// when something was actually added and a source is loaded.
    pub fn add_extra_color(&mut self, rgb: [u8; 3]) {
        if self.params.extra_bg_colors.len() >= MAX_EXTRA_BG_COLORS {
            return;
        }
        let is_dup = self.params.extra_bg_colors.iter().any(|c| {
            let dr = c[0] as i32 - rgb[0] as i32;
            let dg = c[1] as i32 - rgb[1] as i32;
            let db = c[2] as i32 - rgb[2] as i32;
            dr * dr + dg * dg + db * db <= EXTRA_COLOR_DEDUP_DIST_SQ
        });
        if is_dup {
            return;
        }
        self.params.extra_bg_colors.push(rgb);
        if self.has_source() {
            self.rerun_preview();
        }
        // Eyedropper sampling mutates params.extra_bg_colors → canvas
        // preview must rebuild (previously this site did NOT invalidate
        // the shell-side cache because eyedropper dabs bypassed the bus;
        // refreshing it eagerly here closes a 1-frame staleness gap).
        self.params_dirty = true;
    }

    /// Remove the extra background colour at `idx` (bounds-checked).
    /// Re-runs the preview when the index was valid and a source is
    /// loaded.
    pub fn remove_extra_color(&mut self, idx: usize) {
        if idx >= self.params.extra_bg_colors.len() {
            return;
        }
        self.params.extra_bg_colors.remove(idx);
        if self.has_source() {
            self.rerun_preview();
        }
        // Same rationale as `add_extra_color`.
        self.params_dirty = true;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::params::BgRemovalUiEdit;

    #[test]
    fn add_extra_color_dedups_near_duplicates_and_caps() {
        let mut t = BgRemovalTool::default();
        t.add_extra_color([100, 100, 100]);
        // A colour within ~24 RGB of an existing one is skipped.
        t.add_extra_color([110, 100, 100]);
        assert_eq!(t.extra_colors().len(), 1, "near-duplicate must be skipped");
        // A clearly different colour is appended.
        t.add_extra_color([10, 200, 30]);
        assert_eq!(t.extra_colors().len(), 2);

        // Cap at MAX_EXTRA_BG_COLORS with well-separated colours.
        // Grid in (R, G) with a 64-step so every pair is ≥ 64 apart
        // on at least one channel — far beyond the dedup radius.
        let mut t2 = BgRemovalTool::default();
        for i in 0..(MAX_EXTRA_BG_COLORS + 5) {
            let r = ((i % 4) * 64) as u8;
            let g = ((i / 4) * 64) as u8;
            t2.add_extra_color([r, g, 0]);
        }
        assert_eq!(t2.extra_colors().len(), MAX_EXTRA_BG_COLORS);
    }

    #[test]
    fn remove_extra_color_removes_right_index_and_is_bounds_checked() {
        let mut t = BgRemovalTool::default();
        t.add_extra_color([200, 0, 0]);
        t.add_extra_color([0, 200, 0]);
        t.add_extra_color([0, 0, 200]);
        t.remove_extra_color(1); // remove green
        assert_eq!(t.extra_colors(), &[[200, 0, 0], [0, 0, 200]]);
        // Out-of-bounds is a no-op.
        t.remove_extra_color(99);
        assert_eq!(t.extra_colors().len(), 2);
    }

    #[test]
    fn ui_snapshot_reflects_extra_colors_and_armed() {
        let mut t = BgRemovalTool::default();
        assert!(t.ui_snapshot().extra_colors.is_empty());
        assert!(!t.ui_snapshot().eyedropper_armed);
        t.add_extra_color([1, 2, 3]);
        t.set_eyedropper_armed(true);
        let s = t.ui_snapshot();
        assert_eq!(s.extra_colors, vec![[1, 2, 3]]);
        assert!(s.eyedropper_armed);
    }

    #[test]
    fn toggle_eyedropper_edit_flips_armed() {
        let mut t = BgRemovalTool::default();
        assert!(!t.is_eyedropper_armed());
        t.apply_ui_edit(BgRemovalUiEdit::ToggleEyedropper);
        assert!(t.is_eyedropper_armed());
        t.apply_ui_edit(BgRemovalUiEdit::ToggleEyedropper);
        assert!(!t.is_eyedropper_armed());
    }

    #[test]
    fn remove_extra_color_edit_removes_index() {
        let mut t = BgRemovalTool::default();
        t.add_extra_color([200, 0, 0]);
        t.add_extra_color([0, 200, 0]);
        t.apply_ui_edit(BgRemovalUiEdit::RemoveExtraColor(0));
        assert_eq!(t.extra_colors(), &[[0, 200, 0]]);
    }

    #[test]
    fn apply_disarms_eyedropper() {
        let mut t = BgRemovalTool::default();
        t.set_eyedropper_armed(true);
        t.apply_ui_edit(BgRemovalUiEdit::Apply);
        assert!(!t.is_eyedropper_armed());
    }
}
