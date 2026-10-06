//! ⭐ **Com um QUADRO activo, o teclado é dele primeiro** (MiroClone, W1) — depois dos modais e antes
//! de todo atalho da cena: sem isto `Delete` apagaria o objecto seleccionado da cena, que está
//! escondida por baixo. A decisão (que tecla é de quem) é do `board_keys`; aqui só se traduz a tecla
//! física e se faz a ponte da área de transferência do sistema.

use ph2d_editor_core::screens::hero::board_keys::{self, BoardKey};
use winit::event::ElementState;
use winit::keyboard::{KeyCode, PhysicalKey};

/// A tecla física → a do quadro.
fn board_key(code: KeyCode) -> BoardKey {
    match code {
        KeyCode::KeyA => BoardKey::Char('a'),
        KeyCode::KeyB => BoardKey::Char('b'),
        KeyCode::KeyC => BoardKey::Char('c'),
        KeyCode::KeyD => BoardKey::Char('d'),
        KeyCode::KeyE => BoardKey::Char('e'),
        KeyCode::KeyF => BoardKey::Char('f'),
        KeyCode::KeyG => BoardKey::Char('g'),
        KeyCode::KeyH => BoardKey::Char('h'),
        KeyCode::KeyI => BoardKey::Char('i'),
        KeyCode::KeyJ => BoardKey::Char('j'),
        KeyCode::KeyK => BoardKey::Char('k'),
        KeyCode::KeyL => BoardKey::Char('l'),
        KeyCode::KeyM => BoardKey::Char('m'),
        KeyCode::KeyN => BoardKey::Char('n'),
        KeyCode::KeyO => BoardKey::Char('o'),
        KeyCode::KeyP => BoardKey::Char('p'),
        KeyCode::KeyQ => BoardKey::Char('q'),
        KeyCode::KeyR => BoardKey::Char('r'),
        KeyCode::KeyS => BoardKey::Char('s'),
        KeyCode::KeyT => BoardKey::Char('t'),
        KeyCode::KeyU => BoardKey::Char('u'),
        KeyCode::KeyV => BoardKey::Char('v'),
        KeyCode::KeyW => BoardKey::Char('w'),
        KeyCode::KeyX => BoardKey::Char('x'),
        KeyCode::KeyY => BoardKey::Char('y'),
        KeyCode::KeyZ => BoardKey::Char('z'),
        KeyCode::Digit0 => BoardKey::Char('0'),
        KeyCode::Digit1 => BoardKey::Char('1'),
        KeyCode::Digit2 => BoardKey::Char('2'),
        KeyCode::Digit3 => BoardKey::Char('3'),
        KeyCode::Digit4 => BoardKey::Char('4'),
        KeyCode::Digit5 => BoardKey::Char('5'),
        KeyCode::Digit6 => BoardKey::Char('6'),
        KeyCode::Digit7 => BoardKey::Char('7'),
        KeyCode::Digit8 => BoardKey::Char('8'),
        KeyCode::Digit9 => BoardKey::Char('9'),
        KeyCode::Enter | KeyCode::NumpadEnter => BoardKey::Enter,
        KeyCode::Escape => BoardKey::Escape,
        KeyCode::Backspace => BoardKey::Backspace,
        KeyCode::Delete => BoardKey::Delete,
        KeyCode::ArrowLeft => BoardKey::Left,
        KeyCode::ArrowRight => BoardKey::Right,
        KeyCode::ArrowUp => BoardKey::Up,
        KeyCode::ArrowDown => BoardKey::Down,
        KeyCode::Home => BoardKey::Home,
        KeyCode::End => BoardKey::End,
        KeyCode::Space => BoardKey::Space,
        _ => BoardKey::Other,
    }
}

impl crate::App {
    /// O quadro activo toma esta tecla? `true` ⇒ o despacho pára aqui.
    pub(crate) fn board_owns_the_keyboard(
        &mut self,
        physical_key: PhysicalKey,
        state: ElementState,
        text: Option<&str>,
    ) -> bool {
        let PhysicalKey::Code(code) = physical_key else {
            return false;
        };
        if self.text_entry_focused() {
            return false;
        }
        let mods = Self::convert_modifiers(self.modifiers);
        let k = board_key(code);
        let Some(g) = self.gfx.as_mut() else {
            return false;
        };
        let Some(hero) = g.hero_screen.as_mut() else {
            return false;
        };
        if hero.documents.active().is_none() {
            return false;
        }
        let pressed = state == ElementState::Pressed;
        let paste = (pressed && (mods.ctrl || mods.meta) && k == BoardKey::Char('v'))
            .then(|| g.clipboard.as_mut().and_then(|c| c.get_text().ok()))
            .flatten();
        let out = board_keys::key(hero, &mut g.text_system, k, mods, pressed, text, paste);
        if let Some(copy) = out.copy
            && let Some(c) = g.clipboard.as_mut()
        {
            let _ = c.set_text(copy);
        }
        if out.consumed {
            self.any_input_this_frame = true;
        }
        out.consumed
    }
}
