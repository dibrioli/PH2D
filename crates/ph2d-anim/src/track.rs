//! [`Track`] — a keyframe store implementing [`AttributeEvaluator`].
//!
//! A track resolves one parameter over absolute time. Keys are stored sorted by
//! their [`RationalTime`]; [`Track::sample`] finds the active segment, normalizes
//! to `u ∈ [0, 1]`, applies the segment's [`Interp`], and blends the two keys'
//! values via [`ph2d_vector_traits::LinearInterp`].
//!
//! # Authoring identity
//!
//! Keys carry a stable [`KeyId`] (monotonic per track) so the editor can refer to
//! a key across re-sorts (moving a key changes its index, never its id). The id
//! lives in a slice parallel to the keys — [`Key`] itself stays plain sampleable
//! data. The authoring ops ([`Track::insert_key`], [`Track::move_key`], the bulk
//! ops, …) keep the two slices in lockstep and reset the playback cursor.
//!
//! # HR-3 (zero-alloc hot path)
//!
//! [`Track::sample`] never allocates (it touches only `keys` + the cursor, never
//! the id bookkeeping). Beyond the binary search it keeps a monotonic **playback
//! cursor** (a relaxed [`AtomicUsize`]) so forward playback is `O(1)` amortized.
//! The atomic keeps `Track` `Send + Sync` (the cursor is a pure hint — every read
//! is validated against `t` before use, so a racy load is corrected by a
//! binary-search fallback).

use core::sync::atomic::{AtomicUsize, Ordering};

use ph2d_vector_traits::{AnimValue, AttributeEvaluator};
use serde::{Deserialize, Serialize};

use crate::curve::{Interp, interpolate};
use crate::time::RationalTime;

#[path = "extrap.rs"]
mod extrap;
#[path = "track_buffer.rs"]
mod track_buffer;
#[path = "track_fit.rs"]
mod track_fit;
#[path = "track_reverse.rs"]
mod track_reverse;
pub use extrap::{Extrap, ExtrapSide};
pub use track_buffer::CurveSnapshot;

/// Stable identity of a key within its [`Track`] — monotonic, never reused, so
/// the editor can track a key across re-sorts. Distinct tracks may share id
/// values; a `KeyId` is only meaningful together with its owning track.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct KeyId(u64);

impl KeyId {
    /// Wrap a raw key id (mirror of [`crate::AnimTarget::new`]) — lets a UI layer
    /// that carried the id as a primitive reconstruct the typed id.
    #[must_use]
    pub const fn new(id: u64) -> Self {
        Self(id)
    }

    /// The raw id.
    #[must_use]
    pub const fn get(self) -> u64 {
        self.0
    }
}

/// The keys of a time range as fit input, PREPARED for the channel they carry.
/// Returned by [`Track::range_samples`].
#[derive(Debug, Clone, PartialEq)]
pub struct RangeSamples {
    /// The ids of the keys the fit will replace.
    pub ids: Vec<KeyId>,
    /// Their `(time, value)` samples: angle-unwrapped if the channel is angular,
    /// and low-passed.
    pub samples: Vec<(f64, f64)>,
}

/// One keyframe: a time, a value, and the interpolation leaving it.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Key {
    /// Absolute time of the key.
    pub t: RationalTime,
    /// Value at the key.
    #[serde(with = "crate::anim_value_serde")]
    pub value: AnimValue,
    /// Interpolation from this key toward the next.
    pub interp: Interp,
}

/// A keyframe store for a single animated parameter.
///
/// Sampling before the first key returns the first value; after the last key,
/// the last value (flat-clamped ends). An empty track returns [`Track::default_value`].
#[derive(Debug, Serialize, Deserialize)]
#[serde(from = "TrackData", into = "TrackData")]
pub struct Track {
    /// Keys sorted ascending by `t`.
    keys: Vec<Key>,
    /// Stable ids, parallel to `keys` (same index ⇒ same key).
    ids: Vec<KeyId>,
    /// Roving flags, parallel to `keys` — a roving key's TIME is derived, not
    /// authored ([`Track::resolve_roving`]). Persistent CONTENT (unlike `ids`):
    /// it serializes and participates in equality. Kept as a parallel vec so
    /// [`Key`] stays a plain `(t, value, interp)` literal everywhere.
    roving: Vec<bool>,
    /// Next id to hand out (monotonic).
    next_id: u64,
    /// Returned when the track is empty (documented explicit default).
    default: AnimValue,
    /// Extrapolation BEFORE the first key (crown-jewels plan §6). Default
    /// [`Extrap::Hold`] is the flat-clamp — a Hold/Hold track never enters the
    /// extrapolation path, so it is byte-identical to the pre-feature engine
    /// (the fade fingerprint pin). Persistent CONTENT (like `roving`): serializes
    /// and participates in equality.
    pre: Extrap,
    /// Extrapolation AFTER the last key (independent of `pre` — the AE loopIn /
    /// loopOut, Unreal Pre/Post Infinity).
    post: Extrap,
    /// Monotonic playback hint (index of the last active segment).
    cursor: AtomicUsize,
}

/// Content equality ignores the atomic playback `cursor` (a transient sampling
/// hint) and the session-local `ids`/`next_id` — two tracks are equal when they
/// hold the same keys + roving flags + default. Used by history dedup
/// ([`crate::TimelineHistory`]).
impl PartialEq for Track {
    fn eq(&self, other: &Self) -> bool {
        self.keys == other.keys
            && self.roving == other.roving
            && self.default == other.default
            && self.pre == other.pre
            && self.post == other.post
    }
}

/// Serde proxy for [`Track`] — persists keys + roving flags + default and
/// rebuilds through [`Track::new`] (fresh ids, reset cursor; the atomic cursor
/// + session-local ids never serialize).
#[derive(Serialize, Deserialize)]
struct TrackData {
    keys: Vec<Key>,
    #[serde(with = "crate::anim_value_serde")]
    default: AnimValue,
    /// Appended last (postcard positional). A payload shorter than `keys` pads
    /// with `false` on load — tolerant of a pre-roving producer.
    #[serde(default)]
    roving: Vec<bool>,
    /// Extrapolation before the first key. Appended (postcard positional), so
    /// this rides on `DOC_VERSION 13 -> 14`; default [`Extrap::Hold`] makes an
    /// old-shaped document re-read as the flat-clamp it always was.
    #[serde(default)]
    pre: Extrap,
    /// Extrapolation after the last key.
    #[serde(default)]
    post: Extrap,
}

impl From<Track> for TrackData {
    fn from(t: Track) -> Self {
        Self {
            keys: t.keys,
            default: t.default,
            roving: t.roving,
            pre: t.pre,
            post: t.post,
        }
    }
}

impl From<TrackData> for Track {
    fn from(d: TrackData) -> Self {
        let mut roving = d.roving;
        roving.resize(d.keys.len(), false);
        let mut t = Track::new(d.keys).with_default(d.default);
        t.roving = roving;
        t.pre = d.pre;
        t.post = d.post;
        t
    }
}

impl Track {
    /// Build a track from keys (sorted by time internally). Each key gets a fresh
    /// [`KeyId`] in sorted order.
    ///
    /// The empty-track default is the first key's value, or `AnimValue::Float(0.0)`
    /// when there are no keys. Override with [`Track::with_default`].
    #[must_use]
    pub fn new(mut keys: Vec<Key>) -> Self {
        keys.sort_by_key(|k| k.t);
        let default = keys.first().map_or(AnimValue::Float(0.0), |k| k.value);
        let ids = (0..keys.len() as u64).map(KeyId).collect();
        Self {
            next_id: keys.len() as u64,
            roving: vec![false; keys.len()],
            keys,
            ids,
            default,
            pre: Extrap::Hold,
            post: Extrap::Hold,
            cursor: AtomicUsize::new(0),
        }
    }

    /// A track that holds `value` for all time (single stepped key at `t = 0`).
    #[must_use]
    pub fn constant(value: AnimValue) -> Self {
        Self::new(vec![Key {
            t: RationalTime::ZERO,
            value,
            interp: Interp::Hold,
        }])
    }

    /// Set the value returned when the track is empty.
    #[must_use]
    pub fn with_default(mut self, default: AnimValue) -> Self {
        self.default = default;
        self
    }

    /// The value returned when the track has no keys.
    #[must_use]
    pub fn default_value(&self) -> AnimValue {
        self.default
    }

    /// Extrapolation before the first key ([`Extrap::Hold`] by default).
    #[must_use]
    pub fn pre(&self) -> Extrap {
        self.pre
    }

    /// Extrapolation after the last key ([`Extrap::Hold`] by default).
    #[must_use]
    pub fn post(&self) -> Extrap {
        self.post
    }

    /// Set the pre-range extrapolation (authoring). Does not touch the keys or
    /// the playback cursor.
    pub fn set_pre(&mut self, mode: Extrap) {
        self.pre = mode;
    }

    /// Set the post-range extrapolation (authoring).
    pub fn set_post(&mut self, mode: Extrap) {
        self.post = mode;
    }

    /// The keys, sorted by time.
    #[must_use]
    pub fn keys(&self) -> &[Key] {
        &self.keys
    }

    /// The stable ids, parallel to [`Track::keys`].
    #[must_use]
    pub fn ids(&self) -> &[KeyId] {
        &self.ids
    }

    /// `true` if the track has no keys.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.keys.is_empty()
    }

    /// Number of keys.
    #[must_use]
    pub fn len(&self) -> usize {
        self.keys.len()
    }

    // ─── Authoring (W0.T1) ──────────────────────────────────────────────────

    /// Index of the key with `id`, if present (linear scan — key counts per
    /// track are small; authoring is off the hot path).
    #[must_use]
    pub fn index_of(&self, id: KeyId) -> Option<usize> {
        self.ids.iter().position(|&x| x == id)
    }

    /// The id of the key at `index`, if any.
    #[must_use]
    pub fn key_id(&self, index: usize) -> Option<KeyId> {
        self.ids.get(index).copied()
    }

    /// The key with `id`, if present.
    #[must_use]
    pub fn key(&self, id: KeyId) -> Option<Key> {
        self.index_of(id).map(|i| self.keys[i])
    }

    /// Insert a key, keeping the track sorted; returns its stable [`KeyId`].
    /// Ties (equal `t`) insert after existing equal-time keys (stable).
    pub fn insert_key(&mut self, t: RationalTime, value: AnimValue, interp: Interp) -> KeyId {
        let idx = self.keys.partition_point(|k| k.t <= t);
        let id = KeyId(self.next_id);
        self.next_id += 1;
        self.keys.insert(idx, Key { t, value, interp });
        self.ids.insert(idx, id);
        self.roving.insert(idx, false);
        self.invalidate_cursor();
        id
    }

    /// Insert a key at `t`, or **update** the existing key at exactly `t` (same
    /// `RationalTime`). Returns the key's [`KeyId`]. Used by capture-the-pose
    /// authoring (K / auto-key), which must not stack duplicate keys at one time.
    pub fn upsert_key(&mut self, t: RationalTime, value: AnimValue, interp: Interp) -> KeyId {
        if let Some(i) = self.keys.iter().position(|k| k.t == t) {
            self.keys[i].value = value;
            self.keys[i].interp = interp;
            self.invalidate_cursor();
            self.ids[i]
        } else {
            self.insert_key(t, value, interp)
        }
    }

    /// Insert a key at `t` with `interp`, or — if a key already sits at exactly
    /// `t` — update **only its value**, keeping the interpolation it has.
    ///
    /// This is the capture-the-pose path (`K`, auto-key): re-keying an instant
    /// records the new pose, it does not throw away the easing the author drew
    /// in the graph editor. [`Track::upsert_key`], which replaces both, is for
    /// paste/duplicate — there the incoming key IS the whole key.
    pub fn upsert_value(&mut self, t: RationalTime, value: AnimValue, interp: Interp) -> KeyId {
        if let Some(i) = self.keys.iter().position(|k| k.t == t) {
            self.keys[i].value = value;
            self.invalidate_cursor();
            self.ids[i]
        } else {
            self.insert_key(t, value, interp)
        }
    }

    /// Remove the key with `id`. Returns `true` if it existed.
    pub fn remove_key(&mut self, id: KeyId) -> bool {
        match self.index_of(id) {
            Some(i) => {
                self.keys.remove(i);
                self.ids.remove(i);
                self.roving.remove(i);
                self.invalidate_cursor();
                true
            }
            None => false,
        }
    }

    /// Move the key with `id` to `new_t`, re-sorting. Returns `true` if it existed.
    pub fn move_key(&mut self, id: KeyId, new_t: RationalTime) -> bool {
        match self.index_of(id) {
            Some(i) => {
                self.keys[i].t = new_t;
                self.resort();
                true
            }
            None => false,
        }
    }

    /// Set the value of the key with `id` (no re-sort). Returns `true` if it existed.
    pub fn set_value(&mut self, id: KeyId, value: AnimValue) -> bool {
        match self.index_of(id) {
            Some(i) => {
                self.keys[i].value = value;
                true
            }
            None => false,
        }
    }

    /// Set the interpolation leaving the key with `id` (no re-sort). Returns
    /// `true` if it existed.
    pub fn set_interp(&mut self, id: KeyId, interp: Interp) -> bool {
        match self.index_of(id) {
            Some(i) => {
                self.keys[i].interp = interp;
                true
            }
            None => false,
        }
    }

    // ─── Bulk authoring (W0.T5) ─────────────────────────────────────────────

    /// Shift each listed key by `delta` (relative bulk drag), re-sorting once.
    /// Unknown ids are ignored.
    ///
    /// The offset is applied with exact rational arithmetic, so a key dragged by
    /// a whole number of frames lands *exactly* on the frame — round-tripping
    /// through `to_seconds` would leave it a fraction of a microsecond off, and
    /// every later equality test (upsert, overwrite-on-duplicate) would miss.
    pub fn move_keys(&mut self, ids: &[KeyId], delta: RationalTime) {
        for &id in ids {
            if let Some(i) = self.index_of(id) {
                self.keys[i].t = self.keys[i].t + delta;
            }
        }
        self.merge_moved_over_stationary(ids);
        self.resort();
    }

    /// After a move, a moved key that lands on a stationary key's exact time
    /// **absorbs** it: the two become one, the moved key winning (its value and
    /// interpolation stay). Overlapping keys must not silently stack two at one
    /// instant.
    ///
    /// The move is rigid — one `delta` for every id in `ids` — so moved keys keep
    /// their relative spacing and never collide with **each other**; a collision
    /// is always a moved key meeting a key that stayed put. The moved one wins,
    /// matching the duplicate-overwrite rule ([`Track::upsert_key`]).
    fn merge_moved_over_stationary(&mut self, moved_ids: &[KeyId]) {
        let moved_times: Vec<RationalTime> = moved_ids
            .iter()
            .filter_map(|&id| self.index_of(id).map(|i| self.keys[i].t))
            .collect();
        if moved_times.is_empty() {
            return;
        }
        let mut i = 0;
        while i < self.keys.len() {
            let stationary = !moved_ids.contains(&self.ids[i]);
            if stationary && moved_times.iter().any(|t| *t == self.keys[i].t) {
                self.keys.remove(i);
                self.ids.remove(i);
                self.roving.remove(i);
            } else {
                i += 1;
            }
        }
        self.invalidate_cursor();
    }

    /// Scale each listed key's time about `pivot_seconds` by `factor`
    /// (`t' = pivot + (t − pivot)·factor`), re-sorting once. Unknown ids ignored.
    pub fn scale_keys(&mut self, ids: &[KeyId], pivot_seconds: f64, factor: f64) {
        for &id in ids {
            if let Some(i) = self.index_of(id) {
                let s = pivot_seconds + (self.keys[i].t.to_seconds() - pivot_seconds) * factor;
                self.keys[i].t = RationalTime::from_seconds(s);
            }
        }
        self.resort();
    }

    /// **Distribute** the listed keys uniformly in time: the earliest and latest
    /// keep their exact times, and every key between them slides to equal spacing
    /// (`t_i = t_first + (t_last − t_first)·i/n`, in time order). Fewer than three
    /// listed keys is a no-op — with only the two endpoints there is nothing
    /// between them to respace. Unknown ids are ignored.
    ///
    /// Re-sorts once. A respaced key landing on a stationary (non-listed) key's
    /// exact time **absorbs** it, the moved one winning — the same rule as
    /// [`Track::move_keys`]. Interior keys move by *different* deltas (this is not
    /// a rigid shift), so the merge is told exactly which ids moved.
    pub fn distribute_keys(&mut self, ids: &[KeyId]) {
        // The listed keys that exist, in time order — the endpoints of THIS set
        // define the span, so they must be the extremes of the listed keys.
        let mut listed: Vec<(KeyId, RationalTime)> = ids
            .iter()
            .filter_map(|&id| self.index_of(id).map(|i| (id, self.keys[i].t)))
            .collect();
        if listed.len() < 3 {
            return;
        }
        listed.sort_by_key(|&(_, t)| t);
        let t0 = listed.first().expect("len >= 3").1.to_seconds();
        let t1 = listed.last().expect("len >= 3").1.to_seconds();
        let n = listed.len() - 1;
        // Only the interior keys move; the endpoints already sit at t0/t1.
        let moved: Vec<KeyId> = listed[1..n].iter().map(|(id, _)| *id).collect();
        for (rank, (id, _)) in listed.iter().enumerate().take(n).skip(1) {
            let s = t0 + (t1 - t0) * (rank as f64 / n as f64);
            if let Some(i) = self.index_of(*id) {
                self.keys[i].t = RationalTime::from_seconds(s);
            }
        }
        self.merge_moved_over_stationary(&moved);
        self.resort();
    }

    /// Remove every listed key. Unknown ids ignored.
    pub fn remove_keys(&mut self, ids: &[KeyId]) {
        let mut idxs: Vec<usize> = ids.iter().filter_map(|&id| self.index_of(id)).collect();
        idxs.sort_unstable();
        idxs.dedup();
        for &i in idxs.iter().rev() {
            self.keys.remove(i);
            self.ids.remove(i);
            self.roving.remove(i);
        }
        if !idxs.is_empty() {
            self.invalidate_cursor();
        }
    }

    /// Duplicate each listed key, offset by `delta`. Returns the copies' ids, in
    /// the order the sources were listed (for re-selecting the duplicates).
    ///
    /// A copy that lands exactly on an existing key **overwrites** it (and
    /// inherits its id) — a track may not hold two keys at one instant. The
    /// sources are read before anything is written, so a copy can safely land on
    /// another key of the same batch.
    pub fn duplicate_keys(&mut self, ids: &[KeyId], delta: RationalTime) -> Vec<KeyId> {
        // Snapshot first — writing shifts indices, and a copy may overwrite a
        // source that has not been read yet.
        let dups: Vec<Key> = ids
            .iter()
            .filter_map(|&id| self.key(id))
            .map(|k| Key {
                t: k.t + delta,
                ..k
            })
            .collect();
        dups.into_iter()
            .map(|k| self.upsert_key(k.t, k.value, k.interp))
            .collect()
    }

    // ─── Internals ──────────────────────────────────────────────────────────

    /// Re-sort keys + ids + roving flags in lockstep by time (stable) and reset
    /// the cursor.
    fn resort(&mut self) {
        let mut triples: Vec<(Key, KeyId, bool)> = self
            .keys
            .drain(..)
            .zip(self.ids.drain(..))
            .zip(self.roving.drain(..))
            .map(|((k, id), r)| (k, id, r))
            .collect();
        triples.sort_by_key(|p| p.0.t);
        for (k, id, r) in triples {
            self.keys.push(k);
            self.ids.push(id);
            self.roving.push(r);
        }
        self.invalidate_cursor();
    }

    #[inline]
    fn invalidate_cursor(&self) {
        self.cursor.store(0, Ordering::Relaxed);
    }

    /// `true` if `t` (seconds) lies in segment `[keys[i], keys[i + 1])`.
    #[inline]
    fn in_segment(&self, i: usize, t: f64) -> bool {
        let t0 = self.keys[i].t.to_seconds();
        let t1 = self.keys[i + 1].t.to_seconds();
        t >= t0 && t < t1
    }

    /// Binary search for the largest segment index `i` with `keys[i].t <= t`.
    /// Callers guarantee `first_t < t < last_t`, so `i ∈ 0..=len-2`.
    fn search(&self, t: f64) -> usize {
        self.keys
            .partition_point(|k| k.t.to_seconds() <= t)
            .saturating_sub(1)
            .min(self.keys.len() - 2)
    }

    /// Blend the two keys of segment `idx` at absolute time `t`. The shared tail
    /// of [`Track::sample`] and [`Track::value_at_in_range`] — one place decides
    /// what a segment reads, so the cursor path and the extrapolation path can
    /// never disagree.
    #[inline]
    fn segment_value(&self, idx: usize, t: f64) -> AnimValue {
        let k0 = self.keys[idx];
        let k1 = self.keys[idx + 1];
        let t0 = k0.t.to_seconds();
        let t1 = k1.t.to_seconds();
        let span = t1 - t0;
        let u = if span > 0.0 { (t - t0) / span } else { 0.0 };
        interpolate(k0.value, k1.value, k0.interp, u)
    }

    /// The value at `t`, assuming `t` lies WITHIN the keyed range — the sampler
    /// the extrapolation path calls after mapping an out-of-range `t` back into
    /// `[first_t, last_t]` (Loop/PingPong). Binary-search based, so it does NOT
    /// touch the forward-play cursor (a wrapped `t` jumps all over the range and
    /// would only thrash the hint). Exact boundaries return the boundary key.
    pub(crate) fn value_at_in_range(&self, t: f64) -> AnimValue {
        let n = self.keys.len();
        if t <= self.keys[0].t.to_seconds() {
            return self.keys[0].value;
        }
        if t >= self.keys[n - 1].t.to_seconds() {
            return self.keys[n - 1].value;
        }
        let idx = self.search(t);
        self.segment_value(idx, t)
    }
}

impl AttributeEvaluator for Track {
    fn sample(&self, t: f64) -> AnimValue {
        let n = self.keys.len();
        if n == 0 {
            return self.default;
        }
        if n == 1 {
            return self.keys[0].value;
        }

        // Ends. `Extrap::Hold` (the default) is the historical flat-clamp and
        // returns the boundary key value DIRECTLY — byte-identical to the
        // pre-extrapolation engine, which is what pins the fade fingerprint. A
        // non-Hold mode only diverges for a `t` STRICTLY outside the range; an
        // exact boundary still returns the boundary key.
        let t0 = self.keys[0].t.to_seconds();
        let tn = self.keys[n - 1].t.to_seconds();
        if t <= t0 {
            if t < t0 && self.pre != Extrap::Hold {
                return extrap::extrapolate(self, extrap::ExtrapSide::Pre, self.pre, t, t0, tn);
            }
            return self.keys[0].value;
        }
        if t >= tn {
            if t > tn && self.post != Extrap::Hold {
                return extrap::extrapolate(self, extrap::ExtrapSide::Post, self.post, t, t0, tn);
            }
            return self.keys[n - 1].value;
        }

        // Cursor fast path: the current or next segment (typical forward play),
        // else fall back to a binary search. The hint is only trusted after
        // `in_segment` re-validates it against `t`.
        let hint = self.cursor.load(Ordering::Relaxed).min(n - 2);
        let idx = if self.in_segment(hint, t) {
            hint
        } else if hint < n - 2 && self.in_segment(hint + 1, t) {
            hint + 1
        } else {
            self.search(t)
        };
        self.cursor.store(idx, Ordering::Relaxed);
        self.segment_value(idx, t)
    }
}

impl Clone for Track {
    fn clone(&self) -> Self {
        Self {
            keys: self.keys.clone(),
            ids: self.ids.clone(),
            roving: self.roving.clone(),
            next_id: self.next_id,
            default: self.default,
            pre: self.pre,
            post: self.post,
            cursor: AtomicUsize::new(self.cursor.load(Ordering::Relaxed)),
        }
    }
}

// Roving keys ("rove across time") — child module so the parallel `roving` vec
// stays a private invariant of `Track`.
#[path = "rove.rs"]
mod rove;
