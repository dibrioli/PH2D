//! Top-level [`Event`] union: gamepad button down/up, axis change,
//! pencil event. Shells emit these; [`crate::InputState::apply_event`]
//! routes them to the right per-device handler.

use crate::gamepad::{GamepadAxis, GamepadButton};
use crate::pencil::PencilEvent;

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Event {
    GamepadButtonDown(GamepadButton),
    GamepadButtonUp(GamepadButton),
    GamepadAxis { axis: GamepadAxis, value: f32 },
    Pencil(PencilEvent),
}
