// ph2d-chrome-sync:z=300 (dispatch priority, ADR-0107; lower = earlier)
//! TopBar transport chips — **Play / Pause / Reset** drive the ONE clock
//! (`ph2d_core::Playhead`, W4.T7). The chrome layer can't touch the playhead
//! (it lives in the shell's `App`), so each chip raises
//! [`EditorAction::Transport`]; the shell drain in `render_loop::mod` calls
//! `playhead.play()` / `pause()` / `rewind()`. Physics, Motion, Timeline and
//! Flip all ride this same clock, so these three chips drive every
//! time-based subsystem at once.
//!
//! Before this, the chips were painted + registered (`topbar/mod.rs`) but the
//! click only printed their name — inert. This is the missing dispatch seam.
//!
//! Central wiring: ids `TOPBAR_PLAY_BUTTON` / `TOPBAR_PAUSE` / `TOPBAR_RESET`
//! (`ids/chrome/topbar.rs`) + the `TopBarCluster::play()` chips in
//! `screens/hero/fixture.rs` + registration in `topbar/mod.rs::populate`.
//! Mirror of `chrome::motion_toggle`.

use crate::action_bus::{EditorAction, TransportCmd};
use crate::ids;
use crate::interaction::WidgetEvent;
use crate::screens::hero::HeroScreen;

pub fn apply(hero: &mut HeroScreen, event: WidgetEvent) -> bool {
    let WidgetEvent::Click(id) = event else {
        return false;
    };
    let cmd = if id == ids::TOPBAR_PLAY_BUTTON {
        TransportCmd::Play
    } else if id == ids::TOPBAR_PAUSE {
        TransportCmd::Pause
    } else if id == ids::TOPBAR_RESET {
        TransportCmd::Reset
    } else {
        return false;
    };
    hero.bus.push(EditorAction::Transport(cmd));
    true
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::screens::hero::HeroScreen;
    use ph2d_a11y::NodeId;

    /// The seam gate: clicking each transport chip, driven through the WHOLE
    /// `dispatch_all` chain (not this handler in isolation), raises exactly
    /// its command. Proves the `ph2d-chrome-sync` regen actually wired
    /// `transport` in — and that each id maps to the right command.
    ///
    /// Mutation-tested: dropping any id branch (or the whole handler from the
    /// chain) leaves the click unconsumed / the action unraised → RED.
    #[test]
    fn clicking_a_transport_chip_raises_its_command_through_the_full_dispatch() {
        for (id, expected) in [
            (ids::TOPBAR_PLAY_BUTTON, TransportCmd::Play),
            (ids::TOPBAR_PAUSE, TransportCmd::Pause),
            (ids::TOPBAR_RESET, TransportCmd::Reset),
        ] {
            let mut hero = HeroScreen::new(NodeId(1));
            let consumed = super::super::dispatch_all(&mut hero, WidgetEvent::Click(id));
            assert!(
                consumed,
                "dispatch_all ignored the transport click for {id:?}"
            );
            let got: Vec<EditorAction> = hero.bus.drain().collect();
            assert_eq!(got.len(), 1, "expected exactly one action for {id:?}");
            assert!(
                matches!(got[0], EditorAction::Transport(c) if c == expected),
                "transport chip {id:?} raised the wrong command (expected {expected:?})"
            );
        }
    }
}
