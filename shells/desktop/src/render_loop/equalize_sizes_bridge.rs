//! Equalize Sizes panel ⟷ tool bridge.
//!
//! Run once per frame BEFORE `paint_hero_screen`. Does, in order:
//!
//! 1. Drives the panel's visibility (shown iff `equalize_sizes`
//!    is the active tool).
//! 2. Publishes the per-frame `EqualizeSizesUiSnapshot` the panel
//!    paints next frame.
//! 3. Returns the current multi-selection iff Apply fired this frame
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
//!   future iteration not in scope of this wiring (DIRETRIZ §3.8.3.1
//!   shape: production tools currently use `as_any_mut` downcast; the
//!   generic preview channel is fan-out future work).
//!
//! Smoke note: the Apply bake itself still gives the visual feedback —
//! Cancel from the panel emits `CancelActiveTool` and the orchestrator
//! cleans up.

use ph2d_editor::HeroScreen;
use ph2d_editor::ToolRegistry;

/// Returns `Some(entity_bits_list)` iff Apply fired this frame.
pub(super) fn dispatch(hero: &mut HeroScreen, tools: &mut ToolRegistry) -> Option<Vec<u64>> {
    let active = tools
        .active()
        .map(|t| t.id() == ph2d_editor::ToolId::new("equalize_sizes"))
        .unwrap_or(false);
    hero.panel_visibility.insert("equalize_sizes", active);

    if !active {
        #[cfg(feature = "panel-equalize-sizes")]
        ph2d_panel_equalize_sizes::set_current_equalize_sizes_snapshot(None);
        return None;
    }

    let mut apply: Option<Vec<u64>> = None;
    if let Some(tool) = tools.active_mut()
        && let Some(eqs) = tool
            .as_any_mut()
            .downcast_mut::<ph2d_tool_equalize_sizes::EqualizeSizesTool>()
    {
        if eqs.take_pending_apply() {
            let bits_list: Vec<u64> = hero.gizmo.iter_selected().collect();
            if !bits_list.is_empty() {
                apply = Some(bits_list);
            }
        }
        #[cfg(feature = "panel-equalize-sizes")]
        ph2d_panel_equalize_sizes::set_current_equalize_sizes_snapshot(Some(eqs.ui_snapshot()));
    }

    apply
}
