//! Upscale panel ⟷ tool bridge + on-canvas live preview.
//!
//! Wave 10 / Etapa 2 refactor (ADR-0041): uses `ph2d-tool-runtime`
//! helpers (mirror of `bgremoval_preview.rs` from Etapa 1.B) so the
//! generic raster I/O lifecycle stays in one place. Upscale-specific
//! bits (panel snapshot publish, panel reset propagation) keep their
//! downcast — ADR-0040 §3 documented exception.
//!
//! Frame order:
//!
//! 1. (Generic) Source push via [`ph2d_tool_runtime::drive_source_push`]
//!    when primary selection drifts (rebuilds the thumbnail + canvas
//!    source inside the tool).
//! 2. (Mixed) Drain `current_preview` into the shell cache via
//!    [`ph2d_tool_runtime::drive_preview_cache`]; capture multi-sprite
//!    Apply selection via
//!    [`ph2d_tool_runtime::drive_pending_commit`]; Upscale-specific
//!    panel-reset + snapshot publish via downcast.
//! 3. (Generic) Deactivate cleanup via
//!    [`ph2d_tool_runtime::drive_deactivate_cleanup`] when no longer
//!    the active raster tool.
//! 4. (Upscale-specific) On-canvas overlay paints the cached preview
//!    RGBA on top of the primary sprite's footprint.

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

    // ── Inactive path — clear LOCAL bridge state only ────────────────────
    // Wave 10 / Etapa 2 audit [C1 CRITICAL fix]: previous version called
    // `drive_deactivate_cleanup` on `tools.active_mut()` here — but that
    // returns the CURRENTLY-ACTIVE tool (which may be another RasterEditTool
    // like BgR or CEQ), NOT the Upscale tool we're a bridge for. Calling
    // `RasterEditTool::deactivate()` on the wrong tool zeroes the state
    // of whichever raster tool happens to be active (drains pending_apply,
    // params_dirty, cached_canvas_preview, etc.) — destroying its drag.
    //
    // The Upscale tool's own `Tool::on_deactivate` already fires when
    // `ToolRegistry::set_active` switches AWAY from Upscale (see
    // `tool.rs::ToolRegistry::set_active`); that path mirrors the
    // RasterEditTool::deactivate semantics. The bridge only needs to
    // clear its own shell-side cache here.
    if !active {
        *last_pushed_entity = None;
        *upscale_preview = None;
        #[cfg(feature = "panel-upscale")]
        ph2d_panel_upscale::set_current_upscale_snapshot(None);
        return None;
    }

    // ── (Generic) Source push when primary selection drifts ───────────────
    if let Some(tool) = tools.active_mut()
        && let Some(raster) = tool.as_raster_edit_mut()
    {
        ph2d_tool_runtime::drive_source_push(
            raster,
            hero.gizmo.selection,
            last_pushed_entity,
            |entity| {
                let src = crate::hero_intents::texture_edit::read_sprite_source(
                    entity,
                    sim,
                    renderer,
                    asset_db,
                    atlas_asset_map,
                )?;
                let straight = src.image.into_straight();
                Some(ph2d_tool_runtime::RasterSource {
                    pixels: straight.pixels,
                    width: straight.width,
                    height: straight.height,
                })
            },
        );
    }

    // ── (Mixed) Generic preview cache + Apply capture + Upscale-specific ──
    // Single downcast block: both runtime-helpers (need &mut dyn
    // RasterEditTool) and Upscale-specific concerns share the borrow.
    let mut apply: Option<Vec<u64>> = None;
    let mut needs_panel_reset = false;
    if let Some(tool) = tools.active_mut()
        && let Some(ups) = tool
            .as_any_mut()
            .downcast_mut::<ph2d_tool_upscale::UpscaleTool>()
    {
        // (Generic) Drain current_preview into shell cache.
        ph2d_tool_runtime::drive_preview_cache(ups, hero.gizmo.selection, upscale_preview);

        // (Generic) Capture multi-sprite Apply selection.
        let bits = ph2d_tool_runtime::drive_pending_commit(ups, hero.gizmo.iter_selected());
        if !bits.is_empty() {
            apply = Some(bits);
        }

        // (Upscale-specific) Panel-store reset propagation.
        if ups.take_pending_panel_reset() {
            needs_panel_reset = true;
        }

        // (Upscale-specific) Snapshot publish for the docked panel.
        #[cfg(feature = "panel-upscale")]
        ph2d_panel_upscale::set_current_upscale_snapshot(Some(ups.ui_snapshot()));
    }

    // Reset just fired — re-populate panel store (post-borrow because
    // hero.store aliases tools.active_mut() above).
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

    // ── On-canvas overlay (Upscale-specific paint) ────────────────────────
    // Paint the preview RGBA over the primary sprite's footprint — the
    // sprite stays underneath. Upscale never reduces dims, so the overlay
    // covers the sprite footprint (which we scale to match the on-screen
    // sprite size; the underlying sprite shows around any aspect-fit gap,
    // but Upscale preserves source aspect, so the overlay fits the
    // footprint exactly when factor > 1).
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
