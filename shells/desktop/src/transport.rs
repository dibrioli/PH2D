//! Apply a TopBar transport command to the ONE clock (`Playhead`, W4.T7).
//!
//! The shell owns the `Playhead`; editor-core only *raises*
//! `EditorAction::Transport` (it can't touch the clock). This is the single
//! door that maps the command to playhead ops, so the button and any future
//! keyboard shortcut can't diverge ([[feedback_two_doors_to_the_same_question_diverge]]).
//!
//! Physics, Motion, Timeline and Flip all ride this clock, so one command
//! moves every time-based subsystem at once.

use ph2d_core::Playhead;
use ph2d_editor::action_bus::TransportCmd;

/// Drive the clock from a transport command.
pub(crate) fn apply(cmd: TransportCmd, playhead: &mut Playhead) {
    match cmd {
        TransportCmd::Play => playhead.play(),
        TransportCmd::Pause => playhead.pause(),
        // `rewind()` alone KEEPS the play state (Playhead doc), so pause
        // explicitly: Reset means "back to the start, stopped".
        TransportCmd::Reset => {
            playhead.rewind();
            playhead.pause();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ph() -> Playhead {
        Playhead::new(1.0 / 60.0)
    }

    #[test]
    fn play_starts_the_clock() {
        let mut p = ph();
        p.pause(); // a fresh Playhead defaults to playing (the app pauses it at boot)
        assert!(!p.is_playing());
        apply(TransportCmd::Play, &mut p);
        assert!(p.is_playing(), "Play did not start the clock");
    }

    #[test]
    fn pause_stops_the_clock() {
        let mut p = ph();
        p.play();
        apply(TransportCmd::Pause, &mut p);
        assert!(!p.is_playing(), "Pause did not stop the clock");
    }

    /// The mutation target: `rewind()` alone keeps the play state, so a
    /// Reset that forgets to `pause()` leaves the clock rolling, and a Reset
    /// that forgets to `rewind()` leaves the time where it was.
    #[test]
    fn reset_rewinds_to_start_and_stops() {
        let mut p = ph();
        p.play();
        p.seek(1.5);
        assert!(p.time() > 0.0 && p.is_playing());
        apply(TransportCmd::Reset, &mut p);
        assert_eq!(p.time(), 0.0, "Reset did not rewind to the start");
        assert!(
            !p.is_playing(),
            "Reset left the clock playing (rewind keeps the play state — pause is required)"
        );
    }
}
