//! **O TOQUE, do lado da shell** (F5, D10) — o contacto da janela entra no reconhecedor
//! (`ph2d_editor_core::interaction::touch`, a lei e os gates), e o que ele pede sai pelas portas que
//! o rato e as teclas JÁ usam: o botão do rato, o desfazer da barra, o `Ctrl+Space`, o clique direito.

use ph2d_editor_core::interaction::touch::CHIPS;
use ph2d_editor_core::interaction::{TouchOut, TouchPhase};
use ph2d_editor_core::{Toast, ToastQueue, ZenMode};
use ph2d_i18n::tr;
use winit::dpi::PhysicalPosition;
use winit::event::{ElementState, MouseButton, Touch};
use winit::keyboard::ModifiersState;

thread_local! {
    /// Os modificadores do TECLADO (só ele) — `App::modifiers` é este OU o toque. ⚠️ Aqui e não num
    /// campo do `App`: o `App` só larga campos (`the_app_only_sheds_fields`).
    static KEYBOARD: std::cell::Cell<ModifiersState> =
        const { std::cell::Cell::new(ModifiersState::empty()) };
}

/// O teclado mudou (o `ModifiersChanged` da janela).
pub(crate) fn set_keyboard(state: ModifiersState) {
    KEYBOARD.with(|k| k.set(state));
}

/// ⭐ **O modo foco** — a porta ÚNICA do `Ctrl+Space`, de *Ver ▸ Modo foco* e do toque de 4 dedos.
pub(crate) fn toggle_focus(zen: &mut ZenMode, toasts: &mut ToastQueue) -> bool {
    if !zen.try_toggle() {
        return false;
    }
    toasts.push(Toast::info(tr(if zen.is_active() {
        "shell.input_handlers.zen_mode_on_zones"
    } else {
        "shell.input_handlers.zen_mode_off_zones"
    })));
    true
}

impl crate::App {
    fn touch_scale(&self) -> f32 {
        self.host.as_ref().map_or(1.0, |h| h.scale().get())
    }

    /// Um contacto da janela. As posições vão ao reconhecedor em pt LÓGICOS (a folga do UIKit).
    pub(crate) fn on_touch(&mut self, t: Touch) {
        let s = self.touch_scale();
        let (x, y) = (t.location.x as f32, t.location.y as f32);
        let phase = match t.phase {
            winit::event::TouchPhase::Started => TouchPhase::Start,
            winit::event::TouchPhase::Moved => TouchPhase::Move,
            winit::event::TouchPhase::Ended => TouchPhase::End,
            winit::event::TouchPhase::Cancelled => TouchPhase::Cancel,
        };
        let now = Self::timestamp_ns();
        let Some(hero) = self.gfx.as_mut().and_then(|g| g.hero_screen.as_mut()) else {
            return;
        };
        let chip = hero.chrome_hit(x, y).filter(|id| CHIPS.contains(id));
        let outs = hero
            .store
            .touch_mut()
            .feed(t.id, phase, [x / s, y / s], now, chip);
        self.apply_touch(outs);
    }

    /// Em todo quadro: o tocar-e-segurar não espera por outro evento.
    pub(crate) fn touch_tick(&mut self) {
        let now = Self::timestamp_ns();
        let out = self
            .gfx
            .as_mut()
            .and_then(|g| g.hero_screen.as_mut())
            .and_then(|h| h.store.touch_mut().tick(now));
        self.apply_touch(out.into_iter().collect());
    }

    fn apply_touch(&mut self, outs: Vec<TouchOut>) {
        let s = self.touch_scale();
        let at = |p: [f32; 2]| PhysicalPosition::new(f64::from(p[0] * s), f64::from(p[1] * s));
        for o in outs {
            match o {
                TouchOut::Press(p) | TouchOut::Drag(p) | TouchOut::Release(p) => {
                    self.on_cursor_moved(at(p));
                    if let TouchOut::Press(_) | TouchOut::Release(_) = o {
                        let st = if matches!(o, TouchOut::Press(_)) {
                            ElementState::Pressed
                        } else {
                            ElementState::Released
                        };
                        self.on_mouse_input(st, MouseButton::Left);
                    }
                }
                TouchOut::Undo => self.undo_button = Some(false),
                TouchOut::Redo => self.undo_button = Some(true),
                TouchOut::Focus => {
                    if let Some(g) = self.gfx.as_mut() {
                        self.title_dirty |= toggle_focus(&mut g.zen, &mut g.toasts);
                    }
                }
                TouchOut::ContextMenu(p) => {
                    self.on_cursor_moved(at(p));
                    self.on_mouse_input(ElementState::Pressed, MouseButton::Right);
                    self.on_mouse_input(ElementState::Released, MouseButton::Right);
                }
            }
        }
        self.sync_touch_modifiers();
    }

    /// Os modificadores efectivos = o teclado OU o toque (dedos extra, widget segurado, travados).
    pub(crate) fn effective_modifiers(&self) -> ModifiersState {
        let t = self
            .gfx
            .as_ref()
            .and_then(|g| g.hero_screen.as_ref())
            .map(|h| h.store.touch().modifiers())
            .unwrap_or_default();
        let mut want = KEYBOARD.with(std::cell::Cell::get);
        want.set(ModifiersState::SHIFT, want.shift_key() || t.shift);
        want.set(ModifiersState::ALT, want.alt_key() || t.alt);
        want.set(ModifiersState::CONTROL, want.control_key() || t.ctrl);
        want
    }

    /// Em todo quadro e a cada contacto: um botão do widget travado pelo RATO também conta.
    pub(crate) fn sync_touch_modifiers(&mut self) {
        let want = self.effective_modifiers();
        if want != self.modifiers {
            self.apply_modifiers(want);
        }
    }
}
