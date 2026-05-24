//! [`EqualizeSizesTool`] — stateful editor Tool for multi-sprite canvas
//! normalization.
//!
//! ## Cross-sprite shape
//!
//! Unlike BgRemoval / Padding (which each bake ONE active sprite), this
//! tool consumes the whole selection at Apply. The shell does the
//! `hero.gizmo.iter_selected()` walk + collects per-entity inputs +
//! calls [`EqualizeSizesTool::run_full_resolution_multi`] in one batch.
//! The tool itself holds only the *params* (target mode, fixed W/H,
//! grid unit, toggles, upscale algorithm) + the pending-apply latch.
//!
//! Following the BgRemoval/Padding pattern, the shell reaches this
//! concrete type via `as_any_mut` downcast (the public method below) —
//! `RasterEditTool` (renamed from `ImageEditTool` in ADR-0041) would be
//! the wrong shape because it assumes single-sprite (`set_source` /
//! `run_full` of one buffer), and the generic channel isn't wired for
//! cross-sprite yet (Etapa 2 of the Wave 10 perfection plan addresses
//! the 5 single-sprite tools; multi-sprite EqualizeSizes stays on the
//! downcast pattern as a documented exception, vide DIRETRIZ §3.8.3.1).

use ph2d_editor_core::floating_panel::{FloatingPanel, PanelAnchor, ToolId};
use ph2d_editor_core::tool::{PanelEvent, Tool};

use crate::algorithm::{SpriteInput, SpriteOutput, run_equalize_sizes};
use crate::ids;
use crate::params::{
    EqualizeSizesParams, EqualizeSizesUiEdit, EqualizeSizesUiSnapshot, TargetMode,
    UpscaleAlgorithm, apply_ui_edit, snapshot_from_params,
};

/// Editor Tool implementing Equalize Sizes (Stateful, sabor 3).
#[derive(Clone, Debug, Default)]
pub struct EqualizeSizesTool {
    params: EqualizeSizesParams,
    /// Set `true` when the user presses Apply; the host drains via
    /// [`Self::take_pending_apply`] each frame and runs
    /// [`Self::run_full_resolution_multi`] over the selection.
    pending_apply: bool,
    /// Set `true` by Reset / on_activate. Shell drains via
    /// `take_pending_panel_reset` and re-runs `Panel::populate` so
    /// the slider knobs / chips snap back to defaults.
    pending_panel_reset: bool,
}

impl EqualizeSizesTool {
    /// Live params — the shell reads this at Apply time alongside the
    /// per-sprite inputs.
    pub fn params(&self) -> &EqualizeSizesParams {
        &self.params
    }

    /// Project the params into the snapshot the typed panel paints.
    pub fn ui_snapshot(&self) -> EqualizeSizesUiSnapshot {
        snapshot_from_params(&self.params)
    }

    /// Apply one panel-originated edit. Routes through
    /// [`apply_ui_edit`] (the only site that clamps + commits values);
    /// the only side effect at the tool layer is flipping the
    /// pending-apply latch on `Apply`.
    pub fn apply_ui_edit(&mut self, edit: EqualizeSizesUiEdit) {
        if matches!(edit, EqualizeSizesUiEdit::ResetAll) {
            self.pending_panel_reset = true;
        }
        if apply_ui_edit(&mut self.params, edit) {
            self.pending_apply = true;
        }
    }

    pub fn take_pending_panel_reset(&mut self) -> bool {
        std::mem::take(&mut self.pending_panel_reset)
    }

    /// Drain the pending-apply latch. Returns `true` exactly once after
    /// each Apply press; the host then calls
    /// [`Self::run_full_resolution_multi`] with the live selection.
    pub fn take_pending_apply(&mut self) -> bool {
        let p = self.pending_apply;
        self.pending_apply = false;
        p
    }

    /// Run the bake at full resolution over the current selection. Pure
    /// orchestration call — the shell collects [`SpriteInput`]s from
    /// `hero.gizmo.iter_selected()` (with the active `Transform.scale`
    /// applied) and forwards the result back into each entity's texture
    /// + transform via `texture_edit::commit_edited_texture`.
    pub fn run_full_resolution_multi(&self, inputs: &[SpriteInput]) -> Vec<SpriteOutput> {
        run_equalize_sizes(inputs, &self.params)
    }
}

impl Tool for EqualizeSizesTool {
    fn id(&self) -> ToolId {
        ToolId::new("equalize_sizes")
    }

    fn label(&self) -> &str {
        "Equalize Sizes"
    }

    fn icon_slug(&self) -> &str {
        "equalize-sizes"
    }

    fn build_panel(&self) -> FloatingPanel {
        // Like the Padding tool, the typed `ph2d-panel-equalize-sizes`
        // crate paints the real UI. A minimal panel shell is still
        // returned so `Tool::build_panel` has a value to mount; it
        // carries no controls of its own.
        let mut panel = FloatingPanel::new(self.id(), "Equalize Sizes");
        panel.anchor = PanelAnchor::BottomCenter;
        panel
    }

    fn on_activate(&mut self) {
        // Defaults load on every fresh panel open (mode / W,H / grid /
        // toggles back to `EqualizeSizesParams::default()`).
        self.apply_ui_edit(EqualizeSizesUiEdit::ResetAll);
    }

    fn on_deactivate(&mut self) {
        // Drop only the pending-apply latch on deactivation.
        self.pending_apply = false;
    }

    fn handle_panel_event(&mut self, event: PanelEvent) {
        // Map docked-panel `NodeId`s to typed `EqualizeSizesUiEdit`s.
        // Every clamp/projection lives in `params::apply_ui_edit` —
        // never duplicate it here.
        match event {
            // ── Target mode (3-way radio) ────────────────────────────
            PanelEvent::Click(id) if id == ids::EQS_MODE_MAX => {
                self.apply_ui_edit(EqualizeSizesUiEdit::SetMode(TargetMode::MaxOfSelection));
            }
            PanelEvent::Click(id) if id == ids::EQS_MODE_FIXED => {
                self.apply_ui_edit(EqualizeSizesUiEdit::SetMode(TargetMode::Fixed));
            }
            PanelEvent::Click(id) if id == ids::EQS_MODE_GRID => {
                self.apply_ui_edit(EqualizeSizesUiEdit::SetMode(TargetMode::GridUnit));
            }
            // ── Fixed W/H chips (raw px, rounded) ────────────────────
            PanelEvent::SetValue(id, v) if id == ids::EQS_FIXED_W => {
                let w = (v.round().max(0.0) as u32).max(1);
                self.apply_ui_edit(EqualizeSizesUiEdit::SetFixedW(w));
            }
            PanelEvent::SetValue(id, v) if id == ids::EQS_FIXED_H => {
                let h = (v.round().max(0.0) as u32).max(1);
                self.apply_ui_edit(EqualizeSizesUiEdit::SetFixedH(h));
            }
            // ── Grid-mode controls (cell from Grid Snap, offset + arrange here) ──
            //
            // `grid_unit` is owned by the Grid Snap tool — the bridge
            // calls `apply_ui_edit(SetGridUnit(cell_px))` each frame to
            // keep `params.grid_unit` in sync. Here we only handle the
            // panel's own controls: offset chip/slider (in px) and the
            // "Arrange on Grid" toggle.
            PanelEvent::SetValue(id, v)
                if id == ids::EQS_GRID_OFFSET || id == ids::EQS_GRID_OFFSET_NUM =>
            {
                let o = v.round().max(0.0) as u32;
                self.apply_ui_edit(EqualizeSizesUiEdit::SetGridOffset(o));
            }
            PanelEvent::Click(id) if id == ids::EQS_ARRANGE_ON_GRID => {
                self.apply_ui_edit(EqualizeSizesUiEdit::ToggleArrangeOnGrid);
            }
            // ── Toggles ──────────────────────────────────────────────
            PanelEvent::Click(id) if id == ids::EQS_UPSCALE_IF_SMALLER => {
                self.apply_ui_edit(EqualizeSizesUiEdit::ToggleUpscaleIfSmaller);
            }
            PanelEvent::Click(id) if id == ids::EQS_RASTERIZE_AFTER => {
                self.apply_ui_edit(EqualizeSizesUiEdit::ToggleRasterizeAfter);
            }
            // ── Upscale algorithm (3-way radio) ──────────────────────
            PanelEvent::Click(id) if id == ids::EQS_ALG_LANCZOS => {
                self.apply_ui_edit(EqualizeSizesUiEdit::SetUpscaleAlgorithm(
                    UpscaleAlgorithm::Lanczos3,
                ));
            }
            PanelEvent::Click(id) if id == ids::EQS_ALG_NEAREST => {
                self.apply_ui_edit(EqualizeSizesUiEdit::SetUpscaleAlgorithm(
                    UpscaleAlgorithm::Nearest,
                ));
            }
            PanelEvent::Click(id) if id == ids::EQS_ALG_XBR => {
                self.apply_ui_edit(EqualizeSizesUiEdit::SetUpscaleAlgorithm(
                    UpscaleAlgorithm::Xbr,
                ));
            }
            // ── Apply (arms the latch the shell drains next frame) ───
            PanelEvent::Click(id) if id == ids::EQS_APPLY => {
                self.apply_ui_edit(EqualizeSizesUiEdit::Apply);
            }
            PanelEvent::Click(id) if id == ids::EQS_RESET => {
                self.apply_ui_edit(EqualizeSizesUiEdit::ResetAll);
            }
            // Cancel is handled at the panel layer via
            // `EditorAction::CancelActiveTool` (shell switches back to
            // the default tool). The tool itself does nothing for it.
            _ => {}
        }
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_tool_is_at_defaults_and_not_pending() {
        let t = EqualizeSizesTool::default();
        assert_eq!(*t.params(), EqualizeSizesParams::default());
        let mut t = t;
        assert!(!t.take_pending_apply());
    }

    #[test]
    fn id_label_icon() {
        let t = EqualizeSizesTool::default();
        assert_eq!(t.id(), ToolId::new("equalize_sizes"));
        assert_eq!(t.label(), "Equalize Sizes");
        assert_eq!(t.icon_slug(), "equalize-sizes");
    }

    #[test]
    fn mode_clicks_route_to_apply_ui_edit() {
        let mut t = EqualizeSizesTool::default();
        t.handle_panel_event(PanelEvent::Click(ids::EQS_MODE_FIXED));
        assert_eq!(t.params().target_mode, TargetMode::Fixed);
        t.handle_panel_event(PanelEvent::Click(ids::EQS_MODE_GRID));
        assert_eq!(t.params().target_mode, TargetMode::GridUnit);
        t.handle_panel_event(PanelEvent::Click(ids::EQS_MODE_MAX));
        assert_eq!(t.params().target_mode, TargetMode::MaxOfSelection);
    }

    #[test]
    fn fixed_chips_route_through_params_clamp() {
        let mut t = EqualizeSizesTool::default();
        t.handle_panel_event(PanelEvent::SetValue(ids::EQS_FIXED_W, 512.0));
        assert_eq!(t.params().fixed_w, 512);
        t.handle_panel_event(PanelEvent::SetValue(ids::EQS_FIXED_H, 99_999.0));
        // params::apply_ui_edit clamps to EQS_MAX_FIXED_DIM (4096).
        assert_eq!(t.params().fixed_h, crate::params::EQS_MAX_FIXED_DIM);
    }

    #[test]
    fn arrange_on_grid_toggle_flips_via_panel_click() {
        let mut t = EqualizeSizesTool::default();
        assert!(!t.params().arrange_on_grid);
        t.handle_panel_event(PanelEvent::Click(ids::EQS_ARRANGE_ON_GRID));
        assert!(t.params().arrange_on_grid);
        t.handle_panel_event(PanelEvent::Click(ids::EQS_ARRANGE_ON_GRID));
        assert!(!t.params().arrange_on_grid);
    }

    #[test]
    fn grid_offset_routes_through_panel_event() {
        let mut t = EqualizeSizesTool::default();
        // Default grid_unit is 32, so offset clamps to 0..16.
        t.handle_panel_event(PanelEvent::SetValue(ids::EQS_GRID_OFFSET, 4.0));
        assert_eq!(t.params().grid_offset, 4);
        t.handle_panel_event(PanelEvent::SetValue(ids::EQS_GRID_OFFSET_NUM, 25.0));
        assert_eq!(t.params().grid_offset, 16, "offset must clamp to cell/2");
    }

    #[test]
    fn toggles_flip_on_click() {
        let mut t = EqualizeSizesTool::default();
        assert!(!t.params().upscale_if_smaller);
        t.handle_panel_event(PanelEvent::Click(ids::EQS_UPSCALE_IF_SMALLER));
        assert!(t.params().upscale_if_smaller);

        assert!(t.params().rasterize_after);
        t.handle_panel_event(PanelEvent::Click(ids::EQS_RASTERIZE_AFTER));
        assert!(!t.params().rasterize_after);
    }

    #[test]
    fn algorithm_radio_routes() {
        let mut t = EqualizeSizesTool::default();
        t.handle_panel_event(PanelEvent::Click(ids::EQS_ALG_NEAREST));
        assert_eq!(t.params().upscale_algorithm, UpscaleAlgorithm::Nearest);
        t.handle_panel_event(PanelEvent::Click(ids::EQS_ALG_XBR));
        assert_eq!(t.params().upscale_algorithm, UpscaleAlgorithm::Xbr);
        t.handle_panel_event(PanelEvent::Click(ids::EQS_ALG_LANCZOS));
        assert_eq!(t.params().upscale_algorithm, UpscaleAlgorithm::Lanczos3);
    }

    #[test]
    fn apply_arms_pending_once_then_drains() {
        let mut t = EqualizeSizesTool::default();
        t.handle_panel_event(PanelEvent::Click(ids::EQS_APPLY));
        assert!(t.take_pending_apply());
        assert!(!t.take_pending_apply());
    }

    #[test]
    fn on_deactivate_clears_pending_but_keeps_params() {
        let mut t = EqualizeSizesTool::default();
        t.handle_panel_event(PanelEvent::Click(ids::EQS_MODE_FIXED));
        t.handle_panel_event(PanelEvent::SetValue(ids::EQS_FIXED_W, 123.0));
        t.handle_panel_event(PanelEvent::Click(ids::EQS_APPLY));
        t.on_deactivate();
        assert_eq!(t.params().target_mode, TargetMode::Fixed);
        assert_eq!(t.params().fixed_w, 123);
        assert!(!t.take_pending_apply());
    }

    #[test]
    fn on_activate_resets_params_and_arms_panel_repopulate() {
        // Regression cover (§12.3 / §12.4 UI_Bugs): `on_activate` must
        // route through `apply_ui_edit::ResetAll` so (a) params snap to
        // defaults AND (b) `pending_panel_reset` arms so the shell
        // bridge re-runs `Panel::populate(store)` and the slider knobs
        // visually snap back to defaults.
        let mut t = EqualizeSizesTool::default();
        // Dirty the state — simulate a prior session.
        t.handle_panel_event(PanelEvent::Click(ids::EQS_MODE_FIXED));
        t.handle_panel_event(PanelEvent::SetValue(ids::EQS_FIXED_W, 512.0));
        t.handle_panel_event(PanelEvent::Click(ids::EQS_ARRANGE_ON_GRID));
        assert_eq!(t.params().target_mode, TargetMode::Fixed);
        assert_eq!(t.params().fixed_w, 512);
        assert!(t.params().arrange_on_grid);
        // Drain any stray reset flag first.
        let _ = t.take_pending_panel_reset();

        t.on_activate();

        assert_eq!(*t.params(), EqualizeSizesParams::default());
        assert!(
            t.take_pending_panel_reset(),
            "on_activate must arm pending_panel_reset so the shell repopulates"
        );
    }

    #[test]
    fn run_full_resolution_multi_runs_algorithm() {
        let mut t = EqualizeSizesTool::default();
        // Switch to Fixed 32x32 + rasterize_after so the output is
        // predictable for the smoke check.
        t.handle_panel_event(PanelEvent::Click(ids::EQS_MODE_FIXED));
        t.handle_panel_event(PanelEvent::SetValue(ids::EQS_FIXED_W, 32.0));
        t.handle_panel_event(PanelEvent::SetValue(ids::EQS_FIXED_H, 32.0));
        let input = SpriteInput {
            rgba: vec![0u8; 64 * 64 * 4],
            width: 64,
            height: 64,
            scale_x: 1.0,
            scale_y: 1.0,
        };
        let out = t.run_full_resolution_multi(std::slice::from_ref(&input));
        assert_eq!(out.len(), 1);
        assert_eq!((out[0].width, out[0].height), (32, 32));
    }
}
