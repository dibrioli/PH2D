//! **O TOQUE, do lado da shell** (F5–F6, D10) — o contacto da janela entra no reconhecedor
//! (`ph2d_editor_core::interaction::touch`, a lei e os gates), e o que ele pede sai pelas portas que
//! o rato e as teclas JÁ usam: o botão do rato, o desfazer da barra, o `Ctrl+Space`, o clique
//! direito — e, para navegar com dois dedos (F6), o botão do MEIO e a RODA, nunca uma 2.ª câmera.
//!
//! ⚠️ A pinça dá zoom à volta do ponto entre os dedos; a roda dá zoom à volta do centro. A
//! diferença é um pan (`Camera2d::anchor_pan_px`), e ele passa pela porta do pan: com o botão do
//! meio pousado, o cursor anda o arrasto e volta ao sítio sem andar (o `pan_anchor` volta com ele).

use ph2d_editor_core::interaction::touch::CHIPS;
use ph2d_editor_core::interaction::{TouchOut, TouchPhase};
use ph2d_render::Camera2d;
use winit::dpi::PhysicalPosition;
use winit::event::{ElementState, MouseButton, MouseScrollDelta, Touch};
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

    /// Em todo quadro: o tocar-e-segurar não espera por outro evento, e os contactos sintéticos de
    /// uma cena de smoke saem por aqui.
    pub(crate) fn touch_tick(&mut self) {
        let now = Self::timestamp_ns();
        let s = self.touch_scale();
        let outs = self
            .gfx
            .as_mut()
            .and_then(|g| g.hero_screen.as_mut())
            .map(|h| {
                let t = h.store.touch_mut();
                t.note_scale(s);
                t.tick(now)
            })
            .unwrap_or_default();
        self.apply_touch(outs);
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
                        self.title_dirty |= g.zen.toggle_announced(&mut g.toasts);
                    }
                }
                TouchOut::ContextMenu(p) => {
                    self.on_cursor_moved(at(p));
                    self.on_mouse_input(ElementState::Pressed, MouseButton::Right);
                    self.on_mouse_input(ElementState::Released, MouseButton::Right);
                }
                TouchOut::NavBegin(p) => {
                    // F8: a memória do beliscão ao contrário lê a vista (nunca a escreve).
                    if let Some(g) = self.gfx.as_mut() {
                        g.view_flight.gesture_starts(g.camera.view());
                    }
                    self.on_cursor_moved(at(p));
                    self.on_mouse_input(ElementState::Pressed, MouseButton::Middle);
                }
                TouchOut::Pan(d) => {
                    let (x, y) = self.last_pointer;
                    self.on_cursor_moved(PhysicalPosition::new(
                        f64::from(x + d[0] * s),
                        f64::from(y + d[1] * s),
                    ));
                }
                TouchOut::Zoom { factor, about } => {
                    self.pinch(factor, [about[0] * s, about[1] * s])
                }
                TouchOut::NavEnd => {
                    self.on_mouse_input(ElementState::Released, MouseButton::Middle)
                }
                // F8: o beliscão rápido pede pela porta do menu *Ver* (ao contrário: a vista de antes).
                TouchOut::QuickPinch { closing } => {
                    use ph2d_editor_core::{ViewFocusKind as K, action_bus::EditorAction as A};
                    let kind = if closing { K::All } else { K::Previous };
                    if let Some(h) = self.gfx.as_mut().and_then(|g| g.hero_screen.as_mut()) {
                        h.bus.push(A::SetViewFocus { kind });
                    }
                }
            }
        }
        self.sync_touch_modifiers();
    }

    /// A pinça: um entalhe de roda com o factor dela e, se a CENA o tomou, o arrasto que mantém o
    /// mundo sob os dedos parado — pela porta do pan, com o botão do meio já pousado.
    fn pinch(&mut self, factor: f32, about_px: [f32; 2]) {
        let Some(gfx) = self.gfx.as_ref() else {
            return;
        };
        let live = gfx.camera.height_world;
        let before = gfx.canvas_zoom.goal(live);
        let dy = f64::from(Camera2d::wheel_px_for(factor));
        self.on_mouse_wheel(MouseScrollDelta::PixelDelta(PhysicalPosition::new(0.0, dy)));
        let Some(gfx) = self.gfx.as_ref() else {
            return;
        };
        let after = gfx.canvas_zoom.goal(live);
        if after == before {
            // Um quadro, uma moldura ou a paleta tomou a roda, ou a câmera está no limite.
            return;
        }
        let win = gfx.scene_window();
        let centre = [win.width as f32 / 2.0, win.height as f32 / 2.0];
        let [dx, dy] = Camera2d::anchor_pan_px(about_px, centre, before, after, live);
        let anchor = self.last_pointer;
        self.on_cursor_moved(PhysicalPosition::new(
            f64::from(anchor.0 + dx),
            f64::from(anchor.1 + dy),
        ));
        self.last_pointer = anchor;
        if self.pan_anchor.is_some() {
            self.pan_anchor = Some(anchor);
        }
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
