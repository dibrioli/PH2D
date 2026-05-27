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
    try_run_pipeline(rgba, w, h, params, protect, force_remove, scratch).expect(
        "run_pipeline: buffer shape mismatch (use try_run_pipeline for recoverable callers)",
    );
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
    /// Audit T1.6 R9 T1-C1 (HANDOFF_bgremoval_audit_carryovers §2.1):
    /// `w × h × 4` (rgba) or `w × h × 3` (color guide) overflows
    /// `usize`. PH2D ships 64-bit so this requires absurd dimensions
    /// (~`u32::MAX` on each axis — `n * 4 ≈ 2^66 > 2^64`), but the
    /// variant exists for defense-in-depth on a hypothetical 32-bit
    /// port and to surface a clean error before `scratch::ensure`
    /// would panic inside `Vec::resize` (OOM) or wrap silently. The
    /// caller surfaces this the same way as `BufferShape` — clear
    /// the output buffer and let the next preview tick retry with
    /// sane dimensions.
    DimensionsTooLarge { w: u32, h: u32 },
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
            PipelineError::DimensionsTooLarge { w, h } => write!(
                f,
                "bgremoval pipeline: dimensions {w}×{h} overflow usize (w*h*4 cannot be addressed)"
            ),
        }
    }
}

impl std::error::Error for PipelineError {}

/// Fallible variant of [`run_pipeline`] — returns `Err(PipelineError)`
/// on buffer-shape mismatch instead of panicking. Audit T1.6 R7 J1-4.
///
/// **Audit T1.6 R8 M1-2 — scratch contract on `Err` return:** if
/// this function returns `Err(PipelineError::BufferShape{...})`,
/// the `scratch` argument is left in **whatever state the prior
/// successful call put it in** — `scratch.ensure(w, h, ...)` is
/// deliberately NOT called before the shape checks, because the
/// checks reject before we know the geometry is consistent with
/// the scratch's prior `(w, h)`. The caller MUST NOT assume scratch
/// matches the new `(w, h)` after an `Err`. The next call (after
/// the caller fixes the buffer shape) re-runs `scratch.ensure` and
/// grows the internal vectors as needed — safe by construction.
/// A retry with a LARGER image immediately after an Err is fine;
/// reading `scratch.output_rgba` between Err and the retry would
/// see the prior call's pixels.
pub fn try_run_pipeline(
    rgba: &[u8],
    w: u32,
    h: u32,
    params: &BgRemovalParams,
    protect: Option<&[u8]>,
    force_remove: Option<&[u8]>,
    scratch: &mut BgRemovalScratch,
) -> Result<(), PipelineError> {
    // Audit T1.6 R9 T1-C1: validate `w * h * 4` fits in `usize` BEFORE
    // computing the shape-check expected lengths and BEFORE calling
    // `scratch.ensure` (which calls `Vec::resize(n*4, …)` and would
    // panic on overflow / OOM rather than return a clean error). On
    // 64-bit this is theoretical (requires `w, h ≈ u32::MAX`); the
    // check costs one extra mul + branch and eliminates the panic
    // surface for hypothetical 32-bit ports and adversarial inputs.
    // `n*4 >= n*3 >= n`, so validating the largest multiplier covers
    // every internal buffer sizing inside `scratch::ensure`.
    let mask_expected = (w as usize)
        .checked_mul(h as usize)
        .ok_or(PipelineError::DimensionsTooLarge { w, h })?;
    let expected = mask_expected
        .checked_mul(4)
        .ok_or(PipelineError::DimensionsTooLarge { w, h })?;
    if rgba.len() != expected {
        return Err(PipelineError::BufferShape {
            which: "rgba",
            actual: rgba.len(),
            expected,
        });
    }
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

#[cfg(test)]
mod tests {
    use super::*;

    /// Audit T1.6 R8 N1-4: `try_run_pipeline` returns
    /// `Err(PipelineError::BufferShape{...})` (NOT panic) on rgba
    /// length mismatch. The shape-check is the contract that
    /// distinguishes the fallible variant from the panicking one;
    /// without this gate a regression dropping the check would
    /// silently fall through to the chroma::segment indexing path
    /// and panic anyway — defeating the whole point of the J1-4 fix.
    #[test]
    fn try_run_pipeline_returns_err_on_rgba_length_mismatch() {
        let params = BgRemovalParams::default();
        let mut scratch = BgRemovalScratch::default();
        // Declare 4x4 (expected 64 bytes) but pass only 12 bytes.
        let too_small = vec![0u8; 12];
        let res = try_run_pipeline(&too_small, 4, 4, &params, None, None, &mut scratch);
        match res {
            Err(PipelineError::BufferShape {
                which,
                actual,
                expected,
            }) => {
                assert_eq!(which, "rgba");
                assert_eq!(actual, 12);
                assert_eq!(expected, 64);
            }
            other => panic!("expected BufferShape rgba err, got {other:?}"),
        }
    }

    /// Audit T1.6 R8 N1-4: protect mask length mismatch also routes
    /// through the fallible Err arm.
    #[test]
    fn try_run_pipeline_returns_err_on_protect_mask_mismatch() {
        let params = BgRemovalParams::default();
        let mut scratch = BgRemovalScratch::default();
        let rgba = vec![0u8; 4 * 4 * 4]; // 64 bytes, ok
        let bad_protect = vec![0u8; 5]; // expected 16
        let res = try_run_pipeline(&rgba, 4, 4, &params, Some(&bad_protect), None, &mut scratch);
        match res {
            Err(PipelineError::BufferShape {
                which,
                actual,
                expected,
            }) => {
                assert_eq!(which, "protect");
                assert_eq!(actual, 5);
                assert_eq!(expected, 16);
            }
            other => panic!("expected BufferShape protect err, got {other:?}"),
        }
    }

    /// Audit T1.6 R8 N1-4: force_remove mask length mismatch also
    /// routes through the fallible Err arm.
    #[test]
    fn try_run_pipeline_returns_err_on_force_remove_mask_mismatch() {
        let params = BgRemovalParams::default();
        let mut scratch = BgRemovalScratch::default();
        let rgba = vec![0u8; 4 * 4 * 4];
        let bad_force = vec![0u8; 3];
        let res = try_run_pipeline(&rgba, 4, 4, &params, None, Some(&bad_force), &mut scratch);
        match res {
            Err(PipelineError::BufferShape {
                which,
                actual,
                expected,
            }) => {
                assert_eq!(which, "force_remove");
                assert_eq!(actual, 3);
                assert_eq!(expected, 16);
            }
            other => panic!("expected BufferShape force_remove err, got {other:?}"),
        }
    }

    /// Audit T1.6 R8 N1-4: well-formed call returns `Ok(())`.
    #[test]
    fn try_run_pipeline_returns_ok_on_well_formed_input() {
        let params = BgRemovalParams::default();
        let mut scratch = BgRemovalScratch::default();
        let rgba = vec![0u8; 4 * 4 * 4];
        let res = try_run_pipeline(&rgba, 4, 4, &params, None, None, &mut scratch);
        assert!(res.is_ok(), "well-formed input must return Ok, got {res:?}");
    }

    /// Audit T1.6 R9 T1-C1: dimensions whose `w * h * 4` overflows
    /// `usize` route through `Err(DimensionsTooLarge)` instead of
    /// panicking inside `scratch::ensure`'s `Vec::resize`. The gate
    /// is the validation BEFORE `scratch.ensure` is called — without
    /// it, `(u32::MAX as usize) * (u32::MAX as usize) * 4` wraps to
    /// a small number on 64-bit (`(2^32-1)^2 * 4 mod 2^64`) and
    /// `Vec::resize` silently allocates a tiny buffer; subsequent
    /// indexing in chroma::segment then reads OOB.
    #[test]
    fn try_run_pipeline_rejects_dimensions_that_overflow_usize() {
        let params = BgRemovalParams::default();
        let mut scratch = BgRemovalScratch::default();
        // `u32::MAX * u32::MAX = 2^64 - 2^33 + 1`, which fits usize on
        // 64-bit; the overflow is on the next `* 4` step. The empty
        // rgba slice is intentional — the dimension check must reject
        // before any shape check or scratch allocation.
        let res = try_run_pipeline(&[], u32::MAX, u32::MAX, &params, None, None, &mut scratch);
        match res {
            Err(PipelineError::DimensionsTooLarge { w, h }) => {
                assert_eq!(w, u32::MAX);
                assert_eq!(h, u32::MAX);
            }
            other => panic!("expected DimensionsTooLarge err, got {other:?}"),
        }
        // Scratch must remain untouched (the buffers were never sized
        // to the absurd dimensions). delta_e is the canary; if it
        // resized to anything, the validation came too late.
        assert!(
            scratch.delta_e.is_empty(),
            "scratch.delta_e leaked past the dimension check"
        );
    }
}
