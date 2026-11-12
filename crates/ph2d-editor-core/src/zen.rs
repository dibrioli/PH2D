//! [`ZenMode`] — workspace state for the canvas-first promise
//! (ADR-0023 §2 "Modo Zen").
//!
//! When `active`, the layout system collapses every zone except the
//! center (canvas), and `ph2d_editor_core::Layout::zen` flips true. UI
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

use std::time::{Duration, Instant};

#[derive(Copy, Clone, Debug, Default)]
pub struct ZenMode {
    pub active: bool,
    /// The last toggle: debounces a 4-finger tap that would fire on every event handler call. By the
    /// CLOCK, never by frames — the on-demand loop (B23.2) draws 4 frames/s at rest, and a frame count
    /// stretched the 0,5 s cooldown to ~4 s.
    last_toggle: Option<Instant>,
}

impl ZenMode {
    pub const COOLDOWN: Duration = Duration::from_millis(500);

    pub fn new() -> Self {
        Self::default()
    }

    pub fn is_active(&self) -> bool {
        self.active
    }

    /// Toggle, but only if the cooldown elapsed. Returns whether the toggle actually fired.
    pub fn try_toggle(&mut self) -> bool {
        self.try_toggle_at(Instant::now())
    }

    /// [`Self::try_toggle`] at `now` (the gates drive the clock).
    pub fn try_toggle_at(&mut self, now: Instant) -> bool {
        if self
            .last_toggle
            .is_some_and(|t| now.saturating_duration_since(t) < Self::COOLDOWN)
        {
            return false;
        }
        self.active = !self.active;
        self.last_toggle = Some(now);
        true
    }

    /// ⭐ **O modo foco, anunciado** — a porta ÚNICA do `Ctrl+Space`, de *Ver ▸ Modo foco* e do
    /// toque de 4 dedos (F5): alterna e diz ao artista o que aconteceu. `true` = alternou.
    pub fn toggle_announced(&mut self, toasts: &mut crate::ToastQueue) -> bool {
        if !self.try_toggle() {
            return false;
        }
        toasts.push(crate::Toast::info(ph2d_i18n::tr(if self.is_active() {
            "shell.input_handlers.zen_mode_on_zones"
        } else {
            "shell.input_handlers.zen_mode_off_zones"
        })));
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
    fn second_toggle_blocked_by_cooldown() {
        let (mut z, t0) = (ZenMode::new(), Instant::now());
        assert!(z.try_toggle_at(t0)); // ON
        assert!(!z.try_toggle_at(t0 + Duration::from_millis(100))); // immediate retry blocked
        assert!(z.is_active());
    }

    /// B23.2: the cooldown is half a second of CLOCK, whatever the frame rate — nothing ticks per frame.
    #[test]
    fn o_cooldown_do_zen_e_pelo_relogio_nao_por_quadros() {
        let (mut z, t0) = (ZenMode::new(), Instant::now());
        assert!(z.try_toggle_at(t0));
        assert!(!z.try_toggle_at(t0 + ZenMode::COOLDOWN - Duration::from_millis(1)));
        assert!(z.try_toggle_at(t0 + ZenMode::COOLDOWN));
        assert!(!z.is_active());
    }
}
