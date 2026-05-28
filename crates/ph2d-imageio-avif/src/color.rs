//! Color-space classification + transfer-function math for AVIF.
//!
//! This module is **pure safe Rust** — no FFI. It maps the AVIF `nclx`
//! coded-colour triplet (colour primaries / transfer characteristics /
//! matrix coefficients, ISO/IEC 23091-2) onto the engine's
//! [`ColorProfile`] and decides whether a decoded image takes the LDR
//! 8-bit [`DecodedImage::Flat`](ph2d_imageio::DecodedImage::Flat) path
//! or the HDR scene-linear
//! [`FlatHdr`](ph2d_imageio::DecodedImage::FlatHdr) path.
//!
//! ### Determinism boundary (HR-5)
//!
//! The transfer functions below use `std` transcendentals
//! (`powf`/`exp`/`ln`), **not** the `libm`-pinned ones the
//! replay-hash domain uses. This is deliberate and correct: AVIF
//! pixel decode is *content I/O*, not simulation state. The AV1 YUV
//! decode itself (dav1d C SIMD) is already not bit-identical across
//! OS/CPU, so the decoded pixels are outside the HR-5 replay-hash
//! boundary by construction — tagging the EOTF math with `libm`
//! would buy zero determinism while adding an exact-pin gate to
//! maintain. Round-trip tests assert forward/inverse *consistency*
//! (the property that matters), not cross-OS bit-equality.

use ph2d_imageio::ColorProfile;

// `libavif-sys` exposes the nclx enums as bindgen integer consts. We
// compare against the spec values (ISO/IEC 23091-2 / CICP) directly so
// this module needs no FFI dependency and stays unit-testable.

/// CICP colour-primaries codes. The full reference set is named for
/// readability; not every value is branched on (the unmatched ones
/// fall into the sRGB/Rec.709 default), hence `dead_code` is allowed.
#[allow(dead_code)]
pub mod primaries {
    pub const BT709: u16 = 1;
    pub const UNSPECIFIED: u16 = 2;
    pub const BT601: u16 = 6;
    pub const BT2020: u16 = 9;
    /// SMPTE EG 432-1 (Display P3 primaries).
    pub const SMPTE432: u16 = 12;
}

/// CICP transfer-characteristics codes (see [`primaries`] re: `dead_code`).
#[allow(dead_code)]
pub mod transfer {
    pub const BT709: u16 = 1;
    pub const UNSPECIFIED: u16 = 2;
    pub const LINEAR: u16 = 8;
    pub const SRGB: u16 = 13;
    pub const BT2020_10BIT: u16 = 14;
    pub const BT2020_12BIT: u16 = 15;
    /// SMPTE ST 2084 (Perceptual Quantizer / HDR10).
    pub const SMPTE2084_PQ: u16 = 16;
    /// ARIB STD-B67 (Hybrid Log-Gamma).
    pub const HLG: u16 = 18;
}

/// Which inverse transfer function linearises the decoded signal.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TransferFn {
    /// SMPTE ST 2084 (PQ). Scene-linear normalised so 100 cd/m²
    /// (SDR reference white) = `1.0`; PQ peak (10000 cd/m²) = `100.0`.
    Pq,
    /// ARIB STD-B67 (HLG), scene-referred inverse-OETF.
    Hlg,
    /// Scene-linear already — passthrough.
    Linear,
    /// sRGB / BT.709-shaped gamma (used for high-bit-depth SDR or
    /// wide-gamut-SDR content promoted to the float path).
    Gamma,
}

/// Decode plan derived from the `nclx` triplet + bit depth.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DecodePlan {
    /// 8-bit `DecodedImage::Flat` with this (already-encoded) profile.
    Ldr8 { profile: ColorProfile },
    /// Scene-linear `DecodedImage::FlatHdr`: decode to high-bit-depth
    /// RGBA, linearise via `transfer`, tag with `profile`.
    HdrLinear {
        profile: ColorProfile,
        transfer: TransferFn,
    },
}

/// Decide the decode path from the AVIF image's coded colour info.
///
/// HDR/float path is taken when the content cannot be faithfully
/// represented as 8-bit sRGB: a PQ/HLG/linear transfer, a bit depth
/// above 8, or BT.2020 primaries (whose gamut sRGB can't hold). LDR
/// content keeps its encoded bytes and maps primaries → profile.
#[must_use]
pub fn classify(depth: u32, color_primaries: u16, transfer_characteristics: u16) -> DecodePlan {
    use transfer as t;

    let is_hdr_transfer = matches!(
        transfer_characteristics,
        t::SMPTE2084_PQ | t::HLG | t::LINEAR | t::BT2020_10BIT | t::BT2020_12BIT
    );
    let wide_gamut = color_primaries == primaries::BT2020;
    let high_depth = depth > 8;

    if is_hdr_transfer || wide_gamut || high_depth {
        let profile = if wide_gamut {
            ColorProfile::LinearRec2020
        } else {
            ColorProfile::LinearRec709
        };
        let transfer = match transfer_characteristics {
            t::SMPTE2084_PQ => TransferFn::Pq,
            t::HLG => TransferFn::Hlg,
            t::LINEAR => TransferFn::Linear,
            // BT2020_10/12-bit + BT709 + sRGB + unspecified are all
            // gamma-shaped; linearise with the sRGB/BT.709 inverse.
            _ => TransferFn::Gamma,
        };
        DecodePlan::HdrLinear { profile, transfer }
    } else {
        let profile = match color_primaries {
            primaries::SMPTE432 => ColorProfile::DisplayP3,
            // BT709 / BT601 / unspecified / unknown → sRGB (the W1
            // default; BT.601 primaries are close enough for 8-bit
            // SDR that a dedicated profile isn't warranted).
            _ => ColorProfile::Srgb,
        };
        DecodePlan::Ldr8 { profile }
    }
}

/// Map a linear [`ColorProfile`] back to the AVIF nclx triplet for
/// **encode**. Returns `(primaries, transfer, matrix)` CICP codes.
/// Matrix is BT.2020-NCL for Rec2020, BT.709 otherwise (identity is
/// reserved for the lossless RGB path which we don't emit here).
#[must_use]
pub fn nclx_for_encode(profile: &ColorProfile, transfer: TransferFn) -> (u16, u16, u16) {
    let prim = match profile {
        ColorProfile::LinearRec2020 => primaries::BT2020,
        ColorProfile::DisplayP3 => primaries::SMPTE432,
        _ => primaries::BT709,
    };
    let tc = match transfer {
        TransferFn::Pq => transfer::SMPTE2084_PQ,
        TransferFn::Hlg => transfer::HLG,
        TransferFn::Linear => transfer::LINEAR,
        TransferFn::Gamma => transfer::SRGB,
    };
    let matrix = if prim == primaries::BT2020 {
        9 // BT2020-NCL
    } else {
        1 // BT709
    };
    (prim, tc, matrix)
}

// ── Transfer functions ──────────────────────────────────────────────
//
// PQ constants per SMPTE ST 2084. HLG constants per ARIB STD-B67 /
// Rec. ITU-R BT.2100.

const PQ_M1: f32 = 2610.0 / 16384.0;
const PQ_M2: f32 = 2523.0 / 4096.0 * 128.0;
const PQ_C1: f32 = 3424.0 / 4096.0;
const PQ_C2: f32 = 2413.0 / 4096.0 * 32.0;
const PQ_C3: f32 = 2392.0 / 4096.0 * 32.0;
/// SDR reference white (cd/m²) mapped to scene-linear `1.0`.
const PQ_REF_WHITE_NITS: f32 = 100.0;
/// PQ peak luminance (cd/m²).
const PQ_PEAK_NITS: f32 = 10000.0;

const HLG_A: f32 = 0.178_832_77;
const HLG_B: f32 = 0.284_668_92; // 1 - 4a
const HLG_C: f32 = 0.559_910_7; // 0.5 - a·ln(4a)

/// Linearise one encoded signal sample in `[0, 1]` → scene-linear.
#[must_use]
pub fn eotf(signal: f32, kind: TransferFn) -> f32 {
    let s = signal.clamp(0.0, 1.0);
    match kind {
        TransferFn::Linear => s,
        TransferFn::Gamma => srgb_to_linear(s),
        TransferFn::Pq => {
            let np = s.powf(1.0 / PQ_M2);
            let num = (np - PQ_C1).max(0.0);
            let den = PQ_C2 - PQ_C3 * np;
            // den > 0 for all s in [0,1]; guard against fp edge anyway.
            if den <= 0.0 {
                return 0.0;
            }
            let y = (num / den).powf(1.0 / PQ_M1); // [0,1] of peak
            y * (PQ_PEAK_NITS / PQ_REF_WHITE_NITS)
        }
        TransferFn::Hlg => {
            if s <= 0.5 {
                (s * s) / 3.0
            } else {
                (((s - HLG_C) / HLG_A).exp() + HLG_B) / 12.0
            }
        }
    }
}

/// Inverse of [`eotf`]: scene-linear → encoded signal in `[0, 1]`.
#[must_use]
pub fn oetf(linear: f32, kind: TransferFn) -> f32 {
    let l = linear.max(0.0);
    let signal = match kind {
        TransferFn::Linear => l,
        TransferFn::Gamma => linear_to_srgb(l),
        TransferFn::Pq => {
            let y = (l * (PQ_REF_WHITE_NITS / PQ_PEAK_NITS)).clamp(0.0, 1.0);
            let ym = y.powf(PQ_M1);
            ((PQ_C1 + PQ_C2 * ym) / (1.0 + PQ_C3 * ym)).powf(PQ_M2)
        }
        TransferFn::Hlg => {
            if l <= 1.0 / 12.0 {
                (3.0 * l).sqrt()
            } else {
                HLG_A * (12.0 * l - HLG_B).ln() + HLG_C
            }
        }
    };
    signal.clamp(0.0, 1.0)
}

/// sRGB EOTF (encoded → linear), IEC 61966-2-1.
#[must_use]
pub fn srgb_to_linear(c: f32) -> f32 {
    if c <= 0.040_45 {
        c / 12.92
    } else {
        ((c + 0.055) / 1.055).powf(2.4)
    }
}

/// sRGB OETF (linear → encoded).
#[must_use]
pub fn linear_to_srgb(l: f32) -> f32 {
    if l <= 0.003_130_8 {
        12.92 * l
    } else {
        1.055 * l.powf(1.0 / 2.4) - 0.055
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn classify_8bit_srgb_is_ldr() {
        let plan = classify(8, primaries::BT709, transfer::SRGB);
        assert_eq!(
            plan,
            DecodePlan::Ldr8 {
                profile: ColorProfile::Srgb
            }
        );
    }

    #[test]
    fn classify_8bit_p3_is_ldr_displayp3() {
        let plan = classify(8, primaries::SMPTE432, transfer::SRGB);
        assert_eq!(
            plan,
            DecodePlan::Ldr8 {
                profile: ColorProfile::DisplayP3
            }
        );
    }

    #[test]
    fn classify_pq_is_hdr_rec2020() {
        // HDR10: PQ transfer + BT2020 primaries + 10-bit.
        let plan = classify(10, primaries::BT2020, transfer::SMPTE2084_PQ);
        assert_eq!(
            plan,
            DecodePlan::HdrLinear {
                profile: ColorProfile::LinearRec2020,
                transfer: TransferFn::Pq,
            }
        );
    }

    #[test]
    fn classify_hlg_is_hdr() {
        let plan = classify(10, primaries::BT2020, transfer::HLG);
        assert!(matches!(
            plan,
            DecodePlan::HdrLinear {
                transfer: TransferFn::Hlg,
                ..
            }
        ));
    }

    #[test]
    fn classify_10bit_bt709_promotes_to_hdr_float() {
        // High bit depth alone forces the float path even with an
        // SDR-shaped transfer — 10-bit can't round-trip through 8-bit.
        let plan = classify(10, primaries::BT709, transfer::BT709);
        assert_eq!(
            plan,
            DecodePlan::HdrLinear {
                profile: ColorProfile::LinearRec709,
                transfer: TransferFn::Gamma,
            }
        );
    }

    #[test]
    fn pq_eotf_oetf_roundtrip() {
        // Forward/inverse consistency across the scene-linear range
        // (this is what the HDR round-trip relies on, not absolute nits).
        for &l in &[0.0_f32, 0.18, 1.0, 5.0, 50.0, 100.0] {
            let signal = oetf(l, TransferFn::Pq);
            let back = eotf(signal, TransferFn::Pq);
            assert!(
                (back - l).abs() < 1e-3 * l.max(1.0),
                "PQ roundtrip l={l} -> signal={signal} -> {back}"
            );
        }
    }

    #[test]
    fn hlg_eotf_oetf_roundtrip() {
        // HLG scene-linear normalises to [0, 1] (signal 1.0 → linear
        // 1.0); values above 1.0 are not representable and clamp.
        for &l in &[0.0_f32, 0.05, 0.5, 0.9, 1.0] {
            let signal = oetf(l, TransferFn::Hlg);
            let back = eotf(signal, TransferFn::Hlg);
            assert!(
                (back - l).abs() < 1e-3 * l.max(1.0),
                "HLG roundtrip l={l} -> signal={signal} -> {back}"
            );
        }
    }

    #[test]
    fn pq_reference_white_maps_to_one() {
        // PQ signal for 100 cd/m² (scene-linear 1.0) ≈ 0.508; decoding
        // it back must land at ~1.0 scene-linear.
        let signal = oetf(1.0, TransferFn::Pq);
        assert!((eotf(signal, TransferFn::Pq) - 1.0).abs() < 1e-3);
    }

    #[test]
    fn srgb_transfer_roundtrip() {
        for &l in &[0.0_f32, 0.001, 0.5, 1.0] {
            let back = srgb_to_linear(linear_to_srgb(l));
            assert!((back - l).abs() < 1e-4, "sRGB roundtrip {l} -> {back}");
        }
    }

    #[test]
    fn gamma_eotf_matches_srgb() {
        assert_eq!(eotf(0.5, TransferFn::Gamma), srgb_to_linear(0.5));
        assert_eq!(oetf(0.5, TransferFn::Gamma), linear_to_srgb(0.5));
    }
}
