//! Gaussian Mixture Model (5 components × 3-channel RGB) used as
//! the per-side colour model in GrabCut. Mirrors OpenCV's `cv::GMM`
//! shape: 5 components, full 3×3 covariance per component, weight
//! prior, k-means++-style deterministic init from a sub-sampled
//! pixel set.
//!
//! Public API:
//! - [`Gmm5::default()`] — zero-initialised model.
//! - [`Gmm5::init_kmeans_pp(pixels, seed)`] — seed centroids from
//!   `pixels`. Subsample to `INIT_BUDGET` for speed.
//! - [`Gmm5::fit(pixels)`] — one E/M iteration (assign + re-estimate).
//! - [`Gmm5::neg_log_prob(rgb)`] — `-log p(rgb | model)`, the
//!   weight used for unary edges in the graph cut.
//!
//! This file is a STUB — bodies land in a follow-up pass. The
//! signatures are stable so `grabcut/graph.rs` and
//! `grabcut/mod.rs` can be sketched against them.

/// Maximum samples k-means++ init reads from. Above this we
/// uniformly subsample for speed (`OpenCV` does the same).
pub const INIT_BUDGET: usize = 50_000;
/// Components per side (Rother et al.).
pub const COMPONENTS: usize = 5;
/// Diagonal jitter added when the per-component covariance det
/// drops below `SINGULAR_DET` (OpenCV PR #27120).
pub const SINGULAR_DIAG_BUMP: f32 = 1.0;
/// Determinant threshold below which the covariance is treated as
/// singular and the diagonal bump fires.
pub const SINGULAR_DET: f32 = 1.0;
/// Log-probability clamp for numerical safety: any per-component
/// likelihood below `exp(-LOG_PROB_CLAMP)` is treated as
/// `LOG_PROB_CLAMP` in the negative-log domain.
pub const LOG_PROB_CLAMP: f32 = 30.0;

/// 5-component, 3-channel Gaussian Mixture Model with full
/// covariance.
#[derive(Clone, Debug, Default)]
pub struct Gmm5 {
    /// `[mean_r, mean_g, mean_b]` per component, packed by
    /// component then channel.
    pub means: [[f32; 3]; COMPONENTS],
    /// Full 3×3 covariance per component, row-major.
    pub covs: [[f32; 9]; COMPONENTS],
    /// Cached inverse covariance, row-major.
    pub inv_covs: [[f32; 9]; COMPONENTS],
    /// Cached `1.0 / sqrt(det)` per component (the Gaussian
    /// normalisation factor).
    pub norms: [f32; COMPONENTS],
    /// Mixing weight (prior) per component, summing to 1.
    pub weights: [f32; COMPONENTS],
    /// Per-component sample count from the most recent fit. Used
    /// to detect degenerate components.
    pub counts: [u32; COMPONENTS],
}

impl Gmm5 {
    /// Seed the model from `pixels` (RGB, length `n*3`) using a
    /// deterministic k-means++-style init. Sub-samples to
    /// `INIT_BUDGET` if `pixels` is larger.
    pub fn init_kmeans_pp(&mut self, _pixels: &[u8], _seed: u64) {
        // STUB — implementation lands with the M2 GMM pass.
    }

    /// One full E/M step on `pixels` (RGB, length `n*3`): assign
    /// each pixel to its most likely component, recompute means,
    /// covariances, and weights.
    pub fn fit(&mut self, _pixels: &[u8]) {
        // STUB — implementation lands with the M2 GMM pass.
    }

    /// Return `-log p(rgb | model)`, clamped at `LOG_PROB_CLAMP`.
    /// Used as the t-link weight to source / sink in the graph.
    pub fn neg_log_prob(&self, _rgb: [u8; 3]) -> f32 {
        // STUB — returns a placeholder constant so the orchestrator
        // path is exercisable.
        LOG_PROB_CLAMP
    }
}

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
    fn neg_log_prob_stub_returns_clamp() {
        let g = Gmm5::default();
        assert_eq!(g.neg_log_prob([128, 128, 128]), LOG_PROB_CLAMP);
    }
}
