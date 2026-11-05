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
            // ⭐⭐⭐ **Renomear o VALOR de uma propriedade** (report do Enio, 2026-08-31).
            // ⚠️ O sujeito é a RECEITA; o gesto nasce sobre a cópia, que é onde o artista
            // está a olhar. Ver `ph2d-panel-inspector/src/event_value.rs`.
            // ⭐⭐⭐ **GRAVAR A VARIAÇÃO** (Enio, 2026-09-01) — o botão do cartão.
            // §11 Physics Body. Fans out over a BulkSelect like its
            // siblings — "make all of these physical" is the gesture
            // an artist actually performs.
            EditorAction::InspectorPhysicsEdit { entity_bits, edit } => {
                // ⚠️ **Join does NOT fan out.** Every other §11 edit is
                // per-entity ("make all of these static"), but joining
                // is one gesture over a PAIR — fanned out it would
                // create one joint per selected body, i.e. two joints
                // between the same two objects, on the very click that
                // is supposed to make one.
                if matches!(edit, ph2d_editor_core::PhysicsFieldEdit::Join) {
                    // ⚠️ **2 ou MAIS** (W-J4): três corpos marcados
                    // fazem uma CORRENTE de N−1 joints, na ordem da
                    // seleção. Não é fan-out (isso criaria um joint por
                    // corpo, entre os mesmos dois) — é UMA operação
                    // sobre a sequência, que a `join_selected_chain`
                    // executa depois do laço.
                    if pd.inspector_selection.len() >= 2 {
                        pd.join_chain = true;
                    }
                } else if matches!(edit, ph2d_editor_core::PhysicsFieldEdit::Rig) {
                    // ⚠️ **Nem o Rig faz fan-out** (W-Rig), e a razão é a
                    // do Bake mais que a do Join: cada corrida do gerador
                    // percorre a MESMA subárvore, então espalhado ele
                    // rodaria N vezes sobre o mesmo trabalho — a 2ª em
                    // diante achariam tudo já ligado e não fariam nada,
                    // mas o toast contaria a 1ª N vezes.
                    pd.rig_now = true;
                } else if matches!(edit, ph2d_editor_core::PhysicsFieldEdit::JoinDraw) {
                    // ARMA o gesto de canvas (sem operando, como os
                    // eyedroppers do §12): quem nomeia os dois corpos é
                    // o press e o release, não a seleção. Armado aqui e
                    // honrado no `input_dispatch`.
                    pd.join_draw_arm = true;
                } else if matches!(edit, ph2d_editor_core::PhysicsFieldEdit::Bake) {
                    // WARNING: **Bake does not fan out either**, and the
                    // cost of getting it wrong is bigger than Join's:
                    // ONE bake runs the whole simulation once and writes
                    // every selected body's curves from that single run.
                    // Fanned out it would re-simulate the entire scene
                    // once per selected body - same numbers, N times the
                    // work - and file a separate undo step for each, so
                    // undoing "the bake" would take as many Ctrl+Z
                    // presses as there were objects.
                    pd.bake_request = Some(if pd.inspector_selection.is_empty() {
                        vec![entity_bits]
                    } else {
                        pd.inspector_selection.clone()
                    });
                } else if let ph2d_editor_core::PhysicsFieldEdit::BakeChannels(tag) = edit {
                    // A GLOBAL bake option, not a per-body edit (like
                    // Bake itself): it says how the NEXT bake behaves.
                    // No fan-out, no Collider write — just the app state
                    // the Bake button reads.
                    self.bake_channels = ph2d_app_physics::bake::BakeChannels::from_tag(tag);
                } else if let ph2d_editor_core::PhysicsFieldEdit::JoinKind(tag) = edit {
                    // The pending join KIND, the same class as BakeChannels:
                    // an app-state option the Join gesture reads, not a
                    // per-body edit. No fan-out, no Collider write.
                    self.physics.join_kind = tag;
                } else if pd.inspector_selection.is_empty() {
                    pd.physics_edits.push((entity_bits, edit));
                } else {
                    for &t in &pd.inspector_selection {
                        pd.physics_edits.push((t, edit));
                    }
                }
            }
            // §12 Physics Joint. No fan-out either, and for a simpler
            // reason: the section only ever describes one joint object.
            EditorAction::InspectorJointEdit { entity_bits, edit } => {
                // The eyedropper ARMS a canvas pick (shell state), it is
                // not a component edit — handled here where `self` is
                // freely mutable, exactly like `Join` sets `join_request`.
                // The next canvas click resolves it (`input_dispatch`).
                match edit {
                    ph2d_editor_core::JointFieldEdit::PickBodyA => {
                        self.joint_body_pick = Some((entity_bits, false));
                    }
                    ph2d_editor_core::JointFieldEdit::PickBodyB => {
                        self.joint_body_pick = Some((entity_bits, true));
                    }
                    // ⚠️ **O ÚNICO fan-out da §12** (W-JointCopy). O
                    // resto da seção descreve UM joint e edita UM; um
                    // paste existe para carimbar o rig inteiro, e sem
                    // isto o gesto é *digitar quinze campos, dez vezes*.
                    // Espalhado sobre a seleção crua: quem não for joint
                    // cai no early-return de `paste_joint_properties`,
                    // do mesmo jeito que o fan-out do §11 atravessa
                    // entidades sem `Collider`.
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
            // §14 Platform Player. Sem fan-out, e pela razão da §12: a
            // seção descreve UM personagem, o que está selecionado.
            EditorAction::InspectorPlayerEdit { entity_bits, edit } => {
                // ⚠️ **O `ClearRun` é o único verbo da §14 que não é uma
                // escrita de componente** (W17): a fita de entrada mora na
                // shell, então ele é honrado AQUI, onde o `self` é
                // mutável — o lugar e a razão exatos do `Join` da §11 e do
                // eyedropper da §12.
                //
                // ⚠️ E interceptar não é higiene: descartar é idempotente,
                // então espalhá-lo pela seleção não corromperia nada HOJE.
                // É precisamente essa forma que apodrece — o Ctrl+V do
                // editor de nós colava duas vezes porque um dispatch
                // duplicado "nunca tinha importado enquanto todos os
                // verbos eram idempotentes".
                //
                // ⚠️ **E a troca em si mora numa PORTA** (`run_stash`,
                // W25), porque o painel de MUNDO é uma segunda VISTA da
                // mesma corrida: duas cópias do `mem::take` fariam a
                // mesma coisa hoje e divergiriam no dia em que o
                // descarte ganhar um caso especial.
                if matches!(edit, ph2d_editor_core::PlayerFieldEdit::ClearRun) {
                    // ⚠️ **Descartar GUARDA** (W24): a corrida sai do
                    // documento e fica na sessão, porque o clique era
                    // irreversível — a fita não é `ProjectState`, então
                    // sem isto o único caminho de volta era reabrir o
                    // arquivo.
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
                // W3: o eyedropper ARMA aqui (onde `self` é mutável), como
                // o do joint e pela mesma razão — o pick é estado da
                // shell, não uma escrita de componente.
                if matches!(edit, ph2d_editor_core::WheelFieldEdit::PickMountBody) {
                    self.wheel_body_pick = Some(entity_bits);
                } else if matches!(edit, ph2d_editor_core::WheelFieldEdit::PickRope) {
                    // W1: o mesmo lugar e a mesma razão — o pick é estado
                    // da shell. O alvo é a ROTA, resolvido no Down.
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
