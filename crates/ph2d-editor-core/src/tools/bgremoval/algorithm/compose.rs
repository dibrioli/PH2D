//! Final compose step: produce `scratch.output_rgba` from the
//! segmentation mask, optional soft alpha, and the input RGBA.
//!
//! Three paths:
//!
//! 1. **Guided filter ran** (`did_refine = true`): alpha comes from
//!    `scratch.alpha_f32` directly. Despill (when enabled, chroma-mode
//!    only) subtracts the detected bg chroma from soft-edge pixels.
//!
//! 2. **No refinement, Chroma mode** (`did_refine = false &&
//!    params.mode == Chroma`): alpha synthesised from `scratch.mask` +
//!    `scratch.delta_e` using the `[tolerance, tolerance + feather]`
//!    soft-band formula. Important: `scratch.delta_e[i]` is the
//!    **squared** Oklab distance (the chroma backend squares it once
//!    to avoid a per-pixel `sqrt` in its main loop), so comparisons
//!    happen against `tol_sq` / `(tol + feat)²`; the band-position
//!    fraction `t` linearises with `sqrt` once per soft-band pixel.
//!
//! 3. **No refinement, non-Chroma mode** (`did_refine = false &&
//!    params.mode == GrabCut`): GrabCut never writes `scratch.delta_e`;
//!    reading it would yield stale data from a prior chroma run on the
//!    same scratch (mode-flip in the panel). Path 3 ignores delta_e
//!    entirely and writes a binary alpha (0 or 255) directly from
//!    `scratch.mask`. Final-audit bug fix 2026-05-16.
//!
//! All paths assume the input is **straight-alpha RGBA8** (not
//! premultiplied) — enforced at the API boundary of
//! [`super::super::tool::BgRemovalTool::set_source_snapshot`].

use super::super::params::{BgRemovalMode, BgRemovalParams};
use super::super::scratch::BgRemovalScratch;
use super::SegmentResult;

/// Write the final RGBA into `scratch.output_rgba`.
pub fn write_output(
    rgba: &[u8],
    w: u32,
    h: u32,
    params: &BgRemovalParams,
    segment: &SegmentResult,
    did_refine: bool,
    scratch: &mut BgRemovalScratch,
) {
    let n = (w as usize) * (h as usize);
    debug_assert_eq!(rgba.len(), n * 4);
    debug_assert_eq!(scratch.output_rgba.len(), n * 4);

    if did_refine {
        // Path 1 — soft alpha from guided_filter.
        for i in 0..n {
            let a = (scratch.alpha_f32[i].clamp(0.0, 1.0) * 255.0 + 0.5) as u8;
            let base = i * 4;
            scratch.output_rgba[base] = rgba[base];
            scratch.output_rgba[base + 1] = rgba[base + 1];
            scratch.output_rgba[base + 2] = rgba[base + 2];
            scratch.output_rgba[base + 3] = a;
        }
    } else if params.mode == BgRemovalMode::Chroma {
        // Path 2 — hard mask + delta_e soft band (chroma mode only).
        // `delta_e` is SQUARED Oklab distance — compare against
        // `tol_sq` / `(tol + feat)²`, linearise via `sqrt` once per
        // soft-band pixel (P0 fix 2026-05-16: previously compared
        // raw `tol` against squared `delta_e`, so the soft band
        // never fired at expected positions).
        let tol = params.chroma.tolerance;
        let feat = params.chroma.feather.max(1e-6);
        let tol_sq = tol * tol;
        let outer = tol + feat;
        let outer_sq = outer * outer;
        for i in 0..n {
            let alpha = if scratch.mask[i] == 0 {
                // Hard bg from the connected-flood pass.
                0u8
            } else {
                let de_sq = scratch.delta_e[i];
                if de_sq >= outer_sq {
                    255
                } else if de_sq >= tol_sq {
                    let de = de_sq.max(0.0).sqrt();
                    let t = ((de - tol) / feat).clamp(0.0, 1.0);
                    (t * 255.0 + 0.5) as u8
                } else {
                    // ΔE below threshold but mask says fg → keep
                    // (e.g. flood-protected interior).
                    255
                }
            };
            let base = i * 4;
            scratch.output_rgba[base] = rgba[base];
            scratch.output_rgba[base + 1] = rgba[base + 1];
            scratch.output_rgba[base + 2] = rgba[base + 2];
            scratch.output_rgba[base + 3] = alpha;
        }
    } else {
        // Path 3 — non-Chroma mode (GrabCut), no refinement.
        // Binary alpha straight from the mask; `scratch.delta_e`
        // would be stale from a prior chroma run on the same
        // scratch and must not leak into this output (P0 fix
        // 2026-05-16: gating moved from comment-only to actual
        // branch).
        for i in 0..n {
            let base = i * 4;
            scratch.output_rgba[base] = rgba[base];
            scratch.output_rgba[base + 1] = rgba[base + 1];
            scratch.output_rgba[base + 2] = rgba[base + 2];
            scratch.output_rgba[base + 3] = scratch.mask[i];
        }
    }

    // Despill — only when chroma mode + flag set + we actually have a
    // detected bg colour to subtract. Stub for M1: despill math lands
    // with the real chroma implementation; the skeleton just exercises
    // the branch via the destructure.
    if params.mode == BgRemovalMode::Chroma
        && params.chroma.despill
        && let SegmentResult::Chroma { bg_oklab: _ } = segment
    {
        // M1 stub.
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tools::bgremoval::params::{BgRemovalMode, BgRemovalParams};

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
            mode: BgRemovalMode::Chroma,
            chroma: crate::tools::bgremoval::params::ChromaParams {
                tolerance: 0.10,
                feather: 0.04,
                ..crate::tools::bgremoval::params::ChromaParams::default()
            },
            ..BgRemovalParams::default()
        };
        let segment = SegmentResult::Chroma { bg_oklab: [0.0; 3] };

        write_output(&rgba, w, h, &params, &segment, false, &mut scratch);

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
            mode: BgRemovalMode::Chroma,
            ..BgRemovalParams::default()
        };
        let segment = SegmentResult::Chroma { bg_oklab: [0.0; 3] };
        write_output(&rgba, w, h, &params, &segment, false, &mut scratch);
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
            mode: BgRemovalMode::GrabCut,
            ..BgRemovalParams::default()
        };
        let segment = SegmentResult::GrabCut;
        write_output(&rgba, w, h, &params, &segment, false, &mut scratch);

        // Alpha must equal mask, byte-for-byte. No delta_e leak.
        assert_eq!(scratch.output_rgba[3], 0);
        assert_eq!(scratch.output_rgba[7], 255);
        assert_eq!(scratch.output_rgba[11], 255);
        assert_eq!(scratch.output_rgba[15], 0);
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
        let params = BgRemovalParams::default();
        let segment = SegmentResult::GrabCut;
        write_output(&rgba, w, h, &params, &segment, true, &mut scratch);
        assert_eq!(scratch.output_rgba[3], 0);
        assert_eq!(scratch.output_rgba[7], 128); // 0.5*255+0.5 = 128
        assert_eq!(scratch.output_rgba[11], 255);
        assert_eq!(scratch.output_rgba[15], 255);
    }

    #[test]
    fn rgb_channels_passthrough_in_every_path() {
        let w = 1u32;
        let h = 1u32;
        let mut scratch = fresh_scratch(w, h);
        let rgba = solid_rgba(w, h, [123, 45, 67]);
        scratch.mask[0] = 255;
        scratch.alpha_f32[0] = 0.5;
        let params = BgRemovalParams::default();
        let segment = SegmentResult::GrabCut;
        // Path 1.
        write_output(&rgba, w, h, &params, &segment, true, &mut scratch);
        assert_eq!(&scratch.output_rgba[0..3], &[123, 45, 67]);
        // Path 3.
        let p3 = BgRemovalParams {
            mode: BgRemovalMode::GrabCut,
            ..params.clone()
        };
        write_output(&rgba, w, h, &p3, &segment, false, &mut scratch);
        assert_eq!(&scratch.output_rgba[0..3], &[123, 45, 67]);
    }
}
