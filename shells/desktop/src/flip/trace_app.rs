//! **O invólucro de shell** do Trace (W2/L5 Fase B 2.ª volta, 2026-09-12).
//!
//! A lei vive em [`ph2d_app_flip::trace`]. O que sobra aqui é o bloco `impl crate::App`:
//! destrancar o `Option<AppGfx>`, derivar o afim mundo→objeto e ler o modificador — o resto
//! é da família.

use ph2d_app_flip::ctx::FlipFrame;

impl crate::App {
    /// A tool Flip quer o canvas para DESLOCAR fantasmas agora?
    #[must_use]
    pub(crate) fn flip_wants_trace(&self) -> bool {
        ph2d_app_flip::trace::wants(&self.flip_state)
    }

    /// Pen-down no modo Trace.
    pub(crate) fn flip_trace_canvas_down(&mut self, x: f32, y: f32) -> bool {
        let w2o = self.flip_active_world_to_object();
        let ctrl = self.modifiers.control_key();
        let playhead = self.playhead;
        let Some(gfx) = self.gfx.as_mut() else {
            return false;
        };
        let win = gfx.scene_window();
        let f = FlipFrame {
            flip: &mut gfx.flip,
            playhead: &playhead,
            camera: &gfx.camera,
            win,
        };
        ph2d_app_flip::trace::canvas_down(&mut self.flip_state, &f, &w2o, ctrl, x, y)
    }

    /// Movimento com arrasto de trace aberto.
    pub(crate) fn flip_trace_canvas_move(&mut self, x: f32, y: f32) -> bool {
        let w2o = self.flip_active_world_to_object();
        let playhead = self.playhead;
        let Some(gfx) = self.gfx.as_mut() else {
            return false;
        };
        let win = gfx.scene_window();
        let f = FlipFrame {
            flip: &mut gfx.flip,
            playhead: &playhead,
            camera: &gfx.camera,
            win,
        };
        ph2d_app_flip::trace::canvas_move(&mut self.flip_state, &f, &w2o, x, y)
    }

    /// Pen-up: fecha o arrasto de trace.
    pub(crate) fn flip_trace_canvas_up(&mut self) -> bool {
        ph2d_app_flip::trace::canvas_up(&mut self.flip_state)
    }
}
