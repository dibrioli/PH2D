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
    /// An **authored duration wins over the derived end** (the AE composition-
    /// duration model, Enio 2026-07-23): the scene's `scene_length` here, a clip's
    /// `length_override` in [`Self::clip_end_seconds`], a container's in
    /// [`Self::container_length_seconds`]. Content past it stays authored and is
    /// CUT from playback ([`Self::cut_source`]).
    #[must_use]
    pub fn view_end_seconds(&self, keys: bool) -> f64 {
        if keys {
            self.end_seconds()
        } else {
            self.scene_length.unwrap_or_else(|| {
                self.stack_end_seconds()
                    .unwrap_or_else(|| self.end_seconds())
            })
        }
    }

    /// **Which authored duration CLOSES this view** — `Some(end)` iff the view has an
    /// EXPLICIT duration (the AE composition end that darkens the dead zone and pins
    /// the playhead), `None` when the end is merely derived and the timeline stays
    /// open-ended.
    ///
    /// This is the *authored* companion of [`Self::view_end_seconds`], and it MUST
    /// fall through the same way that value does, or the two disagree about the same
    /// timeline. In particular **without a stack the clip IS the timeline**
    /// (`ruler_clock`: both tabs rule the one clock there is), so a clip's authored
    /// `length_override` closes the scene view too — the exact case the veil and the
    /// playhead clamp missed by keying on `keys_mode`, which the panel publishes as
    /// `shows_keys() && stacked()` and so reads FALSE on the Keys tab when nothing is
    /// arranged. The Dur(s) box (via `clip_end_seconds`) already showed the right
    /// number; only the two consumers that decide *is it authored* were asking a
    /// different door ([[feedback_two_doors_to_the_same_question_diverge]], Enio
    /// 2026-07-23).
    ///
    /// - `container`: `Some(c)` inside a container's editing view → its own override.
    /// - `keys_mode`: the panel's published flag (soloing a clip out of a stack).
    #[must_use]
    pub fn view_authored_end(&self, container: Option<usize>, keys_mode: bool) -> Option<f64> {
        if let Some(c) = container {
            return self.container_length_override(c);
        }
        if keys_mode {
            return self.clip_length_override(self.active_clip);
        }
        // Arrange / no-solo: the scene's authored length, or — when NO lane holds a
        // strip — the active clip's (the clip is the timeline). With a real stack
        // `stack_end_seconds` is `Some`, so the clip override never leaks in.
        self.scene_length.or_else(|| {
            self.stack_end_seconds()
                .is_none()
                .then(|| self.clip_length_override(self.active_clip))
                .flatten()
        })
    }

    /// The last timeline second any strip occupies, across every lane, or `None`
    /// when the stack holds none.
    ///
    /// A strip's outward lead-in reaches BACKWARD, so it never moves this end — but the
    /// outward lead-OUT reaches FORWARD past `t_end`, and it is content (the pose is
    /// still travelling), so the end is [`crate::ClipStrip::lead_end`]. An end that
    /// stopped at `t_end` made a freshly armed loop bracket the fade out of itself, and
    /// the travel the artist authored never played (Enio, 2026-07-20).
    #[must_use]
    pub fn stack_end_seconds(&self) -> Option<f64> {
        self.host_end_seconds(crate::StackHost::Document)
    }

    /// [`Self::stack_end_seconds`] for ANY stack — the document's, or a container's interior
    /// (ADR-0133).
    ///
    /// The scene's answer delegates here rather than the other way round: "where does this
    /// stack end" is one question, and a container answering it by a second rule is how an
    /// interior's ruler comes to disagree with the scene's about the same strips. `None` for
    /// a host that holds no strip — or that does not exist.
    #[must_use]
    pub fn host_end_seconds(&self, host: crate::StackHost) -> Option<f64> {
        self.host_stack(host)?
            .iter()
            .flat_map(|l| &l.strips)
            .map(crate::ClipStrip::lead_end)
            .fold(None, |acc: Option<f64>, t| {
                Some(acc.map_or(t, |m| m.max(t)))
            })
    }

    /// [`Self::end_seconds`] for any clip — what a strip placed on it is sized to.
    /// An authored [`crate::NamedClip::length_override`] IS the end, wherever the
    /// content lies (shorter cuts, longer extends — both are the point).
    #[must_use]
    pub fn clip_end_seconds(&self, index: usize) -> f64 {
        let Some(named) = self.clips.get(index) else {
            return 0.0;
        };
        if let Some(len) = named.length_override {
            return len;
        }
        let last_key = named
            .clip
            .tracks()
            .iter()
            .filter_map(|(_, track)| track.keys().last())
            .map(|k| k.t.to_seconds())
            .fold(0.0_f64, f64::max);
        named.clip.duration().to_seconds().max(last_key)
    }

    /// **How long container `index` is, in seconds — the ONE door** (Enio,
    /// 2026-07-23). An authored [`crate::NamedContainer::length_override`] wins;
    /// otherwise the interior's extent through [`crate::container_bar_seconds`]
    /// (an empty interior is 2 s). Before this door existed, FOUR call sites
    /// composed the same two functions by hand — the snapshot's bar, the bridge's
    /// loop brace, `source_length`'s slice window and the unplaced ruler's axis —
    /// which is exactly the drift the override would have had to be added to
    /// four times. `0.0` for a container that does not exist (callers refuse it).
    #[must_use]
    pub fn container_length_seconds(&self, index: usize) -> f64 {
        let Some(c) = self.containers().get(index) else {
            return 0.0;
        };
        c.length_override.unwrap_or_else(|| {
            crate::container_bar_seconds(
                self.host_end_seconds(crate::StackHost::Container(index))
                    .unwrap_or(0.0),
            )
        })
    }

    /// **The cut**: the latest instant of `source`'s own clock that PLAYS, given
    /// its authored duration — or `t` untouched when none is authored. This is
    /// what "an explicit duration cuts the excess" means at the evaluator: every
    /// clock handed to a strip's interior or tracks passes through here
    /// (`stack_frames`), so content past the cut holds the cut's pose instead of
    /// playing on — non-destructively, the keys and strips stay authored.
    #[must_use]
    pub fn cut_source(&self, source: crate::StripSource, t: f64) -> f64 {
        match source {
            crate::StripSource::Clip(i) => self.clip_cut(i as usize, t),
            crate::StripSource::Container(i) => self.container_cut(i as usize, t),
        }
    }

    /// [`Self::cut_source`] by clip index — also the solo (no-stack) path's cut.
    #[must_use]
    pub fn clip_cut(&self, index: usize, t: f64) -> f64 {
        self.clips
            .get(index)
            .and_then(|c| c.length_override)
            .map_or(t, |len| t.min(len))
    }

    /// [`Self::cut_source`] by container index — also the rooted frame 0's cut.
    #[must_use]
    pub fn container_cut(&self, index: usize, t: f64) -> f64 {
        self.containers()
            .get(index)
            .and_then(|c| c.length_override)
            .map_or(t, |len| t.min(len))
    }

    /// [`Self::cut_source`] for the SCENE's own clock (`scene_length`) — frame 0
    /// of an un-rooted scratch.
    #[must_use]
    pub fn cut_scene(&self, t: f64) -> f64 {
        self.scene_length.map_or(t, |len| t.min(len))
    }

    /// Clip `index`'s AUTHORED duration, if any — `None` means derived. The
    /// readers that must distinguish "authored" from "derived" (the beyond-end
    /// shade, the playhead clamp) ask this; everything that just wants "the end"
    /// asks [`Self::clip_end_seconds`].
    #[must_use]
    pub fn clip_length_override(&self, index: usize) -> Option<f64> {
        self.clips.get(index).and_then(|c| c.length_override)
    }

    /// Container `index`'s AUTHORED duration, if any — sibling of
    /// [`Self::clip_length_override`].
    #[must_use]
    pub fn container_length_override(&self, index: usize) -> Option<f64> {
        self.containers().get(index).and_then(|c| c.length_override)
    }

    /// Author clip `index`'s explicit duration (`None` clears it — back to the
    /// derived end). Non-positive values clear too: a zero-length clip is a
    /// timeline nobody can grab, and 0 is the numeric box's "clear" gesture.
    pub fn set_clip_length_override(&mut self, index: usize, len: Option<f64>) {
        if let Some(c) = self.clips.get_mut(index) {
            c.length_override = len.filter(|l| *l > 0.0);
        }
    }

    /// Author container `index`'s explicit duration — same contract as the clip's.
    pub fn set_container_length_override(&mut self, index: usize, len: Option<f64>) {
        if let Some(c) = self.containers_mut().get_mut(index) {
            c.length_override = len.filter(|l| *l > 0.0);
        }
    }

    /// Author the SCENE's explicit duration — same contract as the clip's.
    pub fn set_scene_length(&mut self, len: Option<f64>) {
        self.scene_length = len.filter(|l| *l > 0.0);
    }
}
