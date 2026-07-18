//! **Where the content ends.**
//!
//! Split out of `doc.rs` under the 700-LOC workspace cap, and a unit in its own
//! right: transport (go-to-end, a freshly armed loop) and the panel's fit all ask
//! this one question, and they must get ONE family of answers or the ruler and the
//! playhead will disagree about the same timeline.

use super::TimelineDoc;

impl TimelineDoc {
    /// Where "the end" of the active clip is, in seconds: the authored clip
    /// duration, or the last keyframe if the animation runs past it.
    ///
    /// A fresh clip has duration `0` and `insert_key` never extends it, so the
    /// authored duration alone would pin "go to end" at `t = 0` for every
    /// hand-keyed animation. Transport (go-to-end, the default loop range) reads
    /// THIS, not `active_clip().duration()`.
    #[must_use]
    pub fn end_seconds(&self) -> f64 {
        self.clip_end_seconds(self.active_clip)
    }

    /// **Where the content THIS VIEW shows ends** — what "the end" means to
    /// go-to-end and to a freshly armed loop.
    ///
    /// One ruler, two contents (`ph2d-panel-timeline::tab`): on the Keys view the
    /// end is the active clip's last frame; on Arrange it is the last second any
    /// strip occupies. Asking the clip in Arrange sized both to the active CLIP's
    /// duration — and when the first strip plays that clip 1:1 from the top, that
    /// is exactly the first strip, so an armed loop bracketed one strip out of the
    /// set and looked deliberate doing it (Enio, 2026-07-16).
    ///
    /// Falls back to the clip when no lane holds a strip: a document nobody has
    /// arranged has no stack to bracket, and the clip is the timeline.
    #[must_use]
    pub fn view_end_seconds(&self, keys: bool) -> f64 {
        if keys {
            self.end_seconds()
        } else {
            self.stack_end_seconds()
                .unwrap_or_else(|| self.end_seconds())
        }
    }

    /// The last timeline second any strip occupies, across every lane, or `None`
    /// when the stack holds none.
    ///
    /// A strip's outward lead-in reaches BACKWARD, so it never moves this end.
    #[must_use]
    pub fn stack_end_seconds(&self) -> Option<f64> {
        self.stack
            .iter()
            .flat_map(|l| &l.strips)
            .map(|s| s.t_end)
            .fold(None, |acc: Option<f64>, t| {
                Some(acc.map_or(t, |m| m.max(t)))
            })
    }

    /// [`Self::end_seconds`] for any clip — what a strip placed on it is sized to.
    #[must_use]
    pub fn clip_end_seconds(&self, index: usize) -> f64 {
        let Some(named) = self.clips.get(index) else {
            return 0.0;
        };
        let last_key = named
            .clip
            .tracks()
            .iter()
            .filter_map(|(_, track)| track.keys().last())
            .map(|k| k.t.to_seconds())
            .fold(0.0_f64, f64::max);
        named.clip.duration().to_seconds().max(last_key)
    }
}
