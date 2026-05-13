//! K-means border-color detection.
//!
//! Why k-means instead of a plain RGB average (legacy): when the
//! border band has more than one real color (gradient, checker, vignette),
//! the mean is a third color that doesn't exist in the image, and the
//! mask falls back to a degraded color-key. K-means with k=4 finds the
//! actual cluster centers and returns them — typically the user gets
//! the dominant background color cleanly even on busy backgrounds.
//!
//! Algorithm: k-means++ seeding + Lloyd iterations in OKLab space.
//! Distance is `oklab_dist_sq` (consistent with the masker that
//! consumes these centers). Iterations cap at 8 — converged or not,
//! 8 passes over a ~few-thousand sample band cost < 1 ms.

use super::oklab::{Oklab, oklab_dist_sq, srgb_to_oklab};
use super::params::RgbColor;

/// Configuration knobs for `detect_border_colors`. Defaults match the
/// legacy auto behavior except `k` (legacy did mean → equivalent to
/// k=1 without the spread guarantee).
#[derive(Copy, Clone, Debug)]
pub struct BorderDetectOpts {
    /// Number of clusters. Range 1..=8. Default 4.
    pub k: u8,
    /// Alpha threshold under which a pixel is treated as transparent
    /// and skipped (matches legacy `200` lower bound). 0..=255.
    pub min_alpha: u8,
    /// Maximum Lloyd iterations. Default 8.
    pub max_iters: u8,
    /// Seed for k-means++ candidate selection. Fixed seed → identical
    /// output across runs and platforms (HR-5 spirit: even though
    /// background removal isn't simulation, reproducibility helps
    /// tests and user-visible determinism).
    pub seed: u64,
}

impl Default for BorderDetectOpts {
    fn default() -> Self {
        Self {
            k: 4,
            min_alpha: 200,
            max_iters: 8,
            seed: 0x9E37_79B9_7F4A_7C15,
        }
    }
}

/// Detect 1..=k dominant border colors. Returns at least one color
/// (white if the border was entirely transparent — matches legacy).
/// Sorted by cluster population (largest first), so the Integrator
/// can pick the top-N for display.
pub fn detect_border_colors(rgba: &[u8], w: u32, h: u32, opts: BorderDetectOpts) -> Vec<RgbColor> {
    let samples = collect_border_samples(rgba, w, h, opts.min_alpha);
    if samples.is_empty() {
        return vec![RgbColor::new(255, 255, 255)];
    }

    let k = (opts.k as usize).clamp(1, 8).min(samples.len()).max(1);

    // K-means++ initialization.
    let mut rng = SplitMix64::new(opts.seed);
    let mut centers: Vec<Oklab> = Vec::with_capacity(k);
    let first = samples[(rng.next_u64() as usize) % samples.len()].lab;
    centers.push(first);

    let mut d2: Vec<f32> = vec![f32::INFINITY; samples.len()];
    for _ in 1..k {
        let mut sum = 0.0_f32;
        for (i, s) in samples.iter().enumerate() {
            let d = oklab_dist_sq(s.lab, *centers.last().unwrap());
            if d < d2[i] {
                d2[i] = d;
            }
            sum += d2[i];
        }
        if sum <= f32::EPSILON {
            // Already covered all distinct samples — duplicate the
            // last center; clamp() below will drop duplicates anyway.
            centers.push(*centers.last().unwrap());
            continue;
        }
        let target = (rng.next_f32() * sum).min(sum - f32::EPSILON);
        let mut acc = 0.0_f32;
        let mut picked = samples.len() - 1;
        for (i, &d) in d2.iter().enumerate() {
            acc += d;
            if acc >= target {
                picked = i;
                break;
            }
        }
        centers.push(samples[picked].lab);
    }

    // Lloyd's iterations.
    let mut assign: Vec<u8> = vec![0; samples.len()];
    let mut counts: Vec<u32> = vec![0; k];
    let mut sums: Vec<[f32; 3]> = vec![[0.0; 3]; k];
    let mut rgb_sums: Vec<[u64; 3]> = vec![[0; 3]; k];

    for _ in 0..opts.max_iters {
        let mut any_moved = false;
        for (i, s) in samples.iter().enumerate() {
            let mut best = 0u8;
            let mut best_d = f32::INFINITY;
            for (c_idx, c) in centers.iter().enumerate() {
                let d = oklab_dist_sq(s.lab, *c);
                if d < best_d {
                    best_d = d;
                    best = c_idx as u8;
                }
            }
            if assign[i] != best {
                assign[i] = best;
                any_moved = true;
            }
        }

        // Recompute centers from cluster means (in OKLab).
        for s in sums.iter_mut() {
            *s = [0.0; 3];
        }
        for c in counts.iter_mut() {
            *c = 0;
        }
        for (i, s) in samples.iter().enumerate() {
            let c = assign[i] as usize;
            sums[c][0] += s.lab.l;
            sums[c][1] += s.lab.a;
            sums[c][2] += s.lab.b;
            counts[c] += 1;
        }
        for (c_idx, c) in centers.iter_mut().enumerate() {
            if counts[c_idx] > 0 {
                let n = counts[c_idx] as f32;
                *c = Oklab {
                    l: sums[c_idx][0] / n,
                    a: sums[c_idx][1] / n,
                    b: sums[c_idx][2] / n,
                };
            }
        }

        if !any_moved {
            break;
        }
    }

    // Recover RGB centers by averaging the source RGB of each
    // cluster's members (more faithful than converting OKLab back —
    // round-trip introduces drift).
    for r in rgb_sums.iter_mut() {
        *r = [0; 3];
    }
    for c in counts.iter_mut() {
        *c = 0;
    }
    for (i, s) in samples.iter().enumerate() {
        let c = assign[i] as usize;
        rgb_sums[c][0] += s.rgb[0] as u64;
        rgb_sums[c][1] += s.rgb[1] as u64;
        rgb_sums[c][2] += s.rgb[2] as u64;
        counts[c] += 1;
    }

    let mut result: Vec<(u32, RgbColor)> = (0..k)
        .filter(|&i| counts[i] > 0)
        .map(|i| {
            let n = counts[i] as u64;
            (
                counts[i],
                RgbColor {
                    r: (rgb_sums[i][0] / n) as u8,
                    g: (rgb_sums[i][1] / n) as u8,
                    b: (rgb_sums[i][2] / n) as u8,
                },
            )
        })
        .collect();

    // Sort by cluster size (largest first).
    result.sort_by_key(|x| std::cmp::Reverse(x.0));

    // Deduplicate near-identical centers (post-merge of empty
    // clusters or seed collisions).
    let mut out: Vec<RgbColor> = Vec::with_capacity(result.len());
    for (_, c) in result {
        if !out.iter().any(|p| close(*p, c, 6)) {
            out.push(c);
        }
    }

    if out.is_empty() {
        out.push(RgbColor::new(255, 255, 255));
    }
    out
}

#[inline]
fn close(a: RgbColor, b: RgbColor, eps: i32) -> bool {
    (a.r as i32 - b.r as i32).abs() <= eps
        && (a.g as i32 - b.g as i32).abs() <= eps
        && (a.b as i32 - b.b as i32).abs() <= eps
}

#[derive(Copy, Clone)]
struct Sample {
    rgb: [u8; 3],
    lab: Oklab,
}

/// Adaptive border-band sampler. Picks a band 2..=10 px wide around
/// the image edge (matches legacy) and yields one sample per pixel
/// inside it, skipping the transparent ones.
fn collect_border_samples(rgba: &[u8], w: u32, h: u32, min_alpha: u8) -> Vec<Sample> {
    let (w, h) = (w as usize, h as usize);
    let border = ((w.min(h) as f32 * 0.02) as usize).clamp(2, 10);
    let mut out: Vec<Sample> = Vec::new();

    let push = |out: &mut Vec<Sample>, x: usize, y: usize| {
        let idx = (y * w + x) * 4;
        if idx + 3 >= rgba.len() {
            return;
        }
        let a = rgba[idx + 3];
        if a < min_alpha {
            return;
        }
        let r = rgba[idx];
        let g = rgba[idx + 1];
        let b = rgba[idx + 2];
        out.push(Sample {
            rgb: [r, g, b],
            lab: srgb_to_oklab(r, g, b),
        });
    };

    // Top + bottom bands.
    for x in 0..w {
        for dy in 0..border {
            push(&mut out, x, dy);
            if h > dy {
                push(&mut out, x, h - 1 - dy);
            }
        }
    }
    // Left + right bands (skip corners — already sampled).
    for y in border..h.saturating_sub(border) {
        for dx in 0..border {
            push(&mut out, dx, y);
            if w > dx {
                push(&mut out, w - 1 - dx, y);
            }
        }
    }

    out
}

/// Tiny SplitMix64 — deterministic PRNG, no dep. Same generator as
/// the Rust stdlib's `SmallRng` skipping crate adds. Plenty for
/// k-means++ candidate weights.
struct SplitMix64 {
    state: u64,
}

impl SplitMix64 {
    fn new(seed: u64) -> Self {
        Self {
            state: seed.wrapping_add(0x9E37_79B9_7F4A_7C15),
        }
    }
    fn next_u64(&mut self) -> u64 {
        let mut z = self.state;
        self.state = z.wrapping_add(0x9E37_79B9_7F4A_7C15);
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^ (z >> 31)
    }
    fn next_f32(&mut self) -> f32 {
        // 24 bits of mantissa → uniform in [0, 1).
        ((self.next_u64() >> 40) as f32) / (1u32 << 24) as f32
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn solid(w: u32, h: u32, r: u8, g: u8, b: u8) -> Vec<u8> {
        let mut buf = vec![0u8; (w * h * 4) as usize];
        for i in 0..(w * h) as usize {
            buf[i * 4] = r;
            buf[i * 4 + 1] = g;
            buf[i * 4 + 2] = b;
            buf[i * 4 + 3] = 255;
        }
        buf
    }

    #[test]
    fn uniform_border_returns_that_color() {
        let img = solid(32, 32, 240, 180, 200);
        let out = detect_border_colors(&img, 32, 32, BorderDetectOpts::default());
        assert!(!out.is_empty());
        let c = out[0];
        // ±2 for round-trip drift through OKLab + recomputation.
        assert!((c.r as i32 - 240).abs() < 3);
        assert!((c.g as i32 - 180).abs() < 3);
        assert!((c.b as i32 - 200).abs() < 3);
    }

    #[test]
    fn fully_transparent_border_falls_back_to_white() {
        let img = vec![0u8; 32 * 32 * 4];
        let out = detect_border_colors(&img, 32, 32, BorderDetectOpts::default());
        assert_eq!(out.len(), 1);
        assert_eq!(out[0], RgbColor::new(255, 255, 255));
    }

    #[test]
    fn two_color_border_yields_two_clusters() {
        // Left half border red, right half blue.
        let (w, h) = (40u32, 40u32);
        let mut img = solid(w, h, 0, 0, 0);
        for y in 0..h as usize {
            for x in 0..w as usize {
                let idx = (y * w as usize + x) * 4;
                let on_border = x < 2 || x >= w as usize - 2 || y < 2 || y >= h as usize - 2;
                if on_border {
                    if x < w as usize / 2 {
                        img[idx] = 240;
                        img[idx + 1] = 30;
                        img[idx + 2] = 30;
                    } else {
                        img[idx] = 30;
                        img[idx + 1] = 30;
                        img[idx + 2] = 240;
                    }
                    img[idx + 3] = 255;
                } else {
                    img[idx + 3] = 0;
                }
            }
        }
        let out = detect_border_colors(&img, w, h, BorderDetectOpts::default());
        assert!(out.len() >= 2, "expected ≥2 clusters, got {}", out.len());
        // Both source colors should appear in the top-2.
        let has_red = out.iter().any(|c| c.r > 200 && c.g < 80 && c.b < 80);
        let has_blue = out.iter().any(|c| c.r < 80 && c.g < 80 && c.b > 200);
        assert!(has_red && has_blue);
    }

    #[test]
    fn k_one_returns_single_cluster() {
        let img = solid(32, 32, 100, 150, 200);
        let opts = BorderDetectOpts {
            k: 1,
            ..Default::default()
        };
        let out = detect_border_colors(&img, 32, 32, opts);
        assert_eq!(out.len(), 1);
    }

    #[test]
    fn deterministic_across_runs() {
        let img = solid(32, 32, 123, 45, 67);
        let a = detect_border_colors(&img, 32, 32, BorderDetectOpts::default());
        let b = detect_border_colors(&img, 32, 32, BorderDetectOpts::default());
        assert_eq!(a, b);
    }
}
