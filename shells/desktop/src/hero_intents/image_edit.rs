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
use ph2d_editor::{Toast, ToastQueue};
use ph2d_render::{Sprite, SpriteRenderer};
use ph2d_tool_bgremoval::BgRemovalTool;

use super::texture_edit;
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
            drop_undo_pre_source_if_individual(renderer, image_edit_undo);
            *image_edit_undo = Some(ImageEditSnapshot {
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
/// Mirrors [`drain_make_square`] (texture chokepoint + undo snapshot +
/// `max_texture_dimension_2d` cap). The recenter fix-up uses the
/// `PaddingResult::pivot_delta_*` directly — it's the signed shift of
/// the original content's top-left inside the new canvas, so the
/// recenter formula (same as `recenter_after_pad`, but accepting a
/// signed offset for the crop case) keeps the content world-fixed for
/// pure-expand, pure-crop, AND mixed specs.
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
    image_edit_undo: &mut Option<ImageEditSnapshot>,
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
    let result = ph2d_tool_padding::add_padding(&src.image.pixels, src_w, src_h, spec);
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
            drop_undo_pre_source_if_individual(renderer, image_edit_undo);
            *image_edit_undo = Some(ImageEditSnapshot {
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

/// Drain one `EditorAction::OneShotImageOp { tool_id:
/// "color_equalization", entity_bits }`: push the sprite's source
/// RGBA into the active `ColorEqualizationTool` and run the full-res
/// pipeline (CLAHE + brightness/contrast/saturation + auto-WB). The
/// caller iterates a `Vec<u64>` from `color_equalization_bridge` (one
/// drain per selected sprite) so multi-select Apply works.
///
/// Same shape as `drain_bgremoval` but with PRESERVE-alpha output:
/// Color EQ only edits the RGB channels, never crops or scales, so
/// dimensions match the source and the alpha mode round-trips
/// straight → straight.
///
/// Returns `true` if a toast was pushed (caller marks title dirty).
#[allow(clippy::too_many_arguments)]
pub(crate) fn drain_color_equalization(
    entity_bits: u64,
    sim: &mut SimWorld,
    renderer: &mut SpriteRenderer,
    asset_db: &AssetDb,
    atlas_asset_map: &BTreeMap<u32, AssetId>,
    toasts: &mut ToastQueue,
    image_edit_undo: &mut Option<ImageEditSnapshot>,
    ceq: &mut ph2d_tool_color_equalization::ColorEqualizationTool,
) -> bool {
    let entity = ph2d_ecs::Entity::from_bits(entity_bits);
    let Some(src) =
        texture_edit::read_sprite_source(entity, sim, renderer, asset_db, atlas_asset_map)
    else {
        toasts.push(Toast::error(
            "Color EQ: source unavailable (Atlas key missing or readback failed)",
        ));
        return true;
    };
    let old_size_world = src.old_size_world;
    let old_source = src.old_source;
    let old_translation = src.old_translation;
    let old_premultiplied = src.old_premultiplied;
    let old_anchor = src.old_anchor;
    // Color EQ reasons about TRUE colors in straight-alpha sRGB →
    // un-premultiply if the sprite was baked premultiplied (mirror of
    // bgremoval). Dimensions are preserved, alpha is preserved.
    let straight = src.image.into_straight();
    let (w, h) = (straight.width, straight.height);
    let mut out: Vec<u8> = Vec::new();
    ceq.set_source_snapshot(straight.pixels, w, h);
    let (out_w, out_h) = ceq.run_full_resolution(&mut out);
    debug_assert_eq!((out_w, out_h), (w, h), "Color EQ must preserve dimensions");
    // Keep the OUTPUT in straight-alpha — Color EQ leaves alpha
    // untouched, so we round-trip exactly. The chokepoint flips
    // `Sprite.premultiplied` to match.
    let edited = ph2d_render::SpriteImage::new(out_w, out_h, out, ph2d_render::AlphaMode::Straight);
    match texture_edit::commit_edited_texture(entity, sim, renderer, &edited, old_size_world) {
        Err(err) => {
            toasts.push(Toast::error(format!("Color EQ failed: {err}")));
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
                label: "Color EQ",
            });
            toasts.push(Toast::success("Color EQ applied · Cmd+Z to undo"));
            true
        }
    }
}

/// Drain the Equalize Sizes Apply latch over the multi-selection.
///
/// Cross-sprite shape: collects per-entity `SpriteInput`s
/// (`{ rgba, w, h, scale_x, scale_y }`) from `bits_list`, runs
/// [`EqualizeSizesTool::run_full_resolution_multi`] in one batch (Max
/// mode needs the global max, so per-sprite broadcast doesn't work),
/// then iterates the outputs back into each entity. Two output shapes:
///
/// - **Buffer changed** (`rasterize_after = true`, or upscale-if-smaller
///   grew the source): swap to a fresh Individual texture via
///   [`texture_edit::commit_edited_texture`] + write the new
///   `Sprite.size` AND reset `Transform.scale` to the signed unit from
///   the output.
/// - **Fit-by-scale only** (`rasterize_after = false`, no upscale):
///   write `Transform.scale.x/y = new_scale_x/y`; the texture stays
///   untouched.
///
/// Single-level undo captures the LAST successful sprite (consistent
/// with `drain_color_equalization`; cross-sprite undo is M14.x scope).
///
/// Returns `true` if a toast was pushed (caller marks title dirty).
#[allow(clippy::too_many_arguments)]
pub(crate) fn drain_equalize_sizes(
    bits_list: &[u64],
    project_pixels_per_meter: f32,
    sim: &mut SimWorld,
    renderer: &mut SpriteRenderer,
    asset_db: &AssetDb,
    atlas_asset_map: &BTreeMap<u32, AssetId>,
    toasts: &mut ToastQueue,
    image_edit_undo: &mut Option<ImageEditSnapshot>,
    eqs: &mut ph2d_tool_equalize_sizes::EqualizeSizesTool,
) -> bool {
    if bits_list.is_empty() {
        return false;
    }
    let px_per_m = project_pixels_per_meter.max(EPS_PIXELS_PER_METER);
    let max_dim = renderer.max_texture_dimension_2d();

    // Phase 1: gather per-entity inputs in the original `bits_list`
    // order. We pair each `SpriteInput` with the source-read metadata
    // (`old_*` + alpha mode) so the commit phase can rebuild the
    // `SpriteImage` + restore the undo snapshot without re-reading.
    struct Entry {
        bits: u64,
        src: texture_edit::SourceRead,
        // The straight-alpha pixel buffer fed to the tool; reused
        // verbatim into the output `SpriteImage` whenever the algorithm
        // didn't realloc (fit-by-scale, no upscale).
        input_alpha: ph2d_render::AlphaMode,
    }
    let mut entries: Vec<Entry> = Vec::with_capacity(bits_list.len());
    let mut inputs: Vec<ph2d_tool_equalize_sizes::SpriteInput> =
        Vec::with_capacity(bits_list.len());
    let mut skipped = 0usize;
    for &bits in bits_list {
        let entity = ph2d_ecs::Entity::from_bits(bits);
        let Some((scale_x, scale_y)) = sim
            .world()
            .get::<ph2d_ecs::Transform>(entity)
            .map(|t| (t.scale.x, t.scale.y))
        else {
            skipped += 1;
            continue;
        };
        let Some(src) =
            texture_edit::read_sprite_source(entity, sim, renderer, asset_db, atlas_asset_map)
        else {
            skipped += 1;
            continue;
        };
        // Convert to straight-alpha for the resample kernels: Lanczos3 /
        // Mitchell-Netravali interpolate RGBA per-channel; resampling a
        // premultiplied buffer would attenuate edges where alpha drops.
        // We round-trip back to the source alpha mode at commit (below).
        let source_alpha = src.image.alpha;
        let straight = src.image.into_straight();
        let (w, h) = (straight.width, straight.height);
        let input = ph2d_tool_equalize_sizes::SpriteInput {
            rgba: straight.pixels,
            width: w,
            height: h,
            scale_x,
            scale_y,
        };
        // Re-wrap the straight buffer back into a `SourceRead` shell so
        // the commit phase still has the `old_*` fields — but we keep a
        // separate handle to the original alpha mode (the round-trip
        // target) since `src.image` was moved.
        let src_shell = texture_edit::SourceRead {
            image: ph2d_render::SpriteImage::new(
                w,
                h,
                Vec::new(),
                ph2d_render::AlphaMode::Straight,
            ),
            old_size_world: src.old_size_world,
            old_translation: src.old_translation,
            old_source: src.old_source,
            old_premultiplied: src.old_premultiplied,
            old_anchor: src.old_anchor,
        };
        entries.push(Entry {
            bits,
            src: src_shell,
            input_alpha: source_alpha,
        });
        inputs.push(input);
    }
    if entries.is_empty() {
        toasts.push(Toast::error(
            "Equalize Sizes: no eligible sprites in selection",
        ));
        return true;
    }

    // Phase 2: cross-sprite bake.
    let outputs = eqs.run_full_resolution_multi(&inputs);
    debug_assert_eq!(outputs.len(), entries.len());

    // Phase 3: commit per-entity. We need a separate counter for
    // "actually changed" because `out.changed = false` skips quietly.
    let mut applied = 0usize;
    let mut last_success: Option<(u64, texture_edit::SourceRead, u32, bool)> = None;
    for (entry, out) in entries.into_iter().zip(outputs.into_iter()) {
        if !out.changed {
            continue;
        }
        let entity = ph2d_ecs::Entity::from_bits(entry.bits);
        let buffer_changed =
            (out.width, out.height) != (entry.src.image.width, entry.src.image.height);
        let mut produced_individual: Option<u32> = None;
        if buffer_changed {
            if out.width > max_dim || out.height > max_dim {
                toasts.push(Toast::error(format!(
                    "Equalize Sizes: target exceeds GPU texture limit ({} px max, would need {} × {} px)",
                    max_dim, out.width, out.height
                )));
                continue;
            }
            // Round-trip back to source alpha mode at the chokepoint —
            // `commit_edited_texture` derives `Sprite.premultiplied`
            // from `img.alpha`, so we hand the straight-from-resample
            // buffer in straight if the source was straight, or
            // re-premultiplied otherwise.
            let edited_straight = ph2d_render::SpriteImage::new(
                out.width,
                out.height,
                out.rgba,
                ph2d_render::AlphaMode::Straight,
            );
            let edited = if entry.input_alpha.is_premultiplied() {
                edited_straight.into_premultiplied()
            } else {
                edited_straight
            };
            let new_size_world = [out.width as f32 / px_per_m, out.height as f32 / px_per_m];
            match texture_edit::commit_edited_texture(
                entity,
                sim,
                renderer,
                &edited,
                new_size_world,
            ) {
                Err(err) => {
                    toasts.push(Toast::error(format!("Equalize Sizes failed: {err}")));
                    continue;
                }
                Ok(texture_id) => {
                    produced_individual = Some(texture_id);
                }
            }
        }
        if let Some(mut t) = sim.world_mut().get_mut::<ph2d_ecs::Transform>(entity) {
            t.scale.x = out.new_scale_x;
            t.scale.y = out.new_scale_y;
        }
        applied += 1;
        if let Some(tex_id) = produced_individual {
            last_success = Some((entry.bits, entry.src, tex_id, true));
        } else if last_success.is_none() {
            // Fit-by-scale only — preserve enough metadata for an undo
            // that restores the scale (no texture id since none was
            // produced). We mark this as "no-individual" so the undo
            // drainer skips the texture release.
            last_success = Some((entry.bits, entry.src, 0, false));
        }
    }

    if applied == 0 {
        toasts.push(Toast::info("Equalize Sizes: nothing to change"));
        return true;
    }

    // Single-level undo: snapshot the LAST successful sprite. Cross-
    // sprite undo (a stack of N snapshots) is M14.x scope.
    if let Some((bits, src, texture_id, had_individual)) = last_success {
        drop_undo_pre_source_if_individual(renderer, image_edit_undo);
        *image_edit_undo = Some(ImageEditSnapshot {
            entity_bits: bits,
            pre_source: src.old_source,
            pre_size: src.old_size_world,
            pre_translation: src.old_translation,
            pre_premultiplied: src.old_premultiplied,
            pre_anchor: src.old_anchor,
            // `0` is a sentinel for fit-by-scale (no individual texture
            // was acquired); the undo drainer's
            // `drop_undo_pre_source_if_individual` guard already
            // tolerates a missing acquire.
            post_individual_id: if had_individual { texture_id } else { 0 },
            label: "Equalize Sizes",
        });
    }

    if skipped > 0 {
        toasts.push(Toast::success(format!(
            "Equalize Sizes · {} sprite(s), {} skipped · Cmd+Z undoes last",
            applied, skipped
        )));
    } else if applied == 1 {
        toasts.push(Toast::success(
            "Equalize Sizes applied · Cmd+Z to undo",
        ));
    } else {
        toasts.push(Toast::success(format!(
            "Equalize Sizes · {} sprites · Cmd+Z undoes last",
            applied
        )));
    }
    true
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
    let Some(src) =
        texture_edit::read_sprite_source(entity, sim, renderer, asset_db, atlas_asset_map)
    else {
        toasts.push(Toast::error(
            "Bg Removal: source unavailable (Atlas key missing or readback failed)",
        ));
        return true;
    };
    let old_size_world = src.old_size_world;
    let old_source = src.old_source;
    let old_translation = src.old_translation;
    let old_premultiplied = src.old_premultiplied;
    // Segmentation reasons about TRUE colours → feed it straight alpha
    // (a re-run on an already-baked premultiplied sprite is recovered here).
    let straight = src.image.into_straight();
    let mut out: Vec<u8> = Vec::new();
    bg.set_source_snapshot(straight.pixels, straight.width, straight.height);
    let (out_w, out_h) = bg.run_full_resolution(&mut out);
    // Fringe fix: the algorithm emits STRAIGHT-alpha RGBA (the anti-aliased
    // edge band is real line-art, never altered). Bake it PREMULTIPLIED so
    // the sprite shader's bilinear sample composites the edge like the
    // Vello preview (premultiply-before-sample) — no purple/dark fringe.
    // The chokepoint flips `Sprite.premultiplied` from this image's mode;
    // dimensions are preserved (alpha-only edit).
    let edited = ph2d_render::SpriteImage::new(out_w, out_h, out, ph2d_render::AlphaMode::Straight)
        .into_premultiplied();
    match texture_edit::commit_edited_texture(entity, sim, renderer, &edited, old_size_world) {
        Err(err) => {
            toasts.push(Toast::error(format!("Bg Removal failed: {err}")));
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
                pre_anchor: src.old_anchor,
                post_individual_id: texture_id,
                label: "Bg Removal",
            });
            toasts.push(Toast::success("Bg Removal applied · Cmd+Z to undo"));
            *last_bgremoval_pushed_entity = None;
            true
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
                sprite.anchor = snap.pre_anchor;
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
