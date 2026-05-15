//! GrabCut (Rother, Kolmogorov, Blake 2004) on a downsampled 1024²
//! image; mask upsampled via nearest, refined by the Guided Filter
//! post-process.
//!
//! Memory layout, β computation, and max-flow are ported from
//! OpenCV. See the per-file Apache-2.0 attribution at the top of
//! `maxflow.rs` (lands in the M2 implementation pass).
//!
//! This file is the M2 stub — real implementation lands after the
//! M1 implementation pass + audit.

use super::super::params::GrabCutParams;
use super::super::scratch::BgRemovalScratch;
use super::SegmentResult;

/// Run GrabCut segmentation. Writes `scratch.mask[i]` = 0 (background)
/// or 255 (foreground).
///
/// Internally downsamples to `min(input, 1024²)` and upsamples the
/// final mask via nearest neighbour. The Guided Filter refinement
/// step (when enabled) absorbs the resulting aliasing.
pub fn segment(
    rgba: &[u8],
    w: u32,
    h: u32,
    params: &GrabCutParams,
    scratch: &mut BgRemovalScratch,
) -> SegmentResult {
    let n = (w as usize) * (h as usize);
    debug_assert_eq!(rgba.len(), n * 4);
    let _ = params;
    // STUB — M2 implementation marks everything as fg so the skeleton
    // build is well-defined.
    for v in &mut scratch.mask[..n] {
        *v = 255;
    }
    SegmentResult::GrabCut
}
