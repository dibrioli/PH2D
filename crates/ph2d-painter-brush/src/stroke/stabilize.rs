//! The lazy-mouse **stabilizer** (smooth-stroke) for the `Space` method: the running filtered position
//! `stab_pos` lags the raw cursor by the stabilizer intensity, filtering hand tremor, and [`Stroke::settle`]
//! catches it up to the cursor on a pause. A child module of [`super`] so it keeps private access to
//! `Stroke`'s internals (`walk_smoothed`/`stab_pos`/the blend consts) and the `dist` free fn; split out to
//! keep `stroke.rs` under the workspace LOC cap.

use super::*;

impl Stroke {
    /// Catch the stabilizer up to the cursor while the stroke is held but the pointer is parked:
    /// step the filtered position toward the last raw sample and paint the spline up to it, so a
    /// high-stabilizer stroke reaches the cursor on a pause — not only at pointer-up. No-op when
    /// there is no lag to flush (stabilizer off, already caught up, or a non-`Space` method). The
    /// tool drives this from its per-frame tick while the pointer is stationary.
    pub fn settle(&mut self, out: &mut Vec<Dab>) {
        out.clear();
        if !self.started || self.spec.stroke_method != StrokeMethod::Space {
            return;
        }
        let s = self.spec.stabilizer.clamp(0.0, 1.0);
        if s <= f32::EPSILON {
            return;
        }
        let d = dist(self.stab_pos, self.last_raw_pos);
        if d <= SETTLE_EPS_PX {
            return; // already at the cursor
        }
        // Converge faster than the in-stroke blend (no new input to smooth on a pause — the artist
        // just wants the line to arrive); snap the final sub-pixel (the exponential never reaches).
        let blend = (1.0 - s * (1.0 - STABILIZER_MIN_BLEND)).max(SETTLE_BLEND_FLOOR);
        self.stab_pos = if d * (1.0 - blend) <= SETTLE_EPS_PX {
            self.last_raw_pos
        } else {
            [
                self.stab_pos[0] + (self.last_raw_pos[0] - self.stab_pos[0]) * blend,
                self.stab_pos[1] + (self.last_raw_pos[1] - self.stab_pos[1]) * blend,
            ]
        };
        self.walk_smoothed(
            StrokePoint {
                pos: self.stab_pos,
                pressure: self.last_raw_pressure,
            },
            out,
        );
        // A parked catch-up can be where the heading first settles (high-stabilizer stroke), so route
        // these dabs through the Rake warm-up too — otherwise the held opening would never release.
        self.warmup_gate(out);
    }

    /// Lazy-mouse stabilizer: blend the running filtered position [`Self::stab_pos`] toward the
    /// incoming sample by `1 − intensity` (clamped to a floor), so a higher intensity lags more and
    /// filters out hand tremor. At intensity `0` the filtered point is the sample itself (raw,
    /// real-time). Position only — pressure passes through.
    pub(super) fn stabilize(&mut self, sample: StrokePoint) -> StrokePoint {
        let s = self.spec.stabilizer.clamp(0.0, 1.0);
        if s <= f32::EPSILON {
            self.stab_pos = sample.pos;
            return sample;
        }
        let blend = 1.0 - s * (1.0 - STABILIZER_MIN_BLEND);
        self.stab_pos = [
            self.stab_pos[0] + (sample.pos[0] - self.stab_pos[0]) * blend,
            self.stab_pos[1] + (sample.pos[1] - self.stab_pos[1]) * blend,
        ];
        StrokePoint {
            pos: self.stab_pos,
            pressure: sample.pressure,
        }
    }
}
