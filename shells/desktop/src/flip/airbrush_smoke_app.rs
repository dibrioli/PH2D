//! **O invólucro de shell** do `airbrush_smoke` (W2/L5 Fase B, 2026-09-11).
//!
//! A cena inteira vive em [`ph2d_app_flip::airbrush_smoke::arm`]. O que sobra aqui é o único
//! passo que precisa da `App`: **destrancar o `AppGfx`** e entregar à família os três tipos
//! que ela pediu. ⚠️ O guarda `gfx.is_none()` tinha de ficar deste lado — quem tem o
//! `Option` é a shell, e passá-lo para dentro seria atravessar o handle.

impl crate::App {
    pub(crate) fn flip_airbrush_smoke(&mut self) {
        if let Some(gfx) = self.gfx.as_mut() {
            ph2d_app_flip::airbrush_smoke::arm(&mut gfx.flip, &mut gfx.tools, &mut self.playhead);
        }
    }
}
