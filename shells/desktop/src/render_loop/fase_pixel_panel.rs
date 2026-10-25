//! Fase do quadro: **o painel PIXEL** (ADR-0192) — composição só: a família aplica os pedidos do
//! painel, abre/fecha-o no flanco do Pixel Sprite ▸ Edit e devolve o instantâneo que se publica. A
//! lei vive na `ph2d_app_pixel::panel`.

impl crate::App {
    /// Depois do modo do objecto (o Edit deste quadro já está decidido) e antes de os painéis pintarem.
    pub(super) fn fase_pixel_panel(&mut self) {
        let Some(gfx) = self.gfx.as_mut() else {
            return;
        };
        let Some(hero) = gfx.hero_screen.as_mut() else {
            return;
        };
        let info = ph2d_app_pixel::panel::frame(&mut self.pixel, gfx.sim.world(), hero);
        #[cfg(feature = "panel-pixel-tools")]
        ph2d_panel_pixel_tools::set_current_pixel(info);
        #[cfg(not(feature = "panel-pixel-tools"))]
        let _ = info;
    }
}
