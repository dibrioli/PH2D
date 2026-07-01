//! Whole-canvas **mask** operations — one-click filters over a grayscale layer mask (RGBA8 where
//! `R=G=B` is the coverage luma, `α=255`; white reveals, black conceals). Applied to the WHOLE buffer
//! by the Mask tool's canvas-op buttons (Expand / Contract / Blur / Sharpen / Invert / Clear).
//!
//! Everything here is transcendental-free (HR-5): morphology is min/max, the blur reuses the binomial
//! kernel in [`crate::blur`], and Sharpen is an unsharp-mask (`v + (v − blur)·amount`). Results are
//! written back as opaque grayscale so the compositor's Rec.601 read (`mask_value`) is exact.

use crate::blur::{blur_region, kernel_radius};

/// The unsharp-mask strength for [`MaskCanvasOp::Sharpen`] — how hard the mask edges are pushed apart
/// (harden the transition). `1.0` doubles the local contrast against the blurred base.
const SHARPEN_AMOUNT: f32 = 1.0;

/// A one-click whole-canvas mask operation. The tool maps each button to one of these; `radius` (the
/// brush Size, px) sets the morphology / blur extent so the user controls the amount with the Size slider.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum MaskCanvasOp {
    /// Grow the revealed (white) region — a max filter (dilate).
    Expand,
    /// Shrink the revealed (white) region — a min filter (erode).
    Contract,
    /// Soften the mask — a binomial blur (feather the edges).
    Blur,
    /// Harden the mask edges — an unsharp mask (push values toward 0 / 1 at transitions).
    Sharpen,
    /// Invert coverage — `255 − v` (swap revealed ↔ concealed).
    Invert,
    /// Reset to fully revealed (opaque white) — erase all mask painting.
    Clear,
}

/// Apply `op` to the whole grayscale mask `buf` (`w·h·4` RGBA8). `radius` is the brush Size in px; the
/// morphology / blur extent scales with it via [`kernel_radius`]. No-op on a degenerate / short buffer.
pub fn apply_mask_op(buf: &mut [u8], w: u32, h: u32, op: MaskCanvasOp, radius: f32) {
    let (wu, hu) = (w as usize, h as usize);
    if wu == 0 || hu == 0 || buf.len() < wu * hu * 4 {
        return;
    }
    match op {
        MaskCanvasOp::Expand => morph(buf, wu, hu, kernel_radius(radius), true),
        MaskCanvasOp::Contract => morph(buf, wu, hu, kernel_radius(radius), false),
        MaskCanvasOp::Blur => blur_whole(buf, w, h, kernel_radius(radius)),
        MaskCanvasOp::Sharpen => sharpen_whole(buf, w, h, kernel_radius(radius)),
        MaskCanvasOp::Invert => invert_whole(buf, wu * hu),
        MaskCanvasOp::Clear => fill_whole(buf, wu * hu, 255),
    }
}

/// Rec.601 luma (`0..=255`) of one RGBA8 texel at byte offset `b`. Grayscale masks have `R=G=B` so this
/// equals the R channel, but reading luma keeps the ops correct even if a non-gray buffer slips in.
#[inline]
fn luma(buf: &[u8], b: usize) -> f32 {
    0.299 * f32::from(buf[b]) + 0.587 * f32::from(buf[b + 1]) + 0.114 * f32::from(buf[b + 2])
}

/// Write an opaque grayscale texel (`R=G=B=v`, `α=255`) at pixel index `i`.
#[inline]
fn put_gray(buf: &mut [u8], i: usize, v: u8) {
    let b = i * 4;
    buf[b] = v;
    buf[b + 1] = v;
    buf[b + 2] = v;
    buf[b + 3] = 255;
}

/// Separable **morphology**: `dilate` = max over the `(2k+1)²` neighbourhood (grow white), else min
/// (erode, shrink white). Clamp-to-edge apron. Two O(w·h·k) passes — fine for a one-click op.
fn morph(buf: &mut [u8], w: usize, h: usize, k: usize, dilate: bool) {
    if k == 0 {
        return;
    }
    // Extract the coverage luma plane once.
    let mut src: Vec<u8> = (0..w * h).map(|i| luma(buf, i * 4).round() as u8).collect();
    let mut tmp = vec![0u8; w * h];
    let pick = |a: u8, b: u8| if dilate { a.max(b) } else { a.min(b) };
    // Horizontal pass: src → tmp.
    for y in 0..h {
        let row = y * w;
        for x in 0..w {
            let mut acc = src[row + x];
            for d in 1..=k {
                let xl = x.saturating_sub(d);
                let xr = (x + d).min(w - 1);
                acc = pick(acc, src[row + xl]);
                acc = pick(acc, src[row + xr]);
            }
            tmp[row + x] = acc;
        }
    }
    // Vertical pass: tmp → src.
    for y in 0..h {
        for x in 0..w {
            let mut acc = tmp[y * w + x];
            for d in 1..=k {
                let yt = y.saturating_sub(d);
                let yb = (y + d).min(h - 1);
                acc = pick(acc, tmp[yt * w + x]);
                acc = pick(acc, tmp[yb * w + x]);
            }
            src[y * w + x] = acc;
        }
    }
    for (i, &v) in src.iter().enumerate() {
        put_gray(buf, i, v);
    }
}

/// Soften the whole mask with the binomial blur (reuse [`blur_region`] over the full canvas), writing the
/// blurred luma back as opaque grayscale. A fully-opaque grayscale buffer blurs identically in premul.
fn blur_whole(buf: &mut [u8], w: u32, h: u32, k: usize) {
    if k == 0 {
        return;
    }
    let blurred = blur_region(
        buf,
        w as i64,
        h as i64,
        0,
        0,
        w as usize,
        h as usize,
        k,
        [false, false],
    );
    for (i, px) in blurred.iter().enumerate() {
        let v = (0.299 * px[0] + 0.587 * px[1] + 0.114 * px[2])
            .round()
            .clamp(0.0, 255.0) as u8;
        put_gray(buf, i, v);
    }
}

/// Harden the mask edges: unsharp mask `v' = clamp(v + (v − blur)·amount)`. Amplifies the transition so
/// a soft edge becomes crisp (the "make harder" op). Reuses the same binomial blur as the low-pass base.
fn sharpen_whole(buf: &mut [u8], w: u32, h: u32, k: usize) {
    if k == 0 {
        return;
    }
    let blurred = blur_region(
        buf,
        w as i64,
        h as i64,
        0,
        0,
        w as usize,
        h as usize,
        k,
        [false, false],
    );
    for (i, px) in blurred.iter().enumerate() {
        let o = luma(buf, i * 4);
        let b = 0.299 * px[0] + 0.587 * px[1] + 0.114 * px[2];
        let v = (o + (o - b) * SHARPEN_AMOUNT).round().clamp(0.0, 255.0) as u8;
        put_gray(buf, i, v);
    }
}

/// Invert coverage: `255 − luma`, written back opaque grayscale.
fn invert_whole(buf: &mut [u8], n: usize) {
    for i in 0..n {
        let v = 255 - luma(buf, i * 4).round().clamp(0.0, 255.0) as u8;
        put_gray(buf, i, v);
    }
}

/// Fill the whole mask with a flat coverage `v` (opaque grayscale) — Clear uses `255` (fully revealed).
fn fill_whole(buf: &mut [u8], n: usize, v: u8) {
    for i in 0..n {
        put_gray(buf, i, v);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A 4×4 grayscale RGBA mask, all pixels `v`.
    fn flat(v: u8) -> (Vec<u8>, u32, u32) {
        (vec![v; 4 * 4 * 4], 4, 4)
    }

    /// Byte offset of texel `(x, y)` in the 4-wide test mask (a fn, so the width factor isn't a literal
    /// `1 *` / `2 *` that clippy's `identity_op` would flag).
    fn idx4(x: usize, y: usize) -> usize {
        (y * 4 + x) * 4
    }

    #[test]
    fn clear_fills_white() {
        let (mut buf, w, h) = flat(0);
        apply_mask_op(&mut buf, w, h, MaskCanvasOp::Clear, 10.0);
        assert!(buf.iter().all(|&b| b == 255));
    }

    #[test]
    fn invert_swaps_black_white() {
        let (mut buf, w, h) = flat(0);
        apply_mask_op(&mut buf, w, h, MaskCanvasOp::Invert, 10.0);
        // Black → white; alpha stays opaque.
        assert!(buf.iter().all(|&b| b == 255));
    }

    #[test]
    fn expand_grows_a_white_dot() {
        // Single white texel at (1,1) on black; dilate must spread it to neighbours.
        let (mut buf, w, h) = flat(0);
        let center = idx4(1, 1);
        for c in 0..4 {
            buf[center + c] = 255;
        }
        apply_mask_op(&mut buf, w, h, MaskCanvasOp::Expand, 10.0);
        // A neighbour of the dot is now white (grew outward).
        let neighbour = idx4(2, 1);
        assert_eq!(buf[neighbour], 255);
    }

    #[test]
    fn contract_shrinks_a_black_hole() {
        // Mostly white with a single black hole; erode spreads the black (shrinks white).
        let (mut buf, w, h) = flat(255);
        let hole = idx4(2, 2);
        for c in 0..3 {
            buf[hole + c] = 0;
        }
        apply_mask_op(&mut buf, w, h, MaskCanvasOp::Contract, 10.0);
        let neighbour = idx4(1, 2);
        assert_eq!(buf[neighbour], 0);
    }

    #[test]
    fn blur_is_a_no_op_on_a_flat_field() {
        let (mut buf, w, h) = flat(128);
        apply_mask_op(&mut buf, w, h, MaskCanvasOp::Blur, 10.0);
        // A constant field blurs to itself (±1 rounding).
        assert!(buf.iter().step_by(4).all(|&r| (r as i32 - 128).abs() <= 1));
    }

    #[test]
    fn degenerate_buffer_is_ignored() {
        let mut buf = vec![0u8; 3]; // too short
        apply_mask_op(&mut buf, 4, 4, MaskCanvasOp::Clear, 10.0);
        assert_eq!(buf, vec![0u8; 3]); // untouched
    }
}
