//! **O dreno do barramento — a física do Inspector.** Sub-dreno irmão de
//! `fase_bus_inspector.rs`, com a mesma forma: devolve a acção que não é dele.

use super::*;
use ph2d_editor_core::action_bus::EditorAction;

impl crate::App {
    /// O Inspector — a física (§11 corpo, §12 junta, §14 player e a roda). ⚠️ As interceptações que NÃO se espalham
    /// correm antes do fan-out de cada braço, e os quatro braços ficam juntos por isso.
    pub(super) fn fase_bus_physics_sections(
        &mut self,
        action: EditorAction,
        pd: &mut DrainOut,
    ) -> Option<EditorAction> {
        match action {
            // §11 Physics Body: espalha-se pela seleção, como as irmãs.
            EditorAction::InspectorPhysicsEdit { entity_bits, edit } => {
                // ⚠️ Join é UM gesto sobre o PAR: espalhado, faria dois joints entre os mesmos dois.
                if matches!(edit, ph2d_editor_core::PhysicsFieldEdit::Join) {
                    // ⚠️ 2 ou MAIS (W-J4): uma corrente de N−1 joints, depois do laço.
                    if pd.inspector_selection.len() >= 2 {
                        pd.join_chain = true;
                    }
                } else if matches!(edit, ph2d_editor_core::PhysicsFieldEdit::Rig) {
                    // ⚠️ Nem o Rig se espalha (W-Rig): percorre a mesma subárvore N vezes.
                    pd.rig_now = true;
                } else if matches!(edit, ph2d_editor_core::PhysicsFieldEdit::JoinDraw) {
                    // ARMA o gesto de canvas: o press e o release nomeiam os dois corpos (`input_dispatch`).
                    pd.join_draw_arm = true;
                } else if matches!(edit, ph2d_editor_core::PhysicsFieldEdit::Bake) {
                    // ⚠️ Nem o Bake: simula a cena uma vez e escreve as curvas de todos num passo só.
                    pd.bake_request = Some(if pd.inspector_selection.is_empty() {
                        vec![entity_bits]
                    } else {
                        pd.inspector_selection.clone()
                    });
                } else if let ph2d_editor_core::PhysicsFieldEdit::BakeChannels(tag) = edit {
                    // Opção global do próximo Bake, não edição por corpo.
                    self.bake_channels = ph2d_app_physics::bake::BakeChannels::from_tag(tag);
                } else if let ph2d_editor_core::PhysicsFieldEdit::JoinKind(tag) = edit {
                    // O tipo do próximo Join: a mesma classe do BakeChannels.
                    self.physics.join_kind = tag;
                } else if matches!(edit, ph2d_editor_core::PhysicsFieldEdit::EditShapeOnCanvas) {
                    if let Some(g) = self.gfx.as_mut() {
                        ph2d_shape_handles::alterna(g.sim.world_mut(), entity_bits);
                    }
                } else if matches!(edit, ph2d_editor_core::PhysicsFieldEdit::ShapeDrag { .. })
                    || pd.inspector_selection.is_empty()
                {
                    // O arrasto das alças é do objecto agarrado, nunca da seleção inteira.
                    pd.physics_edits.push((entity_bits, edit));
                } else {
                    for &t in &pd.inspector_selection {
                        pd.physics_edits.push((t, edit));
                    }
                }
            }
            // §12 Physics Joint: a secção descreve UM joint; não se espalha.
            EditorAction::InspectorJointEdit { entity_bits, edit } => {
                // O conta-gotas arma um clique no canvas: estado da shell, resolvido no `input_dispatch`.
                match edit {
                    ph2d_editor_core::JointFieldEdit::PickBodyA => {
                        self.joint_body_pick = Some((entity_bits, false));
                    }
                    ph2d_editor_core::JointFieldEdit::PickBodyB => {
                        self.joint_body_pick = Some((entity_bits, true));
                    }
                    // ⚠️ O único fan-out da §12 (W-JointCopy): o paste carimba o rig inteiro; quem não é
                    // joint cai no early-return de `paste_joint_properties`.
                    ph2d_editor_core::JointFieldEdit::PasteProperties
                        if !pd.inspector_selection.is_empty() =>
                    {
                        for &t in &pd.inspector_selection {
                            pd.joint_edits.push((t, edit));
                        }
                    }
                    _ => pd.joint_edits.push((entity_bits, edit)),
                }
            }
            // §14 Platform Player: a secção descreve UM personagem; não se espalha.
            EditorAction::InspectorPlayerEdit { entity_bits, edit } => {
                // ⚠️ O `ClearRun` não é escrita de componente (W17): a fita mora na shell, e a troca
                // mora numa porta (`run_stash`, W25) partilhada com o painel de Mundo.
                if matches!(edit, ph2d_editor_core::PlayerFieldEdit::ClearRun) {
                    // ⚠️ Descartar GUARDA (W24): a fita não é `ProjectState`; a corrida fica na sessão.
                    ph2d_app_physics::run_stash::apply(
                        ph2d_app_physics::run_stash::RunVerb::Discard,
                        &mut self.player_tape,
                        &mut self.discarded_run,
                    );
                } else if matches!(edit, ph2d_editor_core::PlayerFieldEdit::RestoreRun) {
                    ph2d_app_physics::run_stash::apply(
                        ph2d_app_physics::run_stash::RunVerb::Restore,
                        &mut self.player_tape,
                        &mut self.discarded_run,
                    );
                } else {
                    pd.player_edits.push((entity_bits, edit));
                }
            }
            EditorAction::InspectorWheelEdit { entity_bits, edit } => {
                // W3: o eyedropper arma aqui: o pick é estado da shell.
                if matches!(edit, ph2d_editor_core::WheelFieldEdit::PickMountBody) {
                    self.wheel_body_pick = Some(entity_bits);
                } else if matches!(edit, ph2d_editor_core::WheelFieldEdit::PickRope) {
                    // W1: o mesmo; o alvo é a ROTA, resolvido no Down.
                    self.wheel_rope_pick = Some(entity_bits);
                } else {
                    pd.wheel_edits.push((entity_bits, edit));
                }
            }
            other => return Some(other),
        }
        None
    }
}
