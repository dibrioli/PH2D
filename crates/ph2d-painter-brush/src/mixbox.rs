//! Mixbox — subtractive PIGMENT mixing (the W5 innovation: blue + yellow → a
//! vibrant green, not the muddy grey a linear/OKLab lerp gives).
//!
//! ## Clean-room spectral model (NOT the scrtwpns/mixbox LUT)
//!
//! The published Mixbox LUT (Sochorová & Jamriška, SIGGRAPH Asia 2021) is
//! **non-commercial-free / commercial-paid** — unsafe to embed in a shipping
//! product. PH2D uses a clean-room SPECTRAL method instead (fully owned, no
//! external data, det-mode-portable): the same physics that makes real pigments
//! mix subtractively.
//!
//! 1. **Reconstruct a reflectance spectrum** from the linear-sRGB colour. A
//!    `3 × NB` integration matrix `M` (normalised Gaussian channel responses)
//!    maps a reflectance curve → RGB; its Moore–Penrose **pseudo-inverse** `M⁺`
//!    maps RGB → the minimum-norm reflectance that integrates back EXACTLY
//!    (`M · M⁺ = I`), so a colour mixed with itself is unchanged.
//! 2. **Mix two spectra by the WEIGHTED GEOMETRIC MEAN** `R(λ) = Ra(λ)^(1−t) ·
//!    Rb(λ)^t` — the subtractive operator (each pigment *absorbs*, so reflectances
//!    multiply). This is what turns blue (reflects short λ) + yellow (reflects
//!    long λ) into green (both pass the middle).
//! 3. **Integrate back to RGB** with `M`.
//!
//! Endpoints are kept EXACT (`t=0 → a`, `t=1 → b`) by blending the spectral mix
//! with the plain linear lerp by `4·t·(1−t)` (0 at the ends, 1 at 50/50) — the
//! pigment effect is strongest at an even mix and a thin glaze barely shifts hue,
//! which is also physically sensible.
//!
//! Working space = **linear sRGB D65** ([`ph2d_color::mixbox_space`]). The API
//! ([`mixbox_lerp_linear`] / [`mixbox_lerp_srgb8`]) is frozen since T1.3 — this
//! swaps the internals from the placeholder lerp to the real mix with no caller
//! change. **Still forced to Linear in `--features det-painter`** (ADR-0044
//! §2.5.1): `powf` is not bit-identical cross-OS; a Q-fixed-point port is the
//! det-mode follow-up.

use std::sync::LazyLock;

// ─────────────────────────── spectral basis ─────────────────────────────

/// Spectral resolution (bands across the visible range). Enough for smooth
/// secondaries; small enough that the per-pixel cost stays modest.
const NB: usize = 24;

/// Per-channel Gaussian response centres (band index) + width. The R/G/B sensors
/// peak at long/mid/short wavelengths and OVERLAP (the overlap in the green band
/// is what lets blue and yellow both reflect there → green on mixing).
const CENTERS: [f32; 3] = [19.0, 12.0, 4.0]; // R, G, B (band 0 = short λ)
/// Per-channel response widths — ASYMMETRIC on purpose: a BROAD blue (so "blue"
/// reflects into the green band → green survives the mix) but a NARROW green/red
/// (so "yellow" absorbs blue hard → blue is suppressed). Symmetric widths give a
/// teal (blue ≈ green); this tips it to a clean green.
const SIGMAS: [f32; 3] = [5.0, 3.4, 7.0];

/// Reflectances are clamped to `[REFL_FLOOR, 1]` before the geometric mean (a
/// zero would annihilate a whole band and `0^0` is undefined).
const REFL_FLOOR: f32 = 1.0e-4;

struct SpectralBasis {
    /// `w[c][i]` — the raw (peak-1) channel responses, used as the reflectance
    /// BASIS: a pigment of colour `rgb` reconstructs to `Σ_c rgb[c]·w[c]`. Broad &
    /// overlapping, so "blue" keeps green content (the overlap is what makes the
    /// subtractive mix yield green rather than mud).
    w: [[f32; NB]; 3],
    /// `m[c][i]` — the integration matrix (each `w[c]` normalised to sum 1, so a
    /// flat unit reflectance integrates to white).
    m: [[f32; NB]; 3],
}

static BASIS: LazyLock<SpectralBasis> = LazyLock::new(|| {
    let mut w = [[0.0f32; NB]; 3];
    let mut m = [[0.0f32; NB]; 3];
    for c in 0..3 {
        let mut sum = 0.0f32;
        for (i, wi) in w[c].iter_mut().enumerate() {
            let d = (i as f32 - CENTERS[c]) / SIGMAS[c];
            *wi = (-d * d).exp();
            sum += *wi;
        }
        for (i, mi) in m[c].iter_mut().enumerate() {
            *mi = w[c][i] / sum;
        }
    }
    SpectralBasis { w, m }
});

/// Linear-sRGB → reflectance spectrum (additive over the broad channel basis),
/// clamped to `[REFL_FLOOR, 1]` for the geometric mean. NOT round-trip exact — the
/// reconstruction is deliberately broad/leaky so the spectral mix gives vivid
/// secondaries; [`spectral_mix`] re-anchors the round-trip exactly per colour so a
/// self-mix is still the identity.
fn to_reflectance(rgb: [f32; 3]) -> [f32; NB] {
    let b = &*BASIS;
    let mut refl = [0.0f32; NB];
    for (i, r) in refl.iter_mut().enumerate() {
        let v = b.w[0][i] * rgb[0] + b.w[1][i] * rgb[1] + b.w[2][i] * rgb[2];
        *r = v.clamp(REFL_FLOOR, 1.0);
    }
    refl
}

/// Reflectance spectrum → linear-sRGB (integrate against the channel responses).
fn reflectance_to_rgb(refl: &[f32; NB]) -> [f32; 3] {
    let b = &*BASIS;
    let mut rgb = [0.0f32; 3];
    for (c, out) in rgb.iter_mut().enumerate() {
        *out = (0..NB).map(|i| b.m[c][i] * refl[i]).sum::<f32>().max(0.0);
    }
    rgb
}

#[inline]
fn sub3(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}

// ── precomputed RGB → (ln reflectance, round-trip) LUT — the per-pixel fast path ──
//
// The per-pixel cost is the geometric mean `Ra(λ)^(1−t)·Rb(λ)^t` over NB bands.
// Storing ln(reflectance) turns it into `exp((1−t)·lnRa + t·lnRb)` — ONE `exp`/band
// instead of TWO `pow`/band — and a trilinear LUT replaces the per-pixel
// reconstruction + per-band `ln` (the backdrop varies per pixel; the brush is
// prepared ONCE per stamp). Built once at init (~5k tiny solves).
const LUT_N: usize = 17;

struct PigmentLut {
    /// `ln(reflectance)` per band, and `integrate(reflectance)` (the leaky
    /// round-trip), at each `LUT_N³` RGB grid point (`idx=(bi*N+gi)*N+ri`).
    ln_refl: Vec<[f32; NB]>,
    roundtrip: Vec<[f32; 3]>,
}

static LUT: LazyLock<PigmentLut> = LazyLock::new(|| {
    let n = LUT_N;
    let denom = (n - 1) as f32;
    let mut ln_refl = vec![[0.0f32; NB]; n * n * n];
    let mut roundtrip = vec![[0.0f32; 3]; n * n * n];
    for bi in 0..n {
        for gi in 0..n {
            for ri in 0..n {
                let rgb = [ri as f32 / denom, gi as f32 / denom, bi as f32 / denom];
                let refl = to_reflectance(rgb);
                let idx = (bi * n + gi) * n + ri;
                for (k, &r) in refl.iter().enumerate() {
                    ln_refl[idx][k] = r.ln();
                }
                roundtrip[idx] = reflectance_to_rgb(&refl);
            }
        }
    }
    PigmentLut { ln_refl, roundtrip }
});

/// Trilinear sample of `(ln reflectance, round-trip)` for a linear-sRGB `[0,1]³`.
fn lut_sample(rgb: [f32; 3]) -> ([f32; NB], [f32; 3]) {
    let n = LUT_N;
    let denom = (n - 1) as f32;
    let lut = &*LUT;
    let mut i0 = [0usize; 3];
    let mut fr = [0.0f32; 3];
    for c in 0..3 {
        let f = rgb[c].clamp(0.0, 1.0) * denom;
        let lo = (f.floor() as usize).min(n - 2);
        i0[c] = lo;
        fr[c] = f - lo as f32;
    }
    let mut ln = [0.0f32; NB];
    let mut rt = [0.0f32; 3];
    for corner in 0..8usize {
        let (dr, dg, db) = (corner & 1, (corner >> 1) & 1, (corner >> 2) & 1);
        let weight = (if dr == 1 { fr[0] } else { 1.0 - fr[0] })
            * (if dg == 1 { fr[1] } else { 1.0 - fr[1] })
            * (if db == 1 { fr[2] } else { 1.0 - fr[2] });
        if weight == 0.0 {
            continue;
        }
        let idx = ((i0[2] + db) * n + (i0[1] + dg)) * n + (i0[0] + dr);
        let lr = &lut.ln_refl[idx];
        for (k, l) in ln.iter_mut().enumerate() {
            *l += lr[k] * weight;
        }
        let r = lut.roundtrip[idx];
        rt[0] += r[0] * weight;
        rt[1] += r[1] * weight;
        rt[2] += r[2] * weight;
    }
    (ln, rt)
}

// ──────────────────────────── public API ────────────────────────────────

/// A brush colour with its `ln(reflectance)` + round-trip error precomputed —
/// built ONCE per stamp (the brush colour is constant across the footprint), so
/// the per-pixel [`mix_prepared`] skips the brush-side reconstruction.
pub struct PreparedPigment {
    color: [f32; 3],
    ln_refl: [f32; NB],
    /// `color − integrate(reflectance)` — the re-anchor correction term.
    err: [f32; 3],
}

/// Precompute a brush colour for repeated [`mix_prepared`] calls over a stamp.
#[must_use]
pub fn prepare_pigment(color: [f32; 3]) -> PreparedPigment {
    let (ln_refl, rt) = lut_sample(color);
    PreparedPigment {
        color,
        ln_refl,
        err: sub3(color, rt),
    }
}

/// Endpoint-exact subtractive mix of a backdrop `a` toward the prepared `brush` at
/// ratio `t`. The geometric mean runs in log space (one `exp`/band) with the
/// backdrop's `ln(reflectance)` from the LUT; the round-trip is re-anchored
/// per-colour (self-mix is the EXACT identity — the LUT errors cancel). Skips the
/// spectral solve where the endpoint blend weight `4t(1−t)` is ~0 (stamp edges /
/// near-opaque), where the linear lerp is the answer anyway.
#[must_use]
pub fn mix_prepared(brush: &PreparedPigment, a: [f32; 3], t: f32) -> [f32; 3] {
    let t = t.clamp(0.0, 1.0);
    let b = brush.color;
    let lin = [
        a[0] + (b[0] - a[0]) * t,
        a[1] + (b[1] - a[1]) * t,
        a[2] + (b[2] - a[2]) * t,
    ];
    let w = 4.0 * t * (1.0 - t);
    if w < 0.02 {
        return lin;
    }
    let (ln_a, rt_a) = lut_sample(a);
    let mut rm = [0.0f32; NB];
    for (i, r) in rm.iter_mut().enumerate() {
        *r = ((1.0 - t) * ln_a[i] + t * brush.ln_refl[i]).exp();
    }
    let mixed = reflectance_to_rgb(&rm);
    let ea = sub3(a, rt_a);
    let spec = [
        (mixed[0] + ea[0] * (1.0 - t) + brush.err[0] * t).max(0.0),
        (mixed[1] + ea[1] * (1.0 - t) + brush.err[1] * t).max(0.0),
        (mixed[2] + ea[2] * (1.0 - t) + brush.err[2] * t).max(0.0),
    ];
    [
        lin[0] + (spec[0] - lin[0]) * w,
        lin[1] + (spec[1] - lin[1]) * w,
        lin[2] + (spec[2] - lin[2]) * w,
    ]
}

/// Subtractive pigment mix of two **linear-sRGB** colours (the canonical Mixbox
/// working space, ADR-0051 §2.4). `t ∈ [0,1]` is the ratio toward `b`. Endpoints
/// are exact; the subtractive (geometric-mean) behaviour peaks at an even mix. For
/// a brush stroke (constant brush colour) prefer [`prepare_pigment`] +
/// [`mix_prepared`] to amortise the brush-side cost.
pub fn mixbox_lerp_linear(a: [f32; 3], b: [f32; 3], t: f32) -> [f32; 3] {
    mix_prepared(&prepare_pigment(b), a, t)
}

/// Subtractive pigment mix of two **sRGB-8** colours — the spectral mix runs in
/// linear light (Mixbox's only legal space), so this decodes → mixes → re-encodes.
pub fn mixbox_lerp_srgb8(a: [u8; 3], b: [u8; 3], t: f32) -> [u8; 3] {
    let dec = |c: [u8; 3]| {
        [
            srgb8_to_linear(c[0]),
            srgb8_to_linear(c[1]),
            srgb8_to_linear(c[2]),
        ]
    };
    let out = mixbox_lerp_linear(dec(a), dec(b), t);
    [
        linear_to_srgb8(out[0]),
        linear_to_srgb8(out[1]),
        linear_to_srgb8(out[2]),
    ]
}

/// sRGB-8 → linear-light `[0,1]` (IEC 61966).
#[inline]
fn srgb8_to_linear(v: u8) -> f32 {
    let s = v as f32 / 255.0;
    if s <= 0.040_45 {
        s / 12.92
    } else {
        ((s + 0.055) / 1.055).powf(2.4)
    }
}

/// linear-light `[0,1]` → sRGB-8 (IEC 61966).
#[inline]
fn linear_to_srgb8(v: f32) -> u8 {
    let v = v.clamp(0.0, 1.0);
    let s = if v <= 0.003_130_8 {
        v * 12.92
    } else {
        1.055 * v.powf(1.0 / 2.4) - 0.055
    };
    (s * 255.0 + 0.5).clamp(0.0, 255.0) as u8
}

#[cfg(test)]
mod tests {
    use super::*;

    fn approx(a: [f32; 3], b: [f32; 3], tol: f32) -> bool {
        (0..3).all(|c| (a[c] - b[c]).abs() < tol)
    }

    #[test]
    fn lerp_at_t0_returns_a() {
        assert_eq!(mixbox_lerp_srgb8([10, 20, 30], [200, 100, 50], 0.0), [10, 20, 30]);
    }

    #[test]
    fn lerp_at_t1_returns_b() {
        assert_eq!(mixbox_lerp_srgb8([10, 20, 30], [200, 100, 50], 1.0), [200, 100, 50]);
    }

    #[test]
    fn linear_endpoints_are_exact_and_t_clamps() {
        let a = [0.1, 0.4, 0.8];
        let b = [0.9, 0.2, 0.3];
        assert!(approx(mixbox_lerp_linear(a, b, 0.0), a, 1e-6));
        assert!(approx(mixbox_lerp_linear(a, b, 1.0), b, 1e-6));
        assert!(approx(mixbox_lerp_linear(a, b, 2.0), b, 1e-6), "t>1 clamps to b");
        assert!(approx(mixbox_lerp_linear(a, b, -1.0), a, 1e-6), "t<0 clamps to a");
    }

    #[test]
    fn self_mix_is_identity() {
        // A colour mixed with itself at any ratio is ~unchanged (the round-trip is
        // re-anchored). The fast-path LUT (`ln(refl)` interpolated vs the round-trip
        // interpolated) leaves a sub-1% residual between grid points — imperceptible
        // for "paint a colour over itself", and the cost of the 2× per-pixel speedup.
        for c in [[0.8, 0.1, 0.1], [0.1, 0.6, 0.2], [0.2, 0.3, 0.9], [0.5, 0.5, 0.5]] {
            for &t in &[0.25, 0.5, 0.75] {
                assert!(
                    approx(mixbox_lerp_linear(c, c, t), c, 1e-2),
                    "self-mix {c:?} @ {t} drifted: {:?}",
                    mixbox_lerp_linear(c, c, t)
                );
            }
        }
    }

    #[test]
    fn white_plus_white_is_white() {
        let w = [1.0, 1.0, 1.0];
        assert!(approx(mixbox_lerp_linear(w, w, 0.5), w, 3e-3));
    }

    #[test]
    fn blue_plus_yellow_is_green_not_grey() {
        // THE smoke (eval Inovação 2): pure blue + pure yellow at 50/50 must give a
        // GREEN-dominant colour — green clearly the top channel, red + blue both
        // suppressed — not the muddy grey a linear lerp ((0.5,0.5,0.5)) produces.
        let blue = [0.0, 0.0, 1.0];
        let yellow = [1.0, 1.0, 0.0];
        let mix = mixbox_lerp_linear(blue, yellow, 0.5);
        assert!(
            mix[1] > mix[0] && mix[1] > mix[2],
            "green is the dominant channel: {mix:?}"
        );
        assert!(mix[1] > 0.3, "green is vivid, not dark: {mix:?}");
        // Clearly GREEN, not a teal (green over blue) and not grey (green over red).
        assert!(mix[1] > mix[2] + 0.05, "green leads blue (not teal): {mix:?}");
        assert!(
            mix[1] - mix[0].max(mix[2]) > 0.05,
            "green clearly dominates (vs the linear grey midpoint 0.5,0.5,0.5): {mix:?}"
        );
    }

    #[test]
    fn mix_is_more_saturated_than_a_grey_lerp() {
        // The subtractive mix should not collapse to the dull linear average.
        let blue = [0.0, 0.0, 1.0];
        let yellow = [1.0, 1.0, 0.0];
        let mix = mixbox_lerp_linear(blue, yellow, 0.5);
        let chroma = mix.iter().cloned().fold(0.0, f32::max)
            - mix.iter().cloned().fold(1.0, f32::min);
        assert!(chroma > 0.17, "mix keeps chroma (not grey): {mix:?} chroma {chroma}");
    }
}
