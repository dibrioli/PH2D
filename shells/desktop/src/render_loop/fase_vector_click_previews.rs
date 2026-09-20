//! **Fase do quadro: OS TRÊS REALCES DE UM CLIQUE DA CANETA** — o pedaço que o Trim vai apagar, a
//! face que o Balde vai preencher e o ponto que a caneta ia inserir.
//!
//! ⚠️ Ela saiu da [`super::fase_vector_overlays`] na integração de 2026-09-20, por tecto de LOC
//! (`214` contra `200`) e **por responsabilidade** — a prosa dos três blocos já os declarava um
//! assunto só: *«ao lado dos dois realces acima e pela mesma razão: ele responde a «o que este
//! clique faria?»»*.
//!
//! ⭐⭐ **Ela RE-DERIVA o `gfx`, e é isso que a torna possível:** a mãe tem `self.gfx` emprestado
//! até à última leitura do `vector_scene`, e um `self.fase_…(` com esse empréstimo vivo não
//! compila. Chamada DEPOIS dele, o empréstimo já morreu (NLL) e a fase-filha pode pedir o `self`
//! inteiro. ⚠️ A ordem no quadro fica **idêntica**: o texto emendado põe este corpo no sítio da
//! chamada.

use super::*;

impl crate::App {
    /// Ver o cabeçalho do módulo.
    pub(super) fn fase_vector_click_previews(
        &mut self,
        cam_affine: ph2d_vector::Affine,
    ) -> Option<()> {
        let gfx = self.gfx.as_mut()?;
        let FrameGfx { vector_scene, .. } = FrameGfx::of(gfx);
        // ⭐⭐⭐ **O REALCE DO TRIM** (plano 38): o pedaço que o clique vai apagar, a vermelho,
        // como no Fusion. ⚠️ **A geometria vem da MESMA porta que o corte** — ela é calculada
        // no dreno do ponteiro e guardada, então o que acende neste quadro é literalmente o que
        // o `vec_trim::apply` vai comer. Uma segunda conta aqui seria a divergência mais cara
        // que uma ferramenta destrutiva pode ter.
        if !self.vec.trim_piece.is_empty() {
            ph2d_vec_render::draw_trim_piece(&self.vec.trim_piece, cam_affine, vector_scene);
        }
        // ⭐⭐⭐ **A FACE que o Balde vai preencher** (plano 40), na TINTA que ele vai depositar.
        // ⚠️ A geometria vem da MESMA porta que o preenchimento usa; e a tinta é a corrente,
        // não uma cor neutra — um realce noutra cor prometeria uma coisa e entregaria outra.
        // ⚠️ A tinta é lida do CAMPO (`vec_pen`), e não pelo `bucket_paint()`: um método em
        // `&self` pede o objecto INTEIRO emprestado, e aqui há um empréstimo mútuo vivo. Os
        // campos são disjuntos; a lei do `alpha == 0` é a mesma dos dois lados.
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
