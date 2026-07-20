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

/// Hard cap on the Expand/Contract disc radius (px) — a click grows/shrinks a controllable, repeatable
/// amount rather than one huge jump (which mangled the mask — Enio).
const MORPH_MAX: usize = 16;

/// A one-click whole-canvas mask operation. The tool maps each button to one of these; `radius` (the
/// brush Size, px) sets the morphology / blur extent so the user controls the amount with the Size slider.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum MaskCanvasOp {
    /// Grow the painted (CONCEALED) region — a min filter over a disc (dilate the mask outward).
    Expand,
    /// Shrink the painted (CONCEALED) region — a max filter over a disc (erode the mask inward).
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

/// Apply `op` to the whole grayscale mask `buf` (`w·h·4` RGBA8). `radius` is the brush Size in px. No-op
/// on a degenerate / short buffer. Expand/Contract grow/shrink the PAINTED (concealed) region — since a
/// mask is white/revealed by default, the mask the artist builds is the black area (Enio: they were
/// inverted before). Blur/Sharpen scale with [`kernel_radius`]; Expand/Contract with [`morph_radius`].
pub fn apply_mask_op(buf: &mut [u8], w: u32, h: u32, op: MaskCanvasOp, radius: f32) {
    let (wu, hu) = (w as usize, h as usize);
    if wu == 0 || hu == 0 || buf.len() < wu * hu * 4 {
        return;
    }
    match op {
        MaskCanvasOp::Expand => morph(buf, wu, hu, morph_radius(radius), true),
        MaskCanvasOp::Contract => morph(buf, wu, hu, morph_radius(radius), false),
        MaskCanvasOp::Blur => blur_whole(buf, w, h, kernel_radius(radius)),
        MaskCanvasOp::Sharpen => sharpen_whole(buf, w, h, kernel_radius(radius)),
        MaskCanvasOp::Invert => invert_whole(buf, wu * hu),
        MaskCanvasOp::Clear => fill_whole(buf, wu * hu, 255),
    }
}

/// The Expand/Contract disc radius (px): a modest fraction of the brush Size, capped at [`MORPH_MAX`],
/// so each click nudges the mask edge a little (repeat to go further) instead of one mask-mangling jump.
#[must_use]
fn morph_radius(radius: f32) -> usize {
    ((radius * 0.15).round() as i64).clamp(1, MORPH_MAX as i64) as usize
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

/// Circular **morphology** over a disc of radius `k`. `grow_concealed` = min filter (spreads the DARK /
/// concealed region → Expand grows the painted mask); else max filter (spreads white → Contract erodes
/// it). A round structuring element (Euclidean disc, `dx² + dy² ≤ k²`) keeps growth/shrink SMOOTH — the
/// old separable-square element left boxy, jagged edges (Enio). Clamp-to-edge; snapshots the source so
/// overlapping reads never feed back. O(w·h·πk²), and `k` is capped modestly by [`morph_radius`].
fn morph(buf: &mut [u8], w: usize, h: usize, k: usize, grow_concealed: bool) {
    if k == 0 {
        return;
    }
    // The disc offsets, computed once (a round SE, not a square box).
    let ki = k as i32;
    let kk = ki * ki;
    let mut offs: Vec<(i32, i32)> = Vec::new();
    for dy in -ki..=ki {
        for dx in -ki..=ki {
            if dx * dx + dy * dy <= kk {
                offs.push((dx, dy));
            }
        }
    }
    let src: Vec<u8> = (0..w * h).map(|i| luma(buf, i * 4).round() as u8).collect();
    let (wi, hi) = (w as i32, h as i32);
    for y in 0..h {
        for x in 0..w {
            let mut acc = src[y * w + x];
            for &(dx, dy) in &offs {
                let sx = (x as i32 + dx).clamp(0, wi - 1) as usize;
                let sy = (y as i32 + dy).clamp(0, hi - 1) as usize;
                let v = src[sy * w + sx];
                acc = if grow_concealed {
                    acc.min(v)
                } else {
                    acc.max(v)
                };
            }
            put_gray(buf, y * w + x, acc);
        }
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
    fn expand_grows_the_painted_concealed_region() {
        // A black (concealed) dot at (1,1) on a white/revealed mask; Expand spreads the black to a
        // neighbour — the painted mask grows outward (radius 5 → disc k=1, so growth is one texel).
        let (mut buf, w, h) = flat(255);
        let center = idx4(1, 1);
        for c in 0..3 {
            buf[center + c] = 0;
        }
        apply_mask_op(&mut buf, w, h, MaskCanvasOp::Expand, 5.0);
        assert_eq!(
            buf[idx4(2, 1)],
            0,
            "Expand grew the concealed (black) region"
        );
    }

    #[test]
    fn contract_shrinks_the_painted_concealed_region() {
        // A lone black (concealed) dot at (2,2) on white; Contract erodes it back to revealed (white).
        let (mut buf, w, h) = flat(255);
        let dot = idx4(2, 2);
        for c in 0..3 {
            buf[dot + c] = 0;
        }
        apply_mask_op(&mut buf, w, h, MaskCanvasOp::Contract, 5.0);
        assert_eq!(
            buf[dot], 255,
            "Contract eroded the lone concealed dot back to revealed"
        );
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
