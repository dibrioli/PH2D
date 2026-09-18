//! **Fase do quadro: OS VERBOS DE COMPONENTE** — os verbos de componente (fazer, instanciar, soltar…) pedidos pelos painéis (OBRA 2 da `line/render-loop`, 2026-09-13).

use super::*;

/// Os pedidos que o dreno do barramento recolheu neste quadro para esta fase.
pub(super) struct ComponentVerbsIntents {
    pub(super) pending_component: Option<crate::vec_component_edit::ComponentEdit>,
}

impl crate::App {
    /// Ver o cabeçalho do módulo.
    pub(super) fn fase_component_verbs(&mut self, intents: ComponentVerbsIntents) {
        // O `gfx` re-derivado; os guardas do quadro já correram na `fase_chrome_clock`.
        let Some(gfx) = self.gfx.as_mut() else {
            return;
        };
        // ⭐ **A janela da CENA, não a do quadro** — sob um split do centro a cena desenha num
        // sub-rectângulo e a projecção MUDA (ver [`crate::scene_mapping`]). Fora do split é a
        // janela inteira, bit a bit.
        let janela_da_cena = gfx.scene_window();
        let FrameGfx {
            sim,
            camera,
            toasts,
            vec_scene,
            hero_screen,
            component_registry,
            ..
        } = FrameGfx::of(gfx);
        // O bloco do quadro só chama esta fase com o `HeroScreen` vivo.
        let Some(hero) = hero_screen.as_mut() else {
            return;
        };
        let ComponentVerbsIntents { pending_component } = intents;
        if let Some(verb) = pending_component {
            let sel: Vec<ph2d_vec_scene::VecPathId> = self.vec.pen.selected_paths().to_vec();
            // ⭐⭐⭐ **O CLIQUE da secção *Prefab*, e há UM motor** (F4.6c fechada, 2026-09-07).
            //
            // ⚠️ **Aqui viveu um `if armed() { … } else { … }`** — o modelo geral de um lado e o
            // motor `VecInstance` do outro, com uma variável de ambiente a escolher. O `else`
            // morreu com a fatia: *dois motores para o mesmo estado é pior que um motor lento*,
            // e a régua de que nada se perdeu está em `instance_piece_override_tests.rs`, que a
            // wave anterior escreveu **como pré-condição desta**.
            //
            // ⚠️ **O sujeito resolve-se ANTES dos documentos** — o mapa `path ⟺ entidade`
            // entra no `OwnedDocs` emprestado mutavelmente, e pedi-lo outra vez lá dentro
            // seria o segundo empréstimo.
            //
            // ⚠️ **Pela MESMA função que a secção usa para se MOSTRAR** — duas resoluções
            // dariam um botão oferecido sobre o grupo e um clique a agir sobre um filho.
            let subject = crate::vec_component_general::subject_of(
                &self.vec.entities,
                &sel,
                (hero.gizmo.selected_len() == 1)
                    .then_some(hero.gizmo.selection)
                    .flatten(),
            );
            let step = crate::input_dispatch::screen_offset_world(
                camera,
                janela_da_cena,
                crate::input_dispatch::PASTE_OFFSET_PX,
            );
            let mut select_out = None;
            let mut arm_pick = false;
            if let Some(subject) = subject {
                let mut docs = ph2d_app_components::instance_docs::OwnedDocs {
                    vec_scene,
                    vec_entities: &mut self.vec.entities,
                };
                if crate::vec_component_general::dispatch(
                    verb,
                    sim,
                    component_registry,
                    &mut self.instance_echo,
                    subject,
                    toasts,
                    &mut docs,
                    [step.0 as f32, step.1 as f32],
                    &mut select_out,
                    &mut arm_pick,
                ) {
                    self.title_dirty = true;
                }
            }
            if let Some(bits) = select_out {
                hero.gizmo.replace_selection(Some(bits));
            }
            // ⭐⭐ **A shell só ESCREVE o pick — quem decide é o módulo do modo.** O
            // `PathPick` vive no `App`, e por isso o dreno não lhe chega; mas a pergunta
            // *«este verbo abre o gesto de duas mãos?»* é lei do modo, e fica lá.
            if arm_pick && let Some(&at) = sel.first() {
                self.vec.path_pick = Some(crate::vec_pick::PathPick::InstanceMain(at));
            }
        }
    }
}
