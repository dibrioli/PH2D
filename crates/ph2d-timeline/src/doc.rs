//! [`TimelineDoc`] — the app-general timeline **document**: the editable state
//! the panel drives and the project saves.
//!
//! It owns named [`Clip`]s, the [`TargetBinding`]s that say which scene object
//! each clip track drives, and [`Marker`]s. v1 edits a single clip ("Main");
//! multi-clip is data-ready (a `Vec`), UI deferred (W5). The document is the
//! authority; a [`crate::TimelineState`] wraps it with panel selection +
//! history.
//!
//! Targets are **allocated** here (opaque, HR-8) so two entities animating the
//! same [`PropKind`] get distinct tracks in one clip.

use ph2d_anim::{AnimTarget, AnimValue, Clip, Interp, KeyId, RationalTime, Track};
use serde::{Deserialize, Serialize};

use crate::binding::TargetBinding;
use crate::prop::PropKind;

/// On-disk schema version for the timeline document (HR-14). Written explicitly
/// as the first field (never trust `serde(default)` under a positional format).
pub const DOC_VERSION: u32 = 1;

/// The default display frame rate for a fresh document.
pub const DEFAULT_FPS: f64 = 24.0;

/// A named point in time on the timeline (UI in W4; data lives here from W1).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Marker {
    /// Marker position (drift-free rational time).
    pub t: RationalTime,
    /// Author-visible label (a raw string — markers are user content, not HR-15
    /// UI chrome).
    pub label: String,
}

/// A clip with an author-visible name.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct NamedClip {
    /// Author-visible clip name.
    pub name: String,
    /// The animation data.
    pub clip: Clip,
}

/// The editable timeline document (see module docs).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TimelineDoc {
    /// Schema version (first field on the wire).
    pub version: u32,
    /// Display frame rate for the ruler + frame readouts.
    pub fps_display: f64,
    clips: Vec<NamedClip>,
    active_clip: usize,
    bindings: Vec<TargetBinding>,
    markers: Vec<Marker>,
    /// Monotonic opaque-target allocator (see module docs).
    next_target: u64,
}

impl Default for TimelineDoc {
    fn default() -> Self {
        Self::new()
    }
}

impl TimelineDoc {
    /// A fresh document: one empty clip named "Main", 24 fps, no bindings.
    #[must_use]
    pub fn new() -> Self {
        Self {
            version: DOC_VERSION,
            fps_display: DEFAULT_FPS,
            clips: vec![NamedClip {
                name: "Main".to_string(),
                clip: Clip::new(RationalTime::from_seconds(0.0)),
            }],
            active_clip: 0,
            bindings: Vec::new(),
            markers: Vec::new(),
            next_target: 0,
        }
    }

    /// All clips.
    #[must_use]
    pub fn clips(&self) -> &[NamedClip] {
        &self.clips
    }

    /// The index of the clip currently edited.
    #[must_use]
    pub fn active_index(&self) -> usize {
        self.active_clip
    }

    /// Select which clip is edited (clamped to a valid index).
    pub fn set_active(&mut self, index: usize) {
        if index < self.clips.len() {
            self.active_clip = index;
        }
    }

    /// The clip currently edited.
    #[must_use]
    pub fn active_clip(&self) -> &Clip {
        &self.clips[self.active_clip].clip
    }

    /// Where "the end" of the active clip is, in seconds: the authored clip
    /// duration, or the last keyframe if the animation runs past it.
    ///
    /// A fresh clip has duration `0` and `insert_key` never extends it, so the
    /// authored duration alone would pin "go to end" at `t = 0` for every
    /// hand-keyed animation. Transport (go-to-end, the default loop range) reads
    /// THIS, not `active_clip().duration()`.
    #[must_use]
    pub fn end_seconds(&self) -> f64 {
        let clip = self.active_clip();
        let last_key = clip
            .tracks()
            .iter()
            .filter_map(|(_, track)| track.keys().last())
            .map(|k| k.t.to_seconds())
            .fold(0.0_f64, f64::max);
        clip.duration().to_seconds().max(last_key)
    }

    /// The clip currently edited, mutably.
    pub fn active_clip_mut(&mut self) -> &mut Clip {
        &mut self.clips[self.active_clip].clip
    }

    /// All document bindings.
    #[must_use]
    pub fn bindings(&self) -> &[TargetBinding] {
        &self.bindings
    }

    /// All document bindings, mutably (used by liveness + wire-id resolution).
    pub fn bindings_mut(&mut self) -> &mut [TargetBinding] {
        &mut self.bindings
    }

    /// The binding for a live `(entity, prop)`, if bound.
    #[must_use]
    pub fn binding_for(&self, entity: u64, prop: PropKind) -> Option<&TargetBinding> {
        self.bindings
            .iter()
            .find(|b| b.entity == entity && b.prop == prop)
    }

    /// The binding a target names, if any.
    #[must_use]
    pub fn binding(&self, target: AnimTarget) -> Option<&TargetBinding> {
        self.bindings.iter().find(|b| b.target == target)
    }

    /// Bind `(entity, prop)`, returning the (existing or freshly allocated)
    /// opaque target. Idempotent per `(entity, prop)`; does **not** create a
    /// track (that happens lazily on first key — see [`TimelineDoc::insert_key`]).
    pub fn bind(&mut self, entity: u64, prop: PropKind) -> AnimTarget {
        if let Some(b) = self.binding_for(entity, prop) {
            return b.target;
        }
        let target = AnimTarget::new(self.next_target);
        self.next_target += 1;
        self.bindings.push(TargetBinding::new(target, entity, prop));
        target
    }

    /// Remove a binding and its track from the active clip. Returns `true` if a
    /// binding was removed.
    pub fn unbind(&mut self, entity: u64, prop: PropKind) -> bool {
        let Some(pos) = self
            .bindings
            .iter()
            .position(|b| b.entity == entity && b.prop == prop)
        else {
            return false;
        };
        let target = self.bindings.remove(pos).target;
        self.active_clip_mut().remove_track(target);
        true
    }

    /// Insert a key on `(entity, prop)`'s track in the active clip, binding +
    /// creating the track if needed. Returns the target and the new key id.
    pub fn insert_key(
        &mut self,
        entity: u64,
        prop: PropKind,
        t: RationalTime,
        value: AnimValue,
        interp: Interp,
    ) -> (AnimTarget, KeyId) {
        let target = self.bind(entity, prop);
        let track = self
            .active_clip_mut()
            .track_or_insert(target, || Track::new(vec![]).with_default(value));
        let id = track.insert_key(t, value, interp);
        (target, id)
    }

    /// Like [`TimelineDoc::insert_key`] but **updates** the key at exactly `t`
    /// rather than stacking a duplicate — the capture-the-pose path (K /
    /// auto-key) upserts one key per playhead time.
    ///
    /// Re-keying an existing instant records the new pose and **keeps that key's
    /// interpolation**: nudging the sprite on canvas with auto-key armed must not
    /// silently undo the easing the author drew in the graph editor. `interp` is
    /// therefore only the default for a key this call creates.
    pub fn upsert_key(
        &mut self,
        entity: u64,
        prop: PropKind,
        t: RationalTime,
        value: AnimValue,
        interp: Interp,
    ) -> (AnimTarget, KeyId) {
        let target = self.bind(entity, prop);
        let track = self
            .active_clip_mut()
            .track_or_insert(target, || Track::new(vec![]).with_default(value));
        let id = track.upsert_value(t, value, interp);
        (target, id)
    }

    /// All markers (sorted by insertion; the panel sorts for display).
    #[must_use]
    pub fn markers(&self) -> &[Marker] {
        &self.markers
    }

    /// Add a marker; returns its index.
    pub fn add_marker(&mut self, t: RationalTime, label: impl Into<String>) -> usize {
        self.markers.push(Marker {
            t,
            label: label.into(),
        });
        self.markers.len() - 1
    }

    /// Remove the marker at `index`, returning `true` if it existed.
    pub fn remove_marker(&mut self, index: usize) -> bool {
        if index < self.markers.len() {
            self.markers.remove(index);
            true
        } else {
            false
        }
    }

    /// After loading, reseat the target allocator past every bound target so new
    /// bindings never collide with loaded ones (defensive against hand-edited
    /// or older files).
    pub fn reseat_allocator(&mut self) {
        let max = self.bindings.iter().map(|b| b.target.get()).max();
        if let Some(m) = max {
            self.next_target = self.next_target.max(m + 1);
        }
    }
}
