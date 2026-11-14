//! O botão do rato do winit vira o `PointerEvent` da porta única ([`App::pointer_button`]). Irmão do
//! `input_dispatch` pelo tecto de 600 LOC da shell (HR-18).

use crate::App;
use ph2d_host::{PointerEvent, PointerKind, PointerSource};
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

    /// A faixa única (sem a barra do sistema) toma o Down primário do rato E da caneta antes do editor:
    /// arrastar, redimensionar, maximizar. `true` = a janela tomou-o.
    pub(crate) fn ramo_faixa_unica(&mut self, evt: PointerEvent, button: MouseButton) -> bool {
        evt.kind == PointerKind::Down
            && button == MouseButton::Left
            && self.faixa_unica_down(evt.x, evt.y, matches!(evt.source, PointerSource::Pen(_)))
    }

    /// Foco perdido ou ponteiro fora: o `Up` nunca chega (Alt+Tab, grab; W3, B03). O foco solta as teclas; o
    /// traço do Painter fecha no último ponto de dentro, com o seu passo de undo (salvo cena que conduz).
    pub(crate) fn perde_a_mao(&mut self, foco: bool) {
        ph2d_pen::porta::poe_dentro(false);
        if foco {
            self.input.apply_event(ph2d_input::Event::FocusLost);
        }
        if !ph2d_app_painter::macro_track::caminho::a_cena_conduz_o_ponteiro() {
            self.painter_canvas_up();
        }
    }
}
