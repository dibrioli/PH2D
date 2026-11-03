//! ⭐⭐ **A tinta de um cartão** (`docs/CartoesUI` §5) — a ÚNICA porta por onde o cartão, os seus
//! campos, segmentos, listas suspensas e marcas resolvem cor.
//!
//! Sem tema próprio (ou com o mesmo da lista) o cartão é o do Kanban: os tokens `card-*`/`kanban-*`
//! no tema da lista, byte a byte o aprovado na C3. Com tema próprio, esses tokens traduzem-se para
//! os tons DO tema ([`ph2d_tokens::derive::card_tone`]) e o acento dele — o cartão inteiro muda,
//! não só o texto (report do dono 01/11).

use crate::paint::{resolve, token_to_vello};
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
    pub fn of_card(own: Option<Theme>, list: Theme) -> Self {
        match own {
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

thread_local! {
    static CORPO: std::cell::Cell<Option<CardInk>> = const { std::cell::Cell::new(None) };
}

/// Corre `f` com a tinta do cartão publicada; repõe a anterior no fim (também num pânico).
pub fn com<R>(ink: CardInk, f: impl FnOnce() -> R) -> R {
    struct Repoe(Option<CardInk>);
    impl Drop for Repoe {
        fn drop(&mut self) {
            CORPO.with(|c| c.set(self.0));
        }
    }
    let _repoe = Repoe(CORPO.with(|c| c.replace(Some(ink))));
    f()
}

/// O fundo de uma caixa de campo no cartão cujo corpo se pinta — `None` fora de um cartão.
#[must_use]
pub fn card_field_fill() -> Option<Color> {
    card_body().map(|ink| ink.color(ColorToken::CardField))
}

/// A tinta do cartão cujo corpo se está a pintar — `None` fora de um cartão.
#[must_use]
pub fn card_body() -> Option<CardInk> {
    CORPO.with(std::cell::Cell::get)
}
