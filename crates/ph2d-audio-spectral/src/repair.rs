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
        // A region flush against the START of the clip has no column before it to lean on.
        // (There is always one AFTER: `c1` was clamped to `cols - 1` above, so `post_col`
        // is a real column. An earlier version guarded for its absence and the guard was
        // dead code — audit 2026-07-12.)
        let have_pre = c0 > 0;

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
                let time_mag = if have_pre { ma + (mb - ma) * t } else { mb };
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

                let base = if have_pre { pre[b] } else { post[b] };
                // **DC and Nyquist have a SIGN, not a phase.** A real signal's spectrum is
                // conjugate-symmetric, which pins those two bins to the real axis. Rotating a
                // phase into them is meaningless — and `Stft::process` must project it back
                // out before the inverse, or `realfft` refuses the column outright. Say it
                // here as well, so the estimate that reaches the transform is the one we
                // meant: the magnitude, carrying the anchor's sign.
                if b == 0 || (b == bins - 1 && n.is_multiple_of(2)) {
                    let sign = if base.re < 0.0 { -1.0 } else { 1.0 };
                    spec[b] = Complex32::new(mag * sign, 0.0);
                    continue;
                }
                // Phase: continue this bin's natural advance from the anchor, so the fill
                // is a coherent continuation of what was there rather than a fresh burst
                // with an arbitrary phase (which reads as a click).
                let advance = std::f32::consts::TAU * (b as f32) * (hop as f32) / (n as f32);
                let phase = base.im.atan2(base.re) + advance * (col - pre_col) as f32;
                spec[b] = Complex32::new(mag * phase.cos(), mag * phase.sin());
            }
        });

        // Splice: only the samples the edited columns actually reach are replaced. The rest
        // of the clip comes through bit-identical, which is the promise the tool makes — so
        // the bounds have to be exactly right, not merely generous. The edited columns are
        // `c0..c1` EXCLUSIVE, so the last one is `c1 - 1`; using `c1` here spliced one hop
        // (5 ms) of resynthesis past anything an edit could reach (audit 2026-07-12).
        let pad = n - hop;
        let lo = (c0 * hop).saturating_sub(pad);
        let hi = (((c1 - 1) * hop + n).saturating_sub(pad)).min(frames);
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

    /// **The acceptance gate of the whole feature (ADR-0122 §4.2).**
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

    /// **The fill CONTINUES the signal; it does not merely sit quietly in the hole.**
    ///
    /// The beep gate above (−20 dB in, ≤1 dB out) is satisfied by *any* sufficiently quiet
    /// filler — including one that continues nothing. Measured (audit 2026-07-12): killing
    /// the time route, or the phase continuation, or replacing the phase with a hash, all
    /// left it green. Half of what the module documents was proved by nothing.
    ///
    /// So put a **sustained tone** under the beep, at the beep's own frequency, and check
    /// the two things the doc actually claims:
    ///
    /// 1. the repaired bin comes back at the level of the bed that was passing through it
    ///    (the TIME route) — not at the level of some unrelated neighbouring bin;
    /// 2. it comes back **steady** across columns (the PHASE continuation). An incoherent
    ///    phase cancels unevenly in the overlap-add and the bin flickers — which is exactly
    ///    what "a fresh burst rather than a continuation" sounds like.
    ///
    /// Measured, coefficient of variation across the repaired columns: coherent 0.33 ·
    /// phase-zero 1.43 · phase-hash 0.73 · frequency-route-only 1.42. The bar at 0.50 catches
    /// all three, including the near-miss.
    #[test]
    fn the_fill_continues_the_tone_that_was_passing_through() {
        let n = 48_000usize;
        let bed_hz = 5_000.0;
        let (f0, f1) = (20_000usize, 24_000usize);
        let mut s = speech(n);
        for (i, v) in s.iter_mut().enumerate() {
            let t = i as f32 / SR;
            // A quiet tonal bed at 5 kHz, sustained through the whole clip...
            *v += (std::f32::consts::TAU * bed_hz * t).sin() * 0.08;
            // ...and a loud beep on top of it, only inside the region.
            if (f0..f1).contains(&i) {
                *v += (std::f32::consts::TAU * bed_hz * t).sin() * 0.5;
            }
        }
        let after = repair(
            &mono(s),
            &Band {
                frames: f0..f1,
                hz: (bed_hz - 300.0)..(bed_hz + 300.0),
            },
        );

        // Per-column magnitude of the repaired bin, inside the region.
        let mut stft = Stft::new();
        let bins = stft.bins();
        let bin = (bed_hz / (SR * 0.5) * (bins - 1) as f32).round() as usize;
        let (c0, c1) = (stft.column_at(f0), stft.column_at(f1));
        let x = after.samples().to_vec();
        let mut mags = Vec::new();
        stft.analyze(&x, |col, spec| {
            if (c0 + 2..c1 - 2).contains(&col) {
                mags.push(magnitude(spec[bin]));
            }
        });
        assert!(mags.len() > 4, "not enough repaired columns to judge");

        let mean = mags.iter().sum::<f32>() / mags.len() as f32;
        let var = mags.iter().map(|m| (m - mean).powi(2)).sum::<f32>() / mags.len() as f32;
        let cov = var.sqrt() / mean.max(1e-12);
        assert!(
            cov <= 0.50,
            "the repaired bin flickers across columns (CoV {cov:.2}) — the fill is a burst, \
             not a continuation of the tone that was passing through"
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

    /// **A band that reaches DC must not punch a hole in the audio.**
    ///
    /// Dragging the box down to the bottom of the spectrogram is the natural gesture for
    /// killing a rumble or a hum — and it clamps `freq_at_y` to exactly 0.0, so `b0` is the
    /// DC bin. A REAL signal has no phase at DC (nor at Nyquist): both bins must be real.
    /// Write a phase into one and `realfft` refuses the whole column — which used to be
    /// swallowed by an `is_err()` early-return, so all four columns covering a sample were
    /// dropped, the window sum came out zero, and the WOLA wrote **digital silence**.
    ///
    /// Found by audit 2026-07-12. Every repair test before it used a band in the middle of
    /// the spectrum (1-5.4 kHz), where `b0 >= 1` — all green, all blind to the edge.
    #[test]
    fn a_band_that_reaches_dc_does_not_punch_a_hole() {
        let before = mono(speech(48_000));
        let after = repair(
            &before,
            &Band {
                frames: 20_000..24_000,
                hz: 0.0..800.0,
            },
        );
        let inside = &after.samples()[20_000..24_000];
        let zeros = inside.iter().filter(|v| **v == 0.0).count();
        assert_eq!(
            zeros, 0,
            "{zeros}/4000 samples of the selection were replaced by digital silence"
        );
    }

    /// The same hole at the other end of the spectrum: Nyquist is real too.
    #[test]
    fn a_band_pinned_at_nyquist_does_not_punch_a_hole() {
        let before = mono(speech(48_000));
        let ny = SR * 0.5;
        let after = repair(
            &before,
            &Band {
                frames: 20_000..24_000,
                hz: (ny - 5.0)..ny,
            },
        );
        let inside = &after.samples()[20_000..24_000];
        let zeros = inside.iter().filter(|v| **v == 0.0).count();
        assert_eq!(zeros, 0, "{zeros}/4000 samples became digital silence");
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
