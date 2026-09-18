//! **O invólucro de shell** do gesto de Edit (W2/L5 Fase B 2.ª volta, 2026-09-12).
//!
//! A lei vive em [`ph2d_app_flip::edit_gesture`]. Aqui fica o `impl crate::App`: destrancar o
//! `Option<AppGfx>`, derivar os afins e marcar o `title_dirty` a partir do valor de retorno.

use ph2d_app_flip::ctx::FlipFrame;

impl crate::App {
    /// Pen-move no modo Edit.
    pub(crate) fn flip_edit_canvas_move(&mut self, x: f32, y: f32) -> bool {
        if self.flip_state.edit_gesture.is_none() {
            return false;
        }
        let w2o = self.flip_active_world_to_object();
        let playhead = self.playhead;
        let Some(gfx) = self.gfx.as_mut() else {
            // Havia gesto mas não há janela: consome na mesma (o `return true` de antes).
            return true;
        };
        let win = gfx.scene_window();
        let mut f = FlipFrame {
            flip: &mut gfx.flip,
            playhead: &playhead,
            camera: &gfx.camera,
            win,
        };
        let (live, dirty) =
            ph2d_app_flip::edit_gesture::canvas_move(&mut self.flip_state, &mut f, &w2o, x, y);
        if dirty {
            self.title_dirty = true;
        }
        live
    }

    /// Pen-up no modo Edit.
    pub(crate) fn flip_edit_canvas_up(&mut self) -> bool {
        if self.flip_state.edit_gesture.is_none() {
            return false;
        }
        let w2l = self.flip_active_world_to_local();
        let playhead = self.playhead;
        let Some(gfx) = self.gfx.as_mut() else {
            // Havia gesto mas não há janela: consome na mesma, e o gesto morre — era o que
            // o `flip_screen_to_local` devolvendo `None` fazia em cada braço.
            self.flip_state.edit_gesture = None;
            return true;
        };
        let win = gfx.scene_window();
        let mut f = FlipFrame {
            flip: &mut gfx.flip,
            playhead: &playhead,
            camera: &gfx.camera,
            win,
        };
        let (consumed, dirty) =
            ph2d_app_flip::edit_gesture::canvas_up(&mut self.flip_state, &mut f, &w2l);
        if dirty {
            self.title_dirty = true;
        }
        consumed
    }
}
