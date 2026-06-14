//! Spectral subtractive colour for the Wash mode (ADR-0086 §8.1) — the **optional** Kubelka–Munk
//! pigment-mixing model, side-by-side with the default RGB Beer–Lambert path (this module does NOT
//! replace it; the colour model is a per-brush choice). RGB-only mixing is metameric: blue+yellow
//! collapses to grey because three channels can't keep the green both pigments reflect. Mixing in a
//! small SPECTRAL basis fixes it (blue+yellow→green).
//!
//! Design (transparent watercolour, no scattering ⇒ spectral Beer–Lambert over the backdrop):
//! - **N=16 wavelength samples**, three smooth Gaussian channel curves `C = [r,g,b]` (overlapping
//!   in green — that overlap is what makes the mix vibrant).
//! - **RGB→spectrum** (upsample): `S(λ) = Σ_ch rgb_ch · C_ch(λ)` (= `Cᵀ·rgb`).
//! - **spectrum→RGB**: `rgb = G⁻¹·(C·S)`, `G = C·Cᵀ` (3×3) ⇒ the round-trip rgb→S→rgb is the
//!   identity by construction (no embedded CMF tables; everything is computed from the curves).
//! - **4 base pigments** defined by masstone RGB; each pigment's absorbance per unit concentration
//!   is `ap_i(λ) = −ln(clamp(S_{mᵢ}/S_white, ε, 1))` so one unit over white reproduces its masstone.
//! - **Mix**: concentrations add (linear ⇒ the solver's conservative gather transports them
//!   UNCHANGED — same vec4, same physics); colour = `backdrop_spectrum · exp(−Σ cᵢ apᵢ)` → RGB.
//! - **Unmix** (brush RGB → 4 concentrations): NNLS `M c ≈ A_target`, `M[λ,i]=apᵢ(λ)`,
//!   `A_target(λ) = −ln(clamp(S_target/S_white, ε, 1))`, `c ≥ 0`.
//!
//! Pure Rust, zero deps — the GPU composite (WGSL) mirrors `compose_over` using the tables this
//! exposes ([`KmModel::pigment_absorbance`], [`KmModel::upsample_basis`], [`KmModel::to_rgb_matrix`]).

// Index-based loops over parallel fixed-size spectral arrays (curves/white/absorb, all `[_; N]`)
// read clearer than zipped iterators here — this is dense numeric code mirrored 1:1 in WGSL.
#![allow(clippy::needless_range_loop)]

/// Wavelength samples (400–700 nm, 20 nm spacing).
pub const N: usize = 16;
/// Number of base pigments (the vec4 the field carries in K–M mode).
pub const PIGMENTS: usize = 4;

/// Reference total concentration (Σc) at which a hue is reproduced (ADR-0089 §2.2). The unmix
/// resolves concentrations summing to `K_REF`; the K–M display composite reads the field's
/// concentration *ratio* re-scaled to `K_REF` for the HUE, and takes coverage (lightness vs the
/// backdrop, incl. edge-darkening) from the *actual* total. So the painted hue equals the picked hue
/// at ANY accumulated mass — the magnitude→hue drift (`exp(−c·a)` ⇒ `Tᶜ`) that turned red→orange is
/// gone. Tuned (see `saturated_hues_reproduce_faithfully`) so saturated sRGB primaries land inside
/// the 4-pigment gamut at this magnitude.
pub const K_REF: f32 = 3.0;

/// Default coverage softness `k` for the display composites: `cover = 1 − exp(−eff/k)` where the
/// *effective mass* is `mass` (Linear) or `Σc / K_REF` (K–M) — so both modes build opacity at the
/// same rate for the same stroke. The bridge passes the brush's `coverage_k`; this is the fallback /
/// test default. Lower = more opaque per unit paint.
pub const COVER_K: f32 = 0.6;

const LAMBDA0: f32 = 400.0;
const LAMBDA_STEP: f32 = 20.0;
const EPS: f32 = 1.0e-4;

/// Base pigment masstones (linear sRGB), in field-channel order: Cyan, Magenta, Yellow, Black.
/// CMY span the subtractive gamut; K (black) reaches neutral darks without muddy CMY overlap.
pub const PIGMENT_MASSTONE: [[f32; 3]; PIGMENTS] = [
    [0.10, 0.55, 0.80], // cyan (phthalo-ish)
    [0.80, 0.10, 0.45], // magenta (quinacridone-ish)
    [0.92, 0.78, 0.08], // yellow (hansa-ish)
    [0.04, 0.04, 0.05], // black (neutral darkener)
];

fn gaussian(lambda: f32, center: f32, sigma: f32) -> f32 {
    let z = (lambda - center) / sigma;
    (-0.5 * z * z).exp()
}

/// The precomputed spectral operators (built once; cheap to clone).
#[derive(Clone, Debug)]
pub struct KmModel {
    /// Channel curves `C_ch(λ)` — rows R,G,B. Doubles as the RGB→spectrum basis.
    curves: [[f32; N]; 3],
    /// White spectrum `S_white(λ) = Σ_ch C_ch(λ)`.
    white: [f32; N],
    /// `G⁻¹` where `G = C·Cᵀ` (3×3) — the spectrum→RGB normaliser.
    g_inv: [[f32; 3]; 3],
    /// Per-pigment absorbance `apᵢ(λ)` (unit concentration ⇒ masstone over white).
    absorb: [[f32; N]; PIGMENTS],
}

impl Default for KmModel {
    fn default() -> Self {
        Self::new()
    }
}

impl KmModel {
    /// Build the model from the analytic Gaussian channel curves + the base pigment masstones.
    #[must_use]
    pub fn new() -> Self {
        // Channel sensitivity / basis curves — overlapping Gaussians (B short, G mid, R long).
        let centers = [610.0_f32, 540.0, 460.0]; // R, G, B
        let sigma = 46.0_f32;
        let mut curves = [[0.0_f32; N]; 3];
        for (ch, &c) in centers.iter().enumerate() {
            for k in 0..N {
                let lambda = LAMBDA0 + LAMBDA_STEP * k as f32;
                curves[ch][k] = gaussian(lambda, c, sigma);
            }
        }
        let mut white = [0.0_f32; N];
        for k in 0..N {
            white[k] = curves[0][k] + curves[1][k] + curves[2][k];
        }
        // G = C·Cᵀ (3×3), then invert.
        let mut g = [[0.0_f32; 3]; 3];
        for i in 0..3 {
            for j in 0..3 {
                let mut s = 0.0;
                for k in 0..N {
                    s += curves[i][k] * curves[j][k];
                }
                g[i][j] = s;
            }
        }
        let g_inv = invert3(&g);
        // Pigment absorbances from masstones: apᵢ = −ln(clamp(S_mᵢ / S_white, ε, 1)).
        let mut absorb = [[0.0_f32; N]; PIGMENTS];
        for p in 0..PIGMENTS {
            let s_m = upsample(&curves, PIGMENT_MASSTONE[p]);
            for k in 0..N {
                let ratio = (s_m[k] / white[k]).clamp(EPS, 1.0);
                absorb[p][k] = -ratio.ln();
            }
        }
        Self { curves, white, g_inv, absorb }
    }

    /// RGB→spectrum upsample basis (the channel curves), `[3][N]` — row per RGB channel.
    #[must_use]
    pub fn upsample_basis(&self) -> &[[f32; N]; 3] {
        &self.curves
    }
    /// White spectrum `S_white(λ)`.
    #[must_use]
    pub fn white_spectrum(&self) -> &[f32; N] {
        &self.white
    }
    /// Per-pigment absorbance table `apᵢ(λ)`, `[PIGMENTS][N]`.
    #[must_use]
    pub fn pigment_absorbance(&self) -> &[[f32; N]; PIGMENTS] {
        &self.absorb
    }
    /// Per-pigment **RGB** absorbance `−ln(masstoneᵢ)` (3 channels), for the non-spectral "Linear"
    /// composite that reads the SAME concentration field as K–M (so a model flip is a pure
    /// re-render — no re-encode). Mixing here is metameric (blue+yellow→grey), which is exactly the
    /// "RGB look" the Linear mode is meant to show next to K–M's vibrant green.
    #[must_use]
    pub fn pigment_rgb_absorbance(&self) -> [[f32; 3]; PIGMENTS] {
        let mut out = [[0.0_f32; 3]; PIGMENTS];
        for p in 0..PIGMENTS {
            for ch in 0..3 {
                out[p][ch] = -PIGMENT_MASSTONE[p][ch].clamp(EPS, 1.0).ln();
            }
        }
        out
    }

    /// The spectrum→linear-RGB operator `G⁻¹·C`, returned as `[3][N]` (rgb = M·spectrum).
    #[must_use]
    pub fn to_rgb_matrix(&self) -> [[f32; N]; 3] {
        let mut m = [[0.0_f32; N]; 3];
        for (i, row) in m.iter_mut().enumerate() {
            for (k, cell) in row.iter_mut().enumerate() {
                *cell = self.g_inv[i][0] * self.curves[0][k]
                    + self.g_inv[i][1] * self.curves[1][k]
                    + self.g_inv[i][2] * self.curves[2][k];
            }
        }
        m
    }

    /// Linear-sRGB → 4 non-negative pigment concentrations **summing to [`K_REF`]** whose K–M
    /// composite at that magnitude reproduces `rgb`'s hue (ADR-0089 §2.2). Deposit `× mass`; the
    /// display composite re-normalises the accumulated field back to `K_REF` for the hue and takes
    /// coverage from the total, so the painted hue matches the picked hue at ANY mass (no `Tᶜ`
    /// drift). A saturated pick lands near the gamut boundary; a desaturated pick maps to the nearest
    /// in-gamut hue at `K_REF` (its lightness comes from coverage, the watercolor way).
    #[must_use]
    pub fn rgb_to_concentrations(&self, rgb: [f32; 3]) -> [f32; PIGMENTS] {
        let s_t = upsample(&self.curves, rgb);
        let mut a_target = [0.0_f32; N];
        for k in 0..N {
            let ratio = (s_t[k] / self.white[k]).clamp(EPS, 1.0);
            a_target[k] = -ratio.ln();
        }
        // NNLS via multiplicative updates (M, a_target ≥ 0 ⇒ c stays ≥ 0).
        let m = &self.absorb;
        // Precompute Mᵀ a_target and Mᵀ M.
        let mut mta = [0.0_f32; PIGMENTS];
        for (p, mtap) in mta.iter_mut().enumerate() {
            let mut s = 0.0;
            for k in 0..N {
                s += m[p][k] * a_target[k];
            }
            *mtap = s;
        }
        let mut mtm = [[0.0_f32; PIGMENTS]; PIGMENTS];
        for i in 0..PIGMENTS {
            for j in 0..PIGMENTS {
                let mut s = 0.0;
                for k in 0..N {
                    s += m[i][k] * m[j][k];
                }
                mtm[i][j] = s;
            }
        }
        let mut c = [0.25_f32; PIGMENTS];
        for _ in 0..64 {
            for i in 0..PIGMENTS {
                let mut denom = 0.0;
                for j in 0..PIGMENTS {
                    denom += mtm[i][j] * c[j];
                }
                c[i] *= mta[i] / (denom + 1.0e-6);
            }
        }
        // Refine in COLOUR space, PROJECTED onto the `Σc = K_REF` simplex (ADR-0089 §2.2). The
        // absorbance LSQ above weights wavelengths, not the final RGB, so saturated hues drift (pure
        // red leans yellow ⇒ orange). Gradient-descend the actual composite error
        // `‖compose_over(white,c) − rgb‖²` AT the fixed display magnitude `K_REF`, so the returned
        // concentrations reproduce the hue the composite will actually show (which always re-scales
        // the field's ratio to `K_REF`). Projecting each step keeps the optimisation on the magnitude
        // the kernel reads — the piece the old free unmix missed (it solved at a magnitude the
        // capped/accumulated field never matched). `J[·][p] = to_rgb(−white·exp(−A)·apₚ)`.
        let project = |c: &mut [f32; PIGMENTS]| {
            let s: f32 = c.iter().sum();
            if s > 1.0e-6 {
                for v in c.iter_mut() {
                    *v *= K_REF / s;
                }
            }
        };
        for _ in 0..120 {
            project(&mut c);
            let cur = self.compose_over([1.0, 1.0, 1.0], c);
            let res = [cur[0] - rgb[0], cur[1] - rgb[1], cur[2] - rgb[2]];
            let mut a = [0.0_f32; N];
            for k in 0..N {
                let mut s = 0.0;
                for p in 0..PIGMENTS {
                    s += c[p] * self.absorb[p][k];
                }
                a[k] = s;
            }
            for p in 0..PIGMENTS {
                let mut dspec = [0.0_f32; N];
                for k in 0..N {
                    dspec[k] = -self.white[k] * (-a[k]).exp() * self.absorb[p][k];
                }
                let dcol = self.to_rgb_raw(&dspec);
                let grad = dcol[0] * res[0] + dcol[1] * res[1] + dcol[2] * res[2];
                c[p] = (c[p] - 0.6 * grad).max(0.0);
            }
        }
        project(&mut c); // return on the K_REF simplex — the magnitude the composite reads
        c
    }

    /// **Mixbox unmix** (ADR-0091) — linear-sRGB → 4 non-negative pigment concentrations whose K–M
    /// composite best matches `rgb`, with **NO fixed-magnitude constraint** (unlike the legacy
    /// [`Self::rgb_to_concentrations`], which forces `Σc = K_REF` and so discards the pick's value).
    /// The remaining fit error is carried by the additive residual ([`Self::pigment_residual`]), so a
    /// pick INSIDE the 4-pigment gamut resolves with ~zero residual and one OUTSIDE gets the nearest
    /// pigment mix + a residual that restores exact fidelity at the composite. Minimises
    /// `‖compose_over(white,c) − rgb‖²` in colour space (the absorbance NNLS only seeds it).
    #[must_use]
    pub fn unmix(&self, rgb: [f32; 3]) -> [f32; PIGMENTS] {
        let s_t = upsample(&self.curves, rgb);
        let mut a_target = [0.0_f32; N];
        for k in 0..N {
            a_target[k] = -(s_t[k] / self.white[k]).clamp(EPS, 1.0).ln();
        }
        let m = &self.absorb;
        let mut mta = [0.0_f32; PIGMENTS];
        for (p, v) in mta.iter_mut().enumerate() {
            let mut s = 0.0;
            for k in 0..N {
                s += m[p][k] * a_target[k];
            }
            *v = s;
        }
        let mut mtm = [[0.0_f32; PIGMENTS]; PIGMENTS];
        for i in 0..PIGMENTS {
            for j in 0..PIGMENTS {
                let mut s = 0.0;
                for k in 0..N {
                    s += m[i][k] * m[j][k];
                }
                mtm[i][j] = s;
            }
        }
        let mut c = [0.25_f32; PIGMENTS];
        for _ in 0..64 {
            for i in 0..PIGMENTS {
                let mut denom = 0.0;
                for j in 0..PIGMENTS {
                    denom += mtm[i][j] * c[j];
                }
                c[i] *= mta[i] / (denom + 1.0e-6);
            }
        }
        // Refine in COLOUR space (minimise the actual composite error), c ≥ 0, NO magnitude projection
        // — the residual mops up the rest, which is exactly what keeps a single picked colour faithful.
        for _ in 0..120 {
            let cur = self.compose_over([1.0, 1.0, 1.0], c);
            let res = [cur[0] - rgb[0], cur[1] - rgb[1], cur[2] - rgb[2]];
            let mut a = [0.0_f32; N];
            for k in 0..N {
                let mut s = 0.0;
                for p in 0..PIGMENTS {
                    s += c[p] * self.absorb[p][k];
                }
                a[k] = s;
            }
            for p in 0..PIGMENTS {
                let mut dspec = [0.0_f32; N];
                for k in 0..N {
                    dspec[k] = -self.white[k] * (-a[k]).exp() * self.absorb[p][k];
                }
                let dcol = self.to_rgb_raw(&dspec);
                let grad = dcol[0] * res[0] + dcol[1] * res[1] + dcol[2] * res[2];
                c[p] = (c[p] - 0.6 * grad).max(0.0);
            }
        }
        c
    }

    /// **Mixbox encode `F(rgb)`** (ADR-0091) — pigment concentrations + the additive RGB residual
    /// `r = rgb − mix(c)`. Decoding `mix(c) + r` returns `rgb` EXACTLY ⇒ a single picked colour is
    /// faithful; only MIXING two colours (averaging their latents) shows the spectral pigment behaviour
    /// (blue+yellow→green). This is the identity-preserving latent the state-of-the-art uses (Sochorová
    /// & Jamriška, *Practical Pigment Mixing*, SIGGRAPH Asia 2021 — the model Rebelle ships).
    #[must_use]
    pub fn pigment_residual(&self, rgb: [f32; 3]) -> ([f32; PIGMENTS], [f32; 3]) {
        let c = self.unmix(rgb);
        let mixc = self.compose_over([1.0, 1.0, 1.0], c);
        (c, [rgb[0] - mixc[0], rgb[1] - mixc[1], rgb[2] - mixc[2]])
    }

    /// **Mixbox K–M display** (ADR-0091) — decode the accumulated latent `mix(c̄) + r̄` over `backdrop`,
    /// coverage from `mass`. `c_avg`/`res_avg` are the MASS-WEIGHTED averages the field carries (the
    /// caller divides the premultiplied sums by `mass`). A single colour decodes to itself (faithful);
    /// a wet mix decodes to the spectral pigment result. Replaces the value-collapsing
    /// [`Self::compose_km_display`] (which normalised every colour to `K_REF`).
    #[must_use]
    pub fn compose_km_mixbox(&self, backdrop: [f32; 3], c_avg: [f32; PIGMENTS], res_avg: [f32; 3], mass: f32, cover_k: f32) -> [f32; 3] {
        if mass < 1.0e-6 {
            return backdrop;
        }
        let pig = self.compose_over([1.0, 1.0, 1.0], c_avg);
        let color = [pig[0] + res_avg[0], pig[1] + res_avg[1], pig[2] + res_avg[2]];
        let cover = 1.0 - (-mass / cover_k.max(1.0e-3)).exp();
        [
            backdrop[0] + (color[0] - backdrop[0]) * cover,
            backdrop[1] + (color[1] - backdrop[1]) * cover,
            backdrop[2] + (color[2] - backdrop[2]) * cover,
        ]
    }

    /// **K–M display composite** (ADR-0089 §2.2) — the Rust mirror of the WGSL `km_compose`. Hue from
    /// the concentration RATIO re-scaled to [`K_REF`] (mass-independent ⇒ faithful), coverage from the
    /// actual total `Σconc` via `1 − exp(−(Σconc/K_REF)/cover_k)` (drives lightness AND edge-darkening
    /// — a thicker rim reads as MORE OPAQUE of the same hue, never a shifted one). Alpha-mixed over the
    /// linear-sRGB `backdrop`. `conc` zero ⇒ the bare backdrop.
    #[must_use]
    pub fn compose_km_display(&self, backdrop: [f32; 3], conc: [f32; PIGMENTS], cover_k: f32) -> [f32; 3] {
        let total: f32 = conc.iter().sum();
        if total < 1.0e-6 {
            return backdrop;
        }
        let mut ratio = [0.0_f32; PIGMENTS];
        for p in 0..PIGMENTS {
            ratio[p] = conc[p] / total * K_REF;
        }
        let hue = self.compose_over([1.0, 1.0, 1.0], ratio);
        let cover = 1.0 - (-(total / K_REF) / cover_k.max(1.0e-3)).exp();
        [
            backdrop[0] + (hue[0] - backdrop[0]) * cover,
            backdrop[1] + (hue[1] - backdrop[1]) * cover,
            backdrop[2] + (hue[2] - backdrop[2]) * cover,
        ]
    }

    /// Composite 4 pigment concentrations over a linear-sRGB backdrop (the Rust mirror of the WGSL
    /// `cs_composite` K–M branch). `conc` is the per-cell summed concentration vector.
    #[must_use]
    pub fn compose_over(&self, backdrop: [f32; 3], conc: [f32; PIGMENTS]) -> [f32; 3] {
        let b = upsample(&self.curves, backdrop);
        let mut spec = [0.0_f32; N];
        for k in 0..N {
            let mut a = 0.0;
            for p in 0..PIGMENTS {
                a += conc[p] * self.absorb[p][k];
            }
            spec[k] = b[k] * (-a).exp();
        }
        self.to_rgb(&spec)
    }

    /// Spectrum → linear sRGB via `G⁻¹·(C·S)` (final colour ⇒ clamped ≥ 0).
    fn to_rgb(&self, spec: &[f32; N]) -> [f32; 3] {
        let r = self.to_rgb_raw(spec);
        [r[0].max(0.0), r[1].max(0.0), r[2].max(0.0)]
    }

    /// Spectrum → linear sRGB, UNclamped (used as a Jacobian column in the colour-space unmix, where
    /// the derivative spectrum is signed).
    fn to_rgb_raw(&self, spec: &[f32; N]) -> [f32; 3] {
        let mut cs = [0.0_f32; 3];
        for (i, csi) in cs.iter_mut().enumerate() {
            let mut s = 0.0;
            for k in 0..N {
                s += self.curves[i][k] * spec[k];
            }
            *csi = s;
        }
        let mut rgb = [0.0_f32; 3];
        for i in 0..3 {
            rgb[i] = self.g_inv[i][0] * cs[0] + self.g_inv[i][1] * cs[1] + self.g_inv[i][2] * cs[2];
        }
        rgb
    }
}

/// **Linear/RGB display composite** (ADR-0089 §2.2) — the Rust mirror of the WGSL `linear_compose`.
/// The `dye` field carries the picked colour PRE-MULTIPLIED by mass (`rgb·mass`) + the accumulated
/// `mass`; un-premultiplying (`rgb·mass / mass`) recovers the picked colour EXACTLY (faithful by
/// construction), while wet-in-wet overlap of different colours blends as a mass-weighted RGB average
/// = metameric (blue+yellow→grey), the deliberate contrast to K–M's spectral green. Coverage matches
/// [`KmModel::compose_km_display`] (same `cover_k`, effective mass = `mass`). Alpha-mixed over the
/// linear-sRGB `backdrop`; `mass` zero ⇒ the bare backdrop.
#[must_use]
pub fn compose_linear_display(backdrop: [f32; 3], dye_premul: [f32; 3], mass: f32, cover_k: f32) -> [f32; 3] {
    if mass < 1.0e-6 {
        return backdrop;
    }
    let color = [dye_premul[0] / mass, dye_premul[1] / mass, dye_premul[2] / mass];
    let cover = 1.0 - (-mass / cover_k.max(1.0e-3)).exp();
    [
        backdrop[0] + (color[0] - backdrop[0]) * cover,
        backdrop[1] + (color[1] - backdrop[1]) * cover,
        backdrop[2] + (color[2] - backdrop[2]) * cover,
    ]
}

/// `S(λ) = Σ_ch rgb_ch · C_ch(λ)` (clamped ≥ 0).
fn upsample(curves: &[[f32; N]; 3], rgb: [f32; 3]) -> [f32; N] {
    let mut s = [0.0_f32; N];
    for k in 0..N {
        s[k] = (rgb[0] * curves[0][k] + rgb[1] * curves[1][k] + rgb[2] * curves[2][k]).max(0.0);
    }
    s
}

/// Invert a 3×3 matrix (Cramer). `G = C·Cᵀ` is symmetric positive-definite ⇒ always invertible here.
fn invert3(m: &[[f32; 3]; 3]) -> [[f32; 3]; 3] {
    let a = m[0][0];
    let b = m[0][1];
    let c = m[0][2];
    let d = m[1][0];
    let e = m[1][1];
    let f = m[1][2];
    let g = m[2][0];
    let h = m[2][1];
    let i = m[2][2];
    let det = a * (e * i - f * h) - b * (d * i - f * g) + c * (d * h - e * g);
    let inv_det = 1.0 / det;
    [
        [(e * i - f * h) * inv_det, (c * h - b * i) * inv_det, (b * f - c * e) * inv_det],
        [(f * g - d * i) * inv_det, (a * i - c * g) * inv_det, (c * d - a * f) * inv_det],
        [(d * h - e * g) * inv_det, (b * g - a * h) * inv_det, (a * e - b * d) * inv_det],
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    fn argmax(rgb: [f32; 3]) -> usize {
        let mut mi = 0;
        for i in 1..3 {
            if rgb[i] > rgb[mi] {
                mi = i;
            }
        }
        mi
    }

    #[test]
    fn white_round_trips() {
        let km = KmModel::new();
        let out = km.compose_over([1.0, 1.0, 1.0], [0.0; PIGMENTS]); // no pigment ⇒ backdrop
        for c in out {
            assert!((c - 1.0).abs() < 0.02, "white must round-trip (got {out:?})");
        }
    }

    #[test]
    fn pigment_masstone_reproduces() {
        let km = KmModel::new();
        // A unit concentration of each base pigment over white ≈ its masstone hue (argmax matches).
        for p in 0..PIGMENTS {
            let mut conc = [0.0_f32; PIGMENTS];
            conc[p] = 1.0;
            let out = km.compose_over([1.0, 1.0, 1.0], conc);
            let want = PIGMENT_MASSTONE[p];
            // Skip near-neutral black (argmax meaningless); check the chromatic primaries' dominant ch.
            if p < 3 {
                assert_eq!(argmax(out), argmax(want), "pigment {p}: hue {out:?} vs masstone {want:?}");
            }
        }
    }

    #[test]
    fn blue_plus_yellow_makes_green_not_grey() {
        let km = KmModel::new();
        let cb = km.rgb_to_concentrations([0.05, 0.05, 0.85]); // blue
        let cy = km.rgb_to_concentrations([0.90, 0.80, 0.05]); // yellow
        let mut mix = [0.0_f32; PIGMENTS];
        for p in 0..PIGMENTS {
            mix[p] = cb[p] + cy[p];
        }
        let out = km.compose_over([1.0, 1.0, 1.0], mix);
        eprintln!("blue+yellow K–M = {out:?}");
        // The win vs RGB Beer–Lambert (which gives grey): green is the dominant channel and clearly
        // above grey (not r≈g≈b).
        assert_eq!(argmax(out), 1, "blue+yellow must mix to GREEN (g dominant), got {out:?}");
        let avg = (out[0] + out[1] + out[2]) / 3.0;
        assert!(out[1] > avg * 1.15, "green must stand out from grey (g={} avg={avg})", out[1]);
    }

    #[test]
    fn stacking_darkens() {
        let km = KmModel::new();
        let c = km.rgb_to_concentrations([0.8, 0.1, 0.1]); // red
        let single = km.compose_over([1.0, 1.0, 1.0], c);
        let double = km.compose_over([1.0, 1.0, 1.0], [c[0] * 2.0, c[1] * 2.0, c[2] * 2.0, c[3] * 2.0]);
        let lum = |x: [f32; 3]| x[0] + x[1] + x[2];
        assert!(lum(double) < lum(single), "raw spectral glaze must darken with concentration ({single:?} → {double:?})");
    }

    // ── ADR-0089 §2.2 — colour fidelity (the BUG-C fix) ──────────────────────────────────────────

    fn scale(c: [f32; PIGMENTS], k: f32) -> [f32; PIGMENTS] {
        let mut o = c;
        for v in &mut o {
            *v *= k;
        }
        o
    }

    /// The reported bug: pure red painted ORANGE in K–M. Each saturated pick must reproduce with the
    /// correct DOMINANT channel and clearly above the others, at the `K_REF` magnitude the K–M display
    /// composite reads. (Prints the composed RGB so the gamut fit / `K_REF` can be eyeballed.)
    #[test]
    fn saturated_hues_reproduce_faithfully() {
        let km = KmModel::new();
        let cases: [(&str, [f32; 3], usize); 3] = [
            ("red", [1.0, 0.0, 0.0], 0),
            ("green", [0.0, 1.0, 0.0], 1),
            ("blue", [0.0, 0.0, 1.0], 2),
        ];
        for (name, rgb, dom) in cases {
            let c = km.rgb_to_concentrations(rgb);
            let sum: f32 = c.iter().sum();
            let out = km.compose_over([1.0, 1.0, 1.0], c); // hue at Σ = K_REF (full coverage)
            eprintln!("{name}: pick {rgb:?} → Σc={sum:.2} conc={c:?} → composed {out:?}");
            assert!((sum - K_REF).abs() < 0.05, "{name}: unmix must sum to K_REF (got {sum})");
            assert_eq!(argmax(out), dom, "{name}: dominant channel must stay {name} (got {out:?})");
            // The killer for red→orange: the dominant channel must lead the next by a clear margin
            // (orange = G close to R; faithful red = G well below R).
            let mut sorted = out;
            sorted.sort_by(|a, b| b.partial_cmp(a).unwrap());
            assert!(sorted[0] > sorted[1] + 0.12, "{name}: hue must be clearly saturated, not muddy ({out:?})");
        }
    }

    /// Linear/RGB mode (the DEFAULT): a thick (opaque) dye stroke must return the picked colour
    /// essentially bit-for-bit — the "sem pigment vermelho fica amarelo" fix.
    #[test]
    fn dye_reproduces_picked_colour_exactly() {
        for rgb in [[1.0, 0.0, 0.0], [0.2, 0.7, 0.9], [0.5, 0.5, 0.5], [0.9, 0.6, 0.1]] {
            let mass = 6.0; // thick ⇒ cover ≈ 1 ⇒ backdrop washed out
            let dye = [rgb[0] * mass, rgb[1] * mass, rgb[2] * mass];
            let out = compose_linear_display([1.0, 1.0, 1.0], dye, mass, COVER_K);
            for ch in 0..3 {
                assert!((out[ch] - rgb[ch]).abs() < 0.02, "dye must reproduce {rgb:?} faithfully (got {out:?})");
            }
        }
    }

    /// Linear wet-in-wet: blue + yellow dye average to a desaturated grey/brown (metameric) — the
    /// deliberate contrast to K–M's spectral green, both read from the live field.
    #[test]
    fn dye_blue_plus_yellow_is_metameric_grey() {
        let blue = [0.05, 0.05, 0.85];
        let yellow = [0.90, 0.80, 0.05];
        let m = 1.0;
        // Overlap = premultiplied sum (what the field accumulates), un-premultiplied at composite.
        let dye = [blue[0] * m + yellow[0] * m, blue[1] * m + yellow[1] * m, blue[2] * m + yellow[2] * m];
        let out = compose_linear_display([1.0, 1.0, 1.0], dye, 2.0 * m, COVER_K);
        eprintln!("Linear blue+yellow = {out:?}");
        // NOT green-dominant (the whole point of Linear vs K–M): green must not stand out from grey.
        let avg = (out[0] + out[1] + out[2]) / 3.0;
        assert!(out[1] < avg + 0.08, "Linear mix must be metameric (grey), not green: {out:?}");
    }

    /// K–M hue is MASS-INDEPENDENT (ADR-0089 §2.2): the same ratio at a thin vs a thick total composes
    /// to the same hue direction (only coverage/lightness differs) — the magnitude→hue drift is gone.
    #[test]
    fn km_hue_is_mass_independent() {
        let km = KmModel::new();
        let c = km.rgb_to_concentrations([1.0, 0.0, 0.0]); // red ratio, Σ = K_REF
        let thin = km.compose_km_display([1.0, 1.0, 1.0], scale(c, 0.25), COVER_K);
        let thick = km.compose_km_display([1.0, 1.0, 1.0], scale(c, 5.0), COVER_K);
        eprintln!("K–M red thin={thin:?} thick={thick:?}");
        // Coverage must rise with mass (thick is further from the white backdrop).
        let dist = |x: [f32; 3]| (1.0 - x[0]).abs() + (1.0 - x[1]).abs() + (1.0 - x[2]).abs();
        assert!(dist(thick) > dist(thin), "thicker paint must cover more");
        // The HUE (direction of backdrop→colour) must match: normalise (white − out) and compare.
        let dir = |x: [f32; 3]| {
            let v = [1.0 - x[0], 1.0 - x[1], 1.0 - x[2]];
            let n = (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt().max(1.0e-6);
            [v[0] / n, v[1] / n, v[2] / n]
        };
        let (a, b) = (dir(thin), dir(thick));
        let dot = a[0] * b[0] + a[1] * b[1] + a[2] * b[2];
        assert!(dot > 0.995, "K–M hue must be mass-independent (thin·thick dir = {dot:.4})");
    }

    // ── ADR-0091 — Mixbox residual: a single picked colour is FAITHFUL, mixing stays spectral ──────

    /// The bug Enio caught: in Pigment mode red/orange/yellow all collapsed to orange and the two blues
    /// to one blue, because the legacy composite normalised every colour to `K_REF` (discarding value).
    /// The Mixbox decode `mix(c)+r` must reproduce EACH picked colour at full coverage ⇒ the colours
    /// stay distinct.
    #[test]
    fn pigment_mode_reproduces_picked_colour() {
        let km = KmModel::new();
        let cases: [(&str, [f32; 3]); 7] = [
            ("red", [0.85, 0.12, 0.12]),
            ("orange", [0.90, 0.45, 0.05]),
            ("yellow", [0.92, 0.85, 0.08]),
            ("light blue", [0.45, 0.70, 0.92]),
            ("dark blue", [0.08, 0.10, 0.55]),
            ("green", [0.20, 0.65, 0.22]),
            ("magenta", [0.85, 0.10, 0.70]),
        ];
        for (name, rgb) in cases {
            let (c, r) = km.pigment_residual(rgb);
            let out = km.compose_km_mixbox([1.0, 1.0, 1.0], c, r, 8.0, COVER_K);
            eprintln!("{name}: pick {rgb:?} → pigment {out:?}  (residual {r:?})");
            for ch in 0..3 {
                assert!(
                    (out[ch] - rgb[ch]).abs() < 0.04,
                    "{name}: Pigment mode must reproduce the picked colour faithfully (got {out:?}, want {rgb:?})"
                );
            }
        }
    }

    /// ...and the spectral win is preserved: mixing blue + yellow WET-ON-WET (mass-weighted average of
    /// their latents) still composes to GREEN, not the metameric grey of RGB.
    #[test]
    fn pigment_mix_blue_plus_yellow_is_green() {
        let km = KmModel::new();
        let (cb, rb) = km.pigment_residual([0.05, 0.05, 0.85]); // blue
        let (cy, ry) = km.pigment_residual([0.90, 0.80, 0.05]); // yellow
        let mut c = [0.0_f32; PIGMENTS];
        for p in 0..PIGMENTS {
            c[p] = 0.5 * (cb[p] + cy[p]);
        }
        let r = [0.5 * (rb[0] + ry[0]), 0.5 * (rb[1] + ry[1]), 0.5 * (rb[2] + ry[2])];
        let out = km.compose_km_mixbox([1.0, 1.0, 1.0], c, r, 8.0, COVER_K);
        eprintln!("pigment blue+yellow = {out:?}");
        assert!(out[1] > out[0] && out[1] > out[2], "blue+yellow must mix to GREEN (g dominant), got {out:?}");
        let avg = (out[0] + out[1] + out[2]) / 3.0;
        assert!(out[1] > avg * 1.10, "green must clearly stand out from grey (g={} avg={avg})", out[1]);
    }
}
