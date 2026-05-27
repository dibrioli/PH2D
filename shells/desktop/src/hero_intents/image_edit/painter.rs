//! Drain Painter — single-sprite bake of the painted canvas (W1 T1.5).
//!
//! Painter is stateful workhorse: between begin_stroke / end_stroke its
//! `canvas_rgba` is the live working canvas. On Apply, `run_full()` returns
//! the canvas RGBA which we bake back into the sprite as a fresh Individual
//! texture — same path as `drain_bgremoval` (alpha-mode preserved → premul
//! for the sprite shader's bilinear sample).

use std::collections::BTreeMap;

use ph2d_asset::{AssetDb, AssetId};
use ph2d_ecs::SimWorld;
use ph2d_editor::tool::RasterEditTool;
use ph2d_editor::{Toast, ToastQueue};
use ph2d_render::SpriteRenderer;
use ph2d_tool_painter::PainterTool;

use crate::ImageEditSnapshot;
use crate::hero_intents::texture_edit;

/// Drain a `pending_painter` Tool Action: pull the painted canvas at the
/// sprite's full resolution and swap to a fresh Individual texture with
/// the same dimensions (alpha + colour mutation). Caller gates on
/// `painter_active` so the active tool can be downcast to `PainterTool`.
/// Returns `true` if a toast was pushed.
#[allow(clippy::too_many_arguments)]
pub(crate) fn drain_painter(
    entity_bits: u64,
    sim: &mut SimWorld,
    renderer: &mut SpriteRenderer,
    asset_db: &AssetDb,
    atlas_asset_map: &BTreeMap<u32, AssetId>,
    toasts: &mut ToastQueue,
    pending_undo_entries: &mut Vec<ImageEditSnapshot>,
    painter: &mut PainterTool,
    last_painter_pushed_entity: &mut Option<u64>,
) -> bool {
    // **R3-LF-5 guard:** skip the bake when the user clicked Apply
    // without painting any stamp since the last `set_source`. Identity-
    // baking the source pixels wastes a fresh Individual texture + an
    // undo slot for no visual change.
    // **Audit T1.6 R9 — revert R7 J1-1 pt-BR translation.** R7
    // audit lens J1 mis-cited HR-15 as mandating pt-BR. User
    // feedback `feedback_app_ui_english_only` (2026-05-26) is
    // authoritative and explicit: app UI strings stay in English.
    // R7 L1-5 still applies (debug_assert distinguishes "no stroke
    // opened" from "Apply ran with no paint" at the queue_pointer
    // boundary).
    if !painter.has_painted_since_source() {
        toasts.push(Toast::info("Painter: no strokes to apply"));
        return true;
    }
    let entity = ph2d_ecs::Entity::from_bits(entity_bits);
    let Some(src) =
        texture_edit::read_sprite_source(entity, sim, renderer, asset_db, atlas_asset_map)
    else {
        toasts.push(Toast::error(
            "Painter: source unavailable (Atlas key missing or readback failed)",
        ));
        return true;
    };
    let old_size_world = src.old_size_world;
    let old_source = src.old_source;
    let old_translation = src.old_translation;
    let old_premultiplied = src.old_premultiplied;
    let old_anchor = src.old_anchor;
    // Painter's canvas storage is RGBA8 straight (the CPU stamp pipeline
    // does the premul dance per-pixel). `run_full()` returns it as-is —
    // the bake below premultiplies for the sprite shader's bilinear
    // sample, identical to bgremoval's discipline.
    let (canvas, w, h) = (painter as &mut dyn RasterEditTool).run_full();
    if canvas.is_empty() || w == 0 || h == 0 {
        toasts.push(Toast::error("Painter: empty canvas (no source pushed)"));
        return true;
    }
    let edited =
        ph2d_render::SpriteImage::from_bytes(w, h, canvas, ph2d_render::AlphaMode::Straight)
            .into_premultiplied();
    match texture_edit::commit_edited_texture(entity, sim, renderer, &edited, old_size_world) {
        Err(err) => {
            toasts.push(Toast::error(format!("Painter failed: {err}")));
            true
        }
        Ok(texture_id) => {
            pending_undo_entries.push(ImageEditSnapshot {
                entity_bits,
                pre_source: old_source,
                pre_size: old_size_world,
                pre_translation: old_translation,
                pre_premultiplied: old_premultiplied,
                pre_anchor: old_anchor,
                post_individual_id: texture_id,
                label: "Painter",
            });
            toasts.push(Toast::success("Painter applied · Cmd+Z to undo"));
            *last_painter_pushed_entity = None;
            true
        }
    }
}
