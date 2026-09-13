//! **Fase do quadro: AS LEITURAS DA FÍSICA que os instantâneos publicam** — quantos joints um Paste atinge (§12), o
//! readout e o veredito do player selecionado (§14), as alças do gizmo de ponto e o encaixe do arrasto de âncora.
//! Fase-filha da [`super`] (`fase_snapshots_publish`), que as passa ao `snapshots::publish` (OBRA 3 da
//! `line/render-bodies`, 2026-09-13).
//!
//! ⚠️ Eram argumentos a meio da lista da chamada; aqui correm ANTES dela. Nenhum escreve nada — as quatro famílias
//! de alças, o `player_view`/`player_liveness` da ponte e a contagem só LEEM o mundo, a ponte e a seleção —, logo a
//! ordem nova não se observa.

use super::*;

/// O que a fase devolve, com o nome do argumento do `snapshots::publish` que recebe cada valor.
pub(super) struct SnapshotReadouts {
    pub(super) joint_paste_targets: usize,
    pub(super) player_live: Option<ph2d_physics_ecs::PlayerView>,
    pub(super) player_law: ph2d_physics_ecs::PlayerLiveness,
    pub(super) joint_anchor_handles: Vec<ph2d_editor_core::gizmo::PointHandle>,
    pub(super) joint_anchor_snap: Option<[f32; 2]>,
}

impl crate::App {
    /// Ver o cabeçalho do módulo.
    pub(super) fn fase_snapshot_readouts(
        &mut self,
        window_size: ph2d_host::WindowSize,
    ) -> Option<SnapshotReadouts> {
        // O `gfx` re-derivado, com os mesmos guardas da fase-mãe.
        let gfx = self.gfx.as_mut()?;
        let FrameGfx {
            sim,
            camera,
            hero_screen,
            physics,
            ..
        } = FrameGfx::of(gfx);
        let hero = hero_screen.as_ref()?;
        // W-JointCopy: quantos joints um Paste atingiria. `0` sem nada
        // copiado — e é o zero que tira o botão da tela.
        //
        // ⚠️ Contado sobre a SELEÇÃO, porque o Paste é a única edição da
        // §12 que faz fan-out; e contando só quem de fato carrega um
        // `PhysicsJoint`, senão o rótulo prometeria dez alvos numa
        // seleção de nove sprites e um joint.
        let joint_paste_targets = if self.joint_clipboard.is_some() {
            hero.gizmo
                .iter_selected()
                .filter(|&b| {
                    sim.world()
                        .get::<ph2d_physics_ecs::PhysicsJoint>(ph2d_ecs::Entity::from_bits(b))
                        .is_some()
                })
                .count()
        } else {
            0
        };
        // `W-PlayerOut` A3: o readout do player SELECIONADO. Resolvido
        // aqui porque `publish` não recebe a ponte, e pela porta única —
        // `None` fora de um player, e também com a física desarmada, que
        // é o que faz a §14 dizer *"not simulating"* em vez de mostrar
        // números de uma corrida que acabou.
        let player_live = hero
            .gizmo
            .selection
            .and_then(|b| physics.player_view(ph2d_ecs::Entity::from_bits(b)))
            .copied();
        // **O que a LEI de facto lê deste personagem** — resolvido aqui
        // pela mesma razão do readout acima (`publish` não recebe a
        // ponte) e pela MESMA porta que decide quem escreve a pose. A
        // shell re-derivá-lo do `PlayerMode` era a segunda cópia que
        // fazia a §14 pintar doze cards vivos sobre um player ASSADO,
        // que a lei não dirige.
        let player_law = hero
            .gizmo
            .selection
            .map_or(ph2d_physics_ecs::PlayerLiveness::INERT, |b| {
                physics.player_liveness(sim.world(), ph2d_ecs::Entity::from_bits(b))
            });
        // W-J2/W-J2b: every grabbable joint anchor. Resolved HERE
        // because `publish` does not take the bridge, and through the
        // SAME door `sync_joint_pivots` uses for the A pivot — two
        // derivations of "where is this anchor" is how two dots would
        // come to disagree. Rest-only (the rule lives in the callee):
        // during play the overlay draws the SOLVER's anchors, and these
        // authored ones would describe a pose the artist is not editing.
        let joint_anchor_handles = {
            // As DUAS famílias numa lista só: as âncoras (sempre) e os
            // grips de parâmetro (só com o overlay de joints na tela —
            // eles agarram a geometria DELE).
            let at_rest = !self.playhead.is_playing();
            let mut hs = point_gizmo::joint_anchor_handles(sim, physics, at_rest);
            hs.extend(point_gizmo::joint_param_handles(
                physics,
                camera,
                window_size,
                self.show_colliders,
                at_rest,
            ));
            // E as alças da RODA selecionada (W-Pulley W1). Terceira
            // família, e a única que lê a SELEÇÃO: uma corda com seis
            // roldanas publicaria doze alças sobrepostas.
            hs.extend(point_gizmo::wheel_handles(
                sim,
                hero.gizmo.selection,
                self.show_colliders,
                at_rest,
            ));
            // E a QUARTA: os limitadores da corda (W-RopeStop). De toda
            // polia, como as âncoras — a marca É a feature, e escondê-la
            // atrás de uma seleção faria o artista ter de descobrir que
            // ela existe antes de poder descobri-la.
            hs.extend(point_gizmo::rope_stop_handles(
                sim,
                physics,
                self.show_colliders,
                at_rest,
            ));
            hs
        };
        // The candidate a live anchor drag has caught (the crosshair).
        let joint_anchor_snap = self.physics.joint_anchor_drag.and_then(|d| d.snap);
        Some(SnapshotReadouts {
            joint_paste_targets,
            player_live,
            player_law,
            joint_anchor_handles,
            joint_anchor_snap,
        })
    }
}
