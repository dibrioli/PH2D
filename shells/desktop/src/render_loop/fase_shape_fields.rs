//! **Fase do quadro: OS CAMPOS DE FORMA** — o latch da forma armada, o foco do painel e a semente;
//! a lei mora em `ph2d_app_vec::shape_params::frame_fields` (F9 tirou-a daqui).

use super::*;

impl crate::App {
    /// Ver o cabeçalho do módulo.
    pub(super) fn fase_shape_fields(&mut self) {
        // O `gfx` re-derivado; os guardas do quadro já correram na `fase_chrome_clock`.
        let Some(gfx) = self.gfx.as_mut() else {
            return;
        };
        let FrameGfx {
            sim,
            tools,
            hero_screen,
            ..
        } = FrameGfx::of(gfx);
        let Some(hero) = hero_screen.as_mut() else {
            return;
        };
        #[cfg(feature = "panel-vector")]
        ph2d_app_vec::shape_params::frame_fields(
            &mut self.vec,
            sim,
            tools,
            &mut hero.store,
            ph2d_editor_core::DocPx::of(&hero.project),
        );
    }
}
