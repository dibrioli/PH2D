//! Denoise — learn what the noise looks like, then take that shape out of every column.
//!
//! ## The trap this module exists to avoid
//!
//! The obvious algorithm is spectral subtraction: measure the noise magnitude per bin, and
//! subtract it. It works, it is four lines long, and it produces **musical noise** — the
//! artefact that has sunk more denoisers than any other.
//!
//! The mechanism is worth stating, because it explains every design choice below. The noise
//! in a bin is *random*: the profile is its average, so at any instant the real noise is
//! sometimes above that average and sometimes below. Subtract the average and the bins that
//! happened to be quiet go to zero, while the bins that happened to be loud survive as
//! isolated, short-lived peaks. Those survivors are tones — a scatter of random bins,
//! blinking on and off — and the ear, which forgave the original hiss instantly, hears
//! them as a shower of little bells. The measured noise floor went *down*; the recording
//! got *worse*.
//!
//! ## The fix: decide from history, not from this instant
//!
//! The gain applied to a bin is driven by the **a priori SNR** — an estimate of how much
//! signal is in that bin — and the decision-directed rule (Ephraim & Malah, 1984) builds it
//! mostly from *what was there last column* rather than from the noisy instantaneous
//! measurement:
//!
//! ```text
//! ξ(c) = α · Ŝ²(c-1)/N²  +  (1-α) · max(γ(c) - 1, 0)          α = 0.98
//! G(c) = ξ / (1 + ξ)                                          (Wiener)
//! ```
//!
//! With α at 0.98 the gain is a heavily smoothed quantity. A bin cannot flicker: a random
//! upward excursion of the noise moves ξ by 2 % of the way, which is nowhere near enough to
//! open the gate. A real signal, present column after column, walks it open and holds it
//! there. The blinking stops — which is precisely what `the_residual_does_not_sparkle`
//! measures, and precisely what plain subtraction fails.
//!
//! A **gain floor** finishes the job: the residual is attenuated, never annihilated. A bed
//! of quiet hiss is what the ear expects under a recording, and leaving it there is what
//! keeps the survivors from standing out against a dead-silent background.

use crate::stft::{Stft, magnitude};
use ph2d_audio::SampleData;
use realfft::num_complex::Complex32;
use std::ops::Range;

/// The decision-directed smoothing constant (Ephraim & Malah). 0.98 is the canonical value:
/// high enough to stop bins flickering, low enough that a real onset still opens the gain
/// within a few columns (~10 ms at this hop) rather than being swallowed.
const ALPHA: f32 = 0.98;

/// Noise floor, per bin — the shape of what is to be removed.
#[derive(Debug, Clone)]
pub struct NoiseProfile {
    mag: Vec<f32>,
    sample_rate: u32,
}

impl NoiseProfile {
    /// Learn the noise from a stretch of the clip that is **only** noise — the room tone
    /// before the take, the hiss between two words.
    ///
    /// Averaged over columns and channels. `None` when the range is too short to hold a
    /// column, because a profile learned from nothing would silently gate the whole clip.
    pub fn learn(data: &SampleData, frames: Range<usize>) -> Option<Self> {
        let format = data.format();
        let channels = format.channel_count().max(1);
        let total = data.frame_count();
        let lo = frames.start.min(total);
        let hi = frames.end.min(total);
        if hi <= lo {
            return None;
        }
        let mut stft = Stft::new();
        let bins = stft.bins();
        let mut mag = vec![0.0f32; bins];
        let mut counted = 0usize;
        for ch in 0..channels {
            let x: Vec<f32> = (lo..hi)
                .map(|f| data.samples()[f * channels + ch])
                .collect();
            // Skip the columns that straddle the padding: they see zeros the clip does not
            // contain, and would drag the profile down (and under-subtract for it).
            let edge = stft.window_size() / stft.hop();
            let cols = stft.columns(x.len());
            if cols <= 2 * edge {
                continue;
            }
            stft.analyze(&x, |col, spec| {
                if col < edge || col >= cols - edge {
                    return;
                }
                for (m, c) in mag.iter_mut().zip(spec) {
                    *m += magnitude(*c);
                }
                counted += 1;
            });
        }
        if counted == 0 {
            return None;
        }
        for m in &mut mag {
            *m /= counted as f32;
        }
        Some(Self {
            mag,
            sample_rate: format.sample_rate,
        })
    }

    /// The learned magnitude per bin.
    pub fn bins(&self) -> &[f32] {
        &self.mag
    }

    /// The rate the profile was learned at — a profile from one clip does not describe
    /// another clip's noise if the rates differ.
    pub fn sample_rate(&self) -> u32 {
        self.sample_rate
    }
}

/// Suppress `profile`'s noise in `data`. `amount` runs 0 (bypass) to 1 (as hard as this
/// gets).
///
/// **`amount == 0` returns the input untouched, byte for byte** — the same neutral-point
/// guarantee every effect in the rack makes. A tool whose "off" is a resynthesis is a tool
/// you cannot A/B.
pub fn denoise(data: &SampleData, profile: &NoiseProfile, amount: f32) -> SampleData {
    let format = data.format();
    if amount <= 0.0 || data.is_empty() || profile.sample_rate != format.sample_rate {
        return data.clone();
    }
    let amount = amount.min(1.0);
    let channels = format.channel_count().max(1);
    let frames = data.frame_count();

    // **There is no over-subtraction here, and that is a decision, not an omission.**
    //
    // The textbook move is to subtract MORE than the measured average — the profile is a
    // mean, so half the noise sits above it — on the theory that the margin is what stops
    // the loud excursions from surviving as musical noise. This code did that, and the
    // justification was written down with confidence. It is false. Measured (audit
    // 2026-07-12), at full amount, over-subtracting by 3×:
    //
    // | | SNR | residual hiss |
    // |---|---|---|
    // | with over-subtraction | 8.2 dB | -30.0 dB |
    // | without | **14.9 dB** | -25.2 dB |
    //
    // It buys 4.8 dB more suppression and costs 6.7 dB of the signal — it distorts more
    // than it removes. And the coefficient of variation (the musical-noise measure) does not
    // budge without it: the smoothing is what handles the excursions, exactly as the module
    // docs above say, and the over-subtraction was doing nothing but damage.
    //
    // What remains is the gain floor: the residual is attenuated, never annihilated. A bed of
    // quiet hiss is what the ear expects under a recording, and leaving it there is what keeps
    // any survivor from standing out against a dead-silent background. `amount` sets how deep.
    let floor_db = -(6.0 + 24.0 * amount);
    let floor = 10f32.powf(floor_db / 20.0);

    let mut stft = Stft::new();
    let bins = stft.bins();
    let mut out = vec![0.0f32; frames * channels];
    for ch in 0..channels {
        let x: Vec<f32> = (0..frames)
            .map(|f| data.samples()[f * channels + ch])
            .collect();
        // Ŝ² of the previous column, per bin — the memory the decision-directed rule is
        // built on, and the only reason this does not sparkle.
        let mut prev = vec![0.0f32; bins];
        let y = stft.process(&x, |_, spec| {
            for (b, c) in spec.iter_mut().enumerate() {
                let noise = profile.mag.get(b).copied().unwrap_or(0.0);
                let n2 = noise * noise;
                if n2 <= 0.0 {
                    continue;
                }
                let y2 = c.re * c.re + c.im * c.im;
                // Posterior SNR: what this instant claims. Noisy, and on its own it is the
                // road to musical noise.
                let gamma = y2 / n2;
                // A priori SNR: mostly what the PREVIOUS column concluded. This is the
                // whole trick.
                let xi = ALPHA * (prev[b] / n2) + (1.0 - ALPHA) * (gamma - 1.0).max(0.0);
                let gain = (xi / (1.0 + xi)).max(floor);
                *c = Complex32::new(c.re * gain, c.im * gain);
                prev[b] = y2 * gain * gain;
            }
        });
        for f in 0..frames {
            out[f * channels + ch] = y[f];
        }
    }
    SampleData::from_interleaved(out, format)
}

#[cfg(test)]
mod tests {
    use super::*;
    use ph2d_audio::{AudioFormat, ChannelLayout};

    const SR: f32 = 48_000.0;

    fn mono(s: Vec<f32>) -> SampleData {
        SampleData::from_interleaved(
            s,
            AudioFormat {
                sample_rate: 48_000,
                channels: ChannelLayout::Mono,
            },
        )
    }

    /// A cheap deterministic white-noise source — a test may not be flaky, so no `rand`.
    fn noise(n: usize, amp: f32, seed: u64) -> Vec<f32> {
        let mut s = seed | 1;
        (0..n)
            .map(|_| {
                // splitmix64, the same generator the brush jitter uses.
                s = s.wrapping_add(0x9E37_79B9_7F4A_7C15);
                let mut z = s;
                z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
                z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
                z ^= z >> 31;
                ((z >> 40) as f32 / 8_388_608.0 - 1.0) * amp
            })
            .collect()
    }

    /// Speech-ish: harmonics of 150 Hz under a slow envelope.
    fn speech(n: usize, offset: usize) -> Vec<f32> {
        (0..n)
            .map(|i| {
                let t = (i + offset) as f32 / SR;
                let env = 0.5 + 0.5 * (std::f32::consts::TAU * 2.0 * t).sin();
                let h: f32 = (1..=8)
                    .map(|k| (std::f32::consts::TAU * 150.0 * k as f32 * t).sin() / k as f32)
                    .sum();
                h * env * 0.25
            })
            .collect()
    }

    fn energy(x: &[f32]) -> f32 {
        x.iter().map(|s| s * s).sum::<f32>().max(1e-20)
    }

    /// SNR of `got` against the clean reference, in dB.
    fn snr(clean: &[f32], got: &[f32]) -> f32 {
        let err: f32 = clean
            .iter()
            .zip(got)
            .map(|(c, g)| (c - g) * (c - g))
            .sum::<f32>()
            .max(1e-20);
        10.0 * (energy(clean) / err).log10()
    }

    /// Build: 0.5 s of noise alone (the profile is learned here), then 1 s of speech at
    /// 0 dB SNR — signal and noise carrying equal energy, the level the gate is set at.
    fn fixture() -> (SampleData, Vec<f32>, Range<usize>, Range<usize>) {
        let (lead, body) = (24_000usize, 48_000usize);
        let clean = speech(body, 0);
        let target = energy(&clean) / body as f32;
        // Scale the noise so its per-sample energy matches the signal's: SNR = 0 dB.
        let raw = noise(lead + body, 1.0, 0x5EED);
        let raw_pow = energy(&raw[lead..]) / body as f32;
        let k = (target / raw_pow).sqrt();
        let mut s = vec![0.0f32; lead + body];
        for (i, v) in s.iter_mut().enumerate() {
            *v = raw[i] * k + if i >= lead { clean[i - lead] } else { 0.0 };
        }
        (mono(s), clean, 0..lead, lead..lead + body)
    }

    /// **The acceptance gate (ADR-0115 §4.3): ≥ 8 dB of SNR improvement at 0 dB in.**
    #[test]
    fn denoise_lifts_the_signal_out_of_the_noise() {
        let (data, clean, profile_range, body) = fixture();
        let profile = NoiseProfile::learn(&data, profile_range).expect("profile");
        let out = denoise(&data, &profile, 0.6);

        let before = snr(&clean, &data.samples()[body.clone()]);
        let after = snr(&clean, &out.samples()[body]);
        let gain = after - before;
        assert!(
            before.abs() < 1.0,
            "the fixture is not at 0 dB SNR ({before:.1} dB) — the gate would be measuring the wrong thing"
        );
        assert!(
            gain >= 8.0,
            "denoise only bought {gain:.1} dB (in {before:.1} → out {after:.1}); the gate is 8 dB"
        );
    }

    /// **…and it actually removes the hiss.**
    ///
    /// SNR alone is a gameable target: a denoiser that does almost nothing scores well on it,
    /// because it distorts almost nothing. So the amount of noise ACTUALLY taken out is
    /// pinned too. The pair is the contract — take the hiss out (this), and do not wreck the
    /// signal doing it (the gate above). Measured at amount 0.6: -19.8 dB of residual hiss.
    ///
    /// This is the gate that would have caught the over-subtraction being removed if removing
    /// it had cost real suppression. It did not (audit 2026-07-12) — but nothing was watching
    /// until now.
    #[test]
    fn denoise_actually_removes_the_hiss() {
        let (data, _, profile_range, _) = fixture();
        let profile = NoiseProfile::learn(&data, profile_range.clone()).expect("profile");
        let out = denoise(&data, &profile, 0.6);

        let rms = |v: &[f32]| (v.iter().map(|s| s * s).sum::<f32>() / v.len() as f32).sqrt();
        let before = rms(&data.samples()[profile_range.clone()]);
        let after = rms(&out.samples()[profile_range]);
        let atten = 20.0 * (after / before.max(1e-20)).log10();
        assert!(
            atten <= -15.0,
            "the hiss only dropped {atten:.1} dB — the denoiser is protecting the signal by \
             declining to do its job"
        );
    }

    /// **The musical-noise gate (ADR-0115 §4.3, the "sem musical noise" half).**
    ///
    /// Musical noise is a *statistical* signature, not a level: the residual stops being a
    /// smooth hiss and becomes a scatter of bins blinking on and off. That shows up as a
    /// rise in each bin's coefficient of variation over time — the residual sparkles.
    ///
    /// So: measure the CoV of the noise before, and of the residual after. It may fall; it
    /// may not meaningfully rise.
    ///
    /// **The threshold is set from the measurement, not from taste.** With the
    /// decision-directed rule the residual comes back at a CoV *ratio of 1.00* — literally
    /// as smooth as the hiss it replaced. Swap in plain spectral subtraction (keep only
    /// `max(γ-1, 0)`, drop the memory) and the ratio jumps to **1.25–1.36** across every
    /// amount. 1.10 sits between the two with room on both sides. The first draft of this
    /// gate allowed 1.5, and *passed with plain subtraction* — a loose oracle proves
    /// nothing, and this one was caught by mutating the code it was supposed to be guarding
    /// ([[feedback_loose_oracle_hides_systematic_bias]]).
    #[test]
    fn the_residual_does_not_sparkle() {
        let (data, _, profile_range, _) = fixture();
        let profile = NoiseProfile::learn(&data, profile_range.clone()).expect("profile");
        let out = denoise(&data, &profile, 0.6);

        let before = cov(&data, profile_range.clone());
        let after = cov(&out, profile_range);
        assert!(
            after <= before * 1.10,
            "the residual sparkles: coefficient of variation went {before:.3} → {after:.3} \
             (ratio {:.2}; plain spectral subtraction scores 1.25-1.36 here, the \
             decision-directed rule scores 1.00 — this is musical noise)",
            after / before
        );
    }

    /// Mean coefficient of variation (std/mean) of each bin's magnitude across columns —
    /// how much each bin flickers over time.
    fn cov(data: &SampleData, frames: Range<usize>) -> f32 {
        let mut stft = Stft::new();
        let bins = stft.bins();
        let x: Vec<f32> = data.samples()[frames].to_vec();
        let (mut sum, mut sq, mut cols) = (vec![0.0f32; bins], vec![0.0f32; bins], 0usize);
        let edge = stft.window_size() / stft.hop();
        let total = stft.columns(x.len());
        stft.analyze(&x, |col, spec| {
            if col < edge || col + edge >= total {
                return;
            }
            for (b, c) in spec.iter().enumerate() {
                let m = magnitude(*c);
                sum[b] += m;
                sq[b] += m * m;
            }
            cols += 1;
        });
        if cols == 0 {
            return 0.0;
        }
        let n = cols as f32;
        let mut acc = 0.0;
        let mut used = 0;
        for b in 0..bins {
            let mean = sum[b] / n;
            if mean <= 1e-9 {
                continue;
            }
            let var = (sq[b] / n - mean * mean).max(0.0);
            acc += var.sqrt() / mean;
            used += 1;
        }
        if used == 0 { 0.0 } else { acc / used as f32 }
    }

    /// **Zero is bypass, byte for byte** — the rack's non-negotiable invariant. An "off"
    /// that quietly resynthesises the clip is an "off" you cannot A/B against.
    #[test]
    fn amount_zero_is_byte_identical() {
        let (data, _, profile_range, _) = fixture();
        let profile = NoiseProfile::learn(&data, profile_range).expect("profile");
        let out = denoise(&data, &profile, 0.0);
        assert_eq!(data.samples(), out.samples());
    }

    /// A profile cannot be learned from nothing. Returning a zeroed profile instead would
    /// gate the entire clip to the floor — a silent clip, from a silent mistake.
    #[test]
    fn an_empty_range_learns_nothing() {
        let (data, ..) = fixture();
        assert!(NoiseProfile::learn(&data, 100..100).is_none());
    }
}
