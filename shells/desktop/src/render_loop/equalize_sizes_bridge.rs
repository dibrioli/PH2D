//! Equalize Sizes panel ⟷ tool bridge.
//!
//! Run once per frame BEFORE `paint_hero_screen`. Does, in order:
//!
//! 1. Drives the panel's visibility (shown iff `equalize_sizes`
//!    is the active tool).
//! 2. Syncs `params.grid_unit` (pixels) from the live
//!    `GridSnapState::square_cfg.cell_size` (meters) via
//!    `hero.project.pixels_per_meter`. The panel has no slider/chip
//!    for grid unit — the Grid Snap tool owns it. Square kind only in
//!    v1; the user picks the unit there, the EQS panel reflects it
//!    read-only.
//! 3. Publishes the per-frame `EqualizeSizesUiSnapshot` the panel
//!    paints next frame.
//! 4. Returns the current multi-selection iff Apply fired this frame
//!    — the caller runs the full-resolution multi bake via
//!    `drain_equalize_sizes`.
//!
//! Differences from `color_equalization_bridge` (sabor 3 sibling):
//! - **No source-bitmap push.** Equalize Sizes is cross-sprite; the
//!   per-entity input (`SpriteInput { rgba, w, h, scale_x, scale_y }`)
//!   is collected at Apply time inside the drain rather than pushed
//!   per-frame (the tool has no `set_source_snapshot` channel — the
//!   `run_full_resolution_multi` takes the whole slice at once).
//! - **No on-canvas preview cache (`take_params_dirty` / `preview_rgba`
//!   path).** Slider drags do NOT update the canvas in real time at
//!   this iteration — the user sees the effect after Apply. The tool
//!   exposes no preview channel and a transform-only live preview
//!   (rewriting `Transform.scale` in PresentWorld per frame) is a
//!   future iteration not in scope.

use ph2d_editor::HeroScreen;
use ph2d_editor::ToolRegistry;
use ph2d_tool_equalize_sizes::params::EqualizeSizesUiEdit;

/// Returns `Some(entity_bits_list)` iff Apply fired this frame.
pub(super) fn dispatch(hero: &mut HeroScreen, tools: &mut ToolRegistry) -> Option<Vec<u64>> {
    let active = tools
        .active()
        .map(|t| t.id() == ph2d_editor::ToolId::new("equalize_sizes"))
        .unwrap_or(false);
    hero.panel_visibility.insert("equalize_sizes", active);
    // Image tools dock into the Inspector slot — hide Inspector while
    // the tool is active, restore on deactivate. Edge-triggered (Wave
    // 10 Etapa 4 smoke fix — see bgremoval_preview.rs for rationale).
    {
        use std::sync::atomic::{AtomicBool, Ordering};
        static LAST_ACTIVE: AtomicBool = AtomicBool::new(false);
        let was = LAST_ACTIVE.swap(active, Ordering::Relaxed);
        if was != active {
            hero.panel_visibility.insert("inspector", !active);
        }
    }

    if !active {
        #[cfg(feature = "panel-equalize-sizes")]
        ph2d_panel_equalize_sizes::set_current_equalize_sizes_snapshot(None);
        return None;
    }

    // Live cell size in pixels (Square kind). Other kinds keep
    // whatever the last sync wrote — the panel's "Align" toggle is a
    // no-op outside Square in v1.
    let cell_m = hero.grid.snap_state.square_cfg.cell_size;
    let ppm = hero.project.pixels_per_meter.max(1.0e-3);
    let cell_px = (cell_m * ppm).round().max(1.0) as u32;

    let mut apply: Option<Vec<u64>> = None;
    let mut needs_panel_reset = false;
    if let Some(tool) = tools.active_mut()
        && let Some(eqs) = tool
            .as_any_mut()
            .downcast_mut::<ph2d_tool_equalize_sizes::EqualizeSizesTool>()
    {
        // Sync grid cell from the Grid Snap state. This routes through
        // `apply_ui_edit::SetGridUnit` so the clamp (1..EQS_MAX_GRID_UNIT)
        // is honored single-source-of-truth.
        eqs.apply_ui_edit(EqualizeSizesUiEdit::SetGridUnit(cell_px));

        if eqs.take_pending_apply() {
            let bits_list: Vec<u64> = hero.gizmo.iter_selected().collect();
            if !bits_list.is_empty() {
                apply = Some(bits_list);
            }
        }
        if eqs.take_pending_panel_reset() {
            needs_panel_reset = true;
        }
        #[cfg(feature = "panel-equalize-sizes")]
        ph2d_panel_equalize_sizes::set_current_equalize_sizes_snapshot(Some(eqs.ui_snapshot()));
    }
    if needs_panel_reset {
        ph2d_editor::panel::with_registry_opt(|reg| {
            if let Some(idx) = reg.find_by_panel_node_id(ph2d_tool_equalize_sizes::ids::EQS_PANEL) {
                reg.panels_mut()[idx].populate(&mut hero.store);
            }
        });
    }

    apply
}
