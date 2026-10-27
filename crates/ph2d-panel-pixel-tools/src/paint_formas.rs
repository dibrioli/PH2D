//! ⭐ **As opções da forma na mão** (P6b, dono 26/10: *«as formas do Vector, com opções específicas
//! no painel à direita como fillet»*): os campos do catálogo do Vector, com os nomes dele, por ordem;
//! as escolhas (o tipo do canto) são segmentos; as quatro têm «Fill» (P6c: no lugar das ferramentas
//! cheias).

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
        let fill = (
            SHAPE_FILL,
            tr("panel.pixel.shape_fill"),
            info.options.shape_fill,
        );
        self.caixas(&[fill], sec, y)
    }
}
