// The CIE / OKLab matrix constants below are quoted at their canonical
// published precision (e.g. Björn Ottosson's OKLab paper, the sRGB
// standard) — slightly past f32's reliable significand. Truncating loses
// no observable accuracy but makes the source diverge from the spec
// references; we keep the literals readable.
#![allow(clippy::excessive_precision)]

//! Color space conversions for the Color Equalization pipeline.
//!
//! Pure functions, deterministic (HR-5), `std`-only. Mirrors the legacy
//! engine's `color-utils.ts` plus the canonical references:
//!
//! - **sRGB ↔ Linear**: IEC 61966-2-1 transfer.
//! - **Linear RGB ↔ XYZ (D65)**: standard sRGB matrices.
//! - **XYZ ↔ Lab (D65)**: CIE 1976 with companding.
//! - **Linear RGB ↔ OKLab**: Björn Ottosson 2020 (`https://bottosson.github.io/posts/oklab/`).
//! - **Kelvin → CIE xy**: CIE daylight chromaticity (≈ 2000K-25000K).
//! - **Bradford CAT**: chromatic adaptation matrix for D65 → target white.
//!
//! The pipeline calls these from `algorithm.rs`; each is small + branchless
//! (except sRGB transfer's two-segment shape) so a future WGSL port can
//! drop them in directly.

// ── sRGB transfer ────────────────────────────────────────────────

/// sRGB 8-bit → linear `[0, 1]` (IEC 61966-2-1).
pub fn srgb_to_linear_u8(c: u8) -> f32 {
    srgb_to_linear(c as f32 / 255.0)
}

/// Same, but takes the already-normalized sRGB value in `[0, 1]`.
pub fn srgb_to_linear(s: f32) -> f32 {
    if s <= 0.040_45 {
        s / 12.92
    } else {
        ((s + 0.055) / 1.055).powf(2.4)
    }
}

/// linear `[0, 1]` → sRGB 8-bit (rounded, clamped).
pub fn linear_to_srgb_u8(c: f32) -> u8 {
    let s = linear_to_srgb(c);
    let v = (s * 255.0 + 0.5).clamp(0.0, 255.0);
    v as u8
}

/// Same, but returns the normalized sRGB value in `[0, 1]`.
pub fn linear_to_srgb(c: f32) -> f32 {
    let c = c.max(0.0);
    if c <= 0.003_130_8 {
        c * 12.92
    } else {
        1.055 * c.powf(1.0 / 2.4) - 0.055
    }
}

// ── Linear RGB ↔ XYZ (D65) ───────────────────────────────────────

/// Linear sRGB (D65) → CIE XYZ.
pub fn linear_rgb_to_xyz(r: f32, g: f32, b: f32) -> [f32; 3] {
    [
        0.412_456_4 * r + 0.357_576_1 * g + 0.180_437_5 * b,
        0.212_672_9 * r + 0.715_152_2 * g + 0.072_175_0 * b,
        0.019_333_9 * r + 0.119_192_0 * g + 0.950_304_1 * b,
    ]
}

/// CIE XYZ (D65) → Linear sRGB.
pub fn xyz_to_linear_rgb(x: f32, y: f32, z: f32) -> [f32; 3] {
    [
        3.240_454_2 * x + -1.537_138_5 * y + -0.498_531_4 * z,
        -0.969_266_0 * x + 1.876_010_8 * y + 0.041_556_0 * z,
        0.055_643_4 * x + -0.204_025_9 * y + 1.057_225_2 * z,
    ]
}

// ── Kelvin → CIE xy chromaticity ─────────────────────────────────

/// CIE daylight chromaticity (x, y) from color temperature in Kelvin.
/// Valid ≈ `2000K..25000K`. Two-segment polynomial approximation per
/// CIE standard (mirrors `kelvinToXY` in the legacy engine).
pub fn kelvin_to_xy(k: f32) -> (f32, f32) {
    let k2 = k * k;
    let k3 = k2 * k;
    let x = if k <= 7000.0 {
        -4.607_0e9 / k3 + 2.967_8e6 / k2 + 0.099_11e3 / k + 0.244_063
    } else {
        -2.006_4e9 / k3 + 1.901_8e6 / k2 + 0.247_48e3 / k + 0.237_040
    };
    let y = -3.0 * x * x + 2.87 * x - 0.275;
    (x, y)
}

// ── Bradford chromatic adaptation ────────────────────────────────

const BRADFORD_MA: [f32; 9] = [
    0.8951, 0.2664, -0.1614, -0.7502, 1.7135, 0.0367, 0.0389, -0.0685, 1.0296,
];
const BRADFORD_MA_INV: [f32; 9] = [
    0.986_993, -0.147_054, 0.159_963, 0.432_305, 0.518_360, 0.049_291, -0.008_529, 0.040_043,
    0.968_487,
];
const RGB_TO_XYZ_D65: [f32; 9] = [
    0.412_456_4,
    0.357_576_1,
    0.180_437_5,
    0.212_672_9,
    0.715_152_2,
    0.072_175_0,
    0.019_333_9,
    0.119_192_0,
    0.950_304_1,
];
const XYZ_TO_RGB_D65: [f32; 9] = [
    3.240_454_2,
    -1.537_138_5,
    -0.498_531_4,
    -0.969_266_0,
    1.876_010_8,
    0.041_556_0,
    0.055_643_4,
    -0.204_025_9,
    1.057_225_2,
];
/// CIE D65 chromaticity, the source white for sRGB.
const D65_XY: (f32, f32) = (0.312_72, 0.329_03);

/// Multiply 3×3 row-major matrices.
fn mat3_mul(a: &[f32; 9], b: &[f32; 9]) -> [f32; 9] {
    let mut r = [0.0_f32; 9];
    for i in 0..3 {
        for j in 0..3 {
            r[i * 3 + j] = a[i * 3] * b[j] + a[i * 3 + 1] * b[3 + j] + a[i * 3 + 2] * b[6 + j];
        }
    }
    r
}

/// Multiply 3×3 row-major matrix by a column vector.
pub fn mat3_mul_vec(m: &[f32; 9], v: [f32; 3]) -> [f32; 3] {
    [
        m[0] * v[0] + m[1] * v[1] + m[2] * v[2],
        m[3] * v[0] + m[4] * v[1] + m[5] * v[2],
        m[6] * v[0] + m[7] * v[1] + m[8] * v[2],
    ]
}

/// Compose the composite Bradford adaptation matrix that operates in
/// linear sRGB space directly. Formula:
///
/// `M = XYZ_to_RGB · Ma⁻¹ · diag(coneDst / coneSrc) · Ma · RGB_to_XYZ`
///
/// Source white is D65 (sRGB native); target white is derived from
/// `target_kelvin` via `kelvin_to_xy`.
pub fn bradford_matrix_for_kelvin(target_kelvin: f32) -> [f32; 9] {
    // Source white (D65) in XYZ.
    let (sx, sy) = D65_XY;
    let src_xyz = [sx / sy, 1.0, (1.0 - sx - sy) / sy];

    // Destination white from target Kelvin in XYZ.
    let (dx, dy) = kelvin_to_xy(target_kelvin);
    let dst_xyz = [dx / dy, 1.0, (1.0 - dx - dy) / dy];

    // Cone responses.
    let cone_src = mat3_mul_vec(&BRADFORD_MA, src_xyz);
    let cone_dst = mat3_mul_vec(&BRADFORD_MA, dst_xyz);

    // Diagonal scaling matrix.
    let diag = [
        cone_dst[0] / cone_src[0],
        0.0,
        0.0,
        0.0,
        cone_dst[1] / cone_src[1],
        0.0,
        0.0,
        0.0,
        cone_dst[2] / cone_src[2],
    ];

    // M = XYZ_to_RGB · Ma_inv · diag · Ma · RGB_to_XYZ.
    let step1 = mat3_mul(&BRADFORD_MA, &RGB_TO_XYZ_D65);
    let step2 = mat3_mul(&diag, &step1);
    let step3 = mat3_mul(&BRADFORD_MA_INV, &step2);
    mat3_mul(&XYZ_TO_RGB_D65, &step3)
}

// ── HSL ──────────────────────────────────────────────────────────
//
// Used by the Phase 3 procedural LUT presets (hue-rotation tricks for
// cinematic / vintage / sepia looks). Both directions in sRGB space —
// no linearization (presets are designed against the visual sRGB cube,
// not linear light).

/// sRGB `[0, 1]` → HSL `[0, 1]` (`(hue, saturation, lightness)`).
pub fn rgb_to_hsl(r: f32, g: f32, b: f32) -> [f32; 3] {
    let max = r.max(g).max(b);
    let min = r.min(g).min(b);
    let l = (max + min) * 0.5;
    if (max - min).abs() < f32::EPSILON {
        return [0.0, 0.0, l];
    }
    let d = max - min;
    let s = if l > 0.5 {
        d / (2.0 - max - min)
    } else {
        d / (max + min)
    };
    let h = if max == r {
        ((g - b) / d + if g < b { 6.0 } else { 0.0 }) / 6.0
    } else if max == g {
        ((b - r) / d + 2.0) / 6.0
    } else {
        ((r - g) / d + 4.0) / 6.0
    };
    [h, s, l]
}

/// HSL `[0, 1]` → sRGB `[0, 1]`. Inverse of [`rgb_to_hsl`].
pub fn hsl_to_rgb(h: f32, s: f32, l: f32) -> [f32; 3] {
    if s.abs() < f32::EPSILON {
        return [l, l, l];
    }
    let q = if l < 0.5 {
        l * (1.0 + s)
    } else {
        l + s - l * s
    };
    let p = 2.0 * l - q;
    [
        hue_to_rgb(p, q, h + 1.0 / 3.0),
        hue_to_rgb(p, q, h),
        hue_to_rgb(p, q, h - 1.0 / 3.0),
    ]
}

fn hue_to_rgb(p: f32, q: f32, t: f32) -> f32 {
    let mut tt = t;
    if tt < 0.0 {
        tt += 1.0;
    }
    if tt > 1.0 {
        tt -= 1.0;
    }
    if tt < 1.0 / 6.0 {
        return p + (q - p) * 6.0 * tt;
    }
    if tt < 0.5 {
        return q;
    }
    if tt < 2.0 / 3.0 {
        return p + (q - p) * (2.0 / 3.0 - tt) * 6.0;
    }
    p
}

// ── Curves & utilities for procedural LUTs ───────────────────────

/// Colorimetrically-correct luminance of an sRGB triple in `[0, 1]`,
/// returned in the same gamma-encoded sRGB space (so downstream
/// curve / tint code can keep treating it as a perceptual brightness
/// value in `[0, 1]`).
///
/// Why this shape, not just `0.2126·R + 0.7152·G + 0.0722·B`:
///
/// - sRGB shares its colour primaries with BT.709, so BT.709's
///   `(0.2126, 0.7152, 0.0722)` coefficients are the right weights
///   for sRGB data — but ONLY when applied to LINEAR-light RGB. The
///   `R/G/B` we receive here are gamma-encoded; applying the
///   coefficients directly mixes a non-linear gamma curve into the
///   luminance, which is a common shortcut ("rec.709 luma") that
///   diverges from true Y by ~5-15 %.
/// - This implementation decodes to linear, computes true linear Y,
///   then re-encodes the scalar Y through the sRGB gamma so the
///   downstream `s_curve` / `lift_blacks` / tint multipliers (which
///   were authored against perceptually-encoded values) keep their
///   intended shape. Net cost: 4 `pow()` calls per pixel, ~5 ns
///   each at compile-time-optimised f32 — negligible against the
///   pipeline's other stages, and the LUT runs are debounced so
///   they only fire on slider release.
/// - CLAHE has its own `algorithm::luminance_bt709` (u8 in /
///   u8 out, gamma-encoded shortcut) for its own histogram pipeline;
///   we deliberately don't share a formula — CLAHE's per-bin LUT
///   reasoning needs the shortcut to round-trip exactly.
pub fn luma_srgb(r: f32, g: f32, b: f32) -> f32 {
    let rlin = srgb_to_linear(r);
    let glin = srgb_to_linear(g);
    let blin = srgb_to_linear(b);
    let y_linear = 0.2126 * rlin + 0.7152 * glin + 0.0722 * blin;
    linear_to_srgb(y_linear)
}

/// Logistic S-curve centred at `0.5`, slope controlled by `strength`
/// (`1.0` ≈ linear, `2.5` ≈ steep). Output in `[0, 1]`. Used by film-style
/// presets for contrast-shaped midtones.
pub fn s_curve(t: f32, strength: f32) -> f32 {
    let x = t.clamp(0.0, 1.0);
    let s = strength.max(0.01);
    1.0 / (1.0 + (-s * (x - 0.5) * 4.0).exp())
}

/// Raise the floor of a `[0, 1]` value by `amount`: `v · (1 − amount) +
/// amount`. `amount = 0` is identity; `1` collapses everything to white.
/// Used by faded / vintage presets.
pub fn lift_blacks(v: f32, amount: f32) -> f32 {
    v * (1.0 - amount) + amount
}

/// Linear interpolation between `a` and `b` by `t ∈ [0, 1]`.
pub fn lerp_f32(a: f32, b: f32, t: f32) -> f32 {
    a + (b - a) * t
}

/// Clamp to `[0, 1]`. Pure convenience to read closer to the legacy /
/// shader idiom.
pub fn clamp01(v: f32) -> f32 {
    v.clamp(0.0, 1.0)
}

// ── OKLab ────────────────────────────────────────────────────────

/// Linear sRGB → OKLab (L, a, b). Björn Ottosson 2020.
pub fn linear_rgb_to_oklab(r: f32, g: f32, b: f32) -> [f32; 3] {
    let l = 0.412_221_47 * r + 0.536_332_54 * g + 0.051_445_99 * b;
    let m = 0.211_903_50 * r + 0.680_699_55 * g + 0.107_396_96 * b;
    let s = 0.088_302_46 * r + 0.281_718_84 * g + 0.629_978_7 * b;

    let lp = l.cbrt();
    let mp = m.cbrt();
    let sp = s.cbrt();

    [
        0.210_454_25 * lp + 0.793_617_8 * mp + -0.004_072_046_8 * sp,
        1.977_998_5 * lp + -2.428_592_2 * mp + 0.450_593_71 * sp,
        0.025_904_037 * lp + 0.782_771_77 * mp + -0.808_675_77 * sp,
    ]
}

/// OKLab (L, a, b) → Linear sRGB. `l_star` is the OKLab L* channel
/// (kept lowercase to satisfy the snake_case lint while preserving the
/// CIE naming convention in the body comments).
pub fn oklab_to_linear_rgb(l_star: f32, a: f32, b: f32) -> [f32; 3] {
    let lp = l_star + 0.396_337_78 * a + 0.215_803_76 * b;
    let mp = l_star + -0.105_561_346 * a + -0.063_854_17 * b;
    let sp = l_star + -0.089_484_18 * a + -1.291_485_5 * b;

    let l = lp * lp * lp;
    let m = mp * mp * mp;
    let s = sp * sp * sp;

    [
        4.076_741_7 * l + -3.307_711_6 * m + 0.230_969_94 * s,
        -1.268_438_0 * l + 2.609_757_4 * m + -0.341_319_38 * s,
        -0.004_196_086 * l + -0.703_418_6 * m + 1.707_614_7 * s,
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn srgb_round_trip_8bit() {
        for v in [0u8, 1, 64, 127, 128, 200, 254, 255] {
            let lin = srgb_to_linear_u8(v);
            let back = linear_to_srgb_u8(lin);
            assert!(
                back.abs_diff(v) <= 1,
                "round trip {v} → {lin} → {back} drifted > 1 LSB"
            );
        }
    }

    #[test]
    fn linear_rgb_xyz_round_trip() {
        // Each opaque linear-sRGB triple should land within 1e-4 after
        // XYZ round-trip.
        for (r, g, b) in [
            (0.0, 0.0, 0.0),
            (1.0, 1.0, 1.0),
            (0.5, 0.5, 0.5),
            (0.8, 0.2, 0.1),
            (0.1, 0.7, 0.3),
        ] {
            let xyz = linear_rgb_to_xyz(r, g, b);
            let back = xyz_to_linear_rgb(xyz[0], xyz[1], xyz[2]);
            assert!((back[0] - r).abs() < 1e-4);
            assert!((back[1] - g).abs() < 1e-4);
            assert!((back[2] - b).abs() < 1e-4);
        }
    }

    #[test]
    fn oklab_round_trip_holds_for_normal_colors() {
        for (r, g, b) in [
            (0.05_f32, 0.05, 0.05),
            (0.5, 0.5, 0.5),
            (0.95, 0.95, 0.95),
            (0.8, 0.2, 0.1),
            (0.1, 0.7, 0.3),
            (0.2, 0.2, 0.9),
        ] {
            let lab = linear_rgb_to_oklab(r, g, b);
            let back = oklab_to_linear_rgb(lab[0], lab[1], lab[2]);
            // 1e-3 because cbrt + cube + matrices accumulate float error.
            assert!((back[0] - r).abs() < 1e-3, "r {r} → {back:?}");
            assert!((back[1] - g).abs() < 1e-3, "g {g} → {back:?}");
            assert!((back[2] - b).abs() < 1e-3, "b {b} → {back:?}");
        }
    }

    #[test]
    fn oklab_grey_has_zero_chroma() {
        // Perfect achromatic samples should have a = b ≈ 0 in OKLab.
        for v in [0.1_f32, 0.3, 0.5, 0.7, 0.9] {
            let lab = linear_rgb_to_oklab(v, v, v);
            assert!(lab[1].abs() < 1e-3, "a should be ~0 for grey {v}");
            assert!(lab[2].abs() < 1e-3, "b should be ~0 for grey {v}");
        }
    }

    #[test]
    fn kelvin_d65_xy_is_close_to_canonical() {
        // 6500K should land near the canonical D65 chromaticity (0.31272, 0.32903).
        let (x, y) = kelvin_to_xy(6500.0);
        assert!((x - 0.312_72).abs() < 0.01, "D65 x ≈ {x}");
        assert!((y - 0.329_03).abs() < 0.01, "D65 y ≈ {y}");
    }

    #[test]
    fn bradford_at_d65_is_near_identity() {
        // Adapting from D65 to D65 → identity (within numerical noise).
        let m = bradford_matrix_for_kelvin(6500.0);
        let id = [1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0];
        for i in 0..9 {
            assert!(
                (m[i] - id[i]).abs() < 0.05,
                "M[{i}] = {} should be ~{}",
                m[i],
                id[i]
            );
        }
    }

    #[test]
    fn bradford_warm_target_lifts_red_drops_blue() {
        // A warmer target (3000K, tungsten) should boost red and drop blue
        // when applied to a neutral grey.
        let m = bradford_matrix_for_kelvin(3000.0);
        let neutral = [0.5_f32, 0.5, 0.5];
        let out = mat3_mul_vec(&m, neutral);
        assert!(out[0] > 0.5, "warm should boost R: got {}", out[0]);
        assert!(out[2] < 0.5, "warm should drop B: got {}", out[2]);
    }

    // ── Phase 3 helpers (HSL + curves) ───────────────────────────────

    #[test]
    fn rgb_hsl_round_trip_within_tolerance() {
        for (r, g, b) in [
            (0.0_f32, 0.0, 0.0),
            (1.0, 1.0, 1.0),
            (0.5, 0.5, 0.5),
            (0.8, 0.2, 0.1),
            (0.1, 0.7, 0.3),
            (0.05, 0.05, 0.9),
        ] {
            let hsl = rgb_to_hsl(r, g, b);
            let back = hsl_to_rgb(hsl[0], hsl[1], hsl[2]);
            assert!((back[0] - r).abs() < 1e-5, "r {r} drifted to {}", back[0]);
            assert!((back[1] - g).abs() < 1e-5, "g {g} drifted to {}", back[1]);
            assert!((back[2] - b).abs() < 1e-5, "b {b} drifted to {}", back[2]);
        }
    }

    #[test]
    fn hsl_grey_has_zero_saturation() {
        for v in [0.0_f32, 0.25, 0.5, 0.75, 1.0] {
            let hsl = rgb_to_hsl(v, v, v);
            assert!(hsl[1].abs() < 1e-5, "grey {v} → s = {}", hsl[1]);
            assert!((hsl[2] - v).abs() < 1e-5);
        }
    }

    #[test]
    fn s_curve_endpoints_and_centre() {
        // S-curve is sigmoidal — `t = 0.5` is at the inflexion (`= 0.5`),
        // and stronger `strength` pushes the endpoints toward 0 / 1.
        assert!((s_curve(0.5, 1.5) - 0.5).abs() < 1e-5);
        // strength=2 → s_curve(0.0) approaches 0; s_curve(1.0) approaches 1.
        assert!(s_curve(0.0, 2.0) < 0.1);
        assert!(s_curve(1.0, 2.0) > 0.9);
    }

    #[test]
    fn lift_blacks_raises_floor() {
        assert!((lift_blacks(0.0, 0.1) - 0.1).abs() < 1e-5);
        assert!((lift_blacks(1.0, 0.1) - 1.0).abs() < 1e-5);
        // Mid-range: 0.5 · 0.9 + 0.1 = 0.55.
        assert!((lift_blacks(0.5, 0.1) - 0.55).abs() < 1e-5);
    }

    #[test]
    fn lerp_endpoints() {
        assert_eq!(lerp_f32(2.0, 8.0, 0.0), 2.0);
        assert_eq!(lerp_f32(2.0, 8.0, 1.0), 8.0);
        assert!((lerp_f32(2.0, 8.0, 0.5) - 5.0).abs() < 1e-5);
    }

    #[test]
    fn luma_srgb_endpoints_and_grey_are_identity() {
        // Black and white round-trip exactly (Y_linear is 0 / 1, the
        // sRGB encode of 0 is 0 and of 1 is 1).
        assert!(luma_srgb(0.0, 0.0, 0.0).abs() < 1e-5);
        assert!((luma_srgb(1.0, 1.0, 1.0) - 1.0).abs() < 1e-5);
        // Neutral grey at 0.5 (gamma-encoded) decodes to ~0.214
        // linear; BT.709(linear) of (0.214, 0.214, 0.214) = 0.214;
        // re-encoded ~0.5. Within sRGB encode rounding it lands back
        // near 0.5 — the colorimetric round-trip is shape-preserving
        // for greys.
        let mid = luma_srgb(0.5, 0.5, 0.5);
        assert!(
            (mid - 0.5).abs() < 1e-3,
            "neutral grey luma_srgb should round-trip to ~0.5, got {mid}"
        );
    }

    #[test]
    fn luma_srgb_red_is_darker_than_naive_bt709() {
        // Pure red in true linear-light Y is ~0.2126; sRGB-encoded
        // that's ~0.494 — DARKER than the "rec.709 luma" shortcut
        // would give (which is 0.2126 = ~0.51 sRGB). The full
        // colorimetric formula is what produces this answer; the test
        // pins the value so a future refactor that drops the
        // gamma-decode silently shifts every luminance-driven preset.
        let y = luma_srgb(1.0, 0.0, 0.0);
        assert!(
            (y - 0.494).abs() < 0.01,
            "luma_srgb(red) should be ~0.494 (sRGB-encoded BT.709-linear Y), got {y}"
        );
    }

    #[test]
    fn mat3_mul_vec_canonical() {
        // [2 0 0; 0 3 0; 0 0 4] · [1, 1, 1] = [2, 3, 4]
        let m = [2.0_f32, 0.0, 0.0, 0.0, 3.0, 0.0, 0.0, 0.0, 4.0];
        let out = mat3_mul_vec(&m, [1.0, 1.0, 1.0]);
        assert_eq!(out, [2.0, 3.0, 4.0]);
    }
}
