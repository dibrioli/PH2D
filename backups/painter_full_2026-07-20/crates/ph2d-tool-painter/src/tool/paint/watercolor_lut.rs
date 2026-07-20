//! Watercolor **optical LUTs** + the pigment-body helpers — the transcendental-free core of the
//! composite ([`super::watercolor_render`]). The `s2l`/`ln`/`exp` tables run every transcendental ONCE at
//! init (HR-5), so the per-pixel loop is pure table lookups + lerps. Split from [`super::watercolor_field`]
//! for the workspace LOC cap; re-exported there (`pub(super) use`) so existing paths stay stable.

use ph2d_color::srgb::{linear_to_srgb_byte, srgb_to_linear_byte};
use std::sync::OnceLock;

// ── LUTs (built once; the ln/exp/pow run only here, never per pixel — HR-5) ──────────────────────────
const L2S_N: usize = 4096;
const EXP_N: usize = 2048;
/// Largest `|exponent|` the Beer–Lambert `exp` LUT spans; `exp(-32) ≈ 1.3e-14 ≈ 0`, so anything past
/// this is transmittance 0 (opaque pigment) — a safe clamp for even a very dense wash.
const EXP_MAX: f32 = 32.0;
/// Body/Opacity coverage rate (#17): `bodyCov = opacity·(1 − e^{−BODY_OD_GAIN·od})`. A default wash's
/// optical depth is low (`od ≈ fill·depth ≈ 0.14`), so the gain lifts modest deposits into a visible body
/// — a light pigment shows at its hue instead of at `Tᵢ ≈ 1` (near-invisible). Tuned high so Opacity bites
/// at the thin default wash; Opacity `0` skips the fold (byte-identical). LITERAL-PX-OK: optical tuning.
const BODY_OD_GAIN: f32 = 6.0;

pub(super) struct Luts {
    /// sRGB byte → linear-light intensity.
    pub(super) s2l: [f32; 256],
    /// `ln(max(linear, 1e-4))` per sRGB byte — the pigment's log-transmittance (clamp avoids `-∞` on black).
    pub(super) lnl: [f32; 256],
    /// linear-light intensity (`[0, 1]`, quantised to `L2S_N + 1`) → sRGB byte.
    pub(super) l2s: [u8; L2S_N + 1],
    /// `exp(-mag)` for `mag ∈ [0, EXP_MAX]` (quantised to `EXP_N + 1`) — Beer–Lambert transmittance.
    pub(super) exp_neg: [f32; EXP_N + 1],
    /// `−ln(max(x, 1e-4))` for `x ∈ [0, 1]` (quantised to `EXP_N + 1`) — the absorbance of a linear
    /// ratio, for the log-space (density-proportional) Wet lift + subtractive colour mix.
    pub(super) ln_mag: [f32; EXP_N + 1],
}

pub(super) fn luts() -> &'static Luts {
    static LUTS: OnceLock<Luts> = OnceLock::new();
    LUTS.get_or_init(|| {
        let mut s2l = [0.0f32; 256];
        let mut lnl = [0.0f32; 256];
        for i in 0..256 {
            let lin = srgb_to_linear_byte(i as u8);
            s2l[i] = lin;
            lnl[i] = lin.max(1e-4).ln();
        }
        let mut l2s = [0u8; L2S_N + 1];
        for (i, slot) in l2s.iter_mut().enumerate() {
            *slot = linear_to_srgb_byte(i as f32 / L2S_N as f32);
        }
        let mut exp_neg = [0.0f32; EXP_N + 1];
        for (i, slot) in exp_neg.iter_mut().enumerate() {
            *slot = (-(EXP_MAX * i as f32 / EXP_N as f32)).exp();
        }
        let mut ln_mag = [0.0f32; EXP_N + 1];
        for (i, slot) in ln_mag.iter_mut().enumerate() {
            *slot = -(i as f32 / EXP_N as f32).max(1e-4).ln();
        }
        Luts {
            s2l,
            lnl,
            l2s,
            exp_neg,
            ln_mag,
        }
    })
}

impl Luts {
    /// linear intensity → sRGB byte via the LUT (clamped index).
    #[inline]
    pub(super) fn l2s_byte(&self, v: f32) -> u8 {
        let idx = (v.clamp(0.0, 1.0) * L2S_N as f32) as usize;
        self.l2s[idx.min(L2S_N)]
    }
    /// Beer–Lambert transmittance `pigment^(od)` for a pigment byte `c` and optical depth `od ≥ 0`,
    /// via `exp(lnl[c]·od)` looked up in the `exp` LUT (`lnl[c] ≤ 0` ⇒ the exponent is `≤ 0`).
    #[inline]
    pub(super) fn transmittance(&self, c: u8, od: f32) -> f32 {
        let mag = -self.lnl[c as usize] * od; // = |exponent|, ≥ 0
        let idx = (mag / EXP_MAX * EXP_N as f32) as usize;
        self.exp_neg[idx.min(EXP_N)]
    }
    /// `−ln(x)` of a linear ratio `x ∈ [0, 1]`, LUT + lerp (the interpolation kills the quantisation
    /// banding a raw lookup would print into smooth tint gradients).
    #[inline]
    pub(super) fn absorbance(&self, x: f32) -> f32 {
        let f = x.clamp(0.0, 1.0) * EXP_N as f32;
        let i = (f as usize).min(EXP_N - 1);
        let t = f - i as f32;
        self.ln_mag[i] + (self.ln_mag[i + 1] - self.ln_mag[i]) * t
    }
    /// `exp(−mag)` for `mag ≥ 0`, LUT + lerp (twin of [`Self::absorbance`]).
    #[inline]
    pub(super) fn exp_mag(&self, mag: f32) -> f32 {
        let f = (mag / EXP_MAX).clamp(0.0, 1.0) * EXP_N as f32;
        let i = (f as usize).min(EXP_N - 1);
        let t = f - i as f32;
        self.exp_neg[i] + (self.exp_neg[i + 1] - self.exp_neg[i]) * t
    }
    /// **Body / Opacity** coverage (#17): how strongly the pigment lays its OWN colour over the
    /// transmittance result. `opacity·(1 − e^{−k·od})`, value-independent so light pigments show at
    /// their hue. `0` ⇒ `0` (the fold is a no-op → byte-identical pure-Beer–Lambert path).
    #[inline]
    pub(super) fn body_cov(&self, opacity: f32, od: f32) -> f32 {
        if opacity > 0.0 {
            opacity * (1.0 - self.exp_mag(BODY_OD_GAIN * od))
        } else {
            0.0
        }
    }
}

/// Minimal straight alpha so the un-premultiply `L = (app − ground·(1−a))/a` stays in `[0,1]` for an
/// appearance `app` over a linear `ground`: darkening (`app < g`) needs `a ≥ 1 − app/g`, lightening (a
/// light body over a dark ground) needs `a ≥ (app − g)/(1 − g)`. `0` when the pixel already fits.
#[inline]
pub(super) fn gamut_alpha(app: f32, g: f32) -> f32 {
    if app < g {
        1.0 - app / g.max(1e-4)
    } else if g < 1.0 {
        (app - g) / (1.0 - g).max(1e-4)
    } else {
        0.0
    }
}
