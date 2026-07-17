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
use crate::curve_fit::FitKey;
use crate::curve_prep::FitChannel;
use crate::time::RationalTime;

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
    /// Monotonic playback hint (index of the last active segment).
    cursor: AtomicUsize,
}

/// Content equality ignores the atomic playback `cursor` (a transient sampling
/// hint) and the session-local `ids`/`next_id` — two tracks are equal when they
/// hold the same keys + roving flags + default. Used by history dedup
/// ([`crate::TimelineHistory`]).
impl PartialEq for Track {
    fn eq(&self, other: &Self) -> bool {
        self.keys == other.keys && self.roving == other.roving && self.default == other.default
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
}

impl From<Track> for TrackData {
    fn from(t: Track) -> Self {
        Self {
            keys: t.keys,
            default: t.default,
            roving: t.roving,
        }
    }
}

impl From<TrackData> for Track {
    fn from(d: TrackData) -> Self {
        let mut roving = d.roving;
        roving.resize(d.keys.len(), false);
        let mut t = Track::new(d.keys).with_default(d.default);
        t.roving = roving;
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

    /// **Play this track backwards inside `[0, span]`**: every key's time becomes
    /// `span − t`, and every segment's shape is mirrored with it.
    ///
    /// The subtle half is that a key's `Interp` describes the segment **leaving**
    /// it, so reversal moves each interp to a different key: the segment between
    /// keys `i` and `i+1` is, reversed, the one leaving key `n−2−i`. Mirror the
    /// times and leave the interps in place and the curve keeps its old
    /// accelerations while the values run backwards — every ease-out becomes an
    /// ease-in, which is precisely the thing reversing is meant to preserve. The
    /// LAST key has no outgoing segment, so its interp is not data: it is dropped,
    /// and the new last key inherits the same non-role.
    ///
    /// Ids ride with their keys ([`KeyId`] names a key, not a position), and so do
    /// the roving flags — a key whose time is derived is still that key afterwards.
    pub fn reverse_about(&mut self, span: f64) {
        if self.keys.is_empty() {
            return;
        }
        let n = self.keys.len();
        for k in &mut self.keys {
            k.t = RationalTime::from_seconds(span - k.t.to_seconds());
        }
        // Hand each segment's shape to the key that will own it after the flip. The
        // walk is over the OLD order, so read the interps out first.
        let old: Vec<Interp> = self.keys.iter().map(|k| k.interp).collect();
        for i in 0..n {
            // Key `i` ends up at position `n-1-i`; the segment leaving it there is
            // the OLD segment leaving key `n-2-(n-1-i)` = `i-1`, played backwards.
            self.keys[i].interp = if i == 0 {
                // The new LAST key: no outgoing segment. Keep it inert.
                Interp::Linear
            } else {
                old[i - 1].reversed()
            };
        }
        self.keys.reverse();
        self.ids.reverse();
        self.roving.reverse();
        // The times were mirrored, so the reverse above already restores sort order
        // — but a `span` that does not bracket every key (a key past the clip's end)
        // would not. Re-sort rather than assume the caller measured well.
        self.resort();
        self.invalidate_cursor();
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

    /// Replace the non-roving keys in `[t_min, t_max]` seconds (inclusive) with a
    /// MINIMAL cubic-Bézier fit within value `tol` — the record-cleanup path
    /// ([`crate::fit_fcurve`], Schneider). Dense one-key-per-frame recordings
    /// become clean [`Interp::BezierW`] curves of a few keys, precise to `tol`.
    /// `smooth_passes` low-passes the recorded values first
    /// ([`crate::smooth_values`]) — mocap tremor would otherwise make the fit
    /// over-subdivide; `0` disables it (a clean synthetic curve needs none).
    ///
    /// Endpoints are pinned to the range's first/last sampled values, so keys
    /// OUTSIDE the range keep their segments (the neighbour's interpolation is
    /// untouched — it still points at a key of the same time and value). Roving
    /// keys are skipped (their time is derived, not sampled). Returns `true` when
    /// it reduced the key count; a no-op (fewer than 3 in-range keys, or no
    /// reduction) returns `false` and leaves the track byte-identical.
    pub fn simplify_range(
        &mut self,
        t_min: f64,
        t_max: f64,
        tol: f64,
        channel: FitChannel,
        smooth_passes: usize,
    ) -> bool {
        let Some(rs) = self.range_samples(t_min, t_max, channel, smooth_passes) else {
            return false;
        };
        let fitted = crate::fit_fcurve(&rs.samples, tol, channel.bounds);
        self.apply_fit(&rs.ids, &rs.samples, &fitted)
    }

    /// The PREPARED `(time, value)` samples of the non-roving scalar keys in
    /// `[t_min, t_max]`, with their ids — the input a fit consumes. `channel` says
    /// what the numbers mean (an angle is unwrapped, a bounded channel carries its
    /// bound) and `smooth_passes` low-passes the tremor out
    /// ([`crate::curve_prep::prepare`]).
    ///
    /// `None` when there is nothing to fit (fewer than 3 in-range keys, or a
    /// non-scalar key in range). Public so a caller can fit SEVERAL tracks
    /// together and align their key times ([`Track::simplify_range_at`]).
    #[must_use]
    pub fn range_samples(
        &self,
        t_min: f64,
        t_max: f64,
        channel: FitChannel,
        smooth_passes: usize,
    ) -> Option<RangeSamples> {
        let mut ids = Vec::new();
        let mut samples = Vec::new();
        for i in 0..self.keys.len() {
            let ts = self.keys[i].t.to_seconds();
            if ts >= t_min && ts <= t_max && !self.roving[i] {
                // Only scalar keys fit; a non-scalar in range would break the
                // (t, value) sampling, so leave the whole range alone.
                let AnimValue::Float(v) = self.keys[i].value else {
                    return None;
                };
                ids.push(self.ids[i]);
                samples.push((ts, f64::from(v)));
            }
        }
        if samples.len() < 3 {
            return None;
        }
        crate::prepare(&mut samples, channel, smooth_passes);
        Some(RangeSamples { ids, samples })
    }

    /// Like [`Track::simplify_range`], but the fitted keys land at the GIVEN
    /// times — the aligned-columns path ([`crate::fit_fcurve_at`]). Every track of
    /// one recording session re-fitted at the same times reads as clean dope-sheet
    /// columns, so an animator can grab a column and re-time every channel at once.
    ///
    pub fn simplify_range_at(
        &mut self,
        t_min: f64,
        t_max: f64,
        times: &[f64],
        channel: FitChannel,
        smooth_passes: usize,
    ) -> bool {
        let Some(rs) = self.range_samples(t_min, t_max, channel, smooth_passes) else {
            return false;
        };
        let fitted = crate::fit_fcurve_at(&rs.samples, times, channel.bounds);
        self.apply_fit(&rs.ids, &rs.samples, &fitted)
    }

    /// Swap the keys `ids` (the fitted range) for `fitted`. No-op when the fit did
    /// not reduce the count.
    fn apply_fit(&mut self, ids: &[KeyId], samples: &[(f64, f64)], fitted: &[FitKey]) -> bool {
        if fitted.len() >= samples.len() {
            return false; // nothing to gain — keep the originals
        }
        self.remove_keys(ids);
        for fk in fitted {
            self.insert_key(
                RationalTime::from_seconds(fk.t),
                AnimValue::Float(fk.v as f32),
                fk.interp,
            );
        }
        true
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

        // Flat-clamped ends.
        if t <= self.keys[0].t.to_seconds() {
            return self.keys[0].value;
        }
        if t >= self.keys[n - 1].t.to_seconds() {
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

        let k0 = self.keys[idx];
        let k1 = self.keys[idx + 1];
        let t0 = k0.t.to_seconds();
        let t1 = k1.t.to_seconds();
        let span = t1 - t0;
        let u = if span > 0.0 { (t - t0) / span } else { 0.0 };
        interpolate(k0.value, k1.value, k0.interp, u)
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
            cursor: AtomicUsize::new(self.cursor.load(Ordering::Relaxed)),
        }
    }
}

// Roving keys ("rove across time") — child module so the parallel `roving` vec
// stays a private invariant of `Track`.
#[path = "rove.rs"]
mod rove;
