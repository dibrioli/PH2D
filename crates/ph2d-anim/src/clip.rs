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
///
/// **Invariant: `tracks` is sorted by `AnimTarget`.** Every lookup is a binary
/// search, because the apply resolves one track per binding, once per frame:
/// with a linear scan that is O(bindings x tracks), and since a binding creates
/// a track, the two grow together — a quadratic that a clip stack would have
/// turned cubic (ADR-0115 §4). Nothing reads `tracks()` in insertion order (the
/// one consumer folds a max over it), so ordering by target costs nothing.
///
/// The invariant is upheld by every mutator here **and on deserialization** — a
/// document saved before the invariant existed can hold an unsorted vec, and a
/// binary search over it would miss tracks silently, killing the animation.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Clip {
    #[serde(deserialize_with = "de_sorted_tracks")]
    tracks: Vec<(AnimTarget, Track)>,
    duration: RationalTime,
}

/// Restore the sorted-by-target invariant on load (see [`Clip`]).
fn de_sorted_tracks<'de, D>(de: D) -> Result<Vec<(AnimTarget, Track)>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let mut tracks: Vec<(AnimTarget, Track)> = Vec::deserialize(de)?;
    tracks.sort_by_key(|(t, _)| *t);
    Ok(tracks)
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
        self.push(target, track);
        self
    }

    /// Attach a track for a target, keeping the tracks sorted (see [`Clip`]).
    pub fn push(&mut self, target: AnimTarget, track: Track) {
        let at = self.slot(target).unwrap_or_else(|i| i);
        self.tracks.insert(at, (target, track));
    }

    /// The index of `target`'s track, or `Err(insertion point)`.
    ///
    /// `partition_point` rather than `binary_search_by_key` so a duplicate target
    /// resolves to the FIRST of its run — the "first match" the linear scan gave.
    fn slot(&self, target: AnimTarget) -> Result<usize, usize> {
        let i = self.tracks.partition_point(|(t, _)| *t < target);
        match self.tracks.get(i) {
            Some((t, _)) if *t == target => Ok(i),
            _ => Err(i),
        }
    }

    /// The clip's total duration.
    #[must_use]
    pub fn duration(&self) -> RationalTime {
        self.duration
    }

    /// Set the clip's authored duration. Keys past it are kept (sampling
    /// flat-clamps); a shorter clip simply stops looping earlier.
    pub fn set_duration(&mut self, duration: RationalTime) {
        self.duration = duration;
    }

    /// All `(target, track)` pairs.
    #[must_use]
    pub fn tracks(&self) -> &[(AnimTarget, Track)] {
        &self.tracks
    }

    /// The track bound to `target`, if any (first match).
    #[must_use]
    pub fn track(&self, target: AnimTarget) -> Option<&Track> {
        let i = self.slot(target).ok()?;
        Some(&self.tracks[i].1)
    }

    /// Re-derive every roving key's time in every track
    /// ([`Track::resolve_roving`]). Idempotent; call after any batch of edits.
    pub fn resolve_roving(&mut self) {
        for (_, track) in &mut self.tracks {
            track.resolve_roving();
        }
    }

    /// The track bound to `target` for mutation (first match).
    pub fn track_mut(&mut self, target: AnimTarget) -> Option<&mut Track> {
        let i = self.slot(target).ok()?;
        Some(&mut self.tracks[i].1)
    }

    /// The track bound to `target`, inserting an empty one (via `make`) if none
    /// exists. Used by the document's authoring path to lazily create a track
    /// on first key.
    pub fn track_or_insert(
        &mut self,
        target: AnimTarget,
        make: impl FnOnce() -> Track,
    ) -> &mut Track {
        let idx = match self.slot(target) {
            Ok(i) => i,
            Err(at) => {
                self.tracks.insert(at, (target, make()));
                at
            }
        };
        &mut self.tracks[idx].1
    }

    /// Remove the track bound to `target`, returning `true` if one was removed.
    pub fn remove_track(&mut self, target: AnimTarget) -> bool {
        let before = self.tracks.len();
        self.tracks.retain(|(t, _)| *t != target);
        self.tracks.len() != before
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
