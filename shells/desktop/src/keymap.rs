//! Translate winit's [`KeyCode`] into the editor's canonical
//! `KEY_*` constants.
//!
//! Extracted from [`main`] (Track B2). Pure free fn — the editor
//! pipeline doesn't know about winit; this is the one + only place
//! the shell bridges between them.

use winit::keyboard::KeyCode;

/// Map a winit [`KeyCode`] into the editor's canonical KEY_*
/// constants (the values `dispatch_key` matches against). Returns
/// `None` for keys the editor pipeline doesn't currently consume.
pub fn winit_to_editor_keycode(code: KeyCode) -> Option<u32> {
    use ph2d_editor::interaction::{
        KEY_ARROW_DOWN, KEY_ARROW_LEFT, KEY_ARROW_RIGHT, KEY_ARROW_UP, KEY_BACKSPACE, KEY_DELETE,
        KEY_ENTER, KEY_ESCAPE, KEY_KEY_A, KEY_KEY_C, KEY_KEY_D, KEY_KEY_F, KEY_KEY_K, KEY_KEY_P,
        KEY_KEY_V, KEY_KEY_X, KEY_SPACE, KEY_TAB,
    };
    Some(match code {
        KeyCode::Tab => KEY_TAB,
        KeyCode::Enter | KeyCode::NumpadEnter => KEY_ENTER,
        KeyCode::Space => KEY_SPACE,
        KeyCode::Escape => KEY_ESCAPE,
        KeyCode::Backspace => KEY_BACKSPACE,
        KeyCode::Delete => KEY_DELETE,
        KeyCode::ArrowUp => KEY_ARROW_UP,
        KeyCode::ArrowDown => KEY_ARROW_DOWN,
        KeyCode::ArrowLeft => KEY_ARROW_LEFT,
        KeyCode::ArrowRight => KEY_ARROW_RIGHT,
        KeyCode::KeyA => KEY_KEY_A,
        KeyCode::KeyC => KEY_KEY_C,
        // Motion Nodes M0.T3 — graph-editor shortcut letters (D duplicate,
        // F fit, K knife, P probe). Consumed only while a graph surface has
        // focus; otherwise they fall through unmapped as before.
        KeyCode::KeyD => KEY_KEY_D,
        KeyCode::KeyF => KEY_KEY_F,
        KeyCode::KeyK => KEY_KEY_K,
        KeyCode::KeyP => KEY_KEY_P,
        KeyCode::KeyV => KEY_KEY_V,
        KeyCode::KeyX => KEY_KEY_X,
        _ => return None,
    })
}
