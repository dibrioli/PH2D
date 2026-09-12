//! **O invólucro de shell** da correção de pares (W2/L5 Fase B 2.ª volta, 2026-09-12).
//!
//! A lei vive em [`crate::flip::tween_correct`]. Aqui fica o `impl crate::App`: destrancar o
//! `Option<AppGfx>` e entregar o documento, o relógio, a câmera e o mundo do ECS.

use crate::flip::ctx::FlipFrame;

impl crate::App {
    /// A tool Flip quer o canvas para RE-PAREAR agora?
    #[must_use]
    pub(crate) fn flip_wants_tween_pairs(&self) -> bool {
        crate::flip::tween_correct::wants(&self.flip_state)
    }

    /// Re-pina a sessão ao intervalo atual quando o artista navega.
    pub(crate) fn flip_tween_pairs_upkeep(&mut self) {
        let playhead = self.playhead;
        let Some(gfx) = self.gfx.as_ref() else {
            return;
        };
        crate::flip::tween_correct::upkeep(&mut self.flip_state, &gfx.flip, &playhead);
    }

    /// Pen-down no modo Pairs.
    pub(crate) fn flip_tween_pairs_canvas_down(&mut self, x: f32, y: f32) -> bool {
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
        crate::flip::tween_correct::canvas_down(&mut self.flip_state, &f, &gfx.sim, x, y)
    }
}
