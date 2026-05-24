//! Drain `OneShotImageOp { tool_id: "trim_transparency" }` — see the
//! function docstring for the full contract.

use std::collections::BTreeMap;

use ph2d_asset::{AssetDb, AssetId};
use ph2d_ecs::SimWorld;
use ph2d_editor::{Toast, ToastQueue};
use ph2d_render::SpriteRenderer;

use crate::hero_intents::texture_edit;
use crate::{EPS_PIXELS_PER_METER, ImageEditSnapshot};

/// Drain a `pending_trim_transparency` Tool Action: read the
/// sprite's source RGBA pixels, run the trim algorithm, and (if any
/// transparent border was found) re-source the sprite to a fresh
/// `IndividualTextureStore` entry at the trimmed dimensions.
/// Atlas-shared sprites cannot be edited in-place (would corrupt
/// every sibling sharing the same key); we materialise the trim
/// result as a NEW individual texture and repoint only this entity.
///
/// World-position preservation: after the crop, the entity's
/// `Transform.translation` is shifted by
/// `ph2d_editor::image_edit::recenter_after_crop` so the *visual* center of the
/// surviving opaque content stays put even when it lived off-center
/// inside the original frame.
///
/// Returns `true` if a toast was pushed (caller marks title dirty).
#[allow(clippy::too_many_arguments)]
pub(crate) fn drain_trim_transparency(
    entity_bits: u64,
    project_pixels_per_meter: f32,
    sim: &mut SimWorld,
    renderer: &mut SpriteRenderer,
    asset_db: &AssetDb,
    atlas_asset_map: &BTreeMap<u32, AssetId>,
    toasts: &mut ToastQueue,
    pending_undo_entries: &mut Vec<ImageEditSnapshot>,
) -> bool {
    let entity = ph2d_ecs::Entity::from_bits(entity_bits);
    let px_per_m = project_pixels_per_meter.max(EPS_PIXELS_PER_METER);
    let Some(src) =
        texture_edit::read_sprite_source(entity, sim, renderer, asset_db, atlas_asset_map)
    else {
        toasts.push(Toast::error(ph2d_i18n::tr(
            "tool.trim_transparency.toast.unavailable",
        )));
        return true;
    };
    let result = ph2d_tool_trim_transparency::trim_transparency(
        &src.image.pixels,
        src.image.width,
        src.image.height,
        0,
    );
    if !result.trimmed {
        toasts.push(Toast::info(ph2d_i18n::tr(
            "tool.trim_transparency.toast.nothing",
        )));
        return true;
    }
    // Color-agnostic crop: PRESERVE the source alpha mode (no
    // un-premultiply round-trip). A premultiplied BG-Removal result
    // stays byte-exact + fringe-free after a Trim — the chokepoint
    // re-derives `Sprite.premultiplied` from this image's mode.
    let new_size = [
        result.width as f32 / px_per_m,
        result.height as f32 / px_per_m,
    ];
    let new_translation = ph2d_editor::image_edit::recenter_after_crop(
        src.old_translation,
        src.old_size_world,
        [src.image.width, src.image.height],
        ph2d_editor::image_edit::PixelBounds {
            x: result.bounds.x,
            y: result.bounds.y,
            width: result.bounds.width,
            height: result.bounds.height,
        },
    );
    let edited =
        ph2d_render::SpriteImage::new(result.width, result.height, result.pixels, src.image.alpha);
    match texture_edit::commit_edited_texture(entity, sim, renderer, &edited, new_size) {
        Err(err) => {
            toasts.push(Toast::error(format!("Trim failed: {err}")));
            true
        }
        Ok(texture_id) => {
            if let Some(mut transform) = sim.world_mut().get_mut::<ph2d_ecs::Transform>(entity) {
                transform.translation.x = new_translation[0];
                transform.translation.y = new_translation[1];
            }
            pending_undo_entries.push(ImageEditSnapshot {
                entity_bits,
                pre_source: src.old_source,
                pre_size: src.old_size_world,
                pre_translation: src.old_translation,
                pre_premultiplied: src.old_premultiplied,
                pre_anchor: src.old_anchor,
                post_individual_id: texture_id,
                label: "Trim",
            });
            toasts.push(Toast::success(format!(
                "Trimmed · {} × {} px · Cmd+Z to undo",
                result.width, result.height
            )));
            true
        }
    }
}
