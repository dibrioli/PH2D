//! Why a key was not written where the animator asked for it (ADR-0115 R9).
//!
//! Under a clip stack, "key it here" can genuinely have **no answer**. The
//! document's job is to refuse — writing a key that quietly moves the object is
//! the one outcome that is never acceptable — but a refusal the animator cannot
//! see is indistinguishable from a bug: they drag, the object snaps back, and
//! nothing says why.
//!
//! So the refusal is a **value**, not an absence. It travels from the evaluator
//! (which is the only place that knows the reason) to the shell (which is the
//! only place that can speak), and the shell says it out loud.

/// The three ways "key it here" can fail to name a place in the clip being edited.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum KeyRefusal {
    /// The clip being edited is **not playing** at this instant: no strip of it
    /// covers the playhead, so "here" is nowhere in it.
    NotPlaying,
    /// It is playing **more than once**: "here" names two places in the clip, and
    /// picking one silently would drop the key somewhere the animator never looked.
    PlaysTwice,
    /// A lane above **owns the channel**: the blend is insensitive to what this
    /// clip stores (an `Override` lane at full weight), so no value in it produces
    /// the pose that was asked for. The affine solve is degenerate — `A ~ 0`.
    Overridden,
}

impl KeyRefusal {
    /// One line, for the animator. English by canon
    /// ([[feedback_app_ui_english_only]]); each says what happened AND what the
    /// stack is doing, because "can't key here" without a reason is only slightly
    /// better than silence.
    #[must_use]
    pub fn message(self) -> &'static str {
        match self {
            Self::NotPlaying => "Can't key: the clip you are editing does not play here",
            Self::PlaysTwice => "Can't key: the clip you are editing plays twice here",
            Self::Overridden => "Can't key: a lane above overrides this clip here",
        }
    }
}
