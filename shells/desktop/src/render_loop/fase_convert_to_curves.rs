//! **Fase do quadro: O CONVERTER EM CURVAS** — o *Convert to Curves* da selecção. ⚠️ O bloco a seguir, que publica nos painéis o que a
//! selecção é, fica no ORQUESTRADOR: ele já só chama as suas fases, e a cola delas é do quadro (OBRA 2 da `line/render-loop`, 2026-09-13).

use super::*;

/// Os pedidos que o dreno do barramento recolheu neste quadro para esta fase.
pub(super) struct ConvertToCurvesIntents {
    pub(super) pending_vec_convert: bool,
}

impl crate::App {
    /// Ver o cabeçalho do módulo.
    pub(super) fn fase_convert_to_curves(&mut self, intents: ConvertToCurvesIntents) {
        // O `gfx` re-derivado; os guardas do quadro já correram na `fase_chrome_clock`.
        let Some(gfx) = self.gfx.as_mut() else {
            return;
        };
        let FrameGfx { sim, vec_scene, .. } = FrameGfx::of(gfx);
        let ConvertToCurvesIntents {
            pending_vec_convert,
        } = intents;
        // "Convert to Curves": assa a(s) forma(s) viva(s) selecionada(s) em paths
        // crus — o TEXTO explode num grupo por-letra; as PARAMÉTRICAS descartam o
        // `VecShape` (a geometria já é a forma); e a pilha de EFEITOS é assada no cozido
        // (ADR-0132). A porta única (`vec_convert::to_curves`) usa o MESMO bake do botão
        // "Apply" da seção Effects. Re-seleciona o resultado.
        if pending_vec_convert {
            let sel: Vec<ph2d_vec_scene::VecPathId> = self.vec.pen.selected_paths().to_vec();
            let xf = ph2d_vec_entities::transform::build(sim, &self.vec.entities);
            let new_sel = crate::vec_convert::to_curves(
                sim,
                vec_scene,
                &mut self.vec.entities,
                &mut self.vec.pen,
                &xf,
                &sel,
            );
            self.vec.pen.select_many(&new_sel);
        }
    }
}
