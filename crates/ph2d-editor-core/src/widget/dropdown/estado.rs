//! **O ESTADO de um chip de dropdown e como ele se sente** — saiu do [`super`] por TETO DE LOC
//! (`503 > 500`, integração de 13/11); o corte é de assunto: o pai pinta o chip, este diz em que
//! estado ele está e que borda isso lhe dá.

use crate::paint::resolve;
use ph2d_tokens::{ColorToken, Theme};

#[derive(Copy, Clone, Debug, PartialEq, Eq, Default)]
pub enum DropdownState {
    #[default]
    Normal,
    Hovered,
    Focused,
    Disabled,
}

/// **Como um chip de dropdown se SENTE** ([`ph2d_tokens::visuals::Feel`]); `pub` pela razão do
/// [`chip_border_color`].
#[must_use]
pub fn dropdown_feel(state: DropdownState) -> ph2d_tokens::visuals::Feel {
    use ph2d_tokens::visuals::Feel;
    match state {
        DropdownState::Hovered => Feel::Hovered,
        DropdownState::Focused => Feel::Focused,
        DropdownState::Disabled => Feel::Disabled,
        DropdownState::Normal => Feel::Rest,
    }
}

/// **A cor da borda de um CHIP de dropdown, já com o eixo do hover** — a porta ÚNICA (`pub`: o
/// `ph2d-panel-painter-layers` desenha o seu chip à mão). Só o par `Normal ⇄ Hovered` interpola —
/// o `Accent` do aberto nomeia um ESTADO (a cerca do `text_input::border_color`).
#[must_use]
pub fn chip_border_color(state: DropdownState, hover_t: f32, theme: Theme) -> ph2d_vector::Color {
    crate::motion::hover_axis(
        matches!(state, DropdownState::Normal | DropdownState::Hovered),
        hover_t,
        Some(ColorToken::Border.resolve(theme)),
        Some(ColorToken::BorderEmph.resolve(theme)),
    )
    .map_or_else(
        || {
            resolve(
                match state {
                    DropdownState::Focused => ColorToken::Accent,
                    DropdownState::Hovered => ColorToken::BorderEmph,
                    _ => ColorToken::Border,
                },
                theme,
            )
        },
        crate::paint::token_to_vello,
    )
}
