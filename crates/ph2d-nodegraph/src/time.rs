//! Time scopes: remapping the clock of an upstream subtree (plan §1.5, M2.N1).
//!
//! A [`TimeMap`] is a pure function `t -> t'`. The cook engine
//! ([`crate::cook::Cook::cook_scoped`]) applies one when it descends into the
//! **inputs** of a remapper node, so the node's whole upstream subtree cooks at
//! `t'` while the node itself — and everything downstream — stays on the outer
//! clock. This is the Houdini `timeshift` / Nuke `TimeOffset` reading: the wire
//! carries "this subtree, sampled at another time", not a delayed value.
//!
//! **Why in the substrate and not in a node:** a node only sees its own inputs
//! (`EvalCtx`, FBP black box, ADR-0031); nothing inside a node can change the
//! playhead its *upstream* was pulled at. The remap has to happen in the puller.
//! Keeping it here also earns the memo for free: a `Loop` that returns to the
//! same `t'` hits the cache instead of recomputing the subtree.
//!
//! **Determinism (HR-5):** every mode is `+ - * floor/rem_euclid` on `f64`.
//! No transcendentals, no rounding-mode surprises — a remapped replay is
//! bit-identical across platforms, like the unremapped one.
//!
//! **Isolation:** this module is a leaf (depends on nothing in the crate) and
//! the cook's scoped entry points are additive siblings of the plain ones, so a
//! parallel line that never remaps time compiles and behaves exactly as before.

/// How a [`TimeMap`] rewrites the clock of the subtree above it.
///
/// The vocabulary is the reference catalogue's (`_NODES.md` — timeRemap:
/// `scale` / `loop` / `pingpong` / `freeze` / `reverse`), which is in turn the
/// motion-design standard (After Effects Time Remap, Cavalry Time Offset).
#[derive(Copy, Clone, Debug, PartialEq, Eq, Default)]
pub enum TimeMode {
    /// `t' = t·scale + offset`. Slow motion (`scale < 1`), fast-forward, or a
    /// plain time shift. The identity when `scale = 1, offset = 0`.
    #[default]
    Scale,
    /// `t' = offset + (t·scale) mod duration`. A cycling window: the subtree
    /// replays its first `duration` seconds forever. The memo makes the second
    /// lap free.
    Loop,
    /// A triangle wave of period `2·duration`: forward, then backward. Reads as
    /// a physical back-and-forth rather than the hard cut of `Loop`.
    PingPong,
    /// `t' = offset`, constant. Freezes the subtree on one moment (a still,
    /// while downstream physics keeps running).
    Freeze,
    /// `t' = offset - t·scale`. Runs the subtree backwards.
    Reverse,
}

impl TimeMode {
    /// Decode the `f32` an enum-valued param stores (its option index), by the
    /// declaration order above. Out-of-range → [`TimeMode::Scale`], the
    /// identity: an unknown mode must not invent a time warp.
    #[must_use]
    pub fn from_index(i: f32) -> Self {
        match i.round() as i32 {
            1 => TimeMode::Loop,
            2 => TimeMode::PingPong,
            3 => TimeMode::Freeze,
            4 => TimeMode::Reverse,
            _ => TimeMode::Scale,
        }
    }

    /// The stable discriminant used when hashing a scope (see
    /// [`TimeMap::hash_into`]).
    #[must_use]
    pub fn index(self) -> u8 {
        match self {
            TimeMode::Scale => 0,
            TimeMode::Loop => 1,
            TimeMode::PingPong => 2,
            TimeMode::Freeze => 3,
            TimeMode::Reverse => 4,
        }
    }
}

/// A clock rewrite applied to one node's upstream subtree.
///
/// `duration` is only read by [`TimeMode::Loop`] / [`TimeMode::PingPong`]; a
/// non-positive one degrades to [`TimeMode::Scale`] rather than dividing by
/// zero (a zero-length loop has no meaning, and `NaN` would poison every
/// downstream fingerprint).
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct TimeMap {
    pub mode: TimeMode,
    pub scale: f64,
    pub offset: f64,
    pub duration: f64,
}

impl Default for TimeMap {
    fn default() -> Self {
        Self {
            mode: TimeMode::Scale,
            scale: 1.0,
            offset: 0.0,
            duration: 1.0,
        }
    }
}

impl TimeMap {
    /// Is this the identity map (`t' = t`)? An identity scope is dropped by the
    /// cook, so a remapper left at its defaults costs nothing — no second cache
    /// lane, no missed memo against unscoped consumers of the same subtree.
    #[must_use]
    pub fn is_identity(&self) -> bool {
        self.mode == TimeMode::Scale && self.scale == 1.0 && self.offset == 0.0
    }

    /// `t -> t'`. Total: never panics, never returns `NaN` for finite input.
    #[must_use]
    pub fn apply(&self, t: f64) -> f64 {
        let scaled = t * self.scale;
        let looping = self.duration > 0.0 && self.duration.is_finite();
        match self.mode {
            TimeMode::Scale => scaled + self.offset,
            TimeMode::Freeze => self.offset,
            TimeMode::Reverse => self.offset - scaled,
            TimeMode::Loop if looping => self.offset + scaled.rem_euclid(self.duration),
            TimeMode::PingPong if looping => {
                // Triangle wave: fold the second half of each 2·duration period
                // back on itself, so the subtree plays forward then backward
                // with no discontinuity at the turn.
                let u = scaled.rem_euclid(2.0 * self.duration);
                let folded = if u > self.duration {
                    2.0 * self.duration - u
                } else {
                    u
                };
                self.offset + folded
            }
            // Degenerate duration: behave as the plain scale, never divide by 0.
            TimeMode::Loop | TimeMode::PingPong => scaled + self.offset,
        }
    }

    /// Fold this map into an FNV-1a accumulator, so the cook can key its memo
    /// per *scope chain* (`ScopeKey`). Bit patterns, not values: two maps that
    /// differ only in `-0.0` vs `0.0` are distinguishable, and a `NaN` param
    /// hashes consistently rather than comparing unequal to itself.
    pub fn hash_into(&self, hash: &mut u64) {
        let mut mix = |bytes: &[u8]| {
            for b in bytes {
                *hash ^= u64::from(*b);
                *hash = hash.wrapping_mul(0x0000_0100_0000_01b3);
            }
        };
        mix(&[self.mode.index()]);
        mix(&self.scale.to_bits().to_le_bytes());
        mix(&self.offset.to_bits().to_le_bytes());
        mix(&self.duration.to_bits().to_le_bytes());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn map(mode: TimeMode, scale: f64, offset: f64, duration: f64) -> TimeMap {
        TimeMap {
            mode,
            scale,
            offset,
            duration,
        }
    }

    #[test]
    fn scale_shifts_and_stretches_and_the_default_is_the_identity() {
        assert!(TimeMap::default().is_identity());
        assert_eq!(TimeMap::default().apply(3.25), 3.25);
        // Half speed, shifted a second later.
        let m = map(TimeMode::Scale, 0.5, 1.0, 1.0);
        assert!(!m.is_identity());
        assert_eq!(m.apply(4.0), 3.0);
    }

    #[test]
    fn loop_cycles_and_pingpong_folds() {
        let l = map(TimeMode::Loop, 1.0, 0.0, 2.0);
        assert_eq!(l.apply(0.5), 0.5);
        assert_eq!(l.apply(2.5), 0.5, "the second lap repeats the first");
        assert_eq!(l.apply(-0.5), 1.5, "negative time wraps forward, not back");

        let p = map(TimeMode::PingPong, 1.0, 0.0, 2.0);
        assert_eq!(p.apply(0.5), 0.5);
        assert_eq!(p.apply(2.5), 1.5, "past the turn it runs backwards");
        assert_eq!(p.apply(3.5), 0.5);
        assert_eq!(p.apply(4.0), 0.0, "and lands back at the start");
        // Continuous at the turning point (no jump the eye would catch).
        assert_eq!(p.apply(2.0), 2.0);
    }

    #[test]
    fn freeze_is_constant_and_reverse_runs_backwards() {
        let f = map(TimeMode::Freeze, 1.0, 1.5, 1.0);
        assert_eq!(f.apply(0.0), 1.5);
        assert_eq!(f.apply(99.0), 1.5);

        let r = map(TimeMode::Reverse, 1.0, 10.0, 1.0);
        assert_eq!(r.apply(0.0), 10.0);
        assert_eq!(r.apply(4.0), 6.0);
    }

    /// A zero (or infinite) duration must not divide by zero into `NaN`: a
    /// `NaN` playhead poisons every downstream fingerprint (`NaN != NaN`, so
    /// nothing ever hits the memo again) — a silent, unbounded recompute.
    #[test]
    fn a_degenerate_duration_degrades_to_scale_instead_of_nan() {
        for bad in [0.0, -2.0, f64::INFINITY, f64::NAN] {
            for mode in [TimeMode::Loop, TimeMode::PingPong] {
                let t = map(mode, 1.0, 0.0, bad).apply(3.0);
                assert_eq!(t, 3.0, "mode {mode:?} with duration {bad} fell through");
            }
        }
    }

    #[test]
    fn from_index_round_trips_and_rejects_out_of_range() {
        for mode in [
            TimeMode::Scale,
            TimeMode::Loop,
            TimeMode::PingPong,
            TimeMode::Freeze,
            TimeMode::Reverse,
        ] {
            assert_eq!(TimeMode::from_index(f32::from(mode.index())), mode);
        }
        // Junk decodes to the identity mode, never to a random warp.
        assert_eq!(TimeMode::from_index(-1.0), TimeMode::Scale);
        assert_eq!(TimeMode::from_index(99.0), TimeMode::Scale);
    }

    #[test]
    fn the_scope_hash_separates_maps_that_differ_anywhere() {
        let h = |m: TimeMap| {
            let mut acc = 0xcbf2_9ce4_8422_2325u64;
            m.hash_into(&mut acc);
            acc
        };
        let base = map(TimeMode::Loop, 1.0, 0.0, 2.0);
        assert_eq!(h(base), h(base), "stable");
        assert_ne!(h(base), h(map(TimeMode::PingPong, 1.0, 0.0, 2.0)));
        assert_ne!(h(base), h(map(TimeMode::Loop, 2.0, 0.0, 2.0)));
        assert_ne!(h(base), h(map(TimeMode::Loop, 1.0, 1.0, 2.0)));
        assert_ne!(h(base), h(map(TimeMode::Loop, 1.0, 0.0, 3.0)));
    }
}
