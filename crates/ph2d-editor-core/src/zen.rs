//! [`ZenMode`] — workspace state for the canvas-first promise
//! (ADR-0023 §2 "Modo Zen").
//!
//! When `active`, the layout system collapses every zone except the
//! center (canvas), and `ph2d_editor::Layout::zen` flips true. UI
//! widgets check this and skip rendering. Only a discrete corner
//! indicator remains (per Procreate behavior).
//!
//! Toggle paths (per ADR-0023 §4):
//! - Touch: 4-finger tap
//! - Keyboard: Tab (default; remappable per §4 customization)
//! - Mouse: Q (configurable)
//!
//! M12 skeleton ships [`ZenMode::try_toggle`]. The shell wires any
//! input source (gilrs key, winit Tab, gesture recognizer) into a
//! single call. ph2d-input dep lands when M8 merges (PR #13 in
//! review).

#[derive(Copy, Clone, Debug, Default)]
pub struct ZenMode {
    pub active: bool,
    /// Frames since last toggle. Useful to debounce a 4-finger tap
    /// from auto-firing on every event handler call. Reset to 0
    /// after a toggle; counter increments via [`ZenMode::tick`].
    cooldown_frames: u32,
}

impl ZenMode {
    pub const COOLDOWN_FRAMES: u32 = 30; // ~0.5 s @ 60 Hz

    pub fn new() -> Self {
        Self::default()
    }

    pub fn is_active(&self) -> bool {
        self.active
    }

    /// Increment cooldown counter once per frame. Called by the
    /// shell's per-frame loop.
    pub fn tick(&mut self) {
        self.cooldown_frames = self.cooldown_frames.saturating_add(1);
    }

    /// Toggle, but only if the cooldown elapsed. Returns whether the
    /// toggle actually fired.
    pub fn try_toggle(&mut self) -> bool {
        if self.cooldown_frames < Self::COOLDOWN_FRAMES {
            return false;
        }
        self.active = !self.active;
        self.cooldown_frames = 0;
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_inactive() {
        let z = ZenMode::new();
        assert!(!z.is_active());
    }

    #[test]
    fn cooldown_blocks_immediate_toggle() {
        let mut z = ZenMode::new();
        // Cooldown is 30 frames; at frame 0, toggle should NOT fire.
        assert!(!z.try_toggle());
        assert!(!z.is_active());
    }

    #[test]
    fn toggle_after_cooldown() {
        let mut z = ZenMode::new();
        for _ in 0..ZenMode::COOLDOWN_FRAMES {
            z.tick();
        }
        assert!(z.try_toggle());
        assert!(z.is_active());
    }

    #[test]
    fn second_toggle_blocked_by_cooldown() {
        let mut z = ZenMode::new();
        for _ in 0..ZenMode::COOLDOWN_FRAMES {
            z.tick();
        }
        assert!(z.try_toggle()); // ON
        assert!(!z.try_toggle()); // immediate retry blocked
        assert!(z.is_active());
    }

    #[test]
    fn toggle_round_trips_after_two_cooldowns() {
        let mut z = ZenMode::new();
        for _ in 0..ZenMode::COOLDOWN_FRAMES {
            z.tick();
        }
        z.try_toggle();
        for _ in 0..ZenMode::COOLDOWN_FRAMES {
            z.tick();
        }
        z.try_toggle();
        assert!(!z.is_active());
    }
}
