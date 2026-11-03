//! ⭐⭐ **A tinta de um cartão** (`docs/CartoesUI` §5) — a ÚNICA porta por onde o cartão, os seus
//! campos, segmentos, listas suspensas e marcas resolvem cor.
//!
//! Sem tema próprio (ou com o mesmo da lista) o cartão é o do Kanban: os tokens `card-*`/`kanban-*`
//! no tema da lista, byte a byte o aprovado na C3. Com tema próprio, esses tokens traduzem-se para
//! os tons DO tema ([`ph2d_tokens::derive::card_tone`]) e o acento dele — o cartão inteiro muda,
//! não só o texto (report do dono 01/11).

use crate::interaction::WidgetStore;
use crate::paint::{resolve, token_to_vello};
use ph2d_a11y::NodeId;
use ph2d_tokens::derive::{CardTone, card_tone};
use ph2d_tokens::{ColorToken, Theme};
use ph2d_vector::Color;

/// A tinta de um cartão: o tema em que ele se pinta e se é dele (≠ o da lista).
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct CardInk {
    pub theme: Theme,
    own: bool,
}

impl CardInk {
    /// A tinta da LISTA (e de um cartão sem tema): os tokens do Kanban.
    #[must_use]
    pub const fn of_list(theme: Theme) -> Self {
        Self { theme, own: false }
    }

    /// A tinta do cartão `section` numa lista pintada em `list`.
    #[must_use]
    pub fn of_card(store: &WidgetStore, section: NodeId, list: Theme) -> Self {
        match store.section_theme(section) {
            Some(t) if t != list => Self {
                theme: t,
                own: true,
            },
            _ => Self::of_list(list),
        }
    }

    /// O cartão tem tema próprio?
    #[must_use]
    pub const fn is_own(self) -> bool {
        self.own
    }

    /// ⭐ A cor de `token` neste cartão.
    #[must_use]
    pub fn color(self, token: ColorToken) -> Color {
        if !self.own {
            return resolve(token, self.theme);
        }
        let tom = match token {
            ColorToken::KanbanCard => CardTone::Surface,
            ColorToken::CardField => CardTone::Field,
            ColorToken::CardFieldHover => CardTone::FieldHover,
            ColorToken::KanbanCardRaised => CardTone::Raised,
            other => return resolve(Self::own_token(other), self.theme),
        };
        token_to_vello(card_tone(self.theme, tom))
    }

    /// Os tokens de TINTA do Kanban que, num cartão com tema próprio, são os do tema.
    const fn own_token(token: ColorToken) -> ColorToken {
        match token {
            ColorToken::CardLabel => ColorToken::Text3,
            ColorToken::KanbanBlueBold => ColorToken::Accent,
            ColorToken::KanbanBtnOnFg => ColorToken::AccentFg,
            ColorToken::KanbanListFg | ColorToken::KanbanCardFg => ColorToken::Text1,
            other => other,
        }
    }
}

#[cfg(test)]
#[path = "card_ink/tests.rs"]
mod tests;
