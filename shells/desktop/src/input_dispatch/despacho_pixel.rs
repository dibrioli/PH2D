//! **O Pixel Sprite ▸ Edit** (ADR-0192) — composição só: o ponteiro, a roda e o teclado chegam à
//! `ph2d_app_pixel::compose`, que decide. `true` = consumido.

use super::*;
use ph2d_app_pixel::compose::{Pointer, View};
use winit::keyboard::PhysicalKey;

impl crate::App {
    fn pixel_view(&mut self) -> Option<(&mut ph2d_app_pixel::PixelApp, View<'_>)> {
        let g = self.gfx.as_mut()?;
        let hero = g.hero_screen.as_ref()?;
        let window = crate::scene_mapping::janela(hero.view.center_split, g.surface.size());
        let view = View::of(g.sim.world(), hero, &mut g.camera, &mut g.toasts, window);
        Some((&mut self.pixel, view))
    }

    /// Premir/largar no canvas em Edit.
    pub(super) fn ramo_pixel(
        &mut self,
        kind: PointerKind,
        button: ph2d_host::PointerButton,
        on_canvas: bool,
        menu_open_before: bool,
    ) -> bool {
        let (p, alt) = (self.last_pointer, self.modifiers.alt_key());
        let Some((app, mut v)) = self.pixel_view() else {
            return false;
        };
        match app.pointer(&mut v, kind, button, p, alt, on_canvas && !menu_open_before) {
            Pointer::NotMine => return false,
            Pointer::Done => {}
            Pointer::PanStart => self.pan_anchor = Some(p),
            Pointer::PanEnd => self.pan_anchor = None,
        }
        true
    }

    pub(crate) fn ramo_pixel_mover(&mut self) -> bool {
        let p = self.last_pointer;
        self.pixel_view()
            .is_some_and(|(app, v)| app.pointer_moved(&v, p))
    }

    pub(crate) fn roda_pixel(&mut self, delta: winit::event::MouseScrollDelta) -> bool {
        let p = self.last_pointer;
        let on = self.pointer_on_canvas(p.0, p.1);
        let eaten =
            (self.pixel_view()).is_some_and(|(app, mut v)| app.wheel_event(&mut v, delta, p, on));
        self.any_input_this_frame |= eaten;
        eaten
    }

    /// As teclas do Edit, salvo com um campo de texto a escrever.
    pub(crate) fn pixel_owns_the_keyboard(&mut self, key: PhysicalKey, s: ElementState) -> bool {
        if self.surface_owns_the_keyboard() {
            return false;
        }
        let m = self.modifiers;
        let (ctrl, shift) = (m.control_key() || m.super_key(), m.shift_key());
        let pressed = s == ElementState::Pressed;
        (self.pixel_view())
            .is_some_and(|(app, mut v)| app.keyboard(&mut v, key, pressed, ctrl, shift))
    }
}
