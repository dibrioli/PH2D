//! [`Reverb`] — Freeverb, the public-domain Schroeder/Moorer reverb by Jezar at
//! Dreampoint (8 parallel damped-comb filters into 4 series allpasses, per
//! channel, with a stereo spread). Ported faithfully (DIRETIVA §1: published
//! reference, not invented) — the tunings are the canonical Freeverb values.
//! Presentation math, HR-5 exempt.
//!
//! All delay buffers are allocated once at construction (sized to the sample
//! rate); [`Reverb::process`] never allocates (HR-3).

/// Comb delay lengths at the reference rate (44.1 kHz), scaled at construction.
const COMB_TUNING: [usize; 8] = [1116, 1188, 1277, 1356, 1422, 1491, 1557, 1617];
/// Allpass delay lengths at the reference rate.
const ALLPASS_TUNING: [usize; 4] = [556, 441, 341, 225];
/// Right-channel delay offset for stereo decorrelation.
const STEREO_SPREAD: usize = 23;
/// Input gain compensating the parallel comb sum.
const FIXED_GAIN: f32 = 0.015;
/// `damp` → comb damping coefficient scale.
const SCALE_DAMP: f32 = 0.4;
/// `room_size` → comb feedback scale + offset (feedback = room·scale + offset).
const SCALE_ROOM: f32 = 0.28;
const OFFSET_ROOM: f32 = 0.7;
/// Allpass feedback (fixed in Freeverb).
const ALLPASS_FEEDBACK: f32 = 0.5;
/// The rate the tunings are quoted at.
const REF_RATE: f32 = 44_100.0;

/// One damped feedback-comb filter.
struct Comb {
    buf: Vec<f32>,
    idx: usize,
    store: f32,
    feedback: f32,
    damp1: f32,
    damp2: f32,
}

impl Comb {
    fn new(size: usize) -> Self {
        Self {
            buf: vec![0.0; size.max(1)],
            idx: 0,
            store: 0.0,
            feedback: 0.5,
            damp1: 0.0,
            damp2: 1.0,
        }
    }

    #[inline]
    fn process(&mut self, input: f32) -> f32 {
        let out = self.buf[self.idx];
        // One-pole lowpass in the feedback path = high-frequency damping.
        self.store = out * self.damp2 + self.store * self.damp1;
        self.buf[self.idx] = input + self.store * self.feedback;
        self.idx += 1;
        if self.idx >= self.buf.len() {
            self.idx = 0;
        }
        out
    }

    fn set(&mut self, feedback: f32, damp: f32) {
        self.feedback = feedback;
        self.damp1 = damp;
        self.damp2 = 1.0 - damp;
    }
}

/// One Schroeder allpass filter.
struct Allpass {
    buf: Vec<f32>,
    idx: usize,
}

impl Allpass {
    fn new(size: usize) -> Self {
        Self {
            buf: vec![0.0; size.max(1)],
            idx: 0,
        }
    }

    #[inline]
    fn process(&mut self, input: f32) -> f32 {
        let bufout = self.buf[self.idx];
        let out = -input + bufout;
        self.buf[self.idx] = input + bufout * ALLPASS_FEEDBACK;
        self.idx += 1;
        if self.idx >= self.buf.len() {
            self.idx = 0;
        }
        out
    }
}

/// Stereo Freeverb. Feed it the dry stereo signal; it returns the wet stereo
/// pair. The caller mixes wet/dry.
pub struct Reverb {
    combs_l: [Comb; 8],
    combs_r: [Comb; 8],
    aps_l: [Allpass; 4],
    aps_r: [Allpass; 4],
    room_size: f32,
    damp: f32,
}

impl Reverb {
    /// Build the filter bank sized for `sample_rate`.
    pub fn new(sample_rate: u32) -> Self {
        let scale = sample_rate as f32 / REF_RATE;
        let scaled = |base: usize, spread: usize| (((base + spread) as f32) * scale) as usize;
        let mut r = Self {
            combs_l: std::array::from_fn(|i| Comb::new(scaled(COMB_TUNING[i], 0))),
            combs_r: std::array::from_fn(|i| Comb::new(scaled(COMB_TUNING[i], STEREO_SPREAD))),
            aps_l: std::array::from_fn(|i| Allpass::new(scaled(ALLPASS_TUNING[i], 0))),
            aps_r: std::array::from_fn(|i| Allpass::new(scaled(ALLPASS_TUNING[i], STEREO_SPREAD))),
            room_size: 0.5,
            damp: 0.5,
        };
        r.update();
        r
    }

    /// `room_size` (0..1, decay length) and `damp` (0..1, HF absorption).
    pub fn set_params(&mut self, room_size: f32, damp: f32) {
        self.room_size = room_size.clamp(0.0, 1.0);
        self.damp = damp.clamp(0.0, 1.0);
        self.update();
    }

    fn update(&mut self) {
        let feedback = self.room_size * SCALE_ROOM + OFFSET_ROOM;
        let damp = self.damp * SCALE_DAMP;
        for c in self.combs_l.iter_mut().chain(self.combs_r.iter_mut()) {
            c.set(feedback, damp);
        }
    }

    /// One stereo sample in → the wet stereo pair out.
    #[inline]
    pub fn process(&mut self, in_l: f32, in_r: f32) -> (f32, f32) {
        let input = (in_l + in_r) * FIXED_GAIN;
        let mut out_l = 0.0;
        let mut out_r = 0.0;
        // Combs run in parallel (summed)…
        for c in self.combs_l.iter_mut() {
            out_l += c.process(input);
        }
        for c in self.combs_r.iter_mut() {
            out_r += c.process(input);
        }
        // …then the allpasses run in series.
        for a in self.aps_l.iter_mut() {
            out_l = a.process(out_l);
        }
        for a in self.aps_r.iter_mut() {
            out_r = a.process(out_r);
        }
        (out_l, out_r)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn silence_in_stays_silent() {
        let mut r = Reverb::new(48_000);
        for _ in 0..1000 {
            let (l, rr) = r.process(0.0, 0.0);
            assert_eq!((l, rr), (0.0, 0.0));
        }
    }

    #[test]
    fn impulse_rings_then_decays_and_stays_finite() {
        let mut r = Reverb::new(48_000);
        r.set_params(0.5, 0.5);
        // Impulse in.
        let (l0, _) = r.process(1.0, 1.0);
        // Run the tail; capture the early energy and the late energy.
        let mut early = l0.abs();
        for _ in 0..4_000 {
            let (l, rr) = r.process(0.0, 0.0);
            assert!(
                l.is_finite() && rr.is_finite(),
                "reverb produced a non-finite sample"
            );
            early = early.max(l.abs());
        }
        assert!(early > 0.0, "an impulse must produce a wet tail");
        let mut late = 0.0f32;
        for _ in 0..40_000 {
            let (l, _) = r.process(0.0, 0.0);
            late = late.max(l.abs());
        }
        assert!(
            late < early,
            "the tail must decay over time (early {early} → late {late})"
        );
    }

    #[test]
    fn stereo_spread_decorrelates_channels() {
        // Identical mono input, but the L/R banks differ (stereo spread), so the
        // wet channels are not identical.
        let mut r = Reverb::new(48_000);
        let mut differed = false;
        r.process(1.0, 1.0);
        for _ in 0..2_000 {
            let (l, rr) = r.process(0.0, 0.0);
            if (l - rr).abs() > 1e-6 {
                differed = true;
            }
        }
        assert!(differed, "stereo spread must decorrelate L and R");
    }
}
