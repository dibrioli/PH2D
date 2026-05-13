//! Orchestrator — pulls border colors, runs the selected algorithm,
//! refines, composites. All scratch buffers live in `Workspace` so
//! the caller can amortize allocations across calls (HR-3).

use super::border_detect::{BorderDetectOpts, detect_border_colors};
use super::colorkey::colorkey_mask;
use super::edge_aware::edge_aware_mask;
use super::luminance::luminance_mask;
use super::params::{BgRemovalAlgorithm, BgRemovalParams, RgbColor};
use super::refinement::{apply_expansion, apply_feather, apply_opening_closing, apply_smoothing};

/// Pre-allocated buffer pool reused across `apply()` calls. Sized
/// lazily on first use; capacity grows only.
#[derive(Default)]
pub struct Workspace {
    pub mask: Vec<f32>,
    pub edges: Vec<f32>,
    pub dist: Vec<f32>,
    pub scratch_a: Vec<f32>,
    pub scratch_b: Vec<f32>,
    pub scratch_c: Vec<f32>,
    pub scratch_d: Vec<f32>,
    pub scratch_e: Vec<f32>,
    pub scratch_f: Vec<f32>,
    pub queue: Vec<usize>,
    pub col_f: Vec<f32>,
    pub col_prev: Vec<f32>,
    pub col_v: Vec<usize>,
    pub col_z: Vec<f32>,
    /// Last `apply()` call's mask, mirrored — useful for the UI to
    /// inspect coverage without redoing the whole pipeline.
    pub last_mask: Vec<f32>,
}

impl Workspace {
    pub fn new() -> Self {
        Self::default()
    }

    fn reserve(&mut self, total: usize) {
        for v in [
            &mut self.mask,
            &mut self.edges,
            &mut self.dist,
            &mut self.scratch_a,
            &mut self.scratch_b,
            &mut self.scratch_c,
            &mut self.scratch_d,
            &mut self.scratch_e,
            &mut self.scratch_f,
        ] {
            if v.len() != total {
                v.clear();
                v.resize(total, 0.0);
            }
        }
        self.queue.clear();
        self.queue.reserve(total / 4);
    }
}

/// Run background removal on `rgba` and return a fresh RGBA buffer
/// where the alpha channel reflects the removed mask.
///
/// `protection`, if present, must be `w*h` f32 in 0..=1 — protected
/// pixels stay opaque (combined via `max`).
pub fn apply(
    rgba: &[u8],
    w: u32,
    h: u32,
    params: &BgRemovalParams,
    protection: Option<&[f32]>,
    workspace: &mut Workspace,
) -> Vec<u8> {
    let total = (w as usize) * (h as usize);
    if rgba.len() < total * 4 {
        return rgba.to_vec();
    }
    workspace.reserve(total);

    // Resolve target colors — user samples take priority, else
    // k-means auto-detect from the border band.
    if params.sampled_colors.is_empty() {
        let auto = detect_border_colors(rgba, w, h, BorderDetectOpts::default());
        apply_with_targets(rgba, w, h, params, &auto, protection, workspace)
    } else {
        apply_with_targets(
            rgba,
            w,
            h,
            params,
            &params.sampled_colors,
            protection,
            workspace,
        )
    }
}

fn apply_with_targets(
    rgba: &[u8],
    w: u32,
    h: u32,
    params: &BgRemovalParams,
    targets: &[RgbColor],
    protection: Option<&[f32]>,
    workspace: &mut Workspace,
) -> Vec<u8> {
    let total = (w as usize) * (h as usize);

    // 1. Run the chosen algorithm into `workspace.mask`.
    match params.algorithm {
        BgRemovalAlgorithm::Luminance => {
            luminance_mask(rgba, w, h, params.tolerance, &mut workspace.mask);
        }
        BgRemovalAlgorithm::EdgeAware => {
            edge_aware_mask(
                rgba,
                w,
                h,
                params.tolerance,
                params.edge_threshold,
                targets,
                &mut workspace.mask,
                &mut workspace.edges,
                &mut workspace.queue,
            );
        }
        BgRemovalAlgorithm::Auto | BgRemovalAlgorithm::ColorKey => {
            colorkey_mask(rgba, w, h, params.tolerance, targets, &mut workspace.mask);
        }
    }

    // 2. Refinements.
    if params.auto_clean {
        apply_opening_closing(&mut workspace.mask, w, h, &mut workspace.scratch_a);
    }
    if params.mask_expand != 0.0 {
        apply_expansion(
            &mut workspace.mask,
            w,
            h,
            params.mask_expand,
            &mut workspace.scratch_a,
        );
    }
    if params.smooth_amount > 0.0 {
        apply_smoothing(
            &mut workspace.mask,
            rgba,
            w,
            h,
            params.smooth_amount,
            &mut workspace.scratch_a,
            &mut workspace.scratch_b,
            &mut workspace.scratch_c,
            &mut workspace.scratch_d,
            &mut workspace.scratch_e,
            &mut workspace.scratch_f,
        );
    }
    if params.feather_width > 0.0 && params.feather_strength > 0.0 {
        apply_feather(
            &mut workspace.mask,
            w,
            h,
            params.feather_width,
            params.feather_strength,
            &mut workspace.dist,
            &mut workspace.col_f,
            &mut workspace.col_prev,
            &mut workspace.col_v,
            &mut workspace.col_z,
        );
    }

    // 3. Invert.
    if params.invert_mask {
        for v in workspace.mask.iter_mut() {
            *v = 1.0 - *v;
        }
    }

    // 4. Protection mask — combine via max so protected areas remain
    // visible regardless of algorithm decisions.
    if let Some(prot) = protection
        && prot.len() == total
    {
        for (i, p) in prot.iter().enumerate().take(total) {
            if *p > workspace.mask[i] {
                workspace.mask[i] = *p;
            }
        }
    }

    // Mirror result so the caller can inspect later.
    workspace.last_mask.clear();
    workspace.last_mask.extend_from_slice(&workspace.mask);

    // 5. Composite RGB + mask alpha.
    let mut out = vec![0u8; total * 4];
    for i in 0..total {
        let idx = i * 4;
        out[idx] = rgba[idx];
        out[idx + 1] = rgba[idx + 1];
        out[idx + 2] = rgba[idx + 2];
        out[idx + 3] = (workspace.mask[i].clamp(0.0, 1.0) * 255.0).round() as u8;
    }
    out
}

#[cfg(test)]
mod tests {
    use super::super::params::BgRemovalAlgorithm;
    use super::*;

    /// Build a 16×16 RGBA image with white border and red 8×8 center.
    fn red_on_white_16() -> (Vec<u8>, u32, u32) {
        let (w, h) = (16u32, 16u32);
        let mut buf = vec![255u8; (w * h * 4) as usize];
        for i in 0..(w * h) as usize {
            buf[i * 4 + 3] = 255;
        }
        for y in 4..12 {
            for x in 4..12 {
                let idx = (y * w as usize + x) * 4;
                buf[idx] = 220;
                buf[idx + 1] = 30;
                buf[idx + 2] = 30;
                buf[idx + 3] = 255;
            }
        }
        (buf, w, h)
    }

    #[test]
    fn auto_default_removes_white_border() {
        let (img, w, h) = red_on_white_16();
        let mut ws = Workspace::new();
        let params = BgRemovalParams::default();
        let out = apply(&img, w, h, &params, None, &mut ws);
        // Corner alpha low (background gone).
        assert!(out[3] < 30);
        // Center alpha high (subject preserved).
        let center_idx = (8 * w as usize + 8) * 4;
        assert!(out[center_idx + 3] > 200);
    }

    #[test]
    fn colorkey_explicit_target_removes_that_color() {
        let (img, w, h) = red_on_white_16();
        let mut ws = Workspace::new();
        let mut params = BgRemovalParams {
            algorithm: BgRemovalAlgorithm::ColorKey,
            sampled_colors: vec![RgbColor::new(255, 255, 255)],
            ..Default::default()
        };
        params.clamp();
        let out = apply(&img, w, h, &params, None, &mut ws);
        assert!(out[3] < 30);
        let center_idx = (8 * w as usize + 8) * 4;
        assert!(out[center_idx + 3] > 200);
    }

    #[test]
    fn invert_swaps_foreground_and_background() {
        let (img, w, h) = red_on_white_16();
        let mut ws = Workspace::new();
        let params = BgRemovalParams {
            invert_mask: true,
            ..Default::default()
        };
        let out = apply(&img, w, h, &params, None, &mut ws);
        // After invert: corner stays opaque (was background → now fg).
        assert!(out[3] > 200);
        let center_idx = (8 * w as usize + 8) * 4;
        assert!(out[center_idx + 3] < 30);
    }

    #[test]
    fn protection_mask_keeps_pixels_opaque() {
        let (img, w, h) = red_on_white_16();
        let total = (w * h) as usize;
        let mut prot = vec![0.0; total];
        // Protect the corner pixel.
        prot[0] = 1.0;
        let mut ws = Workspace::new();
        let params = BgRemovalParams::default();
        let out = apply(&img, w, h, &params, Some(&prot), &mut ws);
        // Corner kept opaque despite being background.
        assert!(out[3] > 200);
    }

    #[test]
    fn luminance_algorithm_removes_bright_background() {
        let (img, w, h) = red_on_white_16();
        let mut ws = Workspace::new();
        let params = BgRemovalParams {
            algorithm: BgRemovalAlgorithm::Luminance,
            tolerance: 30.0,
            ..Default::default()
        };
        let out = apply(&img, w, h, &params, None, &mut ws);
        assert!(out[3] < 30);
        let center_idx = (8 * w as usize + 8) * 4;
        assert!(out[center_idx + 3] > 200);
    }

    #[test]
    fn edge_aware_preserves_enclosed_island() {
        // Build a 16×16 image: white outside, black ring, white inside.
        let (w, h) = (16u32, 16u32);
        let mut img = vec![255u8; (w * h * 4) as usize];
        for i in 0..(w * h) as usize {
            img[i * 4 + 3] = 255;
        }
        for y in 4..=11usize {
            for x in 4..=11usize {
                if y == 4 || y == 11 || x == 4 || x == 11 {
                    let idx = (y * w as usize + x) * 4;
                    img[idx] = 0;
                    img[idx + 1] = 0;
                    img[idx + 2] = 0;
                }
            }
        }
        let mut ws = Workspace::new();
        let params = BgRemovalParams {
            algorithm: BgRemovalAlgorithm::EdgeAware,
            ..Default::default()
        };
        let out = apply(&img, w, h, &params, None, &mut ws);
        // Outer background gone.
        assert!(out[3] < 30);
        // Inside-the-ring white pixel preserved.
        let inside_idx = (7 * w as usize + 7) * 4;
        assert!(out[inside_idx + 3] > 200);
    }

    #[test]
    fn workspace_reuse_does_not_grow_unboundedly() {
        let (img, w, h) = red_on_white_16();
        let mut ws = Workspace::new();
        let params = BgRemovalParams::default();
        for _ in 0..5 {
            let _ = apply(&img, w, h, &params, None, &mut ws);
        }
        // Mask buffer matches w*h exactly after each pass.
        assert_eq!(ws.mask.len(), (w * h) as usize);
    }

    #[test]
    fn auto_clean_removes_isolated_noise() {
        // 16×16 mostly white with a single red speck pixel.
        let (w, h) = (16u32, 16u32);
        let mut img = vec![255u8; (w * h * 4) as usize];
        for i in 0..(w * h) as usize {
            img[i * 4 + 3] = 255;
        }
        // Single red pixel — will be foreground without auto_clean,
        // gone after opening.
        let idx = (8 * w as usize + 8) * 4;
        img[idx] = 220;
        img[idx + 1] = 30;
        img[idx + 2] = 30;

        let mut ws = Workspace::new();
        let params = BgRemovalParams {
            auto_clean: true,
            ..Default::default()
        };
        let out = apply(&img, w, h, &params, None, &mut ws);
        // Speck should be erased by the opening pass.
        assert!(
            out[idx + 3] < 100,
            "auto_clean should remove single-pixel speck (got alpha {})",
            out[idx + 3]
        );
    }
}
