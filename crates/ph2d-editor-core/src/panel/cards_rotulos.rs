//! A faixa dos rótulos de uma linha de cartão. Um rótulo que não cabe na coluna QUEBRA em duas
//! linhas («Opacity» / «(%)», maquete F) em vez de virar «Opacity…»; a faixa cresce uma linha só
//! quando algum rótulo da linha precisa, e os de uma linha centram-se nela.

use super::{CardCtx, Cell, Widget};
use crate::widget::card_field;
use ph2d_card_stack::metrics as m;
use ph2d_text::TextSystem;

/// O passo entre as duas linhas de um rótulo quebrado.
const PASSO: f32 = m::LABEL_PX + m::LABEL_BOTTOM;

/// As (no máximo) duas linhas de `l` na largura `w`: quebra na última palavra que cabe; a 2.ª
/// linha leva o resto (e o pintor elide-a, se ainda não couber).
pub(super) fn linhas<'l>(ts: &mut TextSystem, l: &'l str, w: f32) -> (&'l str, Option<&'l str>) {
    if ts.prefix_width(l, m::LABEL_PX) <= w {
        return (l, None);
    }
    let corte = l
        .match_indices(' ')
        .map(|(i, _)| i)
        .filter(|&i| ts.prefix_width(&l[..i], m::LABEL_PX) <= w)
        .last();
    match corte {
        Some(i) => (&l[..i], Some(l[i + 1..].trim_start())),
        None => (l, None),
    }
}

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
/// quatro amostras, que numa coluna só quebrava), menos a marca que mora nesse troço.
fn largura_do_rotulo(cells: &[Cell<'_>], cols: &[(f32, f32)], i: usize) -> f32 {
    let fim = (i + 1..cells.len())
        .find(|&j| cells[j].label.is_some())
        .unwrap_or(cells.len());
    let marca = cells[i..fim].iter().map(reserva).fold(0.0, f32::max);
    let (x0, (xf, wf)) = (cols[i].0, cols[fim - 1]);
    xf + wf - x0 - marca
}

/// Pinta os rótulos (e as marcas de animação) de uma linha a partir de `y`; devolve a altura da
/// faixa, já com o `LABEL_BOTTOM`.
pub(super) fn faixa(cx: &mut CardCtx<'_>, cells: &[Cell<'_>], cols: &[(f32, f32)], y: f32) -> f32 {
    let de_eixos = par_de_eixos(cells);
    let larguras: Vec<f32> = (0..cells.len())
        .map(|i| largura_do_rotulo(cells, cols, i))
        .collect();
    let quebras: Vec<_> = cells
        .iter()
        .zip(&larguras)
        .map(|(c, w)| {
            c.label.map(|l| {
                // No par de eixos a unidade sai antes de o nome quebrar; a dica do campo continua a dizê-la.
                let cabe = cx.text_system.prefix_width(l, m::LABEL_PX) <= *w;
                let l = if de_eixos && !cabe { sem_unidade(l) } else { l };
                linhas(cx.text_system, l, *w)
            })
        })
        .collect();
    let duas = quebras.iter().flatten().any(|(_, b)| b.is_some());
    let extra = if duas { PASSO } else { 0.0 };
    for (i, ((c, r), q)) in cells.iter().zip(cols).zip(&quebras).enumerate() {
        let w = larguras[i];
        match q {
            Some((a, Some(b))) => {
                for (t, ty) in [(a, y), (b, y + PASSO)] {
                    card_field::paint_label(cx.scene, cx.text_system, cx.ink, r.0, ty, w, t);
                }
            }
            Some((a, None)) => {
                let ty = y + extra;
                card_field::paint_label(cx.scene, cx.text_system, cx.ink, r.0, ty, w, a);
            }
            None => {}
        }
        if let Some(k) = c.key {
            card_field::paint_key_mark(
                cx.scene,
                cx.ink,
                cx.hit_index,
                r.0 + r.1 - m::KEY_MARK_BOX * 0.5,
                y + (m::LABEL_PX + extra) * 0.5,
                k,
            );
        }
    }
    extra + m::LABEL_PX + m::LABEL_BOTTOM
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

    /// ⭐ Maquete F: «Opacity (%)» numa coluna estreita quebra em «Opacity» / «(%)», inteiros.
    #[test]
    fn a_label_that_does_not_fit_wraps_onto_two_lines() {
        let mut ts = TextSystem::without_system_fonts();
        let l = "Opacity (%)";
        let w = ts.prefix_width("Opacity", m::LABEL_PX) + 1.0;
        assert_eq!(linhas(&mut ts, l, w), ("Opacity", Some("(%)")));
        let largo = ts.prefix_width(l, m::LABEL_PX) + 1.0;
        assert_eq!(linhas(&mut ts, l, largo), (l, None));
    }
}
