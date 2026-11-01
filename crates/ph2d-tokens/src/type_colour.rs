//! **A cor do TIPO de objecto num tema moderno** (`obj-*`, F42) — a emprestada da família clássica,
//! empurrada em luminosidade só quando não se lê sobre os fundos DESTE tema.
//!
//! Medido na F42: emprestadas tal-e-qual, os temas `PlumberRed`/`PlumberGreen` (fundo saturado)
//! davam `2,09`/`1,94` contra o fundo, abaixo até do `text-3` que o ícone cinzento usava. O alvo é o
//! da WCAG 2.2 SC 1.4.11 (gráfico, não texto): 3:1. Onde a emprestada já passa, sai idêntica.

use crate::color::{Color, oklch_to_srgb, srgb_to_oklch};

/// O contraste mínimo de um ícone de tipo sobre os fundos (WCAG 2.2 SC 1.4.11).
pub const TYPE_ICON_MIN_CONTRAST: f64 = 3.0;

/// `borrowed` legível sobre todos os `grounds`: anda em L (para cima num tema escuro, para baixo
/// num claro) até `TYPE_ICON_MIN_CONTRAST`, mantendo croma e matiz.
#[must_use]
pub(crate) fn legible_on(borrowed: Color, grounds: &[Color], dark: bool) -> Color {
    let worst = |c: &Color| {
        grounds
            .iter()
            .map(|g| g.contrast_ratio(c))
            .fold(f64::MAX, f64::min)
    };
    if worst(&borrowed) >= TYPE_ICON_MIN_CONTRAST {
        return borrowed;
    }
    let (l0, ch, h) = srgb_to_oklch(borrowed.r, borrowed.g, borrowed.b);
    let step = if dark { 0.01 } else { -0.01 };
    let mut best = borrowed;
    let mut l = l0;
    while (0.0..=1.0).contains(&(l + step)) {
        l += step;
        let [r, g, b] = oklch_to_srgb(l, ch, h);
        let c = Color {
            r,
            g,
            b,
            a: borrowed.a,
        };
        if worst(&c) > worst(&best) {
            best = c;
        }
        if worst(&c) >= TYPE_ICON_MIN_CONTRAST {
            return c;
        }
    }
    best
}
