//! Graph construction for GrabCut: β auto-derivation, n-link
//! precomputation (8-connected), trimap → t-link translation, and
//! per-iteration t-link rebuild from the current GMM state.
//!
//! Stays in `graph.rs` (not `maxflow.rs`) because the **graph
//! structure** is independent of which max-flow algorithm consumes
//! it — only n-link / t-link arrays are produced here; the
//! [`super::maxflow`] module takes them as input.
//!
//! ## β derivation
//!
//! Follows OpenCV's `cv::grabCut`: β = `1 / (2 · E[‖c_i − c_j‖²])`
//! averaged over every 8-connected neighbour pair in the image.
//! Clamped at `BETA_FLOOR` to avoid `∞` on monochrome inputs.
//!
//! ## n-link weights
//!
//! `γ · exp(−β · ‖c_i − c_j‖²) / dist(i, j)` where `dist = 1` for
//! orthogonal pairs and `√2` for diagonal pairs. Stored once,
//! reused across iterations (only t-links change between iters).
//!
//! ## t-link weights
//!
//! `BgHard` / `FgHard` pixels get `LAMBDA` to the appropriate
//! terminal. Soft pixels (`BgSoft` / `FgSoft`) get the GMM
//! negative-log-likelihood: `gmm_bg.neg_log_prob(c)` to source,
//! `gmm_fg.neg_log_prob(c)` to sink. The signed `tr_cap` encoding
//! used by [`super::maxflow::BkGraph`] is computed from these as
//! `source_cap − sink_cap` per pixel.

use super::gmm::Gmm5;

// ---------------------------------------------------------------
// Constants
// ---------------------------------------------------------------

/// Smoothness term scale. Matches OpenCV's hard-coded `γ = 50`.
pub const GAMMA: f32 = 50.0;
/// t-link cap for hard-constrained pixels (definite FG / BG). `9 ·
/// γ` is the OpenCV canonical — large enough that no soft
/// likelihood can override the constraint.
pub const LAMBDA: f32 = 9.0 * GAMMA;
/// β floor for monochrome inputs (where E[‖Δc‖²] → 0 ⇒ β → ∞).
/// `1.0` produces well-behaved exponentials in the n-link weight
/// formula even with zero distance.
pub const BETA_FLOOR: f32 = 1.0;
/// 1 / √2 — the diagonal-edge distance factor.
const INV_SQRT_2: f32 = std::f32::consts::FRAC_1_SQRT_2;

// ---------------------------------------------------------------
// Trimap label
// ---------------------------------------------------------------

/// Per-pixel hard label used by the trimap.
#[repr(u8)]
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum TriLabel {
    /// Definite background — locked, contributes only to the BG
    /// GMM and gets a `LAMBDA` t-link to the sink.
    BgHard = 0,
    /// Probable background — soft, updated each iteration.
    BgSoft = 1,
    /// Definite foreground — locked.
    FgHard = 2,
    /// Probable foreground — soft.
    FgSoft = 3,
}

impl TriLabel {
    /// `true` for `FgHard` and `FgSoft` — i.e. the pixel currently
    /// belongs to the foreground side for GMM training purposes.
    pub fn is_fg(self) -> bool {
        matches!(self, TriLabel::FgHard | TriLabel::FgSoft)
    }
}

// ---------------------------------------------------------------
// n-link container
// ---------------------------------------------------------------

/// Pre-computed n-link weights for an 8-connected image graph.
///
/// Indexed by `(y * w + x) * 4 + dir` where `dir ∈ 0..4` is the
/// outgoing direction `[Right, DownRight, Down, DownLeft]`. The
/// other 4 directions are recovered by symmetry — each edge is
/// bidirectional with the same weight, and the symmetric edge is
/// stored at the destination pixel's array slot. This is the
/// representation [`super::maxflow::BkGraph::load_capacities`]
/// expects (no double-counting on its side).
#[derive(Clone, Debug, Default)]
pub struct NLinks {
    pub w: u32,
    pub h: u32,
    /// Length `w * h * 4`. Edges that fall off the image border
    /// are stored as `0`.
    pub edges: Vec<f32>,
}

impl NLinks {
    pub fn ensure(&mut self, w: u32, h: u32) {
        let n4 = (w as usize) * (h as usize) * 4;
        self.w = w;
        self.h = h;
        self.edges.resize(n4, 0.0);
    }
}

// ---------------------------------------------------------------
// β derivation
// ---------------------------------------------------------------

/// Compute β = `1 / (2 · E[‖c_i − c_j‖²])` over every 8-connected
/// neighbour pair in the image. Clamped at `BETA_FLOOR`.
///
/// `rgb_only` is `[R,G,B,R,G,B,…]` of length `w * h * 3` (alpha
/// dropped — see [`super::mod::flatten_rgba`]).
pub fn derive_beta(rgb_only: &[u8], w: u32, h: u32) -> f32 {
    debug_assert_eq!(rgb_only.len(), (w as usize) * (h as usize) * 3);
    if w == 0 || h == 0 {
        return BETA_FLOOR;
    }
    let mut total_sq: f64 = 0.0;
    let mut count: u64 = 0;
    let stride = w as usize;
    let read = |i: usize| -> [f32; 3] {
        [
            rgb_only[i * 3] as f32,
            rgb_only[i * 3 + 1] as f32,
            rgb_only[i * 3 + 2] as f32,
        ]
    };
    let dsq = |a: [f32; 3], b: [f32; 3]| -> f32 {
        let dr = a[0] - b[0];
        let dg = a[1] - b[1];
        let db = a[2] - b[2];
        dr * dr + dg * dg + db * db
    };
    for y in 0..h as usize {
        for x in 0..w as usize {
            let i = y * stride + x;
            let pi = read(i);
            // Right (x+1, y).
            if x + 1 < w as usize {
                let j = i + 1;
                total_sq += dsq(pi, read(j)) as f64;
                count += 1;
            }
            // Down-right (x+1, y+1).
            if x + 1 < w as usize && y + 1 < h as usize {
                let j = i + stride + 1;
                total_sq += dsq(pi, read(j)) as f64;
                count += 1;
            }
            // Down (x, y+1).
            if y + 1 < h as usize {
                let j = i + stride;
                total_sq += dsq(pi, read(j)) as f64;
                count += 1;
            }
            // Down-left (x-1, y+1).
            if x > 0 && y + 1 < h as usize {
                let j = i + stride - 1;
                total_sq += dsq(pi, read(j)) as f64;
                count += 1;
            }
        }
    }
    if count == 0 {
        return BETA_FLOOR;
    }
    let avg = total_sq / count as f64;
    // Bumped from 1e-9 to 1.0 (one squared-channel JND): images
    // with a *single* ±1 stray pixel on an otherwise monochrome
    // plate would otherwise produce a huge β (~5e5 for 1024²),
    // and `exp(-β·d²)` underflows to 0 across the whole image —
    // collapsing n-link weights to 0 and degrading GrabCut to
    // per-pixel GMM classification.
    if avg < 1.0 {
        return BETA_FLOOR;
    }
    (1.0 / (2.0 * avg)) as f32
}

// ---------------------------------------------------------------
// n-link build
// ---------------------------------------------------------------

/// Pre-compute n-link weights from the input. Writes into
/// `out.edges` (resized to `w * h * 4`).
///
/// `weight(i, j) = γ · exp(−β · ‖c_i − c_j‖²) / dist(i, j)`.
pub fn build_n_links(rgb_only: &[u8], w: u32, h: u32, beta: f32, out: &mut NLinks) {
    debug_assert_eq!(rgb_only.len(), (w as usize) * (h as usize) * 3);
    out.ensure(w, h);
    let stride = w as usize;
    let dsq = |i: usize, j: usize| -> f32 {
        let dr = rgb_only[i * 3] as f32 - rgb_only[j * 3] as f32;
        let dg = rgb_only[i * 3 + 1] as f32 - rgb_only[j * 3 + 1] as f32;
        let db = rgb_only[i * 3 + 2] as f32 - rgb_only[j * 3 + 2] as f32;
        dr * dr + dg * dg + db * db
    };
    let weight = |d2: f32, inv_dist: f32| -> f32 { GAMMA * inv_dist * (-beta * d2).exp() };
    for y in 0..h as usize {
        for x in 0..w as usize {
            let i = y * stride + x;
            let base = i * 4;
            // dir 0: right (orthogonal).
            if x + 1 < w as usize {
                out.edges[base] = weight(dsq(i, i + 1), 1.0);
            }
            // dir 1: down-right (diagonal).
            if x + 1 < w as usize && y + 1 < h as usize {
                out.edges[base + 1] = weight(dsq(i, i + stride + 1), INV_SQRT_2);
            }
            // dir 2: down (orthogonal).
            if y + 1 < h as usize {
                out.edges[base + 2] = weight(dsq(i, i + stride), 1.0);
            }
            // dir 3: down-left (diagonal).
            if x > 0 && y + 1 < h as usize {
                out.edges[base + 3] = weight(dsq(i, i + stride - 1), INV_SQRT_2);
            }
        }
    }
}

// ---------------------------------------------------------------
// t-link build
// ---------------------------------------------------------------

/// Update per-pixel source / sink capacities (`tr_cap` inputs of
/// [`super::maxflow::BkGraph::load_capacities`]) given the
/// current GMM pair and trimap. Writes both `source_caps[i]` and
/// `sink_caps[i]` (each length `w * h`).
///
/// - `BgHard` ⇒ source 0, sink LAMBDA.
/// - `FgHard` ⇒ source LAMBDA, sink 0.
/// - `BgSoft` / `FgSoft` ⇒ source `gmm_bg.neg_log_prob`, sink
///   `gmm_fg.neg_log_prob` — pixels likely to be background get
///   high source cap (it's *expensive* to keep them in S-tree),
///   and vice versa. (See Rother et al. §4: t-link signs are the
///   convention adopted by OpenCV's `cv::grabCut`.)
pub fn build_t_links(
    rgb_only: &[u8],
    trimap: &[TriLabel],
    gmm_bg: &Gmm5,
    gmm_fg: &Gmm5,
    source_caps: &mut [f32],
    sink_caps: &mut [f32],
) {
    let n = trimap.len();
    debug_assert_eq!(rgb_only.len(), n * 3);
    debug_assert_eq!(source_caps.len(), n);
    debug_assert_eq!(sink_caps.len(), n);
    for i in 0..n {
        match trimap[i] {
            TriLabel::BgHard => {
                source_caps[i] = 0.0;
                sink_caps[i] = LAMBDA;
            }
            TriLabel::FgHard => {
                source_caps[i] = LAMBDA;
                sink_caps[i] = 0.0;
            }
            TriLabel::BgSoft | TriLabel::FgSoft => {
                let rgb = [rgb_only[i * 3], rgb_only[i * 3 + 1], rgb_only[i * 3 + 2]];
                source_caps[i] = gmm_bg.neg_log_prob(rgb);
                sink_caps[i] = gmm_fg.neg_log_prob(rgb);
            }
        }
    }
}

// ---------------------------------------------------------------
// Tests
// ---------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn derive_beta_floor_on_monochrome() {
        let dummy = vec![100u8; 16 * 16 * 3];
        let b = derive_beta(&dummy, 16, 16);
        assert!(b >= BETA_FLOOR);
        // Monochrome → avg dist = 0 → return floor exactly.
        assert_eq!(b, BETA_FLOOR);
    }

    #[test]
    fn derive_beta_finite_on_varied_image() {
        // Checkerboard pattern: pure black + pure white pixels.
        let mut rgb = vec![0u8; 8 * 8 * 3];
        for y in 0..8 {
            for x in 0..8 {
                let i = (y * 8 + x) * 3;
                let bright = if (x + y) % 2 == 0 { 255 } else { 0 };
                rgb[i] = bright;
                rgb[i + 1] = bright;
                rgb[i + 2] = bright;
            }
        }
        let b = derive_beta(&rgb, 8, 8);
        assert!(b > 0.0 && b.is_finite());
        // Average squared neighbour distance is ~3·255² = ~195k.
        // β ≈ 1 / (2·195k) ≈ 2.5e-6.
        assert!(b < 1e-4, "expected small β for checkerboard, got {b}");
    }

    #[test]
    fn n_link_orthogonal_vs_diagonal_uses_inv_sqrt_2() {
        // 2×2 uniform image → all colour distances are zero → all
        // weights = γ · inv_dist. Diagonal slot has weight γ/√2,
        // orthogonal slot has weight γ.
        let rgb = vec![100u8; 4 * 3];
        let mut nl = NLinks::default();
        build_n_links(&rgb, 2, 2, 1.0, &mut nl);
        // Pixel 0 (top-left): right (orth), down-right (diag), down (orth), down-left (oob).
        assert_eq!(nl.edges[0], GAMMA, "right (orth)");
        assert!(
            (nl.edges[1] - GAMMA * INV_SQRT_2).abs() < 1e-4,
            "down-right (diag)"
        );
        assert_eq!(nl.edges[2], GAMMA, "down (orth)");
        assert_eq!(nl.edges[3], 0.0, "down-left (out of bounds)");
    }

    #[test]
    fn n_link_distance_dependent_exp_falloff() {
        // Two pixels: left black, right white. β = 1. Forward
        // weight: γ · exp(−1 · 3·65025) ≈ 0 because the exponent
        // is huge negative.
        let rgb = vec![0u8, 0, 0, 255, 255, 255];
        let mut nl = NLinks::default();
        build_n_links(&rgb, 2, 1, 1.0, &mut nl);
        // edges[0] = right (orth): weight ≈ 0.
        assert!(
            nl.edges[0] < 1e-6,
            "edge weight should fall off, got {}",
            nl.edges[0]
        );
    }

    #[test]
    fn t_link_hard_labels_use_lambda() {
        let rgb = vec![100u8; 6]; // 2 pixels
        let trimap = vec![TriLabel::BgHard, TriLabel::FgHard];
        let gmm_bg = Gmm5::default();
        let gmm_fg = Gmm5::default();
        let mut src = vec![0.0; 2];
        let mut snk = vec![0.0; 2];
        build_t_links(&rgb, &trimap, &gmm_bg, &gmm_fg, &mut src, &mut snk);
        assert_eq!(src[0], 0.0);
        assert_eq!(snk[0], LAMBDA);
        assert_eq!(src[1], LAMBDA);
        assert_eq!(snk[1], 0.0);
    }

    #[test]
    fn t_link_soft_labels_use_gmm_neg_log_prob() {
        // Trained BG GMM (dark pixels), trained FG GMM (bright).
        // A bright pixel (soft) should have high source-cap (it's
        // costly to keep in S-tree because the BG GMM doesn't fit
        // it).
        let bg_pixels: Vec<u8> = (0..500).flat_map(|_| [10u8, 10, 10]).collect();
        let fg_pixels: Vec<u8> = (0..500).flat_map(|_| [240u8, 240, 240]).collect();
        let mut gmm_bg = Gmm5::default();
        gmm_bg.init_kmeans_pp(&bg_pixels, 42);
        let mut gmm_fg = Gmm5::default();
        gmm_fg.init_kmeans_pp(&fg_pixels, 42);

        let probe_rgb = vec![240u8, 240, 240];
        let trimap = vec![TriLabel::FgSoft];
        let mut src = vec![0.0; 1];
        let mut snk = vec![0.0; 1];
        build_t_links(&probe_rgb, &trimap, &gmm_bg, &gmm_fg, &mut src, &mut snk);
        assert!(
            src[0] > snk[0],
            "bright pixel should be costly under BG GMM: src {} sink {}",
            src[0],
            snk[0]
        );
    }

    #[test]
    fn tri_label_is_fg_classifies_correctly() {
        assert!(!TriLabel::BgHard.is_fg());
        assert!(!TriLabel::BgSoft.is_fg());
        assert!(TriLabel::FgHard.is_fg());
        assert!(TriLabel::FgSoft.is_fg());
    }

    #[test]
    fn gamma_lambda_match_opencv_canon() {
        assert_eq!(GAMMA, 50.0);
        assert_eq!(LAMBDA, 450.0);
    }
}
