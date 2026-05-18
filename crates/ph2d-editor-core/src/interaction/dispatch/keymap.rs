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
pub const KEY_ARROW_UP: u32 = 0xF700;
pub const KEY_ARROW_DOWN: u32 = 0xF701;
pub const KEY_ARROW_LEFT: u32 = 0xF702;
pub const KEY_ARROW_RIGHT: u32 = 0xF703;
