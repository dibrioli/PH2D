//! Keyboard event handling — winit `KeyboardInput` → host key event +
//! hero-pipeline key/text forwarding + M12 demo controls. Extracted
//! from `input_dispatch.rs` to keep that file under the HR-18 LOC cap.

use winit::event::{ElementState, KeyEvent as WinitKeyEvent};
use winit::keyboard::{KeyCode, PhysicalKey};

use ph2d_host::{HostHandler, KeyEvent, KeyKind};

use crate::App;
use crate::forwarding::{forward_key_to_hero, forward_text_to_hero};
use crate::keymap::winit_to_editor_keycode;

impl App {
    pub(crate) fn on_keyboard_input(&mut self, event: WinitKeyEvent) {
        let WinitKeyEvent {
            physical_key,
            state,
            repeat,
            text,
            ..
        } = event;

        let keycode = match physical_key {
            PhysicalKey::Code(code) => code as u32,
            PhysicalKey::Unidentified(_) => 0,
        };
        let kind = match (state, repeat) {
            (ElementState::Pressed, false) => KeyKind::Down,
            (ElementState::Pressed, true) => KeyKind::Repeat,
            (ElementState::Released, _) => KeyKind::Up,
        };
        self.handler.on_key(KeyEvent {
            keycode,
            modifiers: Self::convert_modifiers(self.modifiers),
            kind,
            timestamp_ns: Self::timestamp_ns(),
        });

        // Vector Pen: Escape cancels the in-progress path, or clears the
        // committed scene when none is in progress. Consumed only when
        // the Pen tool is active with something to cancel/clear —
        // otherwise it falls through to the hero pipeline (widget blur).
        if state == ElementState::Pressed
            && !repeat
            && matches!(physical_key, PhysicalKey::Code(KeyCode::Escape))
            && self.try_vector_pen_escape()
        {
            return;
        }
        // Vector Pencil: Escape cancels the in-progress freehand stroke, or
        // clears the committed scene when none is open. Mirror of the Pen.
        if state == ElementState::Pressed
            && !repeat
            && matches!(physical_key, PhysicalKey::Code(KeyCode::Escape))
            && self.try_vector_pencil_escape()
        {
            return;
        }

        // Hero pipeline (ADR-0024): translate winit's physical KeyCode
        // into the editor's KEY_* constants and route to the focused
        // widget.
        if state == ElementState::Pressed
            && let PhysicalKey::Code(code) = physical_key
            && let Some(editor_keycode) = winit_to_editor_keycode(code)
        {
            forward_key_to_hero(
                self.gfx.as_mut(),
                KeyEvent {
                    keycode: editor_keycode,
                    modifiers: Self::convert_modifiers(self.modifiers),
                    kind,
                    timestamp_ns: Self::timestamp_ns(),
                },
            );
        }
        // Printable text from this key event (winit already resolved
        // layout + dead-keys + shift). Send each char through the
        // text-input dispatcher so focused TextInput/NumberInput/
        // Combobox buffers update. EXCEPT for ' ' coming from the
        // physical Space key and 'a'/'A' coming from KeyA with Cmd/
        // Ctrl held — those are inserted by the dispatch's key handler
        // directly.
        if state == ElementState::Pressed
            && let Some(s) = text.as_ref()
        {
            let is_space_key = matches!(physical_key, PhysicalKey::Code(KeyCode::Space));
            let cmd_held = self.modifiers.super_key() || self.modifiers.control_key();
            for ch in s.chars() {
                if ch.is_control() {
                    continue;
                }
                if is_space_key && ch == ' ' {
                    continue;
                }
                // Cmd/Ctrl chord with a letter: skip the text-event so
                // Cmd+A's select-all isn't overwritten by 'a' insertion.
                if cmd_held && ch.is_ascii_alphabetic() {
                    continue;
                }
                forward_text_to_hero(self.gfx.as_mut(), ch);
            }
        }

        // M12 demo controls (only on key Down, no repeat).
        if matches!((state, repeat), (ElementState::Pressed, false))
            && let PhysicalKey::Code(code) = physical_key
        {
            self.handle_editor_key(code);
        }
    }
}
