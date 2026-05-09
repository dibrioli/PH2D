//! gilrs ↔ [`ph2d_input`] adapter.
//!
//! Lives in the shell because it depends on gilrs (a desktop-only dep
//! that ph2d-input deliberately avoids — see ph2d-input's Cargo.toml).
//! The iPad shell will provide its own equivalent translating
//! `GCController` events to `ph2d_input::Event`.

use ph2d_input::{Event, GamepadAxis, GamepadButton};

/// Map a gilrs Button to our [`GamepadButton`].
/// Returns `None` for buttons we deliberately don't surface (e.g.
/// touchpad clicks, paddle buttons) — those land in M9+ when needed.
pub fn map_button(b: gilrs::Button) -> Option<GamepadButton> {
    Some(match b {
        gilrs::Button::South => GamepadButton::South,
        gilrs::Button::East => GamepadButton::East,
        gilrs::Button::West => GamepadButton::West,
        gilrs::Button::North => GamepadButton::North,
        gilrs::Button::LeftTrigger => GamepadButton::LeftBumper,
        gilrs::Button::LeftTrigger2 => GamepadButton::LeftTrigger,
        gilrs::Button::RightTrigger => GamepadButton::RightBumper,
        gilrs::Button::RightTrigger2 => GamepadButton::RightTrigger,
        gilrs::Button::Select => GamepadButton::Select,
        gilrs::Button::Start => GamepadButton::Start,
        gilrs::Button::Mode => GamepadButton::Mode,
        gilrs::Button::LeftThumb => GamepadButton::LeftStick,
        gilrs::Button::RightThumb => GamepadButton::RightStick,
        gilrs::Button::DPadUp => GamepadButton::DPadUp,
        gilrs::Button::DPadDown => GamepadButton::DPadDown,
        gilrs::Button::DPadLeft => GamepadButton::DPadLeft,
        gilrs::Button::DPadRight => GamepadButton::DPadRight,
        // C, Z, paddles, touchpad, etc. — skip until a use case asks.
        _ => return None,
    })
}

pub fn map_axis(a: gilrs::Axis) -> Option<GamepadAxis> {
    Some(match a {
        gilrs::Axis::LeftStickX => GamepadAxis::LeftStickX,
        gilrs::Axis::LeftStickY => GamepadAxis::LeftStickY,
        gilrs::Axis::RightStickX => GamepadAxis::RightStickX,
        gilrs::Axis::RightStickY => GamepadAxis::RightStickY,
        gilrs::Axis::LeftZ => GamepadAxis::LeftTrigger,
        gilrs::Axis::RightZ => GamepadAxis::RightTrigger,
        // DPad-as-axis variants are handled by the button mapping
        // (gilrs emits both forms on some controllers).
        _ => return None,
    })
}

/// Translate a single gilrs event into a [`ph2d_input::Event`].
/// Filtered events (Connected/Disconnected/Dropped, unmapped buttons)
/// return `None` — caller should skip.
pub fn translate(event: gilrs::EventType) -> Option<Event> {
    match event {
        gilrs::EventType::ButtonPressed(b, _code) => map_button(b).map(Event::GamepadButtonDown),
        gilrs::EventType::ButtonReleased(b, _code) => map_button(b).map(Event::GamepadButtonUp),
        gilrs::EventType::AxisChanged(a, value, _code) => {
            map_axis(a).map(|axis| Event::GamepadAxis { axis, value })
        }
        // ButtonRepeated, ButtonChanged (analog), Connected,
        // Disconnected, Dropped, ForceFeedbackEffectCompleted → skip
        // for M8 (logged by the shell's poll loop, not as InputEvent).
        _ => None,
    }
}
