//! **Fase do quadro: AS EDIÇÕES DA FÍSICA NO INSPECTOR** — §12 Physics Joint, §14 Platform Player e §13 Pulley
//! Wheel: cada edição escreve o componente no lugar, e o undo global por-diff captura o passo (OBRA 2 da `line/render-loop`, 2026-09-12).

use super::*;
use ph2d_i18n::tr_with;

/// As edições de física que o dreno do barramento recolheu neste quadro.
pub(super) struct PhysicsEditIntents {
    pub(super) joint_edits: Vec<(u64, ph2d_editor_core::JointFieldEdit)>,
    pub(super) wheel_edits: Vec<(u64, ph2d_editor_core::WheelFieldEdit)>,
    pub(super) player_edits: Vec<(u64, ph2d_editor_core::PlayerFieldEdit)>,
    pub(super) join_draw_arm: bool,
}

impl crate::App {
    /// Ver o cabeçalho do módulo.
    pub(super) fn fase_physics_edits(&mut self, intents: PhysicsEditIntents) {
        // O `gfx` re-derivado; os guardas do quadro já correram na `fase_chrome_clock`.
        let Some(gfx) = self.gfx.as_mut() else {
            return;
        };
        let FrameGfx {
            sim,
            toasts,
            component_registry,
            editor_queue,
            physics,
            ..
        } = FrameGfx::of(gfx);
        let PhysicsEditIntents {
            joint_edits,
            wheel_edits,
            player_edits,
            join_draw_arm,
        } = intents;
        // §12 Physics Joint (W3) — the joint edits and the one gesture that
        // creates a joint. Deletion is NOT here: a joint is an object, so
        // "Delete Joint" despawns it through the same path any other object
        // takes, and gets the same undo step for free.
        // «Same as …» (Joint Editor W2): a afinação de uma junta de grupo ligado vale para as irmãs.
        let joint_edits = ph2d_app_physics::joint_assembly_card::with_linked(sim, &joint_edits);
        for &(bits, edit) in &joint_edits {
            // `PickBodyA/B` never reach here — they arm a canvas pick in the
            // action loop above (shell state, not a component edit). `Remove`
            // despawns the joint object; everything else is a field edit.
            if matches!(edit, ph2d_editor_core::JointFieldEdit::Remove) {
                let e = ph2d_ecs::Entity::from_bits(bits);
                let _ = sim.world_mut().despawn(e);
            } else if let ph2d_editor_core::JointFieldEdit::Linked(on) = edit {
                let e = ph2d_ecs::Entity::from_bits(bits);
                let _ = ph2d_app_physics::joint_assembly_card::set_linked(sim.world_mut(), e, on);
            } else if let ph2d_editor_core::JointFieldEdit::AnchorToWorld(on) = edit {
                let e = ph2d_ecs::Entity::from_bits(bits);
                let took = ph2d_app_physics::joint::anchor_to_world(sim.world_mut(), e, on);
                let armed = self.joint_body_pick;
                self.joint_body_pick =
                    ph2d_app_physics::joint::pick_after_anchor_b(took, bits, armed);
            } else if matches!(edit, ph2d_editor_core::JointFieldEdit::CopyProperties) {
                // ARMA a área de transferência — estado da shell, nenhum
                // componente muda (por isso não passa pelo funil nem pela
                // fila). Guarda o componente INTEIRO: quem decide o que é
                // uma *propriedade* é `with_properties_of`, no paste, e uma
                // segunda triagem aqui seria a segunda resposta que diverge.
                let e = ph2d_ecs::Entity::from_bits(bits);
                self.joint_clipboard = sim
                    .world()
                    .get::<ph2d_physics_ecs::PhysicsJoint>(e)
                    .copied();
            } else if matches!(edit, ph2d_editor_core::JointFieldEdit::PasteProperties) {
                // O fan-out já aconteceu no laço de ações: aqui é sempre UM
                // joint. A fonte é a área de transferência; sem ela o botão
                // nem foi pintado, e este ramo é um no-op honesto.
                if let Some(src) = self.joint_clipboard {
                    ph2d_app_physics::joint::paste_joint_properties(
                        sim,
                        bits,
                        &src,
                        editor_queue,
                        component_registry,
                    );
                    // FLUSH por edição, pela razão do ramo abaixo (o paste só ENFILEIRA e reescreve o
                    // componente inteiro: o fan-out sobre dez joints perderia pastes).
                    if let Err(e) = ph2d_ecs::scene::apply_editor_commands(
                        sim.world_mut(),
                        editor_queue,
                        component_registry,
                    ) {
                        toasts.push(ph2d_editor_core::Toast::error(tr_with(
                            "shell.fase_physics_edits.joint_commit_failed",
                            &[("e", &e)],
                        )));
                    }
                }
            } else if matches!(edit, ph2d_editor_core::JointFieldEdit::AddWheel) {
                // Estrutural como o `Remove`, do outro lado: SPAWNA um
                // objeto. O undo global por-diff o captura como captura
                // qualquer outro spawn, sem um passo próprio a inventar.
                ph2d_app_physics::joint_wheel::add_pulley_wheel(sim, physics, bits);
            } else {
                ph2d_app_physics::joint::apply_joint_edit(
                    sim,
                    bits,
                    edit,
                    editor_queue,
                    component_registry,
                );
                // ⚠️ FLUSH per edit, as `inspector_commits::dispatch` does: `apply_joint_edit` only QUEUES
                // a `SetComponent` (without the flush an edit waited for another to drain it), and it
                // read-modify-writes the WHOLE component, so a 2nd edit in the frame would drop the 1st.
                if let Err(e) = ph2d_ecs::scene::apply_editor_commands(
                    sim.world_mut(),
                    editor_queue,
                    component_registry,
                ) {
                    toasts.push(ph2d_editor_core::Toast::error(tr_with(
                        "shell.fase_physics_edits.joint_commit_failed",
                        &[("e", &e)],
                    )));
                }
            }
        }
        // §14 Platform Player (W5): escreve o componente no lugar; o undo por-diff captura o passo.
        for &(bits, edit) in &player_edits {
            ph2d_app_physics::inspector::player::apply_player_edit(sim, bits, edit);
        }
        // §13 Pulley Wheel (W-Pulley W1). Toda edição é de COMPONENTE — não
        // há aqui o par estrutural da §12 (criar/apagar uma roldana é criar
        // ou apagar um OBJETO, e a Hierarquia já sabe fazer os dois).
        for &(bits, edit) in &wheel_edits {
            let route_changed = ph2d_app_physics::joint_wheel::apply_wheel_edit(
                sim,
                bits,
                edit,
                editor_queue,
                component_registry,
            );
            // FLUSH por edição, pela razão que o laço do joint documenta ao
            // lado: o apply lê-modifica-escreve o componente INTEIRO, então
            // uma segunda edição no mesmo frame que lesse a primeira ainda
            // não aplicada a descartaria em silêncio.
            if let Err(e) = ph2d_ecs::scene::apply_editor_commands(
                sim.world_mut(),
                editor_queue,
                component_registry,
            ) {
                toasts.push(ph2d_editor_core::Toast::error(tr_with(
                    "shell.fase_physics_edits.wheel_commit_failed",
                    &[("e", &e)],
                )));
            }
            // ⚠️ **DEPOIS do flush, e só quando a ROTA mudou.** O `L0` da corda
            // é derivado da rota e era semeado UMA vez; digitar um raio maior
            // deixava a restrição `L(rota) <= L0` violada e o solver comia a
            // diferença num salto (medido: 14,12 m a raio 0,90). É a mesma
            // porta que o arrasto da alça usa — uma resposta, dois gestos.
            if route_changed {
                ph2d_physics_ecs::reseat_wheel_geometry(
                    sim.world_mut(),
                    ph2d_ecs::Entity::from_bits(bits),
                );
            }
        }
        if join_draw_arm {
            // TOGGLE, nao "arma" (W-J4b): o gesto e modal e come o press no
            // canvas, entao sem uma saida o unico jeito de sair era completar
            // um joint que o artista nao queria. Pela porta unica
            // `toggle_joint_draw`, a MESMA que o Esc usa.
            ph2d_app_physics::joint_draw::toggle(
                &mut self.physics.joint_draw_armed,
                &mut self.physics.joint_draw,
            );
        }
    }
}
