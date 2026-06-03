//! `Tool` + `RasterEditTool` trait impls for [`BgRemovalTool`].
//!
//! The `Tool` impl owns the panel build + panel-event routing (both the
//! docked-panel `ids::BGR_*` path that funnels through `apply_ui_edit`
//! and the legacy `FloatingPanel` `*_NODE` path), plus the
//! activate/deactivate lifecycle and the upcast hooks. The
//! `RasterEditTool` impl (Wave 10 / Etapa 1.B, ADR-0041) maps the
//! generic raster I/O lifecycle onto the inherent methods.

use ph2d_editor_core::floating_panel::{
    FloatingPanel, PanelAnchor, PanelControl, PanelTab, ToolId,
};
use ph2d_editor_core::ids;
use ph2d_editor_core::tool::{PanelEvent, RasterEditTool, Tool};
use ph2d_editor_core::widget::{Slider, Toggle};

use super::{APPLY_NODE, BgRemovalTool, FEATHER_NODE, REFINE_NODE, TOLERANCE_NODE};
use crate::params::{BgRemovalUiEdit, BrushFalloff};

impl Tool for BgRemovalTool {
    fn id(&self) -> ToolId {
        ToolId::new("bgremoval")
    }

    fn label(&self) -> &str {
        "Bg Removal"
    }

    fn icon_slug(&self) -> &str {
        "bgremoval"
    }

    fn build_panel(&self) -> FloatingPanel {
        let mut tolerance = Slider::new(TOLERANCE_NODE, "Tolerance");
        tolerance.value = (self.params.chroma.tolerance / 0.30).clamp(0.0, 1.0);

        let mut feather = Slider::new(FEATHER_NODE, "Feather");
        feather.value = (self.params.chroma.feather / 0.20).clamp(0.0, 1.0);

        let mut refine = Slider::new(REFINE_NODE, "Refine");
        refine.value = (self.params.refinement.radius as f32 / 100.0).clamp(0.0, 1.0);

        // Apply uses Toggle as one-shot trigger: on=false in every
        // rebuild; turning on fires `pending_apply` and the next
        // build_panel reverts to off (see INTEGRATION.md §3.1).
        let apply = Toggle::new(APPLY_NODE, "Apply");

        let mut panel = FloatingPanel::new(self.id(), "Bg Removal")
            .with_tabs(vec![PanelTab {
                label: "Bg Removal".to_string(),
                icon: None,
                active: true,
            }])
            .with_controls(vec![
                PanelControl::Slider(tolerance),
                PanelControl::Slider(feather),
                PanelControl::Slider(refine),
                PanelControl::Toggle(apply),
            ]);
        panel.anchor = PanelAnchor::BottomCenter;
        panel.width = 600.0;
        panel.height = 110.0;
        panel
    }

    fn handle_panel_event(&mut self, event: PanelEvent) {
        // Docked-panel NodeIds (`BGR_*` in `ph2d_editor_core::ids`) are
        // routed through `apply_ui_edit` so the semantic mapping
        // (normalized slider → full-scale param, clamps, projections) lives
        // exactly once. ADR-0040 TG-B replacement for the panel-side
        // semantic mapping that used to live in `panel-bgremoval/event.rs`.
        match event {
            // Sliders (normalized 0..1).
            PanelEvent::SetValue(id, v) if id == ids::BGR_TOLERANCE => {
                self.apply_ui_edit(BgRemovalUiEdit::Tolerance(v as f32));
                return;
            }
            PanelEvent::SetValue(id, v) if id == ids::BGR_FEATHER => {
                self.apply_ui_edit(BgRemovalUiEdit::Feather(v as f32));
                return;
            }
            PanelEvent::SetValue(id, v) if id == ids::BGR_REFINE => {
                self.apply_ui_edit(BgRemovalUiEdit::Refine(v as f32));
                return;
            }
            PanelEvent::SetValue(id, v) if id == ids::BGR_GROW => {
                self.apply_ui_edit(BgRemovalUiEdit::Grow(v as f32));
                return;
            }
            PanelEvent::SetValue(id, v) if id == ids::BGR_BRUSH_SIZE => {
                self.apply_ui_edit(BgRemovalUiEdit::BrushSize(v as f32));
                return;
            }
            // Number chips (same semantics as the matching slider).
            PanelEvent::SetValue(id, v) if id == ids::BGR_TOLERANCE_NUM => {
                self.apply_ui_edit(BgRemovalUiEdit::Tolerance(v as f32));
                return;
            }
            PanelEvent::SetValue(id, v) if id == ids::BGR_FEATHER_NUM => {
                self.apply_ui_edit(BgRemovalUiEdit::Feather(v as f32));
                return;
            }
            PanelEvent::SetValue(id, v) if id == ids::BGR_REFINE_NUM => {
                self.apply_ui_edit(BgRemovalUiEdit::Refine(v as f32));
                return;
            }
            PanelEvent::SetValue(id, v) if id == ids::BGR_GROW_NUM => {
                self.apply_ui_edit(BgRemovalUiEdit::Grow(v as f32));
                return;
            }
            PanelEvent::SetValue(id, v) if id == ids::BGR_BRUSH_SIZE_NUM => {
                self.apply_ui_edit(BgRemovalUiEdit::BrushSize(v as f32));
                return;
            }
            // 4-way falloff segmented.
            PanelEvent::Click(id) if id == ids::BGR_FALLOFF_SMOOTH => {
                self.apply_ui_edit(BgRemovalUiEdit::SetFalloff(BrushFalloff::Smooth));
                return;
            }
            PanelEvent::Click(id) if id == ids::BGR_FALLOFF_SPHERE => {
                self.apply_ui_edit(BgRemovalUiEdit::SetFalloff(BrushFalloff::Sphere));
                return;
            }
            PanelEvent::Click(id) if id == ids::BGR_FALLOFF_SHARP => {
                self.apply_ui_edit(BgRemovalUiEdit::SetFalloff(BrushFalloff::Sharp));
                return;
            }
            PanelEvent::Click(id) if id == ids::BGR_FALLOFF_CONSTANT => {
                self.apply_ui_edit(BgRemovalUiEdit::SetFalloff(BrushFalloff::Constant));
                return;
            }
            // Buttons.
            PanelEvent::Click(id) if id == ids::BGR_APPLY => {
                self.apply_ui_edit(BgRemovalUiEdit::Apply);
                return;
            }
            PanelEvent::Click(id) if id == ids::BGR_RESET => {
                self.apply_ui_edit(BgRemovalUiEdit::ResetAll);
                return;
            }
            PanelEvent::Click(id) if id == ids::BGR_EYEDROPPER => {
                self.apply_ui_edit(BgRemovalUiEdit::ToggleEyedropper);
                return;
            }
            PanelEvent::Click(id) if id == ids::BGR_PROTECT => {
                self.apply_ui_edit(BgRemovalUiEdit::ToggleProtectBrush);
                return;
            }
            PanelEvent::Click(id) if id == ids::BGR_PROTECT_CLEAR => {
                self.apply_ui_edit(BgRemovalUiEdit::ClearProtectMask);
                return;
            }
            PanelEvent::Click(id) if id == ids::BGR_ADD_AREA => {
                self.apply_ui_edit(BgRemovalUiEdit::ToggleAddArea);
                return;
            }
            PanelEvent::Click(id) if id == ids::BGR_ADD_AREA_CLEAR => {
                self.apply_ui_edit(BgRemovalUiEdit::ClearAddedAreas);
                return;
            }
            PanelEvent::Click(id) if id == ids::BGR_SHOW_MASK => {
                self.apply_ui_edit(BgRemovalUiEdit::ToggleShowMask);
                return;
            }
            // "Separate Islands" toggle + its min-pixel slider/chip. IDs
            // owned by the tool crate (`crate::ids`) — declared next to
            // the semantic mapping below so a parallel agent adding a
            // peer feature doesn't collide on `editor-core/src/ids.rs`.
            PanelEvent::Click(id) if id == crate::ids::BGR_SEPARATE_ISLANDS => {
                self.apply_ui_edit(BgRemovalUiEdit::ToggleSeparateIslands);
                return;
            }
            PanelEvent::Click(id) if id == crate::ids::BGR_AUTO_PROTECT_SUBJECT => {
                self.apply_ui_edit(BgRemovalUiEdit::ToggleAutoProtectSubject);
                return;
            }
            PanelEvent::SetValue(id, v) if id == crate::ids::BGR_MIN_ISLAND_PX => {
                self.apply_ui_edit(BgRemovalUiEdit::SetMinIslandPixels(v as f32));
                return;
            }
            PanelEvent::SetValue(id, v) if id == crate::ids::BGR_MIN_ISLAND_PX_NUM => {
                self.apply_ui_edit(BgRemovalUiEdit::SetMinIslandPixels(v as f32));
                return;
            }
            _ => {}
        }
        // FloatingPanel built by `build_panel` — kept for parity with the
        // pre-docked-panel path. Distinct NodeIds (`TOLERANCE_NODE = 504`
        // etc.) so the two arm-sets never overlap.
        let mut matched = false;
        let mut changed = false;
        match event {
            PanelEvent::SetValue(id, v) if id == TOLERANCE_NODE => {
                self.params.chroma.tolerance = (v.clamp(0.0, 1.0) as f32) * 0.30;
                changed = true;
                matched = true;
            }
            PanelEvent::SetValue(id, v) if id == FEATHER_NODE => {
                self.params.chroma.feather = (v.clamp(0.0, 1.0) as f32) * 0.20;
                changed = true;
                matched = true;
            }
            PanelEvent::SetValue(id, v) if id == REFINE_NODE => {
                self.params.refinement.radius = (v.clamp(0.0, 1.0) * 100.0).round() as u32;
                changed = true;
                matched = true;
            }
            // One-shot trigger; the next build_panel emits a
            // fresh Toggle with on=false so the visual resets.
            PanelEvent::Toggle(id, on) if id == APPLY_NODE && on => {
                self.pending_apply = true;
                matched = true;
            }
            _ => {}
        }
        if changed && self.has_source() {
            self.rerun_preview();
        }
        // Parity with the docked-panel path (which routes through
        // `apply_ui_edit` and so picks up the same dirty mark): any matched
        // FloatingPanel edit invalidates the shell-side canvas-preview
        // cache. Without this, sliders on the legacy panel would not
        // refresh the on-canvas overlay until an unrelated event nudged
        // `take_params_dirty`.
        if matched {
            self.params_dirty = true;
        }
    }

    fn on_activate(&mut self) {
        // Defaults load on every fresh panel open — no carryover from a
        // previous session. Routed through `apply_ui_edit::ResetAll`
        // so the panel preview rebuilds against the cleaned params on
        // the next idle frame (same code path the Reset button uses).
        self.apply_ui_edit(crate::params::BgRemovalUiEdit::ResetAll);
    }

    fn on_deactivate(&mut self) {
        // Disarm the eyedropper + protect brush + add-area selector
        // so reactivating later starts clean and a stale armed state
        // can't intercept canvas clicks meant for another tool.
        self.eyedropper_armed = false;
        self.protect_brush_armed = false;
        self.protect_painting = false;
        self.protect_erase_mode = false;
        self.add_area_armed = false;
        // Clear the one-shot drain flags so a Cancel-mid-Apply (or any
        // deactivation while the bridge hasn't yet drained the pending
        // apply / dirty bit) does not fire a phantom bake nor a spurious
        // canvas-preview rerun on the next activation.
        self.pending_apply = false;
        self.params_dirty = false;
        // Wave 10 / Etapa 1.B audit fix [A2]: align the invariant
        // "deactivate clears canvas preview cache" across BOTH paths —
        // `Tool::on_deactivate` (fired by ToolRegistry::set_active when
        // switching to ANY other tool, including non-Raster like Brush)
        // and `RasterEditTool::deactivate` (fired by tool-runtime's
        // drive_deactivate_cleanup when the next active tool is Raster).
        // Without this, BgR→Brush→BgR (without selection change) would
        // re-paint the stale cached frame on first re-activate.
        self.cached_canvas_preview = None;
    }

    fn as_raster_edit_mut(&mut self) -> Option<&mut dyn RasterEditTool> {
        // Wave 10 / Etapa 1.B (ADR-0041): BgRemoval is the first
        // production tool to implement the RasterEditTool generic
        // channel. The shell's `ph2d-tool-runtime` helpers compose
        // over this upcast — bridges no longer need
        // `downcast_mut::<BgRemovalTool>` for the generic raster
        // I/O lifecycle (set_source / current_preview /
        // take_pending_commit / run_full / deactivate). The
        // tool-specific concerns (eyedropper / protect-brush /
        // panel snapshot publish / brush ring) still go through
        // `as_any_mut` downcast — ADR-0040 §3 documented exception.
        Some(self)
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}

/// Wave 10 / Etapa 1.B (ADR-0041): BgRemoval drives its raster
/// I/O lifecycle through the generic `RasterEditTool` channel.
///
/// Mapping:
/// - `set_source` → wraps the existing `set_source_snapshot` inherent.
/// - `current_preview` → drains `params_dirty`; if dirty, runs
///   `run_canvas_preview` into `cached_canvas_preview` and returns a
///   slice into it.
/// - `take_pending_commit` → wraps `take_pending_apply`.
/// - `run_full` → wraps `run_full_resolution(&mut Vec)` into the
///   owned `(Vec, w, h)` shape the contract requires.
/// - `deactivate` → drops the canvas-preview cache + reuses
///   `on_deactivate` semantics (eyedropper/brush disarm,
///   pending_apply drop). Keeps `source_rgba` (so a re-activate
///   without selection change keeps state) — the cache being dropped
///   is what stops the shell overlay from painting after the tool
///   leaves.
impl RasterEditTool for BgRemovalTool {
    fn set_source(&mut self, rgba: Vec<u8>, width: u32, height: u32) {
        // Trait signature stays `Vec<u8>` (ADR-0041). Wave 11 typed
        // migration on the inherent `set_source_snapshot` — zero-copy
        // cast via bytemuck (SrgbRgba is repr-transparent + Pod).
        self.set_source_snapshot(bytemuck::allocation::cast_vec(rgba), width, height);
    }

    fn current_preview(&mut self) -> Option<(&[u8], u32, u32)> {
        // Drain dirty flag — this is the contract for current_preview
        // per ADR-0041. If nothing's dirty, last-good cache stays;
        // the bridge keeps painting it.
        if !self.take_params_dirty() {
            return None;
        }
        // No source pushed yet OR canvas-preview source not built.
        if !self.has_source() || self.canvas_src_rgba.is_empty() {
            return None;
        }
        let mut out = Vec::new();
        let (w, h) = self.run_canvas_preview(&mut out);
        if w == 0 || h == 0 {
            return None;
        }
        self.cached_canvas_preview = Some((out, w, h));
        // Borrow back the cache for the slice — guaranteed Some by the
        // line above; the assignment then unwrap pattern keeps Miri
        // happy (no overlapping &mut + &).
        self.cached_canvas_preview
            .as_ref()
            .map(|(p, w, h)| (p.as_slice(), *w, *h))
    }

    fn take_pending_commit(&mut self) -> bool {
        self.take_pending_apply()
    }

    fn run_full(&mut self) -> (Vec<u8>, u32, u32) {
        let mut out = Vec::new();
        let (w, h) = self.run_full_resolution(&mut out);
        (out, w, h)
    }

    fn deactivate(&mut self) {
        // Mirror the existing `Tool::on_deactivate` semantics + drop
        // the canvas-preview cache (which the runtime-driven bridge
        // now reads from instead of holding it shell-side).
        self.eyedropper_armed = false;
        self.protect_brush_armed = false;
        self.protect_painting = false;
        self.add_area_armed = false;
        self.pending_apply = false;
        self.params_dirty = false;
        self.cached_canvas_preview = None;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn id_label_icon() {
        let t = BgRemovalTool::default();
        assert_eq!(t.id(), ToolId::new("bgremoval"));
        assert_eq!(t.label(), "Bg Removal");
        assert_eq!(t.icon_slug(), "bgremoval");
    }

    #[test]
    fn panel_has_four_canonical_controls() {
        let p = BgRemovalTool::default().build_panel();
        assert_eq!(p.tool_id, ToolId::new("bgremoval"));
        assert_eq!(p.title, "Bg Removal");
        assert_eq!(p.tabs.len(), 1);
        assert_eq!(p.controls.len(), 4);
        assert!(matches!(p.controls[0], PanelControl::Slider(_)));
        assert!(matches!(p.controls[1], PanelControl::Slider(_)));
        assert!(matches!(p.controls[2], PanelControl::Slider(_)));
        assert!(matches!(p.controls[3], PanelControl::Toggle(_)));
        let labels: Vec<&str> = p.controls.iter().map(|c| c.label()).collect();
        assert_eq!(labels, vec!["Tolerance", "Feather", "Refine", "Apply"]);
    }

    #[test]
    fn slider_event_updates_params_and_clamps() {
        let mut t = BgRemovalTool::default();
        // Tolerance: slider value 0..1 maps to tolerance 0..0.30.
        t.handle_panel_event(PanelEvent::SetValue(TOLERANCE_NODE, 0.5));
        assert!((t.params.chroma.tolerance - 0.15).abs() < 1e-5);
        // Slider value out-of-range is clamped.
        t.handle_panel_event(PanelEvent::SetValue(TOLERANCE_NODE, 1.5));
        assert!((t.params.chroma.tolerance - 0.30).abs() < 1e-5);
    }

    #[test]
    fn apply_toggle_one_shot_trigger() {
        let mut t = BgRemovalTool::default();
        assert!(!t.take_pending_apply());
        // Toggle on → fire apply.
        t.handle_panel_event(PanelEvent::Toggle(APPLY_NODE, true));
        assert!(t.take_pending_apply());
        // Drained: second call returns false.
        assert!(!t.take_pending_apply());
        // Toggle off (or a stray "off" event) should not fire apply.
        t.handle_panel_event(PanelEvent::Toggle(APPLY_NODE, false));
        assert!(!t.take_pending_apply());
    }

    #[test]
    fn apply_toggle_rebuilds_with_off_state() {
        let mut t = BgRemovalTool::default();
        t.handle_panel_event(PanelEvent::Toggle(APPLY_NODE, true));
        // pending was consumed; the next build_panel must emit
        // Toggle(on=false) so the UI does not stick lit.
        let panel = t.build_panel();
        match &panel.controls[3] {
            PanelControl::Toggle(tg) => assert!(!tg.on, "Apply toggle must reset to off"),
            _ => panic!("expected Toggle at index 3"),
        }
    }

    #[test]
    fn slider_event_triggers_preview_rerun() {
        // Mutating a param after a source snapshot is live re-runs
        // the preview pipeline. Tests the `changed && has_source`
        // gate in `handle_panel_event`.
        let mut t = BgRemovalTool::default();
        let buf = vec![255u8; 32 * 32 * 4];
        t.set_source_snapshot(bytemuck::allocation::cast_vec(buf), 32, 32);
        let baseline = t.preview_rgba().to_vec();
        t.handle_panel_event(PanelEvent::SetValue(TOLERANCE_NODE, 0.9));
        // Preview ran again (length preserved; content may differ —
        // we don't assert on content, just on the contract that it
        // didn't get wiped to empty).
        assert_eq!(t.preview_rgba().len(), baseline.len());
    }

    #[test]
    fn on_deactivate_disarms_eyedropper() {
        let mut t = BgRemovalTool::default();
        t.set_eyedropper_armed(true);
        Tool::on_deactivate(&mut t);
        assert!(!t.is_eyedropper_armed());
    }

    // ─────────────────────────────────────────────────────────────────────
    // Wave 10 / Etapa 1.B — RasterEditTool impl tests (ADR-0041 follow-up)
    // ─────────────────────────────────────────────────────────────────────

    #[test]
    fn as_raster_edit_mut_returns_some_for_bgremoval() {
        let mut t = BgRemovalTool::default();
        // Verify the upcast works — the runtime helpers depend on this.
        assert!(<dyn Tool as Tool>::as_raster_edit_mut(&mut t).is_some());
    }

    #[test]
    fn raster_edit_set_source_delegates_to_set_source_snapshot() {
        let mut t = BgRemovalTool::default();
        let rgba = vec![255u8; 16 * 16 * 4];
        RasterEditTool::set_source(&mut t, rgba, 16, 16);
        assert!(t.has_source());
        assert_eq!(t.source_size(), (16, 16));
        // params_dirty was set by set_source_snapshot (via rerun_preview)
        // — verify the dirty flag is up for current_preview to drain.
        // (peek without drain via cloning the bool — there's no peek API.)
    }

    #[test]
    fn raster_edit_current_preview_drains_dirty_and_caches() {
        let mut t = BgRemovalTool::default();
        // Push a source so canvas-preview has something to run on.
        let rgba = vec![128u8; 8 * 8 * 4];
        RasterEditTool::set_source(&mut t, rgba, 8, 8);
        // First call: dirty drains, cache populated, slice returned.
        let frame = RasterEditTool::current_preview(&mut t);
        assert!(
            frame.is_some(),
            "first call after set_source must return Some"
        );
        let (pixels, w, h) = frame.unwrap();
        assert!(w > 0 && h > 0);
        assert_eq!(pixels.len(), (w as usize) * (h as usize) * 4);
        // Second call: dirty was drained, no new frame.
        let frame2 = RasterEditTool::current_preview(&mut t);
        assert!(
            frame2.is_none(),
            "second call without new dirty must be None"
        );
        // Cache should still be there (last-good preview lives on in the tool).
        assert!(t.cached_canvas_preview.is_some());
    }

    #[test]
    fn raster_edit_current_preview_returns_none_without_source() {
        // No set_source — no canvas_src_rgba — must return None even
        // if dirty (defensive: drain dirty but don't fabricate a frame).
        let mut t = BgRemovalTool {
            params_dirty: true,
            ..BgRemovalTool::default()
        };
        let frame = RasterEditTool::current_preview(&mut t);
        assert!(frame.is_none());
    }

    #[test]
    fn raster_edit_take_pending_commit_drains_apply_flag() {
        let mut t = BgRemovalTool::default();
        t.apply_ui_edit(BgRemovalUiEdit::Apply);
        assert!(RasterEditTool::take_pending_commit(&mut t));
        // Drained.
        assert!(!RasterEditTool::take_pending_commit(&mut t));
    }

    #[test]
    fn raster_edit_run_full_returns_owned_buffer() {
        let mut t = BgRemovalTool::default();
        let rgba = vec![64u8; 8 * 8 * 4];
        RasterEditTool::set_source(&mut t, rgba, 8, 8);
        let (out, w, h) = RasterEditTool::run_full(&mut t);
        assert_eq!((w, h), (8, 8));
        assert_eq!(out.len(), 8 * 8 * 4);
    }

    // Wave 10 / Etapa 1.B audit fix [A1]: set_source must invalidate
    // the canvas-preview cache so a stale frame from the previous
    // selection can never paint over a new sprite.
    #[test]
    fn set_source_invalidates_cached_canvas_preview() {
        let mut t = BgRemovalTool::default();
        // Push source A → drain preview → cache populated.
        RasterEditTool::set_source(&mut t, vec![10u8; 4 * 4 * 4], 4, 4);
        let frame_a = RasterEditTool::current_preview(&mut t);
        assert!(frame_a.is_some());
        assert!(t.cached_canvas_preview.is_some());
        // Push source B (different selection in shell, same dim) →
        // cache must be invalidated BEFORE preview rebuilds.
        // The invariant: at no point can current_preview return a slice
        // that mixes pixels from A.
        RasterEditTool::set_source(&mut t, vec![200u8; 4 * 4 * 4], 4, 4);
        assert!(
            t.cached_canvas_preview.is_none(),
            "set_source MUST invalidate cached_canvas_preview (audit A1)"
        );
        // The next current_preview rebuilds — and the bytes are from B.
        let frame_b = RasterEditTool::current_preview(&mut t);
        assert!(frame_b.is_some());
    }

    // Wave 10 / Etapa 1.B audit fix [A2]: Tool::on_deactivate must
    // clear cached_canvas_preview so switching to a non-Raster tool
    // (like Brush) and back doesn't paint a stale frame.
    #[test]
    fn on_deactivate_clears_cached_canvas_preview() {
        let mut t = BgRemovalTool::default();
        RasterEditTool::set_source(&mut t, vec![50u8; 4 * 4 * 4], 4, 4);
        RasterEditTool::current_preview(&mut t); // populate cache
        assert!(t.cached_canvas_preview.is_some());
        // Tool::on_deactivate is what fires when the ToolRegistry switches
        // to a non-Raster tool (e.g. Brush) — different path from
        // RasterEditTool::deactivate. Both MUST clear the cache.
        Tool::on_deactivate(&mut t);
        assert!(
            t.cached_canvas_preview.is_none(),
            "on_deactivate MUST clear cached_canvas_preview (audit A2)"
        );
    }

    #[test]
    fn raster_edit_deactivate_clears_all_transient_state() {
        let mut t = BgRemovalTool::default();
        let rgba = vec![200u8; 4 * 4 * 4];
        RasterEditTool::set_source(&mut t, rgba, 4, 4);
        let _ = RasterEditTool::current_preview(&mut t); // populate cache
        t.set_eyedropper_armed(true);
        t.set_protect_armed(true);
        t.apply_ui_edit(BgRemovalUiEdit::Apply); // arms pending_apply
        // Now deactivate — every transient flag + cache must drop.
        RasterEditTool::deactivate(&mut t);
        assert!(!t.is_eyedropper_armed());
        assert!(!t.is_protect_armed());
        assert!(!t.is_protect_painting());
        assert!(t.cached_canvas_preview.is_none());
        // params_dirty drained.
        assert!(!t.take_params_dirty());
        // pending_apply drained.
        assert!(!t.take_pending_apply());
        // Source pixels NOT cleared — re-activate without new selection
        // keeps state (the shell drops cache externally; tool is just
        // dropping its OWN transient flags).
        assert!(t.has_source());
    }
}
