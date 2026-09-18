//! **O invólucro de shell** do hover de Segment (W2/L5 Fase B 2.ª volta, 2026-09-12).
//!
//! A lei vive em [`ph2d_app_flip::select_segment`]. Aqui fica o `impl crate::App`: o ponteiro,
//! o afim mundo→local, e a resposta do `edit_gesture` à pergunta *«a tool quer o canvas?»*.

use ph2d_app_flip::ctx::FlipFrame;

impl crate::App {
    /// §4.C — recomputa o pedaço sob o cursor no modo Segment.
    pub(crate) fn flip_segment_hover_refresh(&mut self) {
        let wants_edit = self.flip_wants_edit();
        let cursor = self.last_pointer;
        let w2l = self.flip_active_world_to_local();
        let playhead = self.playhead;
        // ⚠️ O `gfx` entra por `Option`: sem ele o hover fica `None`, mas o GUARDA de modo
        // corre na mesma — limpar o hover ao sair do Segment não depende de haver janela.
        let mut gfx = self.gfx.as_mut();
        let f = gfx.as_mut().map(|g| {
            let win = g.scene_window();
            FlipFrame {
                flip: &mut g.flip,
                playhead: &playhead,
                camera: &g.camera,
                win,
            }
        });
        ph2d_app_flip::select_segment::hover_refresh(
            &mut self.flip_state,
            f.as_ref(),
            &w2l,
            cursor,
            wants_edit,
        );
    }
}
