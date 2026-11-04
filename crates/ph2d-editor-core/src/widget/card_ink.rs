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

    /// O fundo deste cartão no valor do token, para quem mistura a partir dele (a listra da lista).
    #[must_use]
    pub fn superficie(self) -> ph2d_tokens::Color {
        if self.own {
            card_tone(self.theme, CardTone::Surface)
        } else {
            ColorToken::KanbanCard.resolve(self.theme)
        }
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
    static POPOVER: std::cell::Cell<Option<(ph2d_a11y::NodeId, CardInk)>> =
        const { std::cell::Cell::new(None) };
}

/// Guarda da tinta publicada: repõe a anterior ao morrer (também num pânico).
#[must_use]
pub struct Publicada(Option<CardInk>);

impl Drop for Publicada {
    fn drop(&mut self) {
        CORPO.with(|c| c.set(self.0));
    }
}

/// Publica a tinta do cartão até o guarda morrer.
pub fn publica(ink: CardInk) -> Publicada {
    Publicada(CORPO.with(|c| c.replace(Some(ink))))
}

/// Corre `f` com a tinta do cartão publicada; repõe a anterior no fim (também num pânico).
pub fn com<R>(ink: CardInk, f: impl FnOnce() -> R) -> R {
    crate::panel::cards::censo::entra_na_tinta();
    let _repoe = publica(ink);
    f()
}

/// A lista suspensa aberta cujo chip se pintou dentro de um cartão: `(id, tinta)`. O popover pinta-se
/// num passe diferido, fora do corpo do cartão, e é por aqui que leva a tinta dele (T25, 05 §4).
pub(crate) fn lembra_popover(id: ph2d_a11y::NodeId, ink: CardInk) {
    POPOVER.with(|c| c.set(Some((id, ink))));
}

/// A tinta do popover `id` — `None` se o chip dele não veio de um cartão.
pub(crate) fn tinta_do_popover(id: ph2d_a11y::NodeId) -> Option<CardInk> {
    POPOVER
        .with(std::cell::Cell::take)
        .filter(|(i, _)| *i == id)
        .map(|(_, ink)| ink)
}

/// Troca a lembrança do popover (`None` = limpa) e devolve a que estava — o painel cuja lista pode ser
/// de um cartão PRESO (T61) pergunta assim se o corpo dele abriu alguma.
#[must_use]
pub fn troca_popover(
    novo: Option<(ph2d_a11y::NodeId, CardInk)>,
) -> Option<(ph2d_a11y::NodeId, CardInk)> {
    POPOVER.with(|c| c.replace(novo))
}

/// O papel de um token de lista aberta num cartão — os mesmos tons do popover T16 do cartão.
pub(crate) const fn papel_no_popover(token: ColorToken) -> ColorToken {
    match token {
        ColorToken::BgElev => ColorToken::KanbanCardRaised,
        ColorToken::Border => ColorToken::CardField,
        ColorToken::Accent | ColorToken::Bg3 => ColorToken::CardFieldHover,
        ColorToken::AccentFg | ColorToken::Text1 => ColorToken::KanbanCardFg,
        ColorToken::Text2 | ColorToken::Text3 => ColorToken::CardLabel,
        other => other,
    }
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

/// ⭐ **A cor de um widget partilhado** (`docs/CartoesUI` §5): fora de um cartão o token no `theme`,
/// ao bit; dentro, o tom do cartão que faz o mesmo papel (texto, rótulo, acento, hover).
#[must_use]
pub fn tinta(token: ColorToken, theme: Theme) -> Color {
    card_body().map_or_else(
        || resolve(token, theme),
        |ink| ink.color(papel_no_cartao(token)),
    )
}

const fn papel_no_cartao(token: ColorToken) -> ColorToken {
    match token {
        ColorToken::Text1 => ColorToken::KanbanCardFg,
        ColorToken::Text2 | ColorToken::Text3 => ColorToken::CardLabel,
        ColorToken::Accent => ColorToken::KanbanBlueBold,
        ColorToken::AccentFg => ColorToken::KanbanBtnOnFg,
        ColorToken::Bg2 => ColorToken::CardFieldHover,
        ColorToken::Bg1 => ColorToken::CardField,
        ColorToken::BgElev | ColorToken::Bg3 => ColorToken::KanbanCardRaised,
        other => other,
    }
}
