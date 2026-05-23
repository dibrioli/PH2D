//! Drain `OneShotImageOp { tool_id: "make_square" }` — see the
//! function docstring for the full contract.

use std::collections::BTreeMap;

use ph2d_asset::{AssetDb, AssetId};
use ph2d_ecs::SimWorld;
use ph2d_editor::{Toast, ToastQueue};
use ph2d_render::SpriteRenderer;

use crate::hero_intents::texture_edit;
use crate::{EPS_PIXELS_PER_METER, ImageEditSnapshot, drop_undo_pre_source_if_individual};

/// Drain a `pending_make_square` Tool Action: pad the selected
/// sprite's source bitmap to a perfect square, acquire a fresh
/// Individual texture, recenter Transform translation so the
/// content's world center stays fixed across Trim↔Square cycles,
/// store the undo snapshot.
///
/// Audit fixes embedded:
///  - M1: cap output dimension against `max_texture_dimension_2d`
///    BEFORE acquire so over-size triggers a clear toast instead of
///    deferred device-loss.
///  - M2: sub-pixel recenter via `recenter_after_pad` for odd-diff
///    parity with Trim (was accumulating 0.5 px drift).
///  - C1: release the old Individual texture id after a successful
///    re-acquire so GPU memory doesn't leak across repeated edits.
///
/// Returns `true` if a toast was pushed (caller marks title dirty).
#[allow(clippy::too_many_arguments)]
pub(crate) fn drain_make_square(
    entity_bits: u64,
    project_pixels_per_meter: f32,
    sim: &mut SimWorld,
    renderer: &mut SpriteRenderer,
    asset_db: &AssetDb,
    atlas_asset_map: &BTreeMap<u32, AssetId>,
    toasts: &mut ToastQueue,
    image_edit_undo: &mut Option<ImageEditSnapshot>,
) -> bool {
    let entity = ph2d_ecs::Entity::from_bits(entity_bits);
    let px_per_m = project_pixels_per_meter.max(EPS_PIXELS_PER_METER);
    let Some(src) =
        texture_edit::read_sprite_source(entity, sim, renderer, asset_db, atlas_asset_map)
    else {
        toasts.push(Toast::error(ph2d_i18n::tr(
            "tool.make_square.toast.unavailable",
        )));
        return true;
    };
    let result =
        ph2d_tool_make_square::make_square(&src.image.pixels, src.image.width, src.image.height);
    if !result.made_square {
        toasts.push(Toast::info(ph2d_i18n::tr(
            "tool.make_square.toast.already_square",
        )));
        return true;
    }
    if result.size > renderer.max_texture_dimension_2d() {
        toasts.push(Toast::error(format!(
            "Make Square would exceed GPU texture limit ({} px max, would need {} px)",
            renderer.max_texture_dimension_2d(),
            result.size,
        )));
        return true;
    }
    let new_side = result.size as f32 / px_per_m;
    let new_translation = ph2d_editor::image_edit::recenter_after_pad(
        src.old_translation,
        [new_side, new_side],
        [result.size, result.size],
        ph2d_editor::image_edit::PixelBounds {
            x: result.offset_x,
            y: result.offset_y,
            width: src.image.width,
            height: src.image.height,
        },
    );
    // Color-agnostic pad (transparent border): PRESERVE the source alpha
    // mode so a premultiplied BG-Removal result survives Make-Square
    // byte-exact — the chokepoint re-derives `Sprite.premultiplied`.
    let edited =
        ph2d_render::SpriteImage::new(result.size, result.size, result.pixels, src.image.alpha);
    match texture_edit::commit_edited_texture(entity, sim, renderer, &edited, [new_side, new_side])
    {
        Err(err) => {
            toasts.push(Toast::error(format!("Make Square failed: {err}")));
            true
        }
        Ok(texture_id) => {
            if let Some(mut transform) = sim.world_mut().get_mut::<ph2d_ecs::Transform>(entity) {
                transform.translation.x = new_translation[0];
                transform.translation.y = new_translation[1];
            }
            drop_undo_pre_source_if_individual(renderer, image_edit_undo);
            *image_edit_undo = Some(ImageEditSnapshot {
                entity_bits,
                pre_source: src.old_source,
                pre_size: src.old_size_world,
                pre_translation: src.old_translation,
                pre_premultiplied: src.old_premultiplied,
                pre_anchor: src.old_anchor,
                post_individual_id: texture_id,
                label: "Make square",
            });
            toasts.push(Toast::success(format!(
                "Made square · {} × {} px · Cmd+Z to undo",
                result.size, result.size
            )));
            true
        }
    }
}
