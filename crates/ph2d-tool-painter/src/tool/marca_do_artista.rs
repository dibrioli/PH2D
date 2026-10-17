//! **A marca do artista no documento do Painter** (W9, `docs/Macros/07_plano_W9.md`): quem mostra uma
//! macro em cena dá ao Painter a marca da imagem ligada ([`PainterTool::poe_a_marca`]) e lê-a de volta
//! ([`PainterTool::marca_do_artista`]); a história escreve-a ([`crate::undo::marca_viva`]).

use super::PainterTool;
use crate::macro_track::marca::MarcaDoArtista;
use ph2d_painter_brush::Dab;

impl PainterTool {
    /// A marca da imagem ligada (`None` = a imagem não tem macro: nada a marcar).
    pub fn poe_a_marca(&mut self, m: Option<MarcaDoArtista>) {
        self.undo.marca.poe(m);
    }

    /// A marca da imagem ligada, com o pincel do passo em curso.
    #[must_use]
    pub fn marca_do_artista(&self) -> Option<&MarcaDoArtista> {
        self.undo.marca.marca.as_ref()
    }

    /// O pincel cobre cada dab de `dabs`, com o alcance `reach` da ponta (`na_agua` = entram na água).
    pub(crate) fn marca_os_dabs(&mut self, dabs: &[Dab], reach: f32, na_agua: bool) {
        if !self.undo.marca.ativa() {
            return;
        }
        for d in dabs {
            let r = d.radius_px * reach + 0.5;
            self.undo.marca.disco(d.center[0], d.center[1], r, na_agua);
        }
    }
}
