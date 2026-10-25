//! **O Pixel Sprite ▸ Edit** (ADR-0192) — a composição: o ponteiro, a roda e o teclado chegam à
//! família `ph2d-app-pixel`, que decide. `true` = consumido.

use super::*;
use winit::keyboard::PhysicalKey;

impl crate::App {
    fn pixel_canvas<R>(
        &mut self,
        f: impl FnOnce(&mut ph2d_app_pixel::PixelApp, &ph2d_app_pixel::input::Canvas<'_>) -> R,
    ) -> Option<R> {
        let g = self.gfx.as_ref()?;
        let hero = g.hero_screen.as_ref()?;
        let world = g.sim.world();
        let canvas = ph2d_app_pixel::input::Canvas {
            world,
            camera: &g.camera,
            window: crate::scene_mapping::janela(hero.view.center_split, g.surface.size()),
            ppm: hero.project.pixels_per_meter,
            editing: ph2d_app_pixel::mode::editing(world, hero),
        };
        Some(f(&mut self.pixel, &canvas))
    }

    /// Premir/largar no canvas em Edit.
    pub(super) fn ramo_pixel(
        &mut self,
        kind: PointerKind,
        button: ph2d_host::PointerButton,
        on_canvas: bool,
        menu_open_before: bool,
    ) -> bool {
        use ph2d_app_pixel::input::{Button, press, release};
        if kind == PointerKind::Up {
            return release(&mut self.pixel);
        }
        let button = match button {
            ph2d_host::PointerButton::Primary => Button::Primary,
            ph2d_host::PointerButton::Secondary => Button::Secondary,
            _ => return false,
        };
        if kind != PointerKind::Down || !on_canvas || menu_open_before {
            return false;
        }
        let (p, alt) = (self.last_pointer, self.modifiers.alt_key());
        self.pixel_canvas(|app, c| press(app, c, p, button, alt))
            .unwrap_or(false)
    }

    /// Mover: o traço em curso continua (consumido); sem traço, só a célula sob o cursor.
    pub(crate) fn ramo_pixel_mover(&mut self) -> bool {
        let p = self.last_pointer;
        self.pixel_canvas(|app, c| ph2d_app_pixel::input::moved(app, c, p))
            .unwrap_or(false)
    }

    /// A roda sobre o canvas em Edit: zoom inteiro.
    pub(crate) fn roda_pixel(&mut self, delta: winit::event::MouseScrollDelta) -> bool {
        let dy = match delta {
            winit::event::MouseScrollDelta::LineDelta(_, y) => y,
            winit::event::MouseScrollDelta::PixelDelta(p) => p.y as f32,
        };
        let p = self.last_pointer;
        if dy == 0.0 || !self.pointer_on_canvas(p.0, p.1) {
            return false;
        }
        let Some(g) = self.gfx.as_mut() else {
            return false;
        };
        let Some(hero) = g.hero_screen.as_ref() else {
            return false;
        };
        let window = crate::scene_mapping::janela(hero.view.center_split, g.surface.size());
        let ppm = hero.project.pixels_per_meter;
        let editing = ph2d_app_pixel::mode::editing(g.sim.world(), hero);
        let Some(level) = ph2d_app_pixel::input::wheel(
            &self.pixel,
            g.sim.world(),
            &mut g.camera,
            window,
            ppm,
            editing,
            p,
            dy > 0.0,
        ) else {
            return false;
        };
        let n = level.to_string();
        g.toasts.push(Toast::info(ph2d_i18n::tr_with("pixel.zoom", &[("n", &n)])));
        self.any_input_this_frame = true;
        true
    }

    /// As teclas do Edit (B, E, I, X, Ctrl+Z…), salvo com um campo de texto a escrever.
    pub(crate) fn pixel_owns_the_keyboard(&mut self, key: PhysicalKey, state: ElementState) -> bool {
        let PhysicalKey::Code(code) = key else {
            return false;
        };
        if state != ElementState::Pressed || self.surface_owns_the_keyboard() {
            return false;
        }
        let (ctrl, shift) = (
            self.modifiers.control_key() || self.modifiers.super_key(),
            self.modifiers.shift_key(),
        );
        let Some(g) = self.gfx.as_mut() else {
            return false;
        };
        let Some(hero) = g.hero_screen.as_ref() else {
            return false;
        };
        let world = g.sim.world();
        let editing = ph2d_app_pixel::mode::editing(world, hero);
        let Some(answer) =
            ph2d_app_pixel::input::key(&mut self.pixel, world, editing, code, ctrl, shift)
        else {
            return false;
        };
        if let ph2d_app_pixel::input::KeyAnswer::Toast(t) = answer {
            g.toasts.push(Toast::info(t));
        }
        true
    }
}
