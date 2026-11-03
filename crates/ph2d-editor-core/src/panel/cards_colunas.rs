//! As colunas `(x, w)` de uma linha de cartão: as de ícones têm largura fixa; as outras dividem o
//! resto por igual — salvo a caixa da cor, que cresce até ao que mostra (o nome do estilo, maquete
//! F) sem deixar as vizinhas abaixo de [`m::FIELD_VALUE_MIN_W`].

use super::{Cell, Widget};
use ph2d_card_stack::metrics as m;

fn fixa(c: &Cell<'_>) -> Option<f32> {
    match c.widget {
        Widget::Icons(s) | Widget::Bare(s) => Some(m::FIELD_H * s.len() as f32),
        Widget::Badge(_) => Some(m::FIELD_H),
        _ => None,
    }
}

/// `largura(t)` mede `t` a `VALUE_PX`.
pub(super) fn colunas(
    x0: f32,
    w: f32,
    cells: &[Cell<'_>],
    largura: &mut dyn FnMut(&str) -> f32,
) -> Vec<(f32, f32)> {
    let gaps = m::FIELD_GAP * cells.len().saturating_sub(1) as f32;
    let fixas: f32 = cells.iter().filter_map(fixa).sum();
    let flex = cells.iter().filter(|c| fixa(c).is_none()).count().max(1) as f32;
    let livre = (w - gaps - fixas).max(0.0);
    let cada = livre / flex;
    // A caixa da cor: amostra + texto + (corrente partida | folga).
    let cor = cells.iter().find_map(|c| match c.widget {
        Widget::Swatch { text, detach, .. } => Some(
            m::ICON_CELL
                + largura(text).ceil()
                + if detach.is_some() {
                    m::ICON_CELL
                } else {
                    m::FIELD_PAD_R
                },
        ),
        _ => None,
    });
    let cor = cor.map(|need| {
        let teto = livre - (flex - 1.0) * m::FIELD_VALUE_MIN_W;
        need.min(teto).max(cada)
    });
    let resto = cor.map_or(cada, |c| (livre - c) / (flex - 1.0).max(1.0));
    let mut x = x0;
    cells
        .iter()
        .map(|c| {
            let w = fixa(c).unwrap_or(match (c.widget, cor) {
                (Widget::Swatch { .. }, Some(w)) => w,
                _ => resto,
            });
            let col = (x, w);
            x += w + m::FIELD_GAP;
            col
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::panel::cards::Seg;
    use ph2d_a11y::NodeId;

    /// ⭐ Maquete F: a camada ligada mostra o NOME do estilo inteiro, à largura do Inspector da cena
    /// `PH2D_VEC_CARTOES_SMOKE=3` (o corpo do cartão mede `257 px` na foto de 02/11 — antes saía
    /// «Bra…»). A caixa do texto é a da cor menos a amostra e a corrente partida (`cards::swatch`).
    #[test]
    fn a_linked_style_name_is_not_cut_at_the_inspector_width() {
        let olho = [Seg {
            id: NodeId(1),
            icon: None,
            text: "",
            on: false,
        }; 2];
        let mut ts = ph2d_text::TextSystem::without_system_fonts();
        for nome in ["Brand orange", "Brand blue", "FFFFFF"] {
            let cells = [
                Cell::new(Widget::Swatch {
                    id: NodeId(2),
                    rgba: None,
                    text: nome,
                    edit: None,
                    detach: Some(NodeId(3)),
                }),
                Cell::new(Widget::Field {
                    id: NodeId(4),
                    glyph: crate::widget::card_field::Glyph::Text("100"),
                }),
                Cell::new(Widget::Bare(&olho)),
            ];
            let cols = colunas(0.0, 257.0, &cells, &mut |t| ts.prefix_width(t, m::VALUE_PX));
            let texto = cols[0].1 - m::ICON_CELL * 2.0;
            let cabe = crate::text_elide::fit(&mut ts, nome, m::VALUE_PX, texto);
            assert_eq!(cabe, nome, "cortado em {texto} px");
            // A opacidade ainda mostra «100» inteiro, com o ícone.
            let op = cols[1].1 - m::ICON_CELL - m::FIELD_PAD_R;
            assert_eq!(
                crate::text_elide::fit(&mut ts, "100", m::VALUE_PX, op),
                "100"
            );
            assert!(
                cols[0].1 >= cols[1].1,
                "a cor nunca fica mais estreita que a opacidade"
            );
        }
    }
}
