//! Drain one Upscale-bake request — caller iterates per-sprite for
//! cross-sprite Apply.

use std::collections::BTreeMap;

use ph2d_asset::{AssetDb, AssetId};
use ph2d_ecs::SimWorld;
use ph2d_editor::{Toast, ToastQueue};
use ph2d_render::SpriteRenderer;

use crate::hero_intents::texture_edit;
use crate::{EPS_PIXELS_PER_METER, ImageEditSnapshot, drop_undo_pre_source_if_individual};

/// Drain one Upscale-bake request: push the sprite's source RGBA
/// into the active `UpscaleTool`, run the algorithm at full
/// resolution, swap to a fresh Individual texture, **grow
/// `Sprite.size` to the new pixel dim**, and reset `Transform.scale`
/// to `±1` (flip-sign preserved).
///
/// Legacy parity (port of `EditorToolDispatcher.ts case 'upscale'` in
/// `Game-Engine-Legada`):
///
/// ```ts
/// sel.img    = upResult.canvas;
/// sel.w      = upResult.canvas.width;   // pixel dim = world dim there
/// sel.h      = upResult.canvas.height;
/// sel.scaleX = sel.scaleX < 0 ? -1 : 1; // reset transform.scale
/// sel.scaleY = sel.scaleY < 0 ? -1 : 1;
/// ```
///
/// PH2D translation: `Sprite.size` is in WORLD meters (not pixels),
/// so `new_size = out_pixels / pixels_per_meter`. Transform scale
/// reset to `±1` ensures the visual size on screen equals the new
/// world size — without the reset, an existing scale factor would
/// double-multiply against the already-grown texture.
///
/// Cross-sprite: caller iterates `Vec<u64>` (one entry per selected
/// sprite); each call re-pushes the source so the bake matches the
/// live sprite.
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
    let entity = ph2d_ecs::Entity::from_bits(entity_bits);
    let px_per_m = project_pixels_per_meter.max(EPS_PIXELS_PER_METER);
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
    // Capture the prior Transform.scale signs BEFORE the commit so
    // the reset below preserves a horizontal/vertical flip.
    let (scale_sign_x, scale_sign_y) = sim
        .world()
        .get::<ph2d_ecs::Transform>(entity)
        .map(|t| (t.scale.x.signum(), t.scale.y.signum()))
        .unwrap_or((1.0, 1.0));
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
    // Sprite.size GROWS by the upscale factor (in world meters).
    let new_size_world = [out_w as f32 / px_per_m, out_h as f32 / px_per_m];
    match texture_edit::commit_edited_texture(entity, sim, renderer, &edited, new_size_world) {
        Err(err) => {
            toasts.push(Toast::error(format!("Upscale failed: {err}")));
            true
        }
        Ok(texture_id) => {
            // Reset Transform.scale to ±1, preserving the flip sign.
            // Without the reset, an existing scale factor would
            // double-multiply against the already-grown texture and
            // visually double the on-canvas size.
            if let Some(mut t) = sim.world_mut().get_mut::<ph2d_ecs::Transform>(entity) {
                t.scale.x = if scale_sign_x < 0.0 { -1.0 } else { 1.0 };
                t.scale.y = if scale_sign_y < 0.0 { -1.0 } else { 1.0 };
            }
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
