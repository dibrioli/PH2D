//! **O invólucro de shell** da borracha (W2/L5 Fase B 2.ª volta, 2026-09-12).
//!
//! A lei vive em [`ph2d_app_flip::erase`]. O que sobra aqui é o bloco `impl crate::App`:
//! destrancar o `Option<AppGfx>`, derivar o afim mundo→local e entregar à família os tipos
//! que ela pediu — todos de outra crate (`FlipDoc`, `Playhead`, `Camera2d`, `WindowSize`,
//! `Xform`), nunca a `App`.
//! ⚠️ O guarda `gfx.is_none()` fica deste lado: quem tem o `Option` é a shell.

impl crate::App {
    /// A tool Flip quer o canvas para APAGAR agora?
    #[must_use]
    pub(crate) fn flip_wants_erase(&self) -> bool {
        ph2d_app_flip::erase::wants(&self.flip_state)
    }

    /// Pen-down da borracha.
    pub(crate) fn flip_erase_canvas_down(&mut self, x: f32, y: f32) -> bool {
        // Fronteira MUNDO→LOCAL (ADR-0111): a geometria de um objeto já movido pelo gizmo é
        // LOCAL, então o cursor desce ao espaço local e o raio recua pela escala.
        let w2l = self.flip_active_world_to_local();
        let Some(gfx) = self.gfx.as_mut() else {
            return false;
        };
        let win = gfx.surface.size();
        ph2d_app_flip::erase::canvas_down(
            &mut self.flip_state,
            &mut gfx.flip,
            &self.playhead,
            &gfx.camera,
            win,
            &w2l,
            x,
            y,
        )
    }

    /// Move com a borracha em baixo.
    pub(crate) fn flip_erase_canvas_move(&mut self, x: f32, y: f32) -> bool {
        let w2l = self.flip_active_world_to_local();
        let Some(gfx) = self.gfx.as_mut() else {
            return false;
        };
        let win = gfx.surface.size();
        ph2d_app_flip::erase::canvas_move(
            &mut self.flip_state,
            &mut gfx.flip,
            &self.playhead,
            &gfx.camera,
            win,
            &w2l,
            x,
            y,
        )
    }

    /// Pen-up da borracha.
    pub(crate) fn flip_erase_canvas_up(&mut self) -> bool {
        let Some(gfx) = self.gfx.as_mut() else {
            return false;
        };
        ph2d_app_flip::erase::canvas_up(&mut self.flip_state, &mut gfx.flip, &self.playhead)
    }
}
