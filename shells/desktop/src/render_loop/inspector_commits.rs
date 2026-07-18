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
//
// ph2d-loc-cap: 616 LOC — `dispatch` is a sequence of independent per-field
// commit drains (Transform / Visibility / Name / Sprite / Reimport) lifted
// verbatim from mod.rs, plus inline unit tests. Splitting into per-field
// sibling modules is a focused Sprite-Inspector follow-up: side-effecting
// drain code with no isolation, where a blind split risks regressions the
// gate can't catch. Pre-existing debt; tracked exception to unblock the
// accumulated W2/W3 ship (Coord ship-prep 2026-06-02).

use crate::EPS_PIXELS_PER_METER;
use ph2d_asset::{AssetDb, AssetId};
use ph2d_ecs::scene::{
    ComponentRegistry, EditorCommand, EditorCommandQueue, apply_editor_commands,
};
use ph2d_ecs::{SimWorld, Transform, Visibility};
use ph2d_editor::{
    BlendFieldEdit, HeroScreen, InspectorNameInfo, InspectorTransformInfo, InspectorVisibilityInfo,
    OrderingFieldEdit, PhysicsFieldEdit, RequestedSpriteStrategy, SamplingFieldEdit,
    SpriteFieldEdit, Toast, ToastQueue, VisibilityFieldEdit,
};
use ph2d_render::{Sprite, SpriteRenderer};
use std::collections::BTreeMap;

/// Apply one [`SpriteFieldEdit`] to a `Sprite`, enforcing the schema
/// invariants the Inspector widgets can't (anatomia §1.6): `hframes`/
/// `vframes >= 1`, `frame < hframes*vframes`, `opacity ∈ [0, 1]`. The
/// frame index is re-clamped whenever the grid shrinks so a stale frame
/// can never index past the sheet. This is the single authoring write
/// boundary for editable Sprite fields (mirrors `Transform::clamp_skew`).
fn apply_sprite_field(sprite: &mut Sprite, edit: SpriteFieldEdit) {
    match edit {
        SpriteFieldEdit::FlipX(b) => sprite.flip_x = b,
        SpriteFieldEdit::FlipY(b) => sprite.flip_y = b,
        SpriteFieldEdit::Centered(b) => sprite.centered = b,
        SpriteFieldEdit::Offset(o) => sprite.offset = o,
        // Per-axis: preserve the OTHER axis (so a bulk edit of one axis
        // can't stomp a diverging sibling — audit D-1).
        SpriteFieldEdit::OffsetX(x) => sprite.offset[0] = x,
        SpriteFieldEdit::OffsetY(y) => sprite.offset[1] = y,
        SpriteFieldEdit::Hframes(n) => {
            sprite.hframes = n.max(1);
            clamp_frame(sprite);
        }
        SpriteFieldEdit::Vframes(n) => {
            sprite.vframes = n.max(1);
            clamp_frame(sprite);
        }
        SpriteFieldEdit::Frame(f) => {
            sprite.frame = f;
            clamp_frame(sprite);
        }
        SpriteFieldEdit::RegionEnabled(b) => sprite.region_enabled = b,
        SpriteFieldEdit::RegionRect(r) => {
            // Schema invariant (anatomia §1.6): w/h kept `>= 0`. A negative
            // extent would invert the sampled UV; x/y may be negative (the
            // extract clamps the rect into the source).
            sprite.region_rect = [r[0], r[1], r[2].max(0.0), r[3].max(0.0)];
        }
        // Per-axis: preserve the other three components (audit D-1). W/H
        // floor at 0 like the whole-vector path.
        SpriteFieldEdit::RegionX(x) => sprite.region_rect[0] = x,
        SpriteFieldEdit::RegionY(y) => sprite.region_rect[1] = y,
        SpriteFieldEdit::RegionW(w) => sprite.region_rect[2] = w.max(0.0),
        SpriteFieldEdit::RegionH(h) => sprite.region_rect[3] = h.max(0.0),
        SpriteFieldEdit::RegionFilterClip(b) => sprite.region_filter_clip = b,
        SpriteFieldEdit::Tint(c) => sprite.tint = c,
        SpriteFieldEdit::SelfTint(c) => sprite.self_tint = c,
        SpriteFieldEdit::TintFill(b) => sprite.tint_fill = b,
        SpriteFieldEdit::Opacity(o) => sprite.opacity = o.clamp(0.0, 1.0),
        SpriteFieldEdit::PerCornerTint(p) => sprite.per_corner_tint = p,
    }
}

/// Clamp `frame` into `[0, hframes*vframes - 1]`. `hframes`/`vframes`
/// are always `>= 1` here (set via `apply_sprite_field`), so the grid
/// has at least one cell.
fn clamp_frame(sprite: &mut Sprite) {
    let cells = sprite.hframes.saturating_mul(sprite.vframes).max(1);
    if sprite.frame >= cells {
        sprite.frame = cells - 1;
    }
}

// §7 ordering commit handler lives in the sibling `inspector_ordering`
// module (HR-18 LOC + separation): `apply_ordering_edit`.

/// Dispatches the 5 inspector commits. Returns `true` if any pushed
/// a toast.
#[allow(clippy::too_many_arguments)]
pub(super) fn dispatch(
    reimport_entity: Option<u64>,
    transform_edit: Option<InspectorTransformInfo>,
    visibility_edit: Option<InspectorVisibilityInfo>,
    name_edit: Option<InspectorNameInfo>,
    sprite_source_change: Option<(u64, RequestedSpriteStrategy)>,
    sprite_edits: &[(u64, SpriteFieldEdit)],
    ordering_edits: &[(u64, OrderingFieldEdit)],
    sampling_edits: &[(u64, SamplingFieldEdit)],
    blend_edits: &[(u64, BlendFieldEdit)],
    physics_edits: &[(u64, PhysicsFieldEdit)],
    visibility_section_edits: &[(u64, VisibilityFieldEdit)],
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
                    "Reimported at {:.0} px/m · {:.3} × {:.3} m",
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
            // Skew authored via the Inspector Skew X/Y sliders (W2.T2.3).
            // Clamp at this ECS-write boundary (the authoring setter per
            // ADR-0025-amendment-1 §2.5) so tan() stays in its sane range.
            skew_x: Transform::clamp_skew(info.skew_rad[0]),
            skew_y: Transform::clamp_skew(info.skew_rad[1]),
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
    // W2 Sprite Inspector v2: drain editable Sprite field edits (flip,
    // region, sprite-sheet, tint channels, opacity, …). For each, read
    // the entity's current Sprite, apply the one field (clamped), and
    // commit the whole struct through the SAME SetComponent path as
    // Transform. Grouped per-entity isn't necessary — applying edits
    // sequentially re-reads the just-written Sprite each iteration.
    for &(entity_bits, edit) in sprite_edits {
        let entity = ph2d_ecs::Entity::from_bits(entity_bits);
        let Some(mut sprite) = sim.world().get::<Sprite>(entity).copied() else {
            continue;
        };
        // `Sprite.premultiplied` is `#[serde(skip)]` — a runtime hint set
        // by BG-Removal Apply, NOT on the wire. The SetComponent round
        // trip (postcard → from_bytes) would reset it to `false` and
        // silently reintroduce the straight-alpha edge fringe. Capture
        // the live flag and re-assert it after the commit (audit F1).
        let was_premultiplied = sprite.premultiplied;
        apply_sprite_field(&mut sprite, edit);
        match postcard::to_allocvec(&sprite) {
            Ok(data) => {
                let push_res = editor_queue.push(EditorCommand::SetComponent {
                    entity: entity_bits,
                    type_id: sprite_type_id,
                    data,
                });
                if let Err(e) = push_res {
                    toasts.push(Toast::error(format!("Editor queue full: {e}")));
                    title_dirty = true;
                } else if let Err(e) =
                    apply_editor_commands(sim.world_mut(), editor_queue, component_registry)
                {
                    toasts.push(Toast::error(format!("Sprite commit failed: {e}")));
                    title_dirty = true;
                } else if was_premultiplied
                    && let Some(mut s) = sim.world_mut().get_mut::<Sprite>(entity)
                {
                    // Re-assert the serde(skip) runtime hint the wire dropped.
                    s.premultiplied = true;
                }
            }
            Err(e) => {
                toasts.push(Toast::error(format!("Sprite encode failed: {e}")));
                title_dirty = true;
            }
        }
    }
    // W3 Sprite Inspector v2 §7: drain editable ordering edits. Each
    // maps to an OPTIONAL sorting component — `apply_ordering_edit`
    // queues a SetComponent (insert/update) or RemoveComponent (detach)
    // and we apply per edit so a read-modify-write field (YSort /
    // SortingGroup) re-reads the just-written component next iteration.
    for &(entity_bits, edit) in ordering_edits {
        super::inspector_ordering::apply_ordering_edit(
            sim,
            entity_bits,
            edit,
            editor_queue,
            component_registry,
        );
        if let Err(e) = apply_editor_commands(sim.world_mut(), editor_queue, component_registry) {
            toasts.push(Toast::error(format!("Ordering commit failed: {e}")));
            title_dirty = true;
        }
    }
    // W3 §9 sampling edits (TextureFilter/Repeat optional components).
    for &(entity_bits, edit) in sampling_edits {
        super::inspector_ordering::apply_sampling_edit(
            sim,
            entity_bits,
            edit,
            editor_queue,
            component_registry,
        );
        if let Err(e) = apply_editor_commands(sim.world_mut(), editor_queue, component_registry) {
            toasts.push(Toast::error(format!("Sampling commit failed: {e}")));
            title_dirty = true;
        }
    }
    // §10 Material & Blend edits (BlendMode optional component).
    for &(entity_bits, edit) in blend_edits {
        super::inspector_ordering::apply_blend_edit(
            sim,
            entity_bits,
            edit,
            editor_queue,
            component_registry,
        );
        if let Err(e) = apply_editor_commands(sim.world_mut(), editor_queue, component_registry) {
            toasts.push(Toast::error(format!("Blend commit failed: {e}")));
            title_dirty = true;
        }
    }
    for &(entity_bits, edit) in physics_edits {
        super::inspector_physics::apply_physics_edit(
            sim,
            entity_bits,
            edit,
            editor_queue,
            component_registry,
        );
        if let Err(e) = apply_editor_commands(sim.world_mut(), editor_queue, component_registry) {
            toasts.push(Toast::error(format!("Physics commit failed: {e}")));
            title_dirty = true;
        }
    }
    // W3 §8 visibility-section edits (VisibilityLayer / ClipChildren /
    // MaskInteraction / OnScreenEnabler optional components).
    for &(entity_bits, edit) in visibility_section_edits {
        super::inspector_visibility::apply_visibility_section_edit(
            sim,
            entity_bits,
            edit,
            editor_queue,
            component_registry,
        );
        if let Err(e) = apply_editor_commands(sim.world_mut(), editor_queue, component_registry) {
            toasts.push(Toast::error(format!("Visibility commit failed: {e}")));
            title_dirty = true;
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
                                            "Strategy · Individual (texture {})",
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
            // W2.T2: a cooked KTX2 source can't be re-authored into an
            // Atlas/Individual/Hand-packed strategy from the inspector (it
            // has no atlas key nor a CPU-readable bake). Reject any
            // requested change and snap the radio back. Placed before the
            // generic `(Some(_), …)` / `(_, HandPacked)` arms so all three
            // cooked-source cases get this accurate message.
            (Some(ph2d_render::SpriteSource::CookedTexture { .. }), _) => {
                toasts.push(Toast::info(
                    "Cooked textures come from the asset pipeline — render strategy is read-only",
                ));
                title_dirty = true;
                reject_visual_reset(hero, requested);
            }
            (Some(_), RequestedSpriteStrategy::Atlas) => {
                toasts.push(Toast::info(
                    "Individual · Atlas swap is M14.C+ (atlas re-insert path)",
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

#[cfg(test)]
mod sprite_field_tests {
    use super::{apply_sprite_field, clamp_frame};
    use ph2d_editor::SpriteFieldEdit;
    use ph2d_render::Sprite;

    fn sprite() -> Sprite {
        Sprite::atlas(0, [1.0, 1.0], [1.0, 1.0, 1.0, 1.0])
    }

    #[test]
    fn flip_edits_set_the_flags() {
        let mut s = sprite();
        apply_sprite_field(&mut s, SpriteFieldEdit::FlipX(true));
        apply_sprite_field(&mut s, SpriteFieldEdit::FlipY(true));
        assert!(s.flip_x && s.flip_y);
        apply_sprite_field(&mut s, SpriteFieldEdit::FlipX(false));
        assert!(!s.flip_x && s.flip_y);
    }

    #[test]
    fn opacity_is_clamped_to_unit() {
        let mut s = sprite();
        apply_sprite_field(&mut s, SpriteFieldEdit::Opacity(2.5));
        assert_eq!(s.opacity, 1.0);
        apply_sprite_field(&mut s, SpriteFieldEdit::Opacity(-0.3));
        assert_eq!(s.opacity, 0.0);
    }

    #[test]
    fn frame_count_floors_at_one_and_reclamps_frame() {
        let mut s = sprite();
        apply_sprite_field(&mut s, SpriteFieldEdit::Hframes(4));
        apply_sprite_field(&mut s, SpriteFieldEdit::Vframes(2));
        apply_sprite_field(&mut s, SpriteFieldEdit::Frame(7)); // last cell of 4*2
        assert_eq!(s.frame, 7);
        // Shrinking the grid must drag the stale frame back in-range.
        apply_sprite_field(&mut s, SpriteFieldEdit::Vframes(1)); // now 4 cells
        assert_eq!(s.frame, 3);
        // 0 is floored to 1 (never a zero-cell sheet).
        apply_sprite_field(&mut s, SpriteFieldEdit::Hframes(0));
        assert_eq!(s.hframes, 1);
        assert_eq!(s.frame, 0); // 1*1 = 1 cell → frame 0
    }

    #[test]
    fn frame_set_past_grid_is_clamped_immediately() {
        let mut s = sprite();
        // default hframes=vframes=1 → only cell is 0.
        apply_sprite_field(&mut s, SpriteFieldEdit::Frame(99));
        assert_eq!(s.frame, 0);
    }

    #[test]
    fn region_rect_clamps_extent_non_negative_but_keeps_origin() {
        let mut s = sprite();
        apply_sprite_field(
            &mut s,
            SpriteFieldEdit::RegionRect([-4.0, -2.0, -10.0, 8.0]),
        );
        // x/y pass through (extract clamps into the source); w/h floor at 0.
        assert_eq!(s.region_rect, [-4.0, -2.0, 0.0, 8.0]);
    }

    #[test]
    fn per_axis_edits_preserve_the_other_components() {
        // BulkSelect D-1: editing one axis must NOT touch the siblings
        // (so a bulk edit of one axis can't stomp a diverging sibling).
        let mut s = sprite();
        s.offset = [3.0, 5.0];
        apply_sprite_field(&mut s, SpriteFieldEdit::OffsetX(9.0));
        assert_eq!(s.offset, [9.0, 5.0], "OffsetX left Y untouched");

        s.region_rect = [1.0, 2.0, 3.0, 4.0];
        apply_sprite_field(&mut s, SpriteFieldEdit::RegionY(8.0));
        assert_eq!(s.region_rect, [1.0, 8.0, 3.0, 4.0], "RegionY left X/W/H");
        // W/H still floor at 0 per-axis.
        apply_sprite_field(&mut s, SpriteFieldEdit::RegionW(-7.0));
        assert_eq!(
            s.region_rect,
            [1.0, 8.0, 0.0, 4.0],
            "RegionW floored, rest kept"
        );
    }

    #[test]
    fn clamp_frame_is_idempotent_in_range() {
        let mut s = sprite();
        s.hframes = 3;
        s.vframes = 3;
        s.frame = 4;
        clamp_frame(&mut s);
        assert_eq!(s.frame, 4);
    }
}
