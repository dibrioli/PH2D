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
    // The 9 GPU-implemented kinds map to codes 0..=8 (0..=6 scalar, 7/8 the W4
    // display-space transfer LUTs); the rest are None (CPU-only). Codes must
    // match the WGSL `ADJ_*` consts.
    assert_eq!(AdjustmentKind::HueSaturationBrightness.gpu_code(), Some(0));
    assert_eq!(AdjustmentKind::BrightnessContrast.gpu_code(), Some(1));
    assert_eq!(AdjustmentKind::Invert.gpu_code(), Some(2));
    assert_eq!(AdjustmentKind::Posterize.gpu_code(), Some(3));
    assert_eq!(AdjustmentKind::Threshold.gpu_code(), Some(4));
    assert_eq!(AdjustmentKind::Exposure.gpu_code(), Some(5));
    assert_eq!(AdjustmentKind::Vibrance.gpu_code(), Some(6));
    assert_eq!(AdjustmentKind::Curves.gpu_code(), Some(7));
    assert_eq!(AdjustmentKind::Levels.gpu_code(), Some(8));
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
