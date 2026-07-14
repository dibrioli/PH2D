//! The **anti-alias filter** a downward resample cannot do without.
//!
//! # The bug this exists to fix
//!
//! [`crate::conform`] resamples by linear interpolation, which is fine going **up** (44.1 kHz
//! into a 48 kHz project — the paste path, and the only one that existed for a year) and
//! **broken** going down. Decimating 48 kHz to 24 kHz throws away every other sample, and
//! everything above the destination's 12 kHz Nyquist does not disappear: it **folds back down**.
//! A 15 kHz shimmer reappears as a 9 kHz tone that was never in the recording — inharmonic,
//! unmusical, and *louder* the brighter the source was.
//!
//! It went unnoticed because nothing downsampled. The W6 **Mobile** shipping target (24 kHz) is
//! the first caller that does, and it would have shipped that garbage into a game. Exactly the
//! ADR-0118 lesson, again: a routine that is correct for the caller it was written for is not
//! thereby correct — *ask the same question of the other side*.
//!
//! # The fix, ported rather than invented (DIRETIVA §1)
//!
//! Band-limit the signal to the destination's Nyquist **before** resampling — the textbook
//! decimation filter (Smith, *Digital Audio Resampling*). A **windowed-sinc FIR**: an ideal
//! low-pass is a sinc in time, and truncating it to `TAPS` and multiplying by a **Blackman**
//! window trades a slightly wider transition for a stopband deep enough (~ -74 dB) that the
//! fold-down lands under everything else in the mix.
//!
//! Linear phase (the kernel is symmetric), so the filter delays every frequency equally and the
//! delay is compensated exactly — no smearing, and a transient stays where it was.
//!
//! Not used going up: an interpolation creates no energy above the source's Nyquist, so there is
//! nothing to alias and the filter would only dull the result for free.

use ph2d_audio::{AudioFormat, SampleData};

use crate::ops::channels;

/// Kernel length. Odd, so the kernel is symmetric about a whole sample and the group delay is an
/// exact integer — which is what lets the delay be compensated by a shift rather than a
/// resample. 127 taps at 48 kHz put the transition band at roughly 2 kHz, which is narrow enough
/// that a 24 kHz target keeps everything up to ~10.8 kHz and rejects 15 kHz outright.
const TAPS: usize = 127;

/// Cut-off as a fraction of the destination's **Nyquist**, not of its rate: the guard band is
/// what the transition needs to reach the stopband before the fold-over frequency, and 0.9 of
/// Nyquist leaves it exactly that.
const CUTOFF_FRAC: f32 = 0.9;

/// A Blackman-windowed sinc low-pass at `cutoff_hz`, normalised to unity DC gain.
///
/// Unity DC gain is not cosmetic: a kernel that summed to 1.02 would make every resampled clip
/// 0.17 dB louder than its source, and the level would drift a little more with every conform.
fn kernel(sr: f32, cutoff_hz: f32) -> Vec<f32> {
    let m = (TAPS - 1) as f32;
    let fc = (cutoff_hz / sr).clamp(1e-4, 0.5); // cycles per sample
    let pi = std::f32::consts::PI;
    let mut h: Vec<f32> = (0..TAPS)
        .map(|i| {
            let n = i as f32 - m * 0.5;
            // sinc, with the removable singularity at the centre tap taken by its limit.
            let s = if n.abs() < 1e-6 {
                2.0 * fc
            } else {
                (2.0 * pi * fc * n).sin() / (pi * n)
            };
            // Blackman: ~ -74 dB stopband, which is where a fold-down stops mattering.
            let w = 0.42 - 0.5 * (2.0 * pi * i as f32 / m).cos()
                + 0.08 * (4.0 * pi * i as f32 / m).cos();
            s * w
        })
        .collect();
    let sum: f32 = h.iter().sum();
    if sum.abs() > 1e-9 {
        for v in &mut h {
            *v /= sum;
        }
    }
    h
}

/// Band-limit `src` to `target`'s Nyquist, in place of nothing — returns a filtered copy, ready
/// to be resampled down. **A no-op (and byte-identical) when the target rate is not lower**,
/// because there is then nothing to fold.
pub(crate) fn band_limit(src: &SampleData, target: AudioFormat) -> SampleData {
    let src_rate = src.format().sample_rate;
    if target.sample_rate >= src_rate || src_rate == 0 {
        return src.clone();
    }
    let sr = src_rate as f32;
    let nyquist = target.sample_rate as f32 * 0.5;
    let h = kernel(sr, nyquist * CUTOFF_FRAC);

    let ch = channels(src);
    let frames = src.frame_count();
    let s = src.samples();
    // Linear phase: the kernel is symmetric, so it delays everything by exactly half its length.
    // Read that far ahead and the output lands back on the input's timeline — a transient that
    // came out late would be a *worse* bug than the aliasing, and silent.
    let delay = TAPS / 2;

    SampleData::build(s.len(), src.format(), |out| {
        for f in 0..frames {
            for c in 0..ch {
                let mut acc = 0.0f32;
                for (k, &coeff) in h.iter().enumerate() {
                    // The tap this output sample reads, already un-delayed.
                    let t = f + k;
                    if t >= delay && t - delay < frames {
                        acc += coeff * s[(t - delay) * ch + c];
                    }
                }
                out[f * ch + c] = acc;
            }
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use ph2d_audio::ChannelLayout;

    const SR: u32 = 48_000;

    fn tone(hz: f32) -> SampleData {
        let tau = std::f32::consts::TAU;
        let n = SR as usize;
        SampleData::from_interleaved(
            (0..n * 2)
                .map(|i| {
                    let t = (i / 2) as f32 / SR as f32;
                    0.5 * (tau * hz * t).sin()
                })
                .collect(),
            AudioFormat::new(SR, ChannelLayout::Stereo),
        )
    }

    /// RMS, skipping the edges where the kernel is still running off the end of the buffer.
    fn rms(d: &SampleData) -> f32 {
        let s: Vec<f32> = d.samples().iter().step_by(2).copied().collect();
        let body = &s[TAPS..s.len() - TAPS];
        (body.iter().map(|x| x * x).sum::<f32>() / body.len() as f32).sqrt()
    }

    /// **What must not survive.** A 15 kHz tone, band-limited for a 24 kHz target (Nyquist
    /// 12 kHz), has to be gone — if it is not, it folds to 9 kHz on the way down and the game
    /// gets a tone the recording never had.
    #[test]
    fn content_above_the_targets_nyquist_is_removed() {
        let target = AudioFormat::new(24_000, ChannelLayout::Stereo);
        let before = rms(&tone(15_000.0));
        let after = rms(&band_limit(&tone(15_000.0), target));
        let db = 20.0 * (after / before.max(1e-9)).log10();
        println!("15 kHz into a 24 kHz target: {db:.1} dB");
        assert!(
            db < -40.0,
            "a 15 kHz tone survived at {db:.1} dB -- it will fold down to 9 kHz and there is \
             nothing downstream that can tell it from the signal"
        );
    }

    /// **What must survive.** The filter is an anti-alias filter, not a tone control: a 1 kHz
    /// tone is nowhere near the fold-over frequency and must come through at its own level.
    /// Without this, "attenuates 15 kHz" would also be satisfied by attenuating everything.
    #[test]
    fn content_the_target_can_hold_is_left_alone() {
        let target = AudioFormat::new(24_000, ChannelLayout::Stereo);
        let before = rms(&tone(1_000.0));
        let after = rms(&band_limit(&tone(1_000.0), target));
        let db = 20.0 * (after / before.max(1e-9)).log10();
        println!("1 kHz into a 24 kHz target: {db:.2} dB");
        assert!(
            db.abs() < 0.5,
            "the anti-alias filter moved a 1 kHz tone by {db:.2} dB -- it is behaving like an EQ"
        );
    }

    /// **Going UP is byte-identical.** An interpolation creates no energy above the source's
    /// Nyquist, so there is nothing to alias — filtering there would only dull the result, for
    /// free, on the paste path that has worked for a year.
    #[test]
    fn an_upward_conform_is_not_filtered_at_all() {
        let d = tone(15_000.0);
        let up = band_limit(&d, AudioFormat::new(96_000, ChannelLayout::Stereo));
        assert_eq!(
            up.samples(),
            d.samples(),
            "an upward resample was band-limited -- it has nothing to fold, and the filter is \
             throwing away the top of the paste path for nothing"
        );
        // ...and the same rate, likewise.
        let same = band_limit(&d, d.format());
        assert_eq!(same.samples(), d.samples());
    }

    /// **The filter does not move the audio in time.** The kernel is symmetric, so it delays
    /// every frequency by exactly half its length, and that delay is compensated. An
    /// uncompensated one would shift every conformed clip by 1.3 ms — inaudible on a pad, and a
    /// flam on a drum, which is the kind of bug that gets blamed on the drummer.
    #[test]
    fn the_filter_is_linear_phase_and_the_delay_is_compensated() {
        // A click, well inside the buffer.
        let n = SR as usize;
        let at = n / 2;
        let d = SampleData::from_interleaved(
            (0..n * 2)
                .map(|i| if i / 2 == at { 1.0 } else { 0.0 })
                .collect(),
            AudioFormat::new(SR, ChannelLayout::Stereo),
        );
        let out = band_limit(&d, AudioFormat::new(24_000, ChannelLayout::Stereo));
        // The peak of the filtered click is where the click was.
        let peak_at = out
            .samples()
            .iter()
            .step_by(2)
            .enumerate()
            .max_by(|a, b| a.1.abs().total_cmp(&b.1.abs()))
            .map(|(i, _)| i)
            .expect("not empty");
        assert_eq!(
            peak_at, at,
            "the click moved from {at} to {peak_at} -- the group delay is not compensated"
        );
    }
}
