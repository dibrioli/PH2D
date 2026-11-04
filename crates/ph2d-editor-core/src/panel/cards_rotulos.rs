//! A faixa dos rótulos de uma linha de cartão. Um rótulo que não cabe na coluna QUEBRA em duas
//! linhas («Opacity» / «(%)», maquete F) em vez de virar «Opacity…»; a faixa cresce uma linha só
//! quando algum rótulo da linha precisa, e os de uma linha centram-se nela.

use super::{CardCtx, Cell};
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

pub(super) fn reserva(c: &Cell<'_>) -> f32 {
    if c.key.is_some() {
        m::KEY_MARK_BOX
    } else {
        0.0
    }
}

/// Pinta os rótulos (e as marcas de animação) de uma linha a partir de `y`; devolve a altura da
/// faixa, já com o `LABEL_BOTTOM`.
pub(super) fn faixa(cx: &mut CardCtx<'_>, cells: &[Cell<'_>], cols: &[(f32, f32)], y: f32) -> f32 {
    let quebras: Vec<_> = cells
        .iter()
        .zip(cols)
        .map(|(c, r)| c.label.map(|l| linhas(cx.text_system, l, r.1 - reserva(c))))
        .collect();
    let duas = quebras.iter().flatten().any(|(_, b)| b.is_some());
    let extra = if duas { PASSO } else { 0.0 };
    for ((c, r), q) in cells.iter().zip(cols).zip(&quebras) {
        let w = r.1 - reserva(c);
        match q {
            Some((a, Some(b))) => {
                for (t, ty) in [(a, y), (b, y + PASSO)] {
                    card_field::paint_label(cx.scene, cx.text_system, cx.ink, r.0, ty, w, t);
                }
            }
            Some((a, None)) => {
                let ty = y + extra * 0.5;
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
