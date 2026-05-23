//! Color Equalization panel ⟷ tool bridge.
//!
//! Run once per frame BEFORE `paint_hero_screen` (sibling of
//! `padding_bridge.rs` / `bgremoval_preview.rs`). Does, in order:
//!
//! 1. Drives the panel's visibility (shown iff `color_equalization`
//!    is the active tool, keyed `"color_equalization"` to match
//!    `ColorEqualizationPanel::ID`).
//! 2. Pushes the source bitmap of the (multi-select) primary sprite
//!    into the tool's preview cache when the tool just activated or
//!    the primary changed — the panel paints a live thumbnail from
//!    `preview_rgba()` and we'd otherwise see a blank slot.
//! 3. Publishes the per-frame `ColorEqualizationUiSnapshot` the panel
//!    paints next frame. Panel events themselves are routed earlier
//!    in the frame via `EditorAction::ToolPanelEvent →
//!    ColorEqualizationTool::handle_panel_event` (ADR-0040 TG-C), so
//!    by the time this bridge runs the live params already reflect
//!    every edit drained this tick.
//! 4. Returns the current multi-selection iff Apply fired this frame
//!    — the caller runs the full-resolution bake against each entity
//!    via `drain_color_equalization`.

use ph2d_asset::AssetDb;
use ph2d_asset::AssetId;
use ph2d_ecs::SimWorld;
use ph2d_editor::HeroScreen;
use ph2d_editor::ToolRegistry;
use ph2d_render::SpriteRenderer;
use std::collections::BTreeMap;

/// Returns `Some(entity_bits_list)` iff Apply fired this frame — the
/// caller runs `drain_color_equalization` on each. The list is captured
/// while the tool is borrowed so the bake site doesn't have to re-borrow
/// `tools`.
#[allow(clippy::too_many_arguments)]
pub(super) fn dispatch(
    hero: &mut HeroScreen,
    tools: &mut ToolRegistry,
    sim: &mut SimWorld,
    renderer: &mut SpriteRenderer,
    asset_db: &AssetDb,
    atlas_asset_map: &BTreeMap<u32, AssetId>,
    last_pushed_entity: &mut Option<u64>,
) -> Option<Vec<u64>> {
    let active = tools
        .active()
        .map(|t| t.id() == ph2d_editor::ToolId::new("color_equalization"))
        .unwrap_or(false);
    // Visibility: shown iff color_equalization is the active tool.
    hero.panel_visibility.insert("color_equalization", active);

    if !active {
        // Drop the cached "primary already pushed" marker so the next
        // activation pushes the source fresh.
        *last_pushed_entity = None;
        #[cfg(feature = "panel-color-equalization")]
        ph2d_panel_color_equalization::set_current_snapshot(None);
        return None;
    }

    // Refresh the tool's source bitmap when the primary selection
    // changed since the last push (or this is the first frame the
    // tool is active). Without this, `preview_rgba()` returns an
    // empty buffer and the panel's live thumbnail is blank.
    let primary = hero.gizmo.selection;
    if primary != *last_pushed_entity
        && let Some(bits) = primary
    {
        let entity = ph2d_ecs::Entity::from_bits(bits);
        if let Some(src) = crate::hero_intents::texture_edit::read_sprite_source(
            entity,
            sim,
            renderer,
            asset_db,
            atlas_asset_map,
        ) {
            // Color EQ operates on straight-alpha RGB; flip from
            // premultiplied if needed before pushing.
            let straight = src.image.into_straight();
            if let Some(tool) = tools.active_mut()
                && let Some(ceq) =
                    tool.as_any_mut()
                        .downcast_mut::<ph2d_tool_color_equalization::ColorEqualizationTool>()
            {
                ceq.set_source_snapshot(straight.pixels, straight.width, straight.height);
            }
        }
        *last_pushed_entity = primary;
    }

    let mut apply: Option<Vec<u64>> = None;
    if let Some(tool) = tools.active_mut()
        && let Some(ceq) = tool
            .as_any_mut()
            .downcast_mut::<ph2d_tool_color_equalization::ColorEqualizationTool>()
    {
        if ceq.take_pending_apply() {
            // Apply fires over the WHOLE multi-selection (Fase 0e
            // semantics — per-sprite bake, each driven by its own
            // source snapshot pushed by the drain).
            let bits_list: Vec<u64> = hero.gizmo.iter_selected().collect();
            if !bits_list.is_empty() {
                apply = Some(bits_list);
            }
        }
        #[cfg(feature = "panel-color-equalization")]
        ph2d_panel_color_equalization::set_current_snapshot(Some(ceq.ui_snapshot()));
    }

    apply
}
