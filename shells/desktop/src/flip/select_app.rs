//! **O invólucro de shell** da selecção do Edit Mode (W2/L5 Fase B 2.ª volta, 2026-09-12).
//!
//! A lei vive em [`ph2d_app_flip::select`]. Aqui fica o `impl crate::App`: destrancar o
//! `Option<AppGfx>`, derivar os DOIS afins (pose-aware e pose-free), ler o Shift, e marcar o
//! `title_dirty` a partir do valor de retorno.

use ph2d_app_flip::ctx::FlipFrame;

impl crate::App {
    /// O DOMÍNIO do toggle do painel (Stroke|Point) agora.
    #[must_use]
    pub(crate) fn flip_edit_domain_now(&self) -> ph2d_tool_flip::EditDomain {
        ph2d_app_flip::select::edit_domain_now(&self.flip_state)
    }

    /// A tool Flip quer o canvas para SELECIONAR agora?
    #[must_use]
    pub(crate) fn flip_wants_edit(&self) -> bool {
        ph2d_app_flip::select::wants_edit(&self.flip_state)
    }

    /// O clique de seleção.
    pub(crate) fn flip_edit_canvas_down(&mut self, x: f32, y: f32) -> bool {
        let shift = self.modifiers.shift_key();
        let w2l = self.flip_active_world_to_local();
        let w2o = self.flip_active_world_to_object();
        let playhead = self.playhead;
        let Some(gfx) = self.gfx.as_mut() else {
            return false;
        };
        let win = gfx.scene_window();
        let mut f = FlipFrame {
            flip: &mut gfx.flip,
            playhead: &playhead,
            camera: &gfx.camera,
            win,
        };
        let (consumed, dirty) = ph2d_app_flip::select::canvas_down(
            &mut self.flip_state,
            &mut f,
            &mut gfx.toasts,
            &w2l,
            &w2o,
            shift,
            (x, y),
        );
        if dirty {
            self.title_dirty = true;
        }
        consumed
    }

    /// Apaga os traços selecionados do desenho visível.
    pub(crate) fn flip_delete_selected(&mut self) -> bool {
        let playhead = self.playhead;
        let Some(gfx) = self.gfx.as_mut() else {
            return false;
        };
        let deleted =
            ph2d_app_flip::select::delete_selected(&self.flip_state, &mut gfx.flip, &playhead);
        if deleted {
            self.title_dirty = true;
        }
        deleted
    }
}
