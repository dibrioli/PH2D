//! Inspector commit phase — Transform / Visibility / Name / Sprite
//! source-strategy + Reimport drains.
//!
//! Wave 3.2 stage A — extracted from `render_loop::mod.rs` as a free
//! function. Each consumes a snapshot pre-populated by the
//! consolidated bus drain in mod.rs, encodes the new component via
//! `postcard`, pushes a `EditorCommand::SetComponent`, and applies.
//! Returns `true` iff any drain pushed a toast (caller ORs into
//! `title_dirty`).
//!
//! Behavior-preserving lift.

use crate::EPS_PIXELS_PER_METER;
use ph2d_asset::{AssetDb, AssetId};
use ph2d_ecs::scene::{
    ComponentRegistry, EditorCommand, EditorCommandQueue, apply_editor_commands,
};
use ph2d_ecs::{SimWorld, Transform, Visibility};
use ph2d_editor::{
    HeroScreen, InspectorNameInfo, InspectorTransformInfo, InspectorVisibilityInfo,
    RequestedSpriteStrategy, Toast, ToastQueue,
};
use ph2d_render::{Sprite, SpriteRenderer};
use std::collections::BTreeMap;

/// Dispatches the 5 inspector commits. Returns `true` if any pushed
/// a toast.
#[allow(clippy::too_many_arguments)]
pub(super) fn dispatch(
    reimport_entity: Option<u64>,
    transform_edit: Option<InspectorTransformInfo>,
    visibility_edit: Option<InspectorVisibilityInfo>,
    name_edit: Option<InspectorNameInfo>,
    sprite_source_change: Option<(u64, RequestedSpriteStrategy)>,
    hero: &mut HeroScreen,
    sim: &mut SimWorld,
    renderer: &mut SpriteRenderer,
    asset_db: &AssetDb,
    atlas_asset_map: &BTreeMap<u32, AssetId>,
    toasts: &mut ToastQueue,
    editor_queue: &mut EditorCommandQueue,
    component_registry: &ComponentRegistry,
    transform_type_id: u64,
    visibility_type_id: u64,
    name_type_id: u64,
    sprite_type_id: u64,
) -> bool {
    let mut title_dirty = false;

    // M14.5 inspector phase (6.4): drain Reimport intent →
    // re-decode the atlas source's pixel dimensions at the
    // current `project.pixels_per_meter` and write the new
    // world size back to the Sprite component. The texture
    // itself is unchanged; only `Sprite.size` is recomputed.
    if let Some(entity_bits) = reimport_entity {
        let entity = ph2d_ecs::Entity::from_bits(entity_bits);
        let px_per_m = hero.project.pixels_per_meter.max(EPS_PIXELS_PER_METER);
        let new_size = sim.world().get::<Sprite>(entity).and_then(|sprite| {
            let ph2d_render::SpriteSource::Atlas { key } = sprite.source else {
                return None;
            };
            let aid = atlas_asset_map.get(&key)?;
            let asset = asset_db.get(aid)?;
            match &*asset {
                ph2d_asset::Asset::ImageRgba8 { width, height, .. } => {
                    Some([*width as f32 / px_per_m, *height as f32 / px_per_m])
                }
                _ => None,
            }
        });
        if let Some(size) = new_size {
            let sim_w = sim.world_mut();
            if let Some(mut sprite) = sim_w.get_mut::<Sprite>(entity) {
                sprite.size = size;
                toasts.push(Toast::success(format!(
                    "Reimported at {:.0} px/m → {:.3} × {:.3} m",
                    px_per_m, size[0], size[1]
                )));
                title_dirty = true;
            }
        } else {
            toasts.push(Toast::error("Reimport unavailable for this source"));
            title_dirty = true;
        }
    }
    // M14.A: drain Inspector Transform commit → push
    // `EditorCommand::SetComponent` to the editor queue, then
    // apply. **First end-to-end consumer** of the editor
    // command pipeline (every prior `pending_*` field mutated
    // SimWorld directly). When MCP / Luau / multi-agent edits
    // arrive in M14.B+ they share this same code path —
    // governance, audit, conflict resolution all live one
    // level up from the producer.
    if let Some(info) = transform_edit {
        let t = Transform {
            translation: ph2d_core::Vec2::new(info.translation[0], info.translation[1]),
            rotation: info.rotation_rad,
            scale: ph2d_core::Vec2::new(info.scale[0], info.scale[1]),
        };
        match postcard::to_allocvec(&t) {
            Ok(data) => {
                let push_res = editor_queue.push(EditorCommand::SetComponent {
                    entity: info.entity_bits,
                    type_id: transform_type_id,
                    data,
                });
                if let Err(e) = push_res {
                    toasts.push(Toast::error(format!("Editor queue full: {e}")));
                    title_dirty = true;
                } else if let Err(e) =
                    apply_editor_commands(sim.world_mut(), editor_queue, component_registry)
                {
                    toasts.push(Toast::error(format!("Transform commit failed: {e}")));
                    title_dirty = true;
                }
            }
            Err(e) => {
                toasts.push(Toast::error(format!("Transform encode failed: {e}")));
                title_dirty = true;
            }
        }
    }
    // M14.D: drain Inspector Visibility commit → same
    // EditorCommandQueue path as Transform.
    if let Some(info) = visibility_edit {
        let v = Visibility {
            hidden: !info.visible,
        };
        match postcard::to_allocvec(&v) {
            Ok(data) => {
                let push_res = editor_queue.push(EditorCommand::SetComponent {
                    entity: info.entity_bits,
                    type_id: visibility_type_id,
                    data,
                });
                if let Err(e) = push_res {
                    toasts.push(Toast::error(format!("Editor queue full: {e}")));
                    title_dirty = true;
                } else if let Err(e) =
                    apply_editor_commands(sim.world_mut(), editor_queue, component_registry)
                {
                    toasts.push(Toast::error(format!("Visibility commit failed: {e}")));
                    title_dirty = true;
                }
            }
            Err(e) => {
                toasts.push(Toast::error(format!("Visibility encode failed: {e}")));
                title_dirty = true;
            }
        }
    }
    // M14.E: drain Inspector Name commit → push a
    // `Name(string)` postcard via `EditorCommand::SetComponent`.
    if let Some(info) = name_edit {
        let n = ph2d_ecs::Name(info.name.clone());
        match postcard::to_allocvec(&n) {
            Ok(data) => {
                let push_res = editor_queue.push(EditorCommand::SetComponent {
                    entity: info.entity_bits,
                    type_id: name_type_id,
                    data,
                });
                if let Err(e) = push_res {
                    toasts.push(Toast::error(format!("Editor queue full: {e}")));
                    title_dirty = true;
                } else if let Err(e) =
                    apply_editor_commands(sim.world_mut(), editor_queue, component_registry)
                {
                    toasts.push(Toast::error(format!("Name commit failed: {e}")));
                    title_dirty = true;
                }
            }
            Err(e) => {
                toasts.push(Toast::error(format!("Name encode failed: {e}")));
                title_dirty = true;
            }
        }
    }
    // M14.C: drain Render Source Strategy switch. Atlas →
    // Individual works (re-decode source pixels +
    // `acquire_individual` for the renderer, then a canonical
    // `EditorCommand::SetComponent` for the updated `Sprite` —
    // audit fix #8 puts this on the same pipeline as Transform
    // / Visibility / Name). Individual → Atlas and any
    // HandPacked transition surface a toast — atlas re-insert +
    // hand-packed asset picker land in M14.C+.
    if let Some((entity_bits, requested)) = sprite_source_change {
        let entity = ph2d_ecs::Entity::from_bits(entity_bits);
        let current_sprite = sim.world().get::<Sprite>(entity).copied();
        // Audit fix #7 helper: when a swap is rejected (toast
        // path), demote the clicked Strategy button's stored
        // state back to Normal so it doesn't keep painting
        // Pressed/Hovered alongside the still-active button.
        let reject_visual_reset = |hero: &mut HeroScreen, clicked: RequestedSpriteStrategy| {
            let id = match clicked {
                RequestedSpriteStrategy::Atlas => {
                    ph2d_editor::screens::hero::ids::INSP_RENDER_STRATEGY_ATLAS
                }
                RequestedSpriteStrategy::Individual => {
                    ph2d_editor::screens::hero::ids::INSP_RENDER_STRATEGY_INDIVIDUAL
                }
                RequestedSpriteStrategy::HandPacked => {
                    ph2d_editor::screens::hero::ids::INSP_RENDER_STRATEGY_HANDPACKED
                }
            };
            if let Some(ph2d_editor::InteractiveState::Button { state }) = hero.store.get_mut(id) {
                *state = ph2d_editor::widget::ButtonState::Normal;
            }
        };
        match (current_sprite.map(|s| s.source), requested) {
            (
                Some(ph2d_render::SpriteSource::Atlas { key }),
                RequestedSpriteStrategy::Individual,
            ) => {
                let decoded = atlas_asset_map.get(&key).and_then(|aid| {
                    asset_db.get(aid).and_then(|asset| match &*asset {
                        ph2d_asset::Asset::ImageRgba8 {
                            width,
                            height,
                            pixels,
                        } => Some((*width, *height, pixels.clone())),
                        _ => None,
                    })
                });
                match decoded {
                    Some((w, h, pixels)) => match renderer.acquire_individual(w, h, &pixels) {
                        Ok(texture_id) => {
                            // Audit fix #8: route the Sprite mutation through
                            // `EditorCommand::SetComponent` so MCP / Luau / audit-log
                            // consumers see the same flow as Transform / Name. The
                            // renderer-side `acquire_individual` already happened;
                            // what we encode is the updated `Sprite` (size + tint
                            // preserved from the snapshot).
                            let mut updated = current_sprite.unwrap_or(ph2d_render::Sprite::atlas(
                                0,
                                [1.0, 1.0],
                                [1.0, 1.0, 1.0, 1.0],
                            ));
                            updated.source = ph2d_render::SpriteSource::Individual { texture_id };
                            // Freshly-decoded image is straight alpha;
                            // clear any premultiplied flag carried over
                            // from a prior BG-Removal Apply on this sprite.
                            updated.premultiplied = false;
                            match postcard::to_allocvec(&updated) {
                                Ok(data) => {
                                    let push_res = editor_queue.push(EditorCommand::SetComponent {
                                        entity: entity_bits,
                                        type_id: sprite_type_id,
                                        data,
                                    });
                                    if let Err(e) = push_res {
                                        toasts
                                            .push(Toast::error(format!("Editor queue full: {e}")));
                                        title_dirty = true;
                                    } else if let Err(e) = apply_editor_commands(
                                        sim.world_mut(),
                                        editor_queue,
                                        component_registry,
                                    ) {
                                        toasts.push(Toast::error(format!(
                                            "Strategy commit failed: {e}"
                                        )));
                                        title_dirty = true;
                                    } else {
                                        toasts.push(Toast::success(format!(
                                            "Strategy → Individual (texture {})",
                                            texture_id
                                        )));
                                        title_dirty = true;
                                    }
                                }
                                Err(e) => {
                                    toasts.push(Toast::error(format!("Sprite encode failed: {e}")));
                                    title_dirty = true;
                                }
                            }
                        }
                        Err(err) => {
                            toasts.push(Toast::error(format!("Individual acquire failed: {err}")));
                            title_dirty = true;
                            reject_visual_reset(hero, requested);
                        }
                    },
                    None => {
                        toasts.push(Toast::error(
                            "Cannot promote to Individual — source asset missing",
                        ));
                        title_dirty = true;
                        reject_visual_reset(hero, requested);
                    }
                }
            }
            (Some(ph2d_render::SpriteSource::Atlas { .. }), RequestedSpriteStrategy::Atlas)
            | (
                Some(ph2d_render::SpriteSource::Individual { .. }),
                RequestedSpriteStrategy::Individual,
            ) => {
                // No-op: requested matches current. The `apply_event`
                // guard already short-circuits identical clicks, but
                // keep this branch explicit so an out-of-band publish
                // (script, future MCP) doesn't accidentally bounce.
            }
            (Some(_), RequestedSpriteStrategy::Atlas) => {
                toasts.push(Toast::info(
                    "Individual → Atlas swap is M14.C+ (atlas re-insert path)",
                ));
                title_dirty = true;
                reject_visual_reset(hero, requested);
            }
            (_, RequestedSpriteStrategy::HandPacked) => {
                toasts.push(Toast::info(
                    "Hand-packed strategy needs an atlas asset — M14.C+ asset picker",
                ));
                title_dirty = true;
                reject_visual_reset(hero, requested);
            }
            (None, _) => {
                // Entity vanished between commit and drain (despawn,
                // hierarchy delete) — silent no-op.
            }
        }
    }

    title_dirty
}
