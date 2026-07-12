//! The picture: magnitude over time and frequency, cached once and drawn many times.
//!
//! A waveform shows you *how loud*, and nothing else. A cough, a beep, a mains hum and a
//! tape hiss all look like a slightly fatter waveform — and all four are unmistakable in a
//! spectrogram. This is the view that makes the rest of W5 usable, because you cannot
//! repair what you cannot see.
//!
//! Two design points carry the whole thing:
//!
//! **The cache is bounded, not proportional.** A 3-minute clip must not cost more memory
//! than a 3-second one *by a factor of sixty*. So the column count is capped and the hop
//! grows to meet it: the picture gets coarser in time, never bigger than [`MAX_COLS`]
//! columns. HR-13 gives the whole audio subsystem 30 MB; a picture of one clip does not
//! get to spend it.
//!
//! **Downsampling to the screen is peak-hold, not average.** The cache has thousands of
//! columns and the overlay is a few hundred pixels wide. Averaging would dissolve a
//! one-column beep into its neighbours and it would vanish from the display — while
//! remaining perfectly audible. The user has to be able to see the thing they are about to
//! erase, so a pixel takes the LOUDEST value under it.

use crate::stft::{Stft, WINDOW, magnitude};
use ph2d_audio::SampleData;
use std::ops::Range;

/// The quietest level the picture distinguishes. Below this everything is background: 90 dB
/// of range is more than the eye reads out of a colour ramp anyway, and a lower floor just
/// paints dither.
pub const FLOOR_DB: f32 = -90.0;

/// Hop used for the picture — 50 % overlap. The edit path wants 75 % (smooth per-bin gain
/// changes); a picture does not, and halving the columns halves the cache.
const DISPLAY_HOP: usize = WINDOW / 2;

/// The most columns the cache will ever hold, whatever the clip's length. At 513 bins that
/// is ~4 MB — bounded, and far more time resolution than a few hundred pixels of overlay
/// can show.
pub const MAX_COLS: usize = 8_192;

/// A **time-and-frequency region** — what a spectral selection is, and the thing a waveform
/// selection cannot express. `frames` are original sample frames; `hz` is real frequency.
#[derive(Debug, Clone, PartialEq)]
pub struct Band {
    pub frames: Range<usize>,
    pub hz: Range<f32>,
}

/// A clip's magnitude spectrogram, quantised to one byte per bin (0 = [`FLOOR_DB`],
/// 255 = full scale). Bytes, not floats: this is a picture, and the 8-bit ramp is finer
/// than the colour ramp it is drawn through.
pub struct Spectrogram {
    cols: usize,
    bins: usize,
    hop: usize,
    sample_rate: u32,
    frames: usize,
    db: Vec<u8>,
}

impl Spectrogram {
    /// Analyse a clip. Channels are summed to mono — the picture is about *what* is in the
    /// sound, not which side it is on, and a per-channel spectrogram would double the cost
    /// to show two near-identical images.
    pub fn build(data: &SampleData) -> Self {
        let sample_rate = data.format().sample_rate;
        let channels = data.format().channel_count().max(1);
        let frames = data.frame_count();
        let samples = data.samples();
        let mono: Vec<f32> = (0..frames)
            .map(|f| {
                let sum: f32 = (0..channels).map(|c| samples[f * channels + c]).sum();
                sum / channels as f32
            })
            .collect();

        let mut stft = Stft::with(WINDOW, DISPLAY_HOP);
        let bins = stft.bins();
        let analysed = stft.columns(frames);

        // A long clip is DECIMATED, not skipped. The tempting move is to widen the hop
        // until the columns fit — but a hop wider than the window steps OVER audio, and a
        // short beep landing in one of those gaps would be inaudible to the picture while
        // remaining perfectly audible to the ear. So every column is still analysed; groups
        // of `stride` of them are folded into one stored column by **peak-hold**, the same
        // bargain the waveform's peak cache makes. Coarser in time, never blind.
        let stride = analysed.div_ceil(MAX_COLS).max(1);
        let cols = analysed.div_ceil(stride);
        let mut db = vec![0u8; cols * bins];
        // Reference for 0 dB: a full-scale sine through a Hann window peaks at A·N/4 in its
        // bin (the window's coherent gain is 1/2, and a real sine splits its energy between
        // +f and -f). Without this the picture would be calibrated to nothing and "full
        // scale" would sit at whatever colour happened to come out.
        let reference = WINDOW as f32 * 0.25;
        stft.analyze(&mono, |col, spec| {
            let slot = col / stride;
            if slot >= cols {
                return;
            }
            let row = slot * bins;
            for (bin, c) in spec.iter().enumerate() {
                let v = quantise_db(magnitude(*c) / reference);
                db[row + bin] = db[row + bin].max(v);
            }
        });

        Self {
            cols,
            bins,
            hop: stft.hop() * stride,
            sample_rate,
            frames,
            db,
        }
    }

    /// True when there is nothing to draw.
    pub fn is_empty(&self) -> bool {
        self.cols == 0 || self.bins == 0
    }

    /// The clip this picture was built from, in frames — so a stale cache can be spotted.
    pub fn frames(&self) -> usize {
        self.frames
    }

    /// The clip's sample rate.
    pub fn sample_rate(&self) -> u32 {
        self.sample_rate
    }

    /// Columns in the picture.
    pub fn columns(&self) -> usize {
        self.cols
    }

    /// Samples between columns — grows with the clip's length to hold the cache under
    /// [`MAX_COLS`], so it must be read, never assumed.
    pub fn hop(&self) -> usize {
        self.hop
    }

    /// The top of the frequency axis.
    pub fn nyquist(&self) -> f32 {
        self.sample_rate as f32 * 0.5
    }

    /// Render into RGBA at exactly `w` × `h` pixels, ready for `draw_image_rgba`.
    ///
    /// `ramp` is the colour scale from silence to full scale, sampled piecewise-linearly —
    /// the caller passes **resolved theme colours**, so this crate never learns what a
    /// theme is and the picture still belongs to the app (HR-15: no colour is invented
    /// here).
    ///
    /// Frequency runs **up** the image (DC at the bottom, Nyquist at the top) — the
    /// convention every DAW uses, and the one the user's ear expects.
    pub fn rgba(&self, w: usize, h: usize, ramp: &[[u8; 3]]) -> Vec<u8> {
        let mut out = vec![0u8; w * h * 4];
        if self.is_empty() || w == 0 || h == 0 || ramp.is_empty() {
            return out;
        }
        for py in 0..h {
            // Invert y: py = 0 is the TOP of the image, which is the HIGHEST frequency.
            let b0 = (h - 1 - py) * self.bins / h;
            let b1 = ((h - py) * self.bins / h).max(b0 + 1).min(self.bins);
            for px in 0..w {
                let c0 = px * self.cols / w;
                let c1 = ((px + 1) * self.cols / w).max(c0 + 1).min(self.cols);
                // Peak-hold across every cache cell this pixel covers — see the module
                // docs: an averaged beep is an invisible beep.
                let mut peak = 0u8;
                for c in c0..c1 {
                    let row = c * self.bins;
                    for b in b0..b1 {
                        peak = peak.max(self.db[row + b]);
                    }
                }
                let rgb = sample_ramp(ramp, peak as f32 / 255.0);
                let i = (py * w + px) * 4;
                out[i] = rgb[0];
                out[i + 1] = rgb[1];
                out[i + 2] = rgb[2];
                out[i + 3] = 255;
            }
        }
        out
    }
}

/// Amplitude ratio → one byte, linear in **decibels** from [`FLOOR_DB`] to 0.
///
/// Decibels, not amplitude: the ear is logarithmic, and on a linear ramp everything below
/// about -40 dB collapses into the same black — which is where the hiss and the hum and
/// most of what you would want to fix actually live.
fn quantise_db(ratio: f32) -> u8 {
    if ratio <= 0.0 {
        return 0;
    }
    let db = 20.0 * ratio.log10();
    let t = ((db - FLOOR_DB) / -FLOOR_DB).clamp(0.0, 1.0);
    (t * 255.0).round() as u8
}

/// Sample a piecewise-linear colour ramp at `t` in `0..=1`.
fn sample_ramp(ramp: &[[u8; 3]], t: f32) -> [u8; 3] {
    if ramp.len() == 1 {
        return ramp[0];
    }
    let t = t.clamp(0.0, 1.0) * (ramp.len() - 1) as f32;
    let i = (t as usize).min(ramp.len() - 2);
    let f = t - i as f32;
    let (a, b) = (ramp[i], ramp[i + 1]);
    [
        (a[0] as f32 + (b[0] as f32 - a[0] as f32) * f) as u8,
        (a[1] as f32 + (b[1] as f32 - a[1] as f32) * f) as u8,
        (a[2] as f32 + (b[2] as f32 - a[2] as f32) * f) as u8,
    ]
}

#[cfg(test)]
mod tests {
    use super::*;
    use ph2d_audio::{AudioFormat, ChannelLayout};

    fn clip(hz: f32, secs: f32, amp: f32) -> SampleData {
        let sr = 48_000.0;
        let n = (secs * sr) as usize;
        let s: Vec<f32> = (0..n)
            .map(|i| (std::f32::consts::TAU * hz * (i as f32 / sr)).sin() * amp)
            .collect();
        SampleData::from_interleaved(
            s,
            AudioFormat {
                sample_rate: 48_000,
                channels: ChannelLayout::Mono,
            },
        )
    }

    /// **The picture is calibrated.** A full-scale sine has to read as full scale, and its
    /// energy has to sit at its own frequency — otherwise every colour on screen is a
    /// number the user cannot trust, and the frequency they select is not the one they get.
    #[test]
    fn a_full_scale_tone_reads_full_scale_at_its_own_frequency() {
        let sg = Spectrogram::build(&clip(6_000.0, 0.5, 1.0));
        let bin = (6_000.0 / sg.nyquist() * (sg.bins - 1) as f32).round() as usize;
        // Mid-clip column, away from the windows that straddle the padding.
        let col = sg.cols / 2;
        let at = sg.db[col * sg.bins + bin];
        assert!(
            at > 240,
            "a 0 dBFS tone read {at}/255 at its own bin — the picture is miscalibrated"
        );
        // And an unrelated bin is quiet: the energy did not smear across the spectrum.
        let far = sg.db[col * sg.bins + bin / 4];
        assert!(far < 80, "the tone smeared to a distant bin: {far}/255");
    }

    /// **Frequency runs UP the image.** The other half of the mapping `freq_at_y` gates in
    /// the shell — and the half that was NOT gated (audit 2026-07-12), which is worse,
    /// because between them they are a loop: if the PICTURE is flipped, the user sees the
    /// beep at the bottom, drags the box at the bottom, `freq_at_y` faithfully reports
    /// "low", and the repair erases the bass. Every assertion in the crate stayed green for
    /// a flipped picture.
    #[test]
    fn the_top_of_the_image_is_the_highest_frequency() {
        // A tone at a quarter of Nyquist: unambiguously in the lower half.
        let sg = Spectrogram::build(&clip(6_000.0, 0.4, 0.8));
        let (w, h) = (64usize, 128usize);
        let img = sg.rgba(w, h, &[[0, 0, 0], [255, 255, 255]]);
        // i32, not u8: `top + 100` on a byte SILENTLY WRAPS in release (250 + 100 = 94), and
        // the first version of this assertion did exactly that — so the flipped picture
        // compared 96 > 94 and passed. The gate was right about the physics and wrong about
        // the arithmetic, which is its own little lesson about trusting a green.
        let brightest = |rows: std::ops::Range<usize>| -> i32 {
            rows.flat_map(|y| (0..w).map(move |x| (y * w + x) * 4))
                .map(|i| i32::from(img[i]))
                .max()
                .unwrap_or(0)
        };
        let top = brightest(0..h / 2);
        let bottom = brightest(h / 2..h);
        assert!(
            bottom > top + 100,
            "a 6 kHz tone (a quarter of Nyquist) is brighter in the TOP half than the \
             bottom: {top} vs {bottom} — the frequency axis is upside down, and a repair \
             will erase the mirror band"
        );
    }

    /// **The calibration is two-sided.** `quantise_db` clamps at 0 dB, so a reference that
    /// is too HOT saturates and a full-scale test can never see it: an image 6 dB too bright
    /// everywhere still reads 255 at full scale (audit 2026-07-12 — the systematic-bias
    /// class, [[feedback_loose_oracle_hides_systematic_bias]]).
    ///
    /// So anchor at a level the quantiser does NOT clamp. A -20 dBFS tone must read the byte
    /// the dB scale says it should: `(−20 − FLOOR)/−FLOOR × 255 = 198`. Measured: 198 exactly
    /// with the right reference, 215 with one 6 dB hot.
    #[test]
    fn the_picture_is_calibrated_on_both_sides() {
        // -20 dBFS = amplitude 0.1.
        let sg = Spectrogram::build(&clip(6_000.0, 0.4, 0.1));
        let bin = (6_000.0 / sg.nyquist() * (sg.bins - 1) as f32).round() as usize;
        let col = sg.cols / 2;
        let got = sg.db[col * sg.bins + bin];
        let want = ((-20.0 - FLOOR_DB) / -FLOOR_DB * 255.0) as i32;
        assert!(
            (i32::from(got) - want).abs() <= 3,
            "a -20 dBFS tone read {got}/255; the dB scale says {want}. The picture is \
             miscalibrated by roughly {:.1} dB",
            (f32::from(got) - want as f32) / 255.0 * -FLOOR_DB
        );
    }

    /// Quiet is dark and loud is bright, monotonically. If the ramp is not ordered the
    /// picture is decorative rather than informative.
    #[test]
    fn quieter_is_darker() {
        let loud = Spectrogram::build(&clip(6_000.0, 0.3, 1.0));
        let soft = Spectrogram::build(&clip(6_000.0, 0.3, 0.01));
        let bin = (6_000.0 / loud.nyquist() * (loud.bins - 1) as f32).round() as usize;
        let col = loud.cols / 2;
        assert!(
            loud.db[col * loud.bins + bin] > soft.db[col * soft.bins + bin] + 40,
            "a 40 dB drop in level barely moved the picture"
        );
    }

    /// **The cache is bounded.** A long clip gets a coarser picture, never a bigger one —
    /// or one 3-minute sound effect would eat the whole audio RAM budget (HR-13).
    ///
    /// Red if the hop stops growing: a 5-minute clip at a fixed hop would be ~28 000
    /// columns, over 14 MB.
    #[test]
    fn a_long_clip_does_not_grow_the_cache_without_bound() {
        let sg = Spectrogram::build(&clip(440.0, 300.0, 0.5));
        assert!(
            sg.cols <= MAX_COLS,
            "a 5-minute clip produced {} columns (cap is {MAX_COLS})",
            sg.cols
        );
        assert!(
            sg.db.len() < 5_000_000,
            "the cache is {} bytes for one clip",
            sg.db.len()
        );
        // …and the time mapping still spans the clip: a bounded cache that lies about
        // WHERE things are would be worse than no cache.
        assert!(
            sg.hop * sg.cols >= sg.frames,
            "the columns no longer cover the clip"
        );
    }

    /// **A narrow beep survives being drawn small.** This is the peak-hold, and it is the
    /// difference between a repairable artefact and an invisible one.
    ///
    /// The clip is deliberately **long** relative to the image: 8 seconds across 200 pixels
    /// means every pixel is standing on ~4 columns of cache, and the beep occupies one of
    /// them. That is the only configuration in which the peak-hold does anything, and the
    /// first version of this test did not have it — a 1-second clip drawn at 200 px is an
    /// *up*sample, so it passed just as happily with averaging. Mutating the downsample to a
    /// mean is what exposed that; the assertion below now goes red for it, as it always
    /// claimed to ([[feedback_loose_oracle_hides_systematic_bias]]).
    #[test]
    fn a_narrow_beep_survives_the_downsample() {
        let sr = 48_000.0;
        let n = 8 * 48_000usize;
        let mut s = vec![0.0f32; n];
        // 10 ms of 6 kHz, alone in eight seconds of silence.
        for (i, v) in s.iter_mut().enumerate().take(96_480).skip(96_000) {
            *v = (std::f32::consts::TAU * 6_000.0 * (i as f32 / sr)).sin();
        }
        let data = SampleData::from_interleaved(
            s,
            AudioFormat {
                sample_rate: 48_000,
                channels: ChannelLayout::Mono,
            },
        );
        let sg = Spectrogram::build(&data);
        let (w, h) = (200usize, 100usize);
        assert!(
            sg.columns() > w * 2,
            "the fixture is not exercising the downsample: {} columns into {w} pixels",
            sg.columns()
        );
        let img = sg.rgba(w, h, &[[0, 0, 0], [255, 255, 255]]);
        let brightest = img.chunks_exact(4).map(|p| p[0]).max().unwrap_or(0);
        assert!(
            brightest > 200,
            "the beep did not survive the downsample (brightest pixel {brightest}/255) — \
             it is audible and invisible, which is the worst thing a spectrogram can be"
        );
    }
}
