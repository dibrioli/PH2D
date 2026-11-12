//! ⭐ **O HALO de um overlay do canvas** (dono 12/11, GEF 04: *«coloque sombra nas fontes, pontos e
//! linhas, pois em fundos claros não posso ver»*): a lei da sombra dos contornos (GEF02) — `BgScrim`
//! (escura em todos os temas) a [`ph2d_tokens::OUTLINE_SHADOW_ALPHA`] vezes o alfa da tinta, `Thin`
//! mais larga de cada lado, o MESMO tracejado — dada a linhas, a formas cheias e ao texto. O halo
//! sai SEMPRE antes da tinta, pelas portas que gravam (`stroke_path`/`fill_shape`).

use crate::zones::Rect;
use ph2d_text::TextSystem;
use ph2d_tokens::{StrokeToken, Theme};
use ph2d_vector::{Affine, Brush, Color, Fill, Shape, Stroke, VectorScene};

/// O ouvinte do texto pintado, para um teste de outra crate ler o que o seu pintor escreveu com halo
/// (o texto vai por `inner_mut`, fora da gravação da cena).
pub use crate::paint::text::ouve_o_texto;

/// A cor do halo sob uma tinta de cor `cor` (o alfa da tinta passa ao halo: um fantasma tênue tem
/// um halo tênue).
#[must_use]
pub fn cor_do_halo(cor: Color, theme: Theme) -> Color {
    let (sombra, _) = crate::paint_flight::sombra_de_alca(theme);
    sombra.multiply_alpha(cor.components[3])
}

/// O alargamento do halo de cada lado, px de ecrã.
#[must_use]
pub fn largura_do_halo() -> f64 {
    f64::from(StrokeToken::Thin.px())
}

/// O traço do halo sob `traco`: `Thin` mais largo de cada lado, o mesmo tracejado.
#[must_use]
pub fn traco_do_halo(traco: &Stroke) -> Stroke {
    let mut s = traco.clone();
    s.width = traco.width + 2.0 * largura_do_halo();
    s
}

/// Contorna `shape` com `traco` em `cor`, sobre o halo dele.
pub fn stroke_com_halo(
    scene: &mut VectorScene,
    traco: &Stroke,
    cor: Color,
    shape: &impl Shape,
    theme: Theme,
) {
    let halo = Brush::Solid(cor_do_halo(cor, theme));
    scene.stroke_path(&traco_do_halo(traco), Affine::IDENTITY, &halo, shape);
    scene.stroke_path(traco, Affine::IDENTITY, &Brush::Solid(cor), shape);
}

/// Preenche `shape` em `cor`, sobre um anel de halo `Thin` de cada lado da borda.
pub fn fill_com_halo(scene: &mut VectorScene, cor: Color, shape: &impl Shape, theme: Theme) {
    let halo = Brush::Solid(cor_do_halo(cor, theme));
    let anel = Stroke::new(2.0 * largura_do_halo());
    scene.stroke_path(&anel, Affine::IDENTITY, &halo, shape);
    scene.fill_shape(Fill::NonZero, Affine::IDENTITY, &Brush::Solid(cor), shape);
}

/// As oito direcções do halo do texto (o texto repete-se deslocado `Thin` em cada uma, na cor do halo).
pub const DIRECOES_DO_HALO: [(f32, f32); 8] = [
    (-1.0, -1.0),
    (0.0, -1.0),
    (1.0, -1.0),
    (-1.0, 0.0),
    (1.0, 0.0),
    (-1.0, 1.0),
    (0.0, 1.0),
    (1.0, 1.0),
];

/// [`crate::paint::paint_text_centered`] sobre um halo: o texto na cor do halo nas oito direcções,
/// depois o texto em `cor`.
#[allow(clippy::too_many_arguments)]
pub fn paint_text_centered_com_halo(
    text_system: &mut TextSystem,
    scene: &mut VectorScene,
    text: &str,
    rect: Rect,
    font_size: f32,
    cor: Color,
    theme: Theme,
) {
    let halo = cor_do_halo(cor, theme);
    let g = largura_do_halo() as f32;
    for (dx, dy) in DIRECOES_DO_HALO {
        let r = Rect::new(rect.x + dx * g, rect.y + dy * g, rect.w, rect.h);
        crate::paint::paint_text_centered(text_system, scene, text, r, font_size, halo);
    }
    crate::paint::paint_text_centered(text_system, scene, text, rect, font_size, cor);
}

#[cfg(test)]
#[path = "paint_halo_tests.rs"]
mod tests;
