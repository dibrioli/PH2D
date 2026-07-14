//! **Multiband compression** — the rack's compressor, run on three bands that cannot duck
//! one another, over a Linkwitz-Riley crossover.
//!
//! A single-band compressor is level-driven, and the level of most material *is* the bass:
//! a kick drum or a plosive drags the gain down and takes the whole spectrum with it, so
//! the cymbals duck every time the bass hits. Splitting the signal first means the low
//! band's own dynamics stay in the low band.
//!
//! # The crossover is the whole problem
//!
//! Two claims worth pinning, because both are easy to get wrong and neither is visible in a
//! compile:
//!
//! **1. The bands sum FLAT, not to the input.** A 4th-order Linkwitz-Riley section is *two*
//! cascaded Butterworth biquads — that cascade is the definition, and it is what makes
//! `|LP4(f) + HP4(f)| = 1` at every frequency. But the sum is an **allpass**, not an
//! identity: the magnitude is flat and the phase is rotated a full turn through the corner.
//! Reconstruction is exact in *level*, never sample-for-sample. (Measured: the summed
//! impulse differs from the input by 0.07 of full scale, while the magnitude response is
//! flat to ±0.0000 dB.) A gate that demands byte-identity from a crossover cannot be
//! satisfied by any real one, and the only way to make it pass is to loosen it until it
//! measures nothing. The gate below asks for **flat magnitude**, which is the property that
//! actually exists.
//!
//! The rack's byte-identical neutral point is kept where every other effect keeps it — in
//! [`super::Effect::is_bypass`], which short-circuits at `ratio` 1 so the crossover never
//! runs at all.
//!
//! **2. A three-way split needs the low band phase-compensated.** The obvious build is a
//! tree: split at `f1`, then split the high half at `f2`. But then the low band never
//! travelled through the second crossover, so it arrives carrying a phase the other two do
//! not have, and the sum *dips at the low corner*. It is not subtle once the corners close
//! in — measured on the naive tree:
//!
//! ```text
//!   f1=200 f2=2000 :  -0.11 dB      f1=300 f2=600 :  -3.57 dB
//!   f1=200 f2=1000 :  -0.47 dB      f1=400 f2=500 : -11.96 dB
//! ```
//!
//! The fix is standard: run the low band through the second crossover's **allpass**
//! (`LP4 + HP4` at `f2`) so all three bands carry the same rotation. That is the
//! [`Chain::allpass_high`] branch below, and with it the sum is flat to ±0.0000 dB at every
//! spacing. `the_crossover_sums_flat` is red without it.
//!
//! Control thread only, so `exp`/`powf` are free (HR-5 does not apply).

use ph2d_audio::SampleData;
use ph2d_audio::dsp::{Biquad, BiquadCoeffs};

use super::dynamics::{COMPRESS_MAX_MAKEUP, compress};
use crate::ops::channels;

/// Butterworth Q. **Two** sections at this Q, cascaded, make one 4th-order Linkwitz-Riley
/// section — see the module docs. A single section at this Q is a plain Butterworth and
/// does *not* sum flat with its complement.
pub(super) const LR_Q: f32 = std::f32::consts::FRAC_1_SQRT_2;

/// The low/mid corner: below it is body and boom — the kick, the plosive, the rumble that
/// drives a single-band compressor's gain. This is the split that stops it.
pub(super) const XOVER_LOW_HZ: f32 = 200.0;
/// The mid/high corner: above it is presence and air — sibilance, cymbals, consonants.
pub(super) const XOVER_HIGH_HZ: f32 = 2_000.0;

/// One 4th-order Linkwitz-Riley section: two identical Butterworth biquads in cascade.
#[derive(Clone, Copy)]
struct Lr4 {
    a: Biquad,
    b: Biquad,
}

impl Lr4 {
    fn new(coeffs: BiquadCoeffs) -> Self {
        Self {
            a: Biquad::new(coeffs),
            b: Biquad::new(coeffs),
        }
    }

    #[inline]
    fn process(&mut self, x: f32) -> f32 {
        self.b.process(self.a.process(x))
    }
}

/// Which of the three bands a [`Chain`] isolates.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(super) enum Band {
    Low,
    Mid,
    High,
}

/// The filter chain that isolates ONE band, for ONE channel.
///
/// For the **mid** and **high** bands this is just the two crossovers in series, and
/// `allpass_high` is `None`.
///
/// For the **low** band, `second` + `allpass_high` are the two halves of the f2 crossover's
/// **allpass** (`LP4 + HP4`), and together they are the phase compensation the module docs
/// are about. Note that it is the *pair* that matters, not the `Option` alone: this far below
/// f2 the high half carries almost no energy (two 4th-order rolloffs a decade apart is about
/// −96 dB), so dropping it changes the sum by 0.001 dB. What the low band is missing in the
/// naive tree is the f2 stage's **phase**, not its energy — which is why the mutation in
/// `without_the_phase_compensation_the_sum_dips_at_the_low_corner` has to bypass the whole
/// stage, and why a mutation that drops only the high half proves nothing.
pub(super) struct Chain {
    first: Lr4,
    second: Lr4,
    allpass_high: Option<Lr4>,
}

impl Chain {
    pub(super) fn new(which: Band, sr: f32) -> Self {
        let lp1 = BiquadCoeffs::lowpass(sr, XOVER_LOW_HZ, LR_Q);
        let hp1 = BiquadCoeffs::highpass(sr, XOVER_LOW_HZ, LR_Q);
        let lp2 = BiquadCoeffs::lowpass(sr, XOVER_HIGH_HZ, LR_Q);
        let hp2 = BiquadCoeffs::highpass(sr, XOVER_HIGH_HZ, LR_Q);
        match which {
            // Low-pass at f1 — then the f2 crossover's ALLPASS (both halves), which the
            // other two bands got for free by travelling through that split.
            Band::Low => Self {
                first: Lr4::new(lp1),
                second: Lr4::new(lp2),
                allpass_high: Some(Lr4::new(hp2)),
            },
            Band::Mid => Self {
                first: Lr4::new(hp1),
                second: Lr4::new(lp2),
                allpass_high: None,
            },
            Band::High => Self {
                first: Lr4::new(hp1),
                second: Lr4::new(hp2),
                allpass_high: None,
            },
        }
    }

    #[inline]
    pub(super) fn process(&mut self, x: f32) -> f32 {
        let v = self.first.process(x);
        let y = self.second.process(v);
        match &mut self.allpass_high {
            // `LP4(v) + HP4(v)` at f2 — the allpass.
            Some(hi) => y + hi.process(v),
            None => y,
        }
    }
}

/// Isolate one band of `data` — one allocation, written once (ADR-0117 D2).
fn isolate(data: &SampleData, which: Band, sr: f32, ch: usize) -> SampleData {
    let frames = data.frame_count();
    let src = data.samples();
    let mut chains: Vec<Chain> = (0..ch).map(|_| Chain::new(which, sr)).collect();
    SampleData::build(src.len(), data.format(), |out| {
        for f in 0..frames {
            for (c, chain) in chains.iter_mut().enumerate() {
                let i = f * ch + c;
                out[i] = chain.process(src[i]);
            }
        }
    })
}

/// Three-band compression: split, compress each band on its own, sum.
///
/// Each band runs through the **same** [`compress`] the rack's Compress is — the primed
/// envelope (no click at a selection edge) and the peak-preserving make-up come with it, and
/// "Multiband" means exactly "that compressor, per band" rather than a second, subtly
/// different one.
///
/// # The threshold is relative to each band's own peak
///
/// An absolute threshold would be the same bug [`compress`] already documents itself
/// rejecting — its textbook make-up "restores a full-scale signal, and silently amplifies
/// anything quieter". Per band, an absolute threshold silently *ignores* anything quieter:
/// real material sits 15–25 dB louder in the low band than in the high one, so a threshold
/// the bass crosses constantly is a threshold the cymbals never reach, and a three-band
/// compressor degenerates into a bass compressor with two idle bands.
///
/// So `threshold` reads as *how far below this band's own peak compression starts*, which is
/// the same number in every band. `every_band_compresses_on_tilted_material` is red without
/// it — and, deliberately, red on a *spectrally tilted* probe, because a flat one cannot
/// tell the two designs apart.
pub(super) fn multiband(
    data: &SampleData,
    threshold: f32,
    ratio: f32,
    attack_secs: f32,
    release_secs: f32,
) -> SampleData {
    let sr = data.format().sample_rate as f32;
    let ch = channels(data);
    let peak_in = crate::peak(data);

    // The output IS the accumulator: allocated once, then summed into band by band. Only one
    // band is materialised at a time (ADR-0117: an offline rack that holds every intermediate
    // buffer is how the editor reached 4351 MB).
    SampleData::build(data.samples().len(), data.format(), |acc| {
        for which in [Band::Low, Band::Mid, Band::High] {
            let band = isolate(data, which, sr, ch);
            let peak = crate::peak(&band);
            let out = compress(&band, threshold * peak, ratio, attack_secs, release_secs);
            for (a, s) in acc.iter_mut().zip(out.samples()) {
                *a += s;
            }
        }

        // Each band came back at its own peak, but summing them through the crossover's
        // allpass redistributes energy in TIME — the sum's peak is not the input's. Match it,
        // exactly as `compress` does and for the same reason: raising the ratio must reduce
        // the dynamic range, never raise the waveform's amplitude. Attenuation is unbounded
        // (it is always safe and always right); the lift is capped, so a near-silent region
        // cannot be blown up.
        let peak_out = acc.iter().fold(0.0f32, |m, s| m.max(s.abs()));
        if peak_out > f32::EPSILON && peak_in > f32::EPSILON {
            let scale = (peak_in / peak_out).min(COMPRESS_MAX_MAKEUP);
            for s in acc.iter_mut() {
                *s = (*s * scale).clamp(-1.0, 1.0);
            }
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use ph2d_audio::AudioFormat;

    const SR: usize = 48_000;
    /// A power of two, so the DFT bins land where they are asked to.
    const N: usize = 1 << 14;

    /// Magnitude response of an impulse response, in dB, over the audible band only. A
    /// naive DFT: `N` is small, this is a test, and a real FFT would drag `realfft` into a
    /// crate that deliberately does not depend on it (ADR-0122).
    fn magnitude_db(ir: &[f32]) -> Vec<(f32, f32)> {
        let mut out = Vec::new();
        let mut hz = 20.0f32;
        while hz <= 20_000.0 {
            let w = std::f32::consts::TAU * hz / SR as f32;
            let (mut re, mut im) = (0.0f64, 0.0f64);
            for (n, &x) in ir.iter().enumerate() {
                let phase = (w as f64) * n as f64;
                re += x as f64 * phase.cos();
                im -= x as f64 * phase.sin();
            }
            let mag = (re * re + im * im).sqrt().max(1e-30);
            out.push((hz, 20.0 * (mag.log10() as f32)));
            // Log sweep: a dip at a crossover is a narrow feature on a linear axis.
            hz *= 1.02;
        }
        out
    }

    /// The bar for "flat". Set from the MEASURED margin on **both** sides, not picked: the
    /// shipping crossover comes in at +0.0012 dB (f32 and truncation noise) and the naive
    /// tree dips to −0.1135 dB at 262 Hz. So the bar sits 42x clear of the truth and 2.3x
    /// clear of the bug. A bar chosen loose enough to pass is a bar that measures nothing.
    const FLATNESS_BAR_DB: f32 = 0.05;

    /// The frequency at which the response is furthest from flat, and by how much.
    fn worst_deviation(ir: &[f32]) -> (f32, f32) {
        magnitude_db(ir)
            .into_iter()
            .max_by(|a, b| a.1.abs().total_cmp(&b.1.abs()))
            .expect("the sweep is not empty")
    }

    /// Sum the three bands of an impulse, with the low band's phase compensation either
    /// wired (the shipping build) or cut (the mutation the gate has to catch).
    fn summed_impulse(compensate: bool) -> Vec<f32> {
        let sr = SR as f32;
        let mut chains: Vec<Chain> = [Band::Low, Band::Mid, Band::High]
            .into_iter()
            .map(|b| Chain::new(b, sr))
            .collect();
        if !compensate {
            // THE MUTATION: the naive tree — the low band skips the second crossover
            // ENTIRELY. Bypassing only `allpass_high` would prove nothing (that half is
            // ~-96 dB down here, and the sum stays flat to 0.001 dB); what the naive low
            // band is missing is the f2 stage's PHASE.
            chains[0].second = Lr4::new(BiquadCoeffs::identity());
            chains[0].allpass_high = None;
        }
        (0..N)
            .map(|i| {
                let x = if i == 0 { 1.0 } else { 0.0 };
                chains.iter_mut().map(|c| c.process(x)).sum()
            })
            .collect()
    }

    /// **The gate the crossover exists to pass.** The three bands, summed with no compression
    /// anywhere, must reconstruct the input's MAGNITUDE at every audible frequency — that is
    /// what a Linkwitz-Riley crossover promises, and it is the promise a multiband compressor
    /// with two idle bands is built on.
    ///
    /// Not byte-identity: an LR4 sum is an ALLPASS. See the module docs — this is the trap.
    #[test]
    fn the_crossover_sums_flat() {
        let worst = worst_deviation(&summed_impulse(true));
        println!(
            "crossover flatness: worst {:+.4} dB at {:.0} Hz",
            worst.1, worst.0
        );
        assert!(
            worst.1.abs() < FLATNESS_BAR_DB,
            "the crossover is not flat: {:+.4} dB at {:.0} Hz",
            worst.1,
            worst.0
        );
    }

    /// ...and the gate above is not measuring nothing: **cut the low band's phase
    /// compensation and it goes red.** Without this, `the_crossover_sums_flat` would pass on
    /// a build with the classic multi-way crossover bug in it, and the only evidence would be
    /// a dip nobody looked for.
    #[test]
    fn without_the_phase_compensation_the_sum_dips_at_the_low_corner() {
        let worst = worst_deviation(&summed_impulse(false));
        println!("naive tree: worst {:+.4} dB at {:.0} Hz", worst.1, worst.0);
        assert!(
            worst.1 < -FLATNESS_BAR_DB,
            "the naive tree should dip, but the worst deviation was {:+.4} dB at {:.0} Hz — \
             either the compensation is being applied anyway, or the gate is blind",
            worst.1,
            worst.0
        );
        // The dip sits at the LOW corner (the split the low band skipped), not the high one.
        assert!(
            worst.0 < XOVER_HIGH_HZ,
            "the dip should be at the low corner, but it was at {:.0} Hz",
            worst.0
        );
    }

    /// A **spectrally tilted** probe: a loud low tone and a quiet high one, which is what
    /// real material looks like (and what a flat test signal is not).
    fn tilted(low_amp: f32, high_amp: f32) -> SampleData {
        let tau = std::f32::consts::TAU;
        SampleData::from_interleaved(
            (0..SR * 2)
                .map(|i| {
                    let t = (i / 2) as f32 / SR as f32;
                    // An envelope, or a compressor has no dynamics to act on.
                    let env = 0.4 + 0.6 * (tau * 3.0 * t).sin().abs();
                    low_amp * env * (tau * 80.0 * t).sin()
                        + high_amp * env * (tau * 6_000.0 * t).sin()
                })
                .collect(),
            AudioFormat::stereo(SR as u32),
        )
    }

    /// Energy above the mid/high corner, as a stand-in for "did the high band move".
    fn high_band_rms(d: &SampleData) -> f32 {
        let sr = SR as f32;
        let mut f = Chain::new(Band::High, sr);
        let s = d.samples();
        let sum: f32 = s
            .iter()
            .step_by(2)
            .map(|&x| {
                let y = f.process(x);
                y * y
            })
            .sum();
        (sum / (s.len() / 2) as f32).sqrt()
    }

    /// **The threshold has to follow each band's own level.** On tilted material — a bass 26 dB
    /// hotter than the treble, which is ordinary — an ABSOLUTE threshold that the low band
    /// crosses constantly is one the high band never reaches, so the high band passes through
    /// untouched and the "multiband" is a bass compressor with two idle bands.
    ///
    /// Drive it hard and assert the high band's own level actually moved. A flat probe would
    /// pass either way, which is exactly the fixture-without-the-other trap.
    #[test]
    fn every_band_compresses_on_tilted_material() {
        let d = tilted(0.9, 0.045); // 26 dB of tilt
        let before = high_band_rms(&d);
        let out = multiband(&d, 0.2, 12.0, 0.005, 0.05);
        let after = high_band_rms(&out);
        // Compression lifts a band's RMS against its own peak (the make-up is peak-preserving),
        // so a band that actually worked comes back denser.
        let change_db = 20.0 * (after / before.max(1e-9)).log10();
        assert!(
            change_db.abs() > 0.5,
            "the high band came through untouched ({change_db:+.2} dB) — the threshold is not \
             following the band's own level, so only the bass is being compressed"
        );
    }

    /// The compressor's promise, kept per band: raising the ratio reduces the dynamic range
    /// and never raises the waveform's amplitude. (The allpass sum can overshoot — this is
    /// the trim that catches it.)
    #[test]
    fn it_never_raises_the_peak() {
        let d = tilted(0.9, 0.045);
        let peak_in = crate::peak(&d);
        for ratio in [2.0, 8.0, 20.0] {
            let out = multiband(&d, 0.2, ratio, 0.005, 0.05);
            let peak_out = crate::peak(&out);
            assert!(
                peak_out <= peak_in * 1.001,
                "ratio {ratio}: the peak grew from {peak_in:.4} to {peak_out:.4}"
            );
        }
    }
}
