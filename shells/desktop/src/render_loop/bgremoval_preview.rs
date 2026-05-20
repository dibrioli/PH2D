//! Background-Removal panel ⟷ tool bridge + on-canvas live preview.
//!
//! Extracted from `render_loop::mod.rs` (HR-18 LOC cap) as a free
//! function, run once per frame BEFORE `paint_hero_screen`. Behavior-
//! preserving lift. Does, in order:
//!
//! 1. Pushes the active sprite's RGBA into the `BgRemovalTool` snapshot
//!    when the selection drifts (so the tool segments the live pixels).
//! 2. Drives the panel's visibility (shown iff bgremoval is active),
//!    drains the panel's `BgremovalUiEdit`s into the tool, fires the
//!    full-res commit on Apply, and publishes the normalized snapshot
//!    the panel paints next frame.
//! 3. (Re)computes the full-res straight-alpha preview and blits it on
//!    top of the sprite's on-canvas footprint (the sprite itself is
//!    suppressed from the sprite pass while previewing — see
//!    `sim_extract`).

use crate::app_state::BgremovalPreview;
use ph2d_asset::{AssetDb, AssetId};
use ph2d_ecs::SimWorld;
use ph2d_editor::HeroScreen;
use ph2d_editor::ToolRegistry;
use ph2d_host::WindowSize;
use ph2d_render::{Camera2d, Sprite, SpriteRenderer};
use ph2d_vector::VectorScene;
use std::collections::BTreeMap;

#[allow(clippy::too_many_arguments)]
pub(super) fn dispatch(
    hero: &mut HeroScreen,
    tools: &mut ToolRegistry,
    sim: &SimWorld,
    renderer: &mut SpriteRenderer,
    asset_db: &AssetDb,
    atlas_asset_map: &BTreeMap<u32, AssetId>,
    camera: &Camera2d,
    window_size: WindowSize,
    vector_scene: &mut VectorScene,
    mut bgremoval_ui_edits: Vec<ph2d_editor::tools::bgremoval::BgRemovalUiEdit>,
    last_bgremoval_pushed_entity: &mut Option<u64>,
    bgremoval_preview: &mut Option<BgremovalPreview>,
) {
    let bgremoval_is_active = tools
        .active()
        .map(|t| t.id() == ph2d_editor::ToolId::new("bgremoval"))
        .unwrap_or(false);
    // Snapshot push for the active BgRemovalTool — pushed once per
    // (tool-active + new selection) tuple.
    if bgremoval_is_active
        && let Some(bits) = hero.gizmo.selection
        && *last_bgremoval_pushed_entity != Some(bits)
    {
        let entity = ph2d_ecs::Entity::from_bits(bits);
        let snap = sim
            .world()
            .get::<Sprite>(entity)
            .and_then(|sprite| match sprite.source {
                ph2d_render::SpriteSource::Atlas { key } => {
                    let aid = atlas_asset_map.get(&key)?;
                    let asset = asset_db.get(aid)?;
                    match &*asset {
                        ph2d_asset::Asset::ImageRgba8 {
                            width,
                            height,
                            pixels,
                        } => Some((*width, *height, pixels.clone())),
                        _ => None,
                    }
                }
                ph2d_render::SpriteSource::Individual { texture_id } => renderer
                    .readback_individual(texture_id)
                    .ok()
                    .map(|(w, h, pix)| (w, h, pix.into())),
            });
        if let Some((w, h, rgba)) = snap
            && let Some(tool) = tools.active_mut()
            && let Some(bg) = tool
                .as_any_mut()
                .downcast_mut::<ph2d_editor::tools::bgremoval::BgRemovalTool>()
        {
            bg.set_source_snapshot(rgba.to_vec(), w, h);
            *last_bgremoval_pushed_entity = Some(bits);
        }
    }
    // Visibility: shown iff bgremoval is the active tool (keyed
    // "bgremoval" to match `BgRemovalPanel::ID`).
    hero.panel_visibility
        .insert("bgremoval", bgremoval_is_active);
    {
        let params_changed = !bgremoval_ui_edits.is_empty();
        let mut apply_selection: Option<u64> = None;
        if let Some(tool) = tools.active_mut()
            && let Some(bg) = tool
                .as_any_mut()
                .downcast_mut::<ph2d_editor::tools::bgremoval::BgRemovalTool>()
        {
            for edit in bgremoval_ui_edits.drain(..) {
                bg.apply_ui_edit(edit);
            }
            if bg.take_pending_apply() {
                apply_selection = hero.gizmo.selection;
            }
            #[cfg(feature = "panel-bgremoval")]
            ph2d_panel_bgremoval::set_current_bgremoval_snapshot(if bgremoval_is_active {
                Some(bg.ui_snapshot())
            } else {
                None
            });
            if bgremoval_is_active
                && bg.has_source()
                && let Some(bits) = hero.gizmo.selection
            {
                let stale = match &*bgremoval_preview {
                    Some(p) => params_changed || p.entity_bits != bits,
                    None => true,
                };
                if stale {
                    let mut out = Vec::new();
                    let (w, h) = bg.run_full_resolution(&mut out);
                    *bgremoval_preview = Some(BgremovalPreview {
                        entity_bits: bits,
                        rgba: std::sync::Arc::new(out),
                        width: w,
                        height: h,
                    });
                }
            } else {
                *bgremoval_preview = None;
            }
        }
        if !bgremoval_is_active {
            *bgremoval_preview = None;
        }
        if let Some(bits) = apply_selection {
            hero.bus
                .push(ph2d_editor::action_bus::EditorAction::Bgremoval { entity_bits: bits });
            // Committed result becomes the new sprite texture; drop the
            // preview so the overlay stops painting the pre-commit copy.
            *bgremoval_preview = None;
        }
    }
    // On-canvas preview overlay (straight-alpha, on top of the
    // suppressed sprite's footprint).
    if let Some(preview) = &*bgremoval_preview {
        let entity = ph2d_ecs::Entity::from_bits(preview.entity_bits);
        if let (Some(tr), Some(sprite)) = (
            sim.world().get::<ph2d_ecs::Transform>(entity),
            sim.world().get::<Sprite>(entity),
        ) {
            let (tx, ty) = (tr.translation.x, tr.translation.y);
            let (sw, sh) = (sprite.size[0], sprite.size[1]);
            let (x0, y0) = camera.world_to_screen([tx - sw * 0.5, ty + sh * 0.5], window_size);
            let (x1, y1) = camera.world_to_screen([tx + sw * 0.5, ty - sh * 0.5], window_size);
            vector_scene.draw_image_rgba(
                &preview.rgba,
                preview.width,
                preview.height,
                (x0 as f64, y0 as f64, x1 as f64, y1 as f64),
            );
        }
    }
}
