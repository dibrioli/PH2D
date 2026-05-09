//! Apple Pencil events + state.
//!
//! **M8 stub.** No iPad shell exists yet, so this module declares
//! the types that the future shell will produce; sim systems can
//! already query `pencil_squeeze_force()` etc. and get `0.0`.
//! When the iPad shell lands, the only change is the shell
//! translating `UIPencilInteraction` callbacks into [`PencilEvent`]
//! and feeding them through [`PencilState::handle_event`] — no
//! API churn for downstream callers.

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum PencilEvent {
    /// Squeeze gesture reported by Apple Pencil 2+. Force normalized
    /// to `[0.0, 1.0]`. iPadOS may report `Began` (rising), `Changed`
    /// (held / pressure change), `Ended` (released) — squash to a
    /// single force value here; consumers that need phase can read
    /// the [`PencilState`] before/after.
    Squeeze { force: f32 },
    /// User double-tapped the side of the Pencil. iPadOS bubbles this
    /// as a discrete `UIPencilInteraction` callback — fire-and-forget.
    DoubleTap,
    /// Side button press (Pencil Pro). Phase is binary.
    SideButton(bool),
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum PencilButton {
    /// Pencil Pro side-button (squeeze-region tap).
    Side,
}

#[derive(Default, Debug, Clone)]
pub struct PencilState {
    squeeze_force: f32,
    last_double_tap_count: u64,
    side_button_held: bool,
}

impl PencilState {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn handle_event(&mut self, event: PencilEvent) {
        match event {
            PencilEvent::Squeeze { force } => {
                self.squeeze_force = force.clamp(0.0, 1.0);
            }
            PencilEvent::DoubleTap => {
                self.last_double_tap_count = self.last_double_tap_count.saturating_add(1);
            }
            PencilEvent::SideButton(held) => {
                self.side_button_held = held;
            }
        }
    }

    pub fn squeeze_force(&self) -> f32 {
        self.squeeze_force
    }
    pub fn double_tap_count(&self) -> u64 {
        self.last_double_tap_count
    }
    pub fn side_button_held(&self) -> bool {
        self.side_button_held
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn squeeze_clamps_and_persists() {
        let mut p = PencilState::new();
        assert_eq!(p.squeeze_force(), 0.0);
        p.handle_event(PencilEvent::Squeeze { force: 0.5 });
        assert!((p.squeeze_force() - 0.5).abs() < 1e-6);
        p.handle_event(PencilEvent::Squeeze { force: 1.7 });
        assert!((p.squeeze_force() - 1.0).abs() < 1e-6);
        p.handle_event(PencilEvent::Squeeze { force: -0.2 });
        assert!(p.squeeze_force() == 0.0);
    }

    #[test]
    fn double_tap_count_monotonic() {
        let mut p = PencilState::new();
        assert_eq!(p.double_tap_count(), 0);
        p.handle_event(PencilEvent::DoubleTap);
        p.handle_event(PencilEvent::DoubleTap);
        p.handle_event(PencilEvent::DoubleTap);
        assert_eq!(p.double_tap_count(), 3);
    }

    #[test]
    fn side_button_toggles() {
        let mut p = PencilState::new();
        assert!(!p.side_button_held());
        p.handle_event(PencilEvent::SideButton(true));
        assert!(p.side_button_held());
        p.handle_event(PencilEvent::SideButton(false));
        assert!(!p.side_button_held());
    }
}
