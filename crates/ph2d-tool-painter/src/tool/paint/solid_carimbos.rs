//! **O Solid com uma ponta de IMAGEM** — a mancha é um lote de carimbos da própria ponta, com o jitter
//! dela ([`ph2d_painter_brush::solid_carimbos`], smoke do dono 12/10), em vez da mancha lisa na cor do
//! Color. As duas famílias de depósito levam-no pela porta que já tinham: o cumulativo como a CORDA
//! (rascunho dentro da transação da mancha), o re-carimbo junto com o traço, por baixo dele.

use crate::tool::PainterTool;
use ph2d_painter_brush::Dab;

impl PainterTool {
    /// Os carimbos que enchem a mancha deste gesto, na ordem de empilhamento — vazio quando a ponta não
    /// é uma imagem (a mancha lisa de sempre) ou não há região. Saem dos laços ESPELHADOS e não
    /// envolvidos: o despacho do carimbo envolve-os pelo Tiling, como a qualquer dab.
    pub(super) fn solid_fill_stamps(&self) -> Vec<Dab> {
        let brush = self.stroke_spec();
        if !brush.solid_fills_with_stamps(self.paint.shape_image.is_some()) {
            return Vec::new();
        }
        let loops = self.solid_mirrored_loops();
        if loops.is_empty() {
            return Vec::new();
        }
        let (w, h) = self.source_size;
        ph2d_painter_brush::solid_carimbos::fill_dabs(&brush, &loops, w, h, self.paint.seed)
    }
}
