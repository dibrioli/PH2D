//! A tinta de uma linha da árvore: nasceu com os cartões, e o painel de antes passa por ela.

use ph2d_editor_core::paint::resolve;
use ph2d_editor_core::widget::card_ink::CardInk;
use ph2d_tokens::{ColorToken, Theme, TypeToken};

/// A tinta de uma linha: a do painel de antes ou a do cartão (05 §5).
#[derive(Copy, Clone, Debug)]
pub(crate) enum RowInk {
    Panel(Theme),
    Card(CardInk),
}

impl RowInk {
    pub(crate) fn color(self, token: ColorToken) -> ph2d_vector::Color {
        match self {
            Self::Panel(theme) => resolve(token, theme),
            Self::Card(ink) => ink.color(token),
        }
    }

    pub(crate) fn theme(self) -> Theme {
        match self {
            Self::Panel(theme) => theme,
            Self::Card(ink) => ink.theme,
        }
    }

    pub(crate) fn name_px(self) -> f32 {
        match self {
            Self::Panel(_) => TypeToken::Sm.px(),
            Self::Card(_) => ph2d_card_stack::metrics::VALUE_PX,
        }
    }
}
