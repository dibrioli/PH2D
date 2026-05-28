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
mod tests {
    use super::*;

    fn pixel_at(buf: &[u8], w: u32, x: u32, y: u32) -> [u8; 4] {
        let i = ((y * w + x) * 4) as usize;
        [buf[i], buf[i + 1], buf[i + 2], buf[i + 3]]
    }

    /// Fill an `RGBA8` buffer of size `w*h` with `colour`.
    fn solid(w: u32, h: u32, colour: [u8; 4]) -> Vec<u8> {
        let mut v = vec![0u8; (w * h * 4) as usize];
        for i in (0..v.len()).step_by(4) {
            v[i] = colour[0];
            v[i + 1] = colour[1];
            v[i + 2] = colour[2];
            v[i + 3] = colour[3];
        }
        v
    }

    #[test]
    fn identity_transform_returns_unchanged_with_did_change_false() {
        let rgba = solid(4, 3, [10, 20, 30, 255]);
        let r = rasterize(&rgba, 4, 3, 1.0, 1.0, 0.0);
        assert_eq!(r.width, 4);
        assert_eq!(r.height, 3);
        assert!(!r.did_change);
        assert_eq!(r.pixels, rgba);
    }

    #[test]
    fn non_finite_inputs_fall_back_to_identity() {
        let rgba = solid(2, 2, [50, 60, 70, 255]);
        let r = rasterize(&rgba, 2, 2, f32::NAN, f32::INFINITY, f32::NAN);
        assert!(!r.did_change);
        assert_eq!(r.width, 2);
        assert_eq!(r.height, 2);
        assert_eq!(r.pixels, rgba);
    }

    #[test]
    fn zero_width_returns_sentinel() {
        let r = rasterize(&[], 0, 5, 2.0, 2.0, 0.0);
        assert_eq!(r.width, 1);
        assert_eq!(r.height, 1);
        assert_eq!(r.pixels, vec![0, 0, 0, 0]);
        assert!(r.did_change);
    }

    #[test]
    fn zero_height_returns_sentinel() {
        let r = rasterize(&[], 5, 0, 1.5, 1.5, 0.0);
        assert_eq!(r.width, 1);
        assert_eq!(r.height, 1);
        assert!(r.did_change);
    }

    #[test]
    #[should_panic(expected = "rgba buffer length must equal")]
    fn buffer_length_mismatch_panics() {
        let rgba = vec![0u8; 3];
        let _ = rasterize(&rgba, 4, 4, 1.0, 1.0, 0.0);
    }

    #[test]
    fn upscale_doubles_dimensions() {
        let rgba = solid(4, 3, [200, 100, 50, 255]);
        let r = rasterize(&rgba, 4, 3, 2.0, 2.0, 0.0);
        assert_eq!(r.width, 8);
        assert_eq!(r.height, 6);
        assert_eq!(r.pixels.len(), 8 * 6 * 4);
        assert!(r.did_change);
        // Solid colour stays solid (Mitchell partition of unity + edge
        // clamp preserves a constant input exactly within RGB rounding
        // noise of ±1).
        let centre = pixel_at(&r.pixels, r.width, 4, 3);
        assert!(
            (centre[0] as i32 - 200).abs() <= 1
                && (centre[1] as i32 - 100).abs() <= 1
                && (centre[2] as i32 - 50).abs() <= 1
                && centre[3] == 255,
            "centre={:?}",
            centre,
        );
    }

    #[test]
    fn downscale_halves_dimensions() {
        let rgba = solid(8, 6, [128, 64, 32, 255]);
        let r = rasterize(&rgba, 8, 6, 0.5, 0.5, 0.0);
        assert_eq!(r.width, 4);
        assert_eq!(r.height, 3);
        assert!(r.did_change);
        let centre = pixel_at(&r.pixels, r.width, 2, 1);
        assert!(
            (centre[0] as i32 - 128).abs() <= 1
                && (centre[1] as i32 - 64).abs() <= 1
                && (centre[2] as i32 - 32).abs() <= 1
                && centre[3] == 255,
        );
    }

    #[test]
    fn horizontal_flip_reverses_columns() {
        // 4×1 strip with a known left→right gradient. After flipping on
        // X, column 0 holds the old column 3, etc.
        let mut rgba = vec![0u8; 4 * 1 * 4];
        for x in 0..4 {
            let i = (x * 4) as usize;
            rgba[i] = x as u8 * 50;
            rgba[i + 1] = 100;
            rgba[i + 2] = 200;
            rgba[i + 3] = 255;
        }
        let r = rasterize(&rgba, 4, 1, -1.0, 1.0, 0.0);
        assert_eq!(r.width, 4);
        assert_eq!(r.height, 1);
        assert!(r.did_change);
        for x in 0..4u32 {
            let p = pixel_at(&r.pixels, r.width, x, 0);
            assert_eq!(p[0], (3 - x) as u8 * 50, "x={x}");
            assert_eq!(p[3], 255);
        }
    }

    #[test]
    fn vertical_flip_reverses_rows() {
        let mut rgba = vec![0u8; 1 * 4 * 4];
        for y in 0..4 {
            let i = (y * 4) as usize;
            rgba[i + 1] = y as u8 * 50;
            rgba[i + 3] = 255;
        }
        let r = rasterize(&rgba, 1, 4, 1.0, -1.0, 0.0);
        assert_eq!(r.width, 1);
        assert_eq!(r.height, 4);
        for y in 0..4u32 {
            let p = pixel_at(&r.pixels, r.width, 0, y);
            assert_eq!(p[1], (3 - y) as u8 * 50, "y={y}");
        }
    }

    #[test]
    fn double_flip_returns_original_orientation() {
        // 2×2 with distinct corner colours; flipping on both axes is
        // a 180° point-mirror — every corner swaps with the diagonal.
        let mut rgba = vec![0u8; 2 * 2 * 4];
        let cols: [[u8; 4]; 4] = [
            [10, 0, 0, 255],
            [0, 20, 0, 255],
            [0, 0, 30, 255],
            [40, 40, 40, 255],
        ];
        for (idx, c) in cols.iter().enumerate() {
            let i = idx * 4;
            rgba[i..i + 4].copy_from_slice(c);
        }
        let r = rasterize(&rgba, 2, 2, -1.0, -1.0, 0.0);
        assert_eq!(r.width, 2);
        assert_eq!(r.height, 2);
        // top-left becomes bottom-right (cols[0] → idx 3), etc.
        assert_eq!(pixel_at(&r.pixels, 2, 1, 1), cols[0]);
        assert_eq!(pixel_at(&r.pixels, 2, 0, 1), cols[1]);
        assert_eq!(pixel_at(&r.pixels, 2, 1, 0), cols[2]);
        assert_eq!(pixel_at(&r.pixels, 2, 0, 0), cols[3]);
    }

    #[test]
    fn rotation_by_90_degrees_swaps_dimensions() {
        let rgba = solid(8, 4, [100, 100, 100, 255]);
        let r = rasterize(&rgba, 8, 4, 1.0, 1.0, std::f32::consts::FRAC_PI_2);
        // |cos π/2| = 0, |sin π/2| = 1 → bbox = (h, w) = (4, 8).
        assert_eq!(r.width, 4);
        assert_eq!(r.height, 8);
        assert!(r.did_change);
    }

    #[test]
    fn rotation_by_180_degrees_preserves_dimensions() {
        let rgba = solid(6, 4, [80, 80, 80, 255]);
        let r = rasterize(&rgba, 6, 4, 1.0, 1.0, std::f32::consts::PI);
        // |cos π| = 1, |sin π| = 0 → bbox = (w, h).
        assert_eq!(r.width, 6);
        assert_eq!(r.height, 4);
    }

    #[test]
    fn tiny_rotation_is_treated_as_identity_shape() {
        let rgba = solid(4, 3, [50, 60, 70, 255]);
        // Below ROTATION_EPS — rotation pass skipped, only resample (no-op
        // since scale = 1) + flip (none) run. Output keeps (w, h).
        let r = rasterize(&rgba, 4, 3, 1.0, 1.0, ROTATION_EPS / 2.0);
        assert_eq!(r.width, 4);
        assert_eq!(r.height, 3);
        // Identity short-circuit catches scale=1 + rot=0; tiny non-zero
        // rotation goes through resample path but the resample also
        // short-circuits since w_s == width and h_s == height.
        assert!(!r.did_change);
    }

    #[test]
    fn transparent_source_stays_transparent() {
        let rgba = solid(4, 4, [0, 0, 0, 0]);
        let r = rasterize(&rgba, 4, 4, 2.0, 2.0, 0.0);
        // Every output pixel must have alpha 0.
        for chunk in r.pixels.chunks_exact(4) {
            assert_eq!(chunk[3], 0, "transparent input → transparent output");
        }
    }

    #[test]
    fn opaque_solid_stays_solid_under_resample() {
        // Edge replication + Mitchell partition-of-unity should
        // reconstruct a constant input exactly (modulo rounding ±1).
        let rgba = solid(16, 16, [200, 50, 100, 255]);
        let r = rasterize(&rgba, 16, 16, 0.75, 0.75, 0.0);
        for chunk in r.pixels.chunks_exact(4) {
            assert!((chunk[0] as i32 - 200).abs() <= 1);
            assert!((chunk[1] as i32 - 50).abs() <= 1);
            assert!((chunk[2] as i32 - 100).abs() <= 1);
            assert_eq!(chunk[3], 255);
        }
    }

    #[test]
    fn premultiplied_alpha_prevents_colour_bleed_from_transparent_region() {
        // Half opaque red, half transparent magenta. After a 2× upscale
        // the opaque half must still report red — without premult, the
        // Mitchell kernel would blend the magenta RGB into the red
        // through the alpha-zero edge.
        let mut rgba = vec![0u8; 4 * 1 * 4];
        rgba[0..4].copy_from_slice(&[255, 0, 0, 255]); // opaque red
        rgba[4..8].copy_from_slice(&[255, 0, 0, 255]); // opaque red
        rgba[8..12].copy_from_slice(&[255, 0, 255, 0]); // transparent magenta
        rgba[12..16].copy_from_slice(&[255, 0, 255, 0]); // transparent magenta
        let r = rasterize(&rgba, 4, 1, 2.0, 1.0, 0.0);
        // Output is 8×1; inspect the deep-opaque end (pixel 1 — well
        // inside the opaque red half).
        let p = pixel_at(&r.pixels, 8, 1, 0);
        assert_eq!(p[3], 255, "alpha at opaque-deep pixel");
        // Red must be near 255; green near 0 (no magenta bleed).
        assert!((p[0] as i32 - 255).abs() <= 1, "red was {}", p[0]);
        assert_eq!(p[1], 0, "green was {}", p[1]);
    }

    #[test]
    fn mitchell_kernel_partition_of_unity_at_integer_offsets() {
        // Weights at 4 integer source pixels around any sub-pixel
        // sample point sum to ~1.0 — the property that lets us skip
        // post-normalisation.
        for offset_steps in 0..32 {
            let xs_centre = offset_steps as f32 / 32.0;
            let floor = xs_centre.floor() as i32;
            let mut sum = 0.0;
            for k in -1..=2 {
                let xs = floor + k;
                let t = xs as f32 - xs_centre;
                sum += mitchell_kernel(t);
            }
            assert!(
                (sum - 1.0).abs() < 1e-4,
                "weights at xs_centre={xs_centre} sum to {sum}, expected ~1.0",
            );
        }
    }
}
