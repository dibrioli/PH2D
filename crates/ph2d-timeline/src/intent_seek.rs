//! Which intents move the clock by hand (a seek), for every reader at once.

use crate::intent::TimelineIntent;

impl TimelineIntent {
    /// The hand moved the clock this frame — a DISCONTINUITY, not a crossing: the shell's drain
    /// (signals, auto-key) asks here, so a new seek joins every reader at once.
    #[must_use]
    pub fn is_seek(&self) -> bool {
        matches!(
            self,
            Self::Scrub(_) | Self::ScrubExact(_) | Self::SeekFrame(_)
        )
    }
}
