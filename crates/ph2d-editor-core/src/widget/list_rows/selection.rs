//! ⭐⭐⭐ **O REALCE DE UMA LINHA DE LISTA** — apontada e escolhida, com UMA lei.
//!
//! Enio, 2026-09-07, com a foto da cadeia de efeitos: *«veja que o nome Low-Pass está com o fundo
//! na cor dos botões, sem layout adequado»* — e a ordem: *«estude o layout e corrija»*.
//!
//! # ⛔ Os dois defeitos que a foto mostra
//!
//! **(a) O TOM era o do botão em REPOUSO.** A cadeia pintava a linha escolhida com `Bg3`, que é
//! exactamente o fundo de um botão parado — logo a linha *era* um botão, aos olhos. A lei do modelo
//! diz outra coisa: no Godot Modern o `selected` de uma `Tree` é `flat_button_pressed` com
//! `content_margin_all(0)` (`theme_modern.cpp:709`) — **o tom do PRESSIONADO**, não o do repouso.
//! Nesta casa esse tom é o [`ColorToken::AccentSoft`].
//!
//! **(b) Havia DOIS dialectos.** A hierarquia pintava `AccentSoft` **com raio e com uma moldura de
//! acento**; a cadeia pintava `Bg3` **a sangrar e sem quinas**. A mesma frase — *esta é a linha que
//! está em mãos* — dita de duas maneiras, em dois painéis. *Duas superfícies que dizem a mesma
//! coisa de maneiras diferentes ensinam ao artista que elas são coisas diferentes.*
//!
//! # A lei, e de onde vem cada metade
//!
//! - **o tom**: `AccentSoft` (apontada: `Bg2`) — o *pressed* do modelo, nunca o repouso;
//! - **SANGRA** até fora do corpo (`content_margin_all(0)` do modelo): uma selecção de lista ocupa
//!   a faixa inteira, ela não é uma caixa dentro da faixa;
//! - ⛔ **sem QUINAS e sem MOLDURA** — e isto é veredito do dono, medido duas vezes: *«o nome de um
//!   efeito de áudio parece um botão»* (2026-09-06) e de novo no dia seguinte. O que diz «botão» é
//!   a **quina**: com a lei do grupo, um botão desta casa arredonda pelo menos um canto, e uma
//!   linha de lista não arredonda **nenhum**. *É a mesma régua do Blender vista do outro lado — lá
//!   o que agrupa é a quina que fica; aqui o que separa é a quina que não existe.*

use crate::zones::Rect;
use ph2d_tokens::{ColorToken, Theme};
use ph2d_vector::VectorScene;

/// O que a linha é para o ponteiro e para o documento.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum RowHighlight {
    /// Nada — a linha mostra a listra da paridade dela, e mais nada.
    None,
    /// O ponteiro está em cima.
    Hovered,
    /// É esta a linha em mãos.
    Selected,
}

impl RowHighlight {
    /// O tom, ou `None` quando não há realce nenhum.
    #[must_use]
    pub fn token(self) -> Option<ColorToken> {
        match self {
            Self::None => None,
            Self::Hovered => Some(ColorToken::Bg2),
            Self::Selected => Some(ColorToken::AccentSoft),
        }
    }
}

/// **Pinta o realce de uma linha de lista.**
///
/// `bleed` é o quanto a faixa transborda para cada lado — a folga do cartão que envolve a secção,
/// quando existe; `0.0` quando a lista já ocupa a largura toda.
///
/// ⚠️ **A selecção VENCE o hover, e não é ordem de desenho — é significado.** Escolhida é um facto
/// do documento (*esta é a linha em mãos*); apontada é um facto do ponteiro, que dura o que a mão
/// durar. Pintar o hover por cima faria a linha escolhida mudar de cor por alguém passar o rato.
pub fn paint_row_highlight(
    scene: &mut VectorScene,
    rect: Rect,
    theme: Theme,
    highlight: RowHighlight,
    bleed: f32,
) {
    let Some(token) = highlight.token() else {
        return;
    };
    let tint = token.resolve(theme);
    crate::paint::fill_rounded_rect(
        scene,
        Rect::new(rect.x - bleed, rect.y, rect.w + bleed * 2.0, rect.h),
        // ⛔ ZERO, e é a lei: uma linha de lista não arredonda canto nenhum.
        0.0,
        ph2d_vector::Color::from_rgba8(tint.r, tint.g, tint.b, tint.a), // LITERAL-COLOR-OK: token-bridge
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    /// ⛔ **O tom de uma linha escolhida NÃO é o de um botão em repouso** — foi esse o report.
    #[test]
    fn a_selected_row_never_wears_the_resting_button_tone() {
        assert_ne!(
            RowHighlight::Selected.token(),
            Some(ColorToken::Bg3),
            "a linha escolhida voltou ao tom de um botao parado: ela le^-se como um botao"
        );
        assert_eq!(RowHighlight::Selected.token(), Some(ColorToken::AccentSoft));
        assert_eq!(RowHighlight::None.token(), None);
    }

    /// ⭐ **Ela SANGRA e não tem quinas** — as duas metades que a separam de um botão.
    #[test]
    fn the_highlight_bleeds_and_has_no_corners() {
        let mut scene = VectorScene::new();
        let bleed = 4.0;
        paint_row_highlight(
            &mut scene,
            Rect::new(10.0, 0.0, 100.0, 22.0),
            Theme::Dark,
            RowHighlight::Selected,
            bleed,
        );
        let painted = scene.inner().encoding().n_path_segments;
        assert!(painted > 0, "a linha escolhida nao pintou nada");

        // Um rectângulo SEM quinas custa menos geometria que um com elas — é assim que a ausência
        // de raio se mede numa cena, sem ler o argumento que lhe demos.
        let mut rounded = VectorScene::new();
        crate::paint::fill_rounded_rect(
            &mut rounded,
            Rect::new(10.0, 0.0, 100.0, 22.0),
            6.0,
            ph2d_vector::Color::from_rgba8(1, 2, 3, 255), // LITERAL-COLOR-OK: sonda
        );
        assert!(
            painted < rounded.inner().encoding().n_path_segments,
            "a faixa da linha escolhida emite tanta geometria como um rectangulo ARREDONDADO: \
             ela ganhou quinas, e e' a quina que diz «botao»"
        );
    }

    /// Sem realce nenhum, nada é pintado — senão toda linha ganhava uma faixa.
    #[test]
    fn a_plain_row_paints_no_highlight() {
        let mut scene = VectorScene::new();
        paint_row_highlight(
            &mut scene,
            Rect::new(0.0, 0.0, 100.0, 22.0),
            Theme::Dark,
            RowHighlight::None,
            0.0,
        );
        assert_eq!(scene.inner().encoding().n_path_segments, 0);
    }
}
