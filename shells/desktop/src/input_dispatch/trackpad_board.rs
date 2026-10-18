//! W23 — o trackpad num QUADRO activo (MiroClone): dois dedos movem a vista, a pinça dá zoom; a roda
//! do rato continua pela `on_mouse_wheel`. A lei vive em `board_view::scroll`; aqui só a escolha.

use winit::event::MouseScrollDelta;

use crate::App;
use crate::forwarding::cursor_over_hero_panel;

impl App {
    /// W23 — dois dedos no trackpad (píxeis, sem Ctrl) MOVEM um quadro activo (o Miro); o resto é a
    /// roda de sempre. A pinça do toque passa pela roda (`touch_input`), não por aqui.
    pub(crate) fn on_wheel_event(&mut self, delta: MouseScrollDelta) {
        use ph2d_editor_core::screens::hero::board_view::{BoardScroll, scroll};
        let (x, y) = self.last_pointer;
        if let MouseScrollDelta::PixelDelta(p) = delta
            && !(self.modifiers.control_key() || self.modifiers.super_key())
            && !self.command_palette_wheel(p.y as f32)
            && !cursor_over_hero_panel(self.gfx.as_ref(), x, y)
            && let Some(hero) = self.gfx.as_mut().and_then(|g| g.hero_screen.as_mut())
            && scroll(hero, x, y, BoardScroll::Pan([p.x as f32, p.y as f32]))
        {
            self.any_input_this_frame = true;
            return;
        }
        self.on_mouse_wheel(delta);
    }

    /// W23 — a pinça do trackpad (macOS) dá zoom num quadro activo.
    pub(crate) fn on_pinch_gesture(&mut self, delta: f64) {
        use ph2d_editor_core::screens::hero::board_view::{BoardScroll, scroll};
        let (x, y) = self.last_pointer;
        if let Some(hero) = self.gfx.as_mut().and_then(|g| g.hero_screen.as_mut())
            && scroll(hero, x, y, BoardScroll::Pinch(1.0 + delta))
        {
            self.any_input_this_frame = true;
        }
    }
}
