//! **Fase do quadro: A POEIRA DE IMPACTO** (estudo de UI viva, D2) — as faíscas que confirmam o gesto,
//! pintadas por CIMA de todo o chrome, nos dois ramos do ecrã (OBRA 2 da `line/render-loop`, 2026-09-12).
//!
//! ⚠️ Corre DEPOIS do ramo hero e do ramo legado e ANTES do `run_present_phase`, que é quem entrega a
//! cena de vetor à GPU.

use super::*;

impl crate::App {
    /// Ver o cabeçalho do módulo.
    pub(super) fn fase_ui_burst_paint(&mut self) {
        // O `gfx` re-derivado; os guardas do quadro já correram na `fase_chrome_clock`.
        let Some(gfx) = self.gfx.as_mut() else {
            return;
        };
        let FrameGfx { vector_scene, .. } = FrameGfx::of(gfx);
        // ⭐⭐⭐ **A POEIRA DE IMPACTO** (estudo de UI viva, D2) — por CIMA de tudo, porque ela é a
        // confirmação do gesto que acabou de acontecer e nada do chrome a deve tapar.
        //
        // ⚠️ **Fora do `if` acima de propósito**: aquele ramo é o do modo com chrome completo, e uma
        // faísca é confirmação de um gesto que existe nos dois. ⛔ Uma cópia dentro de cada ramo
        // seria a segunda lei a manter em sincronia com a primeira.
        crate::ui_burst_paint::paint(&self.ui_burst, vector_scene);
    }
}
