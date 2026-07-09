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
    /// Next id to hand out (monotonic).
    next_id: u64,
    /// Returned when the track is empty (documented explicit default).
    default: AnimValue,
    /// Monotonic playback hint (index of the last active segment).
    cursor: AtomicUsize,
}

/// Content equality ignores the atomic playback `cursor` (a transient sampling
/// hint) and the session-local `ids`/`next_id` — two tracks are equal when they
/// hold the same keys + default. Used by history dedup ([`crate::TimelineHistory`]).
impl PartialEq for Track {
    fn eq(&self, other: &Self) -> bool {
        self.keys == other.keys && self.default == other.default
    }
}

/// Serde proxy for [`Track`] — persists only keys + default and rebuilds through
/// [`Track::new`] (fresh ids, reset cursor; the atomic cursor + session-local
/// ids never serialize).
#[derive(Serialize, Deserialize)]
struct TrackData {
    keys: Vec<Key>,
    #[serde(with = "crate::anim_value_serde")]
    default: AnimValue,
}

impl From<Track> for TrackData {
    fn from(t: Track) -> Self {
        Self {
            keys: t.keys,
            default: t.default,
        }
    }
}

impl From<TrackData> for Track {
    fn from(d: TrackData) -> Self {
        Track::new(d.keys).with_default(d.default)
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

    /// Remove the key with `id`. Returns `true` if it existed.
    pub fn remove_key(&mut self, id: KeyId) -> bool {
        match self.index_of(id) {
            Some(i) => {
                self.keys.remove(i);
                self.ids.remove(i);
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

    /// Shift each listed key by `delta_seconds` (relative bulk drag), re-sorting
    /// once. Unknown ids are ignored. Times snap to microsecond precision
    /// ([`RationalTime::from_seconds`]).
    pub fn move_keys(&mut self, ids: &[KeyId], delta_seconds: f64) {
        for &id in ids {
            if let Some(i) = self.index_of(id) {
                let s = self.keys[i].t.to_seconds() + delta_seconds;
                self.keys[i].t = RationalTime::from_seconds(s);
            }
        }
        self.resort();
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

    /// Remove every listed key. Unknown ids ignored.
    pub fn remove_keys(&mut self, ids: &[KeyId]) {
        let mut idxs: Vec<usize> = ids.iter().filter_map(|&id| self.index_of(id)).collect();
        idxs.sort_unstable();
        idxs.dedup();
        for &i in idxs.iter().rev() {
            self.keys.remove(i);
            self.ids.remove(i);
        }
        if !idxs.is_empty() {
            self.invalidate_cursor();
        }
    }

    /// Duplicate each listed key, offset by `delta_seconds`, giving each copy a
    /// fresh id. Returns the new ids (for re-selecting the duplicates).
    pub fn duplicate_keys(&mut self, ids: &[KeyId], delta_seconds: f64) -> Vec<KeyId> {
        // Snapshot first — inserting shifts indices.
        let dups: Vec<Key> = ids
            .iter()
            .filter_map(|&id| self.key(id))
            .map(|k| Key {
                t: RationalTime::from_seconds(k.t.to_seconds() + delta_seconds),
                ..k
            })
            .collect();
        dups.into_iter()
            .map(|k| self.insert_key(k.t, k.value, k.interp))
            .collect()
    }

    // ─── Internals ──────────────────────────────────────────────────────────

    /// Re-sort keys + ids in lockstep by time (stable) and reset the cursor.
    fn resort(&mut self) {
        let mut pairs: Vec<(Key, KeyId)> = self.keys.drain(..).zip(self.ids.drain(..)).collect();
        pairs.sort_by_key(|p| p.0.t);
        for (k, id) in pairs {
            self.keys.push(k);
            self.ids.push(id);
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
            next_id: self.next_id,
            default: self.default,
            cursor: AtomicUsize::new(self.cursor.load(Ordering::Relaxed)),
        }
    }
}
