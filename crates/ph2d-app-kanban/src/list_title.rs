//! ⭐ **O título de uma lista** — em MAIÚSCULAS só na pintura: o texto guardado é o que o artista
//! escreveu (editar no lugar mostra-o assim). Medir e pintar passam por AQUI, com a mesma face —
//! o texto mostrado, o tamanho e o peso —, para a elisão e a altura da recolhida nunca divergirem
//! do que se vê (já houve medir `Medium` e pintar `SemiBold`).

use crate::metrics as m;
use ph2d_editor_core::paint::paint_text_title_elided;
use ph2d_text::{FontWeight, TextSystem};
use ph2d_vector::{Color, VectorScene};

/// O tamanho do título.
pub(crate) const PX: f32 = m::LIST_TITLE_PX;
/// O peso: o do [`paint_text_title_elided`], o único pintor que [`paint`] chama.
pub(crate) const WEIGHT: FontWeight = FontWeight::SEMI_BOLD;

/// O que se pinta de `title`.
pub(crate) fn shown(title: &str) -> String {
    title.to_uppercase()
}

/// A largura de `title` numa linha, como se pinta (arredondada para cima: a reticência nunca nasce
/// de uma fracção).
pub(crate) fn width(ts: &mut TextSystem, title: &str) -> f32 {
    ts.prefix_width_weighted(&shown(title), PX, WEIGHT).ceil()
}

/// Pinta `title` em `(x, y)`, cortado a `max_w`. Devolve o texto mostrado.
#[allow(clippy::too_many_arguments)]
pub(crate) fn paint(
    ts: &mut TextSystem,
    scene: &mut VectorScene,
    title: &str,
    x: f32,
    y: f32,
    max_w: f32,
    fg: Color,
) -> String {
    let s = shown(title);
    paint_text_title_elided(ts, scene, &s, x, y, PX, max_w, fg);
    s
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_stored_text_is_shown_in_capitals_and_measured_so() {
        let mut ts = TextSystem::without_system_fonts();
        assert_eq!(shown("Short film — doing"), "SHORT FILM — DOING");
        let caps = ts.prefix_width_weighted("BACKLOG", PX, WEIGHT).ceil();
        assert!((width(&mut ts, "Backlog") - caps).abs() < f32::EPSILON);
    }
}
