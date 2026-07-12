//! De-Clip — put back the peaks a converter cut off.
//!
//! Clipping is *destruction*, not distortion: a signal ran past what the medium could
//! hold, and everything above the ceiling was replaced by the ceiling. The waveform grows a
//! **flat top**. Nothing in the file remembers how high the peak was going — it is a hole,
//! and it has to be reconstructed rather than filtered.
//!
//! Which makes this the same problem as De-Click, wearing a different hat. A click is a
//! stretch of samples the signal's own model cannot explain; a clipped plateau is a stretch
//! of samples the signal never got to write. Both are **gaps**, and the LSAR interpolation
//! next door ([`lsar_interpolate`]) fills a gap with the samples that continue the
//! surrounding signal most plausibly. De-Click brings its own detector for what a click is;
//! this brings one for what a flat top is. The repair is the same code, and deliberately so.
//!
//! ## What "flat top" means, exactly
//!
//! Not merely "loud". A real peak can touch full scale for one sample and be perfectly
//! intact. What marks *clipping* is a **run**: several consecutive samples pinned at the
//! same level, near the ceiling, with the slope gone out of them. Requiring the run is what
//! keeps this from mangling a clean, hot master — and `a_hot_but_clean_peak_is_left_alone`
//! is the gate that says so.

use std::ops::Range;

use ph2d_audio::SampleData;

use super::lpc::{autocorrelation, levinson, lsar_interpolate};
use crate::ops::channels;

/// AR model order. Same as De-Click: enough poles to describe a voice or an instrument
/// across the block, few enough to stay well-conditioned.
const ORDER: usize = 32;

/// Analysis block. The model is refitted per block, so it tracks a changing signal.
pub(super) const BLOCK: usize = 2_048;

/// The longest plateau worth rebuilding (~4 ms at 48 kHz). Past this the audio above the
/// ceiling is not a clipped *peak* any more, it is a clipped *passage*, and there is no
/// longer enough intact signal around it for the model to have an opinion. Left alone — a
/// wrong reconstruction is worse than an honest flat top.
const MAX_RUN: usize = 192;

/// The shortest run that counts as clipping. Two samples at the ceiling is a peak; a
/// converter that has actually run out of headroom holds there.
const MIN_RUN: usize = 3;

/// How still a run has to be to count as flat, relative to the ceiling. A clipped plateau
/// is not perfectly constant (dither, DC, a lossy codec's ringing), so "flat" has to mean
/// *nearly*, not *exactly* — an exact test would find nothing in a real file.
const FLATNESS: f64 = 0.02;

/// Restore clipped peaks. `threshold` is the level (0..1, of full scale) at or above which
/// a flat run counts as clipped; `amount` blends the reconstruction in — **0 is a
/// byte-identical bypass**, which is the rack's non-negotiable neutral point.
pub(super) fn declip(data: &SampleData, threshold: f32, amount: f32) -> SampleData {
    if amount <= 0.0 {
        return data.clone();
    }
    let ch = channels(data);
    let frames = data.frame_count();
    let amount = f64::from(amount.clamp(0.0, 1.0));
    let threshold = f64::from(threshold).clamp(0.1, 1.0);
    let mut out = data.samples().to_vec();

    for c in 0..ch {
        let mut x: Vec<f64> = (0..frames)
            .map(|f| f64::from(data.samples()[f * ch + c]))
            .collect();
        let dry = x.clone();
        let mut start = 0;
        while start < frames {
            let end = (start + BLOCK).min(frames);
            repair_block(&mut x, start..end, threshold);
            start = end;
        }
        // Blend, so Amount is a real control and 0 is exactly the input. The reconstruction
        // only ever *raises* a clipped sample away from the ceiling; blending it back in
        // partially is a legitimate "restore some of the peak" and never an artefact.
        for f in 0..frames {
            let v = dry[f] + (x[f] - dry[f]) * amount;
            out[f * ch + c] = v as f32;
        }
    }
    SampleData::from_interleaved(out, data.format())
}

/// Find the flat tops in one block and rebuild each from the signal around it.
fn repair_block(x: &mut [f64], block: Range<usize>, threshold: f64) {
    let lo = block.start;
    let hi = block.end;
    if hi - lo <= ORDER * 2 {
        return;
    }
    let runs = flat_runs(&x[lo..hi], threshold);
    if runs.is_empty() {
        return;
    }
    // Fit the model to the block as it stands. The plateaus bias it slightly (they are the
    // damage), but they are a small fraction of the block, and this is the same bargain
    // De-Click makes with its bursts.
    let Some((a, _)) = levinson(&autocorrelation(&x[lo..hi], ORDER)) else {
        return;
    };
    for run in runs {
        let gap = (lo + run.start)..(lo + run.end);
        // The interpolation needs intact signal on BOTH sides to lean on. A plateau flush
        // against the edge of the block has only one, so it waits for a block whose
        // boundaries fall elsewhere rather than being rebuilt from half a picture.
        if gap.start < ORDER || gap.end + ORDER > x.len() {
            continue;
        }
        lsar_interpolate(x, &a, gap);
    }
}

/// The flat tops in `x`: runs of at least [`MIN_RUN`] samples, at or above `threshold`, in
/// which the signal has stopped moving.
///
/// The flatness test is what separates clipping from a loud peak. A peak passing through
/// full scale is still *climbing* — sample to sample it changes. A clipped one is pinned:
/// the converter had nowhere left to put it.
fn flat_runs(x: &[f64], threshold: f64) -> Vec<Range<usize>> {
    let flat_eps = FLATNESS * threshold;
    let mut runs = Vec::new();
    let mut i = 0;
    while i < x.len() {
        if x[i].abs() < threshold {
            i += 1;
            continue;
        }
        let sign = x[i].is_sign_positive();
        let mut j = i + 1;
        while j < x.len()
            && x[j].abs() >= threshold
            && x[j].is_sign_positive() == sign
            && (x[j] - x[j - 1]).abs() <= flat_eps
        {
            j += 1;
        }
        let len = j - i;
        if (MIN_RUN..=MAX_RUN).contains(&len) {
            runs.push(i..j);
        }
        i = j.max(i + 1);
    }
    runs
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

    /// A sine at `amp`, hard-clipped at `ceiling` — a converter running out of headroom.
    fn clipped(n: usize, hz: f32, amp: f32, ceiling: f32) -> (Vec<f32>, Vec<f32>) {
        let clean: Vec<f32> = (0..n)
            .map(|i| (std::f32::consts::TAU * hz * (i as f32 / SR)).sin() * amp)
            .collect();
        let cut = clean.iter().map(|s| s.clamp(-ceiling, ceiling)).collect();
        (clean, cut)
    }

    fn peak(x: &[f32]) -> f32 {
        x.iter().fold(0.0f32, |m, s| m.max(s.abs()))
    }

    /// **The peaks come back.** A sine driven 3 dB into a 0.9 ceiling has its tops sheared
    /// flat; de-clipping must push them back UP, past the ceiling, toward where the signal
    /// was actually going — and get closer to the original in the process.
    ///
    /// Both halves matter. "The peak got taller" alone would pass for a gain stage; "it got
    /// closer to the original" alone would pass for doing nothing much. Together they say
    /// the shape was reconstructed.
    #[test]
    fn a_clipped_peak_is_pushed_back_up_toward_where_it_was_going() {
        let (clean, cut) = clipped(8_192, 220.0, 1.4, 0.9);
        let before = mono(cut.clone());
        let after = declip(&before, 0.85, 1.0);

        let restored = peak(after.samples());
        assert!(
            restored > 0.95,
            "the flat top was not lifted off the ceiling (peak {restored}, ceiling 0.9)"
        );

        // Closer to the original than the clipped version was — measured where the damage
        // is, which is the only place a reconstruction can be judged.
        let err = |v: &[f32]| -> f32 {
            clean
                .iter()
                .zip(v)
                .filter(|(c, _)| c.abs() > 0.9)
                .map(|(c, g)| (c - g).abs())
                .sum()
        };
        let was = err(&cut);
        let now = err(after.samples());
        assert!(
            now < was * 0.8,
            "the reconstruction is not closer to the original: error {was:.1} -> {now:.1}"
        );
    }

    /// **A hot but CLEAN master is left alone.** This is the failure that would make the
    /// tool unusable: a peak that merely touches full scale is intact, and "fixing" it would
    /// invent an overshoot that was never in the recording.
    ///
    /// The run requirement ([`MIN_RUN`]) plus the flatness test are what tell the two apart.
    /// Red if either is dropped: a normalized sine touches 1.0 at every crest.
    #[test]
    fn a_hot_but_clean_peak_is_left_alone() {
        // Exactly full scale, never clipped: a normalized master.
        let clean: Vec<f32> = (0..8_192)
            .map(|i| (std::f32::consts::TAU * 220.0 * (i as f32 / SR)).sin())
            .collect();
        let before = mono(clean);
        let after = declip(&before, 0.85, 1.0);
        let moved = before
            .samples()
            .iter()
            .zip(after.samples())
            .map(|(a, b)| (a - b).abs())
            .fold(0.0f32, f32::max);
        assert!(
            moved < 0.01,
            "a clean peak was 'restored' by {moved} — the tool is inventing overshoot"
        );
    }

    /// **Amount 0 is a byte-identical bypass** — the rack's non-negotiable invariant, and
    /// the thing that makes an A/B honest.
    #[test]
    fn amount_zero_is_byte_identical() {
        let (_, cut) = clipped(4_096, 220.0, 1.4, 0.9);
        let before = mono(cut);
        let after = declip(&before, 0.85, 0.0);
        assert_eq!(before.samples(), after.samples());
    }

    /// Amount is a real blend, not a switch: half of it moves the peak half of the way.
    #[test]
    fn amount_scales_the_repair() {
        let (_, cut) = clipped(8_192, 220.0, 1.4, 0.9);
        let before = mono(cut);
        let half = peak(declip(&before, 0.85, 0.5).samples());
        let full = peak(declip(&before, 0.85, 1.0).samples());
        assert!(
            half > 0.9 && half < full,
            "Amount does not scale the repair: 0.5 -> {half}, 1.0 -> {full}"
        );
    }

    /// A run longer than [`MAX_RUN`] is not a clipped peak — it is a clipped *passage*,
    /// with no intact signal left around it to reconstruct from. Left honest rather than
    /// guessed at.
    #[test]
    fn a_very_long_plateau_is_left_alone() {
        let mut s = vec![0.0f32; 4_096];
        for (i, v) in s.iter_mut().enumerate() {
            *v = (std::f32::consts::TAU * 220.0 * (i as f32 / SR)).sin() * 0.5;
        }
        // A quarter-second of solid ceiling — far past MAX_RUN.
        for v in s.iter_mut().take(2_500).skip(1_000) {
            *v = 0.95;
        }
        let before = mono(s);
        let after = declip(&before, 0.85, 1.0);
        for f in 1_100..2_400 {
            assert!(
                (after.samples()[f] - 0.95).abs() < 0.01,
                "a long plateau was reconstructed at frame {f}: {}",
                after.samples()[f]
            );
        }
    }
}
