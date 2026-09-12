//! **O invólucro de shell** do `strip` (W2/L5 Fase B 2.ª volta, 2026-09-12).
//!
//! A lei vive em [`crate::flip::strip`] (e viaja para `ph2d_app_flip::strip`). O que sobra
//! aqui é o bloco `impl crate::App`: destrancar o `Option<AppGfx>` e entregar à família os
//! tipos que ela pediu — todos de outra crate (`FlipDoc`, `Playhead`), nunca a `App`.
//! ⚠️ O guarda `gfx.is_none()` fica deste lado: quem tem o `Option` é a shell.

impl crate::App {
    /// O FPS do objeto Flip ativo. `None` sem objeto.
    pub(crate) fn flip_fps(&self) -> Option<f64> {
        crate::flip::strip::fps(&self.gfx.as_ref()?.flip)
    }

    /// **O flip por DESENHO** — leva o playhead à chave anterior/seguinte, pulando os holds.
    pub(crate) fn flip_step_drawing(&mut self, next: bool) {
        let Some(gfx) = self.gfx.as_ref() else { return };
        crate::flip::strip::step_drawing(
            &mut self.flip_state,
            &gfx.flip,
            &mut self.playhead,
            next,
        );
    }
}
