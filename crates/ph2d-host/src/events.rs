//! Event types crossing the shell↔core boundary.
//!
//! Subset of §13 of the SKILL — minimal for M1. Will grow with later
//! marcos (gamepad in M8, IME in M11, etc.).
//!
//! All types are `#[repr(C)]`-compatible (no enum payloads larger than
//! a u64) so they can be marshaled across the FFI boundary later
//! without reshuffling fields.

/// Window dimensions in **physical pixels**.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct WindowSize {
    pub width: u32,
    pub height: u32,
}

impl WindowSize {
    pub const fn new(width: u32, height: u32) -> Self {
        Self { width, height }
    }
}

/// App lifecycle transitions emitted by the OS.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Lifecycle {
    /// App became active and is in the foreground.
    Foreground,
    /// App is being moved to the background; GPU contexts may be lost
    /// (per ADR-0020). Save state opportunistically.
    Background,
    /// OS warning that memory is tight — drop non-essential caches
    /// (LRU atlases, decompressed audio).
    LowMemory,
    /// App is about to be terminated. Last chance to flush state.
    WillTerminate,
}

/// User pointer event — mouse, touch, or Apple Pencil.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct PointerEvent {
    /// Position in **physical pixels** relative to the surface.
    pub x: f32,
    pub y: f32,
    /// Pressure in [0.0, 1.0]. Mouse always reports 1.0; touch may
    /// report 1.0 if device doesn't support pressure.
    pub pressure: f32,
    pub kind: PointerKind,
    pub source: PointerSource,
    /// Which button caused this event. `Move` events report the
    /// button that's currently being held (or `Primary` if none).
    pub button: PointerButton,
    /// Monotonic clock in nanoseconds (host-defined epoch).
    pub timestamp_ns: u128,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum PointerKind {
    Down,
    Up,
    Move,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Default)]
pub enum PointerButton {
    #[default]
    Primary,
    Secondary,
    Middle,
}

/// Mouse-wheel / trackpad scroll event. `delta_y` is in DIP (already
/// converted from `winit::MouseScrollDelta` by the shell — for line
/// deltas we multiply by ~16 px/line). Positive `delta_y` scrolls
/// content **upwards** (cursor sees content move up).
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct WheelEvent {
    /// Cursor position when the wheel rotated, in physical pixels.
    pub x: f32,
    pub y: f32,
    pub delta_x: f32,
    pub delta_y: f32,
    pub modifiers: Modifiers,
    pub timestamp_ns: u128,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum PointerSource {
    Mouse,
    Touch,
    Pencil,
}

/// Keyboard event — physical key down / up / repeat.
///
/// IME composing string is a separate channel (will be added when text
/// editing widget lands; §11.3 + §13).
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct KeyEvent {
    /// Platform-independent keycode (Latin-1 alphabetics use ASCII;
    /// special keys use winit-style scancodes — to be normalized in M8).
    pub keycode: u32,
    pub modifiers: Modifiers,
    pub kind: KeyKind,
    pub timestamp_ns: u128,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum KeyKind {
    Down,
    Up,
    Repeat,
}

/// Bitflag-like struct for modifier state. Keeping `bool` fields
/// instead of bitflags to avoid a `bitflags` dep in this minimal
/// crate; the cost is 4 bytes vs 1 — negligible.
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
pub struct Modifiers {
    pub shift: bool,
    pub ctrl: bool,
    pub alt: bool,
    /// `Cmd` on macOS / `Win` on Windows / `Super` on Linux.
    pub meta: bool,
}

/// Decision returned by [`crate::HostHandler::on_close_request`].
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum CloseAction {
    /// Allow the OS to close the window.
    Close,
    /// Veto the close (e.g., to prompt "save changes?" first).
    Cancel,
}
