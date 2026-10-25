//! **Fase do quadro: OS COMANDOS DA PILHA DE FILTROS** — a paragem do gradiente de um filtro e os comandos da pilha: acrescentar, tirar,
//! reordenar, esconder, o modo e a mistura (OBRA 2 da `line/render-loop`, 2026-09-13).

use super::*;

/// Os pedidos que o dreno do barramento recolheu neste quadro para esta fase.
pub(super) struct FilterCommandsIntents {
    pub(super) pending_filter_cmd: Option<crate::fx_live::FilterHit>,
    pub(super) pending_filter_stop: Option<(usize, u8, f32)>,
}

impl crate::App {
    /// Ver o cabeçalho do módulo.
    pub(super) fn fase_filter_commands(
        &mut self,
        intents: FilterCommandsIntents,
    ) -> Option<Vec<ph2d_vec_scene::VecPathId>> {
        // O `gfx` re-derivado; os guardas do quadro já correram na `fase_chrome_clock`.
        let gfx = self.gfx.as_mut()?;
        let FrameGfx { sim, .. } = FrameGfx::of(gfx);
        let FilterCommandsIntents {
            pending_filter_cmd,
            pending_filter_stop,
        } = intents;
        let sel: Vec<ph2d_vec_scene::VecPathId> = self.vec.pen.selected_paths().to_vec();
        // A lei de cada comando mora em [`ph2d_app_vec::fx_live_comandos`].
        ph2d_app_vec::fx_live_comandos::aplica(
            sim,
            &self.vec.entities,
            &sel,
            pending_filter_stop,
            pending_filter_cmd,
        );
        Some(sel)
    }
}
