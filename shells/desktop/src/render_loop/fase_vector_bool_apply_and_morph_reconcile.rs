//! **Fase do quadro: O FLATTEN E A RECONCILIAÇÃO DO MORPH** — achatar a seleção (as booleanas vivas e o resto, F50), e
//! reconciliar os conjuntos de Morph States (o par que a cena desenha, a máquina viva e a tabela de States) (OBRA 2 da `line/render-loop`, 2026-09-12).
//!
//! **Corte um nível abaixo:** o bloco do painel vectorial é UM statement de 467 linhas; esta fase é um grupo dos statements do CORPO dele, e a chamada fica dentro do bloco, no sítio do corpo.

use super::*;

/// O pedido de Flatten que o dreno do barramento recolheu neste quadro.
pub(super) struct BoolApplyIntents {
    pub(super) pending_vec_flatten: bool,
}

impl crate::App {
    /// Ver o cabeçalho do módulo.
    pub(super) fn fase_vector_bool_apply_and_morph_reconcile(&mut self, intents: BoolApplyIntents) {
        // O `gfx` re-derivado; os guardas do quadro já correram na `fase_chrome_clock`.
        let Some(gfx) = self.gfx.as_mut() else {
            return;
        };
        let FrameGfx {
            ui_states,
            sim,
            vec_scene,
            ..
        } = FrameGfx::of(gfx);
        let BoolApplyIntents {
            pending_vec_flatten,
        } = intents;
        if pending_vec_flatten {
            let xf = ph2d_vec_entities::transform::build(sim, &self.vec.entities);
            let n = ph2d_app_vec::bool_flatten::flatten(
                sim,
                vec_scene,
                &self.vec.entities,
                &mut self.vec.pen,
                &self.bool_live,
                &xf,
            );
            eprintln!("[ph2d-vec] flatten: {n} path[s]");
            // ⭐ **COMMIT** (D1): consolidar é o gesto que muda o documento de vez, e é
            // exactamente o que uma confirmação pelo ouvido serve.
            self.pending_ui_sound = Some(crate::ui_sound::UiSound::Commit);
        }
        // ⭐⭐⭐ **A RECONCILIAÇÃO dos conjuntos de Morph States** (W11g + W11i) — o par que
        // a cena desenha, a máquina viva, e a tabela de States, todos contra a lista de
        // membros do quadro.
        //
        // ⚠️ **TARDE no quadro, depois do despacho dos painéis, e é uma decisão:** as três
        // rotas que tiram uma forma do conjunto — o ⊘, **apagar** e **arrastar para fora**
        // na Hierarquia — acontecem aqui em cima. A arrumação entra assim na **mesma**
        // fotografia do gesto que a causou, e o artista desfaz tudo num Ctrl+Z; a correr
        // antes, ela chegaria um quadro atrasada e custaria um **segundo** passo.
        crate::morph_machine_drive::reconcile(
            &mut self.morph_machines,
            sim,
            vec_scene,
            &self.vec.entities,
            ui_states,
        );
    }
}
