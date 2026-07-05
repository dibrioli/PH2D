//! [`Adsr`] — a linear Attack/Decay/Sustain/Release amplitude envelope.
//!
//! Per-sample linear segments (no transcendentals). A voice with an envelope
//! multiplies its output by [`Adsr::tick`] and ends once the envelope is
//! [`Adsr::is_finished`] (released to zero).

/// Which segment the envelope is currently in.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum AdsrStage {
    /// Not sounding (before trigger, or after release completes).
    Idle,
    /// Rising 0 → 1 over `attack_secs`.
    Attack,
    /// Falling 1 → `sustain` over `decay_secs`.
    Decay,
    /// Held at `sustain` until release.
    Sustain,
    /// Falling to 0 over `release_secs`.
    Release,
}

/// Envelope shape, in seconds + a sustain level.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct AdsrParams {
    /// Attack time in seconds.
    pub attack_secs: f32,
    /// Decay time in seconds.
    pub decay_secs: f32,
    /// Sustain level, `0.0..=1.0`.
    pub sustain: f32,
    /// Release time in seconds.
    pub release_secs: f32,
}

impl Default for AdsrParams {
    fn default() -> Self {
        Self {
            attack_secs: 0.005,
            decay_secs: 0.05,
            sustain: 0.8,
            release_secs: 0.1,
        }
    }
}

/// A running ADSR envelope generator.
#[derive(Copy, Clone, Debug)]
pub struct Adsr {
    params: AdsrParams,
    sample_rate: f32,
    stage: AdsrStage,
    level: f32,
}

impl Adsr {
    /// Idle envelope for `params` at `sample_rate`.
    pub fn new(params: AdsrParams, sample_rate: f32) -> Self {
        Self {
            params,
            sample_rate: sample_rate.max(1.0),
            stage: AdsrStage::Idle,
            level: 0.0,
        }
    }

    /// Begin the attack segment.
    pub fn trigger(&mut self) {
        self.stage = AdsrStage::Attack;
    }

    /// Begin the release segment (note-off). No-op if already idle.
    pub fn release(&mut self) {
        if self.stage != AdsrStage::Idle {
            self.stage = AdsrStage::Release;
        }
    }

    /// Current segment.
    pub fn stage(&self) -> AdsrStage {
        self.stage
    }

    /// Current amplitude, `0.0..=1.0`.
    pub fn level(&self) -> f32 {
        self.level
    }

    /// True once released to zero — the owning voice can free its slot.
    pub fn is_finished(&self) -> bool {
        self.stage == AdsrStage::Idle && self.level <= 0.0
    }

    /// Advance one sample; returns the amplitude to apply.
    #[inline]
    pub fn tick(&mut self) -> f32 {
        let sustain = self.params.sustain.clamp(0.0, 1.0);
        match self.stage {
            AdsrStage::Idle => self.level = 0.0,
            AdsrStage::Attack => {
                self.level += self.rate(self.params.attack_secs);
                if self.level >= 1.0 {
                    self.level = 1.0;
                    self.stage = AdsrStage::Decay;
                }
            }
            AdsrStage::Decay => {
                self.level -= self.rate(self.params.decay_secs) * (1.0 - sustain);
                if self.level <= sustain {
                    self.level = sustain;
                    self.stage = AdsrStage::Sustain;
                }
            }
            AdsrStage::Sustain => self.level = sustain,
            AdsrStage::Release => {
                self.level -= self.rate(self.params.release_secs);
                if self.level <= 0.0 {
                    self.level = 0.0;
                    self.stage = AdsrStage::Idle;
                }
            }
        }
        self.level
    }

    /// Per-sample increment that spans a full unit over `secs`. Zero time = an
    /// instantaneous segment (advance the whole way in one sample).
    #[inline]
    fn rate(&self, secs: f32) -> f32 {
        if secs > 0.0 {
            1.0 / (secs * self.sample_rate)
        } else {
            1.0
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn params() -> AdsrParams {
        AdsrParams {
            attack_secs: 0.001,
            decay_secs: 0.001,
            sustain: 0.5,
            release_secs: 0.001,
        }
    }

    #[test]
    fn walks_through_all_stages() {
        let mut e = Adsr::new(params(), 48_000.0);
        assert_eq!(e.stage(), AdsrStage::Idle);
        e.trigger();
        // Attack rises to 1.0.
        let mut reached_peak = false;
        for _ in 0..200 {
            let v = e.tick();
            if v >= 1.0 - 1e-4 {
                reached_peak = true;
                break;
            }
        }
        assert!(reached_peak, "attack should reach unity");
        // Decay settles to sustain.
        for _ in 0..400 {
            e.tick();
        }
        assert_eq!(e.stage(), AdsrStage::Sustain);
        assert!((e.level() - 0.5).abs() < 1e-3);
        // Release falls to zero and finishes.
        e.release();
        for _ in 0..400 {
            e.tick();
        }
        assert!(e.is_finished(), "release should end at idle/zero");
        assert!(e.level() <= 0.0);
    }

    #[test]
    fn never_exceeds_unit_range() {
        let mut e = Adsr::new(AdsrParams::default(), 48_000.0);
        e.trigger();
        for _ in 0..48_000 {
            let v = e.tick();
            assert!((0.0..=1.0).contains(&v));
        }
    }
}
