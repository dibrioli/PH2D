//! Keycode constants the dispatcher recognizes.
//!
//! Extracted from [`super`] (Track A1). The shell normalizes its
//! platform-native keycodes (winit, AppKit, win32) to these values
//! before forwarding to [`super::dispatch_key`]; the dispatcher then
//! routes by exact-match on `KeyEvent::keycode`.
//!
//! Values mirror common platform-independent keycodes; arrow keys
//! use the macOS NSEvent function-key range (`0xF700..`). See the
//! shell's `KeyEvent::keycode` field documentation for the
//! canonical mapping.

pub const KEY_TAB: u32 = 0x09;
pub const KEY_ENTER: u32 = 0x0D;
pub const KEY_SPACE: u32 = 0x20;
pub const KEY_ESCAPE: u32 = 0x1B;
pub const KEY_BACKSPACE: u32 = 0x08;
pub const KEY_KEY_A: u32 = 0x41;
pub const KEY_KEY_C: u32 = 0x43;
pub const KEY_KEY_V: u32 = 0x56;
pub const KEY_KEY_X: u32 = 0x58;
// Motion Nodes M0.T3 — graph-editor shortcut letters (ASCII uppercase, same
// scheme as A/C/V/X above) + forward Delete (macOS NSEvent function-key range,
// like the arrows). Consumed by `dispatch_key`'s graph-focus arm.
pub const KEY_KEY_D: u32 = 0x44;
pub const KEY_KEY_F: u32 = 0x46;
pub const KEY_KEY_K: u32 = 0x4B;
pub const KEY_KEY_P: u32 = 0x50;
pub const KEY_DELETE: u32 = 0xF728;
pub const KEY_ARROW_UP: u32 = 0xF700;
pub const KEY_ARROW_DOWN: u32 = 0xF701;
pub const KEY_ARROW_LEFT: u32 = 0xF702;
pub const KEY_ARROW_RIGHT: u32 = 0xF703;
