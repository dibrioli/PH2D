//! ⭐ **As opções da forma na mão** (P6b, dono 26/10: *«as formas do Vector, com opções específicas
//! no painel à direita como fillet»*): os campos do catálogo do Vector, com os nomes dele, por ordem;
//! as escolhas (o tipo do canto) são segmentos; as fechadas têm «Fill» (P6c: no lugar das
//! ferramentas cheias).

use super::*;
use crate::seam_formas::nome;
use crate::seam_formas::{SHAPE_FILL, desc, field_id, option_id};
use ph2d_editor_core::pixel_panel::PixelShape;
use ph2d_tool_vector::shapes::FieldUnit;

/// Os rótulos dos campos de todas as formas (a coluna medida das propriedades).
pub(super) fn nomes() -> Vec<&'static str> {
    crate::seam_formas::fields()
        .map(|(.., f)| nome(f.label))
        .chain([tr("panel.pixel.shape_fill")])
        .collect()
}

impl Corpo<'_> {
    pub(super) fn opcoes_forma(&mut self, info: &PixelPanelInfo, sec: Seccao, y: f32) -> f32 {
        let Some(s) = PixelShape::of_tool(info.tool) else {
            return y;
        };
        let v = info.options.shapes.get(s);
        let mut y = self.linha(tr("panel.pixel.shape_options"), ColorToken::Text3, y);
        for (i, f) in desc(s).fields.iter().enumerate() {
            y = match f.unit {
                FieldUnit::Choice(opcoes) => {
                    let seg: Vec<(&str, bool, NodeId)> = opcoes
                        .iter()
                        .enumerate()
                        .map(|(j, o)| {
                            #[expect(clippy::cast_precision_loss, reason = "índice de escolha")]
                            let on = v.get(i).round() == j as f64;
                            (nome(o), on, option_id(s, i, j))
                        })
                        .collect();
                    self.escolha(nome(f.label), &seg, sec, y)
                }
                _ => self.campo(nome(f.label), field_id(s, i), sec, y),
            };
        }
        // A forma ABERTA do Vector (linha, arco, espiral…) não tem interior: sem «Fill».
        if !s.kind().is_closed() {
            return y;
        }
        let fill = (
            SHAPE_FILL,
            tr("panel.pixel.shape_fill"),
            info.options.shape_fill,
        );
        self.caixas(&[fill], sec, y)
    }
}

impl Corpo<'_> {
    /// P8: as opções do Gradient — Linear | Radial, a matriz DELE (não a da dinâmica) e o «Palette
    /// ramp»; a região (tolerância, contígua, 4/8) é a do balde, logo abaixo.
    pub(super) fn opcoes_gradiente(&mut self, info: &PixelPanelInfo, sec: Seccao, y: f32) -> f32 {
        let o = info.options;
        let y = self.linha(tr("panel.pixel.gradient"), ColorToken::Text3, y);
        let forma = [
            (
                tr("panel.pixel.gradient.linear"),
                !o.gradient_radial,
                ids::GRADIENT_LINEAR,
            ),
            (
                tr("panel.pixel.gradient.radial"),
                o.gradient_radial,
                ids::GRADIENT_RADIAL,
            ),
        ];
        let y = self.escolha(tr("panel.pixel.gradient.shape"), &forma, sec, y);
        let matriz: Vec<(&str, bool, NodeId)> = ids::GRADIENT_DITHERS
            .iter()
            .map(|&(d, id, k)| (tr(k), d == o.gradient_dither, id))
            .collect();
        let y = self.escolha(tr("panel.pixel.gradient.dither"), &matriz, sec, y);
        self.caixas(
            &[(
                ids::GRADIENT_RAMP,
                tr("panel.pixel.gradient.ramp"),
                o.gradient_ramp,
            )],
            sec,
            y,
        )
    }
}
