//! **Fase do quadro: A ESTRATÉGIA DE ORIGEM E O RE-ASSENTO DO PIVÔ** — a troca de estratégia de origem da sprite
//! (`inspector_strategy::dispatch`, que precisa do atlas MUTÁVEL) e o re-assento da âncora A de um joint depois de
//! um commit de Position, pela MESMA porta do dot de canvas (OBRA 2 da `line/render-loop`, 2026-09-12).
//!
//! ⚠️ O `joint_pivot_commit` é capturado pelos commits do Inspector (a fase anterior) e consumido aqui.

use super::*;

/// Os pedidos da troca de estratégia de origem que o dreno do barramento recolheu neste quadro.
pub(super) struct SourceStrategyIntents {
    pub(super) sprite_source_change: Option<(u64, RequestedSpriteStrategy)>,
}

impl crate::App {
    /// Ver o cabeçalho do módulo.
    pub(super) fn fase_source_strategy_and_joint_pivot(
        &mut self,
        intents: SourceStrategyIntents,
        joint_pivot_commit: Option<(u64, [f32; 2])>,
    ) {
        // O `gfx` re-derivado; os guardas do quadro já correram na `fase_chrome_clock`.
        let Some(gfx) = self.gfx.as_mut() else {
            return;
        };
        let FrameGfx {
            renderer,
            sim,
            asset_db,
            toasts,
            hero_screen,
            next_import_cell,
            atlas_asset_map,
            component_registry,
            editor_queue,
            sprite_type_id,
            physics,
            ..
        } = FrameGfx::of(gfx);
        // O bloco do quadro só chama esta fase com o `HeroScreen` vivo.
        let Some(hero) = hero_screen.as_mut() else {
            return;
        };
        let SourceStrategyIntents {
            sprite_source_change,
        } = intents;
        // A troca de ESTRATÉGIA de origem sai por uma porta própria (irmã, pelo teto de LOC):
        // ela precisa do `atlas_asset_map` e do `next_import_cell` em modo MUTÁVEL — a volta
        // ao atlas ocupa uma célula nova —, e o `dispatch` acima recebe o mapa por leitura.
        if inspector_strategy::dispatch(
            sprite_source_change,
            hero,
            sim,
            renderer,
            asset_db,
            atlas_asset_map,
            next_import_cell,
            toasts,
            editor_queue,
            component_registry,
            *sprite_type_id,
        ) {
            self.title_dirty = true;
        }
        // W-J2: re-seat a joint's A anchor after a Position commit — the
        // SAME door the canvas handles write through, so typing a pivot and
        // dragging it mean the same thing. ⚠️ Not the `anchored` sentinel the
        // old tail cleared: that re-derives BOTH locals, so editing X for the
        // A end would silently reset the B end the artist just placed.
        // A joint is a root entity, so its local translation IS world.
        if let Some((bits, world)) = joint_pivot_commit {
            let e = ph2d_ecs::Entity::from_bits(bits);
            if sim
                .world()
                .get::<ph2d_physics_ecs::PhysicsJoint>(e)
                .is_some()
            {
                physics.set_joint_anchor_world(sim, e, ph2d_physics_ecs::JointSide::A, world);
            }
            // W6: e o mesmo commit numa RODLANA montada precisa do sentinela
            // desarmado — o centro dela também é derivado, e o
            // `sync_mounted_wheels` o reescreveria. Pela MESMA porta do dot de
            // canvas; ⚠️ aqui o sentinela É a resposta certa (uma roldana tem
            // UM eixo, então não há segunda metade a perder como no joint).
            ph2d_physics_ecs::reseat_mounted_axle(sim.world_mut(), e);
        }
    }
}
