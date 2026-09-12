//! **O invólucro de shell** do `resample_smoke` (W2/L5 Fase B, 2026-09-11).
//!
//! A cena inteira vive em [`ph2d_app_flip::resample_smoke::arm`], e o ROTEADOR dela também — é
//! por isso que o `const FAMILY` da crate o declara. Aqui fica só destrancar o `AppGfx`.

impl crate::App {
    pub(crate) fn flip_resample_smoke(&mut self) {
        if let Some(gfx) = self.gfx.as_mut() {
            ph2d_app_flip::resample_smoke::arm(&mut gfx.flip, &mut gfx.tools, &mut self.playhead);
        }
    }
}
