//! UI projection + edit dispatch for [`BgRemovalTool`].
//!
//! [`BgRemovalTool::ui_snapshot`] projects the live full-scale params
//! into the normalized snapshot the typed `ph2d-panel-bgremoval`
//! paints; [`BgRemovalTool::apply_ui_edit`] is the inverse — it maps
//! one panel-originated edit back onto the params + transient state.
//! These are the single home for the normalized ↔ full-scale mapping
//! (ADR-0040 TG-B).

use super::BgRemovalTool;
use crate::params::{
    BRUSH_SIZE_FULL_SCALE, BgRemovalUiEdit, BgRemovalUiSnapshot, FEATHER_FULL_SCALE,
    GROW_FULL_SCALE, MIN_ISLAND_PIXELS_FULL_SCALE, REFINE_RADIUS_FULL_SCALE, TOLERANCE_FULL_SCALE,
};

impl BgRemovalTool {
    /// Project the current full-scale params into the normalized
    /// snapshot the typed `ph2d-panel-bgremoval` paints. Published by
    /// the host once per frame while the tool is active (forward of
    /// [`Self::apply_ui_edit`]).
    pub fn ui_snapshot(&self) -> BgRemovalUiSnapshot {
        BgRemovalUiSnapshot {
            tolerance01: (self.params.chroma.tolerance / TOLERANCE_FULL_SCALE).clamp(0.0, 1.0),
            feather01: (self.params.chroma.feather / FEATHER_FULL_SCALE).clamp(0.0, 1.0),
            refine01: (self.params.refinement.radius as f32 / REFINE_RADIUS_FULL_SCALE)
                .clamp(0.0, 1.0),
            grow01: (self.params.grow_px / (2.0 * GROW_FULL_SCALE) + 0.5).clamp(0.0, 1.0),
            extra_colors: self.params.extra_bg_colors.clone(),
            eyedropper_armed: self.eyedropper_armed,
            protect_brush_armed: self.protect_brush_armed,
            has_protect_mask: self.has_protect_mask(),
            brush_size01: (self.brush_radius / BRUSH_SIZE_FULL_SCALE).clamp(0.0, 1.0),
            falloff: self.falloff,
            show_mask: self.show_mask,
            separate_islands: self.params.separate_islands,
            min_island_pixels01: ((self.params.min_island_pixels.saturating_sub(1)) as f32
                / (MIN_ISLAND_PIXELS_FULL_SCALE - 1.0))
                .clamp(0.0, 1.0),
            auto_protect_subject: self.params.auto_protect_subject,
            add_area_armed: self.add_area_armed,
            has_force_remove_mask: self.has_force_remove_mask(),
        }
    }

    /// Apply one panel-originated edit (normalized slider value / mode /
    /// Apply) against the live params. Re-runs the thumbnail preview when
    /// a param actually changed and a source snapshot is loaded. `Apply`
    /// arms the pending-apply flag the host drains via
    /// [`Self::take_pending_apply`]. Inverse of [`Self::ui_snapshot`].
    pub fn apply_ui_edit(&mut self, edit: BgRemovalUiEdit) {
        let mut changed = false;
        match edit {
            BgRemovalUiEdit::Tolerance(v) => {
                self.params.chroma.tolerance = v.clamp(0.0, 1.0) * TOLERANCE_FULL_SCALE;
                // "Add area" mask tracks Tolerance — re-flood from
                // every seed so a slider drag widens / narrows the
                // destructive region exactly like the basal chroma
                // backend widens / narrows extras' reach.
                if !self.add_area_seeds.is_empty() {
                    self.regenerate_force_remove_mask();
                }
                changed = true;
            }
            BgRemovalUiEdit::Feather(v) => {
                let v = v.clamp(0.0, 1.0);
                // Drives BOTH feather paths so the slider is never a
                // no-op regardless of Refine:
                //   • Refine == 0 (compose soft-band path): widens the
                //     `[tol, tol+feather]` ΔE transition band.
                //   • Refine  > 0 (guided-filter path, the default):
                //     maps to the filter's regularisation ε on a log
                //     scale — the canonical "guided feathering" control
                //     (He et al. 2013). Larger ε ⇒ the matte ignores
                //     fine guide edges and blurs softer; smaller ε ⇒
                //     edges hug the luma guide tightly. Range 1e-6
                //     (sharp) … 1e-1 (very soft).
                self.params.chroma.feather = v * FEATHER_FULL_SCALE;
                self.params.refinement.epsilon = 1.0e-6 * 10.0_f32.powf(5.0 * v);
                // "Add area" mask tracks Feather (it sets the outer
                // edge of the soft band) — same retroactive re-flood
                // pattern as Tolerance above.
                if !self.add_area_seeds.is_empty() {
                    self.regenerate_force_remove_mask();
                }
                changed = true;
            }
            BgRemovalUiEdit::Refine(v) => {
                self.params.refinement.radius =
                    (v.clamp(0.0, 1.0) * REFINE_RADIUS_FULL_SCALE).round() as u32;
                changed = true;
            }
            BgRemovalUiEdit::Grow(v) => {
                // Bipolar: 0.5 = neutral, maps to ∓GROW_FULL_SCALE px.
                self.params.grow_px = (v.clamp(0.0, 1.0) - 0.5) * 2.0 * GROW_FULL_SCALE;
                changed = true;
            }
            BgRemovalUiEdit::ToggleEyedropper => {
                // Audit T1.6 E1-3 fix: route through `set_eyedropper_armed`
                // so the 3-way mutex (eyedropper / protect_brush /
                // add_area) is enforced — earlier direct-flip path
                // allowed both eyedropper AND add_area to stay armed
                // simultaneously, with eyedropper silently winning the
                // input_dispatch race because `try_eyedropper_sample`
                // runs before `try_add_area_click`. Setter also keeps
                // snapshot.add_area_armed and the panel button accent in
                // sync. Sampling does change params (via
                // `add_extra_color`) → preview rerun gated inside
                // setter.
                self.set_eyedropper_armed(!self.eyedropper_armed);
            }
            BgRemovalUiEdit::RemoveExtraColor(idx) => {
                // `remove_extra_color` already reruns the preview itself.
                self.remove_extra_color(idx);
            }
            BgRemovalUiEdit::ToggleProtectBrush => {
                // Flip the armed state; arming disarms the eyedropper
                // (`set_protect_armed` enforces the mutual exclusion). No
                // preview rerun — arming changes no params; painting does.
                self.set_protect_armed(!self.protect_brush_armed);
            }
            BgRemovalUiEdit::ClearProtectMask => {
                // `clear_protect_mask` reruns the preview itself.
                self.clear_protect_mask();
            }
            BgRemovalUiEdit::BrushSize(v) => {
                // Brush radius only affects future dabs — no matte rerun.
                self.brush_radius = v.clamp(0.0, 1.0) * BRUSH_SIZE_FULL_SCALE;
            }
            BgRemovalUiEdit::SetFalloff(f) => {
                // Falloff only affects future dabs — no matte rerun.
                self.falloff = f;
            }
            BgRemovalUiEdit::ToggleShowMask => {
                // Overlay-visibility only — no params, no matte rerun.
                self.show_mask = !self.show_mask;
            }
            BgRemovalUiEdit::ToggleSeparateIslands => {
                // Post-process gate only — the matte itself is unchanged
                // by this toggle, so no preview rerun. Effect lands at
                // Apply time (run_full_resolution runs CCL on the baked
                // output when the flag is on).
                self.params.separate_islands = !self.params.separate_islands;
            }
            BgRemovalUiEdit::ToggleAutoProtectSubject => {
                // Edge-aware silhouette upgrade. Affects every pipeline
                // run, so the matte rebuilds on the next preview tick.
                self.params.auto_protect_subject = !self.params.auto_protect_subject;
                // Turning Detect Subject OFF hides the "Add area"
                // button (the panel slot reverts to "Pick Colors").
                // Auto-disarm + clear the force-remove mask so a stale
                // armed state can't eat clicks intended for the
                // eyedropper.
                if !self.params.auto_protect_subject {
                    self.add_area_armed = false;
                    self.force_remove_mask.clear();
                    self.force_remove_mask_w = 0;
                    self.force_remove_mask_h = 0;
                    self.add_area_seeds.clear();
                }
                changed = true;
            }
            BgRemovalUiEdit::ToggleAddArea => {
                // Flip the armed state; arming disarms the eyedropper +
                // protect brush (`set_add_area_armed` enforces the mutex).
                self.set_add_area_armed(!self.add_area_armed);
            }
            BgRemovalUiEdit::ClearAddedAreas => {
                // `clear_force_remove_mask` reruns the preview itself.
                self.clear_force_remove_mask();
            }
            BgRemovalUiEdit::SetMinIslandPixels(v) => {
                // Linear normalized → integer pixel count in
                // [1, MIN_ISLAND_PIXELS_FULL_SCALE]. Like ToggleSeparateIslands,
                // this only matters at Apply time — no preview rerun.
                let v = v.clamp(0.0, 1.0);
                let scaled = (v * (MIN_ISLAND_PIXELS_FULL_SCALE - 1.0)).round() as u32 + 1;
                self.params.min_island_pixels =
                    scaled.clamp(1, MIN_ISLAND_PIXELS_FULL_SCALE as u32);
            }
            BgRemovalUiEdit::Apply => {
                self.pending_apply = true;
                // A commit ends the picking session — disarm so a stale
                // eyedropper / protect brush / add-area selector doesn't
                // keep eating canvas clicks on the freshly baked sprite.
                self.eyedropper_armed = false;
                self.protect_brush_armed = false;
                self.protect_painting = false;
                self.protect_erase_mode = false;
                self.add_area_armed = false;
            }
            BgRemovalUiEdit::ResetAll => {
                // Snap params back to defaults AND wipe the painted
                // protection mask + force-remove mask + extra-bg picks
                // (those are part of the per-session edit, not durable
                // state). Disarm every armed gesture so the next
                // interaction starts clean. The `on_activate` hook
                // calls this too, so a reopened panel always starts
                // from a known clean slate.
                self.params = crate::params::BgRemovalParams::default();
                self.protect_mask.fill(0);
                self.force_remove_mask.clear();
                self.force_remove_mask_w = 0;
                self.force_remove_mask_h = 0;
                self.add_area_seeds.clear();
                self.eyedropper_armed = false;
                self.protect_brush_armed = false;
                self.protect_painting = false;
                self.protect_erase_mode = false;
                self.add_area_armed = false;
                // Stage the panel-store repopulation (shell drains via
                // `take_pending_panel_reset`).
                self.pending_panel_reset = true;
            }
        }
        if changed && self.has_source() {
            self.rerun_preview();
        }
        // Every UI edit marks the shell-side canvas-preview cache stale —
        // parity with the pre-ADR-0040-TG-B `!bgremoval_ui_edits.is_empty()`
        // gate. Some variants (`BrushSize` / `SetFalloff` / `Apply` /
        // toggles) don't change the matte itself, but the previous code
        // already invalidated the canvas preview for them; preserved here.
        self.params_dirty = true;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    // `on_activate` is a `Tool` trait method exercised by the
    // panel-repopulate regression test below.
    use ph2d_editor_core::tool::Tool;

    #[test]
    fn ui_snapshot_reflects_default_params() {
        let t = BgRemovalTool::default();
        let s = t.ui_snapshot();
        // Tuned defaults (Enio 2026-05-20): tolerance 0.6, feather 0.9,
        // refine 0.01, grow chip −0.10 ⇒ grow01 0.45.
        assert!((s.tolerance01 - 0.6).abs() < 1e-5);
        assert!((s.feather01 - 0.9).abs() < 1e-5);
        assert!((s.refine01 - 0.01).abs() < 1e-5);
        assert!((s.grow01 - 0.45).abs() < 1e-5);
    }

    #[test]
    fn grow_edit_maps_bipolar_and_round_trips() {
        let mut t = BgRemovalTool::default();
        // Centre = neutral.
        t.apply_ui_edit(BgRemovalUiEdit::Grow(0.5));
        assert!(t.params.grow_px.abs() < 1e-5);
        // Full shrink (0.0) → −GROW_FULL_SCALE px; full grow (1.0) → +.
        t.apply_ui_edit(BgRemovalUiEdit::Grow(0.0));
        assert!((t.params.grow_px + GROW_FULL_SCALE).abs() < 1e-5);
        assert!(t.ui_snapshot().grow01.abs() < 1e-5);
        t.apply_ui_edit(BgRemovalUiEdit::Grow(1.0));
        assert!((t.params.grow_px - GROW_FULL_SCALE).abs() < 1e-5);
        assert!((t.ui_snapshot().grow01 - 1.0).abs() < 1e-5);
    }

    #[test]
    fn apply_ui_edit_round_trips_through_ui_snapshot() {
        let mut t = BgRemovalTool::default();
        t.apply_ui_edit(BgRemovalUiEdit::Tolerance(0.5));
        t.apply_ui_edit(BgRemovalUiEdit::Feather(0.25));
        t.apply_ui_edit(BgRemovalUiEdit::Refine(0.8));
        let s = t.ui_snapshot();
        assert!((s.tolerance01 - 0.5).abs() < 1e-5);
        assert!((s.feather01 - 0.25).abs() < 1e-5);
        // Refine maps through an integer radius (0.8*100=80 → 80/100).
        assert!((s.refine01 - 0.8).abs() < 1e-2);
        // Full-scale values land where the slider maps expect.
        assert!((t.params.chroma.tolerance - 0.15).abs() < 1e-5);
        assert!((t.params.chroma.feather - 0.05).abs() < 1e-5);
        assert_eq!(t.params.refinement.radius, 80);
    }

    #[test]
    fn apply_ui_edit_clamps_out_of_range() {
        let mut t = BgRemovalTool::default();
        t.apply_ui_edit(BgRemovalUiEdit::Tolerance(2.0));
        assert!((t.ui_snapshot().tolerance01 - 1.0).abs() < 1e-5);
        t.apply_ui_edit(BgRemovalUiEdit::Tolerance(-1.0));
        assert!(t.ui_snapshot().tolerance01.abs() < 1e-5);
    }

    #[test]
    fn apply_ui_edit_apply_arms_pending() {
        let mut t = BgRemovalTool::default();
        assert!(!t.take_pending_apply());
        t.apply_ui_edit(BgRemovalUiEdit::Apply);
        assert!(t.take_pending_apply());
        assert!(!t.take_pending_apply());
    }

    #[test]
    fn toggle_separate_islands_flips_param() {
        let mut t = BgRemovalTool::default();
        assert!(!t.params.separate_islands);
        assert!(!t.ui_snapshot().separate_islands);
        t.apply_ui_edit(BgRemovalUiEdit::ToggleSeparateIslands);
        assert!(t.params.separate_islands);
        assert!(t.ui_snapshot().separate_islands);
        t.apply_ui_edit(BgRemovalUiEdit::ToggleSeparateIslands);
        assert!(!t.params.separate_islands);
    }

    #[test]
    fn set_min_island_pixels_maps_normalized_to_full_scale() {
        let mut t = BgRemovalTool::default();
        // 0.0 → 1 pixel (the minimum useful filter).
        t.apply_ui_edit(BgRemovalUiEdit::SetMinIslandPixels(0.0));
        assert_eq!(t.params.min_island_pixels, 1);
        // 1.0 → full-scale ceiling.
        t.apply_ui_edit(BgRemovalUiEdit::SetMinIslandPixels(1.0));
        assert_eq!(
            t.params.min_island_pixels,
            MIN_ISLAND_PIXELS_FULL_SCALE as u32
        );
        // Out-of-range clamps.
        t.apply_ui_edit(BgRemovalUiEdit::SetMinIslandPixels(2.0));
        assert_eq!(
            t.params.min_island_pixels,
            MIN_ISLAND_PIXELS_FULL_SCALE as u32
        );
    }

    #[test]
    fn on_activate_resets_params_and_arms_panel_repopulate() {
        // Regression cover (§12.3 / §12.4 UI_Bugs): `on_activate` must
        // route through `apply_ui_edit::ResetAll` so (a) params snap to
        // defaults AND (b) `pending_panel_reset` arms so the shell
        // bridge re-runs `Panel::populate(store)` and the slider knobs
        // visually snap back to defaults.
        let default_snap = BgRemovalUiSnapshot::default();
        let dirty_tolerance01 = if (default_snap.tolerance01 - 0.1).abs() < 1e-3 {
            0.9_f32
        } else {
            0.1_f32
        };
        let mut t = BgRemovalTool::default();
        t.apply_ui_edit(BgRemovalUiEdit::Tolerance(dirty_tolerance01));
        assert!(
            (t.ui_snapshot().tolerance01 - default_snap.tolerance01).abs() > 0.01,
            "test setup must actually dirty tolerance01"
        );
        // Drain any stray reset flag first.
        let _ = t.take_pending_panel_reset();

        t.on_activate();

        let after = t.ui_snapshot();
        assert!(
            (after.tolerance01 - default_snap.tolerance01).abs() < 1e-5,
            "on_activate must restore default tolerance01 (got {} expected {})",
            after.tolerance01,
            default_snap.tolerance01,
        );
        assert!(
            t.take_pending_panel_reset(),
            "on_activate must arm pending_panel_reset so the shell repopulates"
        );
    }
}
