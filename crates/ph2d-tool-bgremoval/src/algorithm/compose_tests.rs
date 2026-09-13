//! Testes de `compose.rs` — irmão por tecto de LOC (`workspace_src_files_under_loc_cap`).
//!
//! Corte mecânico: o `mod tests` saiu inteiro, verbatim, do ficheiro que o continha.

use super::*;
use crate::params::BgRemovalParams;

fn fresh_scratch(w: u32, h: u32) -> BgRemovalScratch {
    let mut s = BgRemovalScratch::default();
    s.ensure(w, h, false);
    s
}

fn solid_rgba(w: u32, h: u32, color: [u8; 3]) -> Vec<u8> {
    let n = (w * h) as usize;
    let mut rgba = vec![0u8; n * 4];
    for i in 0..n {
        rgba[i * 4] = color[0];
        rgba[i * 4 + 1] = color[1];
        rgba[i * 4 + 2] = color[2];
        rgba[i * 4 + 3] = 255;
    }
    rgba
}

// --- Path 2 (Chroma, no refinement) — squared-unit correctness ----

#[test]
fn path2_chroma_soft_band_uses_squared_units_at_band_edges() {
    // Audit BUG A (2026-05-16): soft-band comparisons must use
    // squared units (delta_e is ΔE²). With tol = 0.10, feat = 0.04:
    // tol_sq = 0.01, (tol+feat)² = 0.0196.
    //
    // We pick delta_e values *inside* the band (not at the exact
    // boundary, where f32(0.01) ≠ f32(0.10)² by ~1 ULP) so the
    // test doesn't rely on bit-exact boundary semantics.
    let w = 4u32;
    let h = 1u32;
    let mut scratch = fresh_scratch(w, h);
    let rgba = solid_rgba(w, h, [200, 30, 30]);
    for v in scratch.mask.iter_mut().take(4) {
        *v = 255;
    }
    // pixel 0: below tol_sq          → keep as fg (255)
    // pixel 1: linear ΔE ≈ 0.11      → t ≈ 0.25, alpha ≈ 64
    // pixel 2: linear ΔE ≈ 0.12      → t = 0.50, alpha ≈ 128
    // pixel 3: above outer_sq        → 255 (saturated)
    scratch.delta_e[0] = 0.005;
    scratch.delta_e[1] = 0.11_f32 * 0.11_f32; // ~0.0121
    scratch.delta_e[2] = 0.12_f32 * 0.12_f32; // ~0.0144
    scratch.delta_e[3] = 0.025;

    let params = BgRemovalParams {
        chroma: crate::params::ChromaParams {
            tolerance: 0.10,
            feather: 0.04,
            ..crate::params::ChromaParams::default()
        },
        // Pin neutral grow so these tests isolate the compose path
        // (the default params now carry a slight erode).
        grow_px: 0.0,
        ..BgRemovalParams::default()
    };
    let segment = SegmentResult::Chroma { bg_oklab: [0.0; 3] };

    write_output(
        &rgba,
        w,
        h,
        &params,
        &segment,
        false,
        None,
        None,
        &mut scratch,
    );

    assert_eq!(scratch.output_rgba[3], 255, "below tol_sq must keep fg");
    // Quarter-band: t ≈ 0.25 ⇒ alpha ≈ 64.
    let q_alpha = scratch.output_rgba[7];
    assert!(
        (q_alpha as i32 - 64).abs() <= 3,
        "quarter-band alpha = {q_alpha}, expected ~64"
    );
    // Mid-band: t = 0.5 ⇒ alpha ≈ 128.
    let mid_alpha = scratch.output_rgba[11];
    assert!(
        (mid_alpha as i32 - 128).abs() <= 3,
        "mid-band alpha = {mid_alpha}, expected ~128"
    );
    assert_eq!(scratch.output_rgba[15], 255, "above outer_sq must give 255");
}

#[test]
fn path2_mask_zero_gives_hard_bg_regardless_of_delta_e() {
    let w = 1u32;
    let h = 1u32;
    let mut scratch = fresh_scratch(w, h);
    let rgba = solid_rgba(w, h, [10, 20, 30]);
    scratch.mask[0] = 0;
    scratch.delta_e[0] = 0.05; // mid-band but mask says bg
    let params = BgRemovalParams {
        // Pin neutral grow so these tests isolate the compose path
        // (the default params now carry a slight erode).
        grow_px: 0.0,
        ..BgRemovalParams::default()
    };
    let segment = SegmentResult::Chroma { bg_oklab: [0.0; 3] };
    write_output(
        &rgba,
        w,
        h,
        &params,
        &segment,
        false,
        None,
        None,
        &mut scratch,
    );
    assert_eq!(scratch.output_rgba[3], 0, "mask=0 must produce alpha=0");
}

// --- Path 3 (GrabCut, no refinement) — no delta_e leak ------------

#[test]
fn path3_grabcut_no_refine_ignores_stale_delta_e() {
    // Audit BUG B (2026-05-16): GrabCut + radius=0 must NOT read
    // delta_e. We poison delta_e to non-zero values that, under
    // the old buggy code path, would cause some mask=255 pixels
    // to mistakenly be down-mapped via the chroma soft-band.
    let w = 4u32;
    let h = 1u32;
    let mut scratch = fresh_scratch(w, h);
    let rgba = solid_rgba(w, h, [10, 20, 30]);
    // Mask: 0, 255, 255, 0  (binary-only, GrabCut convention).
    scratch.mask[0] = 0;
    scratch.mask[1] = 255;
    scratch.mask[2] = 255;
    scratch.mask[3] = 0;
    // Stale delta_e from a prior chroma run — would corrupt
    // alpha under the old path-2 code.
    scratch.delta_e[0] = 0.01;
    scratch.delta_e[1] = 0.0144; // mid-soft-band under old code
    scratch.delta_e[2] = 0.005;
    scratch.delta_e[3] = 0.02;

    let params = BgRemovalParams {
        // Pin neutral grow so these tests isolate the compose path
        // (the default params now carry a slight erode).
        grow_px: 0.0,
        ..BgRemovalParams::default()
    };
    let segment = SegmentResult::Chroma { bg_oklab: [0.0; 3] };
    write_output(
        &rgba,
        w,
        h,
        &params,
        &segment,
        false,
        None,
        None,
        &mut scratch,
    );

    // Alpha must equal mask, byte-for-byte. No delta_e leak.
    assert_eq!(scratch.output_rgba[3], 0);
    assert_eq!(scratch.output_rgba[7], 255);
    assert_eq!(scratch.output_rgba[11], 255);
    assert_eq!(scratch.output_rgba[15], 0);
}

// --- Protection force-keep --------------------------------------

#[test]
fn protect_forces_protected_pixel_opaque_regardless_of_mask() {
    // 2×1: both pixels classified as background (mask 0). The first
    // is protected → must end fully opaque; the second is not →
    // stays transparent.
    let w = 2u32;
    let h = 1u32;
    let mut scratch = fresh_scratch(w, h);
    let rgba = solid_rgba(w, h, [10, 20, 30]);
    scratch.mask[0] = 0;
    scratch.mask[1] = 0;
    let params = BgRemovalParams {
        grow_px: 0.0,
        ..BgRemovalParams::default()
    };
    let segment = SegmentResult::Chroma { bg_oklab: [0.0; 3] };
    let protect = [255u8, 0u8];
    write_output(
        &rgba,
        w,
        h,
        &params,
        &segment,
        false,
        Some(&protect),
        None,
        &mut scratch,
    );
    assert_eq!(
        scratch.output_rgba[3], 255,
        "protected bg pixel must be forced opaque"
    );
    assert_eq!(
        scratch.output_rgba[7], 0,
        "unprotected bg pixel stays transparent"
    );
}

#[test]
fn protect_soft_strength_blends_proportionally() {
    // A soft-falloff rim (strength 128) over a background pixel
    // (mask 0) must lift alpha to ~128, not slam it to 255 — the
    // feathered keep-edge blends into the matte.
    let w = 2u32;
    let h = 1u32;
    let mut scratch = fresh_scratch(w, h);
    let rgba = solid_rgba(w, h, [10, 20, 30]);
    scratch.mask[0] = 0;
    scratch.mask[1] = 255; // already opaque foreground
    let params = BgRemovalParams {
        grow_px: 0.0,
        ..BgRemovalParams::default()
    };
    let segment = SegmentResult::Chroma { bg_oklab: [0.0; 3] };
    let protect = [128u8, 64u8];
    write_output(
        &rgba,
        w,
        h,
        &params,
        &segment,
        false,
        Some(&protect),
        None,
        &mut scratch,
    );
    // bg pixel lifted to the strength; fg pixel keeps its higher alpha.
    assert_eq!(
        scratch.output_rgba[3], 128,
        "soft rim lifts bg alpha to strength"
    );
    assert_eq!(
        scratch.output_rgba[7], 255,
        "max keeps the higher seg alpha"
    );
}

#[test]
fn despill_skips_protected_pixels_no_magenta_fringe() {
    // A dark line-art pixel at fractional alpha over a green bg would
    // despill to magenta (`(C−(1−a)·bg)/a` zeroes green, amplifies
    // red+blue). A PROTECTED pixel must be left untouched — this is
    // the fix for the pink outline around a painted keep-region.
    let w = 2u32;
    let h = 1u32;
    let bg_oklab = crate::algorithm::chroma::srgb_to_oklab(0, 200, 0);
    let params = BgRemovalParams {
        grow_px: 0.0,
        ..BgRemovalParams::default() // despill defaults on
    };
    let segment = SegmentResult::Chroma { bg_oklab };
    let rgba = solid_rgba(w, h, [20, 20, 20]); // dark line-art
    let make = || {
        let mut s = fresh_scratch(w, h);
        s.alpha_f32[0] = 0.5;
        s.alpha_f32[1] = 0.5;
        s
    };

    // Unprotected → despill rewrites the RGB (green removed).
    let mut s_un = make();
    write_output(&rgba, w, h, &params, &segment, true, None, None, &mut s_un);
    assert_ne!(
        &s_un.output_rgba[0..3],
        &[20, 20, 20],
        "unprotected fractional pixel is despilled"
    );

    // Protected → RGB untouched (no magenta).
    let mut s_pr = make();
    let protect = [255u8, 255u8];
    write_output(
        &rgba,
        w,
        h,
        &params,
        &segment,
        true,
        Some(&protect),
        None,
        &mut s_pr,
    );
    assert_eq!(
        &s_pr.output_rgba[0..3],
        &[20, 20, 20],
        "protected pixel keeps its true colour (despill skipped)"
    );
}

#[test]
fn protect_survives_a_full_shrink() {
    // A protected pixel must survive even an aggressive erode: the
    // force-keep runs AFTER grow/shrink. 3×1 opaque row, shrink by 2,
    // centre pixel protected.
    let w = 3u32;
    let h = 1u32;
    let mut scratch = fresh_scratch(w, h);
    let rgba = solid_rgba(w, h, [200, 50, 50]);
    for i in 0..3 {
        scratch.mask[i] = 255;
        scratch.output_rgba[i * 4 + 3] = 255;
    }
    let params = BgRemovalParams {
        grow_px: -2.0,
        ..BgRemovalParams::default()
    };
    let segment = SegmentResult::Chroma { bg_oklab: [0.0; 3] };
    let protect = [0u8, 255u8, 0u8];
    write_output(
        &rgba,
        w,
        h,
        &params,
        &segment,
        false,
        Some(&protect),
        None,
        &mut scratch,
    );
    assert_eq!(
        scratch.output_rgba[7], 255,
        "protected centre pixel must survive the shrink"
    );
}

// --- Path 1 (refinement) — alpha_f32 round-trip -------------------

#[test]
fn path1_refined_alpha_f32_maps_to_u8_with_round_half_up() {
    let w = 4u32;
    let h = 1u32;
    let mut scratch = fresh_scratch(w, h);
    let rgba = solid_rgba(w, h, [100, 100, 100]);
    scratch.alpha_f32[0] = 0.0;
    scratch.alpha_f32[1] = 0.5;
    scratch.alpha_f32[2] = 1.0;
    scratch.alpha_f32[3] = 1.5; // out-of-range → clamp
    let params = BgRemovalParams {
        grow_px: 0.0, // isolate compose path from the default erode
        ..BgRemovalParams::default()
    };
    let segment = SegmentResult::Chroma { bg_oklab: [0.0; 3] };
    write_output(
        &rgba,
        w,
        h,
        &params,
        &segment,
        true,
        None,
        None,
        &mut scratch,
    );
    assert_eq!(scratch.output_rgba[3], 0);
    assert_eq!(scratch.output_rgba[7], 128); // 0.5*255+0.5 = 128
    assert_eq!(scratch.output_rgba[11], 255);
    assert_eq!(scratch.output_rgba[15], 255);
}

// --- Edge bleed -------------------------------------------------

#[test]
fn bleed_fills_transparent_neighbour_with_foreground_rgb() {
    // 3×1: opaque FG (green) | transparent (cream contamination) |
    // transparent. After bleed the transparent pixels' RGB must
    // become the FG green; alpha must stay 0.
    let w = 3u32;
    let h = 1u32;
    let mut scratch = fresh_scratch(w, h);
    // pixel 0: opaque green source
    scratch.output_rgba[0..4].copy_from_slice(&[0, 200, 0, 255]);
    // pixel 1: transparent but holds cream bg
    scratch.output_rgba[4..8].copy_from_slice(&[240, 230, 210, 0]);
    // pixel 2: transparent cream
    scratch.output_rgba[8..12].copy_from_slice(&[240, 230, 210, 0]);

    bleed_edges(w, h, &mut scratch);

    // Pixel 1 took the green source RGB; alpha preserved at 0.
    assert_eq!(&scratch.output_rgba[4..7], &[0, 200, 0]);
    assert_eq!(scratch.output_rgba[7], 0, "alpha must be untouched");
    // Pixel 2 filled from the now-valid pixel 1 on the second ring.
    assert_eq!(&scratch.output_rgba[8..11], &[0, 200, 0]);
    assert_eq!(scratch.output_rgba[11], 0);
}

// --- Grow / Shrink morphology -----------------------------------

#[test]
fn shrink_erodes_a_one_pixel_border_off_the_matte() {
    // 5×5 solid-opaque block. One erosion pass must zero the outer
    // ring (every border pixel has an off-image / interior min of 0
    // only at the frame edge) — concretely the 4 corners + edges
    // touching the frame go to 0; the 3×3 interior stays 255.
    let (w, h) = (5u32, 5u32);
    let mut scratch = fresh_scratch(w, h);
    for i in 0..(w * h) as usize {
        scratch.output_rgba[i * 4 + 3] = 255;
    }
    grow_shrink_alpha(w, h, -1.0, &mut scratch);
    // Interior 3×3 (rows/cols 1..=3) stays opaque.
    for y in 1..=3 {
        for x in 1..=3 {
            let a = scratch.output_rgba[(y * 5 + x) * 4 + 3];
            assert_eq!(a, 255, "interior ({x},{y}) should survive erosion");
        }
    }
    // Border pixels eroded to 0 (off-image counts as background).
    assert_eq!(scratch.output_rgba[3], 0, "corner (0,0) eroded");
    assert_eq!(scratch.output_rgba[(2 * 5) * 4 + 3], 0, "edge (0,2) eroded");
}

#[test]
fn grow_dilates_into_a_transparent_neighbour() {
    // 3×1: opaque | transparent | transparent. One dilation pass
    // grows the opaque alpha into pixel 1.
    let (w, h) = (3u32, 1u32);
    let mut scratch = fresh_scratch(w, h);
    scratch.output_rgba[3] = 255; // pixel 0 opaque
    grow_shrink_alpha(w, h, 1.0, &mut scratch);
    assert_eq!(scratch.output_rgba[7], 255, "pixel 1 dilated to opaque");
}

#[test]
fn grow_zero_is_a_noop() {
    let (w, h) = (3u32, 3u32);
    let mut scratch = fresh_scratch(w, h);
    let original: Vec<u8> = (0..36).map(|i| i as u8).collect();
    scratch.output_rgba[..36].copy_from_slice(&original);
    grow_shrink_alpha(w, h, 0.0, &mut scratch);
    assert_eq!(&scratch.output_rgba[..36], &original[..]);
}

#[test]
fn bleed_never_touches_partial_alpha_edge_pixels() {
    // The partial-alpha AA band IS the artwork in comic line-art;
    // overwriting it corrupts the edge. The bleed must leave every
    // alpha<255 visible pixel byte-for-byte and only fill the fully
    // transparent collar.
    let w = 3u32;
    let h = 1u32;
    let mut scratch = fresh_scratch(w, h);
    scratch.output_rgba[0..4].copy_from_slice(&[0, 200, 0, 255]); // opaque FG
    scratch.output_rgba[4..8].copy_from_slice(&[180, 60, 90, 128]); // partial edge
    scratch.output_rgba[8..12].copy_from_slice(&[240, 230, 210, 0]); // transparent

    bleed_edges(w, h, &mut scratch);

    // Partial edge pixel: untouched.
    assert_eq!(&scratch.output_rgba[4..8], &[180, 60, 90, 128]);
    // Transparent pixel: filled from a visible neighbour (alpha 0
    // → invisible in the preview, so this never alters it).
    assert_eq!(scratch.output_rgba[11], 0, "alpha untouched");
    assert_ne!(
        &scratch.output_rgba[8..11],
        &[240, 230, 210],
        "transparent collar must take a visible neighbour colour"
    );
}

#[test]
fn bleed_leaves_fully_opaque_image_untouched() {
    let w = 2u32;
    let h = 2u32;
    let mut scratch = fresh_scratch(w, h);
    let original: Vec<u8> = (0..16).map(|i| if i % 4 == 3 { 255 } else { i }).collect();
    scratch.output_rgba[..16].copy_from_slice(&original);
    bleed_edges(w, h, &mut scratch);
    assert_eq!(&scratch.output_rgba[..16], &original[..]);
}

#[test]
fn rgb_channels_passthrough_in_both_paths() {
    let w = 1u32;
    let h = 1u32;
    let mut scratch = fresh_scratch(w, h);
    let rgba = solid_rgba(w, h, [123, 45, 67]);
    scratch.mask[0] = 255;
    scratch.alpha_f32[0] = 0.5;
    let mut params = BgRemovalParams {
        grow_px: 0.0, // isolate compose path from the default erode
        ..BgRemovalParams::default()
    };
    // Disable despill so this test isolates RGB passthrough (despill
    // intentionally rewrites fractional-alpha RGB; covered elsewhere).
    params.chroma.despill = false;
    let segment = SegmentResult::Chroma { bg_oklab: [0.0; 3] };
    // Path 1 (refined).
    write_output(
        &rgba,
        w,
        h,
        &params,
        &segment,
        true,
        None,
        None,
        &mut scratch,
    );
    assert_eq!(&scratch.output_rgba[0..3], &[123, 45, 67]);
    // Path 2 (soft band).
    write_output(
        &rgba,
        w,
        h,
        &params,
        &segment,
        false,
        None,
        None,
        &mut scratch,
    );
    assert_eq!(&scratch.output_rgba[0..3], &[123, 45, 67]);
}
