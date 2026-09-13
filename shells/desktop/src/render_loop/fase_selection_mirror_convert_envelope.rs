//! **Fase do quadro: O CONVERTER E O ENVELOPE NO PAINEL** — se a selecção é convertível em curvas e se é um envelope (a MESMA porta que decide a
//! selecção e executa o dissolve) (OBRA 2 da `line/render-loop`, 2026-09-13).

use super::*;

impl crate::App {
    /// Ver o cabeçalho do módulo.
    pub(super) fn fase_selection_mirror_convert_envelope(&mut self) -> Option<Option<u64>> {
        // O `gfx` re-derivado; os guardas do quadro já correram na `fase_chrome_clock`.
        let gfx = self.gfx.as_mut()?;
        let FrameGfx { sim, vec_scene, .. } = FrameGfx::of(gfx);
        let convertible =
            self.vec.pen.selected_paths().iter().any(|id| {
                crate::vec_convert::is_convertible(sim, &self.vec.entities, vec_scene, *id)
            });
        ph2d_panel_vector::set_current_convertible(convertible);
        // ADR-0129: Expand/Release só são OFERECIDOS quando a seleção é de fato um
        // envelope. A pergunta é a MESMA porta que decide a seleção (selecionar-só-o-
        // container) e executa o dissolve — três consumidores, uma resposta.
        let sel_bits: Vec<u64> = self
            .vec
            .pen
            .selected_paths()
            .iter()
            .filter_map(|id| self.vec.entities.get(id).copied())
            .collect();
        let env_container = crate::envelope_live::sole_container(sim, &sel_bits);
        ph2d_panel_vector::set_current_has_envelope(env_container.is_some());
        Some(env_container)
    }
}
