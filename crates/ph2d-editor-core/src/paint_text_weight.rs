//! Os pintores de texto com o PESO explícito: para uma superfície cujo desenho pede outro peso de
//! corpo que o `Medium` da casa (a aba de scripts, maquete 14: corpo 400). Medem e pintam no mesmo
//! peso, como as irmãs; o padrão de [`super::paint_text`] não muda. O peso é o FINAL, sem o reforço
//! do preset de nitidez ([`ph2d_text::com_peso_exato`]); quem mede para elas mede no mesmo âmbito.

use super::{Lines, paint_elided_weighted, paint_text_lines};
use ph2d_text::{FontWeight, TextSystem};
use ph2d_vector::{Color, VectorScene};

/// [`super::paint_text`] em `weight`: uma linha, cortada no mesmo peso.
#[allow(clippy::too_many_arguments)]
#[track_caller]
pub fn paint_text_in_weight(
    ts: &mut TextSystem,
    scene: &mut VectorScene,
    text: &str,
    [x, y]: [f32; 2],
    font_size: f32,
    max_width: f32,
    color: Color,
    weight: FontWeight,
) {
    let one = Lines::ElideToOne;
    ph2d_text::com_peso_exato(|| {
        paint_text_lines(
            ts, scene, text, x, y, font_size, max_width, color, weight, one,
        );
    });
}

/// [`super::paint_text_block`] em `weight`: quebra e devolve a altura.
#[allow(clippy::too_many_arguments)]
#[track_caller]
pub fn paint_text_block_in_weight(
    ts: &mut TextSystem,
    scene: &mut VectorScene,
    text: &str,
    [x, y]: [f32; 2],
    font_size: f32,
    max_width: f32,
    color: Color,
    weight: FontWeight,
) -> f32 {
    let text = super::cola_o_separador(text);
    let wrap = Lines::Wrap;
    ph2d_text::com_peso_exato(|| {
        paint_text_lines(
            ts, scene, &text, x, y, font_size, max_width, color, weight, wrap,
        )
    })
}

/// [`super::paint_text_elided`] em `weight`.
#[allow(clippy::too_many_arguments)]
#[track_caller]
pub fn paint_text_elided_in_weight(
    ts: &mut TextSystem,
    scene: &mut VectorScene,
    text: &str,
    [x, y]: [f32; 2],
    font_size: f32,
    max_width: f32,
    color: Color,
    weight: FontWeight,
) {
    ph2d_text::com_peso_exato(|| {
        paint_elided_weighted(
            ts, scene, text, x, y, font_size, max_width, color, weight, 0.0,
        );
    });
}
