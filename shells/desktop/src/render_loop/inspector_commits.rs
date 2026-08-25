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

/// Os nomes canónicos que o commit da §2 pode gravar (ADR-0164 F1 passo 6). ⚠️ **Literais e não
/// `type_name`**: o registo é indexado pelo nome CANÓNICO, que é dado à mão no `register_*` e não
/// derivado do tipo Rust — renomear o módulo não pode mudar o que o arquivo diz.
const SPRITE_TYPE: &str = "ph2d::render::Sprite";
const GRID_TYPE: &str = "ph2d::ecs::SpriteGrid";
const REGION_TYPE: &str = "ph2d::ecs::SpriteRegion";
const CORNER_TINT_TYPE: &str = "ph2d::ecs::SpriteCornerTint";

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
    slice_edits: &[(u64, ph2d_editor::SliceFieldEdit)],
    anchor_edits: &[(u64, ph2d_editor::AnchorFieldEdit)],
    anim_edits: &[(u64, ph2d_editor::AnimFieldEdit)],
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
        let Some(mut editables) =
            super::inspector_commits_sprite::SpriteEditables::read(sim.world(), entity)
        else {
            continue;
        };
        // `Sprite.premultiplied` is `#[serde(skip)]` — a runtime hint set
        // by BG-Removal Apply, NOT on the wire. The SetComponent round
        // trip (postcard → from_bytes) would reset it to `false` and
        // silently reintroduce the straight-alpha edge fringe. Capture
        // the live flag and re-assert it after the commit (audit F1).
        let was_premultiplied = editables.sprite.premultiplied;
        let target = super::inspector_commits_sprite::apply_sprite_field(&mut editables, edit);
        // ⚠️ **Retirar um componente não passa pelo `SetComponent`** (ADR-0164 F1 passo 6):
        // desligar a região é uma AUSÊNCIA, e o comando que grava bytes não sabe exprimi-la.
        // O `remove` é direto no mundo, e o passo de undo nasce do diff do quadro como sempre.
        if target == super::inspector_commits_sprite::SpriteEditTarget::RegionRemoved {
            sim.world_mut()
                .entity_mut(entity)
                .remove::<ph2d_ecs::SpriteRegion>();
            continue;
        }
        let encoded = match target {
            super::inspector_commits_sprite::SpriteEditTarget::Sprite => {
                postcard::to_allocvec(&editables.sprite).map(|d| (SPRITE_TYPE, d))
            }
            super::inspector_commits_sprite::SpriteEditTarget::Grid => {
                postcard::to_allocvec(&editables.grid).map(|d| (GRID_TYPE, d))
            }
            super::inspector_commits_sprite::SpriteEditTarget::Region => {
                postcard::to_allocvec(&editables.region.expect("o alvo Region garante-a"))
                    .map(|d| (REGION_TYPE, d))
            }
            super::inspector_commits_sprite::SpriteEditTarget::CornerTint => {
                postcard::to_allocvec(&editables.corner_tint).map(|d| (CORNER_TINT_TYPE, d))
            }
            super::inspector_commits_sprite::SpriteEditTarget::RegionRemoved => unreachable!(),
        };
        match encoded {
            Ok((type_name, data)) => {
                let type_id = if type_name == SPRITE_TYPE {
                    sprite_type_id
                } else {
                    ph2d_ecs::scene::stable_type_id(type_name)
                };
                let push_res = editor_queue.push(EditorCommand::SetComponent {
                    entity: entity_bits,
                    type_id,
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
    // §12 Sockets / Named Anchors (ADR-0072). ⚠️ O commit pode RECUSAR (nome inválido,
    // repetido, cap de 64) e devolve um aviso — recusar em silêncio faria o artista escrever um
    // nome, ver a lista não mudar, e não saber porquê.
    for (entity_bits, edit) in anchor_edits {
        if let Some(t) = super::inspector_anchor::apply_anchor_edit(
            sim,
            *entity_bits,
            edit,
            editor_queue,
            component_registry,
            hero.project.pixels_per_meter,
        ) {
            toasts.push(t);
            continue;
        }
        if let Err(e) = apply_editor_commands(sim.world_mut(), editor_queue, component_registry) {
            toasts.push(Toast::error(format!("Anchor commit failed: {e}")));
            title_dirty = true;
        }
    }
    // §11 Animation (spec Sprite 08). ⚠️ Recusa com aviso, como a §12 — um nome repetido tem de
    // dizer porquê, senão o artista escreve e vê a lista não mudar.
    for (entity_bits, edit) in anim_edits {
        if let Some(t) = super::inspector_anim::apply_anim_edit(
            sim,
            *entity_bits,
            edit,
            editor_queue,
            component_registry,
        ) {
            toasts.push(t);
            continue;
        }
        if let Err(e) = apply_editor_commands(sim.world_mut(), editor_queue, component_registry) {
            toasts.push(Toast::error(format!("Animation commit failed: {e}")));
            title_dirty = true;
        }
    }
    // §5 9-Slice — a autoria de 9-slice (componente opcional; `Attach`/`Detach` são edições
    // como as outras). Ver `inspector_slice`.
    for &(entity_bits, edit) in slice_edits {
        super::inspector_slice::apply_slice_edit(
            sim,
            entity_bits,
            edit,
            editor_queue,
            component_registry,
        );
        if let Err(e) = apply_editor_commands(sim.world_mut(), editor_queue, component_registry) {
            toasts.push(Toast::error(format!("9-Slice commit failed: {e}")));
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
    use super::super::inspector_commits_sprite::{
        SpriteEditTarget, SpriteEditables, apply_sprite_field, clamp_frame,
    };
    use ph2d_ecs::{SpriteCornerTint, SpriteGrid};
    use ph2d_editor::SpriteFieldEdit;
    use ph2d_render::Sprite;

    /// Os quatro editáveis no estado neutro — o que uma sprite sem nenhum dos três componentes
    /// apresenta ao commit (ADR-0164 F1 passo 6).
    fn editables() -> SpriteEditables {
        SpriteEditables {
            sprite: Sprite::atlas(0, [1.0, 1.0], [1.0, 1.0, 1.0, 1.0]),
            grid: SpriteGrid::SINGLE,
            region: None,
            corner_tint: SpriteCornerTint::IDENTITY,
        }
    }

    #[test]
    fn flip_edits_set_the_flags() {
        let mut t = editables();
        assert_eq!(
            apply_sprite_field(&mut t, SpriteFieldEdit::FlipX(true)),
            SpriteEditTarget::Sprite
        );
        apply_sprite_field(&mut t, SpriteFieldEdit::FlipY(true));
        assert!(t.sprite.flip_x && t.sprite.flip_y);
        apply_sprite_field(&mut t, SpriteFieldEdit::FlipX(false));
        assert!(!t.sprite.flip_x && t.sprite.flip_y);
    }

    #[test]
    fn opacity_is_clamped_to_unit() {
        let mut t = editables();
        apply_sprite_field(&mut t, SpriteFieldEdit::Opacity(2.5));
        assert_eq!(t.sprite.opacity, 1.0);
        apply_sprite_field(&mut t, SpriteFieldEdit::Opacity(-0.3));
        assert_eq!(t.sprite.opacity, 0.0);
    }

    #[test]
    fn frame_count_floors_at_one_and_reclamps_frame() {
        let mut t = editables();
        assert_eq!(
            apply_sprite_field(&mut t, SpriteFieldEdit::Hframes(4)),
            SpriteEditTarget::Grid,
            "a grelha e' o alvo, nao a Sprite"
        );
        apply_sprite_field(&mut t, SpriteFieldEdit::Vframes(2));
        apply_sprite_field(&mut t, SpriteFieldEdit::Frame(7)); // last cell of 4*2
        assert_eq!(t.grid.frame, 7);
        // Shrinking the grid must drag the stale frame back in-range.
        apply_sprite_field(&mut t, SpriteFieldEdit::Vframes(1)); // now 4 cells
        assert_eq!(t.grid.frame, 3);
        // 0 is floored to 1 (never a zero-cell sheet).
        apply_sprite_field(&mut t, SpriteFieldEdit::Hframes(0));
        assert_eq!(t.grid.hframes, 1);
        assert_eq!(t.grid.frame, 0); // 1*1 = 1 cell → frame 0
    }

    #[test]
    fn frame_set_past_grid_is_clamped_immediately() {
        let mut t = editables();
        // default hframes=vframes=1 → only cell is 0.
        apply_sprite_field(&mut t, SpriteFieldEdit::Frame(99));
        assert_eq!(t.grid.frame, 0);
    }

    #[test]
    fn region_rect_clamps_extent_non_negative_but_keeps_origin() {
        let mut t = editables();
        apply_sprite_field(
            &mut t,
            SpriteFieldEdit::RegionRect([-4.0, -2.0, -10.0, 8.0]),
        );
        // x/y pass through (extract clamps into the source); w/h floor at 0.
        assert_eq!(t.region.expect("materializou").rect, [-4.0, -2.0, 0.0, 8.0]);
    }

    /// ⭐ **Ligar/desligar a janela é anexar/retirar o componente** (ADR-0164 F1 passo 6) — o
    /// antigo `region_enabled`, dito da única maneira que ele hoje se diz.
    #[test]
    fn the_region_toggle_is_the_components_presence() {
        let mut t = editables();
        assert_eq!(
            apply_sprite_field(&mut t, SpriteFieldEdit::RegionEnabled(true)),
            SpriteEditTarget::Region
        );
        assert!(t.region.is_some());
        assert_eq!(
            apply_sprite_field(&mut t, SpriteFieldEdit::RegionEnabled(false)),
            SpriteEditTarget::RegionRemoved,
            "desligar tem de pedir a REMOCAO — um SetComponent nao sabe exprimir ausencia"
        );
        assert!(t.region.is_none());
    }

    /// ⚠️ **Uma edição de campo da região MATERIALIZA o componente**, em vez de ser um no-op: o
    /// painel ainda mostra as linhas a toda sprite, e um campo que aceita o gesto e não faz nada
    /// é o defeito que a DIRETIVA §2 proíbe.
    #[test]
    fn a_region_field_edit_materialises_the_component() {
        let mut t = editables();
        assert!(t.region.is_none());
        apply_sprite_field(&mut t, SpriteFieldEdit::RegionW(12.0));
        assert_eq!(t.region.expect("materializou").rect[2], 12.0);
        // E o `filter_clip` sai da FONTE dos pixels — esta e' uma sprite de Atlas.
        assert!(t.region.expect("regiao").filter_clip);
    }

    #[test]
    fn per_axis_edits_preserve_the_other_components() {
        // BulkSelect D-1: editing one axis must NOT touch the siblings
        // (so a bulk edit of one axis can't stomp a diverging sibling).
        let mut t = editables();
        t.sprite.offset = [3.0, 5.0];
        apply_sprite_field(&mut t, SpriteFieldEdit::OffsetX(9.0));
        assert_eq!(t.sprite.offset, [9.0, 5.0], "OffsetX left Y untouched");

        t.region = Some(ph2d_ecs::SpriteRegion::for_atlas([1.0, 2.0, 3.0, 4.0]));
        apply_sprite_field(&mut t, SpriteFieldEdit::RegionY(8.0));
        assert_eq!(
            t.region.expect("regiao").rect,
            [1.0, 8.0, 3.0, 4.0],
            "RegionY left X/W/H"
        );
        // W/H still floor at 0 per-axis.
        apply_sprite_field(&mut t, SpriteFieldEdit::RegionW(-7.0));
        assert_eq!(
            t.region.expect("regiao").rect,
            [1.0, 8.0, 0.0, 4.0],
            "RegionW floored, rest kept"
        );
    }

    #[test]
    fn clamp_frame_is_idempotent_in_range() {
        let mut g = SpriteGrid {
            hframes: 3,
            vframes: 3,
            frame: 4,
        };
        clamp_frame(&mut g);
        assert_eq!(g.frame, 4);
    }
}
