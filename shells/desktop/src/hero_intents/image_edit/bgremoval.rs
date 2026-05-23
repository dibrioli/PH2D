//! Drain Bg Removal — single-sprite + Separate Islands spawn.

use std::collections::BTreeMap;

use ph2d_asset::{AssetDb, AssetId};
use ph2d_ecs::SimWorld;
use ph2d_editor::{Toast, ToastQueue};
use ph2d_render::SpriteRenderer;
use ph2d_tool_bgremoval::BgRemovalTool;

use crate::hero_intents::texture_edit;
use crate::{EPS_PIXELS_PER_METER, ImageEditSnapshot, drop_undo_pre_source_if_individual};

/// Drain a `pending_bgremoval` Tool Action: run the BgRemoval
/// algorithm at the sprite's full resolution and swap to a fresh
/// Individual texture with the same dimensions (alpha-only mutation;
/// never crops). Caller gates on `bgremoval_active` so the active
/// tool can be downcast to `BgRemovalTool`. Resets the per-frame
/// snapshot push tracker so the next preview tick reflects the new
/// pixels.
///
/// When the tool's `params.separate_islands` is on, after the bake
/// the tool emits per-island RGBA payloads via
/// `take_pending_islands()`. We drain them here:
/// - `islands[0]` (largest by pixel count) replaces the current
///   sprite's pixels (cropped to its bbox + repositioned by centre).
/// - `islands[1..]` spawn one new sprite each, named
///   `"{base}_island{N}"` (N starts at 2), positioned by their bbox
///   centre in world space.
///
/// Returns `true` if a toast was pushed (caller marks title dirty).
#[allow(clippy::too_many_arguments)]
pub(crate) fn drain_bgremoval(
    entity_bits: u64,
    project_pixels_per_meter: f32,
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
    let px_per_m = project_pixels_per_meter.max(EPS_PIXELS_PER_METER);
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
    let old_anchor = src.old_anchor;
    // Segmentation reasons about TRUE colours → feed it straight alpha
    // (a re-run on an already-baked premultiplied sprite is recovered here).
    let straight = src.image.into_straight();
    let mut out: Vec<u8> = Vec::new();
    bg.set_source_snapshot(straight.pixels, straight.width, straight.height);
    let (out_w, out_h) = bg.run_full_resolution(&mut out);
    // After the bake, the tool may have produced per-island RGBA
    // payloads (when `params.separate_islands` is on). Drain them
    // here so the largest one replaces the current sprite's pixels
    // (cropped to its bbox) and the rest spawn as new sprites — port
    // of the legacy engine's "Separate Islands" affordance.
    let islands = bg.take_pending_islands();

    if islands.is_empty() {
        // Standard "knock out the background, keep the sprite as one
        // object" path — full-resolution bake replaces the texture
        // byte-faithful (dims preserved, alpha-only).
        //
        // Fringe fix: the algorithm emits STRAIGHT-alpha RGBA (the
        // anti-aliased edge band is real line-art, never altered). Bake
        // it PREMULTIPLIED so the sprite shader's bilinear sample
        // composites the edge like the Vello preview
        // (premultiply-before-sample) — no purple/dark fringe.
        let edited =
            ph2d_render::SpriteImage::new(out_w, out_h, out, ph2d_render::AlphaMode::Straight)
                .into_premultiplied();
        return match texture_edit::commit_edited_texture(
            entity,
            sim,
            renderer,
            &edited,
            old_size_world,
        ) {
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
                    pre_anchor: old_anchor,
                    post_individual_id: texture_id,
                    label: "Bg Removal",
                });
                toasts.push(Toast::success("Bg Removal applied · Cmd+Z to undo"));
                *last_bgremoval_pushed_entity = None;
                true
            }
        };
    }

    // ── Separate Islands path ──────────────────────────────────────
    // `islands` is sorted by `pixel_count` DESC (contract).
    // - `islands[0]` replaces the current sprite's pixels (cropped to
    //   its bbox + repositioned by its centre).
    // - `islands[1..]` spawn one new sprite each, named
    //   "{base}_island{N}" (N starts at 2).
    //
    // Bake premultiplied for all (mirror the no-island branch).
    let bake_premul = |w: u32, h: u32, rgba: Vec<u8>| {
        ph2d_render::SpriteImage::new(w, h, rgba, ph2d_render::AlphaMode::Straight)
            .into_premultiplied()
    };
    // Pixel-space (top-left, Y-down) bbox centre → world translation:
    // sprite pixels span world `[old.x - sx/2, old.x + sx/2]` (X) and
    // `[old.y - sy/2, old.y + sy/2]` (Y-up). Pixel (i, j) maps to
    // world `(old.x - sx/2 + (i+0.5)/ppm, old.y + sy/2 - (j+0.5)/ppm)`.
    let island_world_centre = |x: u32, y: u32, w: u32, h: u32| -> [f32; 2] {
        let cx_px = x as f32 + w as f32 * 0.5;
        let cy_px = y as f32 + h as f32 * 0.5;
        [
            old_translation[0] - old_size_world[0] * 0.5 + cx_px / px_per_m,
            old_translation[1] + old_size_world[1] * 0.5 - cy_px / px_per_m,
        ]
    };

    // Borrow the base name BEFORE any spawn — `world_mut().get::<Name>`
    // and `spawn` need disjoint borrows.
    let base_name: String = sim
        .world()
        .get::<ph2d_ecs::Name>(entity)
        .map(|n| n.as_str().to_string())
        .unwrap_or_else(|| format!("sprite_{entity_bits:x}"));

    // Replace current sprite's texture with islands[0] cropped pixels.
    let largest = &islands[0];
    let largest_centre = island_world_centre(largest.x, largest.y, largest.w, largest.h);
    let largest_world = [largest.w as f32 / px_per_m, largest.h as f32 / px_per_m];
    let edited = bake_premul(largest.w, largest.h, largest.rgba.clone());
    match texture_edit::commit_edited_texture(entity, sim, renderer, &edited, largest_world) {
        Err(err) => {
            toasts.push(Toast::error(format!("Bg Removal failed: {err}")));
            return true;
        }
        Ok(texture_id) => {
            if let Some(mut t) = sim.world_mut().get_mut::<ph2d_ecs::Transform>(entity) {
                t.translation.x = largest_centre[0];
                t.translation.y = largest_centre[1];
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
                label: "Bg Removal · Separate Islands",
            });
        }
    }

    // Spawn islands[1..] as new sprites at their respective centres.
    let mut spawned = 0usize;
    let mut spawn_failed = 0usize;
    for (i, island) in islands.iter().enumerate().skip(1) {
        let n = i + 1; // "_island2" for index 1, etc. (contract).
        let centre = island_world_centre(island.x, island.y, island.w, island.h);
        let img = bake_premul(island.w, island.h, island.rgba.clone());
        match renderer.acquire_individual(img.width, img.height, &img.pixels) {
            Err(e) => {
                spawn_failed += 1;
                toasts.push(Toast::error(format!("Island {n} acquire failed: {e}")));
            }
            Ok(tex_id) => {
                let world_w = island.w as f32 / px_per_m;
                let world_h = island.h as f32 / px_per_m;
                let mut transform = ph2d_ecs::Transform::default();
                transform.translation.x = centre[0];
                transform.translation.y = centre[1];
                let mut sprite = ph2d_render::Sprite::individual(
                    tex_id,
                    [world_w, world_h],
                    [1.0, 1.0, 1.0, 1.0],
                );
                sprite.premultiplied = true;
                sim.world_mut().spawn((
                    transform,
                    sprite,
                    ph2d_ecs::Name::new(format!("{base_name}_island{n}")),
                ));
                spawned += 1;
            }
        }
    }

    let msg = if spawn_failed == 0 {
        format!(
            "Bg Removal · {} island(s), {} spawned · Cmd+Z restores",
            islands.len(),
            spawned
        )
    } else {
        format!(
            "Bg Removal · {} island(s), {} spawned, {} failed · Cmd+Z restores",
            islands.len(),
            spawned,
            spawn_failed
        )
    };
    toasts.push(Toast::success(msg));
    *last_bgremoval_pushed_entity = None;
    true
}
