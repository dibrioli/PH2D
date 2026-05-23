//! Drain one Upscale-bake request — caller iterates per-sprite for
//! cross-sprite Apply.

use std::collections::BTreeMap;

use ph2d_asset::{AssetDb, AssetId};
use ph2d_ecs::SimWorld;
use ph2d_editor::{Toast, ToastQueue};
use ph2d_render::SpriteRenderer;

use crate::hero_intents::texture_edit;
use crate::{ImageEditSnapshot, drop_undo_pre_source_if_individual};

/// Drain one Upscale-bake request: push the sprite's source RGBA
/// into the active `UpscaleTool`, run the algorithm at full
/// resolution, swap to a fresh Individual texture (resizing
/// `Sprite.size` to keep the visual on-screen size the same — Upscale
/// rewrites pixels at higher resolution but the world-space extent
/// stays put), and capture undo.
///
/// Cross-sprite: caller iterates `Vec<u64>` (one entry per selected
/// sprite); each call re-pushes the source so the bake matches the
/// live sprite.
///
/// Mirror of `super::color_equalization::drain_color_equalization`
/// (set_source_snapshot → run_full_resolution → texture swap) with
/// one twist: `Sprite.size` is PRESERVED in world space (we pass
/// `old_size_world` to `commit_edited_texture`), so the visual size
/// stays the same and the user sees a higher-resolution version of
/// the same sprite.
///
/// Returns `true` if a toast was pushed (caller marks title dirty).
#[allow(clippy::too_many_arguments)]
pub(crate) fn drain_upscale(
    entity_bits: u64,
    project_pixels_per_meter: f32,
    sim: &mut SimWorld,
    renderer: &mut SpriteRenderer,
    asset_db: &AssetDb,
    atlas_asset_map: &BTreeMap<u32, AssetId>,
    toasts: &mut ToastQueue,
    image_edit_undo: &mut Option<ImageEditSnapshot>,
    ups: &mut ph2d_tool_upscale::UpscaleTool,
) -> bool {
    let _ = project_pixels_per_meter; // size kept in world space (see doc above)
    let entity = ph2d_ecs::Entity::from_bits(entity_bits);
    let Some(src) =
        texture_edit::read_sprite_source(entity, sim, renderer, asset_db, atlas_asset_map)
    else {
        toasts.push(Toast::error("Upscale: source unavailable"));
        return true;
    };
    let old_size_world = src.old_size_world;
    let old_source = src.old_source;
    let old_translation = src.old_translation;
    let old_premultiplied = src.old_premultiplied;
    let old_anchor = src.old_anchor;
    // Resample kernels operate on straight-alpha RGBA per-channel —
    // round-trip back to the source alpha mode at the chokepoint so a
    // premultiplied BgRemoval result survives Upscale byte-faithful.
    let source_alpha = src.image.alpha;
    let straight = src.image.into_straight();
    ups.set_source_snapshot(straight.pixels, straight.width, straight.height);
    let mut out: Vec<u8> = Vec::new();
    let (out_w, out_h) = ups.run_full_resolution(&mut out);
    // GPU texture cap — at 16× the worst case is `16384 × 16384`, which
    // exceeds the typical 8192 limit. Bail with a clear toast so the
    // user can lower the factor.
    let max_dim = renderer.max_texture_dimension_2d();
    if out_w > max_dim || out_h > max_dim {
        toasts.push(Toast::error(format!(
            "Upscale would exceed GPU texture limit ({} px max, would need {} × {} px). Try a smaller scale factor.",
            max_dim, out_w, out_h
        )));
        return true;
    }
    let edited_straight =
        ph2d_render::SpriteImage::new(out_w, out_h, out, ph2d_render::AlphaMode::Straight);
    let edited = if source_alpha.is_premultiplied() {
        edited_straight.into_premultiplied()
    } else {
        edited_straight
    };
    match texture_edit::commit_edited_texture(entity, sim, renderer, &edited, old_size_world) {
        Err(err) => {
            toasts.push(Toast::error(format!("Upscale failed: {err}")));
            true
        }
        Ok(texture_id) => {
            drop_undo_pre_source_if_individual(renderer, image_edit_undo);
            *image_edit_undo = Some(ImageEditSnapshot {
                entity_bits,
                pre_source: old_source,
                pre_size: old_size_world,
                pre_translation: old_translation,
                pre_premultiplied: old_premultiplied,
                pre_anchor: old_anchor,
                post_individual_id: texture_id,
                label: "Upscale",
            });
            toasts.push(Toast::success(format!(
                "Upscaled · {} × {} px · Cmd+Z to undo",
                out_w, out_h
            )));
            true
        }
    }
}
