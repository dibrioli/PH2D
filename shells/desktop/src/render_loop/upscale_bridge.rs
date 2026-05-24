//! Upscale panel ⟷ tool bridge + on-canvas live preview.
//!
//! Mirror of `color_equalization_bridge.rs` (sabor 3 with live preview).
//! Run once per frame BEFORE `paint_hero_screen`. Does, in order:
//!
//! 1. Drives the panel's visibility (shown iff `upscale` is the active
//!    tool, keyed `"upscale"`).
//! 2. Pushes the source bitmap of the (multi-select) primary sprite
//!    into the tool's preview cache when the primary changes.
//! 3. Publishes the per-frame `UpscaleUiSnapshot` the panel paints
//!    next frame.
//! 4. (Re)computes the on-canvas preview RGBA by running the active
//!    algorithm on the cached canvas-preview source when the tool
//!    flags `take_params_dirty()`; caches it shell-side as
//!    `Arc<Vec<u8>>`. The overlay below paints it on top of the
//!    primary sprite's footprint so the user sees the chosen
//!    algorithm + factor LIVE.
//! 5. Returns the current multi-selection iff Apply fired this frame
//!    — the caller runs the full-resolution bake via `drain_upscale`.

use crate::app_state::UpscalePreview;
use ph2d_asset::AssetDb;
use ph2d_asset::AssetId;
use ph2d_ecs::SimWorld;
use ph2d_editor::HeroScreen;
use ph2d_editor::ToolRegistry;
use ph2d_host::WindowSize;
use ph2d_render::{Camera2d, Sprite, SpriteRenderer};
use ph2d_vector::VectorScene;
use std::collections::BTreeMap;
use std::sync::Arc;

/// Returns `Some(entity_bits_list)` iff Apply fired this frame.
#[allow(clippy::too_many_arguments)]
pub(super) fn dispatch(
    hero: &mut HeroScreen,
    tools: &mut ToolRegistry,
    sim: &mut SimWorld,
    renderer: &mut SpriteRenderer,
    asset_db: &AssetDb,
    atlas_asset_map: &BTreeMap<u32, AssetId>,
    camera: &Camera2d,
    window_size: WindowSize,
    vector_scene: &mut VectorScene,
    last_pushed_entity: &mut Option<u64>,
    upscale_preview: &mut Option<UpscalePreview>,
) -> Option<Vec<u64>> {
    let active = tools
        .active()
        .map(|t| t.id() == ph2d_editor::ToolId::new("upscale"))
        .unwrap_or(false);
    hero.panel_visibility.insert("upscale", active);
    // Image tools dock into the Inspector slot — hide Inspector while
    // the tool is active, restore on deactivate.
    hero.panel_visibility.insert("inspector", !active);

    if !active {
        *last_pushed_entity = None;
        *upscale_preview = None;
        #[cfg(feature = "panel-upscale")]
        ph2d_panel_upscale::set_current_upscale_snapshot(None);
        return None;
    }

    // Push source bitmap when the primary selection changed (or the
    // tool just activated). Without this, the tool's preview cache is
    // empty and the on-canvas overlay stays blank.
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
            let straight = src.image.into_straight();
            if let Some(tool) = tools.active_mut()
                && let Some(ups) = tool
                    .as_any_mut()
                    .downcast_mut::<ph2d_tool_upscale::UpscaleTool>()
            {
                ups.set_source_snapshot(straight.pixels, straight.width, straight.height);
            }
        }
        *last_pushed_entity = primary;
        // Selection changed → drop stale preview cache so we rebuild
        // against the new sprite on the next dirty tick.
        *upscale_preview = None;
    }

    let mut apply: Option<Vec<u64>> = None;
    let mut needs_panel_reset = false;
    if let Some(tool) = tools.active_mut()
        && let Some(ups) = tool
            .as_any_mut()
            .downcast_mut::<ph2d_tool_upscale::UpscaleTool>()
    {
        if ups.take_pending_apply() {
            let bits_list: Vec<u64> = hero.gizmo.iter_selected().collect();
            if !bits_list.is_empty() {
                apply = Some(bits_list);
            }
        }
        if ups.take_pending_panel_reset() {
            needs_panel_reset = true;
        }
        // (Re)compute the on-canvas preview when params changed since
        // the last frame. Gate the COPY into the shell-side
        // `Arc<Vec<u8>>` on `take_params_dirty()` so we don't realloc +
        // memcpy every frame, only when there's actually a new image.
        let needs_refresh = ups.take_params_dirty()
            || upscale_preview
                .as_ref()
                .map(|p| Some(p.entity_bits) != primary)
                .unwrap_or(true);
        if needs_refresh && let Some(bits) = primary {
            let mut buf = Vec::new();
            let (w, h) = ups.run_canvas_preview(&mut buf);
            if !buf.is_empty() && w > 0 && h > 0 {
                *upscale_preview = Some(UpscalePreview {
                    entity_bits: bits,
                    rgba: Arc::new(buf),
                    width: w,
                    height: h,
                });
            }
        }
        #[cfg(feature = "panel-upscale")]
        ph2d_panel_upscale::set_current_upscale_snapshot(Some(ups.ui_snapshot()));
    }

    // Reset just fired — re-populate panel store.
    if needs_panel_reset {
        ph2d_editor::panel::with_registry_opt(|reg| {
            if let Some(idx) = reg.find_by_panel_node_id(ph2d_panel_upscale::ids::UPS_PANEL) {
                reg.panels_mut()[idx].populate(&mut hero.store);
            }
        });
    }

    // Apply clears the cache so the overlay stops painting once the
    // bake has replaced the sprite texture (otherwise we'd paint the
    // pre-commit preview on top of the new texture and the user would
    // see a ghost until tool deactivation).
    if apply.is_some() {
        *upscale_preview = None;
    }

    // ── On-canvas overlay ──────────────────────────────────────────
    // Paint the preview RGBA over the primary sprite's footprint —
    // the sprite stays underneath. Upscale never reduces dims, so
    // the overlay covers the sprite footprint (which we scale to
    // match the on-screen sprite size; the underlying sprite shows
    // around any aspect-fit gap, but Upscale preserves source aspect,
    // so the overlay fits the footprint exactly when factor > 1).
    if let Some(preview) = upscale_preview.as_ref() {
        let entity = ph2d_ecs::Entity::from_bits(preview.entity_bits);
        if let (Some(tr), Some(sprite)) = (
            sim.world().get::<ph2d_ecs::Transform>(entity),
            sim.world().get::<Sprite>(entity),
        ) {
            let cx = tr.translation.x + sprite.anchor[0];
            let cy = tr.translation.y + sprite.anchor[1];
            let (sw, sh) = (sprite.size[0], sprite.size[1]);
            let (x0, y0) = camera.world_to_screen([cx - sw * 0.5, cy + sh * 0.5], window_size);
            let (x1, y1) = camera.world_to_screen([cx + sw * 0.5, cy - sh * 0.5], window_size);
            let quality = ph2d_editor::image_quality_for(hero.project.image_filter);
            vector_scene.draw_image_rgba(
                &preview.rgba,
                preview.width,
                preview.height,
                (x0 as f64, y0 as f64, x1 as f64, y1 as f64),
                quality,
            );
        }
    }

    apply
}
