//! Drain the Padding Apply latch — see the function docstring for the
//! full contract.

use std::collections::BTreeMap;

use ph2d_asset::{AssetDb, AssetId};
use ph2d_ecs::SimWorld;
use ph2d_editor::{Toast, ToastQueue};
use ph2d_render::{Sprite, SpriteRenderer};

use crate::hero_intents::texture_edit;
use crate::{EPS_PIXELS_PER_METER, ImageEditSnapshot};

/// Drain a Padding Apply: resize the selected sprite's source bitmap by
/// the tool's signed per-edge `spec` (positive = transparent expand,
/// negative = crop) via `ph2d_tool_padding::add_padding`, acquire a
/// fresh Individual texture, and (when `recenter_pivot`) reproject the
/// Transform translation so the original content's world position holds.
///
/// `recenter_pivot` is the panel's pivot-mode toggle: `true` recalculates
/// the translation (content stays world-fixed); `false` leaves the
/// translation unchanged (the canvas resizes around the current pivot).
///
/// Mirrors [`super::make_square::drain_make_square`] (texture
/// chokepoint, undo snapshot, `max_texture_dimension_2d` cap). The
/// recenter fix-up uses the `PaddingResult::pivot_delta_*` directly —
/// it's the signed shift of the original content's top-left inside the
/// new canvas, so the recenter formula keeps the content world-fixed
/// for pure-expand, pure-crop, AND mixed specs.
///
/// Returns `true` if a toast was pushed (caller marks title dirty).
#[allow(clippy::too_many_arguments)]
pub(crate) fn drain_padding(
    entity_bits: u64,
    spec: ph2d_tool_padding::PaddingSpec,
    recenter_pivot: bool,
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
    if spec.is_noop() {
        toasts.push(Toast::info("Padding: nothing to apply (all edges 0)"));
        return true;
    }
    let Some(src) =
        texture_edit::read_sprite_source(entity, sim, renderer, asset_db, atlas_asset_map)
    else {
        toasts.push(Toast::error("Padding unavailable for this sprite"));
        return true;
    };
    let (src_w, src_h) = (src.image.width, src.image.height);
    // Wave 11 migration (ADR-0042 §6 #2): add_padding now takes typed
    // `&[SrgbRgba]` instead of raw `&[u8]`. `SrgbRgba` is
    // `#[repr(transparent)]` + `bytemuck::Pod`, so the cast is zero-copy.
    let typed_pixels: &[ph2d_color::SrgbRgba] = bytemuck::cast_slice(&src.image.pixels);
    let result = ph2d_tool_padding::add_padding(typed_pixels, src_w, src_h, spec);
    if !result.changed {
        toasts.push(Toast::info("Padding: nothing changed"));
        return true;
    }
    // M1 (make_square precedent): cap BOTH output dims against the GPU
    // texture limit BEFORE acquiring, so an over-size pad surfaces a
    // clear toast instead of a deferred device-loss.
    let max_dim = renderer.max_texture_dimension_2d();
    if result.width > max_dim || result.height > max_dim {
        toasts.push(Toast::error(format!(
            "Padding would exceed GPU texture limit ({} px max, would need {} × {} px)",
            max_dim, result.width, result.height
        )));
        return true;
    }
    let new_size_world = [
        result.width as f32 / px_per_m,
        result.height as f32 / px_per_m,
    ];
    // `(dx, dy)` = world-meter offset of the ORIGINAL content's center
    // from the NEW canvas center (signed; handles both pad and crop —
    // `recenter_after_pad`'s `PixelBounds` is unsigned and can't, so the
    // math is inlined). The original content quad must stay world-fixed
    // in BOTH pivot modes: its center sits at `old_pivot + old_anchor`,
    // and after the resize the content center within the new quad is
    // `new_quad_center + (dx, dy)`. So the new quad center must land at
    // `old_pivot + old_anchor - (dx, dy)` — that invariant is what keeps
    // the existing pixels from sliding on screen.
    let (nw, nh) = (result.width as f32, result.height as f32);
    let center_px_x = result.pivot_delta_x as f32 + src_w as f32 * 0.5;
    let center_px_y = result.pivot_delta_y as f32 + src_h as f32 * 0.5;
    let dx = new_size_world[0] * (center_px_x / nw - 0.5);
    // Y-up flip (pixel space is Y-down).
    let dy = new_size_world[1] * (0.5 - center_px_y / nh);
    let [ox, oy] = src.old_translation;
    let [ax, ay] = src.old_anchor;
    // Pivot mode (panel toggle), with the new quad center fixed at
    // `(ox + ax - dx, oy + ay - dy)`:
    //  - `recenter_pivot = true` (default): move the PIVOT onto the new
    //    quad center, so the sprite stays strictly centered (`anchor =
    //    0`). Reduces to the historical `old - (dx, dy)` when the sprite
    //    had no prior anchor.
    //  - `recenter_pivot = false` (Keep): leave the PIVOT where it is and
    //    push the offset into the anchor instead, so BOTH the content and
    //    the pivot stay world-fixed while only the transparent borders
    //    grow asymmetrically.
    let (new_translation, new_anchor) = if recenter_pivot {
        ([ox + ax - dx, oy + ay - dy], [0.0, 0.0])
    } else {
        (src.old_translation, [ax - dx, ay - dy])
    };
    // Color-agnostic resize (transparent border / crop): PRESERVE the
    // source alpha mode so a premultiplied BG-Removal result survives
    // byte-exact — the chokepoint re-derives `Sprite.premultiplied`.
    let edited =
        ph2d_render::SpriteImage::new(result.width, result.height, result.pixels, src.image.alpha);
    match texture_edit::commit_edited_texture(entity, sim, renderer, &edited, new_size_world) {
        Err(err) => {
            toasts.push(Toast::error(format!("Padding failed: {err}")));
            true
        }
        Ok(texture_id) => {
            if let Some(mut transform) = sim.world_mut().get_mut::<ph2d_ecs::Transform>(entity) {
                transform.translation.x = new_translation[0];
                transform.translation.y = new_translation[1];
            }
            // Keep mode pushes the resize offset into the pivot anchor
            // (Recenter resets it to centered). `commit_edited_texture`
            // doesn't touch `anchor`, so set it here.
            if let Some(mut sprite) = sim.world_mut().get_mut::<Sprite>(entity) {
                sprite.anchor = new_anchor;
            }
            pending_undo_entries.push(ImageEditSnapshot {
                entity_bits,
                pre_source: src.old_source,
                pre_size: src.old_size_world,
                pre_translation: src.old_translation,
                pre_premultiplied: src.old_premultiplied,
                pre_anchor: src.old_anchor,
                post_individual_id: texture_id,
                label: "Padding",
            });
            toasts.push(Toast::success(format!(
                "Padded · {} × {} px · Cmd+Z to undo",
                result.width, result.height
            )));
            true
        }
    }
}
