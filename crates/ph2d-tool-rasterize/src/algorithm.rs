//! Rasterize — Mitchell-Netravali resample + flip + rotation in pure
//! Rust (`std`-only). Bakes a sprite's active Transform — scale (with
//! sign / flip), rotation — into its RGBA8 pixel buffer.
//!
//! ## Algorithm
//!
//! 1. **Resample** to `(round(w * |sx|), round(h * |sy|))` via a 1D
//!    Mitchell-Netravali kernel (B = 1/3, C = 1/3 — the canonical
//!    Mitchell 1988 SIGGRAPH choice) applied **separably**: horizontal
//!    pass then vertical pass. Each pass reads 4 source pixels per
//!    destination pixel; total cost per dst pixel is 8 source reads
//!    instead of 16 for a 2D 4×4 kernel.
//! 2. **Flip** axis-aligned mirror if `sign(scale_x) < 0` or
//!    `sign(scale_y) < 0`. Cheap row/col swap.
//! 3. **Rotate** by `rotation_radians` via a single-pass Mitchell-
//!    Netravali sample (4×4 = 16 source reads per dst pixel), into a
//!    new buffer sized to the axis-aligned bounding box of the rotated
//!    source rectangle. Skipped entirely when |rotation| <
//!    [`ROTATION_EPS`].
//!
//! All filtering happens in **premultiplied alpha** float space so
//! transparent regions never bleed colour into opaque neighbours; the
//! final pass un-premultiplies back to straight-alpha RGBA8.
//!
//! ## Kernel
//!
//! Mitchell-Netravali, B = C = 1/3 — Mitchell & Netravali 1988,
//! *Reconstruction Filters in Computer Graphics*, SIGGRAPH'88 pp.
//! 221-228. Support radius 2 (4 source samples per axis); the kernel
//! is a partition of unity at integer offsets so the weights sum to
//! 1.0 for any sub-pixel sample position.
//!
//! ## Edge handling
//!
//! Resample passes clamp source pixel indices to `[0, dim - 1]` (edge
//! replication — the conventional choice for sprite scale, matches
//! `image::imageops::resize`).
//!
//! Rotation passes treat off-source samples as **fully transparent**
//! (premultiplied zero), so the rotated-bbox corners that fall outside
//! the source rectangle stay transparent rather than replicating an
//! edge pixel into them.
//!
//! ## Determinism
//!
//! Pure f32 arithmetic on byte buffers. No platform-specific intrinsics,
//! no GPU readback, no allocator state. Bit-identical across all
//! supported platforms — safe for inclusion in any deterministic asset
//! cooking pipeline (HR-5 implication). Allocation is fine: this is a
//! user-click action, not a HR-3 hot path.

/// Floats below this radians threshold count as "no rotation" — Mitchell
/// rotation pass is skipped and the bounding box stays at `(w, h)`. The
/// pragmatic limit: at 0.001 rad the corner displacement of a 4 K sprite
/// is < 5 px, so a 1e-6 cutoff (~0.000057°) is well below any user-
/// perceivable rotation.
pub const ROTATION_EPS: f32 = 1e-6;

/// Result of [`rasterize`].
#[derive(Clone, Debug)]
pub struct RasterizeResult {
    /// Baked RGBA8 buffer. `pixels.len() == width * height * 4`.
    pub pixels: Vec<u8>,
    /// Output canvas width (= rotated-bbox width when rotation ≠ 0).
    pub width: u32,
    /// Output canvas height (= rotated-bbox height when rotation ≠ 0).
    pub height: u32,
    /// `false` when the input was identity (`|sx| = |sy| = 1`, both
    /// signs positive, `|rotation| < ROTATION_EPS`) and the buffer was
    /// returned unchanged. The shell skips the asset replacement + undo
    /// entry in that case (same convention as `make_square`).
    pub did_change: bool,
}

/// Bake a Transform into an RGBA8 sprite buffer. Returns a fresh buffer;
/// the input is not modified.
///
/// `rgba` must be exactly `width * height * 4` bytes; mismatch panics
/// (same contract as `make_square`).
///
/// ## Parameters
///
/// - `scale_x`, `scale_y` — sprite scale on each axis. The magnitude
///   drives the resample factor; the sign drives the flip. Non-finite
///   values (`NaN`, `±∞`) fall back to `1.0`.
/// - `rotation_radians` — counter-clockwise rotation in radians (math
///   convention: positive θ rotates +X toward +Y). Non-finite values
///   fall back to `0.0`.
///
/// ## Edge cases
///
/// - **Identity transform** (`|sx| = |sy| = 1`, both positive, rotation
///   below [`ROTATION_EPS`]) → returns a copy of `rgba` with the same
///   `(width, height)` and `did_change = false`.
/// - **Zero source dimension** (`width == 0 || height == 0`) → returns a
///   1×1 transparent sentinel (same shape as `make_square`).
/// - **Non-finite parameters** sanitised to the identity values before
///   any arithmetic so the math never propagates `NaN` into output
///   pixels.
pub fn rasterize(
    rgba: &[u8],
    width: u32,
    height: u32,
    scale_x: f32,
    scale_y: f32,
    rotation_radians: f32,
) -> RasterizeResult {
    if width == 0 || height == 0 {
        return degenerate_sentinel();
    }
    assert_eq!(
        rgba.len(),
        (width as usize)
            .checked_mul(height as usize)
            .and_then(|n| n.checked_mul(4))
            .expect("rgba dimensions overflow usize"),
        "rgba buffer length must equal width * height * 4",
    );

    let sx = sanitize_finite(scale_x, 1.0);
    let sy = sanitize_finite(scale_y, 1.0);
    let rot = sanitize_finite(rotation_radians, 0.0);

    let asx = sx.abs();
    let asy = sy.abs();
    let is_identity = (asx - 1.0).abs() < f32::EPSILON
        && (asy - 1.0).abs() < f32::EPSILON
        && sx > 0.0
        && sy > 0.0
        && rot.abs() < ROTATION_EPS;
    if is_identity {
        return RasterizeResult {
            pixels: rgba.to_vec(),
            width,
            height,
            did_change: false,
        };
    }

    let w_s = (((width as f32) * asx).round() as u32).max(1);
    let h_s = (((height as f32) * asy).round() as u32).max(1);

    // All filtering happens in premultiplied float space.
    let src_pre = to_premult_f32(rgba);

    // Stage A — Mitchell-Netravali resample (separable, two passes).
    let mid = if w_s == width {
        src_pre
    } else {
        resample_horizontal_premult(&src_pre, width, height, w_s)
    };
    let mut scaled = if h_s == height {
        mid
    } else {
        resample_vertical_premult(&mid, w_s, height, h_s)
    };

    // Stage B — axis-aligned flips (sign of scale).
    if sx < 0.0 {
        flip_horizontal_premult(&mut scaled, w_s, h_s);
    }
    if sy < 0.0 {
        flip_vertical_premult(&mut scaled, w_s, h_s);
    }

    // Stage C — Mitchell-Netravali rotation (single 4×4 pass into a
    // bbox-sized output buffer).
    let (out_pre, out_w, out_h) = if rot.abs() < ROTATION_EPS {
        (scaled, w_s, h_s)
    } else {
        rotate_mitchell_premult(&scaled, w_s, h_s, rot)
    };

    RasterizeResult {
        pixels: from_premult_u8(&out_pre),
        width: out_w,
        height: out_h,
        did_change: true,
    }
}

fn degenerate_sentinel() -> RasterizeResult {
    RasterizeResult {
        pixels: vec![0, 0, 0, 0],
        width: 1,
        height: 1,
        did_change: true,
    }
}

#[inline]
fn sanitize_finite(v: f32, fallback: f32) -> f32 {
    if v.is_finite() { v } else { fallback }
}

// ---------------------------------------------------------------------
// Mitchell-Netravali kernel (B = C = 1/3).
// ---------------------------------------------------------------------

/// Mitchell-Netravali 1D kernel value at offset `t` (source pixels from
/// the sample point). Support radius 2 — returns 0 for `|t| >= 2`.
///
/// With B = C = 1/3 the two-piece cubic is the canonical "Mitchell"
/// reconstruction filter — the SIGGRAPH'88 compromise between ringing
/// (lower with higher C) and blurring (lower with higher B). At integer
/// offsets the kernel summed across the 4 contributing source pixels
/// equals 1.0 exactly (partition of unity), so this function does **not**
/// need post-normalisation by the caller.
#[inline]
fn mitchell_kernel(t: f32) -> f32 {
    let at = t.abs();
    const B: f32 = 1.0 / 3.0;
    const C: f32 = 1.0 / 3.0;
    if at < 1.0 {
        let cubic = 12.0 - 9.0 * B - 6.0 * C;
        let quad = -18.0 + 12.0 * B + 6.0 * C;
        let cons = 6.0 - 2.0 * B;
        (cubic * at * at * at + quad * at * at + cons) / 6.0
    } else if at < 2.0 {
        let cubic = -B - 6.0 * C;
        let quad = 6.0 * B + 30.0 * C;
        let linear = -12.0 * B - 48.0 * C;
        let cons = 8.0 * B + 24.0 * C;
        (cubic * at * at * at + quad * at * at + linear * at + cons) / 6.0
    } else {
        0.0
    }
}

// ---------------------------------------------------------------------
// Premultiplied alpha conversion (f32 work buffers in [0, 255]).
// ---------------------------------------------------------------------

fn to_premult_f32(rgba: &[u8]) -> Vec<f32> {
    let mut out = vec![0.0f32; rgba.len()];
    let mut i = 0;
    while i < rgba.len() {
        let a_norm = rgba[i + 3] as f32 / 255.0;
        out[i] = rgba[i] as f32 * a_norm;
        out[i + 1] = rgba[i + 1] as f32 * a_norm;
        out[i + 2] = rgba[i + 2] as f32 * a_norm;
        out[i + 3] = rgba[i + 3] as f32;
        i += 4;
    }
    out
}

fn from_premult_u8(premult: &[f32]) -> Vec<u8> {
    let mut out = vec![0u8; premult.len()];
    let mut i = 0;
    while i < premult.len() {
        let a = premult[i + 3];
        let a_clamped = a.clamp(0.0, 255.0);
        let a_norm = a_clamped / 255.0;
        let (r, g, b) = if a_norm > 1e-6 {
            (
                premult[i] / a_norm,
                premult[i + 1] / a_norm,
                premult[i + 2] / a_norm,
            )
        } else {
            (0.0, 0.0, 0.0)
        };
        out[i] = r.clamp(0.0, 255.0).round() as u8;
        out[i + 1] = g.clamp(0.0, 255.0).round() as u8;
        out[i + 2] = b.clamp(0.0, 255.0).round() as u8;
        out[i + 3] = a_clamped.round() as u8;
        i += 4;
    }
    out
}

// ---------------------------------------------------------------------
// Separable Mitchell-Netravali resample.
// ---------------------------------------------------------------------

fn resample_horizontal_premult(src: &[f32], src_w: u32, h: u32, dst_w: u32) -> Vec<f32> {
    let scale_x = dst_w as f32 / src_w as f32;
    let inv_scale = 1.0 / scale_x;
    let mut out = vec![0.0f32; (dst_w as usize) * (h as usize) * 4];
    for y in 0..h {
        let row_in = (y as usize) * (src_w as usize) * 4;
        let row_out = (y as usize) * (dst_w as usize) * 4;
        for xd in 0..dst_w {
            // Pixel-center convention: dst pixel `xd` has centre at
            // `xd + 0.5`; mapped back through the scale gives the src
            // pixel-centre coord.
            let xs_centre = ((xd as f32) + 0.5) * inv_scale - 0.5;
            let xs_floor = xs_centre.floor() as i32;
            let mut acc = [0.0f32; 4];
            for k in -1..=2 {
                let xs = xs_floor + k;
                let t = xs as f32 - xs_centre;
                let w = mitchell_kernel(t);
                let xs_c = xs.clamp(0, src_w as i32 - 1) as usize;
                let i = row_in + xs_c * 4;
                acc[0] += src[i] * w;
                acc[1] += src[i + 1] * w;
                acc[2] += src[i + 2] * w;
                acc[3] += src[i + 3] * w;
            }
            let oi = row_out + (xd as usize) * 4;
            out[oi] = acc[0];
            out[oi + 1] = acc[1];
            out[oi + 2] = acc[2];
            out[oi + 3] = acc[3];
        }
    }
    out
}

fn resample_vertical_premult(src: &[f32], w: u32, src_h: u32, dst_h: u32) -> Vec<f32> {
    let scale_y = dst_h as f32 / src_h as f32;
    let inv_scale = 1.0 / scale_y;
    let mut out = vec![0.0f32; (w as usize) * (dst_h as usize) * 4];
    for yd in 0..dst_h {
        let ys_centre = ((yd as f32) + 0.5) * inv_scale - 0.5;
        let ys_floor = ys_centre.floor() as i32;
        // Pre-compute the 4 row weights once per dst row (separability:
        // they are independent of the column).
        let mut weights = [0.0f32; 4];
        let mut rows = [0usize; 4];
        for (k_idx, k) in (-1..=2).enumerate() {
            let ys = ys_floor + k;
            let t = ys as f32 - ys_centre;
            weights[k_idx] = mitchell_kernel(t);
            let ys_c = ys.clamp(0, src_h as i32 - 1) as usize;
            rows[k_idx] = ys_c * (w as usize) * 4;
        }
        let row_out = (yd as usize) * (w as usize) * 4;
        for x in 0..w {
            let mut acc = [0.0f32; 4];
            for (k_idx, _) in (-1..=2i32).enumerate() {
                let i = rows[k_idx] + (x as usize) * 4;
                let w_k = weights[k_idx];
                acc[0] += src[i] * w_k;
                acc[1] += src[i + 1] * w_k;
                acc[2] += src[i + 2] * w_k;
                acc[3] += src[i + 3] * w_k;
            }
            let oi = row_out + (x as usize) * 4;
            out[oi] = acc[0];
            out[oi + 1] = acc[1];
            out[oi + 2] = acc[2];
            out[oi + 3] = acc[3];
        }
    }
    out
}

// ---------------------------------------------------------------------
// Axis-aligned flips on the premult buffer (in-place).
// ---------------------------------------------------------------------

fn flip_horizontal_premult(buf: &mut [f32], w: u32, h: u32) {
    let stride = (w as usize) * 4;
    for y in 0..(h as usize) {
        let row = y * stride;
        let mut lo = 0usize;
        let mut hi = (w as usize) - 1;
        while lo < hi {
            let li = row + lo * 4;
            let hi_i = row + hi * 4;
            for c in 0..4 {
                buf.swap(li + c, hi_i + c);
            }
            lo += 1;
            hi -= 1;
        }
    }
}

fn flip_vertical_premult(buf: &mut [f32], w: u32, h: u32) {
    let stride = (w as usize) * 4;
    let mut lo = 0usize;
    let mut hi = (h as usize) - 1;
    while lo < hi {
        let row_lo = lo * stride;
        let row_hi = hi * stride;
        for c in 0..stride {
            buf.swap(row_lo + c, row_hi + c);
        }
        lo += 1;
        hi -= 1;
    }
}

// ---------------------------------------------------------------------
// Mitchell-Netravali rotation (single 4×4 pass).
// ---------------------------------------------------------------------

fn rotate_mitchell_premult(src: &[f32], w: u32, h: u32, theta: f32) -> (Vec<f32>, u32, u32) {
    // T1.3.5 cross-OS bit-identical — rasterize bakes geometry into
    // pixels; the choice of sin/cos impl determines downstream pixel
    // values. Routing through libm keeps the bake reproducible across
    // hosts (matters for golden-pixel goldens + cooked-hash gates).
    let (sin_t, cos_t) = libm::sincosf(theta);
    let abs_cos = cos_t.abs();
    let abs_sin = sin_t.abs();
    // Cardinal-angle robustness: at θ ∈ {π/2, π, 3π/2} the f32 trig
    // result is non-zero by ~6e-8, which would inflate `ceil(...)` by
    // an extra row/column. Subtract a small epsilon before ceil so a
    // numerically-integer dimension round-trips. Epsilon = 1e-3 stays
    // comfortably below 1-pixel for any sprite ≤ 1 000 000 px on a side.
    let bb_w = ((((w as f32) * abs_cos + (h as f32) * abs_sin - 1e-3).ceil() as i32).max(1)) as u32;
    let bb_h = ((((w as f32) * abs_sin + (h as f32) * abs_cos - 1e-3).ceil() as i32).max(1)) as u32;
    let mut out = vec![0.0f32; (bb_w as usize) * (bb_h as usize) * 4];
    let cx_d = (bb_w as f32) * 0.5;
    let cy_d = (bb_h as f32) * 0.5;
    let cx_s = (w as f32) * 0.5;
    let cy_s = (h as f32) * 0.5;
    let stride_src = (w as usize) * 4;
    let stride_dst = (bb_w as usize) * 4;

    for yd in 0..bb_h {
        for xd in 0..bb_w {
            // Pixel-centre coords in the dst frame, origin at bbox centre.
            let dx = ((xd as f32) + 0.5) - cx_d;
            let dy = ((yd as f32) + 0.5) - cy_d;
            // Inverse rotation by -theta: R(-θ) = [ cos, sin; -sin, cos ].
            let sx_real = cos_t * dx + sin_t * dy + cx_s;
            let sy_real = -sin_t * dx + cos_t * dy + cy_s;
            // Source pixel-centre coord (fractional).
            let xs_centre = sx_real - 0.5;
            let ys_centre = sy_real - 0.5;
            let xs_floor = xs_centre.floor() as i32;
            let ys_floor = ys_centre.floor() as i32;
            let mut acc = [0.0f32; 4];
            for ky in -1..=2 {
                let ys = ys_floor + ky;
                let ty = ys as f32 - ys_centre;
                let wy = mitchell_kernel(ty);
                let ys_in = ys >= 0 && ys < h as i32;
                for kx in -1..=2 {
                    let xs = xs_floor + kx;
                    let tx = xs as f32 - xs_centre;
                    let wk = mitchell_kernel(tx) * wy;
                    // Out-of-bounds: contribute transparent (zero) —
                    // rotation corners outside the source rect must
                    // stay transparent, never edge-replicate.
                    if ys_in && xs >= 0 && xs < w as i32 {
                        let i = (ys as usize) * stride_src + (xs as usize) * 4;
                        acc[0] += src[i] * wk;
                        acc[1] += src[i + 1] * wk;
                        acc[2] += src[i + 2] * wk;
                        acc[3] += src[i + 3] * wk;
                    }
                }
            }
            let oi = (yd as usize) * stride_dst + (xd as usize) * 4;
            // Negative lobes of Mitchell can drive premult RGB slightly
            // below zero or above the alpha; clamp to keep the
            // un-premultiply step well-defined.
            out[oi] = acc[0].max(0.0);
            out[oi + 1] = acc[1].max(0.0);
            out[oi + 2] = acc[2].max(0.0);
            out[oi + 3] = acc[3].clamp(0.0, 255.0);
        }
    }
    (out, bb_w, bb_h)
}

// ---------------------------------------------------------------------
// Unit tests.
// ---------------------------------------------------------------------

#[cfg(test)]
#[allow(clippy::field_reassign_with_default, clippy::identity_op)]
#[path = "algorithm_tests.rs"]
mod tests;
