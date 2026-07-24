use super::*;
// Post-split (2026-06-04): the per-kind kernels + LUT/transfer helpers live in
// the sibling `compute` submodule; the white-box tests reach them directly.
use super::compute::*;

/// Every `AdjustmentKind` v1 variant (24) — the public canonical list the
/// "+ Adjustment" menu also iterates. A new variant added to the enum must be
/// added to `AdjustmentKind::ALL` (and `psd_export` + `display_name`), so the
/// behavioral gates below cover it.
const ALL_KINDS: &[AdjustmentKind] = &AdjustmentKind::ALL;

/// Gate `adjustment_layer_kind_params_match` (§2.10): the runtime invariant
/// — a layer built with `neutral_for(kind)` has matching kind/params for
/// every kind.
#[test]
fn adjustment_layer_kind_params_match() {
    for &kind in ALL_KINDS {
        let layer = AdjustmentLayer::new(1, "test", kind);
        assert!(
            layer.kind_params_match(),
            "kind/params mismatch for {kind:?}: params is {:?}",
            layer.params.kind()
        );
        assert_eq!(layer.params.kind(), kind);
    }
}

/// Gate `psd_mapping_is_canonical` (§2.10 + §2.8): every kind maps to a
/// PsdExport; v1 = 16 layered (1:1) + 8 baked; Tier-1 split is 5 + 7.
#[test]
fn psd_mapping_is_canonical() {
    assert_eq!(ALL_KINDS.len(), 24, "v1 ships 24 kinds");
    let layered = ALL_KINDS
        .iter()
        .filter(|k| matches!(k.psd_export(), PsdExport::Layered(_)))
        .count();
    let baked = ALL_KINDS
        .iter()
        .filter(|k| matches!(k.psd_export(), PsdExport::Baked))
        .count();
    assert_eq!(layered, 16, "16 kinds round-trip 1:1 as PSD layers");
    assert_eq!(baked, 8, "8 kinds bake on export");
    // Tier 1 (first 12) splits 5 layered / 7 baked per §2.10.
    let tier1_layered = ALL_KINDS[..12]
        .iter()
        .filter(|k| matches!(k.psd_export(), PsdExport::Layered(_)))
        .count();
    assert_eq!(tier1_layered, 5, "Tier 1: 5 layered, 7 baked");
}

#[test]
fn gpu_code_and_params_contract() {
    // The per-pixel GPU kinds map to codes 0..=11 (0..=6 scalar, 7/8 the W4
    // display-space transfer LUTs, 9/10/11 the coordinate-dependent Noise/Halftone/
    // ColorLookup); spatial kinds use `gpu_spatial_code` instead. Codes must match
    // the WGSL `ADJ_*` consts.
    assert_eq!(AdjustmentKind::HueSaturationBrightness.gpu_code(), Some(0));
    assert_eq!(AdjustmentKind::BrightnessContrast.gpu_code(), Some(1));
    assert_eq!(AdjustmentKind::Invert.gpu_code(), Some(2));
    assert_eq!(AdjustmentKind::Posterize.gpu_code(), Some(3));
    assert_eq!(AdjustmentKind::Threshold.gpu_code(), Some(4));
    assert_eq!(AdjustmentKind::Exposure.gpu_code(), Some(5));
    assert_eq!(AdjustmentKind::Vibrance.gpu_code(), Some(6));
    assert_eq!(AdjustmentKind::Curves.gpu_code(), Some(7));
    assert_eq!(AdjustmentKind::Levels.gpu_code(), Some(8));
    assert_eq!(AdjustmentKind::Noise.gpu_code(), Some(9));
    assert_eq!(AdjustmentKind::Halftone.gpu_code(), Some(10));
    assert_eq!(AdjustmentKind::ColorLookupLut.gpu_code(), Some(11));
    // GaussianBlur is spatial → no per-pixel code (uses gpu_spatial_code).
    assert_eq!(AdjustmentKind::GaussianBlur.gpu_code(), None);
    // params order matches the WGSL apply_adjustment reading.
    let hsb = AdjustmentParams::HueSaturationBrightness(HsbParams {
        h: 0.1,
        s: 0.2,
        b: 0.3,
    });
    assert_eq!(hsb.gpu_params(), [0.1, 0.2, 0.3]);
    let thr = AdjustmentParams::Threshold(ThresholdParams { threshold: 128 });
    assert!((thr.gpu_params()[0] - 128.0 / 255.0).abs() < 1e-6);
    let post = AdjustmentParams::Posterize(PosterizeParams { levels: 6 });
    assert_eq!(post.gpu_params()[0], 6.0);
    // Every GPU-coded kind's neutral params produce finite gpu_params.
    for &kind in &AdjustmentKind::ALL {
        if kind.gpu_code().is_some() {
            let p = AdjustmentParams::neutral_for(kind).gpu_params();
            assert!(
                p.iter().all(|v| v.is_finite()),
                "{kind:?} gpu_params non-finite"
            );
        }
    }
}

#[test]
fn all_kinds_have_nonempty_distinct_display_names() {
    // Every kind in the menu shows a non-empty, unique English label.
    assert_eq!(AdjustmentKind::ALL.len(), 24);
    let mut seen = std::collections::BTreeSet::new();
    for &kind in &AdjustmentKind::ALL {
        let name = kind.display_name();
        assert!(!name.is_empty(), "empty display_name for {kind:?}");
        assert!(seen.insert(name), "duplicate display_name {name:?}");
    }
}

#[test]
fn neutral_params_round_trip_via_postcard() {
    // Postcard serialize/deserialize is the persist path (HR-14); a neutral
    // layer must round-trip identically (no ID-shift in the discriminated
    // union — §2.6).
    for &kind in ALL_KINDS {
        let layer = AdjustmentLayer::new(7, "x", kind);
        let bytes = postcard::to_allocvec(&layer).expect("serialize");
        let back: AdjustmentLayer = postcard::from_bytes(&bytes).expect("deserialize");
        assert_eq!(layer, back, "postcard round-trip for {kind:?}");
    }
}

// ── T4.3 — Hue/Saturation/Brightness compute (Day-4 smoke) ────────────
//
// Invariant/property tests (neutral identity, hue rotation, desaturation,
// brightness extremes, alpha preservation). A pixel-exact golden vs a
// Photoshop reference (SSIM ≥ 0.999, plan §7) needs a real PS-exported
// fixture asset — flagged to the Coord, not faked here.

fn hsb(h: f32, s: f32, b: f32) -> AdjustmentParams {
    AdjustmentParams::HueSaturationBrightness(HsbParams { h, s, b })
}

fn apply(h: f32, s: f32, b: f32, px: [f32; 4]) -> [f32; 4] {
    let mut acc = [px];
    apply_adjustment(
        &AdjustmentKind::HueSaturationBrightness,
        &hsb(h, s, b),
        &mut acc,
    );
    acc[0]
}

#[test]
fn hsb_neutral_is_exact_identity() {
    let px = [0.3, 0.6, 0.1, 0.8];
    assert_eq!(apply(0.0, 0.0, 0.0, px), px, "neutral {{0,0,0}} is a no-op");
}

#[test]
fn hsb_preserves_alpha() {
    let out = apply(0.25, 0.5, -0.3, [0.8, 0.2, 0.2, 0.42]);
    assert_eq!(out[3], 0.42, "alpha (coverage) is never touched");
}

#[test]
fn hsb_saturation_minus_one_is_grayscale() {
    // s = -1 collapses saturation → R == G == B (achromatic).
    let out = apply(0.0, -1.0, 0.0, [1.0, 0.0, 0.0, 1.0]);
    assert!(
        (out[0] - out[1]).abs() < 1e-5 && (out[1] - out[2]).abs() < 1e-5,
        "fully desaturated red is gray: {out:?}"
    );
}

#[test]
fn hsb_brightness_extremes_clamp_to_black_and_white() {
    let black = apply(0.0, 0.0, -1.0, [1.0, 0.0, 0.0, 1.0]);
    assert!(
        black[0] < 1e-4 && black[1] < 1e-4 && black[2] < 1e-4,
        "brightness -1 → black: {black:?}"
    );
    let white = apply(0.0, 0.0, 1.0, [1.0, 0.0, 0.0, 1.0]);
    assert!(
        white[0] > 0.999 && white[1] > 0.999 && white[2] > 0.999,
        "brightness +1 → white: {white:?}"
    );
}

#[test]
fn hsb_hue_rotation_shifts_red_toward_green() {
    // +1/3 turn (120°) rotates red's OKLab chroma vector into the green
    // half-plane → green becomes the dominant channel.
    let out = apply(1.0 / 3.0, 0.0, 0.0, [1.0, 0.0, 0.0, 1.0]);
    assert!(
        out[1] > out[0] && out[1] > out[2],
        "red rotated +1/3 turn is green-dominant: {out:?}"
    );
}

#[test]
fn hsb_full_turn_hue_is_identity() {
    // A whole-turn hue rotation is a 2π chroma rotation → original color
    // (within the OKLab linear round-trip, ~1e-3).
    let px = [0.7, 0.2, 0.4, 1.0];
    let out = apply(1.0, 0.0, 0.0, px);
    for c in 0..3 {
        assert!(
            (out[c] - px[c]).abs() < 5e-3,
            "full-turn ~identity: {out:?}"
        );
    }
}

// ── T4.7 — Brightness/Contrast + generic slider plumbing ──────────────

fn apply_bc(brightness: f32, contrast: f32, px: [f32; 4]) -> [f32; 4] {
    let mut acc = [px];
    apply_adjustment(
        &AdjustmentKind::BrightnessContrast,
        &AdjustmentParams::BrightnessContrast(BrightnessContrastParams {
            brightness,
            contrast,
            legacy: false,
        }),
        &mut acc,
    );
    acc[0]
}

#[test]
fn brightness_contrast_neutral_is_identity() {
    let px = [0.3, 0.6, 0.1, 0.5];
    assert_eq!(apply_bc(0.0, 0.0, px), px);
    assert_eq!(
        apply_bc(0.4, -0.3, [0.5, 0.5, 0.5, 0.42])[3],
        0.42,
        "alpha kept"
    );
}

#[test]
fn contrast_pushes_brights_up_darks_down() {
    // +contrast pushes a bright pixel brighter and a dark pixel darker
    // (around the perceptual mid-gray pivot).
    let bright = apply_bc(0.0, 0.5, [0.8, 0.8, 0.8, 1.0]);
    let dark = apply_bc(0.0, 0.5, [0.05, 0.05, 0.05, 1.0]);
    assert!(bright[0] > 0.8, "bright got brighter: {bright:?}");
    assert!(dark[0] < 0.05, "dark got darker: {dark:?}");
}

#[test]
fn bc_brightness_extremes_clamp_to_black_and_white() {
    assert!(
        apply_bc(-1.0, 0.0, [0.7, 0.4, 0.2, 1.0])
            .iter()
            .take(3)
            .all(|&v| v < 1e-4)
    );
    assert!(
        apply_bc(1.0, 0.0, [0.7, 0.4, 0.2, 1.0])
            .iter()
            .take(3)
            .all(|&v| v > 0.999)
    );
}

// ── T4.x — per-pixel fan-out kinds (Invert/Exposure/Vibrance/Posterize/
// Threshold): invariant tests (neutral identity, extremes, alpha, gray-safe).

fn run(kind: AdjustmentKind, params: AdjustmentParams, px: [f32; 4]) -> [f32; 4] {
    let mut acc = [px];
    apply_adjustment(&kind, &params, &mut acc);
    acc[0]
}

#[test]
fn invert_is_a_display_space_negative() {
    let white = run(
        AdjustmentKind::Invert,
        AdjustmentParams::Invert(InvertParams {}),
        [1.0, 1.0, 1.0, 0.7],
    );
    assert!(
        white.iter().take(3).all(|&v| v < 1e-4),
        "white → black: {white:?}"
    );
    assert_eq!(white[3], 0.7, "alpha preserved");
    let black = run(
        AdjustmentKind::Invert,
        AdjustmentParams::Invert(InvertParams {}),
        [0.0, 0.0, 0.0, 1.0],
    );
    assert!(
        black.iter().take(3).all(|&v| v > 0.999),
        "black → white: {black:?}"
    );
}

#[test]
fn exposure_neutral_identity_and_ev_brightens() {
    let px = [0.2, 0.4, 0.1, 0.55];
    let neutral = run(
        AdjustmentKind::Exposure,
        AdjustmentParams::Exposure(ExposureParams::default()),
        px,
    );
    assert_eq!(neutral, px, "all-zero Exposure is an exact identity");
    // +1 EV doubles linear light.
    let up = run(
        AdjustmentKind::Exposure,
        AdjustmentParams::Exposure(ExposureParams {
            exposure_ev: 1.0,
            offset: 0.0,
            gamma_correction: 0.0,
        }),
        [0.25, 0.25, 0.25, 0.4],
    );
    assert!((up[0] - 0.5).abs() < 1e-5, "+1 EV: 0.25 → 0.5: {up:?}");
    assert_eq!(up[3], 0.4, "alpha preserved");
}

#[test]
fn vibrance_neutral_identity_and_gray_stays_gray() {
    let px = [0.6, 0.3, 0.15, 0.9];
    let neutral = run(
        AdjustmentKind::Vibrance,
        AdjustmentParams::Vibrance(VibranceParams::default()),
        px,
    );
    assert_eq!(neutral, px, "neutral Vibrance is an exact identity");
    // A gray pixel has zero chroma → any vibrance/saturation leaves it gray
    // (no rainbow speckle — the same gray-safety as the OKLab HSB).
    let gray = run(
        AdjustmentKind::Vibrance,
        AdjustmentParams::Vibrance(VibranceParams {
            vibrance: 1.0,
            saturation: 1.0,
        }),
        [0.5, 0.5, 0.5, 1.0],
    );
    assert!(
        (gray[0] - gray[1]).abs() < 1e-4 && (gray[1] - gray[2]).abs() < 1e-4,
        "gray stays gray under max vibrance: {gray:?}"
    );
}

#[test]
fn vibrance_full_desaturation_is_grayscale() {
    let out = run(
        AdjustmentKind::Vibrance,
        AdjustmentParams::Vibrance(VibranceParams {
            vibrance: 0.0,
            saturation: -1.0,
        }),
        [1.0, 0.0, 0.0, 1.0],
    );
    assert!(
        (out[0] - out[1]).abs() < 1e-4 && (out[1] - out[2]).abs() < 1e-4,
        "saturation -1 collapses red to gray: {out:?}"
    );
}

#[test]
fn posterize_two_levels_snaps_to_black_or_white() {
    // levels = 2 → each channel rounds to display 0 or 1.
    for &c in &[0.05_f32, 0.3, 0.7, 0.95] {
        let out = run(
            AdjustmentKind::Posterize,
            AdjustmentParams::Posterize(PosterizeParams { levels: 2 }),
            [c, c, c, 0.5],
        );
        assert!(
            out[0] < 1e-4 || out[0] > 0.999,
            "posterize-2 snaps {c} to an endpoint: {out:?}"
        );
        assert_eq!(out[3], 0.5, "alpha preserved");
    }
}

#[test]
fn threshold_splits_on_luma() {
    let bright = run(
        AdjustmentKind::Threshold,
        AdjustmentParams::Threshold(ThresholdParams { threshold: 128 }),
        [0.9, 0.9, 0.9, 0.6],
    );
    assert!(
        bright.iter().take(3).all(|&v| v > 0.999),
        "bright → white: {bright:?}"
    );
    assert_eq!(bright[3], 0.6, "alpha preserved");
    let dark = run(
        AdjustmentKind::Threshold,
        AdjustmentParams::Threshold(ThresholdParams { threshold: 128 }),
        [0.02, 0.02, 0.02, 1.0],
    );
    assert!(
        dark.iter().take(3).all(|&v| v < 1e-4),
        "dark → black: {dark:?}"
    );
}

#[test]
fn invert_lut_tracks_direct_reference() {
    // The per-call LUT (perf path) must match the exact display-space
    // transfer across the range (handoff §3: cheaper compute, same result).
    for i in 0..=64 {
        let v = i as f32 / 64.0;
        let direct = srgb_to_linear_f32(1.0 - linear_to_srgb_f32(v));
        let out = run(
            AdjustmentKind::Invert,
            AdjustmentParams::Invert(InvertParams {}),
            [v, v, v, 1.0],
        )[0];
        assert!(
            (out - direct).abs() < 2e-3,
            "invert LUT vs direct at {v}: {out} vs {direct}"
        );
    }
}

#[test]
fn fanout_slider_round_trips() {
    // Each new slider kind: set then read returns the same 0..1 (the panel
    // relies on this for stable thumb positions). Quantized kinds (Posterize/
    // Threshold) round-trip on the canonical step grid.
    let mut exp = AdjustmentParams::Exposure(ExposureParams::default());
    for (slot, want) in [(0usize, 0.2_f32), (1, 0.8), (2, 0.35)] {
        set_adjustment_slider_param(&mut exp, slot, want);
    }
    let read = adjustment_slider_params(&exp);
    for (got, want) in read.iter().map(|r| r.1).zip([0.2, 0.8, 0.35]) {
        assert!(
            (got - want).abs() < 1e-6,
            "exposure round-trip {got} vs {want}"
        );
    }
    let mut vib = AdjustmentParams::Vibrance(VibranceParams::default());
    set_adjustment_slider_param(&mut vib, 0, 0.9);
    set_adjustment_slider_param(&mut vib, 1, 0.1);
    let vr = adjustment_slider_params(&vib);
    assert!((vr[0].1 - 0.9).abs() < 1e-6 && (vr[1].1 - 0.1).abs() < 1e-6);
}

// ── W4 bespoke — Levels (generic-slider path) ─────────────────────────

fn levels(
    black_point: f32,
    gamma: f32,
    white_point: f32,
    output_black: f32,
    output_white: f32,
) -> AdjustmentParams {
    AdjustmentParams::Levels(LevelsParams {
        black_point,
        gamma,
        white_point,
        output_black,
        output_white,
    })
}

#[test]
fn levels_neutral_default_is_exact_identity() {
    // The manual neutral Default (NOT the degenerate all-zero derive) is a no-op.
    let d = LevelsParams::default();
    assert_eq!((d.black_point, d.gamma, d.white_point), (0.0, 1.0, 1.0));
    let px = [0.2, 0.4, 0.1, 0.55];
    let out = run(
        AdjustmentKind::Levels,
        AdjustmentParams::Levels(LevelsParams::default()),
        px,
    );
    assert_eq!(out, px, "neutral Levels is an exact identity");
}

#[test]
fn levels_gamma_above_one_brightens_midtones() {
    // A mid-gray pixel (≈ display 0.5) gets brighter under γ = 2 (PS: out = t^(1/γ)).
    let mid = srgb_to_linear_f32(0.5);
    let out = run(
        AdjustmentKind::Levels,
        levels(0.0, 2.0, 1.0, 0.0, 1.0),
        [mid, mid, mid, 1.0],
    );
    assert!(out[0] > mid + 1e-3, "γ=2 lifts the midtone: {out:?}");
    assert_eq!(out[3], 1.0, "alpha preserved");
}

#[test]
fn levels_input_points_clip_to_black_and_white() {
    // black_point=0.25 / white_point=0.75: anything ≤ 0.25 → black, ≥ 0.75 → white.
    let p = levels(0.25, 1.0, 0.75, 0.0, 1.0);
    let lo = run(
        AdjustmentKind::Levels,
        p.clone(),
        [srgb_to_linear_f32(0.15); 4],
    );
    assert!(
        lo[0] < 1e-3,
        "below the input black point clips to black: {lo:?}"
    );
    let hi = run(AdjustmentKind::Levels, p, [srgb_to_linear_f32(0.85); 4]);
    assert!(
        hi[0] > 0.999,
        "above the input white point clips to white: {hi:?}"
    );
}

#[test]
fn levels_output_range_compresses() {
    // output_black=0.2 / output_white=0.8 maps display white → display 0.8.
    let out = run(
        AdjustmentKind::Levels,
        levels(0.0, 1.0, 1.0, 0.2, 0.8),
        [1.0, 1.0, 1.0, 0.4],
    );
    let want = srgb_to_linear_f32(0.8);
    assert!(
        (out[0] - want).abs() < 5e-3,
        "white compresses toward output_white: {out:?} want {want}"
    );
    let out_lo = run(
        AdjustmentKind::Levels,
        levels(0.0, 1.0, 1.0, 0.2, 0.8),
        [0.0, 0.0, 0.0, 1.0],
    );
    let want_lo = srgb_to_linear_f32(0.2);
    assert!(
        (out_lo[0] - want_lo).abs() < 5e-3,
        "black lifts toward output_black: {out_lo:?}"
    );
}

#[test]
fn levels_slider_round_trips() {
    // The 5 generic-slider slots (incl. the log-symmetric gamma) round-trip.
    let mut p = AdjustmentParams::Levels(LevelsParams::default());
    let want = [0.1_f32, 0.7, 0.9, 0.15, 0.85];
    for (slot, &w) in want.iter().enumerate() {
        set_adjustment_slider_param(&mut p, slot, w);
    }
    let read = adjustment_slider_params(&p);
    assert_eq!(read.len(), 5, "Levels exposes 5 generic sliders");
    for (got, w) in read.iter().map(|r| r.1).zip(want) {
        assert!(
            (got - w).abs() < 1e-5,
            "levels slider round-trip {got} vs {w}"
        );
    }
    // Slider 0.5 → neutral γ = 1.
    let mut q = AdjustmentParams::Levels(LevelsParams::default());
    set_adjustment_slider_param(&mut q, 1, 0.5);
    if let AdjustmentParams::Levels(lp) = &q {
        assert!((lp.gamma - 1.0).abs() < 1e-4, "slider 0.5 is neutral γ");
    }
}

#[test]
fn levels_display_lut_neutral_is_identity_ramp() {
    // The exported GPU LUT for a neutral Levels is the display identity ramp.
    let lut = levels_display_lut(&LevelsParams::default());
    for (i, &v) in lut.iter().enumerate() {
        let want = i as f32 / (DISPLAY_LUT_N - 1) as f32;
        assert!((v - want).abs() < 1e-6, "neutral ramp at {i}");
    }
}

#[test]
fn levels_apply_tracks_exported_lut() {
    // apply_levels samples the SAME table the GPU binds (curves/levels parity is
    // "do CPU and GPU read the same LUT"). Replicate the in-kernel sample here.
    let p = LevelsParams {
        black_point: 0.1,
        gamma: 1.6,
        white_point: 0.9,
        output_black: 0.05,
        output_white: 0.95,
    };
    let lut = levels_display_lut(&p);
    for &disp in &[0.0_f32, 0.2, 0.37, 0.5, 0.83, 1.0] {
        let lin = srgb_to_linear_f32(disp);
        let mut acc = [[lin, lin, lin, 1.0]];
        apply_levels(&p, &mut acc);
        let s = linear_to_srgb_f32(lin);
        let t = s.clamp(0.0, 1.0) * (DISPLAY_LUT_N - 1) as f32;
        let idx = t as usize;
        let frac = t - idx as f32;
        let a = lut[idx];
        let b = lut[(idx + 1).min(DISPLAY_LUT_N - 1)];
        let expected = srgb_to_linear_f32(a + (b - a) * frac);
        assert!(
            (acc[0][0] - expected).abs() < 1e-6,
            "apply_levels uses the exported LUT at {disp}: {} vs {expected}",
            acc[0][0]
        );
    }
}

// ── W4 bespoke — Curves (per-channel display-space tone curve) ─────────

fn curves(rgb: Vec<[f32; 2]>) -> AdjustmentParams {
    AdjustmentParams::Curves(CurvesParams {
        points_rgb: ControlPoints { points: rgb },
        points_r: ControlPoints::default(),
        points_g: ControlPoints::default(),
        points_b: ControlPoints::default(),
    })
}

#[test]
fn curves_empty_is_exact_identity() {
    // The neutral default (all-empty point lists) is a bit-exact no-op.
    let px = [0.3, 0.6, 0.1, 0.8];
    let out = run(
        AdjustmentKind::Curves,
        AdjustmentParams::Curves(CurvesParams::default()),
        px,
    );
    assert_eq!(out, px, "empty curves are an exact identity");
}

#[test]
fn curves_diagonal_points_track_identity() {
    // Five evenly-spaced points on the diagonal interpolate back to identity
    // (the seed a fixed-x curve editor would create — must not perturb pixels).
    let diag: Vec<[f32; 2]> = (0..5).map(|i| [i as f32 / 4.0, i as f32 / 4.0]).collect();
    let p = curves(diag);
    for i in 0..=32 {
        let lin = srgb_to_linear_f32(i as f32 / 32.0);
        let out = run(AdjustmentKind::Curves, p.clone(), [lin, lin, lin, 1.0]);
        assert!(
            (out[0] - lin).abs() < 3e-3,
            "diagonal curve ≈ identity at display {}: {} vs {lin}",
            i as f32 / 32.0,
            out[0]
        );
    }
}

#[test]
fn curves_lifting_midtone_brightens() {
    // A control point above the diagonal at display 0.5 lifts the midtones.
    let p = curves(vec![[0.0, 0.0], [0.5, 0.7], [1.0, 1.0]]);
    let mid = srgb_to_linear_f32(0.5);
    let out = run(AdjustmentKind::Curves, p, [mid, mid, mid, 0.6]);
    assert!(out[0] > mid + 1e-2, "midtone lifted: {out:?}");
    assert_eq!(out[3], 0.6, "alpha preserved");
}

#[test]
fn curves_display_lut_is_monotone_and_bounded() {
    // A steep S-curve must stay in [0,1] and non-decreasing (the monotone spline
    // never overshoots — no out-of-gamut wiggle the naive cubic would render).
    let p = CurvesParams {
        points_rgb: ControlPoints {
            points: vec![[0.0, 0.0], [0.2, 0.02], [0.8, 0.98], [1.0, 1.0]],
        },
        ..Default::default()
    };
    let luts = curves_display_luts(&p);
    for lut in &luts {
        let mut prev = -1.0_f32;
        for (i, &v) in lut.iter().enumerate() {
            assert!((0.0..=1.0).contains(&v), "LUT entry {i} out of [0,1]: {v}");
            assert!(v >= prev - 1e-6, "LUT non-monotone at {i}: {v} < {prev}");
            prev = v;
        }
    }
}

#[test]
fn curves_per_channel_only_touches_that_channel() {
    // A red-only curve shifts R but leaves G/B alone (per-channel independence).
    let p = AdjustmentParams::Curves(CurvesParams {
        points_r: ControlPoints {
            points: vec![[0.0, 0.0], [0.5, 0.8], [1.0, 1.0]],
        },
        ..Default::default()
    });
    let v = srgb_to_linear_f32(0.5);
    let out = run(AdjustmentKind::Curves, p, [v, v, v, 1.0]);
    assert!(out[0] > v + 1e-2, "R lifted: {out:?}");
    assert!(
        (out[1] - v).abs() < 3e-3 && (out[2] - v).abs() < 3e-3,
        "G/B unchanged"
    );
}

#[test]
fn curves_apply_tracks_exported_lut() {
    // apply_curves samples the SAME per-channel tables the GPU binds.
    let p = CurvesParams {
        points_rgb: ControlPoints {
            points: vec![[0.0, 0.05], [0.5, 0.4], [1.0, 0.95]],
        },
        ..Default::default()
    };
    let luts = curves_display_luts(&p);
    for &disp in &[0.0_f32, 0.27, 0.5, 0.61, 1.0] {
        let lin = srgb_to_linear_f32(disp);
        let mut acc = [[lin, lin, lin, 1.0]];
        apply_curves(&p, &mut acc);
        let s = linear_to_srgb_f32(lin);
        let t = s.clamp(0.0, 1.0) * (DISPLAY_LUT_N - 1) as f32;
        let idx = t as usize;
        let frac = t - idx as f32;
        let a = luts[0][idx];
        let b = luts[0][(idx + 1).min(DISPLAY_LUT_N - 1)];
        let expected = srgb_to_linear_f32(a + (b - a) * frac);
        assert!(
            (acc[0][0] - expected).abs() < 1e-6,
            "apply_curves uses the exported LUT at {disp}"
        );
    }
}

#[test]
fn slider_params_round_trip() {
    // adjustment_slider_params is the inverse of set_adjustment_slider_param.
    let mut hsb = AdjustmentParams::HueSaturationBrightness(HsbParams::default());
    set_adjustment_slider_param(&mut hsb, 0, 0.25);
    set_adjustment_slider_param(&mut hsb, 1, 0.75);
    set_adjustment_slider_param(&mut hsb, 2, 0.10);
    let read = adjustment_slider_params(&hsb);
    assert_eq!(read.len(), 3);
    for (got, want) in read.iter().map(|r| r.1).zip([0.25, 0.75, 0.10]) {
        assert!(
            (got - want).abs() < 1e-6,
            "hsb slot round-trip: {got} vs {want}"
        );
    }
    let mut bc = AdjustmentParams::BrightnessContrast(BrightnessContrastParams::default());
    set_adjustment_slider_param(&mut bc, 1, 0.9);
    let bc_read = adjustment_slider_params(&bc);
    assert_eq!(bc_read.len(), 2);
    assert!((bc_read[1].1 - 0.9).abs() < 1e-6);
}

// ── W4 BATCH-1 — Photo Filter (warm/cool gel + toggle rack) ───────────

fn photo(temperature: f32, density: f32, preserve: bool, px: [f32; 4]) -> [f32; 4] {
    run(
        AdjustmentKind::PhotoFilter,
        AdjustmentParams::PhotoFilter(PhotoFilterParams {
            temperature,
            density,
            preserve_luminosity: preserve,
        }),
        px,
    )
}

/// Rec.709 linear luma — the same weights `apply_photo_filter` renormalizes on.
fn luma(px: [f32; 4]) -> f32 {
    0.2126 * px[0] + 0.7152 * px[1] + 0.0722 * px[2]
}

#[test]
fn photo_filter_neutral_is_exact_identity() {
    let px = [0.3, 0.55, 0.2, 0.7];
    // The all-zero Default (density 0) is a no-op …
    assert_eq!(
        run(
            AdjustmentKind::PhotoFilter,
            AdjustmentParams::PhotoFilter(PhotoFilterParams::default()),
            px,
        ),
        px,
        "default Photo Filter is an exact identity",
    );
    // … and so is a zero temperature even at full density (the unit gel).
    assert_eq!(
        photo(0.0, 1.0, true, px),
        px,
        "temperature 0 is the unit gel"
    );
}

#[test]
fn photo_filter_warm_shifts_red_above_blue() {
    // A warm gel passes red and cuts blue → a neutral gray reads warmer (R > B).
    let out = photo(0.8, 0.6, false, [0.5, 0.5, 0.5, 1.0]);
    assert!(
        out[0] > out[2] + 1e-3,
        "warm filter pushes red above blue: {out:?}"
    );
    assert_eq!(out[3], 1.0, "alpha preserved");
}

#[test]
fn photo_filter_cool_shifts_blue_above_red() {
    // A cool gel passes blue and cuts red → blue dominates a neutral gray.
    let out = photo(-0.8, 0.6, false, [0.5, 0.5, 0.5, 0.4]);
    assert!(
        out[2] > out[0] + 1e-3,
        "cool filter pushes blue above red: {out:?}"
    );
    assert_eq!(out[3], 0.4, "alpha preserved");
}

#[test]
fn photo_filter_preserve_luminosity_holds_luma() {
    // With preserve-luminosity the gel shifts hue but not brightness: a gray
    // pixel's luma is unchanged (exact for an achromatic input).
    let px = [0.5, 0.5, 0.5, 1.0];
    let kept = photo(0.9, 0.8, true, px);
    assert!(
        (luma(kept) - luma(px)).abs() < 1e-4,
        "preserve-lum keeps luma: {} vs {}",
        luma(kept),
        luma(px)
    );
    // Without preservation the warm multiply darkens (blue cut, red unchanged).
    let dropped = photo(0.9, 0.8, false, px);
    assert!(
        luma(dropped) < luma(px) - 1e-3,
        "no preserve-lum darkens: {} vs {}",
        luma(dropped),
        luma(px)
    );
}

#[test]
fn photo_filter_slider_round_trips() {
    // Temperature (centered) + Density round-trip through the generic rack.
    let mut p = AdjustmentParams::PhotoFilter(PhotoFilterParams::default());
    set_adjustment_slider_param(&mut p, 0, 0.75); // temp +0.5
    set_adjustment_slider_param(&mut p, 1, 0.4); // density 0.4
    let read = adjustment_slider_params(&p);
    assert_eq!(read.len(), 2, "Photo Filter exposes 2 sliders");
    assert!((read[0].1 - 0.75).abs() < 1e-6 && (read[1].1 - 0.4).abs() < 1e-6);
}

#[test]
fn photo_filter_toggle_round_trips() {
    // The generic toggle rack: read reflects the param, set flips it.
    let mut p = AdjustmentParams::PhotoFilter(PhotoFilterParams::default());
    let read = adjustment_toggle_params(&p);
    assert_eq!(read.len(), 1, "Photo Filter exposes 1 toggle");
    assert!(!read[0].1, "default preserve-luminosity is off (neutral)");
    set_adjustment_toggle_param(&mut p, 0, true);
    assert!(
        adjustment_toggle_params(&p)[0].1,
        "set_adjustment_toggle_param flips the toggle"
    );
    // A non-toggle kind exposes no toggles + ignores the setter.
    let mut hsb = AdjustmentParams::HueSaturationBrightness(HsbParams::default());
    assert!(adjustment_toggle_params(&hsb).is_empty());
    set_adjustment_toggle_param(&mut hsb, 0, true); // no-op, must not panic
}

// ── W4 BATCH-1 — Color Balance (per-channel tonal shift + segment rack) ──

fn colorbalance(cr: f32, mg: f32, yb: f32, scope: ToneScope, preserve: bool) -> AdjustmentParams {
    AdjustmentParams::ColorBalance(ColorBalanceParams {
        cyan_red: cr,
        magenta_green: mg,
        yellow_blue: yb,
        scope,
        preserve_luminosity: preserve,
    })
}

#[test]
fn color_balance_neutral_is_exact_identity() {
    let px = [0.3, 0.55, 0.2, 0.7];
    // The all-zero Default (no shifts) is a no-op …
    assert_eq!(
        run(
            AdjustmentKind::ColorBalance,
            AdjustmentParams::ColorBalance(ColorBalanceParams::default()),
            px,
        ),
        px,
        "default Color Balance is an exact identity",
    );
    // … and so is any scope/preserve combo while the shifts are zero.
    assert_eq!(
        run(
            AdjustmentKind::ColorBalance,
            colorbalance(0.0, 0.0, 0.0, ToneScope::Highlights, true),
            px,
        ),
        px,
        "zero shifts ignore scope + preserve",
    );
}

#[test]
fn color_balance_red_bias_lifts_only_red_without_preserve() {
    // A midtone Red-Cyan bias (no preserve) is a pure per-channel shift: R rises,
    // G/B untouched.
    let mid = srgb_to_linear_f32(0.5);
    let out = run(
        AdjustmentKind::ColorBalance,
        colorbalance(1.0, 0.0, 0.0, ToneScope::Midtones, false),
        [mid, mid, mid, 1.0],
    );
    assert!(out[0] > mid + 1e-3, "red bias lifts R: {out:?}");
    assert!(
        (out[1] - mid).abs() < 3e-3 && (out[2] - mid).abs() < 3e-3,
        "G/B untouched (per-channel, no preserve): {out:?}"
    );
    assert_eq!(out[3], 1.0, "alpha preserved");
}

#[test]
fn color_balance_shadow_scope_targets_dark_tones() {
    // A Shadows red bias lifts a dark pixel's red far more than a bright pixel's
    // (the tonal-range weight falls off toward white).
    let p = colorbalance(1.0, 0.0, 0.0, ToneScope::Shadows, false);
    let dark = srgb_to_linear_f32(0.2);
    let bright = srgb_to_linear_f32(0.85);
    let od = run(
        AdjustmentKind::ColorBalance,
        p.clone(),
        [dark, dark, dark, 1.0],
    );
    let ob = run(
        AdjustmentKind::ColorBalance,
        p,
        [bright, bright, bright, 1.0],
    );
    let dark_lift = linear_to_srgb_f32(od[0]) - 0.2;
    let bright_lift = linear_to_srgb_f32(ob[0]) - 0.85;
    assert!(
        dark_lift > bright_lift + 1e-2,
        "shadows scope lifts dark red more than bright: {dark_lift} vs {bright_lift}"
    );
}

#[test]
fn color_balance_preserve_luminosity_holds_display_luma() {
    // With preserve-luminosity the shift moves color but not display brightness.
    let mid = srgb_to_linear_f32(0.5);
    let out = run(
        AdjustmentKind::ColorBalance,
        colorbalance(0.8, 0.0, -0.4, ToneScope::Midtones, true),
        [mid, mid, mid, 1.0],
    );
    let l_in = 0.299 * 0.5 + 0.587 * 0.5 + 0.114 * 0.5;
    let l_out = 0.299 * linear_to_srgb_f32(out[0])
        + 0.587 * linear_to_srgb_f32(out[1])
        + 0.114 * linear_to_srgb_f32(out[2]);
    assert!(
        (l_out - l_in).abs() < 5e-3,
        "preserve-lum keeps display luma: {l_out} vs {l_in}"
    );
}

#[test]
fn color_balance_apply_tracks_exported_lut() {
    // apply_color_balance (no preserve) samples the SAME per-channel tables the
    // GPU binds (the Curves adj_luts machinery, reused).
    let p = ColorBalanceParams {
        cyan_red: 0.6,
        magenta_green: -0.3,
        yellow_blue: 0.4,
        scope: ToneScope::Midtones,
        preserve_luminosity: false,
    };
    let luts = colorbalance_display_luts(&p);
    for &disp in &[0.0_f32, 0.25, 0.5, 0.75, 1.0] {
        let lin = srgb_to_linear_f32(disp);
        let mut acc = [[lin, lin, lin, 1.0]];
        apply_color_balance(&p, &mut acc);
        let s = linear_to_srgb_f32(lin);
        let t = s.clamp(0.0, 1.0) * (DISPLAY_LUT_N - 1) as f32;
        let idx = t as usize;
        let frac = t - idx as f32;
        let a = luts[0][idx];
        let b = luts[0][(idx + 1).min(DISPLAY_LUT_N - 1)];
        let expected = srgb_to_linear_f32(a + (b - a) * frac);
        assert!(
            (acc[0][0] - expected).abs() < 1e-6,
            "apply_color_balance uses the exported LUT at {disp}"
        );
    }
}

#[test]
fn color_balance_segment_round_trips() {
    // The generic segment rack: read reflects the scope, set selects it.
    let mut p = AdjustmentParams::ColorBalance(ColorBalanceParams::default());
    let (opts, sel) = adjustment_segment_params(&p).expect("has a segmented param");
    assert_eq!(opts.len(), 3, "Shadows / Midtones / Highlights");
    assert_eq!(sel, 1, "default scope is Midtones (index 1)");
    set_adjustment_segment_param(&mut p, 2);
    assert_eq!(
        adjustment_segment_params(&p).unwrap().1,
        2,
        "selected Highlights"
    );
    set_adjustment_segment_param(&mut p, 0);
    assert_eq!(
        adjustment_segment_params(&p).unwrap().1,
        0,
        "selected Shadows"
    );
    // A non-segmented kind returns None + ignores the setter (no panic).
    let mut hsb = AdjustmentParams::HueSaturationBrightness(HsbParams::default());
    assert!(adjustment_segment_params(&hsb).is_none());
    set_adjustment_segment_param(&mut hsb, 1);
}

#[test]
fn color_balance_slider_and_toggle_round_trip() {
    let mut p = AdjustmentParams::ColorBalance(ColorBalanceParams::default());
    for (slot, want) in [(0usize, 0.7_f32), (1, 0.2), (2, 0.9)] {
        set_adjustment_slider_param(&mut p, slot, want);
    }
    let read = adjustment_slider_params(&p);
    assert_eq!(read.len(), 3, "Color Balance exposes 3 sliders");
    for (got, want) in read.iter().map(|r| r.1).zip([0.7, 0.2, 0.9]) {
        assert!((got - want).abs() < 1e-6, "cb slider {got} vs {want}");
    }
    let tog = adjustment_toggle_params(&p);
    assert_eq!(tog.len(), 1, "Color Balance exposes 1 toggle");
    assert!(!tog[0].1, "default preserve-luminosity off");
    set_adjustment_toggle_param(&mut p, 0, true);
    assert!(adjustment_toggle_params(&p)[0].1, "toggle flips");
}

// ── W4 BATCH-1 — Channel Mixer (3×4 display matrix + tabs/mono) ────────

fn mixer(r: [f32; 4], g: [f32; 4], b: [f32; 4], mono: bool) -> AdjustmentParams {
    AdjustmentParams::ChannelMixer(ChannelMixerParams {
        red_out: r,
        green_out: g,
        blue_out: b,
        monochromatic: mono,
    })
}

#[test]
fn channel_mixer_default_is_exact_identity() {
    // The manual identity Default (NOT the degenerate all-zero derive) is a no-op.
    let d = ChannelMixerParams::default();
    assert_eq!(d.red_out, [1.0, 0.0, 0.0, 0.0]);
    let px = [0.3, 0.55, 0.2, 0.7];
    assert_eq!(
        run(
            AdjustmentKind::ChannelMixer,
            AdjustmentParams::ChannelMixer(ChannelMixerParams::default()),
            px,
        ),
        px,
        "identity matrix is an exact identity",
    );
}

#[test]
fn channel_mixer_swaps_red_and_blue() {
    // red_out reads Blue, blue_out reads Red → the R and B channels swap.
    let px = [
        srgb_to_linear_f32(0.8),
        srgb_to_linear_f32(0.4),
        srgb_to_linear_f32(0.1),
        1.0,
    ];
    let out = run(
        AdjustmentKind::ChannelMixer,
        mixer(
            [0.0, 0.0, 1.0, 0.0],
            [0.0, 1.0, 0.0, 0.0],
            [1.0, 0.0, 0.0, 0.0],
            false,
        ),
        px,
    );
    assert!(
        (linear_to_srgb_f32(out[0]) - 0.1).abs() < 3e-3,
        "out R = in B: {}",
        linear_to_srgb_f32(out[0])
    );
    assert!(
        (linear_to_srgb_f32(out[2]) - 0.8).abs() < 3e-3,
        "out B = in R: {}",
        linear_to_srgb_f32(out[2])
    );
    assert!(
        (linear_to_srgb_f32(out[1]) - 0.4).abs() < 3e-3,
        "G unchanged"
    );
    assert_eq!(out[3], 1.0, "alpha preserved");
}

#[test]
fn channel_mixer_monochrome_writes_weighted_gray() {
    // Monochrome writes the red_out mix to all three channels (a weighted B&W).
    let px = [
        srgb_to_linear_f32(0.8),
        srgb_to_linear_f32(0.4),
        srgb_to_linear_f32(0.1),
        0.5,
    ];
    let out = run(
        AdjustmentKind::ChannelMixer,
        mixer(
            [0.3, 0.5, 0.2, 0.0],
            [0.0, 1.0, 0.0, 0.0],
            [0.0, 0.0, 1.0, 0.0],
            true,
        ),
        px,
    );
    let gray_disp = 0.3 * 0.8 + 0.5 * 0.4 + 0.2 * 0.1;
    assert!(
        (out[0] - out[1]).abs() < 1e-5 && (out[1] - out[2]).abs() < 1e-5,
        "all channels equal (gray): {out:?}"
    );
    assert!(
        (linear_to_srgb_f32(out[0]) - gray_disp).abs() < 3e-3,
        "gray = weighted display sum: {} vs {gray_disp}",
        linear_to_srgb_f32(out[0])
    );
    assert_eq!(out[3], 0.5, "alpha preserved");
}

#[test]
fn channel_mixer_constant_offsets_output() {
    let v = srgb_to_linear_f32(0.5);
    let out = run(
        AdjustmentKind::ChannelMixer,
        mixer(
            [1.0, 0.0, 0.0, 0.25],
            [0.0, 1.0, 0.0, 0.0],
            [0.0, 0.0, 1.0, 0.0],
            false,
        ),
        [v, v, v, 1.0],
    );
    assert!(
        (linear_to_srgb_f32(out[0]) - 0.75).abs() < 3e-3,
        "constant lifts R to display 0.75: {}",
        linear_to_srgb_f32(out[0])
    );
    assert!(
        (linear_to_srgb_f32(out[1]) - 0.5).abs() < 3e-3,
        "G unchanged"
    );
}

#[test]
fn channel_mixer_slider_and_toggle_round_trip() {
    // The bespoke per-output-row sliders + the mono toggle (generic rack).
    let mut p = ChannelMixerParams::default();
    for (slot, want) in [(0usize, 0.6_f32), (1, 0.9), (2, 0.3), (3, 0.7)] {
        set_channel_mixer_param(&mut p, 1, slot, want); // Green output row
    }
    let read = channel_mixer_slider_params(&p, 1);
    assert_eq!(read.len(), 4, "4 weight sliders (R/G/B source + Const)");
    for (got, want) in read.iter().map(|r| r.1).zip([0.6, 0.9, 0.3, 0.7]) {
        assert!((got - want).abs() < 1e-6, "mixer slider {got} vs {want}");
    }
    // Setting the Green row must not disturb the Red row (still identity).
    assert_eq!(p.red_out, [1.0, 0.0, 0.0, 0.0], "Red row untouched");
    let mut ap = AdjustmentParams::ChannelMixer(p);
    let tog = adjustment_toggle_params(&ap);
    assert_eq!(tog.len(), 1, "Channel Mixer exposes 1 toggle (Monochrome)");
    assert!(!tog[0].1, "default mono off");
    set_adjustment_toggle_param(&mut ap, 0, true);
    assert!(adjustment_toggle_params(&ap)[0].1, "mono flips");
}

// ── W4 BATCH-1 — Black & White (6-hue mix + tint) ─────────────────────

fn bw(weights: [f32; 6], tint: Option<OklchColor>, amount: f32) -> AdjustmentParams {
    let [reds, yellows, greens, cyans, blues, magentas] = weights;
    AdjustmentParams::BlackAndWhite(BlackAndWhiteParams {
        reds,
        yellows,
        greens,
        cyans,
        blues,
        magentas,
        tint_color: tint,
        tint_amount: amount,
    })
}

#[test]
fn black_and_white_default_grayscales() {
    // The manual PS-weight Default converts a saturated color to achromatic gray.
    let d = BlackAndWhiteParams::default();
    assert_eq!((d.reds, d.blues, d.magentas), (0.4, 0.2, 0.8));
    let out = run(
        AdjustmentKind::BlackAndWhite,
        AdjustmentParams::BlackAndWhite(BlackAndWhiteParams::default()),
        [
            srgb_to_linear_f32(0.8),
            srgb_to_linear_f32(0.2),
            srgb_to_linear_f32(0.1),
            0.6,
        ],
    );
    assert!(
        (out[0] - out[1]).abs() < 1e-5 && (out[1] - out[2]).abs() < 1e-5,
        "output is achromatic gray: {out:?}"
    );
    assert_eq!(out[3], 0.6, "alpha preserved");
}

#[test]
fn black_and_white_red_weight_controls_red_brightness() {
    // A pure red maps to a gray scaled by the reds weight (no tint).
    let red = [srgb_to_linear_f32(1.0), 0.0, 0.0, 1.0];
    let hi = run(
        AdjustmentKind::BlackAndWhite,
        bw([2.0, 0.0, 0.0, 0.0, 0.0, 0.0], None, 0.0),
        red,
    );
    let lo = run(AdjustmentKind::BlackAndWhite, bw([0.0; 6], None, 0.0), red);
    assert!(
        linear_to_srgb_f32(hi[0]) > linear_to_srgb_f32(lo[0]) + 0.3,
        "higher reds weight brightens red: {} vs {}",
        linear_to_srgb_f32(hi[0]),
        linear_to_srgb_f32(lo[0])
    );
    assert!(
        linear_to_srgb_f32(lo[0]) < 1e-2,
        "reds weight 0 sends pure red to black"
    );
}

#[test]
fn black_and_white_tint_adds_chroma() {
    let tint = Some(OklchColor::opaque(0.7, 0.1, 70.0));
    let tinted = run(
        AdjustmentKind::BlackAndWhite,
        bw([0.4, 0.6, 0.4, 0.6, 0.2, 0.8], tint, 1.0),
        [srgb_to_linear_f32(0.5); 4],
    );
    let spread = (tinted[0] - tinted[1]).abs() + (tinted[0] - tinted[2]).abs();
    assert!(spread > 1e-3, "tint introduces chroma: {tinted:?}");
    let gray = run(
        AdjustmentKind::BlackAndWhite,
        bw([0.4, 0.6, 0.4, 0.6, 0.2, 0.8], None, 1.0),
        [srgb_to_linear_f32(0.5); 4],
    );
    assert!(
        (gray[0] - gray[1]).abs() < 1e-5 && (gray[1] - gray[2]).abs() < 1e-5,
        "no tint stays achromatic: {gray:?}"
    );
}

#[test]
fn black_and_white_slider_and_tint_toggle_round_trip() {
    let mut p = AdjustmentParams::BlackAndWhite(BlackAndWhiteParams::default());
    assert_eq!(
        adjustment_slider_params(&p).len(),
        6,
        "6 hue sliders, no tint sliders yet"
    );
    let tog = adjustment_toggle_params(&p);
    assert_eq!(tog.len(), 1);
    assert!(!tog[0].1, "tint off by default");
    set_adjustment_toggle_param(&mut p, 0, true);
    assert!(adjustment_toggle_params(&p)[0].1, "tint enabled");
    assert_eq!(
        adjustment_slider_params(&p).len(),
        8,
        "enabling tint reveals the Hue + amount sliders"
    );
    set_adjustment_slider_param(&mut p, 0, 0.9); // reds weight
    set_adjustment_slider_param(&mut p, 6, 0.25); // tint hue → 90°
    set_adjustment_slider_param(&mut p, 7, 0.4); // tint amount
    let read = adjustment_slider_params(&p);
    assert!((read[0].1 - 0.9).abs() < 1e-6, "reds round-trip");
    assert!((read[6].1 - 0.25).abs() < 1e-5, "tint hue round-trip");
    assert!((read[7].1 - 0.4).abs() < 1e-6, "tint amount round-trip");
    set_adjustment_toggle_param(&mut p, 0, false);
    assert_eq!(
        adjustment_slider_params(&p).len(),
        6,
        "disabling tint hides its sliders"
    );
}

// ── W4 BATCH-2 — Gradient Map (luma → gradient, duotone editor) ───────

fn gstop(offset: f32, color: [u8; 4]) -> ColorStop {
    ColorStop { offset, color }
}

fn gmap(stops: Vec<ColorStop>, interp: GradientInterp) -> AdjustmentParams {
    AdjustmentParams::GradientMap(GradientMapParams {
        stops,
        interpolation: interp,
    })
}

#[test]
fn gradient_map_default_maps_to_luma_grayscale() {
    // The black→white duotone Default remaps a color to the gray of its luma.
    let out = run(
        AdjustmentKind::GradientMap,
        AdjustmentParams::GradientMap(GradientMapParams::default()),
        [
            srgb_to_linear_f32(0.8),
            srgb_to_linear_f32(0.2),
            srgb_to_linear_f32(0.1),
            0.6,
        ],
    );
    assert!(
        (out[0] - out[1]).abs() < 1e-4 && (out[1] - out[2]).abs() < 1e-4,
        "achromatic gray: {out:?}"
    );
    assert_eq!(out[3], 0.6, "alpha preserved");
}

#[test]
fn gradient_map_duotone_maps_luma_to_endpoints() {
    let p = gmap(
        vec![gstop(0.0, [0, 0, 0, 255]), gstop(1.0, [255, 0, 0, 255])],
        GradientInterp::Linear,
    );
    let white = run(AdjustmentKind::GradientMap, p.clone(), [1.0, 1.0, 1.0, 1.0]);
    assert!(
        white[0] > 0.99 && white[1] < 1e-3 && white[2] < 1e-3,
        "white (luma 1) → red endpoint: {white:?}"
    );
    let black = run(AdjustmentKind::GradientMap, p, [0.0, 0.0, 0.0, 1.0]);
    assert!(
        black.iter().take(3).all(|&v| v < 1e-3),
        "black (luma 0) → black endpoint: {black:?}"
    );
}

#[test]
fn gradient_map_smooth_bends_the_ramp() {
    let stops = vec![gstop(0.0, [0, 0, 0, 255]), gstop(1.0, [255, 255, 255, 255])];
    let lin = run(
        AdjustmentKind::GradientMap,
        gmap(stops.clone(), GradientInterp::Linear),
        [srgb_to_linear_f32(0.25); 4],
    )[0];
    let smooth = run(
        AdjustmentKind::GradientMap,
        gmap(stops, GradientInterp::Smooth),
        [srgb_to_linear_f32(0.25); 4],
    )[0];
    assert!(
        (lin - smooth).abs() > 1e-3,
        "smoothstep bends the ramp at a midtone: {lin} vs {smooth}"
    );
}

#[test]
fn gradient_map_apply_tracks_lut() {
    let p = GradientMapParams {
        stops: vec![
            gstop(0.0, [10, 20, 30, 255]),
            gstop(1.0, [200, 100, 50, 255]),
        ],
        interpolation: GradientInterp::Linear,
    };
    let lut = gradient_map_lut(&p);
    let encode = build_lut(linear_to_srgb_f32);
    for &disp in &[0.0_f32, 0.3, 0.5, 0.9, 1.0] {
        let lin = srgb_to_linear_f32(disp);
        let luma = sample_lut(&encode, lin); // gray → every channel identical
        let t = luma.clamp(0.0, 1.0) * 255.0;
        let i = t as usize;
        let frac = t - i as f32;
        let a = lut[i.min(255)];
        let b = lut[(i + 1).min(255)];
        let expected = a[0] + (b[0] - a[0]) * frac;
        let mut acc = [[lin, lin, lin, 1.0]];
        apply_gradient_map(&p, &mut acc);
        assert!(
            (acc[0][0] - expected).abs() < 1e-5,
            "apply_gradient_map reads the exported LUT at {disp}"
        );
    }
}

#[test]
fn gradient_map_stop_color_and_segment_round_trip() {
    // The bespoke N-stop editor: per-stop RGB sliders + the interpolation segment.
    let mut p = AdjustmentParams::GradientMap(GradientMapParams::default());
    assert!(
        adjustment_slider_params(&p).is_empty(),
        "Gradient Map has no generic slider rack (bespoke editor)"
    );
    if let AdjustmentParams::GradientMap(g) = &mut p {
        assert_eq!(gradient_stop_color_params(g, 1).len(), 3, "3 RGB per stop");
        set_gradient_stop_color_param(g, 0, 0, 1.0); // stop 0 red → 255
        set_gradient_stop_color_param(g, 1, 2, 0.0); // stop 1 blue → 0
        assert_eq!(g.stops[0].color[0], 255, "stop-0 red set");
        assert_eq!(g.stops[1].color[2], 0, "stop-1 blue set");
        assert!(
            (gradient_stop_color_params(g, 0)[0].1 - 1.0).abs() < 1e-6,
            "RGB round-trip"
        );
    }
    let (opts, sel) = adjustment_segment_params(&p).expect("has a segmented param");
    assert_eq!(opts, vec!["Linear", "Smooth"]);
    assert_eq!(sel, 0, "default interpolation is Linear");
    set_adjustment_segment_param(&mut p, 1);
    assert_eq!(
        adjustment_segment_params(&p).unwrap().1,
        1,
        "Smooth selected"
    );
}

#[test]
fn gradient_map_add_move_remove_stops() {
    let mut g = GradientMapParams::default(); // 2 stops (black@0, white@1)
    assert_eq!(g.stops.len(), 2);
    // Add inserts at the widest gap's midpoint (offset 0.5) with the on-gradient color.
    let idx = add_gradient_stop(&mut g).expect("added");
    assert_eq!(g.stops.len(), 3);
    assert!(
        (g.stops[idx].offset - 0.5).abs() < 1e-4,
        "inserted at the gap midpoint"
    );
    // Move keeps the index stable (no reorder) + clamps to 0..1.
    move_gradient_stop(&mut g, idx, 1.5);
    assert_eq!(g.stops[idx].offset, 1.0, "offset clamps to 1");
    // Remove holds the ≥2-stop floor.
    remove_gradient_stop(&mut g, idx);
    assert_eq!(g.stops.len(), 2);
    remove_gradient_stop(&mut g, 0);
    assert_eq!(g.stops.len(), 2, "never drops below 2 stops");
    // Fill to the 16-stop cap, then add refuses.
    while add_gradient_stop(&mut g).is_some() {}
    assert_eq!(g.stops.len(), 16, "stops at the ≤16 cap");
}

// ── W4 BATCH-2 — Selective Color (9-group CMYK) ───────────────────────

#[test]
fn selective_color_neutral_is_exact_identity() {
    let px = [0.3, 0.55, 0.2, 0.7];
    assert_eq!(
        run(
            AdjustmentKind::SelectiveColor,
            AdjustmentParams::SelectiveColor(SelectiveColorParams::default()),
            px,
        ),
        px,
        "all-zero groups is an exact identity"
    );
}

#[test]
fn selective_color_reds_cyan_reduces_red() {
    let mut sc = SelectiveColorParams::default();
    sc.reds.cyan = 1.0; // full cyan on the Reds group
    let red = [
        srgb_to_linear_f32(0.9),
        srgb_to_linear_f32(0.1),
        srgb_to_linear_f32(0.1),
        1.0,
    ];
    let out = run(
        AdjustmentKind::SelectiveColor,
        AdjustmentParams::SelectiveColor(sc),
        red,
    );
    assert!(
        linear_to_srgb_f32(out[0]) < 0.9 - 1e-2,
        "Reds-cyan reduces a red pixel's red: {}",
        linear_to_srgb_f32(out[0])
    );
    assert_eq!(out[3], 1.0, "alpha preserved");
}

#[test]
fn selective_color_blacks_target_shadows_not_highlights() {
    let mut sc = SelectiveColorParams::default();
    sc.blacks.black = 1.0; // darken via the Blacks group
    let dark = run(
        AdjustmentKind::SelectiveColor,
        AdjustmentParams::SelectiveColor(sc),
        [srgb_to_linear_f32(0.15); 4],
    );
    assert!(
        linear_to_srgb_f32(dark[0]) < 0.15 - 1e-2,
        "Blacks-black darkens a shadow: {}",
        linear_to_srgb_f32(dark[0])
    );
    let bright = run(
        AdjustmentKind::SelectiveColor,
        AdjustmentParams::SelectiveColor(sc),
        [srgb_to_linear_f32(0.9); 4],
    );
    assert!(
        linear_to_srgb_f32(bright[0]) > 0.9 - 5e-2,
        "the Blacks group barely touches a highlight: {}",
        linear_to_srgb_f32(bright[0])
    );
}

#[test]
fn selective_color_slider_and_segment_round_trip() {
    let mut s = SelectiveColorParams::default();
    for (slot, want) in [(0usize, 0.75_f32), (1, 0.2), (2, 0.9), (3, 0.4)] {
        set_selective_color_param(&mut s, 3, slot, want); // Cyans group
    }
    let read = selective_color_slider_params(&s, 3);
    assert_eq!(read.len(), 4, "4 CMYK sliders per group");
    for (got, want) in read.iter().map(|r| r.1).zip([0.75, 0.2, 0.9, 0.4]) {
        assert!((got - want).abs() < 1e-6, "cmyk round-trip {got} vs {want}");
    }
    // Editing the Cyans group leaves the Reds group neutral (0.5 = zero shift).
    assert!(
        selective_color_slider_params(&s, 0)
            .iter()
            .all(|r| (r.1 - 0.5).abs() < 1e-6),
        "Reds group untouched"
    );
    let mut p = AdjustmentParams::SelectiveColor(s);
    let (opts, sel) = adjustment_segment_params(&p).expect("has a segmented param");
    assert_eq!(opts, vec!["Relative", "Absolute"]);
    assert_eq!(sel, 0, "default method is Relative");
    set_adjustment_segment_param(&mut p, 1);
    assert_eq!(
        adjustment_segment_params(&p).unwrap().1,
        1,
        "Absolute selected"
    );
    assert_eq!(SELCOLOR_BUCKETS.len(), 9, "9 color groups");
}

// ── W4 spatial mesh — kernel contract + window/coordinate kernels ──────────────

#[test]
fn gpu_spatial_code_maps_only_the_spatial_kinds() {
    use AdjustmentKind::*;
    // SPATIAL_* codes, in lock-step with `ph2d_render::layer_compositor`.
    assert_eq!(GaussianBlur.gpu_spatial_code(), Some(0));
    assert_eq!(Sharpen.gpu_spatial_code(), Some(1));
    assert_eq!(MotionBlur.gpu_spatial_code(), Some(2));
    assert_eq!(ChromaticAberration.gpu_spatial_code(), Some(3));
    assert_eq!(Bloom.gpu_spatial_code(), Some(4));
    assert_eq!(ShadowsHighlights.gpu_spatial_code(), Some(5));
    // Everything else is per-pixel or not-yet-ported — no spatial code, and the
    // two paths are mutually exclusive (a kind is never both per-pixel & spatial).
    for &k in ALL_KINDS {
        let spatial = k.gpu_spatial_code();
        let perpixel = k.gpu_code();
        assert!(
            !(spatial.is_some() && perpixel.is_some()),
            "{k:?} must not have BOTH a per-pixel and a spatial GPU code"
        );
        if !matches!(
            k,
            GaussianBlur | Sharpen | MotionBlur | ChromaticAberration | Bloom | ShadowsHighlights
        ) {
            assert_eq!(spatial, None, "{k:?} is not a ported spatial kind");
        }
    }
}

#[test]
fn spatial_params_pack_mirrors_the_kernel_contract() {
    let g = AdjustmentParams::GaussianBlur(GaussianBlurParams { radius: 4.0 });
    assert_eq!(
        g.spatial_params(),
        Some([4.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0])
    );
    let s = AdjustmentParams::Sharpen(SharpenParams {
        amount: 0.7,
        radius: 2.0,
        mask_edges: false,
    });
    assert_eq!(
        s.spatial_params(),
        Some([0.7, 2.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0])
    );
    let m = AdjustmentParams::MotionBlur(MotionBlurParams {
        distance: 12.0,
        angle: 1.5,
    });
    assert_eq!(
        m.spatial_params(),
        Some([12.0, 1.5, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0])
    );
    let c = AdjustmentParams::ChromaticAberration(ChromaticAberrationParams {
        red_shift: 3.0,
        green_shift: 0.0,
        blue_shift: -3.0,
        falloff_center: 0.5,
    });
    assert_eq!(
        c.spatial_params(),
        Some([3.0, 0.0, -3.0, 0.5, 0.0, 0.0, 0.0, 0.0])
    );
    // Bloom uses 4; Shadows/Highlights uses all 8 — the render S/H arm reads
    // params[0..8] in this exact order (compositor.rs SPATIAL_SHADOWS_HIGHLIGHTS).
    let b = AdjustmentParams::Bloom(BloomParams {
        threshold: 0.4,
        intensity: 0.8,
        radius: 20.0,
        falloff: 0.15,
    });
    assert_eq!(
        b.spatial_params(),
        Some([0.4, 0.8, 20.0, 0.15, 0.0, 0.0, 0.0, 0.0])
    );
    let sh = AdjustmentParams::ShadowsHighlights(ShadowsHighlightsParams {
        shadows_amount: 0.3,
        shadows_tonal_width: 0.5,
        shadows_radius: 30.0,
        highlights_amount: 0.2,
        highlights_tonal_width: 0.4,
        highlights_radius: 25.0,
        color_correction: 0.1,
        midtone_contrast: 0.05,
    });
    assert_eq!(
        sh.spatial_params(),
        Some([0.3, 0.5, 30.0, 0.2, 0.4, 25.0, 0.1, 0.05])
    );
    // Per-pixel kind → no spatial pack.
    assert_eq!(
        AdjustmentParams::Invert(InvertParams {}).spatial_params(),
        None
    );
}

#[test]
fn gaussian_weights_sum_to_one() {
    for r in [0.5_f32, 1.0, 3.0, 8.0, 50.0] {
        let (w, half) = gaussian_weights(r);
        assert_eq!(w.len(), half as usize + 1);
        // Full kernel = centre + 2·flanks.
        let sum = w[0] + 2.0 * w[1..].iter().sum::<f32>();
        assert!(
            (sum - 1.0).abs() < 1e-5,
            "radius {r}: kernel sum {sum} != 1"
        );
        assert!(w[0] >= w[w.len() - 1], "centre tap is the heaviest");
    }
}

fn flat_window(n: u32, px: [f32; 4]) -> Vec<[f32; 4]> {
    vec![px; (n * n) as usize]
}

#[test]
fn gaussian_blur_of_a_flat_field_is_identity() {
    // Weights sum to 1 + clamp-to-edge ⇒ a constant image is unchanged (incl. the
    // border pixels, which would otherwise leak from outside).
    let n = 8;
    let mut acc = flat_window(n, [0.5, 0.25, 0.75, 1.0]);
    apply_gaussian(
        &GaussianBlurParams { radius: 3.0 },
        &mut acc,
        AdjustWindow::full(n, n),
    );
    for p in &acc {
        assert!(
            (p[0] - 0.5).abs() < 1e-5 && (p[1] - 0.25).abs() < 1e-5 && (p[2] - 0.75).abs() < 1e-5
        );
        // Alpha is blurred now (premultiplied), but a flat OPAQUE field's coverage
        // stays ~1 (the kernel is normalised; interior pixels see all-opaque taps).
        assert!(
            (p[3] - 1.0).abs() < 1e-5,
            "interior coverage stays ~opaque: {}",
            p[3]
        );
    }
}

#[test]
fn gaussian_blur_spreads_an_impulse() {
    let n = 9;
    let mut acc = flat_window(n, [0.0, 0.0, 0.0, 1.0]);
    let centre = (4 * n + 4) as usize;
    acc[centre] = [1.0, 1.0, 1.0, 1.0];
    apply_gaussian(
        &GaussianBlurParams { radius: 2.0 },
        &mut acc,
        AdjustWindow::full(n, n),
    );
    assert!(acc[centre][0] < 1.0, "centre energy spread out");
    let neighbour = (4 * n + 5) as usize;
    assert!(acc[neighbour][0] > 0.0, "energy reached the neighbour");
    // Energy is conserved (separable, normalised, no clamping of a positive field).
    let total: f32 = acc.iter().map(|p| p[0]).sum();
    assert!((total - 1.0).abs() < 1e-3, "blur conserves energy: {total}");
}

#[test]
fn motion_and_chroma_neutral_params_are_identity() {
    let n = 6;
    let orig = flat_window(n, [0.3, 0.6, 0.9, 1.0]);
    let mut a = orig.clone();
    apply_motion_blur(
        &MotionBlurParams {
            distance: 0.0,
            angle: 0.0,
        },
        &mut a,
        AdjustWindow::full(n, n),
    );
    assert_eq!(a, orig, "zero-distance motion is a no-op");
    let mut b = orig.clone();
    apply_chromatic_aberration(
        &ChromaticAberrationParams {
            red_shift: 0.0,
            green_shift: 0.0,
            blue_shift: 0.0,
            falloff_center: 0.0,
        },
        &mut b,
        AdjustWindow::full(n, n),
    );
    assert_eq!(b, orig, "zero-shift chroma is a no-op");
}

#[test]
fn gaussian_feathers_coverage_into_transparency() {
    // An opaque dot on a fully-transparent field: after the blur, coverage (alpha)
    // must SPREAD outward — a neighbour that was transparent now has alpha > 0
    // (soft edge), not stay clipped to the original silhouette (Enio's report).
    let n = 9;
    let mut acc = flat_window(n, [0.0, 0.0, 0.0, 0.0]); // transparent
    let centre = (4 * n + 4) as usize;
    acc[centre] = [1.0, 1.0, 1.0, 1.0]; // one opaque white texel
    apply_gaussian(
        &GaussianBlurParams { radius: 2.0 },
        &mut acc,
        AdjustWindow::full(n, n),
    );
    assert!(
        acc[centre][3] < 1.0,
        "centre coverage softened: {}",
        acc[centre][3]
    );
    let neighbour = (4 * n + 5) as usize;
    assert!(
        acc[neighbour][3] > 0.0,
        "coverage feathered into the transparent neighbour: {}",
        acc[neighbour][3]
    );
}

#[test]
fn premultiplied_blur_ignores_transparent_texel_colour() {
    // The speckle root cause: transparent texels carry arbitrary stale RGB. A
    // straight-space blur/gather would spray that colour into visible pixels
    // (Enio's blue dots). Premultiplied, a transparent texel is (0,0,0,0) → it
    // contributes nothing. An opaque WHITE dot surrounded by transparent-BLUE
    // garbage must NOT pick up any blue.
    let n = 7;
    let mut acc = flat_window(n, [0.0, 0.0, 1.0, 0.0]); // transparent, but RGB=blue
    let centre = (3 * n + 3) as usize;
    acc[centre] = [1.0, 1.0, 1.0, 1.0];
    apply_gaussian(
        &GaussianBlurParams { radius: 2.0 },
        &mut acc,
        AdjustWindow::full(n, n),
    );
    // Wherever there is visible coverage, the colour is neutral (R≈G≈B) — no blue
    // cast leaked in from the transparent texels.
    for p in &acc {
        if p[3] > 1e-3 {
            assert!(
                (p[2] - p[0]).abs() < 1e-3 && (p[2] - p[1]).abs() < 1e-3,
                "visible pixel stayed neutral (no transparent-blue bleed): {p:?}"
            );
        }
    }
}

#[test]
fn chroma_does_not_speckle_transparent_regions() {
    // ChromaticAberration over transparent-blue garbage: every output pixel that
    // is (still) fully transparent must be EXACTLY transparent black — no stray
    // colour speckle survives the premultiplied gather + unpremultiply.
    let n = 8;
    let mut acc = flat_window(n, [0.0, 0.0, 1.0, 0.0]); // transparent blue garbage
    // a small opaque patch so the gather has SOMETHING to move
    for &i in &[
        (3 * n + 3) as usize,
        (3 * n + 4) as usize,
        (4 * n + 3) as usize,
    ] {
        acc[i] = [1.0, 0.2, 0.2, 1.0];
    }
    apply_chromatic_aberration(
        &ChromaticAberrationParams {
            red_shift: 4.0,
            green_shift: 0.0,
            blue_shift: -4.0,
            falloff_center: 0.0,
        },
        &mut acc,
        AdjustWindow::full(n, n),
    );
    for p in &acc {
        if p[3] <= 1e-4 {
            assert_eq!(
                *p,
                [0.0, 0.0, 0.0, 0.0],
                "a transparent output pixel must be clean transparent black, not speckle"
            );
        }
    }
}

#[test]
fn noise_is_deterministic_per_absolute_coordinate() {
    // Same content + same params ⇒ identical (deterministic). AND a sub-window
    // hashes the ABSOLUTE coordinate, so its pixels match the full window's
    // corresponding pixels — the dirty-rect invariant for a coordinate kind.
    let p = NoiseParams {
        amount: 0.5,
        kind: NoiseKind::Uniform,
        monochromatic: false,
    };
    let mut full = flat_window(4, [0.5, 0.5, 0.5, 1.0]);
    apply_noise(&p, &mut full, AdjustWindow::full(4, 4));
    let mut full2 = flat_window(4, [0.5, 0.5, 0.5, 1.0]);
    apply_noise(&p, &mut full2, AdjustWindow::full(4, 4));
    assert_eq!(full, full2, "noise is deterministic");

    // 2×2 sub-window anchored at (1,1) — pixel (lx,ly) = absolute (1+lx,1+ly).
    let mut sub = flat_window(2, [0.5, 0.5, 0.5, 1.0]);
    apply_noise(
        &p,
        &mut sub,
        AdjustWindow {
            width: 2,
            height: 2,
            origin_x: 1,
            origin_y: 1,
        },
    );
    for ly in 0..2u32 {
        for lx in 0..2u32 {
            let s = sub[(ly * 2 + lx) as usize];
            let f = full[((1 + ly) * 4 + (1 + lx)) as usize];
            assert!(
                (s[0] - f[0]).abs() < 1e-6,
                "sub-window noise matches full at absolute ({},{})",
                1 + lx,
                1 + ly
            );
        }
    }
}

#[test]
fn noise_amount_zero_is_identity_and_monochromatic_is_gray_delta() {
    let n = 4;
    let base = [0.4, 0.4, 0.4, 1.0];
    let mut acc = flat_window(n, base);
    apply_noise(
        &NoiseParams {
            amount: 0.0,
            kind: NoiseKind::Gaussian,
            monochromatic: false,
        },
        &mut acc,
        AdjustWindow::full(n, n),
    );
    assert!(acc.iter().all(|p| *p == base), "amount 0 is a no-op");

    // Monochromatic ⇒ the same delta on R/G/B (started from a gray pixel).
    let mut mono = flat_window(n, base);
    apply_noise(
        &NoiseParams {
            amount: 0.8,
            kind: NoiseKind::Uniform,
            monochromatic: true,
        },
        &mut mono,
        AdjustWindow::full(n, n),
    );
    for p in &mono {
        assert!(
            (p[0] - p[1]).abs() < 1e-6 && (p[1] - p[2]).abs() < 1e-6,
            "monochromatic noise keeps R==G==B: {p:?}"
        );
    }
}

#[test]
fn halftone_outputs_only_black_or_white_ink() {
    let n = 16;
    let mut acc = flat_window(n, [0.5, 0.5, 0.5, 0.6]);
    apply_halftone(
        &HalftoneParams {
            dot_size: 4.0,
            angle: 0.3,
            shape: HalftoneShape::Dot,
        },
        &mut acc,
        AdjustWindow::full(n, n),
    );
    let mut saw_ink = false;
    let mut saw_paper = false;
    for p in &acc {
        assert!(
            (p[0] - p[1]).abs() < 1e-6 && (p[1] - p[2]).abs() < 1e-6,
            "halftone is achromatic"
        );
        assert!(
            p[0] < 1e-3 || p[0] > 1.0 - 1e-3,
            "pixel is ink or paper: {p:?}"
        );
        assert_eq!(p[3], 0.6, "alpha preserved");
        saw_ink |= p[0] < 1e-3;
        saw_paper |= p[0] > 1.0 - 1e-3;
    }
    // A 50%-gray field screens to a mix of ink and paper.
    assert!(saw_ink && saw_paper, "midtone gray yields a dot pattern");
}

#[test]
fn halftone_extremes_are_solid() {
    let n = 8;
    let params = HalftoneParams {
        dot_size: 4.0,
        angle: 0.0,
        shape: HalftoneShape::Dot,
    };
    // Pure white ⇒ all paper (no ink).
    let mut white = flat_window(n, [1.0, 1.0, 1.0, 1.0]);
    apply_halftone(&params, &mut white, AdjustWindow::full(n, n));
    assert!(white.iter().all(|p| p[0] > 1.0 - 1e-3), "white → all paper");
}

#[test]
fn windowed_dispatch_delegates_per_pixel_kinds() {
    // A per-pixel kind through `apply_adjustment_windowed` == the flat path.
    let px = [0.8, 0.2, 0.5, 0.9];
    let kind = AdjustmentKind::Invert;
    let params = AdjustmentParams::Invert(InvertParams {});
    let mut a = [px];
    apply_adjustment_windowed(&kind, &params, &mut a, AdjustWindow::full(1, 1));
    let mut b = [px];
    apply_adjustment(&kind, &params, &mut b);
    assert_eq!(
        a, b,
        "windowed dispatch matches the flat path for per-pixel kinds"
    );
}

// ── W4 spatial/coord kinds expose editable UI (the integration gap fix) ────────
//
// Without these, a GaussianBlur layer is dead-in-product: the "+ Adjustment" menu
// can create it, but the panel renders no Radius slider, so the radius stays 0 (a
// no-op blur). These assert each live-compute kind has a slider rack that
// round-trips, plus the Noise/Halftone toggle & segment plumbing.

#[test]
fn spatial_kinds_each_expose_a_slider_rack() {
    // Every W4 spatial/coord kind with LIVE compute must expose ≥1 generic slider
    // (else it cannot be tuned from the panel — the bug Enio caught).
    let expect = [
        (AdjustmentKind::GaussianBlur, 1),
        (AdjustmentKind::MotionBlur, 2),
        (AdjustmentKind::Sharpen, 2),
        (AdjustmentKind::ChromaticAberration, 3),
        (AdjustmentKind::Noise, 1),
        (AdjustmentKind::Halftone, 2),
    ];
    for (kind, n) in expect {
        let params = AdjustmentParams::neutral_for(kind);
        let sliders = adjustment_slider_params(&params);
        assert_eq!(
            sliders.len(),
            n,
            "{kind:?} must expose {n} slider(s), got {sliders:?}"
        );
    }
}

#[test]
fn spatial_slider_round_trips_and_moves_the_param() {
    // Set every slot to 0.75, read it back (round-trip), and confirm the radius/
    // amount actually left its neutral 0 (so compute is reachable from the panel).
    for kind in [
        AdjustmentKind::GaussianBlur,
        AdjustmentKind::MotionBlur,
        AdjustmentKind::Sharpen,
        AdjustmentKind::ChromaticAberration,
        AdjustmentKind::Noise,
        AdjustmentKind::Halftone,
    ] {
        let mut params = AdjustmentParams::neutral_for(kind);
        let n = adjustment_slider_params(&params).len();
        for slot in 0..n {
            set_adjustment_slider_param(&mut params, slot, 0.75);
        }
        for (slot, got) in adjustment_slider_params(&params).iter().enumerate() {
            assert!(
                (got.1 - 0.75).abs() < 1e-6,
                "{kind:?} slot {slot} round-trip: {} vs 0.75",
                got.1
            );
        }
    }
    // GaussianBlur radius specifically: slot 0 = 0.5 → 50 px (mid of 0..100).
    let mut g = AdjustmentParams::GaussianBlur(GaussianBlurParams::default());
    set_adjustment_slider_param(&mut g, 0, 0.5);
    let AdjustmentParams::GaussianBlur(gp) = &g else {
        unreachable!()
    };
    assert!(
        (gp.radius - 50.0).abs() < 1e-3,
        "radius 0.5 → 50px: {}",
        gp.radius
    );
}

#[test]
fn chroma_slider_is_bipolar_centered() {
    // 0.5 = no shift; 1.0 = +MAX; 0.0 = −MAX.
    let mut c = AdjustmentParams::ChromaticAberration(ChromaticAberrationParams::default());
    set_adjustment_slider_param(&mut c, 0, 0.5);
    let AdjustmentParams::ChromaticAberration(cp) = &c else {
        unreachable!()
    };
    assert!(cp.red_shift.abs() < 1e-4, "0.5 centers to no shift");
    assert!(
        (adjustment_slider_params(&c)[0].1 - 0.5).abs() < 1e-6,
        "neutral reads back at 0.5"
    );
}

// ── W4 close — Bloom + Shadows/Highlights ─────────────────────────────────────

#[test]
fn bloom_neutral_is_identity_and_glow_haloes_into_transparency() {
    // intensity 0 → identity (no glow).
    let n = 9;
    let base = flat_window(n, [0.9, 0.9, 0.9, 1.0]);
    let mut acc = base.clone();
    apply_bloom(
        &BloomParams {
            threshold: 0.5,
            intensity: 0.0,
            radius: 3.0,
            falloff: 0.1,
        },
        &mut acc,
        AdjustWindow::full(n, n),
    );
    assert_eq!(acc, base, "intensity 0 is a no-op");

    // A bright opaque dot on transparency: bloom haloes coverage outward.
    let mut acc = flat_window(n, [0.0, 0.0, 0.0, 0.0]);
    let centre = (4 * n + 4) as usize;
    acc[centre] = [1.0, 1.0, 1.0, 1.0];
    apply_bloom(
        &BloomParams {
            threshold: 0.3,
            intensity: 1.5,
            radius: 3.0,
            falloff: 0.1,
        },
        &mut acc,
        AdjustWindow::full(n, n),
    );
    let neighbour = (4 * n + 6) as usize; // two px away, was transparent
    assert!(
        acc[neighbour][3] > 0.0,
        "bloom glow haloes coverage into transparency: {}",
        acc[neighbour][3]
    );
}

#[test]
fn bloom_downsample_factor_matches_the_gpu_mirror() {
    // GOLDEN: must stay bit-identical to `ph2d_render::bloom_downsample_factor`
    // (radius-independent GPU Bloom). Drift on either side diverges the CPU
    // fallback from the GPU glow — `BLOOM_MAX_LOW_RADIUS = 16`, power-of-two factor.
    use super::spatial::bloom_downsample_factor as f;
    assert_eq!(f(0.0), 1);
    assert_eq!(f(16.0), 1, "≤16 → no downsample");
    assert_eq!(f(17.0), 2, "ceil(17/16)=2 → pow2 2");
    assert_eq!(f(32.0), 2);
    assert_eq!(f(33.0), 4, "ceil(33/16)=3 → pow2 4");
    assert_eq!(f(100.0), 8, "ceil(100/16)=7 → pow2 8");
    assert_eq!(f(256.0), 16, "ceil(256/16)=16 → pow2 16");
    assert_eq!(f(1000.0), 32, "clamped to 32");
}

#[test]
fn bloom_downsampled_path_glows_on_a_large_canvas() {
    // A canvas ≥512 px picks a downsample factor > 1 (the FPS-fix path). Verify
    // the bright-pass → downsample → blur → bilinear-upsample chain still produces
    // a glow that spreads beyond a bright patch, and stays finite.
    let n = 512u32;
    let mut acc = vec![[0.0f32, 0.0, 0.0, 0.0]; (n * n) as usize];
    // A bright opaque block in the centre.
    for y in 250..262u32 {
        for x in 250..262u32 {
            acc[(y * n + x) as usize] = [1.0, 1.0, 1.0, 1.0];
        }
    }
    apply_bloom(
        &BloomParams {
            threshold: 0.3,
            intensity: 1.5,
            radius: 24.0,
            falloff: 0.1,
        },
        &mut acc,
        AdjustWindow::full(n, n),
    );
    // A pixel outside the block but within the glow radius (block edge at 261,
    // radius 24 → glow reaches ~285) now has spread coverage.
    let far = (256 * n + 272) as usize;
    assert!(
        acc[far][3] > 0.0,
        "downsampled bloom haloes out: {}",
        acc[far][3]
    );
    assert!(
        acc.iter().all(|p| p.iter().all(|v| v.is_finite())),
        "downsample/upsample stays finite"
    );
}

#[test]
fn shadows_highlights_neutral_identity_and_lifts_shadows() {
    let n = 6;
    // All-amounts-zero → identity (even with seeded widths/radii).
    let base = flat_window(n, [0.1, 0.1, 0.1, 1.0]);
    let mut acc = base.clone();
    apply_shadows_highlights(
        &ShadowsHighlightsParams::default(),
        &mut acc,
        AdjustWindow::full(n, n),
    );
    for (p, b) in acc.iter().zip(base.iter()) {
        for c in 0..3 {
            assert!(
                (p[c] - b[c]).abs() < 1e-5,
                "neutral S/H is identity: {p:?} vs {b:?}"
            );
        }
    }

    // Lifting shadows brightens a dark field; coverage preserved.
    let mut acc = flat_window(n, [0.08, 0.08, 0.08, 0.7]);
    apply_shadows_highlights(
        &ShadowsHighlightsParams {
            shadows_amount: 0.4,
            shadows_tonal_width: 0.6,
            shadows_radius: 0.0, // global tone (no blur) keeps the test deterministic
            ..ShadowsHighlightsParams::default()
        },
        &mut acc,
        AdjustWindow::full(n, n),
    );
    assert!(
        acc[0][0] > 0.08,
        "shadows lifted the dark field: {}",
        acc[0][0]
    );
    assert_eq!(acc[0][3], 0.7, "Shadows/Highlights preserves coverage");
}

#[test]
fn bloom_and_shadows_highlights_slider_round_trip() {
    for kind in [AdjustmentKind::Bloom, AdjustmentKind::ShadowsHighlights] {
        let mut params = AdjustmentParams::neutral_for(kind);
        let n = adjustment_slider_params(&params).len();
        assert!(n >= 4, "{kind:?} exposes a slider rack ({n} slots)");
        for slot in 0..n {
            set_adjustment_slider_param(&mut params, slot, 0.6);
        }
        for (slot, got) in adjustment_slider_params(&params).iter().enumerate() {
            assert!(
                (got.1 - 0.6).abs() < 1e-6,
                "{kind:?} slot {slot} round-trip: {} vs 0.6",
                got.1
            );
        }
    }
    assert_eq!(
        adjustment_slider_params(&AdjustmentParams::ShadowsHighlights(
            ShadowsHighlightsParams::default()
        ))
        .len(),
        8,
        "Shadows/Highlights fills the 8-slot rack"
    );
}

#[test]
fn parallel_kernels_are_deterministic_and_coord_correct_at_scale() {
    // A canvas above the par_rows threshold (>16384 px) runs multithreaded. Two
    // runs must be byte-identical (no data race / nondeterministic interleave),
    // and a deep-interior pixel must match the SAME absolute coord computed in a
    // tiny (serial) sub-window — proving the band-split indexing is correct.
    let n = 200u32; // 40_000 px → parallel path
    let params = NoiseParams {
        amount: 0.6,
        kind: NoiseKind::Gaussian,
        monochromatic: false,
    };
    let mk = || {
        let mut a = vec![[0.5f32, 0.5, 0.5, 1.0]; (n * n) as usize];
        apply_noise(&params, &mut a, AdjustWindow::full(n, n));
        a
    };
    let a = mk();
    let b = mk();
    assert_eq!(a, b, "parallel kernel is deterministic across runs");

    // Same absolute coord (137,151) via a 2×2 serial sub-window must match.
    let mut sub = vec![[0.5f32, 0.5, 0.5, 1.0]; 4];
    apply_noise(
        &params,
        &mut sub,
        AdjustWindow {
            width: 2,
            height: 2,
            origin_x: 137,
            origin_y: 151,
        },
    );
    let full = a[(151 * n + 137) as usize];
    for c in 0..3 {
        assert!(
            (sub[0][c] - full[c]).abs() < 1e-6,
            "parallel large-canvas pixel matches the serial sub-window at (137,151)"
        );
    }
}

#[test]
fn feathers_coverage_set_is_blur_family_plus_bloom() {
    use AdjustmentKind::*;
    for k in [
        GaussianBlur,
        Sharpen,
        MotionBlur,
        ChromaticAberration,
        Bloom,
    ] {
        assert!(k.feathers_coverage(), "{k:?} feathers coverage");
    }
    // Tonal / per-pixel kinds keep coverage — incl. ShadowsHighlights (internal
    // luma blur only) and the coordinate kinds.
    for k in [
        ShadowsHighlights,
        Noise,
        Halftone,
        HueSaturationBrightness,
        ColorLookupLut,
    ] {
        assert!(!k.feathers_coverage(), "{k:?} preserves coverage");
    }
}

// ── W4 close — Color Lookup (built-in looks) ──────────────────────────────────

#[test]
fn color_lookup_none_is_identity_but_a_look_grades() {
    // Handle 0 (None) is a pass-through at any intensity.
    let px = [0.3, 0.5, 0.7, 0.8];
    let mut acc = [px];
    apply_color_lookup(
        &ColorLookupLutParams {
            lut_3d: LutHandle(0),
            intensity: 1.0,
            profile: LutProfile::Srgb,
        },
        &mut acc,
    );
    assert_eq!(acc[0], px, "None preset is identity");

    // The "Warm" look lifts red and eases blue → R rises, B falls; alpha kept.
    let warm = LUT_PRESETS
        .iter()
        .position(|(name, _)| *name == "Warm")
        .expect("Warm preset exists") as u64;
    let mut acc = [[0.4, 0.4, 0.4, 0.5]];
    apply_color_lookup(
        &ColorLookupLutParams {
            lut_3d: LutHandle(warm),
            intensity: 1.0,
            profile: LutProfile::Srgb,
        },
        &mut acc,
    );
    assert!(acc[0][0] > 0.4, "warm lifts red: {}", acc[0][0]);
    assert!(acc[0][2] < 0.4, "warm eases blue: {}", acc[0][2]);
    assert_eq!(acc[0][3], 0.5, "coverage preserved by a colour grade");
}

#[test]
fn color_lookup_intensity_blends_toward_the_look() {
    // intensity 0 → identity; 0.5 → halfway to the full look.
    let base = [0.4, 0.4, 0.4, 1.0];
    let warm = LUT_PRESETS.iter().position(|(n, _)| *n == "Warm").unwrap() as u64;
    let graded = |amt: f32| {
        let mut a = [base];
        apply_color_lookup(
            &ColorLookupLutParams {
                lut_3d: LutHandle(warm),
                intensity: amt,
                profile: LutProfile::Srgb,
            },
            &mut a,
        );
        a[0]
    };
    assert_eq!(graded(0.0), base, "zero intensity is identity");
    let half = graded(0.5)[0];
    let full = graded(1.0)[0];
    assert!(
        half > base[0] && half < full,
        "intensity 0.5 sits between identity and the full look: {half} in ({}, {full})",
        base[0]
    );
}

#[test]
fn color_lookup_look_slider_round_trips_every_preset() {
    // The "Look" slider (slot 0) snaps to each preset index and reads back; slot 1
    // is the intensity. The panel relies on a stable thumb per preset.
    for idx in 0..LUT_PRESET_COUNT as u64 {
        let mut p = AdjustmentParams::ColorLookupLut(ColorLookupLutParams {
            lut_3d: LutHandle(idx),
            intensity: 1.0,
            profile: LutProfile::Srgb,
        });
        let look01 = adjustment_slider_params(&p)[0].1;
        // Round-trip through the setter restores the same preset index.
        set_adjustment_slider_param(&mut p, 0, look01);
        let AdjustmentParams::ColorLookupLut(cp) = &p else {
            unreachable!()
        };
        assert_eq!(
            cp.lut_3d.0, idx,
            "preset {idx} round-trips via the Look slider"
        );
    }
    // Two slots exposed: Look + Amount.
    let p = AdjustmentParams::ColorLookupLut(ColorLookupLutParams::default());
    assert_eq!(adjustment_slider_params(&p).len(), 2);
}

#[test]
fn noise_and_halftone_segments_and_toggle_round_trip() {
    // Noise: distribution segment (Gaussian/Uniform) + monochrome toggle.
    let mut noise = AdjustmentParams::Noise(NoiseParams::default());
    let (opts, sel) = adjustment_segment_params(&noise).expect("Noise has a distribution segment");
    assert_eq!(opts, vec!["Gaussian", "Uniform"]);
    assert_eq!(sel, 0, "default distribution is Gaussian");
    set_adjustment_segment_param(&mut noise, 1);
    assert_eq!(
        adjustment_segment_params(&noise).unwrap().1,
        1,
        "Uniform selected"
    );
    assert_eq!(
        adjustment_toggle_params(&noise),
        vec![("Monochrome", false)]
    );
    set_adjustment_toggle_param(&mut noise, 0, true);
    assert!(
        adjustment_toggle_params(&noise)[0].1,
        "monochrome toggled on"
    );

    // Halftone: cell-shape segment (Dot/Line/Circle).
    let mut ht = AdjustmentParams::Halftone(HalftoneParams::default());
    let (shapes, s0) = adjustment_segment_params(&ht).expect("Halftone has a shape segment");
    assert_eq!(shapes, vec!["Dot", "Line", "Circle"]);
    assert_eq!(s0, 0, "default shape is Dot");
    set_adjustment_segment_param(&mut ht, 2);
    assert_eq!(
        adjustment_segment_params(&ht).unwrap().1,
        2,
        "Circle selected"
    );
}

/// The two GPU-code sets, pinned as SETS — the antidote to the three stale prose
/// claims this file's doc-comments carried (see `gpu_spatial_code` /
/// `feathers_coverage`).
///
/// Prose is not load-bearing here: the painter's `flatten_for_gpu` reads exactly
/// these two functions to route a document to the GPU or to a producer up to 885×
/// slower (`docs/Painter/25_avaliacao_gpu.md`). So the memberships are asserted, and
/// asserted by ENUMERATION of `AdjustmentKind::ALL` — a spot-check of the kinds
/// someone remembered is how the previous list rotted.
#[test]
fn the_gpu_code_sets_are_what_the_routing_believes_they_are() {
    let per_pixel: Vec<AdjustmentKind> = AdjustmentKind::ALL
        .iter()
        .copied()
        .filter(|k| k.gpu_code().is_some())
        .collect();
    let spatial: Vec<AdjustmentKind> = AdjustmentKind::ALL
        .iter()
        .copied()
        .filter(|k| k.gpu_spatial_code().is_some())
        .collect();
    let neither: Vec<AdjustmentKind> = AdjustmentKind::ALL
        .iter()
        .copied()
        .filter(|k| k.gpu_code().is_none() && k.gpu_spatial_code().is_none())
        .collect();

    // No kind may claim BOTH: the flatten checks spatial first, so a kind in both
    // sets would silently never take its per-pixel path.
    for k in &per_pixel {
        assert!(
            k.gpu_spatial_code().is_none(),
            "{k:?} claims a per-pixel AND a spatial code — the flatten would only ever run the spatial one"
        );
    }

    assert_eq!(
        spatial,
        vec![
            AdjustmentKind::GaussianBlur,
            AdjustmentKind::MotionBlur,
            AdjustmentKind::Bloom,
            AdjustmentKind::Sharpen,
            AdjustmentKind::ChromaticAberration,
            AdjustmentKind::ShadowsHighlights,
        ],
        "Bloom and ShadowsHighlights ARE spatial-ported; the doc that said otherwise was stale"
    );
    assert_eq!(
        neither,
        vec![
            AdjustmentKind::ColorBalance,
            AdjustmentKind::GradientMap,
            AdjustmentKind::PhotoFilter,
            AdjustmentKind::SelectiveColor,
            AdjustmentKind::ChannelMixer,
            AdjustmentKind::BlackAndWhite,
        ],
        "these six are the ONLY kinds that still force the painter onto the CPU compositor"
    );
    assert_eq!(
        per_pixel.len() + spatial.len() + neither.len(),
        AdjustmentKind::ALL.len(),
        "every kind lands in exactly one bucket"
    );

    // `feathers_coverage` is a strict SUBSET of the spatial set (S/H is spatial but
    // outputs base coverage) — the direction the old doc had backwards.
    for k in AdjustmentKind::ALL {
        if k.feathers_coverage() {
            assert!(
                k.gpu_spatial_code().is_some(),
                "{k:?} feathers coverage but has no spatial kernel — the sets have drifted"
            );
        }
    }
    assert!(
        !AdjustmentKind::ShadowsHighlights.feathers_coverage(),
        "S/H blurs only an internal luma map; it must not adopt kernel alpha"
    );
}
