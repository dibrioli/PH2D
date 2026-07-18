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
use crate::stack::ClipLane;
use crate::stack_eval::StackScratch;

/// On-disk schema version for the timeline document (HR-14). Written explicitly
/// as the first field (never trust `serde(default)` under a positional format).
/// v2: tracks carry per-key roving flags (`TrackData.roving`, appended field —
/// postcard is positional, so a v1 blob is rejected rather than misread).
/// v3: each clip carries its own loop (`NamedClip.loop_range` + `loop_ping_pong`,
/// appended) — a loop belongs to the animation it brackets, not to the document.
/// v4: the clip **stack** (`TimelineDoc.stack`) and each binding's captured
/// `rest` value (ADR-0115). Both appended; a document with an empty stack behaves
/// byte-for-byte as it did in v3.
/// v5: each clip carries a SECOND loop — one per view
/// (`NamedClip.keys_loop_range`/`keys_loop_ping_pong`, appended). The Arrange tab
/// loops the timeline (the original `loop_range`); the Keys tab loops the clip's own
/// clock, independently (Enio, 2026-07-16). Appended; both `None`/`false` behaves
/// exactly as v4.
/// v6: a strip's fade-in can reach OUTWARD into the gap before it
/// (`ClipStrip.lead_in`, appended) — the travel fade (Enio, 2026-07-16). `0.0` is
/// the old behaviour byte-for-byte.
/// v7: each strip remembers **what its four corners last did** (`ClipStrip.marks`,
/// appended) — the change bars the panel draws over a trim or a stretch (Enio,
/// 2026-07-16). All-zero is the old behaviour, and zero draws nothing.
pub const DOC_VERSION: u32 = 7;

/// The default display frame rate for a fresh document.
pub const DEFAULT_FPS: f64 = 24.0;

/// How many clips a document may hold.
///
/// A real bound, not a guess at what an animator needs: the clip selector is a
/// dropdown, and a dropdown's option ids are a FIXED array of `NodeId`s
/// (`TIMELINE_CLIP_OPT`) — the chrome has no way to mint a hit id at runtime. So
/// the cap is whatever that array is, and it lives HERE, with the data, where
/// [`TimelineDoc::add_clip`] can refuse rather than let the UI silently drop a
/// clip it cannot address. Raising it means growing the id array in lockstep, and
/// a gate holds the two together.
pub const MAX_CLIPS: usize = 16;

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
    /// This clip's `[start, end)` loop range in seconds, if the animator set one.
    ///
    /// **Per clip, not per document** (Enio, 2026-07-12): a loop is a property of
    /// the animation it brackets — "walk" cycles over its own two seconds and
    /// "run" over its own — so one range shared across every clip was simply the
    /// wrong range for all but the one it was drawn on. The `Playhead` still owns
    /// the LIVE loop (it is what wraps the transport); this is where each clip
    /// parks its own, and switching clips swaps it in.
    ///
    /// Appended field — postcard is positional, hence `DOC_VERSION` 2 -> 3.
    pub loop_range: Option<(f64, f64)>,
    /// `true` when this clip's loop **ping-pongs** (plays back and forth) instead
    /// of wrapping. Rides with the range because it IS part of it: a loop is a
    /// span plus what happens at its end. Appended (v3).
    pub loop_ping_pong: bool,
    /// The **Keys view's** loop — this clip's own two seconds, in the clip's OWN
    /// clock, looped independently of the Arrange loop above (Enio, 2026-07-16).
    ///
    /// The two are different clocks with different braces: [`Self::loop_range`]
    /// wraps the TIMELINE playhead (the stack the Arrange tab shows), this one wraps
    /// the CLIP playhead the Keys tab scrubs while you author keys. Setting one never
    /// touches the other — the loop area is independent per clip AND per view. Both
    /// appended (`DOC_VERSION` 4 -> 5).
    pub keys_loop_range: Option<(f64, f64)>,
    /// Mirror of [`Self::loop_ping_pong`] for the Keys-view loop. Appended (v5).
    pub keys_loop_ping_pong: bool,
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
    /// Monotonic strip-id allocator (see `StripId`). Appended (v4).
    next_strip: u64,
    /// The clip stack: lanes of clip instances, bottom to top (ADR-0115).
    ///
    /// **Empty is the default and is not a degenerate case** — an empty stack
    /// means "the active clip drives the scene", which is what this document did
    /// before the stack existed, on the same code path and to the same bytes.
    /// Appended field (v4).
    stack: Vec<ClipLane>,
    /// This frame's live strips and their per-entity clocks (`stack_eval.rs`).
    /// Runtime scratch, not document identity: never serialized, always compares
    /// equal, and its buffers are retained frame to frame (zero-alloc, HR-3).
    #[serde(skip)]
    scratch: StackScratch,
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
                loop_range: None,
                loop_ping_pong: false,
                keys_loop_range: None,
                keys_loop_ping_pong: false,
            }],
            active_clip: 0,
            bindings: Vec::new(),
            markers: Vec::new(),
            next_target: 0,
            next_strip: 0,
            stack: Vec::new(),
            scratch: StackScratch::default(),
        }
    }

    /// Hand out a fresh strip identity. Monotonic and never reused: a stale drag
    /// or undo entry must resolve to "gone", never to somebody else's strip.
    pub(crate) fn alloc_strip_id(&mut self) -> crate::stack::StripId {
        self.next_strip += 1;
        crate::stack::StripId(self.next_strip)
    }

    /// The clip stack, bottom lane first. Empty = the active clip drives.
    #[must_use]
    pub fn stack(&self) -> &[ClipLane] {
        &self.stack
    }

    /// The clip stack, for editing.
    pub fn stack_mut(&mut self) -> &mut Vec<ClipLane> {
        &mut self.stack
    }

    /// Move this frame's scratch out for the apply to refill (it needs the
    /// document immutably while writing to the index). The buffers' capacity
    /// rides along, so [`Self::put_scratch`] returns them warm.
    pub(crate) fn take_scratch(&mut self) -> StackScratch {
        core::mem::take(&mut self.scratch)
    }

    /// Park the scratch back on the document, capacity and all.
    pub(crate) fn put_scratch(&mut self, scratch: StackScratch) {
        self.scratch = scratch;
    }

    /// This frame's resolved strips + clocks, as the apply left them. Key
    /// authoring runs AFTER the apply, on the same playhead, so what it reads
    /// here is exactly what the scene is showing.
    pub(crate) fn scratch(&self) -> &StackScratch {
        &self.scratch
    }

    /// **Make the stack's scratch describe `t`.** Idempotent, and free when it
    /// already does (the apply built it at this same `t` a moment ago).
    ///
    /// Every reader of the scratch — where a key lands, whether a pose is even
    /// reachable — is asking "what is the stack doing *now*", and was being
    /// answered "what was it doing when the apply last ran". In production those
    /// are the same instant, which is exactly what makes the coupling invisible
    /// and exactly the shape of bug that has broken this module three times over
    /// ([[feedback_derived_coordinate_seed_must_match_sample]]). A caller that
    /// authors keys calls this first and is simply right, whether or not an apply
    /// ran before it.
    pub fn prime_stack(&mut self, t: f64) {
        // Unconditional. A "has the time changed" guard would be blind to the half
        // of the dependency that matters more — the DOCUMENT changing at the same
        // instant (a strip moved, a clip deleted) — and a cache that is fresh for
        // one of its two inputs is a cache that lies. The rebuild is a scan of the
        // live strips; the apply already pays it once a frame.
        let mut scratch = self.take_scratch();
        scratch.rebuild(self, t);
        self.put_scratch(scratch);
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

    /// Append a clip named `name` and return its index. Refuses past
    /// [`MAX_CLIPS`] (the selector's option ids are a fixed array) and returns
    /// the active index unchanged.
    ///
    /// The new clip is **empty**, and that is the whole model: BINDINGS are
    /// document-wide (a binding maps an entity's property to a stable target id),
    /// so every clip animates the same objects and only the KEYS differ. A second
    /// clip therefore costs a name and nothing else — "walk" and "run" are two sets
    /// of curves over one rig, which is how After Effects and Unity both read it.
    pub fn add_clip(&mut self, name: String) -> usize {
        if self.clips.len() >= MAX_CLIPS {
            return self.active_clip;
        }
        self.clips.push(NamedClip {
            name,
            clip: Clip::new(RationalTime::from_seconds(0.0)),
            loop_range: None,
            loop_ping_pong: false,
            keys_loop_range: None,
            keys_loop_ping_pong: false,
        });
        self.clips.len() - 1
    }

    /// Rename clip `index` (out of range: no-op).
    pub fn rename_clip(&mut self, index: usize, name: String) {
        if let Some(c) = self.clips.get_mut(index) {
            c.name = name;
        }
    }

    /// **Copy clip `index`** — curves, loop and all — as a new clip at the end, and
    /// return its index. Refuses past [`MAX_CLIPS`], or on an index out of range.
    ///
    /// This is the "start from what I have" button, and it is the one thing
    /// [`Self::add_clip`] cannot be: bindings are document-wide, so a *new* clip is
    /// always empty — a variation ("walk" → "walk, tired") means copying the curves
    /// and editing them, and hand-copying every key is not an authoring workflow.
    ///
    /// The copy is **deep and independent**: the keys carry fresh [`ph2d_anim::KeyId`]s
    /// via [`Clip`]'s own clone, so editing the copy never reaches back into the
    /// original. Its loop travels too — a loop is a property of the animation it
    /// brackets, so a copy of the animation has the same one.
    pub fn duplicate_clip(&mut self, index: usize) -> Option<usize> {
        if self.clips.len() >= MAX_CLIPS {
            return None;
        }
        let src = self.clips.get(index)?;
        let copy = NamedClip {
            name: self.fresh_copy_name(&src.name),
            clip: src.clip.clone(),
            loop_range: src.loop_range,
            loop_ping_pong: src.loop_ping_pong,
            // Both loops travel — a copy of the animation has the same brackets in
            // both views.
            keys_loop_range: src.keys_loop_range,
            keys_loop_ping_pong: src.keys_loop_ping_pong,
        };
        self.clips.push(copy);
        Some(self.clips.len() - 1)
    }

    /// **Play clip `index` backwards**: every track of it mirrored inside
    /// `[0, clip_end_seconds(index)]`. Returns `false` on an index out of range.
    ///
    /// The pivot is the clip's **effective end** — the same door the strip sizing and
    /// go-to-end read ([`Self::clip_end_seconds`]), never `duration()`. A hand-keyed
    /// clip has an authored duration of `0`, so mirroring about *that* would fold every
    /// key onto the negative side of zero and the animation would vanish from the
    /// panel. (This is the two-doors bug that already shipped once here, as a 5-second
    /// clip in a 1-second strip.)
    ///
    /// Reversal is a **mirror, not a re-typing of key times**: each segment's shape
    /// travels with it ([`ph2d_anim::Track::reverse_about`]), so an ease-out stays an
    /// ease-out when read backwards.
    pub fn reverse_clip(&mut self, index: usize) -> bool {
        let span = self.clip_end_seconds(index);
        let Some(c) = self.clips.get_mut(index) else {
            return false;
        };
        c.clip.reverse_about(span);
        true
    }

    /// `"Walk copy"`, then `"Walk copy 2"`… — a name no clip is using yet.
    ///
    /// Two clips sharing a label make the dropdown unreadable and the rename
    /// ambiguous, which is the same reason [`Self::fresh_clip_name`] exists.
    fn fresh_copy_name(&self, of: &str) -> String {
        let first = format!("{of} copy");
        if !self.clips.iter().any(|c| c.name == first) {
            return first;
        }
        for n in 2..=MAX_CLIPS + 1 {
            let candidate = format!("{of} copy {n}");
            if !self.clips.iter().any(|c| c.name == candidate) {
                return candidate;
            }
        }
        first
    }

    /// Delete clip `index`, returning `true` if it went.
    ///
    /// **The last clip never goes** — a document must always have one to edit, and
    /// an empty `clips` would make `active_clip()` panic on the very next frame.
    /// The active index follows the deletion (it shifts down with the clips above
    /// it, and clamps if it WAS the deleted one), so the caller never has to
    /// repair it.
    pub fn remove_clip(&mut self, index: usize) -> bool {
        if self.clips.len() <= 1 || index >= self.clips.len() {
            return false;
        }
        self.clips.remove(index);
        // A strip names its clip by INDEX, and removing one slides every later
        // clip down: without this, every strip above the hole would quietly start
        // playing its neighbour. Strips of the deleted clip go with it.
        self.repoint_strips_after_clip_removal(index);
        if self.active_clip >= index {
            self.active_clip = self.active_clip.saturating_sub(1);
        }
        true
    }

    /// The active clip's loop range for one VIEW, if it has one. `keys` picks the
    /// Keys-view loop (the clip's own clock); `false` picks the Arrange/timeline
    /// loop. The two are stored and returned independently — the loop area is
    /// per clip AND per view (Enio, 2026-07-16).
    #[must_use]
    pub fn active_loop_for(&self, keys: bool) -> Option<(f64, f64)> {
        let c = &self.clips[self.active_clip];
        if keys {
            c.keys_loop_range
        } else {
            c.loop_range
        }
    }

    /// Park `range` on the ACTIVE clip's `keys`/Arrange loop. The caller mirrors it
    /// into the matching `Playhead` (clip clock in Keys, timeline in Arrange), which
    /// owns the live loop — this is the copy that survives a clip switch.
    pub fn set_active_loop_for(&mut self, keys: bool, range: Option<(f64, f64)>) {
        let c = &mut self.clips[self.active_clip];
        if keys {
            c.keys_loop_range = range;
        } else {
            c.loop_range = range;
        }
    }

    /// Whether the active clip's `keys`/Arrange loop ping-pongs.
    #[must_use]
    pub fn active_ping_pong_for(&self, keys: bool) -> bool {
        let c = &self.clips[self.active_clip];
        if keys {
            c.keys_loop_ping_pong
        } else {
            c.loop_ping_pong
        }
    }

    /// Set whether the active clip's `keys`/Arrange loop ping-pongs.
    pub fn set_active_ping_pong_for(&mut self, keys: bool, on: bool) {
        let c = &mut self.clips[self.active_clip];
        if keys {
            c.keys_loop_ping_pong = on;
        } else {
            c.loop_ping_pong = on;
        }
    }

    /// The active clip's Arrange (timeline) loop range — [`Self::active_loop_for`]`(false)`.
    /// The project-load path and older callers read the timeline loop through this.
    #[must_use]
    pub fn active_loop(&self) -> Option<(f64, f64)> {
        self.active_loop_for(false)
    }

    /// Park `range` on the active clip's Arrange loop.
    pub fn set_active_loop(&mut self, range: Option<(f64, f64)>) {
        self.set_active_loop_for(false, range);
    }

    /// Whether the active clip's Arrange loop ping-pongs.
    #[must_use]
    pub fn active_ping_pong(&self) -> bool {
        self.active_ping_pong_for(false)
    }

    /// Set whether the active clip's Arrange loop ping-pongs.
    pub fn set_active_ping_pong(&mut self, on: bool) {
        self.set_active_ping_pong_for(false, on);
    }

    /// A name no clip is using yet — `"Clip 2"`, then `"Clip 3"`… Seeds the
    /// selector's "New Clip" so two clips never share a label (which would make
    /// the dropdown unreadable and the rename ambiguous).
    #[must_use]
    pub fn fresh_clip_name(&self) -> String {
        for n in 2..=MAX_CLIPS + 1 {
            let candidate = format!("Clip {n}");
            if !self.clips.iter().any(|c| c.name == candidate) {
                return candidate;
            }
        }
        "Clip".to_string()
    }

    /// Serialize the document to the versioned on-disk format (postcard). The
    /// schema `version` is the first field, so a loader can reject or migrate an
    /// older file before trusting the rest (W4 save; HR-14).
    ///
    /// Bindings serialize by `wire_id`; the live `entity` bits are `#[serde(skip)]`
    /// — stamp the wire ids first (see [`crate::stamp_wire_ids`]).
    pub fn to_bytes(&self) -> Result<Vec<u8>, String> {
        postcard::to_allocvec(self).map_err(|e| e.to_string())
    }

    /// Load a document saved by [`Self::to_bytes`], rejecting a schema version
    /// this build does not understand. The loaded bindings have null `entity`
    /// bits — resolve them with [`crate::resolve_entities`].
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, String> {
        let doc: TimelineDoc = postcard::from_bytes(bytes).map_err(|e| e.to_string())?;
        if doc.version != DOC_VERSION {
            return Err(format!(
                "timeline schema version {} != {DOC_VERSION}",
                doc.version
            ));
        }
        Ok(doc)
    }

    /// The clip currently edited.
    #[must_use]
    pub fn active_clip(&self) -> &Clip {
        &self.clips[self.active_clip].clip
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
        // Auto-key reaches the document directly (no intent), so this is its
        // roving choke point: a re-keyed value shifts the derived times.
        track.resolve_roving();
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

    /// Move the marker at `index` to `t` (storage order is preserved, so the
    /// index stays valid across a drag). Returns `true` if it existed.
    pub fn move_marker(&mut self, index: usize, t: RationalTime) -> bool {
        match self.markers.get_mut(index) {
            Some(m) => {
                m.t = t;
                true
            }
            None => false,
        }
    }

    /// Relabel the marker at `index`. Returns `true` if it existed. Marker labels
    /// are user content, not HR-15 UI strings.
    pub fn set_marker_label(&mut self, index: usize, label: impl Into<String>) -> bool {
        match self.markers.get_mut(index) {
            Some(m) => {
                m.label = label.into();
                true
            }
            None => false,
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

/// **Where the content ends** — one question, four answers (clip / any clip /
/// stack / this view). A CHILD module (not a sibling) so it reads the document's
/// private fields, the idiom `Track`'s `rove` already uses.
#[path = "doc_extent.rs"]
mod extent;
