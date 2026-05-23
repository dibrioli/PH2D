//! Color Equalization panel ⟷ tool bridge + on-canvas live preview.
//!
//! Mirror of `bgremoval_preview.rs`. Run once per frame BEFORE
//! `paint_hero_screen`. Does, in order:
//!
//! 1. Drives the panel's visibility (shown iff `color_equalization`
//!    is the active tool, keyed `"color_equalization"`).
//! 2. Pushes the source bitmap of the (multi-select) primary sprite
//!    into the tool's preview cache when the primary changes.
//! 3. Publishes the per-frame `ColorEqualizationUiSnapshot` the panel
//!    paints next frame.
//! 4. (Re)computes the full-res straight-alpha preview when the tool
//!    reports `take_params_dirty()` and caches it on the shell side
//!    as `Arc<Vec<u8>>`. The overlay below paints it on top of the
//!    sprite's on-canvas footprint so adjustments are visible LIVE.
//! 5. Returns the current multi-selection iff Apply fired this frame
//!    — the caller runs the full-resolution bake via
//!    `drain_color_equalization`.

use crate::app_state::ColorEqualizationPreview;
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
    color_equalization_preview: &mut Option<ColorEqualizationPreview>,
) -> Option<Vec<u64>> {
    let active = tools
        .active()
        .map(|t| t.id() == ph2d_editor::ToolId::new("color_equalization"))
        .unwrap_or(false);
    hero.panel_visibility.insert("color_equalization", active);

    if !active {
        *last_pushed_entity = None;
        *color_equalization_preview = None;
        #[cfg(feature = "panel-color-equalization")]
        ph2d_panel_color_equalization::set_current_snapshot(None);
        return None;
    }

    // Push source bitmap when the primary selection changed (or the
    // tool just activated). Without this, `preview_rgba()` returns an
    // empty buffer and the canvas overlay stays blank.
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
                && let Some(ceq) =
                    tool.as_any_mut()
                        .downcast_mut::<ph2d_tool_color_equalization::ColorEqualizationTool>()
            {
                ceq.set_source_snapshot(straight.pixels, straight.width, straight.height);
            }
        }
        *last_pushed_entity = primary;
        // Selection changed → drop stale preview cache so we rebuild
        // against the new sprite on the next dirty tick.
        *color_equalization_preview = None;
    }

    let mut apply: Option<Vec<u64>> = None;
    if let Some(tool) = tools.active_mut()
        && let Some(ceq) = tool
            .as_any_mut()
            .downcast_mut::<ph2d_tool_color_equalization::ColorEqualizationTool>()
    {
        if ceq.take_pending_apply() {
            let bits_list: Vec<u64> = hero.gizmo.iter_selected().collect();
            if !bits_list.is_empty() {
                apply = Some(bits_list);
            }
        }
        // (Re)compute the on-canvas preview when params changed since
        // the last frame. The tool's `preview_rgba()` lazily reruns
        // the pipeline on its internal `preview_dirty` flag — we
        // gate the COPY into the shell-side `Arc<Vec<u8>>` on
        // `take_params_dirty()` so we don't realloc + memcpy every
        // frame, only when there's actually a new image.
        let needs_refresh = ceq.take_params_dirty()
            || color_equalization_preview
                .as_ref()
                .map(|p| Some(p.entity_bits) != primary)
                .unwrap_or(true);
        if needs_refresh && let Some(bits) = primary {
            let (rgba, w, h) = ceq.preview_rgba();
            if !rgba.is_empty() && w > 0 && h > 0 {
                *color_equalization_preview = Some(ColorEqualizationPreview {
                    entity_bits: bits,
                    rgba: Arc::new(rgba.to_vec()),
                    width: w,
                    height: h,
                });
            }
        }
        #[cfg(feature = "panel-color-equalization")]
        ph2d_panel_color_equalization::set_current_snapshot(Some(ceq.ui_snapshot()));
    }

    // Apply clears the cache so the overlay stops painting once the
    // bake has replaced the sprite texture (otherwise we'd paint the
    // pre-commit copy on top of the new texture and the user would
    // see a "ghost" until tool deactivation).
    if apply.is_some() {
        *color_equalization_preview = None;
    }

    // ── On-canvas overlay ──────────────────────────────────────────
    // Paint the preview RGBA over the primary sprite's footprint —
    // the sprite stays underneath (no suppression like BgRemoval since
    // Color EQ preserves alpha + dims, so the overlay covers it
    // pixel-perfect with the new RGB values).
    if let Some(preview) = color_equalization_preview.as_ref() {
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
