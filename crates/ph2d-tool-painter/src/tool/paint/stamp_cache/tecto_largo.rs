//! B29.1: quando o traço usa o tecto [`ph2d_painter_brush::TexelLargo`] (decisão 3 do dono, `BACKLOG.md` B29.1).

use crate::tool::PainterTool;
use ph2d_painter_brush::{BrushBlend, BrushSpec};

impl PainterTool {
    /// O carimbo por dab deste traço usa o tecto largo? Escolhe-se no 1.º lote e vale até o pen-up.
    ///
    /// `Largo` só quando a tela pode recompor `pré·(1 − m) + cor·m` (a cor é a MESMA em todo o traço: sem
    /// Randomize Color, sem rampa nem cor do Shape; Mix sem pigmento; sem Preserve Alpha — as condições de
    /// `Invariantes::recompoe`) e nenhum leitor do `stroke_mask` `u8` corre neste traço (Composite de ≥ 2 camadas, aguada,
    /// 2.ª ponta; nem outra rota que já o escreveu). O Solid lê o largo (`solid_deposit`: `Recorte<T>`, a mancha
    /// recompõe; `observa_a_passagem_do_solid`).
    pub(in crate::tool::paint) fn quer_o_tecto_largo(
        &mut self,
        brush: &BrushSpec,
        alpha_locked: bool,
    ) -> bool {
        if let Some(e) = self.paint.tecto_largo.escolha {
            return e;
        }
        #[cfg(test)]
        let veto =
            self.paint.tecto_largo.so_byte || tests::VETO_DA_SONDA.with(std::cell::Cell::get);
        #[cfg(not(test))]
        let veto = false;
        #[cfg(test)]
        let sem_porta = self.paint.tecto_largo.sem_a_porta_do_meio;
        #[cfg(not(test))]
        let sem_porta = false;
        let pig = brush.pigmento();
        // Só o Wet Paint evolui (dono 07/10; decisão do integrador 13/11, BUGS_painter #115): Digital, Impasto e
        // Watercolor ficam no `u8`, os bytes do main 4f46a3573c (`os_outros_meios_nao_mudam`).
        let e = !veto
            && (sem_porta || self.paint_media() == crate::PaintMedia::WetPaint)
            && !alpha_locked
            && brush.blend == BrushBlend::Mix
            && pig.mix <= 0.0
            && !pig.espectral
            && !(brush.color_jitter_enabled && brush.has_colour_jitter_amount())
            && !self.shape_tom_ativo()
            && !self.paint.shape_layers.any_texture_color()
            && self.paint.stroke_mask.is_empty()
            // O plano do Digital (cauda, modo de cor) leva o largo: a cauda limpa-o e repõe-no (`cauda.rs`); a 2.ª
            // ponta do plano recorta pelo `u8`.
            && self.paint.plano_do_traco.is_none() // dentro do plano a sessão está tirada (`stamp_dabs_no_plano`)
            && !(self.traco_no_plano() && self.paint.brush.dual.is_some())
            && self.passagem_do_solid().is_none()
            && !(self.composite_active() && self.camadas_activas() >= 2) // 1 camada: troca o largo por camada
            && !self.watercolor_render_active()
            && !self.dual_na_ferramenta();
        self.paint.tecto_largo.escolha = Some(e);
        e
    }
}

#[cfg(test)]
#[path = "tecto_largo_tests.rs"]
mod tests;
