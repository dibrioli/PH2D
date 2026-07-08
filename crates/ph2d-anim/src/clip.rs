//! [`Clip`] — a named collection of [`Track`]s bound to opaque targets.
//!
//! A clip is what the general app timeline **and** the future `motion.clip`
//! node sample at the playhead. It never resolves what a target *is* — that
//! mapping (key → setter) belongs to the consumer, which is exactly what keeps
//! `ph2d-anim` app-general and isolated.

use ph2d_vector_traits::{AnimValue, AttributeEvaluator};
use serde::{Deserialize, Serialize};

use crate::time::RationalTime;
use crate::track::Track;

/// An **opaque** identity for an animation target.
///
/// `ph2d-anim` treats this as a meaningless key (HR-8: opaque handles). The
/// consumer decides whether it names a sprite's rotation, a layer's opacity, a
/// node param, or a vector property — the same [`Clip`] can drive anything.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub struct AnimTarget(u64);

impl AnimTarget {
    /// Wrap a raw target id.
    #[must_use]
    pub const fn new(id: u64) -> Self {
        Self(id)
    }

    /// The raw target id.
    #[must_use]
    pub const fn get(self) -> u64 {
        self.0
    }
}

impl From<u64> for AnimTarget {
    fn from(id: u64) -> Self {
        Self(id)
    }
}

/// A named collection of tracks plus a total duration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Clip {
    tracks: Vec<(AnimTarget, Track)>,
    duration: RationalTime,
}

impl Clip {
    /// An empty clip with the given duration.
    #[must_use]
    pub fn new(duration: RationalTime) -> Self {
        Self {
            tracks: Vec::new(),
            duration,
        }
    }

    /// Builder-style: attach a track for a target and return `self`.
    #[must_use]
    pub fn with_track(mut self, target: AnimTarget, track: Track) -> Self {
        self.tracks.push((target, track));
        self
    }

    /// Attach a track for a target.
    pub fn push(&mut self, target: AnimTarget, track: Track) {
        self.tracks.push((target, track));
    }

    /// The clip's total duration.
    #[must_use]
    pub fn duration(&self) -> RationalTime {
        self.duration
    }

    /// All `(target, track)` pairs.
    #[must_use]
    pub fn tracks(&self) -> &[(AnimTarget, Track)] {
        &self.tracks
    }

    /// The track bound to `target`, if any (first match).
    #[must_use]
    pub fn track(&self, target: AnimTarget) -> Option<&Track> {
        self.tracks
            .iter()
            .find(|(t, _)| *t == target)
            .map(|(_, track)| track)
    }

    /// Sample the track bound to `target` at time `t` (seconds), or `None` if no
    /// track is bound to that target.
    #[must_use]
    pub fn sample(&self, target: AnimTarget, t: f64) -> Option<AnimValue> {
        self.track(target).map(|track| track.sample(t))
    }

    /// `true` if the clip has no tracks.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.tracks.is_empty()
    }

    /// Number of tracks.
    #[must_use]
    pub fn len(&self) -> usize {
        self.tracks.len()
    }
}
