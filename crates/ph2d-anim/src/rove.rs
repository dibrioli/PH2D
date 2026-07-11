//! Roving keys (AE "rove across time") — the [`Track`] half.
//!
//! A key marked **roving** has no authored time: only its VALUE is authored,
//! and [`Track::resolve_roving`] derives its time so the value moves at
//! **constant speed** between the nearest pinned neighbours (time ∝ accumulated
//! |Δvalue| along the run — the AE spatial model applied to a scalar track).
//! The first and last keys of a track are always treated as pinned — a rove
//! flag on a boundary key is kept (it starts roving if the track later grows
//! past it) but ignored by the resolver, exactly like AE.
//!
//! The resolver is **idempotent** and cheap (two index passes, no allocation),
//! so the timeline calls it after every document edit — one choke point instead
//! of instrumenting each authoring path.
//!
//! A child module of [`crate::track`] so the parallel `roving` vec stays a
//! private invariant of `Track`.

use ph2d_vector_traits::AnimValue;

use super::{KeyId, Track};
use crate::time::RationalTime;

/// The value distance a roving run accumulates across one segment. Scalar
/// tracks (the timeline's case) use |Δ|; a non-scalar pair contributes zero and
/// the run degrades to the uniform fallback.
fn dist(a: AnimValue, b: AnimValue) -> f64 {
    match (a, b) {
        (AnimValue::Float(x), AnimValue::Float(y)) => f64::from((y - x).abs()),
        _ => 0.0,
    }
}

impl Track {
    /// Roving flags, parallel to [`Track::keys`].
    #[must_use]
    pub fn roving(&self) -> &[bool] {
        &self.roving
    }

    /// Whether the key with `id` is roving. `false` for an unknown id.
    #[must_use]
    pub fn is_roving(&self, id: KeyId) -> bool {
        self.index_of(id).is_some_and(|i| self.roving[i])
    }

    /// Mark / unmark the key with `id` as roving. Returns `true` if it existed.
    /// Call [`Track::resolve_roving`] afterwards to re-derive the times.
    pub fn set_roving(&mut self, id: KeyId, on: bool) -> bool {
        match self.index_of(id) {
            Some(i) => {
                self.roving[i] = on;
                true
            }
            None => false,
        }
    }

    /// Re-derive the time of every roving key: each maximal run of consecutive
    /// roving keys is redistributed between its pinned neighbours so the value
    /// travels at constant speed (time ∝ accumulated |Δvalue|; a run with zero
    /// total distance spreads uniformly). Boundary keys are always pinned.
    /// Idempotent; keys stay sorted (the derived times are monotone within the
    /// pinned span).
    pub fn resolve_roving(&mut self) {
        let n = self.keys.len();
        if n < 3 {
            return;
        }
        let mut i = 1;
        while i < n - 1 {
            if !self.roving[i] {
                i += 1;
                continue;
            }
            // Maximal roving run [i, q); p/q are the pinned anchors (the last
            // key anchors even when flagged — boundaries never rove).
            let p = i - 1;
            let mut q = i;
            while q < n - 1 && self.roving[q] {
                q += 1;
            }
            self.redistribute(p, q);
            i = q + 1;
        }
    }

    /// Place keys `p+1..q` between the pinned anchors `p` and `q` at constant
    /// value speed.
    fn redistribute(&mut self, p: usize, q: usize) {
        let t0 = self.keys[p].t.to_seconds();
        let span = self.keys[q].t.to_seconds() - t0;
        let mut total = 0.0;
        for j in p..q {
            total += dist(self.keys[j].value, self.keys[j + 1].value);
        }
        let mut accum = 0.0;
        for j in (p + 1)..q {
            accum += dist(self.keys[j - 1].value, self.keys[j].value);
            let u = if total > 0.0 {
                accum / total
            } else {
                (j - p) as f64 / (q - p) as f64
            };
            self.keys[j].t = RationalTime::from_seconds(t0 + span * u);
        }
        self.invalidate_cursor();
    }
}

#[cfg(test)]
mod tests {
    use super::super::Key;
    use super::*;
    use crate::Interp;

    fn s(t: f64) -> RationalTime {
        RationalTime::from_seconds(t)
    }

    fn key(t: f64, v: f32) -> Key {
        Key {
            t: s(t),
            value: AnimValue::Float(v),
            interp: Interp::Linear,
        }
    }

    fn times(tr: &Track) -> Vec<f64> {
        tr.keys().iter().map(|k| k.t.to_seconds()).collect()
    }

    #[test]
    fn a_roving_key_slides_to_where_the_value_speed_is_constant() {
        // Pinned 0 → 0 and 4 s → 30; the middle key's VALUE is 10 (⅓ of the
        // travel) but it was authored at 3 s. Roving derives t = 4 · 10/30.
        let mut tr = Track::new(vec![key(0.0, 0.0), key(3.0, 10.0), key(4.0, 30.0)]);
        let id = tr.key_id(1).unwrap();
        tr.set_roving(id, true);
        tr.resolve_roving();
        let t = tr.keys()[1].t.to_seconds();
        assert!((t - 4.0 / 3.0).abs() < 1e-5, "t = {t}, expected 4/3");
        // Idempotent: resolving again moves nothing.
        let before = times(&tr);
        tr.resolve_roving();
        assert_eq!(times(&tr), before);
    }

    #[test]
    fn a_run_of_roving_keys_distributes_by_accumulated_travel() {
        // Values 0 → 10 → 30 → 40 over 0..4 s: distances 10/20/10 (total 40),
        // so the roving pair lands at 1 s and 3 s regardless of where it was
        // authored.
        let mut tr = Track::new(vec![
            key(0.0, 0.0),
            key(0.5, 10.0),
            key(0.6, 30.0),
            key(4.0, 40.0),
        ]);
        for i in [1, 2] {
            let id = tr.key_id(i).unwrap();
            tr.set_roving(id, true);
        }
        tr.resolve_roving();
        let t = times(&tr);
        assert!((t[1] - 1.0).abs() < 1e-5, "t1 = {}", t[1]);
        assert!((t[2] - 3.0).abs() < 1e-5, "t2 = {}", t[2]);
    }

    #[test]
    fn a_reversing_value_counts_travel_not_displacement() {
        // 0 → 10 → 0 over 0..2 s: total travel 20, the peak sits halfway even
        // though its DISPLACEMENT from both anchors is equal.
        let mut tr = Track::new(vec![key(0.0, 0.0), key(1.7, 10.0), key(2.0, 0.0)]);
        let id = tr.key_id(1).unwrap();
        tr.set_roving(id, true);
        tr.resolve_roving();
        let t = tr.keys()[1].t.to_seconds();
        assert!((t - 1.0).abs() < 1e-5, "t = {t}");
    }

    #[test]
    fn zero_travel_spreads_uniformly_and_boundaries_never_rove() {
        // All values equal → no distance signal → uniform spacing.
        let mut tr = Track::new(vec![
            key(0.0, 5.0),
            key(0.1, 5.0),
            key(0.2, 5.0),
            key(3.0, 5.0),
        ]);
        for i in [1, 2] {
            let id = tr.key_id(i).unwrap();
            tr.set_roving(id, true);
        }
        tr.resolve_roving();
        let t = times(&tr);
        assert!((t[1] - 1.0).abs() < 1e-5, "t1 = {}", t[1]);
        assert!((t[2] - 2.0).abs() < 1e-5, "t2 = {}", t[2]);
        // Boundary keys keep their times even when flagged.
        let mut tr = Track::new(vec![key(0.0, 0.0), key(1.0, 1.0), key(2.0, 2.0)]);
        for i in [0, 2] {
            let id = tr.key_id(i).unwrap();
            tr.set_roving(id, true);
        }
        tr.resolve_roving();
        assert_eq!(times(&tr), vec![0.0, 1.0, 2.0], "boundaries pinned");
    }

    #[test]
    fn roving_flags_follow_their_keys_through_edits_and_serde() {
        let mut tr = Track::new(vec![key(0.0, 0.0), key(1.0, 10.0), key(2.0, 20.0)]);
        let roving_id = tr.key_id(1).unwrap();
        tr.set_roving(roving_id, true);
        // An insert BEFORE the roving key shifts indices; the flag must follow
        // the key, not the index.
        tr.insert_key(s(0.5), AnimValue::Float(1.0), Interp::Linear);
        assert!(tr.is_roving(roving_id), "flag follows the key");
        assert_eq!(tr.roving().iter().filter(|r| **r).count(), 1);
        // A move that re-sorts keeps the pairing.
        let first = tr.key_id(0).unwrap();
        tr.move_key(first, s(3.0));
        assert!(tr.is_roving(roving_id), "flag survives a resort");
        // Removing the roving key drops its flag.
        tr.remove_key(roving_id);
        assert!(tr.roving().iter().all(|r| !r));
        // Serde round-trip carries the flags (content, unlike ids). JSON keeps
        // the test legible, as in `tests/serde_roundtrip.rs`; the wire format is
        // the document's choice.
        let mut tr = Track::new(vec![key(0.0, 0.0), key(1.0, 10.0), key(2.0, 20.0)]);
        let id = tr.key_id(1).unwrap();
        tr.set_roving(id, true);
        let json = serde_json::to_string(&tr).unwrap();
        let back: Track = serde_json::from_str(&json).unwrap();
        assert_eq!(back.roving(), tr.roving(), "roving persists");
        assert_eq!(back, tr, "content equality includes roving");
    }
}
