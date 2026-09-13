//! **Fase do quadro: O DESPACHO DA HIERARQUIA** — os verbos de linha da Hierarquia pelo `hierarchy::dispatch`, e
//! o fork da sprite duplicada para uma textura PRÓPRIA (objecto independente) (OBRA 2 da `line/render-loop`, 2026-09-12).

use super::*;

/// Os pedidos de linha da Hierarquia que o dreno do barramento recolheu neste quadro.
pub(super) struct HierarchyIntents {
    pub(super) visibility_toggle_row: Option<NodeId>,
    pub(super) lock_toggle_row: Option<NodeId>,
    pub(super) group_toggle_row: Option<NodeId>,
    pub(super) reparent_intent: Option<ph2d_editor_core::screens::hero::HierReparentIntent>,
    pub(super) duplicate_row: Option<NodeId>,
    pub(super) duplicate_made: Option<(u64, u64)>,
    pub(super) add_child_row: Option<NodeId>,
    pub(super) add_root: bool,
    pub(super) reset_transform_row: Option<NodeId>,
    pub(super) revert_to_master_row: Option<NodeId>,
    pub(super) instance_verb_row: Option<(NodeId, ph2d_app_components::instance_verbs::Verb)>,
    pub(super) instance_verb_stable_id: Option<(
        u64,
        ph2d_app_components::instance_verbs::Verb,
        Option<[f32; 2]>,
    )>,
    pub(super) asset_card_verb: Option<(
        ph2d_editor_core::interaction::drag_payload::DragPayload,
        ph2d_editor_core::action_bus::AssetCardAction,
    )>,
    pub(super) delete_row: Option<NodeId>,
    pub(super) hierarchy_row_click: Option<NodeId>,
    pub(super) hierarchy_select_intent: Option<hierarchy::HierarchySelectIntent>,
    pub(super) rename_seed_row: Option<NodeId>,
    pub(super) rename_commit: Option<(NodeId, String)>,
    pub(super) view_focus_kind: Option<ph2d_editor_core::ViewFocusKind>,
}

impl crate::App {
    /// Ver o cabeçalho do módulo.
    pub(super) fn fase_hierarchy_dispatch(
        &mut self,
        intents: HierarchyIntents,
        window_size: ph2d_host::WindowSize,
    ) {
        // O `gfx` re-derivado; os guardas do quadro já correram na `fase_chrome_clock`.
        let Some(gfx) = self.gfx.as_mut() else {
            return;
        };
        let FrameGfx {
            renderer,
            sim,
            present,
            camera,
            asset_db,
            toasts,
            tools,
            vec_scene,
            hero_screen,
            hero_live,
            atlas_asset_map,
            component_registry,
            ..
        } = FrameGfx::of(gfx);
        // O bloco do quadro só chama esta fase com o `HeroScreen` vivo.
        let Some(hero) = hero_screen.as_mut() else {
            return;
        };
        let HierarchyIntents {
            visibility_toggle_row,
            lock_toggle_row,
            group_toggle_row,
            reparent_intent,
            duplicate_row,
            mut duplicate_made,
            add_child_row,
            add_root,
            reset_transform_row,
            revert_to_master_row,
            instance_verb_row,
            instance_verb_stable_id,
            asset_card_verb,
            delete_row,
            hierarchy_row_click,
            hierarchy_select_intent,
            rename_seed_row,
            rename_commit,
            view_focus_kind,
        } = intents;
        if hierarchy::dispatch(
            view_focus_kind,
            visibility_toggle_row,
            lock_toggle_row,
            group_toggle_row,
            reparent_intent,
            duplicate_row,
            add_child_row,
            add_root,
            reset_transform_row,
            revert_to_master_row,
            instance_verb_row,
            instance_verb_stable_id,
            asset_card_verb,
            atlas_asset_map,
            delete_row,
            hierarchy_row_click,
            hierarchy_select_intent,
            rename_seed_row,
            rename_commit,
            hero,
            hero_live,
            sim,
            present,
            camera,
            toasts,
            window_size,
            vec_scene,
            &mut self.vec.entities,
            &mut self.vec.pen,
            &mut duplicate_made,
            component_registry,
            &mut self.instance_echo,
        ) {
            self.title_dirty = true;
        }
        // ⭐⭐⭐ **O *Duplicate* de uma PEÇA da escultura** (ADR-0150, 2026-09-04): a cópia
        // profunda **deixa cair** o `Sculpt3dPieceRef` — copiar o id daria duas entidades
        // sobre a mesma peça (`instance_docs::DROPPED`) —, então a linha nova nasceria vazia.
        // ⇒ regista-se um PEDIDO, e o `sculpt3d::entities::sync` do quadro seguinte duplica a
        // peça e põe o `ref` novo na cópia. *A cena está emprestada neste ponto do laço.*
        #[cfg(feature = "sculpt3d")]
        if let Some((src_bits, new_bits)) = duplicate_made
            && let Some(piece) = sim
                .world()
                .get::<ph2d_ecs::Sculpt3dPieceRef>(ph2d_ecs::Entity::from_bits(src_bits))
        {
            self.sculpt3d.dup = Some((piece.0, new_bits));
        }
        // A duplicated sprite copies the source's `Sprite` component verbatim, so it SHARES the
        // source pixels — and if the source is being painted, the unbaked paint+mask never reaches
        // either entity (the working state is dropped on the next rebind, losing the paint from
        // both). When the source has live paint: bake it so the original persists, then give the
        // copy its OWN texture (a deep copy of the now-painted result) so it is a fully independent
        // object (Enio 2026-06-24). A non-painted duplicate keeps the shared source — Atlas/Individual
        // both fork on the next edit, so they stay independent in practice and keep atlas batching.
        if let Some((src_bits, new_bits)) = duplicate_made
            && self.last_painter_pushed_entity == Some(src_bits)
            && let Some(painter) = tools.active_mut().and_then(|t| {
                t.as_any_mut()
                    .downcast_mut::<ph2d_tool_painter::PainterTool>()
            })
            && painter.has_unbaked_edits()
        {
            crate::hero_intents::auto_commit_painter(
                src_bits,
                sim,
                renderer,
                asset_db,
                atlas_asset_map,
                painter,
                toasts,
            );
            self.last_painter_pushed_entity = None; // bridge re-pushes the freshly-baked source
            let src = ph2d_ecs::Entity::from_bits(src_bits);
            let copy = ph2d_ecs::Entity::from_bits(new_bits);
            if let Some(read) = crate::hero_intents::texture_edit::read_sprite_source(
                src,
                sim,
                renderer,
                asset_db,
                atlas_asset_map,
            ) {
                let _ = crate::hero_intents::texture_edit::commit_edited_texture(
                    copy,
                    sim,
                    renderer,
                    asset_db,
                    &read.image,
                    read.old_size_world,
                    toasts,
                );
            }
        }
    }
}
