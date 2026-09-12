//! **O invólucro de shell** do Colorize (W2/L5 Fase B 2.ª volta, 2026-09-12).
//!
//! A lei vive em [`crate::flip::colorize`]. Aqui fica o `impl crate::App`: destrancar o
//! `Option<AppGfx>`, derivar o afim mundo→local, e traduzir o valor de retorno em
//! `title_dirty`.

use crate::flip::ctx::FlipFrame;

impl crate::App {
    /// A tool Flip quer o canvas para RABISCAR agora?
    #[must_use]
    pub(crate) fn flip_wants_colorize(&self) -> bool {
        crate::flip::colorize::wants(&self.flip_state)
    }

    /// Pen-down: começa um rabisco novo com a cor atual do Colorize.
    pub(crate) fn flip_colorize_canvas_down(&mut self, x: f32, y: f32) -> bool {
        let playhead = self.playhead;
        let Some(gfx) = self.gfx.as_mut() else {
            return false;
        };
        let win = gfx.surface.size();
        let f = FlipFrame {
            flip: &mut gfx.flip,
            playhead: &playhead,
            camera: &gfx.camera,
            win,
        };
        crate::flip::colorize::canvas_down(&mut self.flip_state, &f, x, y)
    }

    /// Pen-move: acumula amostras.
    pub(crate) fn flip_colorize_canvas_move(&mut self, x: f32, y: f32) -> bool {
        let playhead = self.playhead;
        let Some(gfx) = self.gfx.as_mut() else {
            return false;
        };
        let win = gfx.surface.size();
        let f = FlipFrame {
            flip: &mut gfx.flip,
            playhead: &playhead,
            camera: &gfx.camera,
            win,
        };
        crate::flip::colorize::canvas_move(&mut self.flip_state, &f, x, y)
    }

    /// Pen-up: fecha o rabisco em curso e o acumula.
    pub(crate) fn flip_colorize_canvas_up(&mut self) -> bool {
        crate::flip::colorize::canvas_up(&mut self.flip_state)
    }

    /// **Clear** — descarta os rabiscos acumulados.
    pub(crate) fn flip_colorize_clear(&mut self) {
        crate::flip::colorize::clear(&mut self.flip_state);
    }

    /// GPU-data dos rabiscos para o overlay ao vivo.
    #[must_use]
    pub(crate) fn flip_colorize_preview_data(&self) -> Option<ph2d_flip_render::FlipGpuData> {
        let w2l = self.flip_active_world_to_local();
        crate::flip::colorize::preview_data(&self.flip_state, &w2l)
    }

    /// **Apply** — roda o corte LazyBrush e materializa as regiões.
    pub(crate) fn flip_colorize_apply(&mut self) {
        let w2l = self.flip_active_world_to_local();
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
        if crate::flip::colorize::apply(&mut self.flip_state, &mut f, &mut gfx.toasts, &w2l) {
            self.title_dirty = true;
        }
    }
}
