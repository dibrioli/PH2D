//! Fase do quadro: **o painel PIXEL e a coluna da esquerda** (ADR-0192) — composição só; a lei vive
//! na `ph2d_app_pixel::panel` (pedidos, abrir/fechar no Draw, o instantâneo, as barras).

impl crate::App {
    /// Depois do modo do objecto (o Draw deste quadro já está decidido) e antes de os painéis pintarem.
    pub(super) fn fase_pixel_panel(&mut self) {
        let Some(g) = self.gfx.as_mut() else { return };
        let Some(hero) = g.hero_screen.as_mut() else {
            return;
        };
        let (pixel, world) = (&mut self.pixel, g.sim.world());
        let info = ph2d_app_pixel::panel::frame(pixel, world, hero, &mut g.toasts);
        #[cfg(feature = "panel-pixel-tools")]
        {
            let (column, hint) = (
                ph2d_panel_pixel_tools::column::column(&info),
                ph2d_panel_pixel_tools::column::hint(&info),
            );
            ph2d_app_pixel::panel::publish_bars(&self.pixel, hero, column, hint);
            ph2d_panel_pixel_tools::set_current_pixel(info);
        }
        #[cfg(not(feature = "panel-pixel-tools"))]
        let _ = info;
    }
}
