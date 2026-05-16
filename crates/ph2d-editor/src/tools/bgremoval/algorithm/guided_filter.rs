//! Guided Image Filter (He, Sun, Tang — ECCV 2010, TPAMI 2013) used
//! as a post-process refinement step on the binary mask produced by
//! `chroma::segment` or `grabcut::segment`.
//!
//! ## Algorithm (gray guide, vanilla GF)
//!
//! Given a guide image `I` (linearised sRGB luma) and an input
//! mask `p` (binary 0 or 1 per pixel), the filter computes a
//! soft alpha `q` per pixel:
//!
//! ```text
//!   mean_I  = box(I, r)
//!   mean_p  = box(p, r)
//!   mean_II = box(I·I, r)        var_I  = mean_II − mean_I²
//!   mean_Ip = box(I·p, r)        cov_Ip = mean_Ip − mean_I · mean_p
//!   a       = cov_Ip / (var_I + ε)
//!   b       = mean_p − a · mean_I
//!   mean_a  = box(a, r)
//!   mean_b  = box(b, r)
//!   q       = mean_a · I + mean_b
//! ```
//!
//! Window radius `r = params.radius`; ε from `params.epsilon`
//! (default 1e-7, matting-paper-grade). Box filter is implemented
//! as **separable rolling sum** — two 1D passes, O(N·r) total
//! (would be O(N) with a true prefix-sum rolling implementation,
//! but the naive variant is simpler and fast enough on the
//! `MAX_INTERNAL_DIM = 1024` codepath).
//!
//! ## Implementation choices
//!
//! - **Gray guide v1**: RGB → linear → Rec.709 luma. Color guide
//!   (3×3 cov solve per pixel, ~3× cost, better edges on
//!   chroma-similar / luma-distinct boundaries) is a future
//!   addition — pinned at `params.color_guide` for the panel UX.
//! - **No Fast GF wrapper v1**: filter runs at the input dims
//!   directly. For sprite-editor scales this is fine; the
//!   `params.boundary_only` opt that limits processing to the
//!   ±2r mask-boundary band is also future.
//! - **HR-3**: every working buffer comes from `BgRemovalScratch`
//!   (`box_buf_a/b/c/d/e` + `guide_f32` + `alpha_f32`); inner
//!   loops do not allocate. The pipeline reuses 5 work buffers
//!   in a sequential lifecycle — see comments on `refine`.
//! - **HR-5**: sum reductions inside the box filter walk pixels
//!   in row-major order, no `f32::sum` on unordered iterators,
//!   no rayon. Output is bit-identical across runs / platforms.

use super::super::params::GuidedFilterParams;
use super::super::scratch::BgRemovalScratch;

// ---------------------------------------------------------------
// Public entry point
// ---------------------------------------------------------------

/// Refine the binary `scratch.mask` (0/255) into a soft alpha in
/// `scratch.alpha_f32` (∈ [0, 1]) using `rgba` as the colour
/// guide.
///
/// Caller guarantees `params.radius > 0` — the orchestrator
/// (`super::run_pipeline`) skips this call entirely when radius
/// is zero.
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

    if n == 0 || w == 0 || h == 0 {
        return;
    }

    let r = params.radius as usize;
    let eps = params.epsilon.max(1e-12);
    let w_us = w as usize;
    let h_us = h as usize;

    // 0. Materialise inputs.
    //    I → guide_f32 (immutable from here on).
    //    p → alpha_f32 (transient: will be overwritten with q at
    //                   the end of the pipeline).
    rgba_to_linear_luma(rgba, n, &mut scratch.guide_f32);
    mask_to_f32(&scratch.mask, n, &mut scratch.alpha_f32);

    // Local-rename aliases for readability. The scratch borrows
    // below are disjoint by field, so the borrow checker is happy.
    let i_buf: &[f32] = &scratch.guide_f32[..n];
    let p_buf: &[f32] = &scratch.alpha_f32[..n];

    // 1. mean_I = box(I) → box_buf_a.  (tmp: box_buf_e)
    separable_box_filter(
        i_buf,
        w_us,
        h_us,
        r,
        &mut scratch.box_buf_e[..n],
        &mut scratch.box_buf_a[..n],
    );

    // 2. mean_p = box(p) → box_buf_b.  (tmp: box_buf_e)
    separable_box_filter(
        p_buf,
        w_us,
        h_us,
        r,
        &mut scratch.box_buf_e[..n],
        &mut scratch.box_buf_b[..n],
    );

    // 3. I·p → box_buf_c. Then mean_Ip = box(box_buf_c) → box_buf_d.
    for (i, slot) in scratch.box_buf_c.iter_mut().take(n).enumerate() {
        *slot = i_buf[i] * p_buf[i];
    }
    // Box source must differ from tmp; tmp differs from dst — we
    // do the call by snapshotting the src into a local slice and
    // borrowing the rest from scratch.
    {
        let (c_slice, e_slice, d_slice) = three_disjoint_slices_mut(
            &mut scratch.box_buf_c,
            &mut scratch.box_buf_e,
            &mut scratch.box_buf_d,
            n,
        );
        separable_box_filter(c_slice, w_us, h_us, r, e_slice, d_slice);
    }

    // 4. cov_Ip = mean_Ip − mean_I · mean_p → box_buf_d (in place).
    for i in 0..n {
        scratch.box_buf_d[i] -= scratch.box_buf_a[i] * scratch.box_buf_b[i];
    }

    // 5. I·I → box_buf_c (overwrite). Then mean_II = box → alpha_f32
    //    (overwriting p — we don't need p any more, mean_p is in
    //    box_buf_b). Then var_I = mean_II − mean_I² → box_buf_c
    //    (overwrite I·I).
    for (i, slot) in scratch.box_buf_c.iter_mut().take(n).enumerate() {
        let v = i_buf[i];
        *slot = v * v;
    }
    {
        let (c_slice, e_slice, alpha_slice) = three_disjoint_slices_mut(
            &mut scratch.box_buf_c,
            &mut scratch.box_buf_e,
            &mut scratch.alpha_f32,
            n,
        );
        separable_box_filter(c_slice, w_us, h_us, r, e_slice, alpha_slice);
    }
    for i in 0..n {
        let m = scratch.box_buf_a[i];
        // `var_I = mean_II − mean_I²` is mathematically ≥ 0, but FP
        // cancellation on near-constant patches can produce small
        // negative values. Floor at 0 so that `var_I + ε > 0`
        // always holds — without this, a user-set ε ≤ |var_I| from
        // cancellation would yield a tiny negative denominator and
        // a huge negative `a` (M3 audit finding 1, 2026-05-16).
        scratch.box_buf_c[i] = (scratch.alpha_f32[i] - m * m).max(0.0);
    }

    // 6. a = cov_Ip / (var_I + ε) → box_buf_d (overwrite cov_Ip).
    for i in 0..n {
        scratch.box_buf_d[i] /= scratch.box_buf_c[i] + eps;
    }

    // 7. b = mean_p − a · mean_I → box_buf_c (overwrite var_I).
    //    After this, we drop mean_I (box_buf_a) and mean_p (box_buf_b);
    //    the next step writes mean_a / mean_b into them.
    for i in 0..n {
        scratch.box_buf_c[i] = scratch.box_buf_b[i] - scratch.box_buf_d[i] * scratch.box_buf_a[i];
    }

    // 8. mean_a = box(a) → box_buf_a (overwrite mean_I).
    {
        let (d_slice, e_slice, a_slice) = three_disjoint_slices_mut(
            &mut scratch.box_buf_d,
            &mut scratch.box_buf_e,
            &mut scratch.box_buf_a,
            n,
        );
        separable_box_filter(d_slice, w_us, h_us, r, e_slice, a_slice);
    }

    // 9. mean_b = box(b) → box_buf_b (overwrite mean_p).
    {
        let (c_slice, e_slice, b_slice) = three_disjoint_slices_mut(
            &mut scratch.box_buf_c,
            &mut scratch.box_buf_e,
            &mut scratch.box_buf_b,
            n,
        );
        separable_box_filter(c_slice, w_us, h_us, r, e_slice, b_slice);
    }

    // 10. q = mean_a · I + mean_b → alpha_f32 (overwrite mean_II).
    for (i, slot) in scratch.alpha_f32.iter_mut().take(n).enumerate() {
        let v = scratch.box_buf_a[i] * i_buf[i] + scratch.box_buf_b[i];
        *slot = v.clamp(0.0, 1.0);
    }
}

// ---------------------------------------------------------------
// Input materialisation helpers
// ---------------------------------------------------------------

/// Convert sRGB byte to linear-light `[0, 1]` (IEC 61966-2-1).
#[inline(always)]
fn srgb_to_linear(c: u8) -> f32 {
    let cf = (c as f32) / 255.0;
    if cf <= 0.04045 {
        cf / 12.92
    } else {
        ((cf + 0.055) / 1.055).powf(2.4)
    }
}

/// Fill `out[..n]` with the Rec.709 luminance of each input
/// RGBA8 pixel, converted to linear-light. Alpha is ignored —
/// transparent pixels still contribute their (formally undefined)
/// RGB to the guide; the chroma backend that produced the mask
/// already filters those upstream.
fn rgba_to_linear_luma(rgba: &[u8], n: usize, out: &mut Vec<f32>) {
    if out.len() < n {
        out.resize(n, 0.0);
    }
    for (i, slot) in out.iter_mut().take(n).enumerate() {
        let base = i * 4;
        let r = srgb_to_linear(rgba[base]);
        let g = srgb_to_linear(rgba[base + 1]);
        let b = srgb_to_linear(rgba[base + 2]);
        *slot = 0.2126 * r + 0.7152 * g + 0.0722 * b;
    }
}

/// Convert binary mask `[0, 255]` to `f32 [0, 1]`.
fn mask_to_f32(mask: &[u8], n: usize, out: &mut Vec<f32>) {
    if out.len() < n {
        out.resize(n, 0.0);
    }
    for (i, slot) in out.iter_mut().take(n).enumerate() {
        *slot = (mask[i] as f32) * (1.0 / 255.0);
    }
}

// ---------------------------------------------------------------
// Separable box filter
// ---------------------------------------------------------------

/// 2-D box filter with radius `r`, separable into a horizontal
/// then a vertical 1-D pass. Boundary handling: clamp-to-edge
/// (window count varies near edges).
///
/// `src`, `tmp`, `dst` must be 3 distinct slices of length
/// `w * h`. `tmp` holds the horizontal-pass output; `dst` the
/// final 2-D filtered result. Caller is responsible for distinct
/// aliasing — wired through `three_disjoint_slices_mut` in
/// `refine`.
fn separable_box_filter(
    src: &[f32],
    w: usize,
    h: usize,
    r: usize,
    tmp: &mut [f32],
    dst: &mut [f32],
) {
    debug_assert_eq!(src.len(), w * h);
    debug_assert_eq!(tmp.len(), w * h);
    debug_assert_eq!(dst.len(), w * h);

    let r_i = r as i32;
    let w_i = w as i32;
    let h_i = h as i32;

    // Horizontal pass: tmp = horizontal-box(src).
    for y in 0..h {
        let row = y * w;
        for x in 0..w {
            let xl = ((x as i32) - r_i).max(0) as usize;
            let xr = ((x as i32) + r_i).min(w_i - 1) as usize;
            let mut sum = 0.0f32;
            for k in xl..=xr {
                sum += src[row + k];
            }
            let count = (xr - xl + 1) as f32;
            tmp[row + x] = sum / count;
        }
    }

    // Vertical pass: dst = vertical-box(tmp). Outer x iterates by
    // index since we need `x` for both `tmp[k*w + x]` reads and
    // `dst[y*w + x]` writes spanning multiple rows.
    #[allow(clippy::needless_range_loop)]
    for x in 0..w {
        for y in 0..h {
            let yl = ((y as i32) - r_i).max(0) as usize;
            let yr = ((y as i32) + r_i).min(h_i - 1) as usize;
            let mut sum = 0.0f32;
            for k in yl..=yr {
                sum += tmp[k * w + x];
            }
            let count = (yr - yl + 1) as f32;
            dst[y * w + x] = sum / count;
        }
    }
}

/// Borrow three disjoint `&mut [f32]` slices simultaneously from
/// three separate `Vec<f32>` fields of `BgRemovalScratch`. Pure
/// helper — no aliasing math; the slices come from distinct Vecs
/// so Rust trivially proves disjointness.
fn three_disjoint_slices_mut<'a>(
    a: &'a mut [f32],
    b: &'a mut [f32],
    c: &'a mut [f32],
    n: usize,
) -> (&'a mut [f32], &'a mut [f32], &'a mut [f32]) {
    (&mut a[..n], &mut b[..n], &mut c[..n])
}

// ---------------------------------------------------------------
// Tests
// ---------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tools::bgremoval::params::GuidedFilterParams;

    fn default_params() -> GuidedFilterParams {
        GuidedFilterParams::default()
    }

    fn make_rgba(w: u32, h: u32, fill: [u8; 3]) -> Vec<u8> {
        let n = (w as usize) * (h as usize);
        let mut rgba = vec![0u8; n * 4];
        for i in 0..n {
            rgba[i * 4] = fill[0];
            rgba[i * 4 + 1] = fill[1];
            rgba[i * 4 + 2] = fill[2];
            rgba[i * 4 + 3] = 255;
        }
        rgba
    }

    // --- Linear luma helper ---------------------------------------------

    #[test]
    fn srgb_to_linear_endpoints() {
        assert!(srgb_to_linear(0).abs() < 1e-6);
        assert!((srgb_to_linear(255) - 1.0).abs() < 1e-5);
    }

    #[test]
    fn rgba_to_linear_luma_pure_red_matches_coefficient() {
        let rgba = make_rgba(2, 1, [255, 0, 0]);
        let mut out = Vec::new();
        rgba_to_linear_luma(&rgba, 2, &mut out);
        // Pure linear red ⇒ luma = 0.2126.
        assert!((out[0] - 0.2126).abs() < 1e-4);
        assert!((out[1] - 0.2126).abs() < 1e-4);
    }

    #[test]
    fn rgba_to_linear_luma_pure_green_matches_coefficient() {
        let rgba = make_rgba(1, 1, [0, 255, 0]);
        let mut out = Vec::new();
        rgba_to_linear_luma(&rgba, 1, &mut out);
        assert!((out[0] - 0.7152).abs() < 1e-4);
    }

    #[test]
    fn mask_to_f32_endpoints() {
        let mask = vec![0u8, 128, 255];
        let mut out = Vec::new();
        mask_to_f32(&mask, 3, &mut out);
        assert!(out[0].abs() < 1e-6);
        assert!((out[1] - 128.0 / 255.0).abs() < 1e-5);
        assert!((out[2] - 1.0).abs() < 1e-5);
    }

    // --- Box filter -----------------------------------------------------

    #[test]
    fn box_filter_constant_input_preserves_value() {
        let w = 8;
        let h = 8;
        let src = vec![0.5f32; w * h];
        let mut tmp = vec![0.0; w * h];
        let mut dst = vec![0.0; w * h];
        separable_box_filter(&src, w, h, 2, &mut tmp, &mut dst);
        for v in &dst {
            assert!((v - 0.5).abs() < 1e-5);
        }
    }

    #[test]
    fn box_filter_radius_zero_is_identity() {
        let w = 4;
        let h = 4;
        let src: Vec<f32> = (0..16).map(|i| i as f32).collect();
        let mut tmp = vec![0.0; 16];
        let mut dst = vec![0.0; 16];
        separable_box_filter(&src, w, h, 0, &mut tmp, &mut dst);
        for i in 0..16 {
            assert_eq!(dst[i], src[i]);
        }
    }

    #[test]
    fn box_filter_step_function_smooths_edge() {
        // 1-D-like input: left half 0, right half 1.
        let w = 8;
        let h = 1;
        let mut src = vec![0.0; w * h];
        for slot in src.iter_mut().take(8).skip(4) {
            *slot = 1.0;
        }
        let mut tmp = vec![0.0; w * h];
        let mut dst = vec![0.0; w * h];
        separable_box_filter(&src, w, h, 2, &mut tmp, &mut dst);
        // Far-left stays near 0.
        assert!(dst[0] < 0.1);
        // Far-right stays near 1.
        assert!(dst[7] > 0.9);
        // Middle is intermediate (smooth transition).
        assert!(dst[3] > 0.0 && dst[3] < 0.6);
        assert!(dst[4] > 0.4 && dst[4] < 1.0);
    }

    // --- End-to-end refine ----------------------------------------------

    #[test]
    fn refine_uniform_mask_one_stays_one() {
        // All-fg mask + any guide ⇒ output should stay all-fg.
        let w = 16;
        let h = 16;
        let rgba = make_rgba(w, h, [128, 128, 128]);
        let mut scratch = BgRemovalScratch::default();
        scratch.ensure(w, h, false);
        for v in &mut scratch.mask[..(w * h) as usize] {
            *v = 255;
        }
        refine(&rgba, w, h, &default_params(), &mut scratch);
        for v in &scratch.alpha_f32[..(w * h) as usize] {
            assert!((v - 1.0).abs() < 0.01, "got {v}, expected ~1.0");
        }
    }

    #[test]
    fn refine_uniform_mask_zero_stays_zero() {
        let w = 16;
        let h = 16;
        let rgba = make_rgba(w, h, [128, 128, 128]);
        let mut scratch = BgRemovalScratch::default();
        scratch.ensure(w, h, false);
        // Mask is all-0 by default.
        refine(&rgba, w, h, &default_params(), &mut scratch);
        for v in &scratch.alpha_f32[..(w * h) as usize] {
            assert!(v.abs() < 0.01, "got {v}, expected ~0.0");
        }
    }

    #[test]
    fn refine_aliased_step_mask_softens_edge() {
        // 32×1 with a 0/1 step at x=16. Guide is a matching step
        // (left dark, right bright) so the GF should preserve the
        // edge AND not bleed too far. We check: alpha goes from
        // ~0 to ~1, monotonically across the boundary band.
        let w: u32 = 32;
        let h: u32 = 1;
        let mut rgba = vec![0u8; (w * h) as usize * 4];
        for i in 0..(w as usize) {
            let val = if i < 16 { 30u8 } else { 220u8 };
            rgba[i * 4] = val;
            rgba[i * 4 + 1] = val;
            rgba[i * 4 + 2] = val;
            rgba[i * 4 + 3] = 255;
        }
        let mut scratch = BgRemovalScratch::default();
        scratch.ensure(w, h, false);
        for i in 0..16 {
            scratch.mask[i] = 0;
        }
        for i in 16..32 {
            scratch.mask[i] = 255;
        }
        let mut params = default_params();
        params.radius = 4;
        refine(&rgba, w, h, &params, &mut scratch);
        // Endpoints stay close to their original values.
        assert!(scratch.alpha_f32[0] < 0.1);
        assert!(scratch.alpha_f32[31] > 0.9);
        // Strict monotonicity across the boundary.
        for i in 1..32 {
            assert!(
                scratch.alpha_f32[i] >= scratch.alpha_f32[i - 1] - 1e-3,
                "non-monotonic at {i}: {} > {}",
                scratch.alpha_f32[i - 1],
                scratch.alpha_f32[i],
            );
        }
    }

    #[test]
    fn refine_output_is_clamped_to_unit_interval() {
        // GF's analytic output can drift slightly outside [0, 1]
        // on aliased inputs; the final clamp brings it back.
        let w = 8;
        let h = 8;
        let rgba = make_rgba(w, h, [200, 30, 30]);
        let mut scratch = BgRemovalScratch::default();
        scratch.ensure(w, h, false);
        // Checkerboard mask — adversarial for clamping.
        for i in 0..(w * h) as usize {
            scratch.mask[i] = if (i / 2 + i / w as usize).is_multiple_of(2) {
                255
            } else {
                0
            };
        }
        refine(&rgba, w, h, &default_params(), &mut scratch);
        for v in &scratch.alpha_f32[..(w * h) as usize] {
            assert!(*v >= 0.0 && *v <= 1.0, "{v} out of [0,1]");
        }
    }

    #[test]
    fn refine_determinism_two_runs_match_bit_for_bit() {
        let w = 16;
        let h = 16;
        let rgba = make_rgba(w, h, [128, 60, 200]);
        let mut s1 = BgRemovalScratch::default();
        s1.ensure(w, h, false);
        for i in 0..(w * h) as usize / 2 {
            s1.mask[i] = 255;
        }
        let mut s2 = s1.clone();
        refine(&rgba, w, h, &default_params(), &mut s1);
        refine(&rgba, w, h, &default_params(), &mut s2);
        for i in 0..(w * h) as usize {
            assert_eq!(s1.alpha_f32[i].to_bits(), s2.alpha_f32[i].to_bits());
        }
    }

    #[test]
    fn refine_does_not_panic_on_empty_image() {
        let mut scratch = BgRemovalScratch::default();
        scratch.ensure(0, 0, false);
        let mut params = default_params();
        params.radius = 5;
        refine(&[], 0, 0, &params, &mut scratch);
    }

    // --- Audit-driven coverage (2026-05-16) ----------------------------

    #[test]
    fn refine_uniform_guide_with_checker_mask_does_not_nan() {
        // Audit finding 5: with a fully-uniform guide, var_I ≈ 0
        // for every window; only ε in the denominator. Combined
        // with a non-trivial (checker) mask, this exercises the
        // `var_I = 0` branch + `a = 0 / ε` numerator-zero path
        // that the prior uniform-mask tests did not.
        let w = 16;
        let h = 16;
        let rgba = make_rgba(w, h, [128, 128, 128]);
        let mut scratch = BgRemovalScratch::default();
        scratch.ensure(w, h, false);
        for i in 0..(w * h) as usize {
            scratch.mask[i] = if i.is_multiple_of(2) { 255 } else { 0 };
        }
        refine(&rgba, w, h, &default_params(), &mut scratch);
        // Every output must be finite and in [0, 1].
        for v in &scratch.alpha_f32[..(w * h) as usize] {
            assert!(v.is_finite() && (0.0..=1.0).contains(v), "got {v}");
        }
        // Mean alpha should track mask density (~0.5).
        let n = (w * h) as f32;
        let mean: f32 = scratch.alpha_f32[..n as usize].iter().sum::<f32>() / n;
        assert!((mean - 0.5).abs() < 0.05, "mean alpha = {mean}");
    }

    #[test]
    fn refine_radius_larger_than_image_degrades_to_global_mean() {
        // Audit finding 5: r > max(w, h) should not panic. Every
        // window covers the whole image, so q ≈ global mean(p) at
        // every pixel.
        let w = 8;
        let h = 8;
        let rgba = make_rgba(w, h, [100, 150, 200]);
        let mut scratch = BgRemovalScratch::default();
        scratch.ensure(w, h, false);
        // 25 % of pixels fg.
        for i in 0..16 {
            scratch.mask[i] = 255;
        }
        let mut p = default_params();
        p.radius = 100; // ≫ max(w, h) = 8
        refine(&rgba, w, h, &p, &mut scratch);
        let n = (w * h) as f32;
        let mean: f32 = scratch.alpha_f32[..n as usize].iter().sum::<f32>() / n;
        // Mean of output ≈ mean of mask = 16/64 = 0.25 (within FP slack).
        assert!(
            (mean - 0.25).abs() < 0.02,
            "got mean {mean}, expected ~0.25"
        );
        // All outputs near the global mean (not at the extremes).
        for v in &scratch.alpha_f32[..n as usize] {
            assert!(v.is_finite() && (0.0..=1.0).contains(v));
        }
    }

    #[test]
    fn refine_asymmetric_non_square_image_runs_clean() {
        // Audit finding 5: non-square exercises the H/V pass
        // count-divisor difference per axis.
        let w: u32 = 24;
        let h: u32 = 8;
        let rgba = make_rgba(w, h, [60, 180, 60]);
        let mut scratch = BgRemovalScratch::default();
        scratch.ensure(w, h, false);
        // Left half fg, right half bg.
        for y in 0..h as usize {
            for x in 0..(w as usize) / 2 {
                scratch.mask[y * w as usize + x] = 255;
            }
        }
        let mut p = default_params();
        p.radius = 3;
        refine(&rgba, w, h, &p, &mut scratch);
        let n = (w * h) as usize;
        for v in &scratch.alpha_f32[..n] {
            assert!(v.is_finite() && (0.0..=1.0).contains(v));
        }
        // Far-left stays ~1, far-right stays ~0.
        assert!(scratch.alpha_f32[0] > 0.9);
        assert!(scratch.alpha_f32[w as usize - 1] < 0.1);
    }

    #[test]
    fn refine_scratch_capacity_does_not_shrink_across_calls() {
        let w = 32;
        let h = 32;
        let rgba = make_rgba(w, h, [100, 100, 100]);
        let mut scratch = BgRemovalScratch::default();
        scratch.ensure(w, h, false);
        refine(&rgba, w, h, &default_params(), &mut scratch);
        let cap_a = scratch.box_buf_a.capacity();
        let cap_e = scratch.box_buf_e.capacity();
        refine(&rgba, w, h, &default_params(), &mut scratch);
        // Capacity is sticky — HR-3.
        assert!(scratch.box_buf_a.capacity() >= cap_a);
        assert!(scratch.box_buf_e.capacity() >= cap_e);
    }
}
