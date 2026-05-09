#![forbid(unsafe_code)]
//! ph2d-input — input abstraction (M8, HR-1).
//!
//! Pure-data crate: no platform deps. Shells translate their native
//! input source (gilrs on desktop, UIPencilInteraction on iPad) into
//! [`Event`] and call [`InputState::apply_event`]. The current snapshot
//! is then read by sim systems and projected into Luau via
//! `ph2d.input` (wired in `ph2d-script`).
//!
//! Press / hold / release semantics: each tick, call
//! [`InputState::begin_frame`] to snapshot last-frame held state, then
//! apply zero or more events. Queries:
//! - [`GamepadState::pressed`] — true on the first frame the button
//!   went from up → down (one-shot, edge-triggered).
//! - [`GamepadState::held`] — true for every frame the button is down.
//! - [`GamepadState::released`] — true on the first frame the button
//!   went from down → up.
//! - [`GamepadState::axis`] — current value in `[-1.0, 1.0]`, or `0.0`
//!   if no event ever set it.
//!
//! Pencil M8 is **stub-only** — types declared so iPad shell can
//! plumb without breaking ph2d-input's API in M9+. No real handler.

pub mod event;
pub mod gamepad;
pub mod pencil;
pub mod state;

pub use event::Event;
pub use gamepad::{GamepadAxis, GamepadButton, GamepadState};
pub use pencil::{PencilButton, PencilEvent, PencilState};
pub use state::InputState;
