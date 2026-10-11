//! ⭐ **A nota escondida** (MiroClone W12, ADR-0182 §4): o olho fechado no canto de cima à esquerda de
//! toda nota com texto por revelar, e — a quem NÃO a escreveu — os rabiscos no lugar do texto. Os
//! rabiscos são FIXOS: o Miro desenha uma fila por linha do texto (vaza o comprimento), e quem não é o
//! autor nem tem o texto para os medir.
//!
//! Medidas na figura do help do Miro (`facilitacao_miro_notas.md` §6, pm_04: nota de 392 px): o selo
//! do olho com 49 px a 12 px do canto; os rabiscos com 130 × 20 px, ao centro.

use std::sync::OnceLock;

use ph2d_board_model::Element;
use ph2d_vector::{
    Affine, BezPath, Brush, Cap, Color, Join, Rect, RoundedRect, Shape as _, Stroke, VectorScene,
};

/// O lado do selo do olho, em fracção do lado da nota (49 / 392).
pub const EYE_BOX: f64 = 0.125;
/// A distância do selo ao canto, em fracção do lado (12 / 392).
pub const EYE_INSET: f64 = 0.03;
/// Os rabiscos: largura e altura em fracção do lado (130 / 392 · 20 / 392).
pub const SQUIGGLE: [f64; 2] = [0.33, 0.05];
/// O traço dos rabiscos, em fracção da altura deles (a figura: ~4 px em 20).
const SQUIGGLE_STROKE: f64 = 0.2;
/// As voltas dos rabiscos (as da figura: «uuuuu»).
const SQUIGGLE_HUMPS: u32 = 5;
/// A opacidade do fundo do selo e do ícone (o selo é a tinta do texto, levíssima).
const EYE_BOX_ALPHA: f32 = 0.14;
const EYE_ICON_ALPHA: f32 = 0.6;
/// O traço dos ícones Lucide no quadrado de 24.
const LUCIDE_STROKE: f64 = 2.0;
const LUCIDE_BOX: f64 = 24.0;
/// O selo: o raio dos cantos e o ícone dentro dele, em fracções do lado do selo (o ícone a 70 %, ao
/// centro — a figura pm_04).
const EYE_ROUND: f64 = 0.2;
const EYE_ICON: f64 = 0.7;
/// A tolerância de achatar o canto redondo em curvas (unidades do mundo).
const FLATTEN: f64 = 0.1;

/// ⭐ O ícone **Lucide** `eye-closed` (ISC, `docs/design/icons/eye-closed.svg`), num quadrado de 24.
pub(crate) const EYE_CLOSED_SVG: [&str; 5] = [
    "m15 18-.722-3.25",
    "M2 8a10.645 10.645 0 0 0 20 0",
    "m20 15-1.726-2.05",
    "m4 15 1.726-2.05",
    "m9 18 .722-3.25",
];

pub(crate) fn eye_path() -> &'static BezPath {
    static P: OnceLock<BezPath> = OnceLock::new();
    P.get_or_init(|| {
        let mut out = BezPath::new();
        for d in EYE_CLOSED_SVG {
            if let Ok(p) = BezPath::from_svg(d) {
                out.extend(p);
            }
        }
        out
    })
}

/// Os rabiscos em `w × h` a partir de `(x, y)` (voltas de meia onda, como um «u» cursivo).
fn squiggle_path(x: f64, y: f64, w: f64, h: f64) -> BezPath {
    let mut p = BezPath::new();
    let n = f64::from(SQUIGGLE_HUMPS);
    p.move_to((x, y));
    for i in 0..SQUIGGLE_HUMPS {
        let x0 = x + w * f64::from(i) / n;
        let x1 = x + w * f64::from(i + 1) / n;
        let mid = (x0 + x1) / 2.0;
        p.curve_to((x0, y + h), (mid, y + h), (mid, y + h / 2.0));
        p.curve_to((mid, y), (x1, y), (x1, y));
    }
    p
}

/// ⭐ O olho (sempre) e os rabiscos (só a quem não lê o texto) sobre a nota `el`, no espaço `t` da
/// nota (o canto de cima à esquerda é a origem, como o texto).
pub(crate) fn paint(scene: &mut VectorScene, el: &Element, t: Affine, unread: bool, ink: Color) {
    let side = el.w.min(el.h);
    let b = side * EYE_BOX;
    let at = side * EYE_INSET;
    let boxr = RoundedRect::from_rect(Rect::new(at, at, at + b, at + b), b * EYE_ROUND);
    scene.fill_path(
        &boxr.to_path(FLATTEN),
        &Brush::Solid(ink.with_alpha(EYE_BOX_ALPHA)),
        t,
    );
    let k = b * EYE_ICON / LUCIDE_BOX;
    let inset = at + b * (1.0 - EYE_ICON) / 2.0;
    let icon = t * Affine::translate((inset, inset)) * Affine::scale(k);
    let s = Stroke::new(LUCIDE_STROKE)
        .with_caps(Cap::Round)
        .with_join(Join::Round);
    scene.stroke_path(
        &s,
        icon,
        &Brush::Solid(ink.with_alpha(EYE_ICON_ALPHA)),
        eye_path(),
    );
    if unread {
        let [w, h] = [el.w * SQUIGGLE[0], side * SQUIGGLE[1]];
        let p = squiggle_path((el.w - w) / 2.0, (el.h - h) / 2.0, w, h);
        let s = Stroke::new(h * SQUIGGLE_STROKE).with_caps(Cap::Round);
        scene.stroke_path(&s, t, &Brush::Solid(ink), &p);
    }
}

#[cfg(test)]
#[path = "private_tests.rs"]
mod tests;
