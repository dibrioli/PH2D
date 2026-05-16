//! Gaussian Mixture Model (5 components × 3-channel RGB) used as
//! the per-side colour model in GrabCut. Mirrors OpenCV's `cv::GMM`
//! shape: 5 components, full 3×3 covariance per component, weight
//! prior, deterministic k-means++ init + Lloyd iterations to
//! produce the initial hard assignment, then per-component mean +
//! cov re-estimation from that hard assignment.
//!
//! Public API:
//! - [`Gmm5::default()`] — zero-initialised model.
//! - [`Gmm5::init_kmeans_pp(pixels, seed)`] — seed components from
//!   the input pixel cloud. Sub-samples to `INIT_BUDGET` for speed.
//!   Each pixel ends up hard-assigned to one of 5 components; mean,
//!   covariance, and mixing weight are derived from those
//!   assignments.
//! - [`Gmm5::fit(pixels, assignments)`] — re-estimate the model
//!   from a fresh hard-assignment array (one component id per
//!   pixel). Used between GrabCut iterations.
//! - [`Gmm5::assign_component(rgb) -> usize`] — return the index of
//!   the component with the highest single-component likelihood
//!   for `rgb`.
//! - [`Gmm5::neg_log_prob(rgb)`] — `-log p(rgb | model)`, weighted
//!   sum over components, clamped at `LOG_PROB_CLAMP`. This is the
//!   value the graph cut uses as t-link to source / sink.
//!
//! ## Determinism / HR-5
//!
//! All RNG (sub-sample + k-means++ candidate selection) uses
//! `SplitMix64` seeded from the caller-supplied `u64`. Lloyd's
//! iterations are serial. Sum order is deterministic by index.
//! The same input + same seed produce bit-identical output.

// Re-export of the algorithm-wide constants the orchestrator uses.

/// Cap on samples consumed by the init pass. Above this we
/// uniformly subsample (deterministic, stride-based).
pub const INIT_BUDGET: usize = 50_000;
/// Components per side (Rother et al.).
pub const COMPONENTS: usize = 5;
/// Diagonal jitter added when the per-component covariance
/// determinant drops below `SINGULAR_DET` (OpenCV PR #27120).
/// `1.0` is chosen for raw `[0, 255]` channel space — a 1-pixel
/// blur in each channel is invisible but kills singularity.
pub const SINGULAR_DIAG_BUMP: f32 = 1.0;
/// Determinant threshold below which the covariance is treated as
/// singular and the diagonal bump fires.
pub const SINGULAR_DET: f32 = 1.0;
/// `-log p` clamp: any per-pixel likelihood below
/// `exp(-LOG_PROB_CLAMP)` reports as `LOG_PROB_CLAMP` so that the
/// graph cut's t-link weights stay finite even for outliers.
pub const LOG_PROB_CLAMP: f32 = 30.0;
/// Maximum Lloyd iterations during the initial k-means assignment.
const KMEANS_ITERS: u32 = 10;

// ---------------------------------------------------------------
// PRNG
// ---------------------------------------------------------------

/// SplitMix64 — small, fast, deterministic PRNG. Used for the
/// k-means++ candidate-pick draws and the sub-sample stride
/// origin. No external `rand` dep.
#[inline]
fn splitmix64(state: &mut u64) -> u64 {
    *state = state.wrapping_add(0x9E37_79B9_7F4A_7C15);
    let mut z = *state;
    z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    z ^ (z >> 31)
}

#[inline]
fn next_f01(state: &mut u64) -> f32 {
    // Take the top 24 bits → unit-interval f32.
    ((splitmix64(state) >> 40) as f32) / ((1u32 << 24) as f32)
}

// ---------------------------------------------------------------
// Public type
// ---------------------------------------------------------------

/// 5-component, 3-channel Gaussian Mixture Model with full
/// covariance.
#[derive(Clone, Debug, Default)]
pub struct Gmm5 {
    /// `[mean_r, mean_g, mean_b]` per component.
    pub means: [[f32; 3]; COMPONENTS],
    /// Full 3×3 covariance per component, row-major.
    pub covs: [[f32; 9]; COMPONENTS],
    /// Cached inverse covariance, row-major.
    pub inv_covs: [[f32; 9]; COMPONENTS],
    /// Cached `1.0 / (sqrt((2π)³ · det))` normalising factor per
    /// component (so likelihood = `norm · exp(-0.5 · Mahalanobis²)`).
    pub norms: [f32; COMPONENTS],
    /// Mixing weight (prior) per component, summing to 1.
    pub weights: [f32; COMPONENTS],
    /// Per-component sample count from the most recent fit. Empty
    /// (count = 0) components have `weight = 0` and are
    /// effectively ignored during `neg_log_prob`.
    pub counts: [u32; COMPONENTS],
}

impl Gmm5 {
    // -------------------------------------------------------------
    // Init: k-means++ + Lloyd, then fit
    // -------------------------------------------------------------

    /// Seed the model from `pixels` (RGB packed `[R,G,B,R,G,B,…]`,
    /// length `n*3`) using deterministic k-means++ selection of
    /// initial centroids + `KMEANS_ITERS` Lloyd iterations, then
    /// fit the GMM parameters from the resulting hard assignment.
    pub fn init_kmeans_pp(&mut self, pixels: &[u8], seed: u64) {
        let n_total = pixels.len() / 3;
        if n_total == 0 {
            *self = Gmm5::default();
            return;
        }

        // Stride-subsample up to INIT_BUDGET — deterministic order.
        let stride = (n_total / INIT_BUDGET).max(1);
        let n_samples = n_total.div_ceil(stride).min(INIT_BUDGET);
        // `samples` lives on the stack indirectly via repeated reads
        // from `pixels`; we don't materialise it. Instead, indices
        // `i_total = i_sample * stride` give us the source pixel.
        let sample_rgb = |i: usize| -> [f32; 3] {
            let t = (i * stride).min(n_total - 1);
            [
                pixels[t * 3] as f32,
                pixels[t * 3 + 1] as f32,
                pixels[t * 3 + 2] as f32,
            ]
        };

        // --- 1) k-means++ centroid selection ----------------------
        let mut centroids: [[f32; 3]; COMPONENTS] = [[0.0; 3]; COMPONENTS];
        let mut rng = seed;
        // First centroid: pick a sample uniformly at random.
        let first_idx = (splitmix64(&mut rng) as usize) % n_samples;
        centroids[0] = sample_rgb(first_idx);

        // Distance² of every sample to the nearest already-chosen
        // centroid. We re-compute on every centroid pick.
        let mut d_sq = vec![0.0f32; n_samples];
        for (i, d) in d_sq.iter_mut().enumerate() {
            *d = dist_sq(sample_rgb(i), centroids[0]);
        }

        for c in centroids.iter_mut().skip(1) {
            let total: f32 = d_sq.iter().sum();
            let pick_value = next_f01(&mut rng) * total.max(f32::MIN_POSITIVE);
            // Walk the cumulative distribution.
            let mut acc = 0.0f32;
            let mut chosen = n_samples - 1;
            for (i, &d) in d_sq.iter().enumerate() {
                acc += d;
                if acc >= pick_value {
                    chosen = i;
                    break;
                }
            }
            let new_centroid = sample_rgb(chosen);
            *c = new_centroid;
            // Update d_sq with the new centroid.
            for (i, d) in d_sq.iter_mut().enumerate() {
                let new_d = dist_sq(sample_rgb(i), new_centroid);
                if new_d < *d {
                    *d = new_d;
                }
            }
        }

        // --- 2) Lloyd iterations on the sub-sample ----------------
        let mut assign = vec![0u8; n_samples];
        for _ in 0..KMEANS_ITERS {
            // Assign.
            let mut changes = 0u32;
            for (i, a) in assign.iter_mut().enumerate() {
                let p = sample_rgb(i);
                let mut best_k = 0;
                let mut best_d = f32::INFINITY;
                for (k, c) in centroids.iter().enumerate() {
                    let d = dist_sq(p, *c);
                    if d < best_d {
                        best_d = d;
                        best_k = k;
                    }
                }
                if *a != best_k as u8 {
                    *a = best_k as u8;
                    changes += 1;
                }
            }
            if changes == 0 {
                break;
            }
            // Update centroids from assignments.
            let mut sums = [[0.0f32; 3]; COMPONENTS];
            let mut counts = [0u32; COMPONENTS];
            for (i, &a) in assign.iter().enumerate() {
                let k = a as usize;
                let p = sample_rgb(i);
                sums[k][0] += p[0];
                sums[k][1] += p[1];
                sums[k][2] += p[2];
                counts[k] += 1;
            }
            for (k, c) in centroids.iter_mut().enumerate() {
                if counts[k] > 0 {
                    let inv = 1.0 / counts[k] as f32;
                    *c = [sums[k][0] * inv, sums[k][1] * inv, sums[k][2] * inv];
                }
                // If a component went empty, leave its centroid
                // where it was — the next iteration may pull
                // pixels in. (OpenCV does the same.)
            }
        }

        // --- 3) Re-fit GMM from the FULL pixel set, hard-assigned
        //         to the nearest centroid.
        let mut sums = [[0.0f64; 3]; COMPONENTS];
        let mut sums_sq = [[0.0f64; 9]; COMPONENTS]; // outer-product accumulators
        let mut counts = [0u64; COMPONENTS];
        for t in 0..n_total {
            let r = pixels[t * 3] as f32;
            let g = pixels[t * 3 + 1] as f32;
            let b = pixels[t * 3 + 2] as f32;
            let p = [r, g, b];
            // Assign to nearest centroid.
            let mut best_k = 0usize;
            let mut best_d = f32::INFINITY;
            for (k, c) in centroids.iter().enumerate() {
                let d = dist_sq(p, *c);
                if d < best_d {
                    best_d = d;
                    best_k = k;
                }
            }
            // Accumulate. f64 to avoid catastrophic cancellation
            // when computing cov = E[xxᵀ] − E[x]·E[x]ᵀ at 4k scale.
            let pr = r as f64;
            let pg = g as f64;
            let pb = b as f64;
            sums[best_k][0] += pr;
            sums[best_k][1] += pg;
            sums[best_k][2] += pb;
            sums_sq[best_k][0] += pr * pr;
            sums_sq[best_k][1] += pr * pg;
            sums_sq[best_k][2] += pr * pb;
            sums_sq[best_k][3] += pg * pr;
            sums_sq[best_k][4] += pg * pg;
            sums_sq[best_k][5] += pg * pb;
            sums_sq[best_k][6] += pb * pr;
            sums_sq[best_k][7] += pb * pg;
            sums_sq[best_k][8] += pb * pb;
            counts[best_k] += 1;
        }
        self.finalise(&sums, &sums_sq, &counts, n_total as u64);
    }

    /// Re-estimate the model from a caller-supplied hard
    /// assignment. `pixels` is RGB packed; `assignments[i]` ∈
    /// `0..COMPONENTS` says which component each pixel belongs to.
    /// `len(assignments) == len(pixels) / 3`.
    pub fn fit(&mut self, pixels: &[u8], assignments: &[u8]) {
        let n_total = pixels.len() / 3;
        debug_assert_eq!(assignments.len(), n_total);

        let mut sums = [[0.0f64; 3]; COMPONENTS];
        let mut sums_sq = [[0.0f64; 9]; COMPONENTS];
        let mut counts = [0u64; COMPONENTS];
        for t in 0..n_total {
            let k = assignments[t] as usize;
            if k >= COMPONENTS {
                continue;
            }
            let pr = pixels[t * 3] as f64;
            let pg = pixels[t * 3 + 1] as f64;
            let pb = pixels[t * 3 + 2] as f64;
            sums[k][0] += pr;
            sums[k][1] += pg;
            sums[k][2] += pb;
            sums_sq[k][0] += pr * pr;
            sums_sq[k][1] += pr * pg;
            sums_sq[k][2] += pr * pb;
            sums_sq[k][3] += pg * pr;
            sums_sq[k][4] += pg * pg;
            sums_sq[k][5] += pg * pb;
            sums_sq[k][6] += pb * pr;
            sums_sq[k][7] += pb * pg;
            sums_sq[k][8] += pb * pb;
            counts[k] += 1;
        }
        self.finalise(&sums, &sums_sq, &counts, n_total as u64);
    }

    /// Compute final mean, covariance, inverse covariance, and
    /// normalising factor for each component from accumulated
    /// sums.
    fn finalise(
        &mut self,
        sums: &[[f64; 3]; COMPONENTS],
        sums_sq: &[[f64; 9]; COMPONENTS],
        counts: &[u64; COMPONENTS],
        n_total: u64,
    ) {
        for k in 0..COMPONENTS {
            let n = counts[k];
            self.counts[k] = n.min(u32::MAX as u64) as u32;
            if n == 0 {
                self.means[k] = [0.0; 3];
                self.covs[k] = identity_3x3_scaled(SINGULAR_DIAG_BUMP);
                self.inv_covs[k] = identity_3x3_scaled(1.0 / SINGULAR_DIAG_BUMP);
                self.norms[k] = 0.0;
                self.weights[k] = 0.0;
                continue;
            }
            let inv = 1.0 / n as f64;
            let m = [sums[k][0] * inv, sums[k][1] * inv, sums[k][2] * inv];
            self.means[k] = [m[0] as f32, m[1] as f32, m[2] as f32];
            // E[xxᵀ] − mmᵀ
            let mut cov = [0.0f64; 9];
            for i in 0..3 {
                for j in 0..3 {
                    cov[i * 3 + j] = sums_sq[k][i * 3 + j] * inv - m[i] * m[j];
                }
            }
            // Singular guard.
            let det = det_3x3(&cov);
            if det < SINGULAR_DET as f64 {
                for d in 0..3 {
                    cov[d * 3 + d] += SINGULAR_DIAG_BUMP as f64;
                }
            }
            // Cast to f32 and store + invert.
            let cov_f32 = [
                cov[0] as f32,
                cov[1] as f32,
                cov[2] as f32,
                cov[3] as f32,
                cov[4] as f32,
                cov[5] as f32,
                cov[6] as f32,
                cov[7] as f32,
                cov[8] as f32,
            ];
            self.covs[k] = cov_f32;
            let det_f32 = det_3x3_f32(&cov_f32).max(SINGULAR_DET);
            self.inv_covs[k] = inv_3x3(&cov_f32, det_f32);
            // Normaliser for the multivariate Gaussian.
            // p(x) = 1/((2π)^(3/2) · sqrt(det)) · exp(-0.5 · Δᵀ Σ⁻¹ Δ)
            // We absorb the (2π)^(3/2) into the constant. For
            // GrabCut we only ever compare *relative* probabilities
            // between components, so dropping the constant is OK,
            // but we include it for correctness of `neg_log_prob`.
            const TWO_PI_3_2: f32 = 15.749_61; // (2π)^(3/2)
            self.norms[k] = 1.0 / (TWO_PI_3_2 * det_f32.sqrt());
            // Mixing weight = component sample share.
            self.weights[k] = (n as f64 / n_total as f64) as f32;
        }
    }

    // -------------------------------------------------------------
    // Likelihoods
    // -------------------------------------------------------------

    /// Return the index of the component with the highest single-
    /// component likelihood for `rgb`.
    pub fn assign_component(&self, rgb: [u8; 3]) -> usize {
        let p = [rgb[0] as f32, rgb[1] as f32, rgb[2] as f32];
        let mut best_k = 0usize;
        let mut best_log = f32::NEG_INFINITY;
        for k in 0..COMPONENTS {
            if self.weights[k] == 0.0 {
                continue;
            }
            // log p_k(x) = log(norm_k) - 0.5 · Δᵀ Σ⁻¹ Δ
            let m_dist = mahalanobis_sq(p, self.means[k], &self.inv_covs[k]);
            let log_p = self.norms[k].ln() - 0.5 * m_dist;
            if log_p > best_log {
                best_log = log_p;
                best_k = k;
            }
        }
        best_k
    }

    /// Return `-log p(rgb | model)` weighted across components,
    /// clamped at `LOG_PROB_CLAMP`. Used as t-link weight in the
    /// graph cut.
    pub fn neg_log_prob(&self, rgb: [u8; 3]) -> f32 {
        let p = [rgb[0] as f32, rgb[1] as f32, rgb[2] as f32];
        let mut total_p = 0.0f32;
        for k in 0..COMPONENTS {
            let w = self.weights[k];
            if w == 0.0 {
                continue;
            }
            let m_dist = mahalanobis_sq(p, self.means[k], &self.inv_covs[k]);
            // Likelihood from this component; weighted by prior.
            let p_k = self.norms[k] * (-0.5 * m_dist).exp();
            total_p += w * p_k;
        }
        if total_p <= 0.0 || !total_p.is_finite() {
            return LOG_PROB_CLAMP;
        }
        (-total_p.ln()).clamp(0.0, LOG_PROB_CLAMP)
    }
}

// ---------------------------------------------------------------
// 3×3 linear algebra helpers (Cramer's rule — no nalgebra dep)
// ---------------------------------------------------------------

#[inline]
fn dist_sq(a: [f32; 3], b: [f32; 3]) -> f32 {
    let dr = a[0] - b[0];
    let dg = a[1] - b[1];
    let db = a[2] - b[2];
    dr * dr + dg * dg + db * db
}

#[inline]
fn det_3x3(m: &[f64; 9]) -> f64 {
    m[0] * (m[4] * m[8] - m[5] * m[7]) - m[1] * (m[3] * m[8] - m[5] * m[6])
        + m[2] * (m[3] * m[7] - m[4] * m[6])
}

#[inline]
fn det_3x3_f32(m: &[f32; 9]) -> f32 {
    m[0] * (m[4] * m[8] - m[5] * m[7]) - m[1] * (m[3] * m[8] - m[5] * m[6])
        + m[2] * (m[3] * m[7] - m[4] * m[6])
}

/// Inverse of a 3×3 matrix given as row-major `[f32; 9]`. Caller
/// passes the determinant (caller floor it for stability).
fn inv_3x3(m: &[f32; 9], det: f32) -> [f32; 9] {
    let inv_det = 1.0 / det;
    [
        (m[4] * m[8] - m[5] * m[7]) * inv_det,
        (m[2] * m[7] - m[1] * m[8]) * inv_det,
        (m[1] * m[5] - m[2] * m[4]) * inv_det,
        (m[5] * m[6] - m[3] * m[8]) * inv_det,
        (m[0] * m[8] - m[2] * m[6]) * inv_det,
        (m[2] * m[3] - m[0] * m[5]) * inv_det,
        (m[3] * m[7] - m[4] * m[6]) * inv_det,
        (m[1] * m[6] - m[0] * m[7]) * inv_det,
        (m[0] * m[4] - m[1] * m[3]) * inv_det,
    ]
}

#[inline]
fn identity_3x3_scaled(s: f32) -> [f32; 9] {
    [s, 0.0, 0.0, 0.0, s, 0.0, 0.0, 0.0, s]
}

/// Mahalanobis distance² Δᵀ Σ⁻¹ Δ where Δ = `p − mean`.
#[inline]
fn mahalanobis_sq(p: [f32; 3], mean: [f32; 3], inv_cov: &[f32; 9]) -> f32 {
    let d0 = p[0] - mean[0];
    let d1 = p[1] - mean[1];
    let d2 = p[2] - mean[2];
    // Σ⁻¹ · Δ
    let v0 = inv_cov[0] * d0 + inv_cov[1] * d1 + inv_cov[2] * d2;
    let v1 = inv_cov[3] * d0 + inv_cov[4] * d1 + inv_cov[5] * d2;
    let v2 = inv_cov[6] * d0 + inv_cov[7] * d1 + inv_cov[8] * d2;
    // Δᵀ · v
    d0 * v0 + d1 * v1 + d2 * v2
}

// ---------------------------------------------------------------
// Tests
// ---------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_gmm_is_zero_initialised() {
        let g = Gmm5::default();
        assert_eq!(g.weights, [0.0; COMPONENTS]);
        assert_eq!(g.counts, [0u32; COMPONENTS]);
    }

    #[test]
    fn init_empty_pixels_keeps_defaults() {
        let mut g = Gmm5::default();
        g.init_kmeans_pp(&[], 42);
        assert_eq!(g.weights, [0.0; COMPONENTS]);
    }

    #[test]
    fn init_single_color_concentrates_one_component() {
        // 1000 identical pixels — only one component should accrue
        // mass; the others may carry 0 weight.
        let pixels: Vec<u8> = (0..1000)
            .flat_map(|_| [100u8, 150, 200].into_iter())
            .collect();
        let mut g = Gmm5::default();
        g.init_kmeans_pp(&pixels, 42);
        let total: u32 = g.counts.iter().sum();
        assert_eq!(total as usize, 1000);
        // Sum of weights ≈ 1.
        let w_sum: f32 = g.weights.iter().sum();
        assert!((w_sum - 1.0).abs() < 1e-4);
    }

    #[test]
    fn init_two_clusters_separates_means_meaningfully() {
        // 500 dark pixels + 500 bright pixels.
        let mut pixels = Vec::new();
        for _ in 0..500 {
            pixels.extend_from_slice(&[20u8, 20, 20]);
        }
        for _ in 0..500 {
            pixels.extend_from_slice(&[230u8, 230, 230]);
        }
        let mut g = Gmm5::default();
        g.init_kmeans_pp(&pixels, 7);
        // At least two components have non-zero weight, and their
        // means are far apart.
        let active: Vec<usize> = (0..COMPONENTS).filter(|&k| g.counts[k] > 0).collect();
        assert!(active.len() >= 2, "expected ≥2 active components");
        // Find the two extremes by mean luma.
        let mut sorted = active.clone();
        sorted.sort_by(|&a, &b| {
            g.means[a][0]
                .partial_cmp(&g.means[b][0])
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        let dark = g.means[*sorted.first().unwrap()];
        let bright = g.means[*sorted.last().unwrap()];
        assert!(bright[0] - dark[0] > 100.0);
    }

    #[test]
    fn neg_log_prob_finite_for_in_distribution_pixel() {
        let pixels: Vec<u8> = (0..500)
            .flat_map(|i| {
                let v = ((i % 200) + 30) as u8;
                [v, v, v].into_iter()
            })
            .collect();
        let mut g = Gmm5::default();
        g.init_kmeans_pp(&pixels, 1);
        // A pixel near the centre of the distribution should
        // produce a small `neg_log_prob`.
        let val = g.neg_log_prob([130, 130, 130]);
        assert!(val.is_finite());
        assert!(val < LOG_PROB_CLAMP, "got {val} >= clamp {LOG_PROB_CLAMP}");
    }

    #[test]
    fn neg_log_prob_clamps_for_far_outlier() {
        let pixels: Vec<u8> = (0..500).flat_map(|_| [10u8, 10, 10].into_iter()).collect();
        let mut g = Gmm5::default();
        g.init_kmeans_pp(&pixels, 1);
        // A pixel at the opposite extreme should clamp.
        let val = g.neg_log_prob([250, 250, 250]);
        assert!(val <= LOG_PROB_CLAMP);
        assert!(val >= 0.0);
    }

    #[test]
    fn assign_component_returns_valid_index() {
        let pixels: Vec<u8> = (0..200)
            .flat_map(|i| [i as u8, 128, 128].into_iter())
            .collect();
        let mut g = Gmm5::default();
        g.init_kmeans_pp(&pixels, 42);
        let k = g.assign_component([100, 128, 128]);
        assert!(k < COMPONENTS);
    }

    #[test]
    fn determinism_same_seed_same_model() {
        let pixels: Vec<u8> = (0..400)
            .flat_map(|i| {
                let v = (i * 3) as u8;
                [v, v.wrapping_add(50), 200u8].into_iter()
            })
            .collect();
        let mut g1 = Gmm5::default();
        g1.init_kmeans_pp(&pixels, 12345);
        let mut g2 = Gmm5::default();
        g2.init_kmeans_pp(&pixels, 12345);
        assert_eq!(g1.weights, g2.weights);
        assert_eq!(g1.counts, g2.counts);
        // Means may differ by a few ULPs across runs IF the
        // accumulators visit them in different orders. Our impl is
        // serial-deterministic, so bitwise equal.
        for k in 0..COMPONENTS {
            for c in 0..3 {
                assert_eq!(g1.means[k][c].to_bits(), g2.means[k][c].to_bits());
            }
        }
    }

    #[test]
    fn fit_from_explicit_assignments_matches_expected_means() {
        // 4 pixels: 2 dark (component 0), 2 bright (component 1).
        let pixels = vec![10u8, 10, 10, 20, 20, 20, 200, 200, 200, 210, 210, 210];
        let assignments = vec![0u8, 0, 1, 1];
        let mut g = Gmm5::default();
        g.fit(&pixels, &assignments);
        // Component 0 mean = (15, 15, 15).
        assert!((g.means[0][0] - 15.0).abs() < 1e-3);
        // Component 1 mean = (205, 205, 205).
        assert!((g.means[1][0] - 205.0).abs() < 1e-3);
        assert_eq!(g.counts[0], 2);
        assert_eq!(g.counts[1], 2);
    }

    #[test]
    fn singular_diag_bump_fires_on_degenerate_cov() {
        // Two pixels assigned to component 0 with identical colour
        // → covariance is exactly the zero matrix → det = 0 → bump
        // should fire and produce a positive-definite cov.
        let pixels = vec![100u8, 100, 100, 100, 100, 100];
        let assignments = vec![0u8, 0];
        let mut g = Gmm5::default();
        g.fit(&pixels, &assignments);
        // det of bumped cov should be >= SINGULAR_DIAG_BUMP^3.
        let d = det_3x3_f32(&g.covs[0]);
        assert!(d >= SINGULAR_DIAG_BUMP.powi(3) * 0.999);
        // neg_log_prob for the same colour should be finite (the
        // numerical floor blocks `exp` of huge negative).
        let v = g.neg_log_prob([100, 100, 100]);
        assert!(v.is_finite());
    }

    #[test]
    fn det_inv_round_trip_recovers_identity() {
        let m: [f32; 9] = [3.0, 0.0, 0.0, 0.0, 5.0, 0.0, 0.0, 0.0, 7.0];
        let d = det_3x3_f32(&m);
        let inv = inv_3x3(&m, d);
        // m · inv should be the identity.
        for i in 0..3 {
            for j in 0..3 {
                let mut sum = 0.0f32;
                for k in 0..3 {
                    sum += m[i * 3 + k] * inv[k * 3 + j];
                }
                let expected = if i == j { 1.0 } else { 0.0 };
                assert!((sum - expected).abs() < 1e-5, "({i},{j}) = {sum}");
            }
        }
    }

    #[test]
    fn mahalanobis_zero_at_mean() {
        let inv_cov = identity_3x3_scaled(1.0);
        let m = mahalanobis_sq([10.0, 20.0, 30.0], [10.0, 20.0, 30.0], &inv_cov);
        assert_eq!(m, 0.0);
    }

    #[test]
    fn mahalanobis_unit_step_with_identity_inv_cov() {
        let inv_cov = identity_3x3_scaled(1.0);
        let m = mahalanobis_sq([11.0, 20.0, 30.0], [10.0, 20.0, 30.0], &inv_cov);
        assert_eq!(m, 1.0);
    }
}
