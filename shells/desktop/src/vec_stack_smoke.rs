//! A PONTE da cena `PH2D_VEC_STACK_SMOKE` — o prólogo, e só ele.
//!
//! ⭐ **O corpo e o roteador vivem em [`ph2d_app_vec::smoke_stack`]** (W2 Fase B, 2.ª volta).
//! ⚠️ A activação da ferramenta FICA aqui: ela mexe no `gfx.tools`, que é da shell.

impl crate::App {
    /// No prólogo do frame, uma vez. No-op sem a env.
    pub(crate) fn vec_stack_smoke(&mut self) {
        if self.vec.stack_smoke_done || !ph2d_app_vec::smoke_stack::armed() {
            return;
        }
        if self.gfx.is_none() {
            return; // sem mundo ainda; tenta no próximo frame
        }
        self.vec.stack_smoke_done = true;
        let gfx = self.gfx.as_mut().expect("gfx");
        let _ = gfx
            .tools
            .set_active(&ph2d_editor_core::ToolId::new("vector"));
        ph2d_app_vec::smoke_stack::build(&mut gfx.vec_scene);
    }
}
