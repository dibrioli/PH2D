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
//! 1. **Reconstruct a reflectance spectrum** from the linear-sRGB colour, additive
//!    over a broad Gaussian channel basis `w` (reconstruction), then **integrate
//!    back** through a separate, SHARPER response `m` (integration). Decoupling the
//!    two roles is deliberate: a broad reconstruction keeps "blue" reflecting into
//!    the green band (so the mix yields green, not mud), while a sharp integration
//!    stops that same green-band energy from bleeding into the *blue output channel*
//!    (the artefact that turned the secondary into a pale teal). The per-colour
//!    round-trip error is re-anchored in [`mix_prepared`], so endpoints + self-mix
//!    stay EXACT despite the leaky basis.
//! 2. **Mix two spectra by KUBELKA–MUNK** — the textbook model for opaque pigment
//!    mixing (what spectral.js and measured-pigment engines use). Each band's
//!    reflectance `R` maps to absorption/scattering `K/S = (1−R)²/2R`; the `K/S`
//!    values blend LINEARLY by concentration `t`, then invert back to reflectance
//!    `R = 1 + K/S − √((K/S)² + 2·K/S)`. This is the physics that turns blue
//!    (reflects short λ) + yellow (reflects long λ) into a saturated green — a
//!    truer pigment green than the geometric-mean (transparent-glaze) operator it
//!    replaced.
//! 3. **Integrate back to RGB** with `m`.
//!
//! Endpoints are kept EXACT (`t=0 → a`, `t=1 → b`) by blending the spectral mix
//! with the plain linear lerp by `4·t·(1−t)` (0 at the ends, 1 at 50/50) — the
//! pigment effect is strongest at an even mix and a thin glaze barely shifts hue,
//! which is also physically sensible.
//!
//! Working space = **linear sRGB D65** ([`ph2d_color::mixbox_space`]). The API
//! ([`pigment_lerp_linear`] / [`pigment_lerp_srgb8`]) is frozen since T1.3 — this
//! swaps the internals from the placeholder lerp to the real mix with no caller
//! change. **Still forced to Linear in `--features det-painter`** (ADR-0044
//! §2.5.1): `powf` is not bit-identical cross-OS; a Q-fixed-point port is the
//! det-mode follow-up.

use std::sync::LazyLock;

// ─────────────────────────── spectral basis ─────────────────────────────

/// Spectral resolution (bands across the visible range). Enough for smooth
/// secondaries; small enough that the per-pixel cost stays modest.
const NB: usize = 24;

/// `NB` exposed for GPU mirrors of the spectral mix (the WGSL composite port
/// loops over this many bands). Pinned equal to `NB` so the two never drift.
pub const SPECTRAL_BANDS: usize = NB;

/// Integration-response centres (band index) + widths — the matrix `m` that maps a
/// reflectance spectrum → linear sRGB. R/G/B peak at long/mid/short wavelengths;
/// sharp + decoupled so output channels stay clean (no cross-band bleed).
const CENTERS: [f32; 3] = [20.0, 12.0, 3.0]; // R, G, B (band 0 = short λ)
const SIGMAS_INTEG: [f32; 3] = [3.5, 2.6, 3.2];

/// Reflectances are clamped to `[REFL_FLOOR, 1]` before the Kubelka–Munk map (a
/// zero band would send `K/S → ∞`).
const REFL_FLOOR: f32 = 1.0e-4;

/// **7-curve reconstruction base spectra** (the spectral.js / Burns *structure*):
/// any sRGB is split into White + the two nearest of {Cyan,Magenta,Yellow} /
/// {Red,Green,Blue} (see [`rgb_to_weights`]) and reconstructed as a weighted sum of
/// fixed base reflectances. This covers the FULL sRGB gamut with NO residual (so
/// saturated primaries like pure blue don't muddy on mixing) AND has enough freedom
/// to decouple green from purple — which the old 3-channel additive basis could not
/// (its R channel was shared by red+yellow, so a violet-capable basis muddied the
/// green; this one keeps blue+yellow→green AND blue+red→violet, both clean).
///
/// The six COLORED base reflectances are **clean-room DERIVED by us**: each is
/// `base + 2 Gaussian lobes`, the parameters numerically optimised (coordinate
/// descent, `/tmp` lab) so the K–M mixes land on the PUBLIC real-world artist
/// targets — blue+yellow→a real green (#3D933E), blue+red→a real violet (#6A2A8A),
/// complementaries→neutral grey, red+white→pink. These are OUR numbers — no
/// scrtwpns/Burns data. White = flat high reflectance.
/// Per row: `[base, amp1, centre1, sigma1, amp2, centre2, sigma2]`.
const BASE_PARAMS: [[f32; 7]; 6] = [
    [0.067, 23.309, 1.165, 4.925, 8.093, 11.064, 1.979], // Cyan
    [-2.171, 8.815, 1.623, 5.321, 15.469, 24.292, 6.610], // Magenta
    [-1.216, 12.106, 21.620, 5.271, 3.491, 9.991, 2.896], // Yellow
    [0.132, 34.229, 20.032, 1.443, -0.538, 16.676, -1.239], // Red
    [-0.371, 0.498, 21.058, 5.198, 15.181, 11.387, 2.259], // Green
    [-0.944, 5.944, -2.319, 5.700, 1.167, 13.328, 12.408], // Blue
];
/// Flat reflectance of the White base curve.
const WHITE_REFL: f32 = 0.97;

struct SpectralBasis {
    /// 7 base reflectance spectra: `[White, Cyan, Magenta, Yellow, Red, Green, Blue]`.
    base: [[f32; NB]; 7],
    /// Integration matrix (reflectance → linear sRGB), each row normalised to sum 1
    /// so a flat unit reflectance integrates to white.
    m: [[f32; NB]; 3],
}

static BASIS: LazyLock<SpectralBasis> = LazyLock::new(|| {
    // Build the 6 colored base spectra from the derived parameters (White = flat).
    // Mirror of the `/tmp` optimiser: `amp.max(0)`, Gaussian denom `sigma.max(0.4)`.
    let mut base = [[WHITE_REFL; NB]; 7];
    for (k, p) in BASE_PARAMS.iter().enumerate() {
        for (i, bi) in base[k + 1].iter_mut().enumerate() {
            let fi = i as f32;
            let g1 = (-((fi - p[2]) / p[3].abs().max(0.4)).powi(2)).exp();
            let g2 = (-((fi - p[5]) / p[6].abs().max(0.4)).powi(2)).exp();
            *bi = (p[0] + p[1].max(0.0) * g1 + p[4].max(0.0) * g2).clamp(0.015, 0.99);
        }
    }
    // Integration matrix: sharp Gaussians per channel, normalised to sum 1.
    let mut m = [[0.0f32; NB]; 3];
    for c in 0..3 {
        let mut sum = 0.0f32;
        for (i, mi) in m[c].iter_mut().enumerate() {
            let d = (i as f32 - CENTERS[c]) / SIGMAS_INTEG[c];
            *mi = (-d * d).exp();
            sum += *mi;
        }
        for mi in m[c].iter_mut() {
            *mi /= sum;
        }
    }
    SpectralBasis { base, m }
});

/// The constant spectral basis — reconstruction `base[7][NB]` (White + 6 colored
/// curves) + integration `m[3][NB]` (reflectance → linear sRGB). Exposed so a GPU
/// port uploads it ONCE and runs [`to_reflectance`]/[`reflectance_to_rgb`] inline
/// (the per-pixel LUT is a CPU-only perf cache — the GPU's parallel ALUs don't
/// need it). Reads the same `LazyLock` the CPU mix uses, so the numbers are
/// bit-identical to the shipped path.
#[must_use]
pub fn spectral_basis() -> ([[f32; NB]; 7], [[f32; NB]; 3]) {
    let b = &*BASIS;
    (b.base, b.m)
}

/// Kubelka–Munk forward map: reflectance → absorption/scattering ratio `K/S`.
#[inline]
fn refl_to_ks(r: f32) -> f32 {
    let r = r.clamp(REFL_FLOOR, 1.0);
    (1.0 - r) * (1.0 - r) / (2.0 * r)
}

/// Kubelka–Munk inverse map: `K/S` → reflectance.
#[inline]
fn ks_to_refl(k: f32) -> f32 {
    let k = k.max(0.0);
    (1.0 + k - (k * k + 2.0 * k).sqrt()).clamp(0.0, 1.0)
}

/// Split a linear-sRGB colour into the 7 base-curve weights `[W,C,M,Y,R,G,B]`
/// (White + the two nearest secondary/primary). The standard RGB→CMY cube
/// partition: pull out the achromatic White component, then the dominant secondary
/// (Cyan/Magenta/Yellow), leaving a single primary — so at most THREE weights are
/// non-zero and the reconstruction stays smooth + saturated. Full-gamut: every
/// sRGB maps exactly (no residual), which is why mixing saturated primaries no
/// longer muddies.
fn rgb_to_weights(rgb: [f32; 3]) -> [f32; 7] {
    let (mut r, mut g, mut b) = (
        rgb[0].clamp(0.0, 1.0),
        rgb[1].clamp(0.0, 1.0),
        rgb[2].clamp(0.0, 1.0),
    );
    let w = r.min(g).min(b);
    r -= w;
    g -= w;
    b -= w;
    let (mut c, mut m, mut y, mut rr, mut gg, mut bb) = (0.0, 0.0, 0.0, 0.0, 0.0, 0.0);
    if r <= g && r <= b {
        c = g.min(b); // cyan absorbs red; leftover is green or blue
        gg = g - c;
        bb = b - c;
    } else if g <= r && g <= b {
        m = r.min(b); // magenta absorbs green; leftover is red or blue
        rr = r - m;
        bb = b - m;
    } else {
        y = r.min(g); // yellow absorbs blue; leftover is red or green
        rr = r - y;
        gg = g - y;
    }
    [w, c, m, y, rr, gg, bb]
}

/// Linear-sRGB → reflectance spectrum via the 7-curve reconstruction (full gamut,
/// no residual). NOT round-trip exact — [`mix_prepared`] re-anchors per colour so
/// endpoints + self-mix stay the identity; the basis is shaped so the *mixes* land
/// on the real-world targets, not so each colour round-trips.
fn to_reflectance(rgb: [f32; 3]) -> [f32; NB] {
    let b = &*BASIS;
    let wt = rgb_to_weights(rgb);
    let mut refl = [0.0f32; NB];
    for (i, r) in refl.iter_mut().enumerate() {
        let mut v = 0.0f32;
        for (w, base) in wt.iter().zip(b.base.iter()) {
            v += w * base[i];
        }
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

// ── precomputed RGB → (reflectance, round-trip) LUT — the per-pixel fast path ──
//
// The per-pixel cost is reconstructing the BACKDROP's reflectance spectrum (the
// backdrop varies per pixel; the brush is prepared ONCE per stamp). A trilinear
// LUT over a 17³ RGB grid replaces that reconstruction. We store the reflectance
// itself (bounded `[REFL_FLOOR,1]`, smooth under interpolation — unlike `K/S`,
// which spikes toward ∞ on dark bands and interpolates badly) plus the leaky
// round-trip; the K–M map is applied per band at mix time. Built once at init.
const LUT_N: usize = 17;

struct PigmentLut {
    /// Reflectance per band, and `integrate(reflectance)` (the leaky round-trip),
    /// at each `LUT_N³` RGB grid point (`idx=(bi*N+gi)*N+ri`).
    refl: Vec<[f32; NB]>,
    roundtrip: Vec<[f32; 3]>,
}

static LUT: LazyLock<PigmentLut> = LazyLock::new(|| {
    let n = LUT_N;
    let denom = (n - 1) as f32;
    let mut refl_grid = vec![[0.0f32; NB]; n * n * n];
    let mut roundtrip = vec![[0.0f32; 3]; n * n * n];
    for bi in 0..n {
        for gi in 0..n {
            for ri in 0..n {
                let rgb = [ri as f32 / denom, gi as f32 / denom, bi as f32 / denom];
                let refl = to_reflectance(rgb);
                let idx = (bi * n + gi) * n + ri;
                refl_grid[idx] = refl;
                roundtrip[idx] = reflectance_to_rgb(&refl);
            }
        }
    }
    PigmentLut {
        refl: refl_grid,
        roundtrip,
    }
});

/// Trilinear sample of `(reflectance, round-trip)` for a linear-sRGB `[0,1]³`.
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
    let mut refl = [0.0f32; NB];
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
        let lr = &lut.refl[idx];
        for (k, r) in refl.iter_mut().enumerate() {
            *r += lr[k] * weight;
        }
        let r = lut.roundtrip[idx];
        rt[0] += r[0] * weight;
        rt[1] += r[1] * weight;
        rt[2] += r[2] * weight;
    }
    (refl, rt)
}

// ──────────────────────────── public API ────────────────────────────────

/// A brush colour with its per-band `K/S` + round-trip error precomputed — built
/// ONCE per stamp (the brush colour is constant across the footprint), so the
/// per-pixel [`mix_prepared`] only reconstructs the BACKDROP and does the linear
/// `K/S` blend. The brush side is reconstructed EXACTLY (not via the LUT) since
/// it is amortised across the whole footprint.
pub struct PreparedPigment {
    color: [f32; 3],
    /// Kubelka–Munk `K/S` per band for the brush reflectance.
    ks: [f32; NB],
    /// `color − integrate(reflectance)` — the re-anchor correction term.
    err: [f32; 3],
}

impl PreparedPigment {
    /// Brush colour (linear sRGB) — the mix endpoint at `t=1` (`mix_prepared`'s `b`).
    #[must_use]
    pub fn color(&self) -> [f32; 3] {
        self.color
    }
    /// Per-band Kubelka–Munk `K/S` of the brush reflectance (the amortised brush side).
    #[must_use]
    pub fn ks(&self) -> [f32; NB] {
        self.ks
    }
    /// Round-trip re-anchor `color − integrate(reflectance)` (keeps endpoints exact).
    #[must_use]
    pub fn err(&self) -> [f32; 3] {
        self.err
    }
}

/// Precompute a brush colour for repeated [`mix_prepared`] calls over a stamp.
#[must_use]
pub fn prepare_pigment(color: [f32; 3]) -> PreparedPigment {
    let refl = to_reflectance(color);
    let rt = reflectance_to_rgb(&refl);
    let mut ks = [0.0f32; NB];
    for (i, k) in ks.iter_mut().enumerate() {
        *k = refl_to_ks(refl[i]);
    }
    PreparedPigment {
        color,
        ks,
        err: sub3(color, rt),
    }
}

/// Endpoint-exact subtractive (Kubelka–Munk) mix of a backdrop `a` toward the
/// prepared `brush` at concentration `t`. Per band: blend the two `K/S` values
/// linearly and invert back to reflectance; the round-trip is re-anchored
/// per-colour so self-mix + endpoints are the EXACT identity (the leaky-basis
/// errors cancel). Skips the spectral solve where the endpoint blend weight
/// `4t(1−t)` is ~0 (stamp edges / near-opaque), where the linear lerp is the
/// answer anyway.
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
    let (refl_a, rt_a) = lut_sample(a);
    let mut rm = [0.0f32; NB];
    for (i, r) in rm.iter_mut().enumerate() {
        let ks_mix = (1.0 - t) * refl_to_ks(refl_a[i]) + t * brush.ks[i];
        *r = ks_to_refl(ks_mix);
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

/// Exact (no-LUT) twin of [`mix_prepared`]: reconstructs the backdrop reflectance
/// directly with [`to_reflectance`] instead of the trilinear LUT cache. Bit-for-bit
/// the same algorithm; only the backdrop-side fast path differs. The wet-field
/// composite ([`crate::wet_composite`]) uses THIS variant so its GPU port — which
/// has no LUT (ADR-0049 §2; the GPU runs the spectral solve inline) — agrees with
/// the CPU to float precision, and the CPU fallback is the true parity reference.
/// The LUT [`mix_prepared`] stays the per-dab hot-path version.
#[must_use]
pub fn mix_prepared_exact(brush: &PreparedPigment, a: [f32; 3], t: f32) -> [f32; 3] {
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
    let refl_a = to_reflectance(a);
    let rt_a = reflectance_to_rgb(&refl_a);
    let mut rm = [0.0f32; NB];
    for (i, r) in rm.iter_mut().enumerate() {
        let ks_mix = (1.0 - t) * refl_to_ks(refl_a[i]) + t * brush.ks[i];
        *r = ks_to_refl(ks_mix);
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

// ───────────────────── wet-field multi-pigment mix (ADR-0080) ─────────────────────
//
// The wet diffusion field (`crate::diffusion`, + its GPU mirror) carries, per cell, the
// MASS-WEIGHTED ACCUMULATION of a [`PreparedPigment`]: `ks_acc = Σ_i mass_i·ks_i` (per
// band), `err_acc = Σ_i mass_i·err_i` (the round-trip re-anchor), and `mass = Σ_i mass_i`
// (coverage). Because K/S blends LINEARLY by concentration (the K–M law [`mix_prepared`]
// uses), these accumulators TRANSPORT linearly (diffuse/advect/capillary — exactly as the
// old `[f32;3]` pigment did) and the subtractive mix emerges for free: where two pigments
// meet, their `ks_acc`/`err_acc`/`mass` sum, and the per-cell reduction below yields the
// mixed colour — blue+yellow → green, NOT the mud a per-channel-RGB K/S (or a reflectance
// average) gives, because this is the SAME 24-band spectral model as [`mix_prepared`]. A
// single pigment reduces to exactly [`prepare_pigment`]`(colour)`, so the single-pigment
// composite is byte-identical to the pre-ADR-0080 path.

/// Reduce a wet-field cell's mass-weighted K/S accumulation to a [`PreparedPigment`] — the
/// per-cell brush the composite glazes over the backdrop (ADR-0080). `ks_acc` = `Σ mass_i·ks_i`,
/// `err_acc` = `Σ mass_i·err_i`, `mass` = `Σ mass_i`. Yields the mixed pigment's `K/S`
/// (`ks_acc/mass`), its re-anchored colour, and `err`; for a single pigment this is exactly
/// `prepare_pigment(colour)`. `mass ≈ 0` ⇒ a black/zero pigment (the caller skips bare cells).
#[must_use]
pub fn prepared_from_field(ks_acc: &[f32; NB], err_acc: [f32; 3], mass: f32) -> PreparedPigment {
    let inv = if mass > 1.0e-12 { 1.0 / mass } else { 0.0 };
    let mut ks = [0.0f32; NB];
    let mut refl = [0.0f32; NB];
    for i in 0..NB {
        ks[i] = ks_acc[i] * inv;
        refl[i] = ks_to_refl(ks[i]);
    }
    let rt = reflectance_to_rgb(&refl);
    let err = [err_acc[0] * inv, err_acc[1] * inv, err_acc[2] * inv];
    let color = [
        (rt[0] + err[0]).max(0.0),
        (rt[1] + err[1]).max(0.0),
        (rt[2] + err[2]).max(0.0),
    ];
    PreparedPigment { color, ks, err }
}

/// The mixed linear-sRGB colour of a wet-field cell (the `color()` of [`prepared_from_field`])
/// — the convenience reduction for the field's per-cell colour + the parity tests.
#[must_use]
pub fn ks_field_color(ks_acc: &[f32; NB], err_acc: [f32; 3], mass: f32) -> [f32; 3] {
    prepared_from_field(ks_acc, err_acc, mass).color()
}

/// Subtractive pigment mix of two **linear-sRGB** colours (the canonical Mixbox
/// working space, ADR-0051 §2.4). `t ∈ [0,1]` is the ratio toward `b`. Endpoints
/// are exact; the subtractive (geometric-mean) behaviour peaks at an even mix. For
/// a brush stroke (constant brush colour) prefer [`prepare_pigment`] +
/// [`mix_prepared`] to amortise the brush-side cost.
pub fn pigment_lerp_linear(a: [f32; 3], b: [f32; 3], t: f32) -> [f32; 3] {
    mix_prepared(&prepare_pigment(b), a, t)
}

/// Subtractive pigment mix of two **sRGB-8** colours — the spectral mix runs in
/// linear light (Mixbox's only legal space), so this decodes → mixes → re-encodes.
pub fn pigment_lerp_srgb8(a: [u8; 3], b: [u8; 3], t: f32) -> [u8; 3] {
    let dec = |c: [u8; 3]| {
        [
            srgb8_to_linear(c[0]),
            srgb8_to_linear(c[1]),
            srgb8_to_linear(c[2]),
        ]
    };
    let out = pigment_lerp_linear(dec(a), dec(b), t);
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
        assert_eq!(
            pigment_lerp_srgb8([10, 20, 30], [200, 100, 50], 0.0),
            [10, 20, 30]
        );
    }

    #[test]
    fn lerp_at_t1_returns_b() {
        assert_eq!(
            pigment_lerp_srgb8([10, 20, 30], [200, 100, 50], 1.0),
            [200, 100, 50]
        );
    }

    #[test]
    fn linear_endpoints_are_exact_and_t_clamps() {
        let a = [0.1, 0.4, 0.8];
        let b = [0.9, 0.2, 0.3];
        assert!(approx(pigment_lerp_linear(a, b, 0.0), a, 1e-6));
        assert!(approx(pigment_lerp_linear(a, b, 1.0), b, 1e-6));
        assert!(
            approx(pigment_lerp_linear(a, b, 2.0), b, 1e-6),
            "t>1 clamps to b"
        );
        assert!(
            approx(pigment_lerp_linear(a, b, -1.0), a, 1e-6),
            "t<0 clamps to a"
        );
    }

    #[test]
    fn self_mix_is_identity() {
        // A colour mixed with itself at any ratio is ~unchanged (the round-trip is
        // re-anchored). The fast-path LUT (`ln(refl)` interpolated vs the round-trip
        // interpolated) leaves a sub-1% residual between grid points — imperceptible
        // for "paint a colour over itself", and the cost of the 2× per-pixel speedup.
        for c in [
            [0.8, 0.1, 0.1],
            [0.1, 0.6, 0.2],
            [0.2, 0.3, 0.9],
            [0.5, 0.5, 0.5],
        ] {
            for &t in &[0.25, 0.5, 0.75] {
                assert!(
                    approx(pigment_lerp_linear(c, c, t), c, 1e-2),
                    "self-mix {c:?} @ {t} drifted: {:?}",
                    pigment_lerp_linear(c, c, t)
                );
            }
        }
    }

    #[test]
    fn exact_matches_lut_within_residual_and_endpoints_are_exact() {
        // `mix_prepared_exact` is the LUT-free twin: endpoints stay the identity and
        // it tracks the LUT path to within the LUT's own sub-grid residual (~1%), so
        // it is a faithful (more accurate) reference for the GPU composite port.
        for (a, b) in [
            ([0.1, 0.4, 0.8], [0.9, 0.2, 0.3]),
            ([0.0, 0.0, 1.0], [1.0, 1.0, 0.0]),
            ([0.2, 0.6, 0.1], [0.7, 0.1, 0.5]),
        ] {
            let prep = prepare_pigment(b);
            assert!(
                approx(mix_prepared_exact(&prep, a, 0.0), a, 1e-6),
                "t=0 → a"
            );
            assert!(
                approx(mix_prepared_exact(&prep, a, 1.0), b, 1e-6),
                "t=1 → b"
            );
            for &t in &[0.2, 0.5, 0.8] {
                let lut = mix_prepared(&prep, a, t);
                let exact = mix_prepared_exact(&prep, a, t);
                assert!(
                    approx(lut, exact, 1.5e-2),
                    "exact vs LUT drift {a:?}->{b:?}@{t}: {exact:?} vs {lut:?}"
                );
            }
        }
    }

    #[test]
    fn white_plus_white_is_white() {
        let w = [1.0, 1.0, 1.0];
        assert!(approx(pigment_lerp_linear(w, w, 0.5), w, 3e-3));
    }

    #[test]
    fn blue_plus_yellow_is_green_not_grey() {
        // THE smoke (eval Inovação 2): pure blue + pure yellow at 50/50 must give a
        // GREEN-dominant colour — green clearly the top channel, red + blue both
        // suppressed — not the muddy grey a linear lerp ((0.5,0.5,0.5)) produces.
        let blue = [0.0, 0.0, 1.0];
        let yellow = [1.0, 1.0, 0.0];
        let mix = pigment_lerp_linear(blue, yellow, 0.5);
        assert!(
            mix[1] > mix[0] && mix[1] > mix[2],
            "green is the dominant channel: {mix:?}"
        );
        assert!(
            mix[1] > 0.22,
            "green is a real mid green, not dark: {mix:?}"
        );
        // Clearly GREEN, not a teal (green over blue) and not grey (green over red).
        // The 7-curve full-gamut basis suppresses residual red AND blue HARD: both
        // are ~0.06 vs green ~0.25 — a clean grass green, no teal, no olive.
        assert!(
            mix[1] > mix[2] + 0.15,
            "green clearly leads blue (not teal): {mix:?}"
        );
        assert!(
            mix[0] < 0.12 && mix[2] < 0.12,
            "red AND blue both suppressed: {mix:?}"
        );
        assert!(
            mix[1] - mix[0].max(mix[2]) > 0.05,
            "green clearly dominates (vs the linear grey midpoint 0.5,0.5,0.5): {mix:?}"
        );
    }

    #[test]
    fn blue_plus_red_is_violet_not_maroon() {
        // The 7-curve basis fix: pure blue + pure red = a REAL violet (blue clearly
        // present), not the dark red maroon the old 3-channel basis gave (which
        // killed the blue). Blue must lead — or at least strongly survive — vs red.
        let mix = pigment_lerp_linear([0.0, 0.0, 1.0], [1.0, 0.0, 0.0], 0.5);
        assert!(
            mix[2] > mix[0],
            "violet keeps blue dominant over red (not maroon): {mix:?}"
        );
        assert!(
            mix[2] > mix[1] + 0.1,
            "blue clearly above green (a violet, not a mud): {mix:?}"
        );
    }

    #[test]
    fn complementary_cyan_plus_red_neutralises() {
        // Complementaries should desaturate toward neutral grey (chroma collapses),
        // not stay a saturated hue — the hallmark of physically-plausible mixing.
        let mix = pigment_lerp_linear([0.0, 1.0, 1.0], [1.0, 0.0, 0.0], 0.5);
        let chroma =
            mix.iter().cloned().fold(0.0, f32::max) - mix.iter().cloned().fold(1.0, f32::min);
        assert!(
            chroma < 0.12,
            "cyan+red neutralises toward grey: {mix:?} chroma {chroma}"
        );
    }

    /// Accumulate `n` pigments' mass-weighted K/S + err into `(ks_acc, err_acc, mass)` — the
    /// wet-field deposit path (ADR-0080), as a test helper.
    fn accumulate(pigments: &[([f32; 3], f32)]) -> ([f32; NB], [f32; 3], f32) {
        let mut ks_acc = [0.0f32; NB];
        let mut err_acc = [0.0f32; 3];
        let mut mass = 0.0f32;
        for &(c, m) in pigments {
            let p = prepare_pigment(c);
            for (a, &k) in ks_acc.iter_mut().zip(p.ks.iter()) {
                *a += m * k;
            }
            for (e, &pe) in err_acc.iter_mut().zip(p.err.iter()) {
                *e += m * pe;
            }
            mass += m;
        }
        (ks_acc, err_acc, mass)
    }

    #[test]
    fn field_mix_blue_plus_yellow_is_green() {
        // **THE P0 gate (ADR-0080).** The wet-field accumulation path (mass-weighted K/S) mixes
        // blue + yellow to a GREEN-dominant colour — the same subtractive result the validated
        // 2-colour `mix_prepared` gives, NOT the mud a per-channel-RGB K/S (∞ on dark bands →
        // black) or a reflectance average produces.
        let (ks, err, m) = accumulate(&[([0.0, 0.0, 1.0], 1.0), ([1.0, 1.0, 0.0], 1.0)]);
        let mix = ks_field_color(&ks, err, m);
        assert!(
            mix[1] > mix[0] && mix[1] > mix[2],
            "green is the dominant channel: {mix:?}"
        );
        assert!(mix[1] > 0.22, "green is a real mid green, not dark: {mix:?}");
        assert!(
            mix[0] < 0.12 && mix[2] < 0.12,
            "red AND blue both suppressed (not mud, not teal): {mix:?}"
        );
    }

    #[test]
    fn field_mix_equals_mix_prepared_at_5050() {
        // The field accumulation at EQUAL mass is exactly `mix_prepared_exact(brush_b, a, 0.5)`
        // — proof the field reuses the validated K–M (the N-pigment generalisation), not a new
        // model. (The `4t(1-t)` linear blend is glaze-only; the field mix is the pure K/S blend,
        // which `mix_prepared` at t=0.5 reduces to since w = 4·0.5·0.5 = 1.)
        for (a, b) in [
            ([0.0, 0.0, 1.0], [1.0, 1.0, 0.0]),
            ([0.1, 0.4, 0.8], [0.9, 0.2, 0.3]),
            ([0.2, 0.6, 0.1], [0.7, 0.1, 0.5]),
        ] {
            let (ks, err, m) = accumulate(&[(a, 1.0), (b, 1.0)]);
            let field = ks_field_color(&ks, err, m);
            let glaze = mix_prepared_exact(&prepare_pigment(b), a, 0.5);
            assert!(
                approx(field, glaze, 1e-5),
                "field {field:?} vs mix_prepared {glaze:?} for {a:?}+{b:?}"
            );
        }
    }

    #[test]
    fn field_single_pigment_is_endpoint_exact() {
        // §2.3: a SINGLE pigment at any mass reduces to its own colour (the `err` re-anchor
        // undoes the leaky-basis round-trip) — this is what makes the single-colour composite
        // byte-identical to the pre-ADR-0080 path.
        for c in [
            [0.8, 0.1, 0.1],
            [0.1, 0.6, 0.2],
            [0.2, 0.3, 0.9],
            [0.05, 0.05, 0.05],
            [0.0, 0.0, 0.0],
        ] {
            for &m in &[0.3f32, 1.0, 5.0] {
                let (ks, err, mass) = accumulate(&[(c, m)]);
                let out = ks_field_color(&ks, err, mass);
                assert!(approx(out, c, 1e-5), "single pigment {c:?}@{m} -> {out:?}");
            }
        }
    }

    #[test]
    fn field_mix_is_more_saturated_than_a_grey_lerp() {
        // The field mix keeps chroma (not the dull linear average) — the subtractive signature.
        let (ks, err, m) = accumulate(&[([0.0, 0.0, 1.0], 1.0), ([1.0, 1.0, 0.0], 1.0)]);
        let mix = ks_field_color(&ks, err, m);
        let chroma =
            mix.iter().cloned().fold(0.0, f32::max) - mix.iter().cloned().fold(1.0, f32::min);
        assert!(chroma > 0.17, "field mix keeps chroma: {mix:?} chroma {chroma}");
    }

    #[test]
    fn mix_is_more_saturated_than_a_grey_lerp() {
        // The subtractive mix should not collapse to the dull linear average.
        let blue = [0.0, 0.0, 1.0];
        let yellow = [1.0, 1.0, 0.0];
        let mix = pigment_lerp_linear(blue, yellow, 0.5);
        let chroma =
            mix.iter().cloned().fold(0.0, f32::max) - mix.iter().cloned().fold(1.0, f32::min);
        assert!(
            chroma > 0.17,
            "mix keeps chroma (not grey): {mix:?} chroma {chroma}"
        );
    }
}
