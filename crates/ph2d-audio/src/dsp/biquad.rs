//! [`Biquad`] — a second-order IIR filter (transposed direct form II).
//!
//! Coefficients follow the canonical **RBJ Audio EQ Cookbook** (Robert
//! Bristow-Johnson) — ported, not invented (DIRETIVA §1: use the published
//! reference; no magic constants). Presentation math, HR-5 exempt.

use std::f32::consts::TAU;

/// Normalized biquad coefficients (`a0` folded to 1).
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct BiquadCoeffs {
    pub b0: f32,
    pub b1: f32,
    pub b2: f32,
    pub a1: f32,
    pub a2: f32,
}

/// Which response `from_rbj` builds.
enum RbjKind {
    Lowpass,
    Highpass,
    Bandpass,
    Peak,
}

impl BiquadCoeffs {
    /// Pass-through (no filtering).
    pub const fn identity() -> Self {
        Self {
            b0: 1.0,
            b1: 0.0,
            b2: 0.0,
            a1: 0.0,
            a2: 0.0,
        }
    }

    /// Low-pass at `cutoff` Hz with quality `q`.
    pub fn lowpass(sample_rate: f32, cutoff: f32, q: f32) -> Self {
        Self::from_rbj(RbjKind::Lowpass, sample_rate, cutoff, q, 0.0)
    }

    /// High-pass at `cutoff` Hz with quality `q`.
    pub fn highpass(sample_rate: f32, cutoff: f32, q: f32) -> Self {
        Self::from_rbj(RbjKind::Highpass, sample_rate, cutoff, q, 0.0)
    }

    /// Constant-0-dB-peak band-pass centered at `freq` Hz with quality `q`.
    pub fn bandpass(sample_rate: f32, freq: f32, q: f32) -> Self {
        Self::from_rbj(RbjKind::Bandpass, sample_rate, freq, q, 0.0)
    }

    /// Peaking EQ of `gain_db` at `freq` Hz with quality `q`.
    pub fn peak(sample_rate: f32, freq: f32, q: f32, gain_db: f32) -> Self {
        Self::from_rbj(RbjKind::Peak, sample_rate, freq, q, gain_db)
    }

    fn from_rbj(kind: RbjKind, sample_rate: f32, freq: f32, q: f32, gain_db: f32) -> Self {
        let sr = sample_rate.max(1.0);
        let w0 = TAU * (freq.clamp(1.0, sr * 0.5) / sr);
        let (sin_w0, cos_w0) = w0.sin_cos();
        let alpha = sin_w0 / (2.0 * q.max(1e-4));

        let (b0, b1, b2, a0, a1, a2) = match kind {
            RbjKind::Lowpass => {
                let k = (1.0 - cos_w0) * 0.5;
                (k, 1.0 - cos_w0, k, 1.0 + alpha, -2.0 * cos_w0, 1.0 - alpha)
            }
            RbjKind::Highpass => {
                let k = (1.0 + cos_w0) * 0.5;
                (
                    k,
                    -(1.0 + cos_w0),
                    k,
                    1.0 + alpha,
                    -2.0 * cos_w0,
                    1.0 - alpha,
                )
            }
            RbjKind::Bandpass => (alpha, 0.0, -alpha, 1.0 + alpha, -2.0 * cos_w0, 1.0 - alpha),
            RbjKind::Peak => {
                let a = 10f32.powf(gain_db / 40.0);
                (
                    1.0 + alpha * a,
                    -2.0 * cos_w0,
                    1.0 - alpha * a,
                    1.0 + alpha / a,
                    -2.0 * cos_w0,
                    1.0 - alpha / a,
                )
            }
        };

        Self {
            b0: b0 / a0,
            b1: b1 / a0,
            b2: b2 / a0,
            a1: a1 / a0,
            a2: a2 / a0,
        }
    }
}

/// A second-order IIR filter with its state (transposed direct form II).
#[derive(Copy, Clone, Debug)]
pub struct Biquad {
    coeffs: BiquadCoeffs,
    z1: f32,
    z2: f32,
}

impl Biquad {
    /// Build with the given coefficients and cleared state.
    pub fn new(coeffs: BiquadCoeffs) -> Self {
        Self {
            coeffs,
            z1: 0.0,
            z2: 0.0,
        }
    }

    /// Swap coefficients (keeps the delay state — retune without a click).
    pub fn set_coeffs(&mut self, coeffs: BiquadCoeffs) {
        self.coeffs = coeffs;
    }

    /// Clear the delay state.
    pub fn reset(&mut self) {
        self.z1 = 0.0;
        self.z2 = 0.0;
    }

    /// Filter one sample.
    #[inline]
    pub fn process(&mut self, x: f32) -> f32 {
        let c = &self.coeffs;
        let y = c.b0 * x + self.z1;
        self.z1 = c.b1 * x - c.a1 * y + self.z2;
        self.z2 = c.b2 * x - c.a2 * y;
        y
    }
}

impl Default for Biquad {
    fn default() -> Self {
        Self::new(BiquadCoeffs::identity())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn identity_passes_through() {
        let mut f = Biquad::default();
        for &x in &[0.0, 1.0, -0.5, 0.3] {
            assert_eq!(f.process(x), x);
        }
    }

    #[test]
    fn lowpass_has_unity_dc_gain() {
        // DC gain of a low-pass = (b0+b1+b2)/(1+a1+a2) == 1.
        let c = BiquadCoeffs::lowpass(48_000.0, 1_000.0, 0.707);
        let dc = (c.b0 + c.b1 + c.b2) / (1.0 + c.a1 + c.a2);
        assert!(
            (dc - 1.0).abs() < 1e-3,
            "low-pass DC gain must be ~1, got {dc}"
        );
    }

    #[test]
    fn lowpass_output_stays_bounded() {
        let mut f = Biquad::new(BiquadCoeffs::lowpass(48_000.0, 2_000.0, 0.707));
        let mut peak = 0.0f32;
        // Feed a step; a stable low-pass settles toward the input without blowing up.
        for _ in 0..48_000 {
            peak = peak.max(f.process(1.0).abs());
        }
        assert!(
            peak.is_finite() && peak < 4.0,
            "filter must stay stable, peak {peak}"
        );
    }

    #[test]
    fn highpass_blocks_dc() {
        // DC gain of a high-pass ~ 0.
        let c = BiquadCoeffs::highpass(48_000.0, 1_000.0, 0.707);
        let dc = (c.b0 + c.b1 + c.b2) / (1.0 + c.a1 + c.a2);
        assert!(dc.abs() < 1e-3, "high-pass should block DC, got {dc}");
    }
}
