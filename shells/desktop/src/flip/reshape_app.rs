//! **O invólucro de shell** do Sculpt (W2/L5 Fase B 2.ª volta, 2026-09-12).
//!
//! A lei vive em [`ph2d_app_flip::reshape`]. Aqui fica o `impl crate::App`: destrancar o
//! `Option<AppGfx>`, derivar o afim mundo→local, ler o modificador e marcar o `title_dirty`.

use ph2d_app_flip::ctx::FlipFrame;

impl crate::App {
    /// A tool Flip quer o canvas para ESCULPIR agora?
    #[must_use]
    pub(crate) fn flip_wants_reshape(&self) -> bool {
        ph2d_app_flip::reshape::wants(&self.flip_state)
    }

    /// Pen-down do Sculpt.
    pub(crate) fn flip_reshape_canvas_down(&mut self, x: f32, y: f32) -> bool {
        let w2l = self.flip_active_world_to_local();
        let invert = self.modifiers.control_key();
        let playhead = self.playhead;
        let Some(gfx) = self.gfx.as_mut() else {
            return false;
        };
        let win = gfx.surface.size();
        let mut f = FlipFrame {
            flip: &mut gfx.flip,
            playhead: &playhead,
            camera: &gfx.camera,
            win,
        };
        let (consumed, warned) = ph2d_app_flip::reshape::canvas_down(
            &mut self.flip_state,
            &mut f,
            &mut gfx.toasts,
            &w2l,
            invert,
            x,
            y,
        );
        if warned {
            self.title_dirty = true;
        }
        consumed
    }

    /// Move do Sculpt.
    pub(crate) fn flip_reshape_canvas_move(&mut self, x: f32, y: f32) -> bool {
        if self.flip_state.reshape.is_none() {
            return false;
        }
        let w2l = self.flip_active_world_to_local();
        let invert = self.modifiers.control_key();
        let playhead = self.playhead;
        let Some(gfx) = self.gfx.as_mut() else {
            // Havia gesto mas não há janela: consome na mesma (o `return true` de antes).
            return true;
        };
        let win = gfx.surface.size();
        let mut f = FlipFrame {
            flip: &mut gfx.flip,
            playhead: &playhead,
            camera: &gfx.camera,
            win,
        };
        ph2d_app_flip::reshape::canvas_move(&mut self.flip_state, &mut f, &w2l, invert, x, y)
    }

    /// Pen-up do Sculpt.
    pub(crate) fn flip_reshape_canvas_up(&mut self) -> bool {
        ph2d_app_flip::reshape::canvas_up(&mut self.flip_state)
    }
}
