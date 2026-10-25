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

    /// A selecção do Flip no modo Edit: o ESTILO e o DOMÍNIO, depois do quadro (saído do
    /// `render_frame` pelo tecto do `main.rs`).
    pub(crate) fn flip_edit_refresh(&mut self) {
        // **A SELEÇÃO** (`flip_select`, W6): no modo Edit ela é o alvo dos ajustes do
        // painel. Só a MUDANÇA de estilo age.
        // ⭐ W2/L5 2.ª volta: as duas pedem o DOCUMENTO e o relógio, nunca a `App`.
        {
            let playhead = self.playhead;
            if let Some(gfx) = self.gfx.as_mut()
                && ph2d_app_flip::select::flip_edit_style_refresh(
                    &mut self.flip_state,
                    &mut gfx.flip,
                    &playhead,
                )
            {
                self.title_dirty = true;
            }
        }
        // **O DOMÍNIO da seleção** (W8): a troca Stroke↔Point converte a seleção no
        // documento (broadcast/promoção) — uma vez, quando o toggle muda.
        {
            let playhead = self.playhead;
            if let Some(gfx) = self.gfx.as_mut()
                && ph2d_app_flip::select_points::flip_edit_domain_refresh(
                    &mut self.flip_state,
                    &mut gfx.flip,
                    &playhead,
                )
            {
                self.title_dirty = true;
            }
        }
        // Depois do frame (estado já reconciliado pelo `sync`, `self` livre do borrow
        // do render loop): drena um Ctrl+Z/Y pendente e registra a ação do frame na
        // fila de undo global, por diff de estado (ver `undo::post_frame_undo`).
        // ⭐ O SYNC das instâncias (ADR-0164): DEPOIS do quadro e ANTES da captura (senão a escrita
        // do sync vira um passo de undo que ninguém deu). Ver o doc da função.
    }
}
