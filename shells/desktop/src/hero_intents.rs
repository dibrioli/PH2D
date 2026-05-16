//! Inspector / hierarchy / image-action intent drains.
//!
//! PR 9a of `docs/Migracao/2026-05-convention-by-discovery.md`:
//! `render_frame()` in `main.rs` used to inline 20 `hero.pending_*`
//! drains spanning ~1300 LOC. Each drain is a self-contained
//! state-machine collapse: takes a one-shot intent flag from the
//! hero, applies the corresponding mutation to the ECS / asset DB /
//! renderer, pushes a toast on observable change.
//!
//! Drains are extracted here as free functions receiving the exact
//! refs they need (rather than `&mut self`). This keeps the
//! field-level split borrow that `render_frame()`'s destructure of
//! `AppGfx` already provides — passing `&mut self` would conflict
//! with the live destructure.
//!
//! Convention: each fn returns `bool` indicating whether to set
//! `self.title_dirty = true`. The caller in `render_frame()` ORs the
//! flag into a local accumulator so a single field write happens
//! after the destructure ends. PR 9 generic dispatcher will subsume
//! the four `pending_<tool_action>` drains (Trim, MakeSquare,
//! Reimport, BgRemoval) into the registry-driven path.

use std::collections::BTreeMap;

use ph2d_asset::{AssetDb, AssetId};
use ph2d_ecs::{PresentWorld, SimWorld};
use ph2d_editor::{BgRemovalTool, Toast, ToastQueue, ViewFocusKind};
use ph2d_host::WindowSize;
use ph2d_render::{Camera2d, Sprite, SpriteRenderer};

use crate::{EPS_PIXELS_PER_METER, ImageEditSnapshot, drop_undo_pre_source_if_individual};

/// Drain `hero.pending_view_focus`. Per [`ViewFocusKind`]:
///  - `Selected`: pan to gizmo_selection or (0,0).
///  - `Camera`: pan to (0,0) until camera-object exists.
///  - `All`: pan + zoom to fit every sprite (10% pad).
///
/// Returns `true` if a toast was pushed (caller marks title dirty).
pub(crate) fn drain_view_focus(
    kind: ViewFocusKind,
    gizmo_selection: Option<u64>,
    present: &mut PresentWorld,
    camera: &mut Camera2d,
    window_size: WindowSize,
    toasts: &mut ToastQueue,
) -> bool {
    let label = match kind {
        ViewFocusKind::Selected => {
            let target = gizmo_selection
                .and_then(|bits| ph2d_render::selection_bbox_world(present.world_mut(), bits));
            if let Some(bbox) = target {
                let ([cx, cy], _) = bbox.center_half();
                camera.center = [cx, cy];
                "View → Selected"
            } else {
                camera.center = [0.0, 0.0];
                "View → Selected (no selection → origin)"
            }
        }
        ViewFocusKind::Camera => {
            // No camera-object yet — frame the origin.
            camera.center = [0.0, 0.0];
            "View → Camera (origin)"
        }
        ViewFocusKind::All => {
            // Walk PresentWorld for every sprite's bbox and fit
            // camera around the union. 10% pad so handles + the bbox
            // stroke have room.
            let mut q = present
                .world_mut()
                .query::<(&ph2d_ecs::GlobalTransform, &ph2d_render::RenderInstance)>();
            let mut min_x = f32::INFINITY;
            let mut min_y = f32::INFINITY;
            let mut max_x = f32::NEG_INFINITY;
            let mut max_y = f32::NEG_INFINITY;
            let mut count = 0u32;
            for (gt, ri) in q.iter(present.world()) {
                let p = gt.translation();
                let hw = ri.size[0] * 0.5;
                let hh = ri.size[1] * 0.5;
                min_x = min_x.min(p.x - hw);
                min_y = min_y.min(p.y - hh);
                max_x = max_x.max(p.x + hw);
                max_y = max_y.max(p.y + hh);
                count += 1;
            }
            if count > 0 {
                let cx = (min_x + max_x) * 0.5;
                let cy = (min_y + max_y) * 0.5;
                let span_x = max_x - min_x;
                let span_y = max_y - min_y;
                let aspect = (window_size.width as f32) / (window_size.height.max(1) as f32);
                let need_h = span_y.max(span_x / aspect.max(1e-3));
                camera.center = [cx, cy];
                camera.height_world = (need_h * 1.1).max(0.5);
                "View → All"
            } else {
                *camera = Camera2d::default();
                "View → All (empty scene → reset)"
            }
        }
    };
    toasts.push(Toast::info(label));
    true
}

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
/// `ph2d_editor::recenter_after_crop` so the *visual* center of the
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
                        )),
                        _ => None,
                    }
                }
                ph2d_render::SpriteSource::Individual { texture_id } => {
                    match renderer.readback_individual(texture_id) {
                        Ok((w, h, pixels)) => Some((
                            w,
                            h,
                            pixels.into(),
                            old_size_world,
                            old_translation,
                            old_source,
                        )),
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
        Some((width, height, pixels, old_size_world, old_translation, old_source)) => {
            let result = ph2d_editor::trim_transparency(&pixels, width, height, 0);
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
                        let new_translation = ph2d_editor::recenter_after_crop(
                            old_translation,
                            old_size_world,
                            [width, height],
                            ph2d_editor::PixelBounds::from_trim(result.bounds.clone()),
                        );
                        let sim_w = sim.world_mut();
                        if let Some(mut sprite) = sim_w.get_mut::<Sprite>(entity) {
                            sprite.source = ph2d_render::SpriteSource::Individual { texture_id };
                            sprite.size = new_size;
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
                        )),
                        _ => None,
                    }
                }
                ph2d_render::SpriteSource::Individual { texture_id } => {
                    match renderer.readback_individual(texture_id) {
                        Ok((w, h, pixels)) => Some((
                            w,
                            h,
                            pixels.into(),
                            old_size_world,
                            old_translation,
                            old_source,
                        )),
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
        Some((width, height, pixels, old_size_world, old_translation, old_source)) => {
            let result = ph2d_editor::make_square(&pixels, width, height);
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
                        let new_translation = ph2d_editor::recenter_after_pad(
                            old_translation,
                            [new_side, new_side],
                            [result.size, result.size],
                            ph2d_editor::PixelBounds {
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
                        )),
                        _ => None,
                    }
                }
                ph2d_render::SpriteSource::Individual { texture_id } => {
                    match renderer.readback_individual(texture_id) {
                        Ok((w, h, pixels)) => Some((
                            w,
                            h,
                            pixels.into(),
                            old_size_world,
                            old_translation,
                            old_source,
                        )),
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
        Some((width, height, pixels, old_size_world, old_translation, old_source)) => {
            let mut out: Vec<u8> = Vec::new();
            bg.set_source_snapshot(pixels.to_vec(), width, height);
            let (out_w, out_h) = bg.run_full_resolution(&mut out);
            let _ = (width, height); // shadowed by out_*; silence unused.
            match renderer.acquire_individual(out_w, out_h, &out) {
                Err(err) => {
                    toasts.push(Toast::error(format!("Bg Removal failed: {err}")));
                    true
                }
                Ok(texture_id) => {
                    let sim_w = sim.world_mut();
                    if let Some(mut sprite) = sim_w.get_mut::<Sprite>(entity) {
                        sprite.source = ph2d_render::SpriteSource::Individual { texture_id };
                        // Dimensions preserved — size stays.
                    }
                    drop_undo_pre_source_if_individual(renderer, image_edit_undo);
                    *image_edit_undo = Some(ImageEditSnapshot {
                        entity_bits,
                        pre_source: old_source,
                        pre_size: old_size_world,
                        pre_translation: old_translation,
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

/// Drain a `pending_reparent` hierarchy intent: move a dragged
/// hierarchy row to a new parent (or root), positioning it relative
/// to a target sibling (before / after). Guards against cycles
/// (refuses to make dragged a descendant of itself). Re-inserts
/// `ChildOf` on every sibling in the desired order so bevy_ecs's
/// `Children` list reflects the user-chosen sequence.
///
/// Returns `false` — never pushes a toast (silent reparent matches
/// existing UX). Caller does not set title_dirty.
pub(crate) fn drain_reparent(
    intent: ph2d_editor::screens::hero::HierReparentIntent,
    live: &crate::HeroLive,
    sim: &mut SimWorld,
) -> bool {
    use ph2d_ecs::Transform;
    let Some(dragged_bits) = live.bridge.entity_for(intent.dragged) else {
        return false;
    };
    let dragged = ph2d_ecs::Entity::from_bits(dragged_bits);

    let new_parent_entity = if let Some(parent_node) = intent.new_parent
        && let Some(parent_bits) = live.bridge.entity_for(parent_node)
    {
        Some(ph2d_ecs::Entity::from_bits(parent_bits))
    } else if let Some(before_node) = intent.before
        && let Some(target_bits) = live.bridge.entity_for(before_node)
    {
        let target = ph2d_ecs::Entity::from_bits(target_bits);
        sim.world()
            .get::<ph2d_ecs::ChildOf>(target)
            .map(|c| c.parent())
    } else if let Some(after_node) = intent.after
        && let Some(target_bits) = live.bridge.entity_for(after_node)
    {
        let target = ph2d_ecs::Entity::from_bits(target_bits);
        sim.world()
            .get::<ph2d_ecs::ChildOf>(target)
            .map(|c| c.parent())
    } else {
        None
    };
    let sim_w = sim.world_mut();
    let would_cycle = new_parent_entity.is_some_and(|np| {
        let mut current = Some(np);
        while let Some(c) = current {
            if c == dragged {
                return true;
            }
            current = sim_w.get::<ph2d_ecs::ChildOf>(c).map(|c| c.parent());
        }
        false
    });
    if would_cycle {
        return false;
    }
    // Step 1: pick the new ChildOf relation.
    if let Ok(mut entry) = sim_w.get_entity_mut(dragged) {
        match new_parent_entity {
            Some(p) => {
                entry.insert(ph2d_ecs::ChildOf(p));
            }
            None => {
                entry.remove::<ph2d_ecs::ChildOf>();
            }
        }
    }
    // M14.7 polish: root drops need an explicit `RootOrder`.
    if new_parent_entity.is_none() {
        let mut roots: Vec<ph2d_ecs::Entity> = {
            let mut q = sim_w.query_filtered::<ph2d_ecs::Entity, (
                ph2d_ecs::With<Transform>,
                ph2d_ecs::Without<ph2d_ecs::ChildOf>,
            )>();
            let mut acc: Vec<(ph2d_ecs::Entity, u32)> = Vec::new();
            for entity in q.iter(sim_w) {
                if entity == dragged {
                    continue;
                }
                let order = sim_w
                    .get::<ph2d_ecs::RootOrder>(entity)
                    .map(|r| r.0)
                    .unwrap_or(u32::MAX);
                acc.push((entity, order));
            }
            acc.sort_unstable_by(|a, b| {
                a.1.cmp(&b.1)
                    .then_with(|| a.0.to_bits().cmp(&b.0.to_bits()))
            });
            acc.into_iter().map(|(e, _)| e).collect()
        };
        let before_target = intent
            .before
            .and_then(|n| live.bridge.entity_for(n))
            .map(ph2d_ecs::Entity::from_bits);
        let after_target = intent
            .after
            .and_then(|n| live.bridge.entity_for(n))
            .map(ph2d_ecs::Entity::from_bits);
        let insert_at = if let Some(b) = before_target {
            roots.iter().position(|e| *e == b).unwrap_or(roots.len())
        } else if let Some(a) = after_target {
            roots
                .iter()
                .position(|e| *e == a)
                .map(|i| i + 1)
                .unwrap_or(roots.len())
        } else {
            roots.len()
        };
        roots.insert(insert_at.min(roots.len()), dragged);
        for (idx, e) in roots.iter().enumerate() {
            if let Ok(mut entry) = sim_w.get_entity_mut(*e) {
                entry.insert(ph2d_ecs::RootOrder(idx as u32));
            }
        }
    }
    // Step 2: enforce sibling order.
    let target_kind: Option<(ph2d_ecs::Entity, bool)> = if let Some(before_node) = intent.before
        && let Some(b) = live.bridge.entity_for(before_node)
    {
        Some((ph2d_ecs::Entity::from_bits(b), true))
    } else if let Some(after_node) = intent.after
        && let Some(a) = live.bridge.entity_for(after_node)
    {
        Some((ph2d_ecs::Entity::from_bits(a), false))
    } else {
        None
    };
    if let (Some(parent), Some((target, place_before))) = (new_parent_entity, target_kind) {
        let current: Vec<ph2d_ecs::Entity> = sim_w
            .get::<bevy_ecs::hierarchy::Children>(parent)
            .map(|c| c.iter().copied().filter(|e| *e != dragged).collect())
            .unwrap_or_default();
        let mut desired: Vec<ph2d_ecs::Entity> = Vec::with_capacity(current.len() + 1);
        let mut inserted = false;
        for &c in &current {
            if !inserted && c == target && place_before {
                desired.push(dragged);
                inserted = true;
            }
            desired.push(c);
            if !inserted && c == target && !place_before {
                desired.push(dragged);
                inserted = true;
            }
        }
        if !inserted {
            desired.push(dragged);
        }
        for &child in &desired {
            if let Ok(mut entry) = sim_w.get_entity_mut(child) {
                entry.remove::<ph2d_ecs::ChildOf>();
                entry.insert(ph2d_ecs::ChildOf(parent));
            }
        }
    }
    false
}
