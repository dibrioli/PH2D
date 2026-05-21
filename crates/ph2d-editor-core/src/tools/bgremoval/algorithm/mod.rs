//! Pure algorithms for the bgremoval pipeline.
//!
//! No editor / host state, no Vello, no `Tool` trait — every entry
//! point is a free function consuming RGBA + params + scratch. The
//! orchestrator [`run_pipeline`] chains:
//!
//! ```text
//!   segment (chroma | grabcut)  →  refine (guided_filter, optional)  →  compose
//! ```
//!
//! Each stage writes into a named field on
//! [`super::scratch::BgRemovalScratch`] so the per-frame thumbnail
//! re-runs reuse allocations (HR-3).

pub mod chroma;
pub mod compose;
pub mod grabcut;
pub mod guided_filter;

use super::params::{BgRemovalMode, BgRemovalParams};
use super::scratch::BgRemovalScratch;

/// Run the full background-removal pipeline on `rgba` (size `w * h * 4`,
/// RGBA8, straight-alpha) and write the result into
/// `scratch.output_rgba`. The output is the same shape as the input
/// with the alpha channel modified to reflect the segmentation.
///
/// `protect` is an optional freehand foreground-protection mask aligned
/// to the **same `w × h`** as `rgba` (one byte/pixel, `>= 128` =
/// protected / forced-foreground). When present, protected pixels are
/// locked to the foreground side (`TriLabel::FgHard` for GrabCut) and
/// forced opaque in the final compose, for both backends. Pass `None`
/// when no region is painted. The caller (the tool) is responsible for
/// resampling its source-resolution mask to `(w, h)` first.
///
/// # Panics
/// Panics if `rgba.len() != (w * h * 4) as usize`, or if `protect` is
/// `Some` and its length is not `w * h`.
pub fn run_pipeline(
    rgba: &[u8],
    w: u32,
    h: u32,
    params: &BgRemovalParams,
    protect: Option<&[u8]>,
    scratch: &mut BgRemovalScratch,
) {
    let expected = (w as usize) * (h as usize) * 4;
    assert_eq!(
        rgba.len(),
        expected,
        "rgba slice length must equal w*h*4 (was {} expected {})",
        rgba.len(),
        expected
    );
    if let Some(pm) = protect {
        assert_eq!(
            pm.len(),
            (w as usize) * (h as usize),
            "protect mask length must equal w*h (was {} expected {})",
            pm.len(),
            (w as usize) * (h as usize)
        );
    }

    scratch.ensure(w, h, params.refinement.color_guide);

    // Step 1 — primary segmentation. Writes scratch.mask (binary 0/255)
    // and (for chroma) scratch.delta_e. Returns the side-channel
    // SegmentResult so compose can despill against the detected bg.
    let segment_result = match params.mode {
        BgRemovalMode::Chroma => {
            chroma::segment(rgba, w, h, &params.chroma, &params.extra_bg_colors, scratch)
        }
        BgRemovalMode::GrabCut => grabcut::segment(rgba, w, h, &params.grabcut, protect, scratch),
    };

    // Step 2 — refinement (optional). Writes scratch.alpha_f32 if it
    // runs; otherwise compose falls back to mask + delta_e soft band.
    let did_refine = if params.refinement.radius > 0 {
        guided_filter::refine(rgba, w, h, &params.refinement, scratch);
        true
    } else {
        false
    };

    // Step 3 — compose. Writes scratch.output_rgba. The protection mask
    // is applied here as a final force-keep so a painted region stays
    // opaque regardless of backend / refinement path.
    compose::write_output(
        rgba,
        w,
        h,
        params,
        &segment_result,
        did_refine,
        protect,
        scratch,
    );
}

/// Side-channel data the primary segmenter hands to the compose step
/// (e.g. detected bg color for despill).
#[derive(Copy, Clone, Debug)]
pub enum SegmentResult {
    Chroma { bg_oklab: [f32; 3] },
    GrabCut,
}
