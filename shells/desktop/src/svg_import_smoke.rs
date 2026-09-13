//! A PONTE da cena `PH2D_VEC_SVG_SMOKE` — o prólogo, e só ele.
//!
//! ⭐ **O corpo e o roteador vivem em [`ph2d_app_vec::smoke_svg`]** (W2 Fase B, 2.ª volta).

impl crate::App {
    /// No prólogo do frame, uma vez. No-op sem a env.
    pub(crate) fn svg_import_smoke(&mut self) {
        if self.svg_import_smoke_done || !ph2d_app_vec::smoke_svg::armed() {
            return;
        }
        if self.gfx.is_none() {
            return; // sem mundo ainda; tenta no próximo frame
        }
        self.svg_import_smoke_done = true;

        let gfx = self.gfx.as_mut().expect("gfx");
        let ppm = gfx
            .hero_screen
            .as_ref()
            .map_or(ph2d_editor_core::DEFAULT_PIXELS_PER_METER, |h| {
                h.project.pixels_per_meter
            });
        ph2d_app_vec::smoke_svg::build(
            &mut gfx.sim,
            &mut gfx.vec_scene,
            &mut self.vec.entities,
            ppm,
        );
    }
}
