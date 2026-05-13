//! Edge-aware mask: Sobel edge detection + BFS flood fill from borders.
//!
//! Better than `colorkey` at preserving subject detail: flood fill
//! propagates background classification from the image border, but
//! stops at strong gradient edges. Color tolerance is checked against
//! the same OKLab targets used by `colorkey` (or k-means auto-detect).
//!
//! Complexity: O(N) for Sobel + O(N) for BFS. Sequential by design —
//! kept on CPU. Costs ≈15-25 ms on 2k×2k single-thread, fine for
//! debounced editor UX (50 ms target).

use super::oklab::{Oklab, oklab_dist_sq, srgb_to_oklab, tolerance_to_oklab_threshold_sq};
use super::params::RgbColor;

/// Edge-aware mask. `edges_scratch` is a `w*h` Float32 scratch buffer
/// the caller pre-allocates (HR-3). `queue_scratch` is a usize ring
/// the BFS reuses.
// HR-3: each scratch buffer is a distinct caller-owned slice so the
// implementation can avoid every alloc in the hot path. Collapsing
// them into a struct hides the lifetime story without saving any
// indirection — leaving the slot count explicit on the signature.
#[allow(clippy::too_many_arguments)]
pub fn edge_aware_mask(
    rgba: &[u8],
    w: u32,
    h: u32,
    tolerance: f32,
    edge_threshold: f32,
    targets: &[RgbColor],
    mask: &mut [f32],
    edges_scratch: &mut [f32],
    queue_scratch: &mut Vec<usize>,
) {
    let (wu, hu) = (w as usize, h as usize);
    let total = wu * hu;
    debug_assert_eq!(mask.len(), total);
    debug_assert_eq!(edges_scratch.len(), total);
    debug_assert!(!targets.is_empty(), "edge_aware needs ≥1 target");

    // Start fully opaque — only flood-fill-reachable pixels turn
    // transparent. This is the legacy convention; it preserves
    // foreground islands that happen to share a color with the bg.
    for m in mask.iter_mut() {
        *m = 1.0;
    }

    let mut tgt: [Oklab; 16] = [Oklab::default(); 16];
    let n_tgt = targets.len().min(16);
    for (i, t) in targets.iter().take(16).enumerate() {
        tgt[i] = srgb_to_oklab(t.r, t.g, t.b);
    }

    let tol_sq = tolerance_to_oklab_threshold_sq(tolerance);
    let half_sq = tol_sq * 0.25;
    let edge_thr = (edge_threshold.clamp(0.0, 100.0) / 100.0) * 2.0;
    // Color slack used when an edge is strong: if we'd otherwise
    // stop the flood, allow advance only when the pixel still looks
    // very background-ish (legacy heuristic factor 0.3).
    let lenient_sq = tol_sq * 0.3 * 0.3;

    sobel_magnitude(rgba, wu, hu, edges_scratch);

    queue_scratch.clear();
    let mut visited = vec![false; total];

    // Seed BFS from all border pixels.
    for x in 0..wu {
        queue_scratch.push(x); // top row
        queue_scratch.push((hu - 1) * wu + x); // bottom row
    }
    for y in 1..hu.saturating_sub(1) {
        queue_scratch.push(y * wu); // left
        queue_scratch.push(y * wu + wu - 1); // right
    }

    let mut head = 0;
    while head < queue_scratch.len() {
        let pos = queue_scratch[head];
        head += 1;
        if visited[pos] {
            continue;
        }
        visited[pos] = true;

        let idx = pos * 4;
        let a = rgba[idx + 3];
        if a < 10 {
            mask[pos] = 0.0;
            continue;
        }
        let r = rgba[idx];
        let g = rgba[idx + 1];
        let b = rgba[idx + 2];
        let lab = srgb_to_oklab(r, g, b);

        let mut min_d = f32::INFINITY;
        for c in tgt.iter().take(n_tgt) {
            let d = oklab_dist_sq(lab, *c);
            if d < min_d {
                min_d = d;
            }
        }

        // Hard edge AND not very-background-colored → halt.
        if edges_scratch[pos] > edge_thr && min_d > lenient_sq {
            continue;
        }

        if min_d < tol_sq {
            mask[pos] = if min_d < half_sq {
                0.0
            } else {
                let t = (min_d - half_sq) / (tol_sq - half_sq);
                smootherstep(t)
            };

            // Spread to 4-connected neighbors.
            let x = pos % wu;
            let y = pos / wu;
            if x > 0 {
                queue_scratch.push(pos - 1);
            }
            if x + 1 < wu {
                queue_scratch.push(pos + 1);
            }
            if y > 0 {
                queue_scratch.push(pos - wu);
            }
            if y + 1 < hu {
                queue_scratch.push(pos + wu);
            }
        }
    }
}

#[inline]
fn smootherstep(t: f32) -> f32 {
    let t = t.clamp(0.0, 1.0);
    t * t * t * (t * (t * 6.0 - 15.0) + 10.0)
}

/// 3×3 Sobel gradient magnitude over BT.601 luminance.
/// Output normalized roughly to 0..=2 by the kernel sums; the caller
/// thresholds in that scale via `edge_threshold` (0..=100 → 0..=2).
fn sobel_magnitude(rgba: &[u8], w: usize, h: usize, out: &mut [f32]) {
    for o in out.iter_mut() {
        *o = 0.0;
    }
    if w < 3 || h < 3 {
        return;
    }

    let luma = |x: usize, y: usize| -> f32 {
        let idx = (y * w + x) * 4;
        (rgba[idx] as f32 * 0.299 + rgba[idx + 1] as f32 * 0.587 + rgba[idx + 2] as f32 * 0.114)
            / 255.0
    };

    for y in 1..h - 1 {
        for x in 1..w - 1 {
            let tl = luma(x - 1, y - 1);
            let t = luma(x, y - 1);
            let tr = luma(x + 1, y - 1);
            let l = luma(x - 1, y);
            let r = luma(x + 1, y);
            let bl = luma(x - 1, y + 1);
            let bb = luma(x, y + 1);
            let br = luma(x + 1, y + 1);

            let gx = -tl - 2.0 * l - bl + tr + 2.0 * r + br;
            let gy = -tl - 2.0 * t - tr + bl + 2.0 * bb + br;

            out[y * w + x] = (gx * gx + gy * gy).sqrt();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn flood_fill_clears_background_around_centered_subject() {
        // 16×16 white image with 6×6 black square centered.
        let (w, h) = (16u32, 16u32);
        let mut buf = vec![255u8; (w * h * 4) as usize];
        for i in 0..(w * h) as usize {
            buf[i * 4 + 3] = 255;
        }
        for y in 5..11 {
            for x in 5..11 {
                let idx = (y * w as usize + x) * 4;
                buf[idx] = 0;
                buf[idx + 1] = 0;
                buf[idx + 2] = 0;
            }
        }

        let mut mask = vec![1.0; (w * h) as usize];
        let mut edges = vec![0.0; (w * h) as usize];
        let mut queue = Vec::new();
        edge_aware_mask(
            &buf,
            w,
            h,
            30.0,
            50.0,
            &[RgbColor::new(255, 255, 255)],
            &mut mask,
            &mut edges,
            &mut queue,
        );

        // Corner is bg, center is fg.
        assert!(mask[0] < 0.1);
        let center = (8 * w + 8) as usize;
        assert!(mask[center] > 0.9);
    }

    #[test]
    fn isolated_island_with_bg_color_is_preserved() {
        // 16×16 white background, white pixel at center with a black
        // ring around it — flood from border cannot reach the center
        // through the ring, so center stays "foreground" (alpha 1)
        // even though it's white.
        let (w, h) = (16u32, 16u32);
        let mut buf = vec![255u8; (w * h * 4) as usize];
        for i in 0..(w * h) as usize {
            buf[i * 4 + 3] = 255;
        }
        // Black 6×6 frame at rows 5-10, cols 5-10 (hollow).
        for y in 5..=10usize {
            for x in 5..=10usize {
                if y == 5 || y == 10 || x == 5 || x == 10 {
                    let idx = (y * w as usize + x) * 4;
                    buf[idx] = 0;
                    buf[idx + 1] = 0;
                    buf[idx + 2] = 0;
                }
            }
        }

        let mut mask = vec![1.0; (w * h) as usize];
        let mut edges = vec![0.0; (w * h) as usize];
        let mut queue = Vec::new();
        edge_aware_mask(
            &buf,
            w,
            h,
            30.0,
            50.0,
            &[RgbColor::new(255, 255, 255)],
            &mut mask,
            &mut edges,
            &mut queue,
        );

        // The white pixel INSIDE the ring should be untouched (alpha 1).
        let inside = (7 * w + 7) as usize;
        assert_eq!(
            mask[inside], 1.0,
            "edge-aware should preserve enclosed island"
        );
        // Corner remains background.
        assert!(mask[0] < 0.1);
    }
}
