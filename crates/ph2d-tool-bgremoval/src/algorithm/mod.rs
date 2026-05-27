//! Pure algorithms for the bgremoval pipeline.
//!
//! No editor / host state, no Vello, no `Tool` trait — every entry
//! point is a free function consuming RGBA + params + scratch. The
//! orchestrator [`run_pipeline`] chains:
//!
//! ```text
//!   segment (chroma)  →  refine (guided_filter, optional)  →  compose
//! ```
//!
//! Each stage writes into a named field on
//! [`super::scratch::BgRemovalScratch`] so the per-frame thumbnail
//! re-runs reuse allocations (HR-3).

pub mod chroma;
pub mod compose;
pub mod guided_filter;
pub mod islands;
pub mod silhouette;

use super::params::BgRemovalParams;
use super::scratch::BgRemovalScratch;

/// Run the full background-removal pipeline on `rgba` (size `w * h * 4`,
/// RGBA8, straight-alpha) and write the result into
/// `scratch.output_rgba`. The output is the same shape as the input
/// with the alpha channel modified to reflect the segmentation.
///
/// `protect` is an optional freehand foreground-protection mask aligned
/// to the **same `w × h`** as `rgba` (one byte/pixel, `>= 128` =
/// protected / forced-foreground). When present, protected pixels are
/// forced opaque in the final compose. Pass `None` when no region is
/// painted. The caller (the tool) is responsible for resampling its
/// source-resolution mask to `(w, h)` first.
///
/// # Panics
/// Panics if `rgba.len() != (w * h * 4) as usize`, or if `protect` is
/// `Some` and its length is not `w * h`.
///
/// **Audit T1.6 R7 J1-4:** the panicking variant exists for internal
/// callers that statically guarantee buffer-length invariants (the
/// tool's own preview / commit paths, where the mask is allocated by
/// the same crate from `(w, h)`). For paths that source buffers from
/// less-controlled places (a future GPU readback, a file-import
/// pipeline that batches sprites with mixed dimensions, the color-
/// equalization live-bake collector that's seen `BufferShapeMismatch`
/// regressions historically), use [`try_run_pipeline`] which returns
/// `Err(PipelineError::BufferShape { … })` instead — the render
/// thread stays alive, and the caller surfaces a `Toast::error` so
/// the user sees what went wrong instead of losing unsaved work in
/// every other tool to a hard process crash.
pub fn run_pipeline(
    rgba: &[u8],
    w: u32,
    h: u32,
    params: &BgRemovalParams,
    protect: Option<&[u8]>,
    force_remove: Option<&[u8]>,
    scratch: &mut BgRemovalScratch,
) {
    try_run_pipeline(rgba, w, h, params, protect, force_remove, scratch)
        .expect("run_pipeline: buffer shape mismatch (use try_run_pipeline for recoverable callers)");
}

/// Error variants returned by [`try_run_pipeline`]. Audit T1.6 R7 J1-4.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum PipelineError {
    /// One of the input buffers has the wrong length for the declared
    /// `(w, h)`. `which` identifies the buffer (`"rgba"`, `"protect"`,
    /// `"force_remove"`); `actual` / `expected` are byte lengths.
    BufferShape {
        which: &'static str,
        actual: usize,
        expected: usize,
    },
}

impl std::fmt::Display for PipelineError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PipelineError::BufferShape {
                which,
                actual,
                expected,
            } => write!(
                f,
                "bgremoval pipeline: {which} buffer length mismatch (was {actual}, expected {expected})"
            ),
        }
    }
}

impl std::error::Error for PipelineError {}

/// Fallible variant of [`run_pipeline`] — returns `Err(PipelineError)`
/// on buffer-shape mismatch instead of panicking. Audit T1.6 R7 J1-4.
pub fn try_run_pipeline(
    rgba: &[u8],
    w: u32,
    h: u32,
    params: &BgRemovalParams,
    protect: Option<&[u8]>,
    force_remove: Option<&[u8]>,
    scratch: &mut BgRemovalScratch,
) -> Result<(), PipelineError> {
    let expected = (w as usize) * (h as usize) * 4;
    if rgba.len() != expected {
        return Err(PipelineError::BufferShape {
            which: "rgba",
            actual: rgba.len(),
            expected,
        });
    }
    let mask_expected = (w as usize) * (h as usize);
    if let Some(pm) = protect
        && pm.len() != mask_expected
    {
        return Err(PipelineError::BufferShape {
            which: "protect",
            actual: pm.len(),
            expected: mask_expected,
        });
    }
    if let Some(fr) = force_remove
        && fr.len() != mask_expected
    {
        return Err(PipelineError::BufferShape {
            which: "force_remove",
            actual: fr.len(),
            expected: mask_expected,
        });
    }

    scratch.ensure(w, h, params.refinement.color_guide);

    // Step 1 — chroma segmentation. Writes scratch.mask (binary 0/255)
    // and scratch.delta_e. Returns the side-channel SegmentResult so
    // compose can despill against the detected bg.
    let segment_result =
        chroma::segment(rgba, w, h, &params.chroma, &params.extra_bg_colors, scratch);

    // Step 1.5 — "Add area" injection (Enio 2026-05-27 "a área nova
    // não é sujeita a ajustes finais com os sliders"). The user's
    // flood-filled connected region enters the pipeline HERE, as hard
    // background — `mask[i] = 0` + `delta_e[i] = 0` — instead of being
    // capped at the end of compose. That puts every force-removed
    // pixel through the SAME downstream path as the auto-detected bg:
    // the Refine guided filter smooths its edge, Grow morphology
    // grows / shrinks it, despill scrubs colour halos, bleed_edges
    // fills the transparent collar. Result: the destructive area is
    // now fully subject to every basal slider (Tolerance / Feather /
    // Refine / Grow), exactly like the rest of the image.
    if let Some(fr) = force_remove {
        // Audit T1.6 R6 clippy: needless_range_loop fix.
        for (idx, &flag) in fr.iter().enumerate().take((w as usize) * (h as usize)) {
            if flag > 0 {
                scratch.mask[idx] = 0;
                scratch.delta_e[idx] = 0.0;
            }
        }
    }

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
    // opaque regardless of the refinement path. `force_remove` is no
    // longer a final cap — the injection at Step 1.5 above made it part
    // of the regular pipeline, so we pass `None` here.
    compose::write_output(
        rgba,
        w,
        h,
        params,
        &segment_result,
        did_refine,
        protect,
        None,
        scratch,
    );
    Ok(())
}

/// Side-channel data the chroma segmenter hands to the compose step
/// (the detected bg colour, for despill).
#[derive(Copy, Clone, Debug)]
pub enum SegmentResult {
    Chroma { bg_oklab: [f32; 3] },
}
