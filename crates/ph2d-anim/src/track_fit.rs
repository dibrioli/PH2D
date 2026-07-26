//! [`Track`]'s record-cleanup FIT methods (Schneider) — split from `track.rs`
//! under the file LOC cap, one subject: turning a dense one-key-per-frame
//! recording into a MINIMAL clean curve. A child module (`impl Track`) so it
//! reaches the private key/id/roving vecs the fit rewrites.

use ph2d_vector_traits::AnimValue;

use super::{KeyId, RangeSamples, Track};
use crate::curve_fit::FitKey;
use crate::curve_prep::FitChannel;
use crate::time::RationalTime;

impl Track {
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
}
