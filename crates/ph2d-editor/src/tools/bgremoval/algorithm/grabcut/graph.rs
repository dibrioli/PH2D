//! Graph construction for GrabCut: β auto-derivation, n-link
//! precomputation (8-connected), and t-link update from the
//! current GMM state.
//!
//! Stays in `graph.rs` (not `maxflow.rs`) because the **graph
//! structure** is independent of which max-flow algorithm consumes
//! it — only n-link / t-link arrays are produced here; the
//! [`super::maxflow`] module takes them as input.
//!
//! This file is a STUB — bodies land in the M2 implementation pass.

use super::gmm::Gmm5;

/// Canonical GrabCut constants (OpenCV `cv::grabCut`):
/// - `GAMMA = 50` controls the smoothness term magnitude;
/// - `LAMBDA = 9 * GAMMA` is the t-link cap for hard-constrained
///   pixels (definite FG / BG).
pub const GAMMA: f32 = 50.0;
pub const LAMBDA: f32 = 9.0 * GAMMA;
/// Floor for the auto-derived β when the image is monochrome
/// (avg neighbour distance = 0 → β = ∞ otherwise).
pub const BETA_FLOOR: f32 = 1.0;

/// Per-pixel hard label used by the trimap.
#[repr(u8)]
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum TriLabel {
    /// Definite background — locked, contributes to BG GMM only.
    BgHard = 0,
    /// Probable background — soft, updated each iteration.
    BgSoft = 1,
    /// Definite foreground — locked, contributes to FG GMM only.
    FgHard = 2,
    /// Probable foreground — soft, updated each iteration.
    FgSoft = 3,
}

/// Pre-computed n-link weights for an 8-connected image graph.
/// Indexed by `(y * w + x) * 4 + dir` where `dir ∈ 0..4` is the
/// outgoing direction (`Right`, `DownRight`, `Down`, `DownLeft`).
/// The other 4 directions are recovered by symmetry — each edge
/// is bidirectional with the same weight.
#[derive(Clone, Debug, Default)]
pub struct NLinks {
    /// Length `w * h * 4`, `f32` weights.
    pub w: u32,
    pub h: u32,
    pub edges: Vec<f32>,
}

/// Auto-derive β = `1 / (2 · E[‖c_i − c_j‖²])` over the 8-edge
/// neighbour set. Clamps to `BETA_FLOOR` for monochrome inputs.
///
/// `rgb_only` is the input RGB packed `[R,G,B,R,G,B,…]` of length
/// `w * h * 3` (caller dropped alpha).
///
/// STUB — body lands with the M2 graph pass.
pub fn derive_beta(_rgb_only: &[u8], _w: u32, _h: u32) -> f32 {
    BETA_FLOOR
}

/// Pre-compute n-link weights from the input. Writes into
/// `out.edges` (resized to `w * h * 4`).
///
/// STUB.
pub fn build_n_links(_rgb_only: &[u8], _w: u32, _h: u32, _beta: f32, _out: &mut NLinks) {
    // STUB — body lands with the M2 graph pass.
}

/// Update t-link weights (per-pixel source/sink capacities) given
/// the current GMM pair and trimap. Writes `source_caps[i]` and
/// `sink_caps[i]` (each length `w * h`).
///
/// Hard-labelled pixels (`BgHard` / `FgHard`) get caps of `LAMBDA`
/// to either source or sink. Soft pixels get `-log P(c | gmm)`.
///
/// STUB.
pub fn build_t_links(
    _rgb_only: &[u8],
    _trimap: &[TriLabel],
    _gmm_bg: &Gmm5,
    _gmm_fg: &Gmm5,
    _source_caps: &mut [f32],
    _sink_caps: &mut [f32],
) {
    // STUB — body lands with the M2 graph pass.
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn derive_beta_returns_at_least_floor() {
        let dummy = vec![0u8; 16 * 16 * 3];
        let b = derive_beta(&dummy, 16, 16);
        assert!(b >= BETA_FLOOR);
    }

    #[test]
    fn tri_label_discriminant_layout_is_compact() {
        assert_eq!(TriLabel::BgHard as u8, 0);
        assert_eq!(TriLabel::BgSoft as u8, 1);
        assert_eq!(TriLabel::FgHard as u8, 2);
        assert_eq!(TriLabel::FgSoft as u8, 3);
    }

    #[test]
    fn gamma_lambda_match_opencv_canon() {
        assert_eq!(GAMMA, 50.0);
        assert_eq!(LAMBDA, 450.0);
    }
}
