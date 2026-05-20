//! Image-edit drains: trim_transparency, make_square, bgremoval,
//! undo_image_edit. All four read the sprite's source RGBA (via Atlas
//! key lookup or Individual readback), apply the algorithm, push the
//! result to a fresh Individual texture, and store an
//! [`ImageEditSnapshot`] so Cmd+Z can restore. Each returns `true`
//! iff a toast was pushed (caller marks `title_dirty`).
//!
//! Wave 3.1 stage A — extracted from `hero_intents.rs` as part of
//! the HR-18 closeout split. Behavior-preserving lift.

use std::collections::BTreeMap;

use ph2d_asset::{AssetDb, AssetId};
use ph2d_ecs::SimWorld;
use ph2d_editor::tools::bgremoval::BgRemovalTool;
use ph2d_editor::{Toast, ToastQueue};
use ph2d_render::{Sprite, SpriteRenderer};

use crate::{EPS_PIXELS_PER_METER, ImageEditSnapshot, drop_undo_pre_source_if_individual};

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
    image_edit_undo: &mut Option<ImageEditSnapshot>,
) -> bool {
    let entity = ph2d_ecs::Entity::from_bits(entity_bits);
    let px_per_m = project_pixels_per_meter.max(EPS_PIXELS_PER_METER);
    let snapshot = {
        let world = sim.world();
        world.get::<Sprite>(entity).and_then(|sprite| {
            let old_size_world = sprite.size;
            let old_source = sprite.source;
            let old_premultiplied = sprite.premultiplied;
            let old_translation = world
                .get::<ph2d_ecs::Transform>(entity)
                .map(|t| [t.translation.x, t.translation.y])
                .unwrap_or([0.0, 0.0]);
            match sprite.source {
                ph2d_render::SpriteSource::Atlas { key } => {
                    let aid = atlas_asset_map.get(&key)?;
                    let asset = asset_db.get(aid)?;
                    match &*asset {
                        ph2d_asset::Asset::ImageRgba8 {
                            width,
                            height,
                            pixels,
                        } => Some((
                            *width,
                            *height,
                            pixels.clone(),
                            old_size_world,
                            old_translation,
                            old_source,
                            old_premultiplied,
                        )),
                        _ => None,
                    }
                }
                ph2d_render::SpriteSource::Individual { texture_id } => {
                    match renderer.readback_individual(texture_id) {
                        Ok((w, h, mut pixels)) => {
                            // If the source texture is premultiplied
                            // (e.g. a prior BG-Removal Apply), recover
                            // straight alpha — the Trim algorithm scans
                            // for transparent borders and expects
                            // straight RGBA.
                            if old_premultiplied {
                                ph2d_render::unpremultiply_rgba8(&mut pixels);
                            }
                            Some((
                                w,
                                h,
                                pixels.into(),
                                old_size_world,
                                old_translation,
                                old_source,
                                old_premultiplied,
                            ))
                        }
                        Err(_) => None,
                    }
                }
            }
        })
    };
    match snapshot {
        None => {
            toasts.push(Toast::error(ph2d_i18n::tr(
                "tool.trim_transparency.toast.unavailable",
            )));
            true
        }
        Some((
            width,
            height,
            pixels,
            old_size_world,
            old_translation,
            old_source,
            old_premultiplied,
        )) => {
            let result = ph2d_editor::tools::trim_transparency(&pixels, width, height, 0);
            if !result.trimmed {
                toasts.push(Toast::info(ph2d_i18n::tr(
                    "tool.trim_transparency.toast.nothing",
                )));
                true
            } else {
                match renderer.acquire_individual(result.width, result.height, &result.pixels) {
                    Err(err) => {
                        toasts.push(Toast::error(format!("Trim failed: {err}")));
                        true
                    }
                    Ok(texture_id) => {
                        let new_size = [
                            result.width as f32 / px_per_m,
                            result.height as f32 / px_per_m,
                        ];
                        let new_translation = ph2d_editor::image_edit::recenter_after_crop(
                            old_translation,
                            old_size_world,
                            [width, height],
                            ph2d_editor::image_edit::PixelBounds::from_trim(result.bounds.clone()),
                        );
                        let sim_w = sim.world_mut();
                        if let Some(mut sprite) = sim_w.get_mut::<Sprite>(entity) {
                            sprite.source = ph2d_render::SpriteSource::Individual { texture_id };
                            sprite.size = new_size;
                            // Trim re-uploads straight-alpha pixels (we
                            // un-premultiplied above if the source was
                            // premultiplied), so the result is straight.
                            sprite.premultiplied = false;
                        }
                        if let Some(mut transform) = sim_w.get_mut::<ph2d_ecs::Transform>(entity) {
                            transform.translation.x = new_translation[0];
                            transform.translation.y = new_translation[1];
                        }
                        drop_undo_pre_source_if_individual(renderer, image_edit_undo);
                        *image_edit_undo = Some(ImageEditSnapshot {
                            entity_bits,
                            pre_source: old_source,
                            pre_size: old_size_world,
                            pre_translation: old_translation,
                            pre_premultiplied: old_premultiplied,
                            post_individual_id: texture_id,
                            label: "Trim",
                        });
                        toasts.push(Toast::success(format!(
                            "Trimmed → {} × {} px · Cmd+Z to undo",
                            result.width, result.height
                        )));
                        true
                    }
                }
            }
        }
    }
}

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
    let snapshot = {
        let world = sim.world();
        world.get::<Sprite>(entity).and_then(|sprite| {
            let old_size_world = sprite.size;
            let old_source = sprite.source;
            let old_premultiplied = sprite.premultiplied;
            let old_translation = world
                .get::<ph2d_ecs::Transform>(entity)
                .map(|t| [t.translation.x, t.translation.y])
                .unwrap_or([0.0, 0.0]);
            match sprite.source {
                ph2d_render::SpriteSource::Atlas { key } => {
                    let aid = atlas_asset_map.get(&key)?;
                    let asset = asset_db.get(aid)?;
                    match &*asset {
                        ph2d_asset::Asset::ImageRgba8 {
                            width,
                            height,
                            pixels,
                        } => Some((
                            *width,
                            *height,
                            pixels.clone(),
                            old_size_world,
                            old_translation,
                            old_source,
                            old_premultiplied,
                        )),
                        _ => None,
                    }
                }
                ph2d_render::SpriteSource::Individual { texture_id } => {
                    match renderer.readback_individual(texture_id) {
                        Ok((w, h, mut pixels)) => {
                            // Recover straight alpha if the source is a
                            // premultiplied BG-Removal bake — Make-Square
                            // pads with transparent straight RGBA.
                            if old_premultiplied {
                                ph2d_render::unpremultiply_rgba8(&mut pixels);
                            }
                            Some((
                                w,
                                h,
                                pixels.into(),
                                old_size_world,
                                old_translation,
                                old_source,
                                old_premultiplied,
                            ))
                        }
                        Err(_) => None,
                    }
                }
            }
        })
    };
    match snapshot {
        None => {
            toasts.push(Toast::error(ph2d_i18n::tr(
                "tool.make_square.toast.unavailable",
            )));
            true
        }
        Some((
            width,
            height,
            pixels,
            old_size_world,
            old_translation,
            old_source,
            old_premultiplied,
        )) => {
            let result = ph2d_editor::tools::make_square(&pixels, width, height);
            if !result.made_square {
                toasts.push(Toast::info(ph2d_i18n::tr(
                    "tool.make_square.toast.already_square",
                )));
                true
            } else if result.size > renderer.max_texture_dimension_2d() {
                toasts.push(Toast::error(format!(
                    "Make Square would exceed GPU texture limit ({} px max, would need {} px)",
                    renderer.max_texture_dimension_2d(),
                    result.size,
                )));
                true
            } else {
                match renderer.acquire_individual(result.size, result.size, &result.pixels) {
                    Err(err) => {
                        toasts.push(Toast::error(format!("Make Square failed: {err}")));
                        true
                    }
                    Ok(texture_id) => {
                        let new_side = result.size as f32 / px_per_m;
                        let new_translation = ph2d_editor::image_edit::recenter_after_pad(
                            old_translation,
                            [new_side, new_side],
                            [result.size, result.size],
                            ph2d_editor::image_edit::PixelBounds {
                                x: result.offset_x,
                                y: result.offset_y,
                                width,
                                height,
                            },
                        );
                        let sim_w = sim.world_mut();
                        if let Some(mut sprite) = sim_w.get_mut::<Sprite>(entity) {
                            sprite.source = ph2d_render::SpriteSource::Individual { texture_id };
                            sprite.size = [new_side, new_side];
                            // Padded result is straight-alpha RGBA.
                            sprite.premultiplied = false;
                        }
                        if let Some(mut transform) = sim_w.get_mut::<ph2d_ecs::Transform>(entity) {
                            transform.translation.x = new_translation[0];
                            transform.translation.y = new_translation[1];
                        }
                        drop_undo_pre_source_if_individual(renderer, image_edit_undo);
                        *image_edit_undo = Some(ImageEditSnapshot {
                            entity_bits,
                            pre_source: old_source,
                            pre_size: old_size_world,
                            pre_translation: old_translation,
                            pre_premultiplied: old_premultiplied,
                            post_individual_id: texture_id,
                            label: "Make square",
                        });
                        toasts.push(Toast::success(format!(
                            "Made square → {} × {} px · Cmd+Z to undo",
                            result.size, result.size
                        )));
                        true
                    }
                }
            }
        }
    }
}

/// Drain a `pending_bgremoval` Tool Action: run the BgRemoval
/// algorithm at the sprite's full resolution and swap to a fresh
/// Individual texture with the same dimensions (alpha-only mutation;
/// never crops). Caller gates on `bgremoval_active` so the active
/// tool can be downcast to `BgRemovalTool`. Resets the per-frame
/// snapshot push tracker so the next preview tick reflects the new
/// pixels.
///
/// Returns `true` if a toast was pushed (caller marks title dirty).
#[allow(clippy::too_many_arguments)]
pub(crate) fn drain_bgremoval(
    entity_bits: u64,
    sim: &mut SimWorld,
    renderer: &mut SpriteRenderer,
    asset_db: &AssetDb,
    atlas_asset_map: &BTreeMap<u32, AssetId>,
    toasts: &mut ToastQueue,
    image_edit_undo: &mut Option<ImageEditSnapshot>,
    bg: &mut BgRemovalTool,
    last_bgremoval_pushed_entity: &mut Option<u64>,
) -> bool {
    let entity = ph2d_ecs::Entity::from_bits(entity_bits);
    let snapshot = {
        let world = sim.world();
        world.get::<Sprite>(entity).and_then(|sprite| {
            let old_size_world = sprite.size;
            let old_source = sprite.source;
            let old_premultiplied = sprite.premultiplied;
            let old_translation = world
                .get::<ph2d_ecs::Transform>(entity)
                .map(|t| [t.translation.x, t.translation.y])
                .unwrap_or([0.0, 0.0]);
            match sprite.source {
                ph2d_render::SpriteSource::Atlas { key } => {
                    let aid = atlas_asset_map.get(&key)?;
                    let asset = asset_db.get(aid)?;
                    match &*asset {
                        ph2d_asset::Asset::ImageRgba8 {
                            width,
                            height,
                            pixels,
                        } => Some((
                            *width,
                            *height,
                            pixels.clone(),
                            old_size_world,
                            old_translation,
                            old_source,
                            old_premultiplied,
                        )),
                        _ => None,
                    }
                }
                ph2d_render::SpriteSource::Individual { texture_id } => {
                    match renderer.readback_individual(texture_id) {
                        Ok((w, h, mut pixels)) => {
                            // Re-running BG-Removal on an already-baked
                            // (premultiplied) sprite: recover straight
                            // alpha so the segmentation sees true colours.
                            if old_premultiplied {
                                ph2d_render::unpremultiply_rgba8(&mut pixels);
                            }
                            Some((
                                w,
                                h,
                                pixels.into(),
                                old_size_world,
                                old_translation,
                                old_source,
                                old_premultiplied,
                            ))
                        }
                        Err(_) => None,
                    }
                }
            }
        })
    };
    match snapshot {
        None => {
            toasts.push(Toast::error(
                "Bg Removal: source unavailable (Atlas key missing or readback failed)",
            ));
            true
        }
        Some((
            width,
            height,
            pixels,
            old_size_world,
            old_translation,
            old_source,
            old_premultiplied,
        )) => {
            let mut out: Vec<u8> = Vec::new();
            bg.set_source_snapshot(pixels.to_vec(), width, height);
            let (out_w, out_h) = bg.run_full_resolution(&mut out);
            let _ = (width, height); // shadowed by out_*; silence unused.
            // Fringe fix: the algorithm emits STRAIGHT-alpha RGBA (the
            // anti-aliased edge band is real line-art and must NOT be
            // altered). We premultiply it before upload so the sprite
            // shader's bilinear sample composites the edge band exactly
            // like the Vello on-canvas preview (premultiply-before-
            // sample) — no purple/dark fringe. The matching
            // `Sprite::premultiplied = true` below flips the per-instance
            // shader flag, and the Image-Tools readback paths
            // un-premultiply to recover the straight art.
            ph2d_render::premultiply_rgba8(&mut out);
            match renderer.acquire_individual(out_w, out_h, &out) {
                Err(err) => {
                    toasts.push(Toast::error(format!("Bg Removal failed: {err}")));
                    true
                }
                Ok(texture_id) => {
                    let sim_w = sim.world_mut();
                    if let Some(mut sprite) = sim_w.get_mut::<Sprite>(entity) {
                        sprite.source = ph2d_render::SpriteSource::Individual { texture_id };
                        // The baked texture is premultiplied (fringe fix).
                        sprite.premultiplied = true;
                        // Dimensions preserved — size stays.
                    }
                    drop_undo_pre_source_if_individual(renderer, image_edit_undo);
                    *image_edit_undo = Some(ImageEditSnapshot {
                        entity_bits,
                        pre_source: old_source,
                        pre_size: old_size_world,
                        pre_translation: old_translation,
                        pre_premultiplied: old_premultiplied,
                        post_individual_id: texture_id,
                        label: "Bg Removal",
                    });
                    toasts.push(Toast::success("Bg Removal applied · Cmd+Z to undo"));
                    *last_bgremoval_pushed_entity = None;
                    true
                }
            }
        }
    }
}

/// Drain a `pending_undo_image_edit` flag: restore the previous
/// `ImageEditSnapshot` (Sprite source / size / Transform translation),
/// release the post-edit texture so it doesn't leak. Single-level
/// undo by design — see SKILL §14 image-edit notes.
///
/// Returns `true` if a toast was pushed (caller marks title dirty).
pub(crate) fn drain_undo_image_edit(
    image_edit_undo: &mut Option<ImageEditSnapshot>,
    sim: &mut SimWorld,
    renderer: &mut SpriteRenderer,
    toasts: &mut ToastQueue,
) -> bool {
    match image_edit_undo.take() {
        None => {
            toasts.push(Toast::info(ph2d_i18n::tr(
                "edit.undo.image_edit.toast_nothing_to_undo",
            )));
            true
        }
        Some(snap) => {
            let entity = ph2d_ecs::Entity::from_bits(snap.entity_bits);
            let sim_w = sim.world_mut();
            if let Some(mut sprite) = sim_w.get_mut::<Sprite>(entity) {
                sprite.source = snap.pre_source;
                sprite.size = snap.pre_size;
                sprite.premultiplied = snap.pre_premultiplied;
            }
            if let Some(mut transform) = sim_w.get_mut::<ph2d_ecs::Transform>(entity) {
                transform.translation.x = snap.pre_translation[0];
                transform.translation.y = snap.pre_translation[1];
            }
            renderer.individual_mut().release(snap.post_individual_id);
            toasts.push(Toast::success(format!(
                "{} · {}",
                ph2d_i18n::tr("edit.undo.image_edit.toast_done"),
                snap.label,
            )));
            true
        }
    }
}
