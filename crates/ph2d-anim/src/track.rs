//! [`Track`] — a keyframe store implementing [`AttributeEvaluator`].
//!
//! A track resolves one parameter over absolute time. Keys are stored sorted by
//! their [`RationalTime`]; [`Track::sample`] finds the active segment, normalizes
//! to `u ∈ [0, 1]`, applies the segment's [`Interp`], and blends the two keys'
//! values via [`ph2d_vector_traits::LinearInterp`].
//!
//! # HR-3 (zero-alloc hot path)
//!
//! [`Track::sample`] never allocates. Beyond the binary search it keeps a
//! monotonic **playback cursor** (a relaxed [`AtomicUsize`]) so forward playback
//! is `O(1)` amortized. The atomic keeps `Track` `Send + Sync` (the cursor is a
//! pure hint — every read is validated against `t` before use, so a racy load is
//! always corrected by a binary-search fallback).

use core::sync::atomic::{AtomicUsize, Ordering};

use ph2d_vector_traits::{AnimValue, AttributeEvaluator};

use crate::curve::{Interp, interpolate};
use crate::time::RationalTime;

/// One keyframe: a time, a value, and the interpolation leaving it.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Key {
    /// Absolute time of the key.
    pub t: RationalTime,
    /// Value at the key.
    pub value: AnimValue,
    /// Interpolation from this key toward the next.
    pub interp: Interp,
}

/// A keyframe store for a single animated parameter.
///
/// Sampling before the first key returns the first value; after the last key,
/// the last value (flat-clamped ends). An empty track returns [`Track::default_value`].
#[derive(Debug)]
pub struct Track {
    /// Keys sorted ascending by `t`.
    keys: Vec<Key>,
    /// Returned when the track is empty (documented explicit default).
    default: AnimValue,
    /// Monotonic playback hint (index of the last active segment).
    cursor: AtomicUsize,
}

impl Track {
    /// Build a track from keys (sorted by time internally).
    ///
    /// The empty-track default is the first key's value, or `AnimValue::Float(0.0)`
    /// when there are no keys. Override with [`Track::with_default`].
    #[must_use]
    pub fn new(mut keys: Vec<Key>) -> Self {
        keys.sort_by_key(|k| k.t);
        let default = keys.first().map_or(AnimValue::Float(0.0), |k| k.value);
        Self {
            keys,
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
            default: self.default,
            cursor: AtomicUsize::new(self.cursor.load(Ordering::Relaxed)),
        }
    }
}
