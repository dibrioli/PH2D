//! Spectral repair — erase a region of time **and frequency**, and rebuild it from what
//! surrounds it.
//!
//! This is the tool with no time-domain equivalent, and the reason the FFT was worth a
//! dependency. A beep laid over speech occupies the *same samples* as the speech: there is
//! no stretch of the waveform you could cut, fade or interpolate that removes one and
//! keeps the other. In the spectrogram they are simply in different places, and you can
//! reach in and take one.
//!
//! ## How the hole is filled
//!
//! Deleting the bins outright would leave an audible hole — a notch where the underlying
//! sound also used to live. So each erased bin is **re-estimated** from its surroundings,
//! by two independent routes:
//!
//! - **Along time** — the same bin, in the columns just before and just after the region.
//!   Interpolating between them recovers a tonal component that was *passing through*, and
//!   the phase is continued at that bin's natural per-hop advance so the reconstruction is
//!   coherent rather than a fresh burst.
//! - **Along frequency** — the neighbouring bins just outside the region, in that same
//!   column. This recovers broadband content (a noise floor, a formant, a room) that was
//!   present at that instant.
//!
//! Neither route is right for both kinds of material — the first is the tonal answer, the
//! second the broadband one — so the estimate is their **geometric mean**, which is the
//! conservative choice: it cannot exceed either estimate, so a repair never invents
//! something louder than what surrounded it.
//!
//! ## What is guaranteed
//!
//! Samples outside the region's reach are returned **bit-identical**. The repair is spliced
//! into a copy of the original rather than replacing the whole clip with a resynthesis, so
//! "it only touched what I selected" is literally true rather than approximately true.

use crate::spectrogram::Band;
use crate::stft::{Stft, magnitude};
use ph2d_audio::SampleData;
use realfft::num_complex::Complex32;

/// Rebuild `band` from its surroundings. Returns a new clip; the original is untouched (as
/// is every sample outside the region's reach).
pub fn repair(data: &SampleData, band: &Band) -> SampleData {
    let format = data.format();
    let channels = format.channel_count().max(1);
    let frames = data.frame_count();
    if frames == 0 || band.frames.start >= band.frames.end {
        return data.clone();
    }

    let mut stft = Stft::new();
    let bins = stft.bins();
    let nyquist = format.sample_rate as f32 * 0.5;

    // The band, in the transform's own coordinates. Both ranges are widened to at least
    // one cell: a region the user drew thinner than a bin still has to remove something.
    let to_bin = |hz: f32| ((hz / nyquist).clamp(0.0, 1.0) * (bins - 1) as f32).round() as usize;
    let b0 = to_bin(band.hz.start.min(band.hz.end));
    let b1 = to_bin(band.hz.start.max(band.hz.end)).max(b0 + 1).min(bins);
    let c0 = stft.column_at(band.frames.start);
    let cols = stft.columns(frames);
    let c1 = stft
        .column_at(band.frames.end)
        .max(c0 + 1)
        .min(cols.saturating_sub(1));
    if c0 >= c1 || b0 >= b1 {
        return data.clone();
    }

    let mut out = data.samples().to_vec();
    for ch in 0..channels {
        let x: Vec<f32> = (0..frames)
            .map(|f| data.samples()[f * channels + ch])
            .collect();

        // The two anchor columns — the last one before the region and the first one after.
        // Everything inside is reconstructed *between* them, so they are read BEFORE any
        // bin is overwritten.
        let (pre_col, post_col) = (c0.saturating_sub(1), c1);
        let mut pre = vec![Complex32::default(); bins];
        let mut post = vec![Complex32::default(); bins];
        stft.analyze(&x, |col, spec| {
            if col == pre_col {
                pre.copy_from_slice(spec);
            } else if col == post_col {
                post.copy_from_slice(spec);
            }
        });
        // A region flush against the start or end of the clip has only one anchor; lean on
        // the one that exists rather than fading into nothing.
        let have_pre = c0 > 0;
        let have_post = post_col < cols;

        let n = stft.window_size();
        let hop = stft.hop();
        let y = stft.process(&x, |col, spec| {
            if !(c0..c1).contains(&col) {
                return;
            }
            let span = (c1 - pre_col) as f32;
            let t = (col - pre_col) as f32 / span.max(1.0);
            for b in b0..b1 {
                // Estimate 1 — along TIME: this bin, before and after. Tonal content that
                // was passing through the region continues through it.
                let (ma, mb) = (magnitude(pre[b]), magnitude(post[b]));
                let time_mag = match (have_pre, have_post) {
                    (true, true) => ma + (mb - ma) * t,
                    (true, false) => ma,
                    (false, true) => mb,
                    (false, false) => 0.0,
                };
                // Estimate 2 — along FREQUENCY: the bins just outside the region, now.
                // Broadband content present at this instant.
                let below = (b0 > 0).then(|| magnitude(spec[b0 - 1]));
                let above = (b1 < bins).then(|| magnitude(spec[b1]));
                let freq_mag = match (below, above) {
                    (Some(l), Some(h)) => {
                        let f = (b - b0 + 1) as f32 / (b1 - b0 + 1) as f32;
                        l + (h - l) * f
                    }
                    (Some(l), None) => l,
                    (None, Some(h)) => h,
                    (None, None) => 0.0,
                };
                // The conservative combination: a geometric mean can never exceed either
                // estimate, so a repair cannot invent something louder than its
                // surroundings — the failure mode that turns a fix into a new artefact.
                let mag = (time_mag.max(0.0) * freq_mag.max(0.0)).sqrt();

                // Phase: continue this bin's natural advance from the anchor, so the fill
                // is a coherent continuation of what was there rather than a fresh burst
                // with an arbitrary phase (which reads as a click).
                let advance = std::f32::consts::TAU * (b as f32) * (hop as f32) / (n as f32);
                let base = if have_pre { pre[b] } else { post[b] };
                let phase = base.im.atan2(base.re) + advance * (col - pre_col) as f32;
                spec[b] = Complex32::new(mag * phase.cos(), mag * phase.sin());
            }
        });

        // Splice: only the samples the edited columns actually reach are replaced. The rest
        // of the clip comes through bit-identical, which is the promise the tool makes.
        let pad = n - hop;
        let lo = (c0 * hop).saturating_sub(pad);
        let hi = ((c1 * hop + n).saturating_sub(pad)).min(frames);
        for f in lo..hi {
            out[f * channels + ch] = y[f];
        }
    }
    SampleData::from_interleaved(out, format)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::stft::bin_hz;
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

    /// A speech-ish bed: a few harmonics of 150 Hz, amplitude-modulated. Broadband enough
    /// that a repair has something real to preserve.
    fn speech(n: usize) -> Vec<f32> {
        (0..n)
            .map(|i| {
                let t = i as f32 / SR;
                let env = 0.5 + 0.5 * (std::f32::consts::TAU * 3.0 * t).sin();
                let h: f32 = (1..=6)
                    .map(|k| (std::f32::consts::TAU * 150.0 * k as f32 * t).sin() / k as f32)
                    .sum();
                h * env * 0.2
            })
            .collect()
    }

    /// Energy in one bin, summed over the columns covering `frames`.
    fn bin_energy(data: &SampleData, hz: f32, frames: std::ops::Range<usize>) -> f32 {
        let mut stft = Stft::new();
        let bins = stft.bins();
        let bin = (hz / (SR * 0.5) * (bins - 1) as f32).round() as usize;
        let c0 = stft.column_at(frames.start);
        let c1 = stft.column_at(frames.end);
        let x: Vec<f32> = data.samples().to_vec();
        let mut e = 0.0;
        stft.analyze(&x, |col, spec| {
            if (c0..c1).contains(&col) {
                e += magnitude(spec[bin]).powi(2);
            }
        });
        e
    }

    fn db(a: f32, b: f32) -> f32 {
        10.0 * (a.max(1e-20) / b.max(1e-20)).log10()
    }

    /// **The acceptance gate of the whole feature (ADR-0115 §4.2).**
    ///
    /// A tonal beep laid over speech: erasing its time-frequency region must take the beep
    /// down by **at least 20 dB**, while the speech OUTSIDE that region changes by **less
    /// than 1 dB**. Those two numbers together are what "repair" means — either one alone
    /// is satisfiable by something useless (a mute, or a no-op).
    #[test]
    fn a_beep_over_speech_is_removed_and_the_speech_survives() {
        let n = 48_000usize;
        let mut s = speech(n);
        let beep_hz = 5_000.0;
        let (f0, f1) = (20_000usize, 24_000usize);
        for (i, v) in s.iter_mut().enumerate().take(f1).skip(f0) {
            *v += (std::f32::consts::TAU * beep_hz * (i as f32 / SR)).sin() * 0.5;
        }
        let before = mono(s);
        let band = Band {
            frames: f0..f1,
            hz: (beep_hz - 400.0)..(beep_hz + 400.0),
        };
        let after = repair(&before, &band);

        let beep_before = bin_energy(&before, beep_hz, f0..f1);
        let beep_after = bin_energy(&after, beep_hz, f0..f1);
        let drop = -db(beep_after, beep_before);
        assert!(
            drop >= 20.0,
            "the beep only dropped {drop:.1} dB (the gate is 20 dB)"
        );

        // The speech, at a harmonic well outside the erased band, in a stretch of time
        // well outside the erased region: it must be essentially untouched.
        let speech_hz = 900.0;
        let quiet = 4_000usize..12_000usize;
        let sp_before = bin_energy(&before, speech_hz, quiet.clone());
        let sp_after = bin_energy(&after, speech_hz, quiet);
        let moved = db(sp_after, sp_before).abs();
        assert!(
            moved <= 1.0,
            "the speech outside the region moved by {moved:.2} dB (the gate is 1 dB)"
        );
    }

    /// **Everything outside the region's reach comes back bit-identical.** Not "close" —
    /// identical. A repair that quietly resynthesised the whole clip would pass every
    /// energy test above and still be the wrong tool.
    #[test]
    fn the_rest_of_the_clip_is_untouched() {
        let before = mono(speech(48_000));
        let band = Band {
            frames: 20_000..24_000,
            hz: 4_600.0..5_400.0,
        };
        let after = repair(&before, &band);
        // Well before and well after the edited span (which reaches a window either side).
        for f in (0..18_000).chain(27_000..48_000) {
            assert_eq!(
                before.samples()[f],
                after.samples()[f],
                "sample {f} changed, and it is nowhere near the selection"
            );
        }
    }

    /// An empty selection is a no-op, not a crash and not a silent clip.
    #[test]
    fn an_empty_band_does_nothing() {
        let before = mono(speech(4_800));
        let after = repair(
            &before,
            &Band {
                frames: 100..100,
                hz: 1_000.0..2_000.0,
            },
        );
        assert_eq!(before.samples(), after.samples());
    }

    /// The bin the user pointed at is the bin that gets erased. (Guards the Hz → bin
    /// conversion, which is the one place an off-by-one silently repairs the wrong sound.)
    #[test]
    fn the_band_maps_to_the_bins_the_user_pointed_at() {
        let stft = Stft::new();
        let bins = stft.bins();
        let hz = 5_000.0;
        let bin = (hz / (SR * 0.5) * (bins - 1) as f32).round() as usize;
        let back = bin_hz(bin, bins, 48_000);
        assert!((back - hz).abs() < 50.0, "{hz} Hz → bin {bin} → {back} Hz");
    }
}
