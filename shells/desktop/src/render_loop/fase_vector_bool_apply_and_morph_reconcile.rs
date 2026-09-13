//! **Fase do quadro: O APPLY BOOLEANO E A RECONCILIAÇÃO DO MORPH** — assar a booleana do grupo seleccionado, e
//! reconciliar os conjuntos de Morph States (o par que a cena desenha, a máquina viva e a tabela de States) (OBRA 2 da `line/render-loop`, 2026-09-12).
//!
//! **Corte um nível abaixo:** o bloco do painel vectorial é UM statement de 467 linhas; esta fase é um grupo dos statements do CORPO dele, e a chamada fica dentro do bloco, no sítio do corpo.

use super::*;

/// O pedido de Apply booleano que o dreno do barramento recolheu neste quadro.
pub(super) struct BoolApplyIntents {
    pub(super) pending_bool_apply: bool,
}

impl crate::App {
    /// Ver o cabeçalho do módulo.
    pub(super) fn fase_vector_bool_apply_and_morph_reconcile(
        &mut self,
        intents: BoolApplyIntents,
        group: Option<ph2d_ecs::Entity>,
    ) {
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
        let BoolApplyIntents { pending_bool_apply } = intents;
        if pending_bool_apply
            && let Some(g) = group
            && let Some(plan) = self.bool_live.plan(g)
        {
            let n = crate::bool_gesture::bake(sim, vec_scene, &mut self.vec.pen, plan, g);
            eprintln!("[ph2d-vec] boolean live: consolidada ({n} path[s])");
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
