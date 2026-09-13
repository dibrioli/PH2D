//! **Fase do quadro: O GRUPO BOOLEANO E O VERBO DA FORMA** — um verbo por forma (a receita lê-se na
//! hierarquia): honrar o clique ANTES de publicar a fileira (OBRA 2 da `line/render-loop`, 2026-09-12).
//!
//! **Corte um nível abaixo:** o bloco do painel vectorial é UM statement de 467 linhas; esta fase é um grupo dos statements do CORPO dele, e a chamada fica dentro do bloco, no sítio do corpo.

use super::*;

/// O pedido de verbo booleano da forma que o dreno do barramento recolheu neste quadro.
pub(super) struct BoolShapeIntents {
    pub(super) pending_bool_shape_op: Option<u8>,
}

impl crate::App {
    /// Ver o cabeçalho do módulo.
    pub(super) fn fase_vector_bool_shape_row(
        &mut self,
        intents: BoolShapeIntents,
        sel: Vec<ph2d_vec_scene::VecPathId>,
    ) -> Option<(Option<ph2d_ecs::Entity>, Vec<ph2d_vec_scene::VecPathId>)> {
        // O `gfx` re-derivado; os guardas do quadro já correram na `fase_chrome_clock`.
        let gfx = self.gfx.as_mut()?;
        let FrameGfx { sim, .. } = FrameGfx::of(gfx);
        let BoolShapeIntents {
            pending_bool_shape_op,
        } = intents;
        let group = crate::bool_gesture::group_of_selection(sim, &self.vec.entities, &sel);
        ph2d_panel_vector::state::set_bool_group_selected(group.is_some());
        // **O VERBO DA FORMA: honrar o clique ANTES de publicar** — a ordem é a mesma do
        // chip do recorte, e pela mesma razão: publicar primeiro deixaria o chip a piscar
        // de volta ao valor antigo por um quadro.
        //
        // ⚠️ O escritor **reconfere** a triagem em vez de confiar no que o painel pintou:
        // entre pintar a fileira e o clique chegar passa um frame, e nele a seleção pode
        // ter mudado.
        // ⚠️ O sujeito é o **PRIMÁRIO**, e não «a seleção». Tocar um filho seleciona o
        // GRUPO inteiro (`input_dispatch`), então uma regra de contagem tornava esta
        // fileira inalcançável por clique — foi o defeito de 22/08. O primário sobrevive
        // à expansão (`set_object_selection` preserva-o) e é a forma que o dedo apontou.
        let primary = self.vec.pen.selected();
        if let Some(code) = pending_bool_shape_op {
            crate::vec_bool_shape::set_selected_shape_op(
                sim,
                &self.vec.entities,
                &self.bool_live,
                &sel,
                primary,
                code,
            );
        }
        ph2d_panel_vector::state::set_bool_shape_row(
            crate::vec_bool_shape::shape_row_of_selection(
                sim,
                &self.vec.entities,
                &self.bool_live,
                &sel,
                primary,
            ),
        );
        Some((group, sel))
    }
}
