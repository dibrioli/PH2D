//! **O invólucro de shell** do ajuste ao vivo do Colorize (W2/L5 Fase B 2.ª volta,
//! 2026-09-12).
//!
//! A lei vive em [`ph2d_app_flip::colorize::live_adjust`]. Aqui fica o `impl crate::App`: a escala do
//! objecto, o `Option<AppGfx>`, e a tradução do valor de retorno em `any_input_this_frame` +
//! `title_dirty` — *a fronteira atravessa-se com um valor, não com um campo alheio*.

use ph2d_app_flip::ctx::FlipFrame;

impl crate::App {
    /// Trap/Bleed em tempo real depois do Apply, fora da thread de UI.
    pub(crate) fn flip_colorize_live_adjust(&mut self) {
        // ⚠️ O guarda `live.is_none()` mora na LEI (o campo é privado ao módulo dela, e
        // duplicá-lo aqui seria a 2.ª resposta à mesma pergunta). Isto custa a derivação do
        // `obj_scale` num frame sem sessão viva — 10 multiplicações, medido irrelevante.
        let wants_colorize = self.flip_wants_colorize();
        let obj_scale = self.flip_active_world_to_local().mean_scale() as f32;
        let playhead = self.playhead;
        let Some(gfx) = self.gfx.as_mut() else {
            return;
        };
        let win = gfx.surface.size();
        let mut f = FlipFrame {
            flip: &mut gfx.flip,
            playhead: &playhead,
            camera: &gfx.camera,
            win,
        };
        let installed = ph2d_app_flip::colorize::live_adjust(
            &mut self.flip_state,
            &mut f,
            wants_colorize,
            obj_scale,
        );
        if installed {
            // Sem isto o `post_frame_undo` pularia o diff num frame sem outro input, e o
            // ajuste ficaria fora do passo.
            self.any_input_this_frame = true;
            self.title_dirty = true;
        }
    }
}
