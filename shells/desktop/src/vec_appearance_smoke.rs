//! A PONTE da cena `PH2D_VEC_APPEARANCE_SMOKE` — o prólogo, e só ele.
//!
//! ⭐ **O corpo e o roteador vivem em [`ph2d_app_vec::smoke_appearance`]** (W2 Fase B, 2.ª volta).
//! O que fica aqui é o que toca a `App`: o `gfx` já existir, e a memória *«esta cena já montou?»*.
//! *O que sai são os CORPOS; o que decide a ordem do quadro fica* (HOWTO §4).

impl crate::App {
    /// No prólogo do frame, uma vez. No-op sem a env.
    pub(crate) fn vec_appearance_smoke(&mut self) {
        if self.vec_state.appearance_smoke_done || !ph2d_app_vec::smoke_appearance::armed() {
            return;
        }
        if self.gfx.is_none() {
            return; // sem mundo ainda; tenta no próximo frame
        }
        self.vec_state.appearance_smoke_done = true;
        ph2d_app_vec::smoke_appearance::build(&mut self.gfx.as_mut().expect("gfx").vec_scene);
    }
}
