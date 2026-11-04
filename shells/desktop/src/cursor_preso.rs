//! O cursor preso do campo Figma: a lei vive em [`ph2d_cursor_preso`]; aqui, a composição.

use winit::dpi::PhysicalPosition;

impl crate::App {
    /// Chamado depois de cada evento do ponteiro e do teclado.
    pub(crate) fn sync_cursor_preso(&mut self) {
        let quer = (self.gfx.as_ref())
            .and_then(|g| g.hero_screen.as_ref())
            .is_some_and(|h| h.store.pointer_lock_wanted());
        let Some(win) = self.window.clone() else {
            return;
        };
        if let Some(p) = self.cursor_preso.sync(&win, quer, self.last_pointer) {
            self.last_pointer = p;
            self.last_cursor = p;
        }
    }

    /// `None` = engolido (o `Locked` move-se pelo cru); senão a posição que o app deve ver.
    pub(crate) fn cursor_moved_preso(
        &mut self,
        pos: PhysicalPosition<f64>,
    ) -> Option<PhysicalPosition<f64>> {
        self.cursor_preso.moved(self.window.as_deref(), pos)
    }

    /// A tecla, e o cursor preso segue o que ela mudou.
    pub(crate) fn on_keyboard_preso(&mut self, event: winit::event::KeyEvent) {
        self.on_keyboard_input(event);
        self.sync_cursor_preso();
    }

    /// O movimento CRU (`DeviceEvent::MouseMotion`) — só conta com o cursor `Locked`.
    pub(crate) fn on_device_event(&mut self, event: winit::event::DeviceEvent) {
        let winit::event::DeviceEvent::MouseMotion { delta } = event else {
            return;
        };
        let Some((x, y)) = self.cursor_preso.raw(delta) else {
            return;
        };
        let b = self
            .held_button
            .unwrap_or(ph2d_host::PointerButton::Primary);
        let t = Self::timestamp_ns();
        let kind = ph2d_host::PointerKind::Move;
        self.pointer_moved(ph2d_pen::ponteiro::do_rato(x, y, kind, b, t));
        self.sync_cursor_preso();
    }

    /// O cursor ↔ sobre a pega de um campo de cartão ou a borda de um cartão solto (T39).
    pub(crate) fn sobre_a_pega_de_um_campo(&self, h: &ph2d_editor_core::HeroScreen) -> bool {
        let (x, y) = h.escala().to_logical(self.last_pointer);
        h.hit_index.hit_with_rect(x, y).is_some_and(|(id, r)| {
            ph2d_editor_core::widget::card_field::ew_resize_under(&h.store, id, r, x)
        })
    }
}
