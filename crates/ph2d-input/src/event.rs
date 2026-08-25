//! Top-level [`Event`] union: gamepad button down/up, axis change,
//! pencil event. Shells emit these; [`crate::InputState::apply_event`]
//! routes them to the right per-device handler.

use crate::gamepad::{GamepadAxis, GamepadButton};
use crate::keyboard::Key;
use crate::pencil::PencilEvent;

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Event {
    GamepadButtonDown(GamepadButton),
    GamepadButtonUp(GamepadButton),
    GamepadAxis {
        axis: GamepadAxis,
        value: f32,
    },
    Pencil(PencilEvent),
    /// ⚠️ **O teclado entrou em 2026-08-24 com o Input Map.** A crate nasceu na M8 com gamepad e
    /// caneta; a esmagadora maioria das ligações que um artista autora é uma tecla.
    ///
    /// A `Key` é o keycode **já normalizado** pela shell — ver [`crate::keyboard`].
    KeyDown(Key),
    KeyUp(Key),
    /// **Larga tudo** — a janela perdeu o foco. É a metade que esta crate pode resolver do `Up`
    /// perdido que deixaria um personagem a andar sozinho para sempre.
    FocusLost,
}
