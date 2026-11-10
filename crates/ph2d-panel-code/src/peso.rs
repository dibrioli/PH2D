//! O peso de cada papel de texto da aba, o da maquete 14 (`docs/CodeEditor/maquetes/
//! 14_quadros_do_depurador.html`): o cartão é `400`, os títulos e o estado `600`, os botões, as
//! teclas e os cabeçalhos `500`. Os três pintores daqui têm a assinatura dos da casa e pintam o
//! CORPO; quem pinta outro papel chama a porta com peso. O peso é o FINAL do CSS, sem o reforço do
//! preset de nitidez ([`ph2d_text::com_peso_exato`]): toda medida da aba vai por [`largura_em`].

use ph2d_editor_core::paint::{
    paint_text_block_in_weight, paint_text_elided_in_weight, paint_text_in_weight,
};
use ph2d_text::{FontWeight, TextSystem};
use ph2d_vector::{Color, VectorScene};

/// O corpo do cartão: listas, linhas, valores, a frase e o rodapé (`.card`, `400`).
pub const CORPO: FontWeight = FontWeight::NORMAL;
/// Os títulos, o estado, os grupos da tabela (`h5`, `.status`, `tr.grp`, `600`).
pub const TITULO: FontWeight = FontWeight::SEMI_BOLD;
/// Os botões, as teclas e os cabeçalhos das colunas (`.btn`, `kbd`, `th`, `h4`, `500`).
pub const BOTAO: FontWeight = FontWeight::MEDIUM;

/// O [`ph2d_editor_core::paint::paint_text`] no peso do corpo.
#[allow(clippy::too_many_arguments)]
#[track_caller]
pub(crate) fn paint_text(
    ts: &mut TextSystem,
    scene: &mut VectorScene,
    text: &str,
    x: f32,
    y: f32,
    font_size: f32,
    max_width: f32,
    color: Color,
) {
    paint_text_in_weight(ts, scene, text, [x, y], font_size, max_width, color, CORPO);
}

/// O [`ph2d_editor_core::paint::paint_text_block`] no peso do corpo.
#[allow(clippy::too_many_arguments)]
#[track_caller]
pub(crate) fn paint_text_block(
    ts: &mut TextSystem,
    scene: &mut VectorScene,
    text: &str,
    x: f32,
    y: f32,
    font_size: f32,
    max_width: f32,
    color: Color,
) -> f32 {
    paint_text_block_in_weight(ts, scene, text, [x, y], font_size, max_width, color, CORPO)
}

/// O [`ph2d_editor_core::paint::paint_text_elided`] no peso do corpo.
#[allow(clippy::too_many_arguments)]
#[track_caller]
pub(crate) fn paint_text_elided(
    ts: &mut TextSystem,
    scene: &mut VectorScene,
    text: &str,
    x: f32,
    y: f32,
    font_size: f32,
    max_width: f32,
    color: Color,
) {
    paint_text_elided_in_weight(ts, scene, text, [x, y], font_size, max_width, color, CORPO);
}

/// O [`ph2d_editor_core::paint::label_column_width`] no peso do corpo.
pub(crate) fn label_column_width<'a>(
    ts: &mut TextSystem,
    font_size: f32,
    rotulos: impl IntoIterator<Item = &'a str>,
) -> f32 {
    rotulos
        .into_iter()
        .fold(0.0_f32, |w, r| w.max(largura_em(ts, r, font_size, CORPO)))
}

/// A largura de `text` no peso do corpo (a `prefix_width` da casa mede em `500`).
pub(crate) fn largura(ts: &mut TextSystem, text: &str, font_size: f32) -> f32 {
    largura_em(ts, text, font_size, CORPO)
}

/// A largura de `text` em `weight`, no peso exacto em que os pintores daqui o pintam.
pub(crate) fn largura_em(
    ts: &mut TextSystem,
    text: &str,
    font_size: f32,
    weight: FontWeight,
) -> f32 {
    ph2d_text::com_peso_exato(|| ts.prefix_width_weighted(text, font_size, weight))
}

/// O [`ph2d_editor_core::paint::paint_text_title`] no peso exacto do título.
#[allow(clippy::too_many_arguments)]
#[track_caller]
pub(crate) fn paint_text_title(
    ts: &mut TextSystem,
    scene: &mut VectorScene,
    text: &str,
    x: f32,
    y: f32,
    font_size: f32,
    max_width: f32,
    color: Color,
) {
    paint_text_in_weight(ts, scene, text, [x, y], font_size, max_width, color, TITULO);
}

/// O [`ph2d_editor_core::paint::paint_text_title_elided`] no peso exacto do título.
#[allow(clippy::too_many_arguments)]
#[track_caller]
pub(crate) fn paint_text_title_elided(
    ts: &mut TextSystem,
    scene: &mut VectorScene,
    text: &str,
    x: f32,
    y: f32,
    font_size: f32,
    max_width: f32,
    color: Color,
) {
    paint_text_elided_in_weight(ts, scene, text, [x, y], font_size, max_width, color, TITULO);
}

#[cfg(test)]
mod tests {
    use super::*;
    use ph2d_editor_core::zones::Rect;

    type Peca = fn(&mut TextSystem, &mut VectorScene, &str, f32);

    /// Os eixos variáveis (onde o peso viaja) que um pintor pôs na cena.
    fn eixos(pinta: impl Fn(&mut TextSystem, &mut VectorScene)) -> Vec<i16> {
        let (mut ts, mut scene) = (TextSystem::new(), VectorScene::new());
        pinta(&mut ts, &mut scene);
        assert!(
            !scene.inner().encoding().resources.glyph_runs.is_empty(),
            "nenhum glifo"
        );
        let r = &scene.inner().encoding().resources.normalized_coords;
        r.clone()
    }

    #[test]
    fn cada_papel_da_aba_pinta_no_peso_da_maquete() {
        let ink = Color::from_rgba8(255, 255, 255, 255);
        let pecas: [(&str, Peca, FontWeight); 4] = [
            (
                "linha",
                |ts, sc, t, f| paint_text(ts, sc, t, 0.0, 0.0, f, f32::INFINITY, ink_of()),
                CORPO,
            ),
            (
                "bloco",
                |ts, sc, t, f| {
                    paint_text_block(ts, sc, t, 0.0, 0.0, f, f32::INFINITY, ink_of());
                },
                CORPO,
            ),
            (
                "cortado",
                |ts, sc, t, f| {
                    paint_text_elided(ts, sc, t, 0.0, 0.0, f, f32::INFINITY, ink_of());
                },
                CORPO,
            ),
            (
                "rotulo",
                |ts, sc, t, f| {
                    let r = Rect::new(0.0, 0.0, f32::INFINITY, f * 2.0);
                    crate::chrome::label_in(sc, ts, t, r, ink_of());
                },
                CORPO,
            ),
        ];
        let f = crate::chrome::font();
        for (nome, pinta, peso) in pecas {
            let t = "Scripts update speed";
            let feito = eixos(|ts, sc| pinta(ts, sc, t, f));
            let esperado = eixos(|ts, sc| {
                paint_text_in_weight(ts, sc, t, [0.0, 0.0], f, f32::INFINITY, ink, peso)
            });
            let medio = eixos(|ts, sc| {
                paint_text_in_weight(ts, sc, t, [0.0, 0.0], f, f32::INFINITY, ink, BOTAO)
            });
            assert_ne!(esperado, medio, "a sonda não separa os pesos");
            assert_eq!(feito, esperado, "{nome}: não pintou no peso da maquete");
        }
    }

    fn ink_of() -> Color {
        Color::from_rgba8(255, 255, 255, 255)
    }
}
