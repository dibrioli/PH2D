//! Inspector commit phase — os drains de Transform, Visibility, Name, campos
//! de Sprite e Reimport. A troca de ESTRATÉGIA de origem mudou-se para o módulo
//! irmão `inspector_strategy`; vide a nota de LOC abaixo.
//!
//! Wave 3.2 stage A — extracted from `render_loop::mod.rs` as a free
//! function. Each consumes a snapshot pre-populated by the
//! consolidated bus drain in mod.rs, encodes the new component via
//! `postcard`, pushes a `EditorCommand::SetComponent`, and applies.
//! Returns `true` iff any drain pushed a toast (caller ORs into
//! `title_dirty`).
//!
//! Behavior-preserving lift.
//!
//! ⚠️ **A exceção de LOC deste arquivo foi RETIRADA em 2026-08-19, e não movida.** Ela pedia por
//! escrito, desde 2026-06-02, *"splitting into per-field sibling modules is a focused
//! Sprite-Inspector follow-up"* — e é exatamente isso que a [`super::inspector_strategy`] é: a
//! troca de estratégia de origem saiu daqui inteira (−157 linhas), o arquivo voltou a **555** e o
//! teto de 600 do HR-18 volta a morder de verdade. *Um marcador que só precisa existir para o
//! gate passar não envelhece com barulho* — este envelheceu 36 linhas em silêncio antes de sair.

use crate::EPS_PIXELS_PER_METER;
use ph2d_asset::{AssetDb, AssetId};
use ph2d_ecs::scene::{
    ComponentRegistry, EditorCommand, EditorCommandQueue, apply_editor_commands,
};
use ph2d_ecs::{SimWorld, Transform, Visibility};
use ph2d_editor::{
    BlendFieldEdit, HeroScreen, InspectorNameInfo, InspectorTransformInfo, OrderingFieldEdit,
    PhysicsFieldEdit, SamplingFieldEdit, SpriteFieldEdit, Toast, ToastQueue, VisibilityFieldEdit,
};
use ph2d_render::Sprite;
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
    visibility_edits: &[(u64, bool)],
    name_edit: Option<InspectorNameInfo>,
    signal_edit: Option<InspectorNameInfo>,
    signal_leave_edit: Option<InspectorNameInfo>,
    sprite_edits: &[(u64, SpriteFieldEdit)],
    ordering_edits: &[(u64, OrderingFieldEdit)],
    sampling_edits: &[(u64, SamplingFieldEdit)],
    blend_edits: &[(u64, BlendFieldEdit)],
    physics_edits: &[(u64, PhysicsFieldEdit)],
    visibility_section_edits: &[(u64, VisibilityFieldEdit)],
    hero: &mut HeroScreen,
    sim: &mut SimWorld,
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
            // ⚠️ `image_dimensions` e não um `match` na variante: perguntar o TAMANHO não deve
            // exigir saber a precisão. Casar `ImageRgba8` aqui fazia o «Real size» desaparecer em
            // silêncio numa imagem de 16 bits (plano `docs/Sprite_projeto/18`, auditoria da W2).
            let (width, height) = asset.image_dimensions()?;
            Some([width as f32 / px_per_m, height as f32 / px_per_m])
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
        // ⚠️ **The joint tail moved out (W-J2).** A joint's Position edit still
        // repositions its A anchor, but it now goes through the bridge's anchor
        // door in `mod.rs` (the `joint_pivot_commit` block — never extracted
        // into the function this comment used to name) rather
        // than clearing `PhysicsJoint::anchored` here — that sentinel re-derives
        // BOTH locals, so typing a new X for the pivot would have thrown away a
        // body-B anchor the artist placed with the second handle. The door needs
        // the `PhysicsBridge` (for body A's rest pose), which this signature's
        // own doc-comment warns against growing further.
    }
    // M14.D: drain Inspector Visibility commit → same
    // EditorCommandQueue path as Transform.
    // ⚠️ **Uma FATIA, não um `Option`** — a caixa «Visible» do topo editava só a primária enquanto
    // a §8 Visibility logo abaixo espalhava pela seleção. Duas linhas vizinhas, comportamentos
    // opostos, aparência idêntica (auditoria `docs/Sprite_projeto/20` §3).
    for &(entity_bits, visible) in visibility_edits {
        let v = Visibility { hidden: !visible };
        match postcard::to_allocvec(&v) {
            Ok(data) => {
                let push_res = editor_queue.push(EditorCommand::SetComponent {
                    entity: entity_bits,
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
    // **O nome do sinal** (W-Signal), pelo MESMO pipeline canônico de componente
    // que o nome da entidade usa — fila de comandos + registro —, e não por uma
    // escrita direta no mundo: um segundo caminho para *"editar um componente"*
    // é como um deles passa a não aparecer no undo.
    //
    // ⚠️ O `type_id` sai do REGISTRO em vez de ser mais um parâmetro enfiado por
    // dez assinaturas: o registro já é argumento desta função, e ele é a fonte
    // de quem sabe o id de um componente.
    //
    // ⚠️ **Os dois extremos, e cada tipo é codificado pelo `Serialize` DELE.**
    // `SignalOnHit`/`SignalOnLeave` são newtypes de `String` e hoje codificam
    // igual — mas serializar uma string e chamá-la de componente amarraria os
    // bytes a esse acidente, e o dia em que um dos dois ganhasse um campo o
    // commit escreveria lixo bem-formado. Cada arm serializa o SEU tipo; o que é
    // partilhado é o CAMINHO, não a codificação.
    for (edit, encoded, type_name) in [
        (
            signal_edit.as_ref(),
            signal_edit
                .as_ref()
                .map(|i| postcard::to_allocvec(&ph2d_physics_ecs::SignalOnHit(i.name.clone()))),
            "ph2d::physics::SignalOnHit",
        ),
        (
            signal_leave_edit.as_ref(),
            signal_leave_edit
                .as_ref()
                .map(|i| postcard::to_allocvec(&ph2d_physics_ecs::SignalOnLeave(i.name.clone()))),
            "ph2d::physics::SignalOnLeave",
        ),
    ] {
        let (Some(info), Some(encoded)) = (edit, encoded) else {
            continue;
        };
        match (encoded, component_registry.get_by_name(type_name)) {
            (Ok(data), Some(entry)) => {
                let push_res = editor_queue.push(EditorCommand::SetComponent {
                    entity: info.entity_bits,
                    type_id: entry.type_id,
                    data,
                });
                if let Err(e) = push_res {
                    toasts.push(Toast::error(format!("Editor queue full: {e}")));
                } else if let Err(e) =
                    apply_editor_commands(sim.world_mut(), editor_queue, component_registry)
                {
                    toasts.push(Toast::error(format!("Signal commit failed: {e}")));
                }
                title_dirty = true;
            }
            (Err(e), _) => {
                toasts.push(Toast::error(format!("Signal encode failed: {e}")));
            }
            (_, None) => {
                toasts.push(Toast::error(format!("{type_name} is not registered")));
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
