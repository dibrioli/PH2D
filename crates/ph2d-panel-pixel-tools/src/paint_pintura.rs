//! **P13 U-Pintura: as opções do ink Dither e do Smudge ▸ Push** — Level e Matrix sob a fila de
//! tintas, Strength na ferramenta Push. Os ids e as faixas moram em [`crate::ids_pintura`].

use super::*;
use crate::ids_pintura as idp;

impl Corpo<'_> {
    /// O Level e a Matrix do ink Dither (só com ele escolhido).
    pub(super) fn opcoes_dither(&mut self, info: &PixelPanelInfo, sec: Seccao, y: f32) -> f32 {
        let o = info.options;
        if o.ink != PixelInk::Dither {
            return y;
        }
        let y = self.campo(tr("pixel.ink.dither.level"), idp::DITHER_LEVEL, sec, y);
        let matriz: Vec<(&str, bool, NodeId)> = idp::DITHER_MATRICES
            .iter()
            .map(|&(d, id, k)| (tr(k), d == o.dither_matrix, id))
            .collect();
        self.escolha(tr("pixel.ink.dither.matrix"), &matriz, sec, y)
    }

    /// A força do Push.
    pub(super) fn opcoes_push(&mut self, info: &PixelPanelInfo, sec: Seccao, y: f32) -> f32 {
        if info.tool != PixelTool::Push {
            return y;
        }
        self.campo(tr("pixel.smudge.push.strength"), idp::PUSH_STRENGTH, sec, y)
    }
}
