//! O botão do rato do winit vira o `PointerEvent` da porta única ([`App::pointer_button`]). Irmão do
//! `input_dispatch` pelo tecto de 600 LOC da shell (HR-18).

use crate::App;
use ph2d_host::PointerKind;
use winit::event::{ElementState, MouseButton};

impl App {
    pub(crate) fn on_mouse_input(&mut self, state: ElementState, button: MouseButton) {
        let kind = match state {
            ElementState::Pressed => PointerKind::Down,
            ElementState::Released => PointerKind::Up,
        };
        let mapped_button = match button {
            MouseButton::Left => ph2d_host::PointerButton::Primary,
            MouseButton::Right => ph2d_host::PointerButton::Secondary,
            MouseButton::Middle => ph2d_host::PointerButton::Middle,
            _ => ph2d_host::PointerButton::Primary,
        };
        let (x, y, t) = (
            self.last_pointer.0,
            self.last_pointer.1,
            Self::timestamp_ns(),
        );
        let evt = ph2d_pen::ponteiro::do_rato(x, y, kind, mapped_button, t);
        self.pointer_button(evt, button);
        self.sync_cursor_preso();
    }
}
