//! Stage 5 — Phase 2 sharpen.
//!
//! Small radius (≤ 1) takes the fast Laplacian 3×3 ([`sharpen_laplacian`]);
//! larger radius takes Unsharp Mask (separable Gaussian blur,
//! [`sharpen_unsharp`]). Both run in **linear sRGB** so ringing stays
//! symmetric across the tonal range. The shared [`gaussian_kernel_1d`] is
//! `pub` so the WGSL Unsharp port ([`crate::gpu::sharpen`]) reuses the exact
//! same kernel — single source of truth.

/// Sharpen via the 3×3 Laplacian kernel `[0,-1,0; -1,5,-1; 0,-1,0]` in
/// **linear sRGB**. `amount` in `[0, 2]`:
/// `result = center + (laplacian − center) · amount`. Fast and CPU-
/// friendly; use this when `radius ≤ 1`.
///
/// Linear-space sharpening avoids the gamma-space asymmetry of the old
/// path (sRGB compresses shadows → equal-amplitude ringing was perceived
/// stronger in dark areas than in highlights). Linear ringing is
/// uniformly visible across the tonal range.
pub fn sharpen_laplacian(rgba: &mut [u8], w: u32, h: u32, amount: f32) {
    use crate::color_utils::{linear_to_srgb_u8, srgb_to_linear_u8};

    if amount <= 0.0 {
        return;
    }
    let w_i = w as i32;
    let h_i = h as i32;
    let stride = w as usize;
    let n_px = (w as usize) * (h as usize);

    // Pre-linearize once; reused for 4 neighbour lookups per pixel × 3
    // channels — amortises the sRGB transfer.
    let mut src_lin: Vec<[f32; 3]> = Vec::with_capacity(n_px);
    for px in rgba.chunks_exact(4) {
        src_lin.push([
            srgb_to_linear_u8(px[0]),
            srgb_to_linear_u8(px[1]),
            srgb_to_linear_u8(px[2]),
        ]);
    }

    for y in 0..h_i {
        for x in 0..w_i {
            let cidx = (y as usize) * stride + (x as usize);
            let ci = cidx * 4;
            if rgba[ci + 3] == 0 {
                continue;
            }
            let center = src_lin[cidx];
            let top = if y > 0 {
                src_lin[cidx - stride]
            } else {
                center
            };
            let bottom = if y < h_i - 1 {
                src_lin[cidx + stride]
            } else {
                center
            };
            let left = if x > 0 { src_lin[cidx - 1] } else { center };
            let right = if x < w_i - 1 {
                src_lin[cidx + 1]
            } else {
                center
            };
            for ch in 0..3 {
                let laplacian = 5.0 * center[ch] - top[ch] - bottom[ch] - left[ch] - right[ch];
                let result = center[ch] + (laplacian - center[ch]) * amount;
                rgba[ci + ch] = linear_to_srgb_u8(result);
            }
        }
    }
}

/// Sharpen via unsharp masking (Gaussian blur → subtract → add scaled
/// difference) in **linear sRGB**. Use this when `radius > 1`. `radius`
/// typically `1.5..3`; `amount` in `[0, 2]`.
///
/// Why linear: Gaussian blur of gamma-encoded values darkens edges
/// (mean(sRGB_dark, sRGB_light) ≠ sRGB(mean(linear_dark, linear_light))).
/// In sharpen the visible effect is asymmetric ringing — undershoots
/// in shadows are exaggerated, overshoots in highlights are muted. The
/// linear-space pipeline keeps both edges symmetric.
///
/// **GPU note**: separable Gaussian blur is the canonical GPU win — two
/// horizontal + vertical passes scale linearly with radius on CPU but
/// stay near-constant on GPU. CPU path here is fine for radius ≤ 5 in
/// 1024² previews; large-radius production sharpen should use WGSL.
pub fn sharpen_unsharp(rgba: &mut [u8], w: u32, h: u32, amount: f32, radius: f32) {
    use crate::color_utils::{linear_to_srgb_u8, srgb_to_linear_u8};

    if amount <= 0.0 || radius <= 0.0 {
        return;
    }
    let kernel = gaussian_kernel_1d(radius);
    let size = kernel.len();
    let half = (size / 2) as i32;
    let total = (w as usize) * (h as usize);
    let w_i = w as i32;
    let h_i = h as i32;

    for ch in 0..3 {
        // Extract channel as **linear** sRGB into f32 buffer.
        let mut channel: Vec<f32> = (0..total)
            .map(|i| srgb_to_linear_u8(rgba[i * 4 + ch]))
            .collect();
        let original_lin: Vec<f32> = channel.clone();

        // Horizontal pass (separable).
        let mut h_pass = vec![0.0_f32; total];
        for y in 0..h_i {
            for x in 0..w_i {
                let mut sum = 0.0;
                let mut wt = 0.0;
                for (k, &kw) in kernel.iter().enumerate() {
                    let sx = (x + k as i32 - half).clamp(0, w_i - 1);
                    sum += channel[y as usize * w as usize + sx as usize] * kw;
                    wt += kw;
                }
                h_pass[y as usize * w as usize + x as usize] = sum / wt;
            }
        }

        // Vertical pass into `channel` (reused as blur output, in linear).
        for y in 0..h_i {
            for x in 0..w_i {
                let mut sum = 0.0;
                let mut wt = 0.0;
                for (k, &kw) in kernel.iter().enumerate() {
                    let sy = (y + k as i32 - half).clamp(0, h_i - 1);
                    sum += h_pass[sy as usize * w as usize + x as usize] * kw;
                    wt += kw;
                }
                channel[y as usize * w as usize + x as usize] = sum / wt;
            }
        }

        // Unsharp combine in linear: `original + amount · (original − blur)`.
        // Encode back to sRGB on write.
        for i in 0..total {
            if rgba[i * 4 + 3] == 0 {
                continue;
            }
            let orig = original_lin[i];
            let blur = channel[i];
            let diff = orig - blur;
            rgba[i * 4 + ch] = linear_to_srgb_u8(orig + amount * diff);
        }
    }
}

/// Normalised 1D Gaussian kernel of odd length `⌈radius·2⌉·2+1`, with
/// `σ = radius / 2`. Centred so index `size / 2` is the peak. `pub`
/// so the WGSL Unsharp port ([`crate::gpu::sharpen`]) can share the
/// exact same kernel — single source of truth.
pub fn gaussian_kernel_1d(radius: f32) -> Vec<f32> {
    let size = ((radius * 2.0).ceil() as usize) * 2 + 1;
    let sigma = (radius / 2.0).max(f32::EPSILON);
    let half = (size / 2) as f32;
    let mut kernel = vec![0.0_f32; size];
    let mut sum = 0.0_f32;
    for (i, k) in kernel.iter_mut().enumerate() {
        let d = i as f32 - half;
        *k = (-(d * d) / (2.0 * sigma * sigma)).exp();
        sum += *k;
    }
    for k in kernel.iter_mut() {
        *k /= sum;
    }
    kernel
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 4×4 RGBA8 with a single solid colour + opaque alpha.
    fn solid(w: u32, h: u32, rgb: [u8; 3]) -> Vec<u8> {
        let mut v = Vec::with_capacity((w * h * 4) as usize);
        for _ in 0..(w * h) {
            v.extend_from_slice(&[rgb[0], rgb[1], rgb[2], 255]);
        }
        v
    }

    #[test]
    fn sharpen_laplacian_zero_amount_is_noop() {
        let mut buf = solid(8, 8, [120, 130, 140]);
        let before = buf.clone();
        sharpen_laplacian(&mut buf, 8, 8, 0.0);
        assert_eq!(buf, before);
    }

    #[test]
    fn sharpen_laplacian_increases_edge_contrast() {
        // 4×4 with a clean vertical edge at x=2: left=80, right=180.
        let mut buf = Vec::with_capacity(4 * 4 * 4);
        for _y in 0..4u32 {
            for x in 0..4u32 {
                let v = if x < 2 { 80u8 } else { 180u8 };
                buf.extend_from_slice(&[v, v, v, 255]);
            }
        }
        // Row 1 (`y = 1`), columns 1 (just-left-of-edge) and 2 (just-right).
        let idx_left = ((4 + 1) * 4) as usize;
        let idx_right = ((4 + 2) * 4) as usize;
        let edge_left_before = buf[idx_left];
        let edge_right_before = buf[idx_right];
        sharpen_laplacian(&mut buf, 4, 4, 1.0);
        let edge_left_after = buf[idx_left];
        let edge_right_after = buf[idx_right];
        let contrast_before = (edge_right_before as i32 - edge_left_before as i32).abs();
        let contrast_after = (edge_right_after as i32 - edge_left_after as i32).abs();
        assert!(
            contrast_after >= contrast_before,
            "Laplacian did not enhance edge: before {contrast_before}, after {contrast_after}"
        );
    }

    #[test]
    fn sharpen_unsharp_zero_amount_is_noop() {
        let mut buf = solid(8, 8, [120, 130, 140]);
        let before = buf.clone();
        sharpen_unsharp(&mut buf, 8, 8, 0.0, 2.0);
        assert_eq!(buf, before);
    }

    #[test]
    fn sharpen_unsharp_returns_non_empty_on_radius_above_one() {
        let mut buf = solid(16, 16, [120, 130, 140]);
        sharpen_unsharp(&mut buf, 16, 16, 0.5, 2.0);
        // Solid colour input → Gaussian blur returns the same value → diff
        // is zero → output equals input. So a basic smoke is: the function
        // ran without panicking + output remains a valid buffer.
        assert_eq!(buf.len(), 16 * 16 * 4);
    }

    #[test]
    fn gaussian_kernel_normalises_to_one() {
        for radius in [1.0_f32, 2.0, 3.0, 5.0] {
            let k = gaussian_kernel_1d(radius);
            let sum: f32 = k.iter().sum();
            assert!(
                (sum - 1.0).abs() < 1e-5,
                "kernel for radius {radius} did not normalize: sum {sum}"
            );
            // Centre is the peak.
            let mid = k.len() / 2;
            for (i, v) in k.iter().enumerate() {
                if i != mid {
                    assert!(*v <= k[mid] + 1e-6, "non-monotonic at radius {radius}");
                }
            }
        }
    }
}
