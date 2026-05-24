//! Drain `OneShotImageOp { tool_id: "rasterize" }` — see the
//! function docstring for the full contract.

use std::collections::BTreeMap;

use ph2d_asset::{AssetDb, AssetId};
use ph2d_ecs::SimWorld;
use ph2d_editor::{Toast, ToastQueue};
use ph2d_render::SpriteRenderer;

use crate::hero_intents::texture_edit;
use crate::{EPS_PIXELS_PER_METER, ImageEditSnapshot};

/// Drain one `OneShotImageOp { tool_id: "rasterize", entity_bits }`:
/// read the sprite's source + live `Transform { scale, rotation }`,
/// call `ph2d_tool_rasterize::rasterize` to bake the visual transform
/// directly into a fresh RGBA buffer, commit via
/// `texture_edit::commit_edited_texture`, then reset `Transform` to
/// identity (scale `(±1, ±1)` flip-preserved → `(1.0, 1.0)`, rotation
/// `0`).
///
/// Mirrors `drain_make_square` / `drain_padding` in shape: chokepoint
/// texture I/O, GPU-limit cap on output dim, single-level undo, toast
/// on outcome.
///
/// Returns `true` if a toast was pushed (caller marks title dirty).
#[allow(clippy::too_many_arguments)]
pub(crate) fn drain_rasterize(
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
    let Some((scale_x, scale_y, rotation)) = sim
        .world()
        .get::<ph2d_ecs::Transform>(entity)
        .map(|t| (t.scale.x, t.scale.y, t.rotation))
    else {
        toasts.push(Toast::error("Rasterize unavailable for this sprite"));
        return true;
    };
    let Some(src) =
        texture_edit::read_sprite_source(entity, sim, renderer, asset_db, atlas_asset_map)
    else {
        toasts.push(Toast::error("Rasterize: source unavailable"));
        return true;
    };
    // Resample kernels operate on straight-alpha RGBA per-channel; a
    // premultiplied source would attenuate edges. Round-trip back to
    // the source alpha mode at commit (the chokepoint derives
    // `Sprite.premultiplied` from `img.alpha`).
    let source_alpha = src.image.alpha;
    let straight = src.image.into_straight();
    let result = ph2d_tool_rasterize::rasterize(
        &straight.pixels,
        straight.width,
        straight.height,
        scale_x,
        scale_y,
        rotation,
    );
    if !result.did_change {
        toasts.push(Toast::info("Rasterize: already at identity Transform"));
        return true;
    }
    // GPU texture cap — the rotated bbox can grow above
    // `max(w, h) * sqrt(2)`; bail out cleanly if we'd exceed the
    // device limit.
    let max_dim = renderer.max_texture_dimension_2d();
    if result.width > max_dim || result.height > max_dim {
        toasts.push(Toast::error(format!(
            "Rasterize would exceed GPU texture limit ({} px max, would need {} × {} px)",
            max_dim, result.width, result.height
        )));
        return true;
    }
    let edited_straight = ph2d_render::SpriteImage::from_bytes(
        result.width,
        result.height,
        result.pixels,
        ph2d_render::AlphaMode::Straight,
    );
    let edited = if source_alpha.is_premultiplied() {
        edited_straight.into_premultiplied()
    } else {
        edited_straight
    };
    let new_size_world = [
        result.width as f32 / px_per_m,
        result.height as f32 / px_per_m,
    ];
    match texture_edit::commit_edited_texture(entity, sim, renderer, &edited, new_size_world) {
        Err(err) => {
            toasts.push(Toast::error(format!("Rasterize failed: {err}")));
            true
        }
        Ok(texture_id) => {
            // Reset Transform to identity. Scale collapses to ±1 with
            // the flip sign preserved (a horizontally flipped sprite
            // stays horizontally flipped — Rasterize bakes the
            // ABSOLUTE scale into pixels, leaving only the sign).
            // Rotation goes to exactly 0.
            if let Some(mut t) = sim.world_mut().get_mut::<ph2d_ecs::Transform>(entity) {
                t.scale.x = if scale_x < 0.0 { -1.0 } else { 1.0 };
                t.scale.y = if scale_y < 0.0 { -1.0 } else { 1.0 };
                t.rotation = 0.0;
            }
            pending_undo_entries.push(ImageEditSnapshot {
                entity_bits,
                pre_source: src.old_source,
                pre_size: src.old_size_world,
                pre_translation: src.old_translation,
                pre_premultiplied: src.old_premultiplied,
                pre_anchor: src.old_anchor,
                post_individual_id: texture_id,
                label: "Rasterize",
            });
            toasts.push(Toast::success(format!(
                "Rasterized · {} × {} px · Cmd+Z to undo",
                result.width, result.height
            )));
            true
        }
    }
}
