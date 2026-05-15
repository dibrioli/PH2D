//! Final compose step: produce `scratch.output_rgba` from the
//! segmentation mask, optional soft alpha, and the input RGBA.
//!
//! Two paths:
//!
//! 1. **Guided filter ran** (`did_refine = true`): alpha comes from
//!    `scratch.alpha_f32` directly. Despill (when enabled, chroma-mode
//!    only) subtracts the detected bg chroma from soft-edge pixels.
//!
//! 2. **No refinement** (`did_refine = false`): alpha synthesised
//!    from `scratch.mask` + `scratch.delta_e` using the
//!    `[tolerance, tolerance + feather]` soft-band formula.
//!
//! Both paths assume the input is **straight-alpha RGBA8** (not
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
    } else {
        // Path 2 — hard mask + delta_e soft band (chroma mode only).
        let tol = params.chroma.tolerance;
        let feat = params.chroma.feather.max(1e-6);
        for i in 0..n {
            let alpha = if scratch.mask[i] == 0 {
                // Hard bg.
                0u8
            } else {
                // Foreground OR in the soft band, depending on ΔE.
                let de = scratch.delta_e[i];
                if de >= tol + feat {
                    255
                } else if de >= tol {
                    let t = (de - tol) / feat;
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
