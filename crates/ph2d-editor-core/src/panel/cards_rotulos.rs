//! A faixa dos rótulos de uma linha de cartão. ⭐ Um rótulo que não cabe fica numa linha com reticências, antes da
//! marca de animação da sua célula (dono 06/09 e 10/11: *«a real é … na label ao estreitar o painel … é regra»*);
//! a quebra em duas linhas da maquete F saiu.

use super::{CardCtx, Cell, Widget};
use crate::widget::card_field;
use ph2d_card_stack::metrics as m;
use ph2d_text::TextSystem;

/// `l` sem a unidade entre parênteses.
fn sem_unidade(l: &str) -> &str {
    l.split_once(" (").map_or(l, |(nome, _)| nome)
}

/// ⭐ **Dois campos da mesma grandeza em eixos** («Width (tiles)» e «Height (tiles)», «Offset X» e «Offset Y»):
/// os nomes diferem numa palavra só, e ela é o eixo. Ficam lado a lado em qualquer largura (dono 05/11).
pub(super) fn par_de_eixos(cells: &[Cell<'_>]) -> bool {
    const EIXOS: [(&str, &str); 3] = [("x", "y"), ("width", "height"), ("w", "h")];
    let [a, b] = cells else {
        return false;
    };
    let campo = |c: &Cell<'_>| matches!(c.widget, Widget::Field { .. } | Widget::FieldFace { .. });
    let (true, true, Some(la), Some(lb)) = (campo(a), campo(b), a.label, b.label) else {
        return false;
    };
    let palavras = |l| sem_unidade(l).split_whitespace().collect::<Vec<_>>();
    let (pa, pb) = (palavras(la), palavras(lb));
    let difere: Vec<_> = pa.iter().zip(&pb).filter(|(p, q)| p != q).collect();
    let [(p, q)] = difere.as_slice() else {
        return false;
    };
    pa.len() == pb.len()
        && EIXOS
            .iter()
            .any(|(e, f)| p.eq_ignore_ascii_case(e) && q.eq_ignore_ascii_case(f))
}

pub(super) fn reserva(c: &Cell<'_>) -> f32 {
    if c.key.is_some() {
        m::KEY_MARK_BOX
    } else {
        0.0
    }
}

/// O rótulo da célula `i` é o do grupo: vai até à próxima célula com rótulo («Per-corner Tint» sobre
/// quatro amostras), e pára antes da primeira marca de animação desse troço — nunca a pinta por cima.
fn largura_do_rotulo(cells: &[Cell<'_>], cols: &[(f32, f32)], i: usize) -> f32 {
    let fim = (i + 1..cells.len())
        .find(|&j| cells[j].label.is_some())
        .unwrap_or(cells.len());
    let ate = (i..fim)
        .find(|&j| cells[j].key.is_some())
        .unwrap_or(fim - 1);
    let (xf, wf) = cols[ate];
    xf + wf - cols[i].0 - reserva(&cells[ate])
}

/// Pinta os rótulos (e as marcas de animação) de uma linha a partir de `y`; devolve a altura da
/// faixa, já com o `LABEL_BOTTOM`.
pub(super) fn faixa(cx: &mut CardCtx<'_>, cells: &[Cell<'_>], cols: &[(f32, f32)], y: f32) -> f32 {
    let de_eixos = par_de_eixos(cells);
    let larguras: Vec<f32> = (0..cells.len())
        .map(|i| largura_do_rotulo(cells, cols, i))
        .collect();
    for (i, (c, r)) in cells.iter().zip(cols).enumerate() {
        let w = larguras[i];
        if let Some(l) = c.label {
            // No par de eixos a unidade sai antes das reticências; a dica do campo continua a dizê-la.
            let cabe = cx.text_system.prefix_width(l, m::LABEL_PX) <= w;
            let l = if de_eixos && !cabe { sem_unidade(l) } else { l };
            card_field::paint_label(cx.scene, cx.text_system, cx.ink, r.0, y, w, l);
        }
        if let Some(k) = c.key {
            card_field::paint_key_mark(
                cx.scene,
                cx.ink,
                cx.hit_index,
                r.0 + r.1 - m::KEY_MARK_BOX * 0.5,
                y + m::LABEL_PX * 0.5,
                k,
            );
        }
    }
    m::LABEL_PX + m::LABEL_BOTTOM
}

/// A faixa de uma linha sem nomes mas com marcas (as camadas de uma pilha abaixo da 1.ª): só a altura
/// da marca, para não repetir «Color · Opacity» em cada camada.
pub(super) fn so_marcas(
    cx: &mut CardCtx<'_>,
    cells: &[Cell<'_>],
    cols: &[(f32, f32)],
    y: f32,
) -> f32 {
    for (c, r) in cells.iter().zip(cols) {
        if let Some(k) = c.key {
            card_field::paint_key_mark(
                cx.scene,
                cx.ink,
                cx.hit_index,
                r.0 + r.1 - m::KEY_MARK_BOX * 0.5,
                y + m::KEY_MARK_BOX * 0.5,
                k,
            );
        }
    }
    m::KEY_MARK_BOX
}

#[cfg(test)]
mod tests {
    use super::*;

    /// ⭐ O rótulo do grupo tem a largura do grupo; sem isto «Per-corner Tint» quebrava sobre as amostras.
    /// *Mutação: `fim = i + 1`.*
    #[test]
    fn a_group_label_spans_the_cells_up_to_the_next_label() {
        use super::super::Widget;
        let seg = |l: Option<&'static str>| {
            let c = Cell::new(Widget::Readout { text: "" });
            match l {
                Some(t) => c.label(t),
                None => c,
            }
        };
        let cells = [seg(Some("A")), seg(None), seg(Some("B")), seg(None)];
        let cols = [(0.0, 40.0), (46.0, 40.0), (92.0, 40.0), (138.0, 40.0)];
        assert_eq!(largura_do_rotulo(&cells, &cols, 0), 86.0);
        assert_eq!(largura_do_rotulo(&cells, &cols, 2), 86.0);
    }

    /// ⭐ Dono 10/11: o rótulo nunca passa por cima da marca da SUA célula — «Stroke width (px)» seguia pela coluna da
    /// caixa «Dashed» e pintava por cima da marca do campo. *Mutação: a marca descontada no fim do troço.*
    #[test]
    fn a_label_stops_before_the_key_mark_of_its_cell() {
        use super::super::Widget;
        let a = Cell::new(Widget::Readout { text: "" }).label("A").key(&[]);
        let b = Cell::new(Widget::Readout { text: "" });
        let cols = [(0.0, 40.0), (46.0, 40.0)];
        assert_eq!(largura_do_rotulo(&[a, b], &cols, 0), 40.0 - m::KEY_MARK_BOX);
    }

    /// ⭐ Só o eixo faz o par: o nome igual menos uma palavra, e ela é o eixo.
    /// *Mutações: comparar só a primeira palavra · aceitar duas palavras diferentes.*
    #[test]
    fn an_axis_pair_differs_only_in_the_axis() {
        use super::super::{Glyph, Widget};
        let par = |a: &'static str, b: &'static str| {
            let c = |l| {
                Cell::new(Widget::Field {
                    id: ph2d_a11y::NodeId(1),
                    glyph: Glyph::Text("X"),
                })
                .label(l)
            };
            par_de_eixos(&[c(a), c(b)])
        };
        for (a, b) in [
            ("Width (tiles)", "Height (tiles)"),
            ("Reference Width (px)", "Reference Height (px)"),
            ("Offset X (px)", "Offset Y (px)"),
        ] {
            assert!(par(a, b), "{a} × {b}");
        }
        for (a, b) in [
            ("Width", "Hardness"),
            ("Offset X", "Size Y"),
            ("Min", "Max"),
            ("X", "Y extra"),
            ("Height", "Width"),
        ] {
            assert!(!par(a, b), "{a} × {b}");
        }
    }
}
