//! A PONTE da cena `PH2D_VEC_FADE_SMOKE` — o prólogo, e só ele.
//!
//! ⭐ **O corpo e o roteador vivem em [`ph2d_app_vec::smoke_fade`]** (W2 Fase B, 2.ª volta).
//! ⚠️ Abrir a régua FICA aqui: ela mexe no `hero_screen`, que é chrome da shell.

impl crate::App {
    /// No prólogo do frame, uma vez. No-op sem a env.
    pub(crate) fn vec_fade_smoke(&mut self) {
        if self.vec.fade_smoke_done || !ph2d_app_vec::smoke_fade::armed() {
            return;
        }
        if self.gfx.is_none() {
            return; // sem mundo ainda; tenta no próximo frame
        }
        self.vec.fade_smoke_done = true;

        let gfx = self.gfx.as_mut().expect("gfx");
        ph2d_app_vec::smoke_fade::build(
            &mut gfx.vec_scene,
            &mut gfx.sim,
            &mut self.vec.entities,
            &mut self.timeline.doc,
        );

        // A régua já aberta — o smoke é sobre o que a curva faz, não sobre achar o painel.
        // (`L` alterna, e é o mesmo gesto que a cena da física usa.)
        if let Some(hero) = self.gfx.as_mut().expect("gfx").hero_screen.as_mut() {
            hero.panel_visibility.insert("timeline", true);
        }
    }
}
