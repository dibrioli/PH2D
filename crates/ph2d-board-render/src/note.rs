//! Por baixo de uma nota: a sombra e, numa pilha, as folhas de baixo (W3; módulo à parte desde a W5,
//! o desenho estava no tecto de LOC).

use ph2d_board_model::{Element, Rgba, Shape, ShapeType};
use ph2d_vector::{Affine, Brush, Color, Rect, Shape as _, VectorScene};

/// A sombra de uma nota (a do Miro: a nota «descola» do quadro; a altura `228` da documentação dele
/// inclui-a, `29` abaixo dos `199`), em fracções do lado. ⚠️ Forma por medir numa captura.
const NOTE_SHADOW_DROP: f64 = 0.05;
const NOTE_SHADOW_BLUR: f64 = 0.04;
/// Quanto cada folha de baixo de uma PILHA espreita (fracção do lado), e quantas há.
const STACK_STEP: f64 = 0.035;
const STACK_SHEETS: u8 = 2;

/// Por baixo de uma nota: a sombra; numa pilha, também as folhas de baixo, a espreitar.
pub(crate) fn paint_under(scene: &mut VectorScene, el: &Element, shape: &Shape, t: Affine) {
    let side = el.w.min(el.h);
    let sheets = if shape.kind == ShapeType::StickyStack {
        STACK_SHEETS
    } else {
        0
    };
    let step = side * STACK_STEP;
    let reach = step * f64::from(sheets);
    // LITERAL-COLOR-OK: a sombra é aspecto do DOCUMENTO (a mesma em qualquer tema, como a do Miro).
    let shadow = Color::from_rgba8(0, 0, 0, 56);
    let drop = side * NOTE_SHADOW_DROP;
    let body = Rect::new(
        side * 0.02,
        drop,
        el.w - side * 0.02,
        el.h + reach + drop * 0.4,
    );
    scene.blurred_rounded_rect(t, body, shadow, 0.0, side * NOTE_SHADOW_BLUR);
    let Some(Rgba([r, g, b, a])) = shape.style.fill else {
        return;
    };
    for k in (1..=sheets).rev() {
        let d = step * f64::from(k);
        // Cada folha de baixo um pouco mais escura: lê-se como papel empilhado.
        let shade = |c: u8| (f64::from(c) * (1.0 - 0.06 * f64::from(k))) as u8;
        let c = Color::from_rgba8(shade(r), shade(g), shade(b), a); // LITERAL-COLOR-OK: cor do documento
        let sheet = Rect::new(d, d, el.w + d, el.h + d).to_path(0.1);
        scene.fill_path(&sheet, &Brush::Solid(c), t);
    }
}
