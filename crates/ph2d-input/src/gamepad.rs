//! Gamepad button + axis enums + per-frame [`GamepadState`].
//!
//! The button enum mirrors the standard mapping that gilrs / SDL /
//! winit / browser GamepadAPI all converge on. Two extra D-pad
//! buttons (DPadUp/Down/Left/Right) are explicit because some
//! controllers report them as buttons, others as a hat-axis — we
//! normalize at the shell layer.

use std::collections::{BTreeMap, BTreeSet};

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum GamepadButton {
    /// Bottom face button (Xbox A, PS Cross, Nintendo B).
    South,
    /// Right face button (Xbox B, PS Circle, Nintendo A).
    East,
    /// Left face button (Xbox X, PS Square, Nintendo Y).
    West,
    /// Top face button (Xbox Y, PS Triangle, Nintendo X).
    North,
    LeftBumper,
    RightBumper,
    LeftTrigger,
    RightTrigger,
    Select,
    Start,
    Mode,
    LeftStick,
    RightStick,
    DPadUp,
    DPadDown,
    DPadLeft,
    DPadRight,
}

impl GamepadButton {
    /// Stable lowercase string used as the Luau-side key in
    /// `ph2d.input`. Don't change without also bumping the script
    /// API contract — user scripts depend on these names.
    pub const fn as_lua_key(self) -> &'static str {
        match self {
            Self::South => "south",
            Self::East => "east",
            Self::West => "west",
            Self::North => "north",
            Self::LeftBumper => "left_bumper",
            Self::RightBumper => "right_bumper",
            Self::LeftTrigger => "left_trigger",
            Self::RightTrigger => "right_trigger",
            Self::Select => "select",
            Self::Start => "start",
            Self::Mode => "mode",
            Self::LeftStick => "left_stick",
            Self::RightStick => "right_stick",
            Self::DPadUp => "dpad_up",
            Self::DPadDown => "dpad_down",
            Self::DPadLeft => "dpad_left",
            Self::DPadRight => "dpad_right",
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum GamepadAxis {
    LeftStickX,
    LeftStickY,
    RightStickX,
    RightStickY,
    /// Trigger pressure normalized to `[0.0, 1.0]`. Some controllers
    /// only report triggers as buttons (binary); shells should still
    /// emit AxisChanged with `0.0` / `1.0` for those.
    LeftTrigger,
    RightTrigger,
}

impl GamepadAxis {
    pub const fn as_lua_key(self) -> &'static str {
        match self {
            Self::LeftStickX => "left_stick_x",
            Self::LeftStickY => "left_stick_y",
            Self::RightStickX => "right_stick_x",
            Self::RightStickY => "right_stick_y",
            Self::LeftTrigger => "left_trigger",
            Self::RightTrigger => "right_trigger",
        }
    }
}

/// Snapshot of one gamepad's state. Multi-pad support lands in M9+
/// (the renderer doesn't care; the script API will route via index).
#[derive(Default, Debug, Clone)]
pub struct GamepadState {
    held: BTreeSet<GamepadButton>,
    last_held: BTreeSet<GamepadButton>,
    axes: BTreeMap<GamepadAxis, f32>,
}

impl GamepadState {
    pub fn new() -> Self {
        Self::default()
    }

    /// Edge-trigger snapshot: copy current `held` into `last_held`
    /// so this frame's `pressed`/`released` queries reflect the
    /// transition since the prior frame. Call ONCE per frame BEFORE
    /// applying events.
    pub fn begin_frame(&mut self) {
        self.last_held.clone_from(&self.held);
    }

    pub fn handle_button_down(&mut self, b: GamepadButton) {
        self.held.insert(b);
    }

    pub fn handle_button_up(&mut self, b: GamepadButton) {
        self.held.remove(&b);
    }

    pub fn handle_axis(&mut self, a: GamepadAxis, value: f32) {
        // Clamp to canonical range — some platforms report outside
        // [-1, 1] under heavy stick deflection.
        self.axes.insert(a, value.clamp(-1.0, 1.0));
    }

    pub fn pressed(&self, b: GamepadButton) -> bool {
        self.held.contains(&b) && !self.last_held.contains(&b)
    }

    pub fn held(&self, b: GamepadButton) -> bool {
        self.held.contains(&b)
    }

    pub fn released(&self, b: GamepadButton) -> bool {
        !self.held.contains(&b) && self.last_held.contains(&b)
    }

    pub fn axis(&self, a: GamepadAxis) -> f32 {
        self.axes.get(&a).copied().unwrap_or(0.0)
    }

    /// Iterate currently-held buttons (sorted by `GamepadButton`'s
    /// Ord). Useful for the script-side input snapshot population
    /// + introspection in tests.
    pub fn iter_held(&self) -> impl Iterator<Item = GamepadButton> + '_ {
        self.held.iter().copied()
    }

    /// Iterate every axis that has ever been set (sorted).
    pub fn iter_axes(&self) -> impl Iterator<Item = (GamepadAxis, f32)> + '_ {
        self.axes.iter().map(|(a, v)| (*a, *v))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pressed_is_edge_triggered() {
        let mut g = GamepadState::new();
        g.begin_frame();
        g.handle_button_down(GamepadButton::South);
        assert!(g.pressed(GamepadButton::South));
        assert!(g.held(GamepadButton::South));
        assert!(!g.released(GamepadButton::South));

        // Next frame — still held but no longer "pressed" (edge gone).
        g.begin_frame();
        assert!(!g.pressed(GamepadButton::South));
        assert!(g.held(GamepadButton::South));
        assert!(!g.released(GamepadButton::South));

        // Release frame.
        g.begin_frame();
        g.handle_button_up(GamepadButton::South);
        assert!(!g.pressed(GamepadButton::South));
        assert!(!g.held(GamepadButton::South));
        assert!(g.released(GamepadButton::South));
    }

    #[test]
    fn axis_clamps_to_canonical_range() {
        let mut g = GamepadState::new();
        g.handle_axis(GamepadAxis::LeftStickX, 1.5);
        assert!((g.axis(GamepadAxis::LeftStickX) - 1.0).abs() < 1e-6);
        g.handle_axis(GamepadAxis::LeftStickX, -2.0);
        assert!((g.axis(GamepadAxis::LeftStickX) + 1.0).abs() < 1e-6);
        g.handle_axis(GamepadAxis::LeftStickX, 0.5);
        assert!((g.axis(GamepadAxis::LeftStickX) - 0.5).abs() < 1e-6);
    }

    #[test]
    fn axis_default_is_zero() {
        let g = GamepadState::new();
        assert_eq!(g.axis(GamepadAxis::RightStickY), 0.0);
    }

    #[test]
    fn lua_keys_are_distinct() {
        // No two enum variants may share a string key — Luau-side
        // collisions would silently overwrite. BTreeSet (not HashSet)
        // per ADR-0022.
        let buttons = [
            GamepadButton::South,
            GamepadButton::East,
            GamepadButton::West,
            GamepadButton::North,
            GamepadButton::LeftBumper,
            GamepadButton::RightBumper,
            GamepadButton::LeftTrigger,
            GamepadButton::RightTrigger,
            GamepadButton::Select,
            GamepadButton::Start,
            GamepadButton::Mode,
            GamepadButton::LeftStick,
            GamepadButton::RightStick,
            GamepadButton::DPadUp,
            GamepadButton::DPadDown,
            GamepadButton::DPadLeft,
            GamepadButton::DPadRight,
        ];
        let keys: BTreeSet<&str> = buttons.iter().map(|b| b.as_lua_key()).collect();
        assert_eq!(keys.len(), buttons.len());
    }
}
