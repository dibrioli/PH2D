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

        // ADR-0108 Fase 1: modo vetorial (flag PH2D_VEC_PEN) — U/I/D/X fazem a
        // booleana (Union/Intersect/Difference/Exclude) das 2 últimas regiões
        // fechadas; Delete/Backspace apaga o path
        // selecionado. Modo de teste dedicado (a pill/menu real entra no cutover,
        // Fase R). Só sem modificadores, pra não colidir com atalhos.
        if self.vector_tool_active()
            && state == ElementState::Pressed
            && !repeat
            && self.modifiers.is_empty()
            && let PhysicalKey::Code(code) = physical_key
        {
            let op = match code {
                KeyCode::KeyU => Some(ph2d_vec_boolean::BoolOp::Union),
                KeyCode::KeyI => Some(ph2d_vec_boolean::BoolOp::Intersect),
                KeyCode::KeyD => Some(ph2d_vec_boolean::BoolOp::Subtract),
                KeyCode::KeyX => Some(ph2d_vec_boolean::BoolOp::Exclude),
                _ => None,
            };
            if let Some(op) = op {
                self.vec_boolean(op);
                return;
            }
            if matches!(code, KeyCode::Delete | KeyCode::Backspace)
                && self.vec_delete_selected_vertex_or_path()
            {
                return;
            }
        }

        // Arrow keys nudge the selection (nodes if any, else the whole path).
        // Allows Shift (coarse 10px, unlike the boolean block above); blocked by
        // Ctrl/Alt/Super and while drawing. Auto-repeat keeps moving but a held
        // arrow coalesces into ONE undo step (records only on the first press).
        if self.vector_tool_active()
            && state == ElementState::Pressed
            && !self.vec_pen.is_drawing()
            && !self.modifiers.control_key()
            && !self.modifiers.alt_key()
            && !self.modifiers.super_key()
            && let PhysicalKey::Code(code) = physical_key
        {
            let step = if self.modifiers.shift_key() {
                10.0
            } else {
                1.0
            };
            let delta = match code {
                KeyCode::ArrowLeft => Some((-step, 0.0)),
                KeyCode::ArrowRight => Some((step, 0.0)),
                KeyCode::ArrowUp => Some((0.0, -step)),
                KeyCode::ArrowDown => Some((0.0, step)),
                _ => None,
            };
            if let Some((dx, dy)) = delta
                && self.vec_nudge_selected(dx, dy, !repeat)
            {
                return;
            }
        }

        // ADR-0108 Fase 2: undo/redo + save/load + clipboard com Ctrl/Cmd. Ctrl+Z
        // desfaz, Ctrl+Shift+Z / Ctrl+Y refaz, Ctrl+S salva, Ctrl+O carrega,
        // Ctrl+C/X/V copia/recorta/cola o path, Ctrl+D duplica. C/X/V cedem o
        // atalho a um campo de texto focado (clipboard de texto do widget).
        if self.vector_tool_active()
            && state == ElementState::Pressed
            && !repeat
            && (self.modifiers.control_key() || self.modifiers.super_key())
            && let PhysicalKey::Code(code) = physical_key
        {
            let text_focused = self.vector_text_field_focused();
            let handled = match code {
                KeyCode::KeyZ if self.modifiers.shift_key() => {
                    self.vec_redo();
                    true
                }
                KeyCode::KeyZ => {
                    self.vec_undo();
                    true
                }
                KeyCode::KeyY => {
                    self.vec_redo();
                    true
                }
                KeyCode::KeyS => {
                    self.vec_save();
                    true
                }
                KeyCode::KeyO => {
                    self.vec_load();
                    true
                }
                KeyCode::KeyC if !text_focused => {
                    self.vec_copy();
                    true
                }
                KeyCode::KeyX if !text_focused => {
                    self.vec_cut();
                    true
                }
                KeyCode::KeyV if !text_focused => {
                    self.vec_paste();
                    true
                }
                KeyCode::KeyD => {
                    self.vec_duplicate_shortcut();
                    true
                }
                _ => false,
            };
            if handled {
                return;
            }
        }

        // Motion Nodes M1 Phase 1b-3: graph undo/redo with Ctrl/Cmd, while the
        // Motion tool is active. Returns early when handled so the same KeyZ does
        // NOT fall through to the painter / image-edit undo in `handle_editor_key`
        // (mirror of the Vector block above).
        if self.motion_tool_active()
            && state == ElementState::Pressed
            && !repeat
            && (self.modifiers.control_key() || self.modifiers.super_key())
            && let PhysicalKey::Code(code) = physical_key
        {
            let handled = match code {
                KeyCode::KeyZ if self.modifiers.shift_key() => {
                    self.motion_redo();
                    true
                }
                KeyCode::KeyZ => {
                    self.motion_undo();
                    true
                }
                KeyCode::KeyY => {
                    self.motion_redo();
                    true
                }
                _ => false,
            };
            if handled {
                return;
            }
        }

        // Vector: Escape ends an in-progress path (it stays in the scene, open).
        // Consumed only while the Vector tool is active and the Pen is drawing,
        // so Escape otherwise falls through to widget blur.
        if state == ElementState::Pressed
            && !repeat
            && matches!(physical_key, PhysicalKey::Code(KeyCode::Escape))
            && self.vector_tool_active()
            && self.vec_pen.is_drawing()
        {
            self.vec_pen.finish();
            return;
        }
        // Painter shapes (Curve/Circle): Escape discards the in-progress shape (reverts the preview);
        // Enter commits it (bakes the painted stroke). Consumed only when a shape session is open (the
        // helpers gate on it), so both keys fall through to widget-blur / text fields otherwise.
        if state == ElementState::Pressed
            && !repeat
            && matches!(physical_key, PhysicalKey::Code(KeyCode::Escape))
            && self.painter_shape_cancel()
        {
            return;
        }
        if state == ElementState::Pressed
            && !repeat
            && matches!(
                physical_key,
                PhysicalKey::Code(KeyCode::Enter | KeyCode::NumpadEnter)
            )
            && self.painter_shape_commit()
        {
            return;
        }

        // Painter brush size: `[` shrinks, `]` grows the active brush
        // (Blender/Photoshop convention). Consumed only when the Painter tool is
        // active (the nudge downcast gates on it), so the brackets fall through
        // otherwise. `Pressed` covers held-key repeat so the size keeps changing.
        if state == ElementState::Pressed
            && let PhysicalKey::Code(code @ (KeyCode::BracketLeft | KeyCode::BracketRight)) =
                physical_key
        {
            let dir = if code == KeyCode::BracketRight { 1 } else { -1 };
            if self.painter_nudge_brush_size(dir) {
                return;
            }
        }

        // Painter eraser toggle: `E` flips erase mode (Blender/PS convention).
        // Consumed only when the Painter tool is active (the toggle gates on it),
        // so `E` falls through otherwise. No modifiers, no repeat.
        if state == ElementState::Pressed
            && !repeat
            && matches!(physical_key, PhysicalKey::Code(KeyCode::KeyE))
            && !(self.modifiers.super_key() || self.modifiers.control_key())
            && self.painter_toggle_eraser()
        {
            return;
        }

        // Painter Curve: Delete/Backspace drops the selected control point (kept ≥ 2). Tried BEFORE
        // the Falloff delete — curve editing is on-canvas, falloff editing is in the Brush dock, so
        // only one can have a selection. Consumed only when a curve point was removed.
        if state == ElementState::Pressed
            && matches!(
                physical_key,
                PhysicalKey::Code(KeyCode::Delete | KeyCode::Backspace)
            )
            && self.painter_curve_delete_selected_point()
        {
            return;
        }

        // Painter Falloff curve: Delete/Backspace drops the selected control
        // point. Consumed only when a Falloff point is selected (the helper gates
        // on it), so the key falls through to other tools otherwise.
        if state == ElementState::Pressed
            && matches!(
                physical_key,
                PhysicalKey::Code(KeyCode::Delete | KeyCode::Backspace)
            )
            && self.painter_delete_selected_falloff_point()
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
