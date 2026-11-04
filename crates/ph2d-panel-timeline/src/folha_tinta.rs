//! A tinta do que rodeia a área de trabalho da Timeline nos cartões: régua, barra de rolagem, zebra e
//! costura sobre a lista, e o tom das listas abertas (`docs/CartoesUI/14_CHROME_TIMELINE_MOTION.md`).
//! Chaves, curvas, playhead e marcadores têm cor própria e não passam por aqui. `PH2D_CARDS=0` dá o
//! token de sempre, ao bit.

use ph2d_a11y::NodeId;
use ph2d_editor_core::paint::resolve;
use ph2d_editor_core::panel::cards::ferramenta::cards_on;
use ph2d_editor_core::widget::card_ink::{self, CardInk, troca_popover};
use ph2d_tokens::{ColorToken, Theme};
use ph2d_vector::Color;

/// O papel de cada token da folha na tinta da lista.
const fn papel(token: ColorToken) -> ColorToken {
    match token {
        ColorToken::TimelineRulerBg => ColorToken::KanbanCard,
        ColorToken::TimelineRowAlt | ColorToken::Border => ColorToken::KanbanSlot,
        ColorToken::Bg2 => ColorToken::CardField,
        ColorToken::TimelineRulerTick | ColorToken::Text2 => ColorToken::CardLabel,
        ColorToken::BgElev => ColorToken::KanbanCardRaised,
        other => other,
    }
}

/// A cor de `token` na folha.
#[must_use]
pub(crate) fn cor(token: ColorToken, theme: Theme) -> Color {
    if cards_on() {
        CardInk::of_list(theme).color(papel(token))
    } else {
        resolve(token, theme)
    }
}

/// A lista aberta de `id` tem a cara do cartão: a porta `paint_dropdown_list` lê esta lembrança.
pub(crate) fn popover_no_tom(id: NodeId, theme: Theme) {
    if cards_on() {
        let _ = troca_popover(Some((id, CardInk::of_list(theme))));
    }
}

/// Corre `f` com a tinta da lista publicada, para os widgets partilhados de uma lista aberta.
pub(crate) fn no_tom(theme: Theme, f: impl FnOnce()) {
    if cards_on() {
        card_ink::com(CardInk::of_list(theme), f);
    } else {
        f();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ph2d_editor_core::panel::cards::ferramenta::force_cards_on_this_thread;

    /// ⭐ Sem cartões é o token de sempre; com cartões, cada peça do cromo sai da tinta da lista, e a
    /// cor do trabalho não muda. *Mutação: tirar um braço de `papel` ⇒ vermelho nesse token.*
    #[test]
    fn a_folha_muda_de_tinta_so_nos_cartoes_e_so_no_cromo() {
        let tema = Theme::Dark;
        force_cards_on_this_thread(Some(false));
        for t in [
            ColorToken::TimelineRulerBg,
            ColorToken::TimelineRowAlt,
            ColorToken::Border,
            ColorToken::Bg2,
        ] {
            assert_eq!(cor(t, tema), resolve(t, tema), "{t:?} sem cartões");
        }
        force_cards_on_this_thread(Some(true));
        for (t, papel) in [
            (ColorToken::TimelineRulerBg, ColorToken::KanbanCard),
            (ColorToken::TimelineRowAlt, ColorToken::KanbanSlot),
            (ColorToken::Border, ColorToken::KanbanSlot),
            (ColorToken::Bg2, ColorToken::CardField),
            (ColorToken::TimelineRulerTick, ColorToken::CardLabel),
            (ColorToken::Text2, ColorToken::CardLabel),
        ] {
            assert_eq!(cor(t, tema), resolve(papel, tema), "{t:?} nos cartões");
        }
        for t in [
            ColorToken::TimelineKey,
            ColorToken::TimelinePlayhead,
            ColorToken::TimelineCurve,
            ColorToken::Accent,
        ] {
            assert_eq!(
                cor(t, tema),
                resolve(t, tema),
                "{t:?} é trabalho, não cromo"
            );
        }
        force_cards_on_this_thread(None);
    }
}
