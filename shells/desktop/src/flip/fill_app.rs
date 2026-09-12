//! **O invólucro de shell** do balde (W2/L5 Fase B 2.ª volta, 2026-09-12).
//!
//! A lei vive em [`crate::flip::fill`]. Aqui fica o `impl crate::App`: destrancar o
//! `Option<AppGfx>`, derivar o afim mundo→local, e marcar o `title_dirty` a partir do
//! [`crate::flip::fill::FillOutcome`] — *a fronteira atravessa-se com um valor*.

use crate::flip::ctx::FlipFrame;

impl crate::App {
    /// A tool Flip quer o canvas para PREENCHER agora?
    #[must_use]
    pub(crate) fn flip_wants_fill(&self) -> bool {
        crate::flip::fill::wants(&self.flip_state)
    }

    /// O clique do balde. `true` = consumido.
    pub(crate) fn flip_fill_canvas_down(&mut self, x: f32, y: f32) -> bool {
        let w2l = self.flip_active_world_to_local();
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
        let out = crate::flip::fill::canvas_down(
            &mut self.flip_state,
            &mut f,
            &mut gfx.toasts,
            &w2l,
            x,
            y,
        );
        if out.warned {
            self.title_dirty = true;
        }
        out.consumed
    }
}
