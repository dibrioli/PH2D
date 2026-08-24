//! [`InputState`] — top-level container for all input devices.
//!
//! Per-frame protocol:
//!   1. Call [`InputState::begin_frame`] (snapshots last-frame held).
//!   2. For each shell input event, call [`InputState::apply_event`].
//!   3. Sim systems / script bridge query the state.

use crate::event::Event;
use crate::gamepad::GamepadState;
use crate::keyboard::KeyboardState;
use crate::pencil::PencilState;

#[derive(Default, Debug, Clone)]
pub struct InputState {
    pub gamepad: GamepadState,
    pub pencil: PencilState,
    /// ⚠️ Entrou em 2026-08-24 com o Input Map — ver [`crate::keyboard`].
    pub keyboard: KeyboardState,
}

impl InputState {
    pub fn new() -> Self {
        Self::default()
    }

    /// Snapshot last-frame held buttons across all per-device states.
    /// Call ONCE per frame BEFORE the shell pumps events.
    pub fn begin_frame(&mut self) {
        self.gamepad.begin_frame();
        self.keyboard.begin_frame();
    }

    pub fn apply_event(&mut self, event: Event) {
        match event {
            Event::GamepadButtonDown(b) => self.gamepad.handle_button_down(b),
            Event::GamepadButtonUp(b) => self.gamepad.handle_button_up(b),
            Event::GamepadAxis { axis, value } => self.gamepad.handle_axis(axis, value),
            Event::Pencil(p) => self.pencil.handle_event(p),
            Event::KeyDown(k) => self.keyboard.handle_key_down(k),
            Event::KeyUp(k) => self.keyboard.handle_key_up(k),
            // ⚠️ So' o TECLADO larga aqui: o gamepad reporta o estado real dele mesmo sem foco, e
            // a caneta nao tem estado segurado que sobreviva a uma troca de janela.
            Event::FocusLost => self.keyboard.release_all(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::gamepad::{GamepadAxis, GamepadButton};
    use crate::pencil::PencilEvent;

    #[test]
    fn round_trip_event_to_query() {
        let mut s = InputState::new();
        s.begin_frame();
        s.apply_event(Event::GamepadButtonDown(GamepadButton::South));
        s.apply_event(Event::GamepadAxis {
            axis: GamepadAxis::LeftStickX,
            value: 0.7,
        });
        s.apply_event(Event::Pencil(PencilEvent::Squeeze { force: 0.3 }));

        assert!(s.gamepad.pressed(GamepadButton::South));
        assert!((s.gamepad.axis(GamepadAxis::LeftStickX) - 0.7).abs() < 1e-6);
        assert!((s.pencil.squeeze_force() - 0.3).abs() < 1e-6);
    }
}
