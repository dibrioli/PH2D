//! The short-time Fourier transform — the one piece every spectral tool stands on.
//!
//! Everything in this crate is the same three steps: cut the signal into overlapping
//! windowed blocks, take each block to the frequency domain, and (for the tools that
//! edit) put it back. Only the middle step differs.
//!
//! **The invariant the crate rests on: analyse → synthesise with no change in between
//! must return the input, sample for sample.** If the round-trip is not transparent then
//! every edit arrives contaminated by the transform itself, and no test can tell a repair
//! from an artefact. `roundtrip_is_transparent` proves it, edges included.
//!
//! Two things buy that transparency, and they are easy to confuse:
//!
//! - **Zero padding** is what makes the EDGES whole. The signal is padded by `n - hop`
//!   before the first sample and after the last, so sample 0 is already covered by a full
//!   set of overlapping windows rather than by one window's fading tail. Without it the
//!   first and last blocks come back attenuated, and a repair near the start of a clip
//!   arrives with a fade baked into it.
//! - **Explicit window-sum normalisation** (WOLA) is what makes it correct at ANY hop. The
//!   textbook shortcut divides by the constant that squared Hann windows happen to sum to —
//!   which is 1.5 at 75 % overlap and 0.75 at 50 %, and this crate uses both. Rather than
//!   trust a constant that is only right for one configuration, we accumulate the window
//!   energy alongside the signal and divide by what actually landed. Change the hop and the
//!   transform stays unity; hardcode the constant and every edit is silently rescaled.
//!   (Same reasoning, same fix, as the WOLA in the formant shifter next door.)

use realfft::num_complex::Complex32;
use realfft::{ComplexToReal, RealFftPlanner, RealToComplex};
use std::sync::Arc;

/// Window size. 1024 at 48 kHz is ~21 ms — long enough to resolve a bass note's harmonics
/// (~47 Hz per bin), short enough that a transient does not smear across the whole window
/// and blunt a repair.
pub const WINDOW: usize = 1024;

/// Hop between windows: 75 % overlap. Enough that a per-bin gain change moves smoothly
/// from block to block instead of pumping (the classic source of musical noise), and the
/// cheapest hop at which Hann-squared satisfies COLA.
pub const HOP: usize = WINDOW / 4;

/// A window sum this small means the sample was barely covered at all; dividing by it
/// would amplify numerical dust into a click.
const WSUM_EPS: f32 = 1e-6;

/// A planned STFT: window, hop, forward/inverse transforms, and every scratch buffer they
/// need, allocated once.
pub struct Stft {
    n: usize,
    hop: usize,
    window: Vec<f32>,
    fwd: Arc<dyn RealToComplex<f32>>,
    inv: Arc<dyn ComplexToReal<f32>>,
    time: Vec<f32>,
    spec: Vec<Complex32>,
}

impl Stft {
    /// Plan an STFT with the default window and hop.
    pub fn new() -> Self {
        Self::with(WINDOW, HOP)
    }

    /// Plan an STFT with an explicit window and hop. `hop` is clamped into `1..=n`: a zero
    /// hop never advances, and a hop past the window leaves gaps no normalisation can fill.
    pub fn with(n: usize, hop: usize) -> Self {
        let n = n.max(2);
        let hop = hop.clamp(1, n);
        let mut planner = RealFftPlanner::<f32>::new();
        let fwd = planner.plan_fft_forward(n);
        let inv = planner.plan_fft_inverse(n);
        Self {
            n,
            hop,
            window: hann(n),
            time: fwd.make_input_vec(),
            spec: fwd.make_output_vec(),
            fwd,
            inv,
        }
    }

    /// Bins per column: `n/2 + 1`, DC through Nyquist. A real signal's spectrum is
    /// conjugate-symmetric, so the upper half carries no information — which is the whole
    /// reason this is a *real* FFT.
    pub fn bins(&self) -> usize {
        self.n / 2 + 1
    }

    /// Samples between columns.
    pub fn hop(&self) -> usize {
        self.hop
    }

    /// The window size.
    pub fn window_size(&self) -> usize {
        self.n
    }

    /// Zero padding in front of (and behind) the signal, so the first and last real
    /// samples are covered by a full set of windows rather than one window's fading tail.
    fn pad(&self) -> usize {
        self.n - self.hop
    }

    /// How many columns a signal of `len` samples produces. Enough that the LAST real
    /// sample is covered by a complete set of overlapping windows — one column short and
    /// the tail of every clip would fade out.
    pub fn columns(&self, len: usize) -> usize {
        (self.pad() + len).div_ceil(self.hop) + 1
    }

    /// The centre of column `col`, in samples of the ORIGINAL signal — which is where the
    /// column speaks from. It can be **negative** for the first columns (they look mostly
    /// at the zero padding), so this is signed on purpose: a time selection converted
    /// without the padding term lands half a window off, and the edit with it.
    pub fn column_centre(&self, col: usize) -> f64 {
        (col * self.hop) as f64 - self.pad() as f64 + self.n as f64 * 0.5
    }

    /// The column whose centre is nearest `sample` — the inverse of [`Self::column_centre`].
    pub fn column_at(&self, sample: usize) -> usize {
        let c = (sample as f64 + self.pad() as f64 - self.n as f64 * 0.5) / self.hop as f64;
        c.round().max(0.0) as usize
    }

    /// Analyse `x`, handing each column's spectrum to `f`. Read-only: for the spectrogram
    /// and for learning a noise profile.
    pub fn analyze(&mut self, x: &[f32], mut f: impl FnMut(usize, &[Complex32])) {
        self.walk(x, |s, col, spec| {
            s.fwd.process(&mut s.time, spec).ok();
            f(col, spec);
        });
    }

    /// Analyse `x`, let `f` **rewrite** each column's spectrum, and synthesise the result.
    ///
    /// This is the editing workhorse — repair and denoise are both "change these bins, put
    /// it back". The output is the same length as the input, and if `f` changes nothing it
    /// *is* the input.
    pub fn process(&mut self, x: &[f32], mut f: impl FnMut(usize, &mut [Complex32])) -> Vec<f32> {
        let (pad, n, hop) = (self.pad(), self.n, self.hop);
        let span = (self.columns(x.len()) - 1) * hop + n;
        let mut out = vec![0.0f32; span];
        let mut wsum = vec![0.0f32; span];
        let mut tmp = vec![Complex32::default(); self.bins()];
        self.walk(x, |s, col, spec| {
            s.fwd.process(&mut s.time, spec).ok();
            f(col, spec);
            // The inverse SCRAMBLES its input, so hand it a copy: the caller's spectrum has
            // to survive for whatever looks back at it (denoise smooths across columns).
            tmp.copy_from_slice(spec);
            if s.inv.process(&mut tmp, &mut s.time).is_err() {
                return;
            }
            let start = col * hop;
            let scale = 1.0 / n as f32; // realfft's inverse is unnormalised
            for i in 0..n {
                out[start + i] += s.time[i] * scale * s.window[i];
                wsum[start + i] += s.window[i] * s.window[i];
            }
        });
        // Divide by the window energy that ACTUALLY landed on each sample, not by the
        // constant it would be in the middle. This is what brings the first and last block
        // back at unity instead of faded.
        for (o, w) in out.iter_mut().zip(&wsum) {
            if *w > WSUM_EPS {
                *o /= *w;
            } else {
                *o = 0.0;
            }
        }
        out.drain(..pad);
        out.truncate(x.len());
        out
    }

    /// Walk the columns of `x`, windowing each block into `self.time` and handing the
    /// per-column spectrum buffer to `body`.
    fn walk(&mut self, x: &[f32], mut body: impl FnMut(&mut Self, usize, &mut [Complex32])) {
        let (pad, n) = (self.pad(), self.n);
        let cols = self.columns(x.len());
        let mut spec = std::mem::take(&mut self.spec);
        for col in 0..cols {
            let start = col * self.hop;
            for i in 0..n {
                // Padded coordinates: `start + i` counts from `pad` samples BEFORE x[0].
                let s = (start + i) as isize - pad as isize;
                let v = if s < 0 {
                    0.0
                } else {
                    x.get(s as usize).copied().unwrap_or(0.0)
                };
                self.time[i] = v * self.window[i];
            }
            body(self, col, &mut spec);
        }
        self.spec = spec;
    }
}

impl Default for Stft {
    fn default() -> Self {
        Self::new()
    }
}

/// A periodic Hann window — the standard choice for overlap-add: its sidelobes fall away
/// fast (a tone leaks into few neighbours, so a repair need not smear), and its square
/// satisfies COLA at 75 % overlap.
fn hann(n: usize) -> Vec<f32> {
    (0..n)
        .map(|i| {
            let t = i as f64 / n as f64;
            (0.5 - 0.5 * (std::f64::consts::TAU * t).cos()) as f32
        })
        .collect()
}

/// Magnitude of a bin.
pub fn magnitude(c: Complex32) -> f32 {
    (c.re * c.re + c.im * c.im).sqrt()
}

/// The frequency (Hz) at the centre of bin `bin`, given `bins` bins (DC..Nyquist).
pub fn bin_hz(bin: usize, bins: usize, sample_rate: u32) -> f32 {
    let top = bins.saturating_sub(1).max(1) as f32;
    bin as f32 / top * (sample_rate as f32 * 0.5)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sine(len: usize, hz: f32, sr: f32) -> Vec<f32> {
        (0..len)
            .map(|i| (std::f32::consts::TAU * hz * (i as f32 / sr)).sin() * 0.8)
            .collect()
    }

    /// **The invariant the whole crate rests on.** Analyse, change nothing, synthesise —
    /// and get the input back, *including the first and last samples*.
    ///
    /// Without it there is no such thing as a spectral edit: every "repair" would arrive
    /// with the transform's own fade baked into it, and no test could tell which artefact
    /// belonged to whom.
    #[test]
    fn roundtrip_is_transparent() {
        let x = sine(4096, 440.0, 48_000.0);
        let mut stft = Stft::new();
        let y = stft.process(&x, |_, _| {});
        assert_eq!(y.len(), x.len());
        let worst = x
            .iter()
            .zip(&y)
            .map(|(a, b)| (a - b).abs())
            .fold(0.0f32, f32::max);
        assert!(worst < 1e-4, "round-trip is not transparent: worst {worst}");
        // The edges get their own assertion: they are what the zero padding exists for, and
        // they are where a transform that is fine in the middle goes quietly wrong.
        let edge = (x[0] - y[0])
            .abs()
            .max((x[x.len() - 1] - y[y.len() - 1]).abs());
        assert!(edge < 1e-4, "the edges came back wrong: {edge}");
    }

    /// **Transparent at ANY hop — which is what the window-sum normalisation is really for.**
    ///
    /// The squared Hann windows sum to 1.5 at 75 % overlap and 0.75 at 50 %, and this crate
    /// uses both (the edit path overlaps by 75 %, the picture by 50 %). Divide by a
    /// hardcoded constant instead of by the energy that actually landed and one of those two
    /// is silently rescaled by 2× — a denoise that quietly doubles the clip, and every
    /// energy-ratio test in the crate still green because ratios do not notice a global gain.
    ///
    /// This is the test that goes red for the shortcut. The default-hop one above does not,
    /// and I assumed for a while that it did: the padding already makes the edges whole, so
    /// at 75 % overlap the constant is simply *correct*. Mutating the code is what said so.
    #[test]
    fn roundtrip_is_transparent_at_any_hop() {
        let x = sine(4096, 440.0, 48_000.0);
        for hop in [WINDOW / 8, WINDOW / 4, WINDOW / 3, WINDOW / 2] {
            let mut stft = Stft::with(WINDOW, hop);
            let y = stft.process(&x, |_, _| {});
            let worst = x
                .iter()
                .zip(&y)
                .map(|(a, b)| (a - b).abs())
                .fold(0.0f32, f32::max);
            assert!(
                worst < 1e-4,
                "hop {hop}: round-trip is not transparent (worst {worst}) — the overlap-add \
                 is not being normalised by the window energy that actually landed"
            );
        }
    }

    /// Odd lengths and lengths shorter than one window still round-trip — a clip is not
    /// obliged to be a multiple of the hop.
    #[test]
    fn awkward_lengths_roundtrip() {
        let mut stft = Stft::new();
        for len in [1usize, 7, 255, 1023, 1025, 3333] {
            let x = sine(len, 1000.0, 48_000.0);
            let y = stft.process(&x, |_, _| {});
            assert_eq!(y.len(), len, "len {len}: length changed");
            let worst = x
                .iter()
                .zip(&y)
                .map(|(a, b)| (a - b).abs())
                .fold(0.0f32, f32::max);
            assert!(worst < 1e-4, "len {len}: worst error {worst}");
        }
    }

    /// A round-trip of silence is silence — no dust from dividing by a tiny window sum.
    #[test]
    fn silence_stays_silent() {
        let mut stft = Stft::new();
        let y = stft.process(&vec![0.0; 3000], |_, _| {});
        assert!(y.iter().all(|s| s.abs() < 1e-9), "silence grew a signal");
    }

    /// A pure tone lands in its own bin — the bin → Hz mapping has to be right, or every
    /// frequency the user selects on screen points somewhere else in the signal.
    #[test]
    fn a_tone_lands_in_its_own_bin() {
        let sr = 48_000.0;
        let hz = 3_000.0;
        let x = sine(8192, hz, sr);
        let mut stft = Stft::new();
        let bins = stft.bins();
        let mut acc = vec![0.0f32; bins];
        stft.analyze(&x, |_, spec| {
            for (a, c) in acc.iter_mut().zip(spec) {
                *a += magnitude(*c);
            }
        });
        let peak = acc
            .iter()
            .enumerate()
            .max_by(|a, b| a.1.total_cmp(b.1))
            .map(|(i, _)| i)
            .unwrap();
        let got = bin_hz(peak, bins, sr as u32);
        assert!(
            (got - hz).abs() < 60.0,
            "the peak bin says {got} Hz, the tone is {hz} Hz"
        );
    }

    /// Time ⇄ column round-trips. A selection the user drew at 1.2 s has to come back as
    /// the columns that actually cover 1.2 s: get the padding term wrong and every edit
    /// lands half a window early.
    #[test]
    fn column_and_sample_agree() {
        let stft = Stft::new();
        for sample in [0usize, 500, 4096, 100_000] {
            let col = stft.column_at(sample);
            let centre = stft.column_centre(col);
            let err = (centre - sample as f64).abs();
            assert!(
                err <= stft.hop() as f64 * 0.5 + 1.0,
                "sample {sample} → column {col} → centre {centre} (off by {err})"
            );
        }
    }
}
