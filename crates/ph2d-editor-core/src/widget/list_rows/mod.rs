//! ⭐⭐⭐ **AS LISTRAS DE UMA LISTA** — o tom alternado que separa duas linhas encostadas.
//!
//! Enio, 2026-09-06, com a foto do *Outliner* do Blender: *«linhas pares e ímpares têm tonalidade
//! discretamente diferente».*
//!
//! ⚠️ **A listra é da LISTA, não da LINHA, e a separação é o desenho inteiro deste módulo.** Uma
//! linha não sabe onde está: o que ela sabe dizer sobre si é o seu ESTADO (apontada, seleccionada,
//! silenciada). A alternância é uma propriedade da **sequência**, e só quem itera a conhece — daí
//! a porta viver aqui e ser chamada pelo laço, com o estado a ser pintado por cima depois.
//!
//! ⛔ **E o índice é o VISUAL, nunca o do dado.** Uma hierarquia salta linhas (um ramo recolhido,
//! um filtro de busca activo), e contar pelo índice do modelo faria duas linhas do mesmo tom ficar
//! encostadas exactamente quando o artista fecha um ramo — o defeito seria intermitente e
//! reportado como *«às vezes as listras somem»*. A porta recebe *quantas já foram pintadas*.

pub mod selection;
pub use selection::{RowHighlight, paint_row_highlight};

use crate::zones::Rect;
use ph2d_tokens::{ColorToken, Theme};
use ph2d_vector::VectorScene;

/// **Pinta a listra da linha de índice visual `painted_before`**, se ela for ímpar.
///
/// `base` é o token da superfície em que a lista assenta — o fundo do painel para uma lista solta,
/// o tom do cartão para uma que viva dentro de uma secção. ⚠️ É um argumento e não uma constante
/// porque as duas coisas existem no app, e adivinhar uma delas pintaria a listra do tom errado
/// exactamente onde ela mais se nota.
///
/// A ordem é **listra → estado**: o realce de hover e o de selecção são pintados a seguir, por
/// cima, e cobrem-na. É a mesma ordem do Blender e a única que mantém *seleccionado* a ler-se
/// igual nas linhas pares e ímpares.
pub fn paint_row_stripe(
    scene: &mut VectorScene,
    rect: Rect,
    theme: Theme,
    base: ColorToken,
    painted_before: usize,
) {
    if painted_before.is_multiple_of(2) {
        return;
    }
    let tint = ph2d_tokens::faint_row_bg(base.resolve(theme));
    crate::paint::fill_rounded_rect(
        scene,
        rect,
        0.0,
        ph2d_vector::Color::from_rgba8(tint.r, tint.g, tint.b, tint.a), // LITERAL-COLOR-OK: token-bridge
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A linha PAR não pinta nada — senão a lista inteira mudava de tom em vez de alternar.
    #[test]
    fn an_even_row_paints_nothing_and_an_odd_one_paints() {
        let mut even = VectorScene::new();
        paint_row_stripe(
            &mut even,
            Rect::new(0.0, 0.0, 100.0, 22.0),
            Theme::Dark,
            ColorToken::PanelBg,
            0,
        );
        let mut odd = VectorScene::new();
        paint_row_stripe(
            &mut odd,
            Rect::new(0.0, 0.0, 100.0, 22.0),
            Theme::Dark,
            ColorToken::PanelBg,
            1,
        );
        assert_eq!(
            even.inner().encoding().n_path_segments,
            0,
            "a linha PAR pintou geometria — a lista muda de tom em vez de alternar"
        );
        assert!(
            odd.inner().encoding().n_path_segments > 0,
            "a linha IMPAR nao pintou nada — nao ha' listra nenhuma"
        );
    }

    /// ⭐ **A listra move-se para LONGE do extremo do fundo**, e por isso serve os oito temas.
    #[test]
    fn the_stripe_lightens_a_dark_theme_and_darkens_a_light_one() {
        for theme in [Theme::Dark, Theme::Oled, Theme::Forge] {
            let bg = ColorToken::PanelBg.resolve(theme);
            let stripe = ph2d_tokens::faint_row_bg(bg);
            assert!(
                stripe.relative_luminance() > bg.relative_luminance(),
                "{theme:?}: a listra nao clareou um fundo escuro ({bg:?} -> {stripe:?})"
            );
        }
        for theme in [Theme::Light, Theme::Sunstone] {
            let bg = ColorToken::PanelBg.resolve(theme);
            let stripe = ph2d_tokens::faint_row_bg(bg);
            assert!(
                stripe.relative_luminance() < bg.relative_luminance(),
                "{theme:?}: a listra nao escureceu um fundo claro ({bg:?} -> {stripe:?})"
            );
        }
    }

    /// ⛔ **Ela é DISCRETA — muito menor que o degrau da escada de cartões.**
    ///
    /// O dono disse *«discretamente diferente»*, e a régua desse *discretamente* é a escada que já
    /// existe: entre um cartão e o painel vão 12 em 255. Uma listra desse tamanho leria como
    /// *«esta linha está dentro daquela»*, que é a confusão que a palavra dele exclui.
    #[test]
    fn the_stripe_is_far_smaller_than_a_card_step() {
        let theme = Theme::Dark;
        let panel = ColorToken::PanelBg.resolve(theme);
        let card = ColorToken::Bg1.resolve(theme);
        let stripe = ph2d_tokens::faint_row_bg(panel);
        let d_stripe = (stripe.r as i16 - panel.r as i16).abs();
        let d_card = (card.r as i16 - panel.r as i16).abs();
        assert!(d_stripe > 0, "a listra nao mudou nada: e' invisivel");
        assert!(
            d_stripe * 2 <= d_card,
            "a listra anda {d_stripe} e o degrau de cartao anda {d_card} — ela deixou de ser \
             discreta e passa a ler-se como aninhamento"
        );
    }
}
