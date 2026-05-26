//! Auto-protect mask via edge-aware subject interior detection.
//!
//! This module is a SIDECAR to [`super::chroma`] — it produces a
//! force-keep mask (255 = inside subject silhouette, 0 = outside) that
//! the host crate merges with the user-painted protect mask before
//! handing it to the EXISTING pipeline. The chroma + compose steps
//! are NOT modified; they just receive a richer protect mask when the
//! Detect-Subject toggle is on.
//!
//! Pipeline (CPU, all per-pixel ops; ~10ms at 1024² on M-series):
//!   1. Luma          : Y = 0.299R + 0.587G + 0.114B (integer Rec.601).
//!   2. Sobel L1 mag  : |gx| + |gy| of luma → edge magnitude (u16).
//!   3. Otsu threshold: histogram-based automatic split → binary edges.
//!   4. Closing 3×3   : dilate(1) + erode(1) on edges (gap repair).
//!   5. Border flood  : 4-connected from all 4 image borders; treat
//!                      closed edges as barriers AND transparent
//!                      pixels as already-outside (no leak through
//!                      pre-keyed holes).
//!   6. Auto-protect  : 255 where alpha > 0 AND NOT reached by flood
//!                      (so the interior AND the silhouette pixels
//!                      themselves are force-kept). Transparent or
//!                      reached pixels stay 0 — the chroma backend
//!                      decides those.
//!
//! Why this fixes "bege interior some" (Enio 2026-05-26): the chroma
//! distance alone classifies an interior beige patch identical to bg
//! beige as background, no matter what the geometry says. The flood
//! from the IMAGE border can't reach that patch because the subject's
//! visual edges (face contour, line-art) form a closed barrier — so
//! the patch is locked as foreground regardless of colour similarity.
//!
//! References:
//! - SCAFF (Scan-flood Fill), Liu et al. 2019,
//!   <https://arxiv.org/pdf/1906.03366>.
//! - Otsu, "A threshold selection method from gray-level histograms",
//!   IEEE T-SMC 9(1), 1979.
//! - Edge-bounded flood fill — canonical pattern, scikit-image:
//!   <https://scikit-image.org/docs/dev/auto_examples/segmentation/plot_floodfill.html>

#![allow(clippy::too_many_arguments)]

/// Detect the subject interior + silhouette and write it as a
/// force-keep mask into `out_protect`.
///
/// All slices must be sized `(w * h)` (one byte/pixel for masks,
/// u16 for sobel_mag). `rgba` is straight-alpha RGBA8.
pub fn detect_subject_interior(
    rgba: &[u8],
    w: u32,
    h: u32,
    luma: &mut [u8],
    sobel_mag: &mut [u16],
    edge_a: &mut [u8],
    edge_b: &mut [u8],
    visited: &mut [u8],
    queue: &mut Vec<u32>,
    out_protect: &mut [u8],
) {
    let wi = w as usize;
    let hi = h as usize;
    let n = wi * hi;
    debug_assert_eq!(rgba.len(), n * 4);
    debug_assert!(luma.len() >= n);
    debug_assert!(sobel_mag.len() >= n);
    debug_assert!(edge_a.len() >= n);
    debug_assert!(edge_b.len() >= n);
    debug_assert!(visited.len() >= n);
    debug_assert!(out_protect.len() >= n);

    if wi < 3 || hi < 3 {
        for v in out_protect.iter_mut().take(n) {
            *v = 0;
        }
        return;
    }

    compute_luma(rgba, n, luma);
    sobel_magnitude_l1(luma, wi, hi, sobel_mag);
    let threshold = otsu_threshold_u16(sobel_mag, n);
    threshold_to_mask(sobel_mag, n, threshold, edge_a);
    // Closing = dilate then erode. Use `edge_b` as scratch for the
    // dilate output, then erode back into `edge_a`.
    dilate3x3(edge_a, edge_b, wi, hi);
    erode3x3(edge_b, edge_a, wi, hi);
    flood_from_border(edge_a, rgba, wi, hi, visited, queue);
    finalise_protect(rgba, visited, n, out_protect);
}

/// Y' = 0.299R + 0.587G + 0.114B, integer-approx (Rec.601).
fn compute_luma(rgba: &[u8], n: usize, out: &mut [u8]) {
    for i in 0..n {
        let base = i * 4;
        let r = rgba[base] as u32;
        let g = rgba[base + 1] as u32;
        let b = rgba[base + 2] as u32;
        // 77 + 150 + 29 = 256 → divide by 256 (>> 8) for [0..255].
        out[i] = ((r * 77 + g * 150 + b * 29) >> 8) as u8;
    }
}

/// L1 Sobel magnitude (|gx| + |gy|) — cheaper than L2 and visually
/// indistinguishable after threshold. Border row/col left at 0.
fn sobel_magnitude_l1(luma: &[u8], w: usize, h: usize, out: &mut [u16]) {
    for v in out.iter_mut().take(w * h) {
        *v = 0;
    }
    for y in 1..h - 1 {
        for x in 1..w - 1 {
            let i = y * w + x;
            let tl = luma[i - w - 1] as i32;
            let t = luma[i - w] as i32;
            let tr = luma[i - w + 1] as i32;
            let l = luma[i - 1] as i32;
            let r = luma[i + 1] as i32;
            let bl = luma[i + w - 1] as i32;
            let bot = luma[i + w] as i32;
            let br = luma[i + w + 1] as i32;
            let gx = (tr + 2 * r + br) - (tl + 2 * l + bl);
            let gy = (bl + 2 * bot + br) - (tl + 2 * t + tr);
            let mag = gx.unsigned_abs() + gy.unsigned_abs();
            out[i] = mag.min(u16::MAX as u32) as u16;
        }
    }
}

/// Otsu's threshold over a 256-bin histogram of `mag`. Returns the
/// magnitude threshold (NOT the bin index) — so the binarisation step
/// can compare directly without rescaling.
fn otsu_threshold_u16(mag: &[u16], n: usize) -> u16 {
    let mut max_mag: u16 = 0;
    for &m in mag.iter().take(n) {
        if m > max_mag {
            max_mag = m;
        }
    }
    if max_mag == 0 {
        return u16::MAX; // No edges at all → threshold above max so mask is all-zero.
    }
    let bin_scale = max_mag as f32 / 255.0;
    let mut hist = [0u32; 256];
    for &m in mag.iter().take(n) {
        let bin = ((m as f32 / bin_scale).round() as usize).min(255);
        hist[bin] += 1;
    }
    let total = n as u64;
    let mut sum_all: u64 = 0;
    for (i, &c) in hist.iter().enumerate() {
        sum_all += (i as u64) * (c as u64);
    }
    let mut sum_b: u64 = 0;
    let mut w_b: u64 = 0;
    let mut max_var: f64 = -1.0;
    let mut best_bin: usize = 0;
    for (t, &c) in hist.iter().enumerate() {
        w_b += c as u64;
        if w_b == 0 {
            continue;
        }
        let w_f = total - w_b;
        if w_f == 0 {
            break;
        }
        sum_b += (t as u64) * (c as u64);
        let m_b = sum_b as f64 / w_b as f64;
        let m_f = (sum_all - sum_b) as f64 / w_f as f64;
        let diff = m_b - m_f;
        let var = (w_b as f64) * (w_f as f64) * diff * diff;
        if var > max_var {
            max_var = var;
            best_bin = t;
        }
    }
    ((best_bin as f32) * bin_scale).round().clamp(1.0, max_mag as f32) as u16
}

fn threshold_to_mask(mag: &[u16], n: usize, threshold: u16, out: &mut [u8]) {
    for i in 0..n {
        out[i] = if mag[i] >= threshold { 1 } else { 0 };
    }
}

fn dilate3x3(src: &[u8], dst: &mut [u8], w: usize, h: usize) {
    for y in 0..h {
        let y_min = y.saturating_sub(1);
        let y_max = (y + 1).min(h - 1);
        for x in 0..w {
            let x_min = x.saturating_sub(1);
            let x_max = (x + 1).min(w - 1);
            let mut v = 0u8;
            'outer: for yy in y_min..=y_max {
                let row = yy * w;
                for xx in x_min..=x_max {
                    if src[row + xx] != 0 {
                        v = 1;
                        break 'outer;
                    }
                }
            }
            dst[y * w + x] = v;
        }
    }
}

fn erode3x3(src: &[u8], dst: &mut [u8], w: usize, h: usize) {
    for y in 0..h {
        let y_min = y.saturating_sub(1);
        let y_max = (y + 1).min(h - 1);
        for x in 0..w {
            let x_min = x.saturating_sub(1);
            let x_max = (x + 1).min(w - 1);
            let mut v = 1u8;
            'outer: for yy in y_min..=y_max {
                let row = yy * w;
                for xx in x_min..=x_max {
                    if src[row + xx] == 0 {
                        v = 0;
                        break 'outer;
                    }
                }
            }
            dst[y * w + x] = v;
        }
    }
}

/// 4-connected flood from every border pixel that is not (a) on a
/// closed edge or (b) already transparent. Marks `visited[i] = 1` for
/// every reached pixel.
fn flood_from_border(
    edge: &[u8],
    rgba: &[u8],
    w: usize,
    h: usize,
    visited: &mut [u8],
    queue: &mut Vec<u32>,
) {
    for v in visited.iter_mut().take(w * h) {
        *v = 0;
    }
    queue.clear();

    let seed = |x: usize, y: usize, queue: &mut Vec<u32>, visited: &mut [u8]| {
        let i = y * w + x;
        if visited[i] != 0 {
            return;
        }
        if edge[i] != 0 {
            return;
        }
        if rgba[i * 4 + 3] == 0 {
            // Already transparent — treat as exterior (mark visited so
            // it doesn't end up in the interior mask, but DON'T expand
            // from it: the existing chroma path owns transparent
            // bookkeeping; adding more reachable pixels via transparent
            // bridges would let bg leaks survive flood barriers).
            visited[i] = 1;
            return;
        }
        visited[i] = 1;
        queue.push(i as u32);
    };

    for x in 0..w {
        seed(x, 0, queue, visited);
        seed(x, h - 1, queue, visited);
    }
    for y in 0..h {
        seed(0, y, queue, visited);
        seed(w - 1, y, queue, visited);
    }

    while let Some(idx) = queue.pop() {
        let i = idx as usize;
        let x = i % w;
        let y = i / w;
        let try_neighbour = |ni: usize, queue: &mut Vec<u32>, visited: &mut [u8]| {
            if visited[ni] != 0 {
                return;
            }
            if edge[ni] != 0 {
                return;
            }
            if rgba[ni * 4 + 3] == 0 {
                visited[ni] = 1;
                return;
            }
            visited[ni] = 1;
            queue.push(ni as u32);
        };
        if x > 0 {
            try_neighbour(i - 1, queue, visited);
        }
        if x + 1 < w {
            try_neighbour(i + 1, queue, visited);
        }
        if y > 0 {
            try_neighbour(i - w, queue, visited);
        }
        if y + 1 < h {
            try_neighbour(i + w, queue, visited);
        }
    }
}

/// `auto_protect = 255` where the pixel is opaque AND not reached by
/// the border flood. Includes silhouette edge pixels (they're barriers,
/// so flood never visits them — their `visited` stays 0 too) so the
/// outline itself is force-kept.
fn finalise_protect(rgba: &[u8], visited: &[u8], n: usize, out: &mut [u8]) {
    for i in 0..n {
        let opaque = rgba[i * 4 + 3] > 0;
        out[i] = if opaque && visited[i] == 0 { 255 } else { 0 };
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Solid rectangle on a uniform background. The auto-protect mask
    /// must mark the rectangle interior as 255 and the surrounding
    /// border as 0.
    #[test]
    fn detect_locks_solid_subject_interior() {
        let w = 40u32;
        let h = 40u32;
        let n = (w as usize) * (h as usize);
        let mut rgba = vec![0u8; n * 4];
        // Beige bg.
        for i in 0..n {
            rgba[i * 4] = 220;
            rgba[i * 4 + 1] = 200;
            rgba[i * 4 + 2] = 170;
            rgba[i * 4 + 3] = 255;
        }
        // Dark subject 20×20 in the middle.
        for y in 10..30 {
            for x in 10..30 {
                let i = y * (w as usize) + x;
                rgba[i * 4] = 40;
                rgba[i * 4 + 1] = 40;
                rgba[i * 4 + 2] = 40;
            }
        }
        let mut luma = vec![0u8; n];
        let mut mag = vec![0u16; n];
        let mut e_a = vec![0u8; n];
        let mut e_b = vec![0u8; n];
        let mut vis = vec![0u8; n];
        let mut q = Vec::with_capacity(n);
        let mut prot = vec![0u8; n];
        detect_subject_interior(
            &rgba, w, h, &mut luma, &mut mag, &mut e_a, &mut e_b, &mut vis, &mut q, &mut prot,
        );
        // Centre of the dark block must be protected.
        let centre = 20 * 40 + 20;
        assert_eq!(prot[centre], 255, "subject interior must be locked");
        // A border bg corner must NOT be protected.
        assert_eq!(prot[0], 0, "corner bg must remain unlocked");
        // Block area ≥ 90% protected (a few rim pixels may flip).
        let mut locked = 0;
        for y in 11..29 {
            for x in 11..29 {
                if prot[y * 40 + x] != 0 {
                    locked += 1;
                }
            }
        }
        let block_area = 18 * 18;
        assert!(
            locked >= (block_area * 9 / 10),
            "≥90% of the subject interior must be locked, got {locked}/{block_area}"
        );
    }

    /// Same-colour interior patch INSIDE the subject. This is the case
    /// the user hit (bege interno some): a region with the bg colour
    /// living inside a closed subject outline. With edge-aware
    /// protection on, the patch must still be locked, even though its
    /// colour is identical to the bg.
    #[test]
    fn protects_interior_patch_matching_bg_colour() {
        let w = 40u32;
        let h = 40u32;
        let n = (w as usize) * (h as usize);
        let bg = [220u8, 200, 170];
        let mut rgba = vec![0u8; n * 4];
        for i in 0..n {
            rgba[i * 4] = bg[0];
            rgba[i * 4 + 1] = bg[1];
            rgba[i * 4 + 2] = bg[2];
            rgba[i * 4 + 3] = 255;
        }
        // Subject = dark outline 20×20 (rim only, 1 px thick).
        for y in 10..30 {
            for x in 10..30 {
                let on_rim = x == 10 || x == 29 || y == 10 || y == 29;
                if on_rim {
                    let i = y * 40 + x;
                    rgba[i * 4] = 20;
                    rgba[i * 4 + 1] = 20;
                    rgba[i * 4 + 2] = 20;
                }
            }
        }
        // Interior of the rim = same beige as the bg (the trap case).
        let mut luma = vec![0u8; n];
        let mut mag = vec![0u16; n];
        let mut e_a = vec![0u8; n];
        let mut e_b = vec![0u8; n];
        let mut vis = vec![0u8; n];
        let mut q = Vec::with_capacity(n);
        let mut prot = vec![0u8; n];
        detect_subject_interior(
            &rgba, w, h, &mut luma, &mut mag, &mut e_a, &mut e_b, &mut vis, &mut q, &mut prot,
        );
        // Interior patch (same colour as bg) must be PROTECTED — the
        // chroma backend alone would have classified it as bg, but the
        // silhouette holds it.
        let inside = 20 * 40 + 20;
        assert_eq!(
            prot[inside], 255,
            "interior beige patch (same as bg) must be locked by silhouette"
        );
        // Outside the rim, the same beige is NOT locked.
        let outside = 5 * 40 + 5;
        assert_eq!(
            prot[outside], 0,
            "bg outside the silhouette must remain unlocked"
        );
    }

    /// A degenerate tiny image must not panic and must return all-zero
    /// — there's no meaningful silhouette to detect under 3 px on a
    /// side (the Sobel kernel needs ≥ 3×3).
    #[test]
    fn under_3x3_image_is_silently_inert() {
        for (w, h) in [(0u32, 0u32), (1, 5), (5, 1), (2, 2)] {
            let n = (w as usize) * (h as usize);
            let rgba = vec![255u8; n.max(1) * 4];
            let mut luma = vec![0u8; n];
            let mut mag = vec![0u16; n];
            let mut e_a = vec![0u8; n];
            let mut e_b = vec![0u8; n];
            let mut vis = vec![0u8; n];
            let mut q = Vec::with_capacity(n.max(1));
            let mut prot = vec![0u8; n];
            detect_subject_interior(
                &rgba[..n * 4],
                w,
                h,
                &mut luma,
                &mut mag,
                &mut e_a,
                &mut e_b,
                &mut vis,
                &mut q,
                &mut prot,
            );
            assert!(
                prot.iter().all(|&v| v == 0),
                "tiny image ({w}×{h}) must produce empty mask"
            );
        }
    }
}
