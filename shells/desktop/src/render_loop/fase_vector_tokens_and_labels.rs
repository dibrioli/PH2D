//! **Fase do quadro: OS TOKENS E AS ETIQUETAS DAS MOLDURAS** — aplicar a escolha de token e depois publicar o
//! que a selecção tem preso, e as etiquetas das molduras publicadas em todo quadro (OBRA 2 da `line/render-loop`, 2026-09-12).
//!
//! **Corte um nível abaixo:** o bloco do painel vectorial é UM statement de 467 linhas; esta fase é um grupo dos statements do CORPO dele, e a chamada fica dentro do bloco, no sítio do corpo.

use super::*;

/// O pedido de vínculo de token que o dreno do barramento recolheu neste quadro.
pub(super) struct TokenBindIntents {
    pub(super) pending_token_bind: Option<(ph2d_ecs::BoundProp, Option<&'static str>)>,
}

impl crate::App {
    /// Ver o cabeçalho do módulo.
    pub(super) fn fase_vector_tokens_and_labels(
        &mut self,
        intents: TokenBindIntents,
        vec_xf: ph2d_vec_scene::VecXforms,
        sel: Vec<ph2d_vec_scene::VecPathId>,
    ) -> Option<(ph2d_vec_scene::VecXforms, Vec<ph2d_vec_scene::VecPathId>)> {
        // O `gfx` re-derivado; os guardas do quadro já correram na `fase_chrome_clock`.
        let gfx = self.gfx.as_mut()?;
        let FrameGfx {
            sim,
            vec_scene,
            hero_screen,
            ..
        } = FrameGfx::of(gfx);
        // O bloco do quadro só chama esta fase com o `HeroScreen` vivo.
        let hero = hero_screen.as_mut()?;
        let TokenBindIntents { pending_token_bind } = intents;
        // **OS TOKENS** (plano UI/UX W4): aplicar a escolha, e depois publicar o que a
        // seleção tem preso. Nesta ordem — publicar antes deixaria o chip a mostrar o
        // token ANTERIOR por um frame, e o artista veria a escolha "não pegar".
        // ⚠️ O detach vem ANTES da escolha do picker: no mesmo frame em que o artista
        // escolhe um TOKEN o `colour_authored` está limpo, então nenhum dos dois pisa no
        // outro — mas na ordem inversa uma cor autorada soltaria o token recém-escolhido.
        crate::vec_bindings::detach_on_authored(sim, &self.vec.entities, &sel);
        if let Some((prop, token)) = pending_token_bind {
            crate::vec_bindings::set_selected_binding(sim, &self.vec.entities, &sel, prop, token);
        }
        ph2d_panel_vector::state::set_token_bindings(crate::vec_bindings::selected_bindings(
            sim,
            &self.vec.entities,
            &sel,
        ));
        // **As ETIQUETAS das molduras** (Enio 2026-08-01) — publicadas em TODO frame, com
        // qualquer ferramenta em mãos. ⚠️ Aqui não vale a cerca da RÉGUA (que só vive com
        // o Vector porque OCUPA a borda do canvas e comeria o pen-down do Painter): uma
        // etiqueta é desenho puro, sem região de hit, e uma moldura é mobília de cena que
        // se precisa reconhecer mesmo enquanto se pinta dentro dela.
        hero.gizmo.frame_labels = crate::vec_frame_labels::frame_labels(
            sim,
            vec_scene,
            &self.vec.entities,
            &vec_xf,
            &sel,
        );
        Some((vec_xf, sel))
    }
}
