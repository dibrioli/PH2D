//! Guided Image Filter (He, Sun, Tang — ECCV 2010, TPAMI 2013) used
//! as a post-process refinement step on the binary mask produced by
//! `chroma::segment` or `grabcut::segment`.
//!
//! Implementation choices (per design review):
//! - Box filter via **separable rolling-sum** (two 1D passes) — not
//!   summed-area tables, to avoid f32 precision loss at 4k.
//! - **Fast Guided Filter** wrapper (s = 4 downsample / bilinear
//!   upsample of `a, b`) keeps 4k apply latency under ~100 ms.
//! - **Color guide** (3×3 covariance + closed-form solve) at full
//!   colour — meaningfully better edges than luma-only on cromatically
//!   similar / luminance-distinct boundaries.
//! - **Boundary-only band**: only pixels within ±2r of the input
//!   mask boundary are filtered; interior is copied verbatim (5-10×
//!   speedup on typical sprite masks).
//!
//! This file is the M3 stub — real implementation lands in the M3
//! pass, after the M1 pass + audit.

use super::super::params::GuidedFilterParams;
use super::super::scratch::BgRemovalScratch;

/// Refine `scratch.mask` (binary 0/255) into `scratch.alpha_f32`
/// (smooth alpha in `[0, 1]`) using `rgba` as the guide image.
///
/// Called only when `params.radius > 0`. The orchestrator
/// (`super::run_pipeline`) ensures this contract.
pub fn refine(
    rgba: &[u8],
    w: u32,
    h: u32,
    params: &GuidedFilterParams,
    scratch: &mut BgRemovalScratch,
) {
    let n = (w as usize) * (h as usize);
    debug_assert_eq!(rgba.len(), n * 4);
    debug_assert!(params.radius > 0);
    // STUB — M3 implementation. For the skeleton, just promote the
    // hard mask to f32 alpha so the compose step can run end-to-end.
    for i in 0..n {
        scratch.alpha_f32[i] = scratch.mask[i] as f32 / 255.0;
    }
}
