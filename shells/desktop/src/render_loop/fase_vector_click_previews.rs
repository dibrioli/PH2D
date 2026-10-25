//! **Fase do quadro: OS TRÊS REALCES DE UM CLIQUE DA CANETA** — o pedaço que o Trim vai apagar, a
//! face que o Balde vai preencher e o ponto que a caneta ia inserir.
//!
//! Filha da [`super::fase_vector_overlays`] (2026-09-20): RE-DERIVA o `gfx` (a mãe já largou o
//! empréstimo), e o texto emendado do quadro põe este corpo no sítio da chamada.

use super::*;

impl crate::App {
    /// Ver o cabeçalho do módulo.
    pub(super) fn fase_vector_click_previews(
        &mut self,
        cam_affine: ph2d_vector::Affine,
    ) -> Option<()> {
        let gfx = self.gfx.as_mut()?;
        let FrameGfx {
            vector_scene,
            hero_screen,
            ..
        } = FrameGfx::of(gfx);
        let theme = hero_screen.as_ref()?.theme;
        // ⭐⭐⭐ **O REALCE DO TRIM** (plano 38): a geometria é a MESMA que o `trim::apply` vai
        // comer (calculada no dreno do ponteiro) — nunca uma segunda conta aqui.
        if !self.vec.trim_piece.is_empty() {
            ph2d_vec_render::draw_trim_piece(&self.vec.trim_piece, cam_affine, theme, vector_scene);
        }
        // ⭐⭐⭐ **A FACE que o Balde vai preencher** (plano 40), pela MESMA porta do preenchimento
        // e na tinta corrente — lida do CAMPO: o `bucket_paint()` pediria o `self` inteiro.
        let tinta_balde = self.vec.pen.style().fill;
        if let Some(face) = self.vec.bucket_face.as_ref()
            && tinta_balde.a != 0
        {
            let t = tinta_balde;
            ph2d_vec_render::draw_bucket_face(
                &face.face,
                [t.r, t.g, t.b, t.a],
                cam_affine,
                vector_scene,
            );
        }
        // ⭐⭐⭐ **ONDE UM CLIQUE DA CANETA PORIA UM PONTO** (report do dono, 2026-09-19: *«não tem
        // indicação visual que você está em cima da linha para criar um ponto»*), ao lado dos dois
        // realces acima e pela mesma razão: ele responde a *«o que este clique faria?»*.
        //
        // ⚠️ **A posição vem da MESMA porta que o clique usa** (`PenTool::previa_de_insercao`, que
        // chama o `insert_hit` e decide o raio com a mesma linha do press). Uma segunda conta aqui
        // acenderia a marca num sítio em que o clique já não insere — o defeito que o realce do Trim
        // e o do Balde nomeiam por escrito, logo acima.
        if let Some(p) = self.vec.previa_insercao {
            ph2d_vec_render::draw_insert_preview(p, cam_affine, vector_scene);
        }
        Some(())
    }
}
