//! OKLab color conversion helpers for chroma segmentation, split out of the
//! former `chroma.rs` (pure mechanical move).

/// Convert one sRGB 8-bit triplet to Oklab. Reference:
/// <https://bottosson.github.io/posts/oklab/>.
///
/// Steps: sRGB byte → linear sRGB ([0,1]) → LMS (cube-root-warped)
/// → Oklab (L, a, b) ∈ approx [0,1] × [-0.4, 0.4] × [-0.4, 0.4].
///
/// The matrix constants come straight from the Oklab paper and
/// must NOT be truncated — `clippy::excessive_precision` is
/// silenced here because changing the digits drifts the colour
/// space.
#[allow(clippy::excessive_precision)]
pub(crate) fn srgb_to_oklab(r: u8, g: u8, b: u8) -> [f32; 3] {
    #[inline(always)]
    fn srgb_to_linear(c: u8) -> f32 {
        let cf = (c as f32) / 255.0;
        if cf <= 0.04045 {
            cf / 12.92
        } else {
            ((cf + 0.055) / 1.055).powf(2.4)
        }
    }

    let rl = srgb_to_linear(r);
    let gl = srgb_to_linear(g);
    let bl = srgb_to_linear(b);

    // Linear sRGB → LMS (Oklab paper, Eq. 1).
    let lm = 0.4122214708 * rl + 0.5363325363 * gl + 0.0514459929 * bl;
    let mm = 0.2119034982 * rl + 0.6806995451 * gl + 0.1073969566 * bl;
    let sm = 0.0883024619 * rl + 0.2817188376 * gl + 0.6299787005 * bl;

    let l_ = lm.cbrt();
    let m_ = mm.cbrt();
    let s_ = sm.cbrt();

    // LMS' → Oklab (Eq. 3).
    [
        0.2104542553 * l_ + 0.7936177850 * m_ - 0.0040720468 * s_,
        1.9779984951 * l_ - 2.4285922050 * m_ + 0.4505937099 * s_,
        0.0259040371 * l_ + 0.7827717662 * m_ - 0.8086757660 * s_,
    ]
}

/// Inverse of [`srgb_to_oklab`]: Oklab → sRGB 8-bit. Used by the
/// compose step's despill / foreground-decontamination pass to know
/// the detected background colour in sRGB. Matrices are the published
/// inverses from Björn Ottosson's Oklab reference (must NOT be
/// truncated). Out-of-gamut results are clamped to `[0, 255]`.
#[allow(clippy::excessive_precision)]
pub(crate) fn oklab_to_srgb8(lab: [f32; 3]) -> [u8; 3] {
    let (l, a, b) = (lab[0], lab[1], lab[2]);
    let l_ = l + 0.3963377774 * a + 0.2158037573 * b;
    let m_ = l - 0.1055613458 * a - 0.0638541728 * b;
    let s_ = l - 0.0894841775 * a - 1.2914855480 * b;
    let lc = l_ * l_ * l_;
    let mc = m_ * m_ * m_;
    let sc = s_ * s_ * s_;
    let rl = 4.0767416621 * lc - 3.3077115913 * mc + 0.2309699292 * sc;
    let gl = -1.2684380046 * lc + 2.6097574011 * mc - 0.3413193965 * sc;
    let bl = -0.0041960863 * lc - 0.7034186147 * mc + 1.7076147010 * sc;

    #[inline(always)]
    fn linear_to_srgb8(c: f32) -> u8 {
        let c = c.clamp(0.0, 1.0);
        let s = if c <= 0.0031308 {
            c * 12.92
        } else {
            1.055 * c.powf(1.0 / 2.4) - 0.055
        };
        (s * 255.0 + 0.5).clamp(0.0, 255.0) as u8
    }
    [
        linear_to_srgb8(rl),
        linear_to_srgb8(gl),
        linear_to_srgb8(bl),
    ]
}

/// Squared Euclidean distance between two Oklab points.
#[inline(always)]
pub(crate) fn oklab_dist_sq(a: [f32; 3], b: [f32; 3]) -> f32 {
    let dl = a[0] - b[0];
    let da = a[1] - b[1];
    let db = a[2] - b[2];
    dl * dl + da * da + db * db
}
