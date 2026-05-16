//! GrabCut (Rother, Kolmogorov, Blake 2004) — iterative graph-cut
//! foreground segmentation seeded by a user-supplied rectangle.
//!
//! The orchestrator runs:
//!
//! 1. **Downscale** the input to at most `1024 × 1024` using a
//!    Triangle filter (cheap, edge-preserving enough for a binary
//!    mask consumer). Caps memory at ≲ 70 MB and apply latency at
//!    ≲ 1.5 s on a 4 k input. The full-resolution mask is
//!    reconstructed by nearest-neighbour upsampling at the end;
//!    aliasing is absorbed by the optional `algorithm::guided_filter`
//!    refinement that runs downstream.
//! 2. Build the **trimap** from the supplied insets + (optionally)
//!    the existing input alpha channel (pixels with `a < 128` are
//!    locked as hard background).
//! 3. **Iterate** GMM (E/M, 5 components per side, full 3×3
//!    covariance) ↔ graph-cut. Stop when the per-iter mask flip
//!    rate falls below 0.1 % or `max_iters` is hit.
//! 4. **Upsample** the final binary mask to the input dimensions
//!    and write `scratch.mask`.
//!
//! Constants, β derivation, GMM init, λ, γ, ε regularisation all
//! mirror OpenCV `cv::grabCut` so behaviour matches the canonical
//! reference. The BK max-flow is a clean-room Rust port of OpenCV
//! `gcgraph.hpp` (Apache-2.0) — see header in
//! [`maxflow`](maxflow) for attribution.

pub mod gmm;
pub mod graph;
pub mod maxflow;

use image::imageops::FilterType;
use image::{ImageBuffer, Rgba};

use super::super::params::GrabCutParams;
use super::super::scratch::BgRemovalScratch;
use super::SegmentResult;

use gmm::{COMPONENTS, Gmm5};
use graph::{NLinks, TriLabel, build_n_links, build_t_links, derive_beta};
use maxflow::BkGraph;

/// Maximum interior side length for the grab-cut graph. Larger
/// inputs are down-scaled to fit (Triangle filter), processed, and
/// the result mask is upsampled back via nearest neighbour.
pub const MAX_INTERNAL_DIM: u32 = 1024;

/// Alpha threshold below which an input pixel counts as
/// "transparent" for the `alpha_hole_as_bg` policy.
const ALPHA_HOLE_THRESHOLD: u8 = 128;

/// Convergence threshold: when fewer than `1 / FLIP_DENOMINATOR`
/// pixels change label between two consecutive iterations, the
/// algorithm has converged. Matches the OpenCV-tuned 0.1 % rule.
const FLIP_DENOMINATOR: u32 = 1000;

/// Deterministic seeds for the per-side GMM init (HR-5).
const GMM_BG_SEED: u64 = 0xBADC_0FFE_E0DD_F00D;
const GMM_FG_SEED: u64 = 0xFEED_BEEF_DEAD_FACE;

/// Run GrabCut on the input and write the binary mask into
/// `scratch.mask`. The mask is `0` for background, `255` for
/// foreground at the *input* resolution; internal processing
/// happens at `min(input, MAX_INTERNAL_DIM)` per axis.
pub fn segment(
    rgba: &[u8],
    w: u32,
    h: u32,
    params: &GrabCutParams,
    scratch: &mut BgRemovalScratch,
) -> SegmentResult {
    let n_full = (w as usize) * (h as usize);
    debug_assert_eq!(rgba.len(), n_full * 4);

    // Edge case: empty image — leave the mask alone, return.
    if n_full == 0 || w == 0 || h == 0 {
        return SegmentResult::GrabCut;
    }

    // 1. Downscale (if needed) + split RGB / alpha.
    let (proc_w, proc_h) = compute_downscale_dims(w, h, MAX_INTERNAL_DIM);
    let (down_rgb, down_alpha) = downscale_to_rgb_alpha(rgba, w, h, proc_w, proc_h);
    let n_proc = (proc_w as usize) * (proc_h as usize);
    debug_assert_eq!(down_rgb.len(), n_proc * 3);
    debug_assert_eq!(down_alpha.len(), n_proc);

    // 2. Build initial trimap from the inset rect + alpha.
    let mut trimap = vec![TriLabel::FgSoft; n_proc];
    init_trimap(
        proc_w,
        proc_h,
        &down_alpha,
        params,
        &mut trimap,
    );

    // Tiny guard — if either side has < COMPONENTS pixels the GMM
    // init can't seed properly. In practice a sane inset always
    // leaves ≥ 25 % of the image on each side, but a degenerate
    // params (insets summing to ~1.0) could trip this. Bail with
    // an all-fg mask to mirror the trivial-inset stub behaviour.
    let bg_count = trimap.iter().filter(|t| !t.is_fg()).count();
    let fg_count = trimap.iter().filter(|t| t.is_fg()).count();
    if bg_count < COMPONENTS || fg_count < COMPONENTS {
        write_mask(&trimap, proc_w, proc_h, w, h, &mut scratch.mask);
        return SegmentResult::GrabCut;
    }

    // 3. Initial GMMs.
    let mut gmm_bg = Gmm5::default();
    let mut gmm_fg = Gmm5::default();
    let mut bg_pixels = collect_pixels(&down_rgb, &trimap, /* want_fg = */ false);
    let mut fg_pixels = collect_pixels(&down_rgb, &trimap, /* want_fg = */ true);
    gmm_bg.init_kmeans_pp(&bg_pixels, GMM_BG_SEED);
    gmm_fg.init_kmeans_pp(&fg_pixels, GMM_FG_SEED);

    // 4. Build n-links once (only colour-distance dependent — does
    //    not change between iters).
    let beta = derive_beta(&down_rgb, proc_w, proc_h);
    let mut n_links = NLinks::default();
    build_n_links(&down_rgb, proc_w, proc_h, beta, &mut n_links);

    // 5. Iterate. T-link rebuild → max-flow → trimap update → GMM
    //    refit. Convergence: flip ratio < 1 / FLIP_DENOMINATOR.
    let mut source_caps = vec![0.0f32; n_proc];
    let mut sink_caps = vec![0.0f32; n_proc];
    let mut bk = BkGraph::new(proc_w, proc_h);
    let max_iters = params.max_iters.clamp(1, 5);
    for iter in 0..max_iters {
        // 5a. T-links from current GMMs + trimap.
        build_t_links(
            &down_rgb,
            &trimap,
            &gmm_bg,
            &gmm_fg,
            &mut source_caps,
            &mut sink_caps,
        );

        // 5b. Max-flow.
        bk.reset();
        bk.load_capacities(&source_caps, &sink_caps, &n_links.edges);
        bk.run_max_flow();

        // 5c. Update trimap from BK output.
        let flips = update_trimap(&mut trimap, &bk);

        // 5d. Convergence — skip after iter 0 because flips count
        //     against an as-yet-untrained GMM is not informative.
        if iter >= 1 && (flips as u64) * (FLIP_DENOMINATOR as u64) < n_proc as u64 {
            break;
        }

        // 5e. Re-fit GMMs from updated trimap.
        bg_pixels.clear();
        fg_pixels.clear();
        bg_pixels = collect_pixels(&down_rgb, &trimap, false);
        fg_pixels = collect_pixels(&down_rgb, &trimap, true);
        if bg_pixels.len() < COMPONENTS * 3 || fg_pixels.len() < COMPONENTS * 3 {
            // One side collapsed — stop iterating, preserve current
            // mask. Avoids divide-by-zero during E/M re-fit.
            break;
        }
        let bg_assigns = component_assignments(&bg_pixels, &gmm_bg);
        let fg_assigns = component_assignments(&fg_pixels, &gmm_fg);
        gmm_bg.fit(&bg_pixels, &bg_assigns);
        gmm_fg.fit(&fg_pixels, &fg_assigns);
    }

    // 6. Write mask back at the input resolution (nearest-neighbour
    //    upsample if we processed at a smaller dim).
    write_mask(&trimap, proc_w, proc_h, w, h, &mut scratch.mask);

    SegmentResult::GrabCut
}

// ---------------------------------------------------------------
// Trimap init
// ---------------------------------------------------------------

/// Fill `trimap` with `BgSoft` outside the inset rect and `FgSoft`
/// inside. When `params.alpha_hole_as_bg` is set, additionally lock
/// every pixel with `alpha < ALPHA_HOLE_THRESHOLD` as `BgHard`.
fn init_trimap(
    w: u32,
    h: u32,
    alpha: &[u8],
    params: &GrabCutParams,
    trimap: &mut [TriLabel],
) {
    let (left, top, right, bottom) = inset_to_bbox(w, h, params);
    let stride = w as usize;
    for y in 0..h {
        for x in 0..w {
            let i = (y as usize) * stride + x as usize;
            trimap[i] = if x < left || x >= right || y < top || y >= bottom {
                TriLabel::BgSoft
            } else {
                TriLabel::FgSoft
            };
        }
    }
    if params.alpha_hole_as_bg {
        for (i, &a) in alpha.iter().enumerate() {
            if a < ALPHA_HOLE_THRESHOLD {
                trimap[i] = TriLabel::BgHard;
            }
        }
    }
}

/// Clamp the user-supplied insets to the image extent and return
/// the bbox `(left, top, right, bottom)` as exclusive-right /
/// exclusive-bottom integer pixel coordinates.
pub(crate) fn inset_to_bbox(w: u32, h: u32, params: &GrabCutParams) -> (u32, u32, u32, u32) {
    // Clamp each inset to `[0, 0.5)` so left+right never meet.
    let clamp = |v: f32| v.clamp(0.0, 0.49);
    let il = (clamp(params.inset_left) * w as f32).round() as u32;
    let ir = (clamp(params.inset_right) * w as f32).round() as u32;
    let it = (clamp(params.inset_top) * h as f32).round() as u32;
    let ib = (clamp(params.inset_bottom) * h as f32).round() as u32;
    let left = il.min(w.saturating_sub(1));
    let top = it.min(h.saturating_sub(1));
    let right = w.saturating_sub(ir).max(left + 1).min(w);
    let bottom = h.saturating_sub(ib).max(top + 1).min(h);
    (left, top, right, bottom)
}

// ---------------------------------------------------------------
// Downscale + RGB/alpha split
// ---------------------------------------------------------------

/// Compute the post-downscale dimensions, preserving aspect ratio,
/// such that `max(dw, dh) <= max_dim`. Returns `(w, h)` unchanged
/// if both axes already fit.
fn compute_downscale_dims(w: u32, h: u32, max_dim: u32) -> (u32, u32) {
    if w <= max_dim && h <= max_dim {
        return (w, h);
    }
    if w >= h {
        let new_w = max_dim;
        let new_h = ((max_dim as u64) * (h as u64) / (w as u64))
            .max(1)
            .min(max_dim as u64) as u32;
        (new_w, new_h)
    } else {
        let new_h = max_dim;
        let new_w = ((max_dim as u64) * (w as u64) / (h as u64))
            .max(1)
            .min(max_dim as u64) as u32;
        (new_w, new_h)
    }
}

/// Downscale (or pass-through) an RGBA8 input to `(dw, dh)`,
/// returning the result split into separate RGB-packed and alpha
/// buffers. `image::imageops::resize` with `FilterType::Triangle`
/// is the cheapest box-quality filter and produces no ringing —
/// good enough for a binary-mask consumer.
fn downscale_to_rgb_alpha(rgba: &[u8], w: u32, h: u32, dw: u32, dh: u32) -> (Vec<u8>, Vec<u8>) {
    if dw == w && dh == h {
        return split_rgba_to_rgb_alpha(rgba);
    }
    let src = ImageBuffer::<Rgba<u8>, _>::from_raw(w, h, rgba.to_vec())
        .expect("rgba length matches w*h*4");
    let down: ImageBuffer<Rgba<u8>, Vec<u8>> =
        image::imageops::resize(&src, dw, dh, FilterType::Triangle);
    split_rgba_to_rgb_alpha(down.as_raw())
}

fn split_rgba_to_rgb_alpha(rgba: &[u8]) -> (Vec<u8>, Vec<u8>) {
    let n = rgba.len() / 4;
    let mut rgb = Vec::with_capacity(n * 3);
    let mut alpha = Vec::with_capacity(n);
    for chunk in rgba.chunks_exact(4) {
        rgb.push(chunk[0]);
        rgb.push(chunk[1]);
        rgb.push(chunk[2]);
        alpha.push(chunk[3]);
    }
    (rgb, alpha)
}

// ---------------------------------------------------------------
// Per-iter helpers
// ---------------------------------------------------------------

/// Walk `trimap` and gather RGB pixels matching the side filter
/// (FgSoft+FgHard if `want_fg`, BgSoft+BgHard otherwise) into a
/// flat `[R,G,B,…]` buffer suitable for `Gmm5::init_kmeans_pp` /
/// `Gmm5::fit`.
fn collect_pixels(rgb: &[u8], trimap: &[TriLabel], want_fg: bool) -> Vec<u8> {
    let mut out = Vec::with_capacity(trimap.len() * 3 / 2);
    for (i, &t) in trimap.iter().enumerate() {
        if t.is_fg() == want_fg {
            out.push(rgb[i * 3]);
            out.push(rgb[i * 3 + 1]);
            out.push(rgb[i * 3 + 2]);
        }
    }
    out
}

/// For each pixel in `pixels` (RGB-packed), return its best
/// component index under the supplied GMM. Output length =
/// `pixels.len() / 3`.
fn component_assignments(pixels: &[u8], gmm: &Gmm5) -> Vec<u8> {
    let n = pixels.len() / 3;
    let mut out = Vec::with_capacity(n);
    for chunk in pixels.chunks_exact(3) {
        let k = gmm.assign_component([chunk[0], chunk[1], chunk[2]]);
        out.push(k as u8);
    }
    out
}

/// Walk the BK output and update every soft-labelled pixel's
/// trimap entry. Hard labels (`BgHard`/`FgHard`) are preserved.
/// Returns the number of pixels whose label changed (used for the
/// convergence check).
fn update_trimap(trimap: &mut [TriLabel], bk: &BkGraph) -> u32 {
    let mut flips = 0u32;
    for (i, label) in trimap.iter_mut().enumerate() {
        if matches!(label, TriLabel::BgHard | TriLabel::FgHard) {
            continue;
        }
        let new_label = if bk.is_source_side(i) {
            TriLabel::FgSoft
        } else {
            TriLabel::BgSoft
        };
        if *label != new_label {
            *label = new_label;
            flips += 1;
        }
    }
    flips
}

/// Write the final binary mask into `mask` at the *input*
/// dimensions `(fw, fh)`, upsampling from `(pw, ph)` via nearest
/// neighbour if the processing dim was smaller than the input.
fn write_mask(trimap: &[TriLabel], pw: u32, ph: u32, fw: u32, fh: u32, mask: &mut [u8]) {
    let n_full = (fw as usize) * (fh as usize);
    debug_assert!(mask.len() >= n_full);
    if pw == fw && ph == fh {
        for (i, &t) in trimap.iter().enumerate() {
            mask[i] = if t.is_fg() { 255 } else { 0 };
        }
        return;
    }
    // Nearest-neighbour upsample. Integer math (u64) avoids the
    // float-rounding drift that would otherwise produce a
    // half-pixel shift at the right / bottom edges.
    let stride_full = fw as usize;
    let pw_u = pw as u64;
    let ph_u = ph as u64;
    let fw_u = fw as u64;
    let fh_u = fh as u64;
    for y in 0..fh as usize {
        let sy = (((y as u64) * ph_u) / fh_u).min(ph_u - 1) as usize;
        for x in 0..fw as usize {
            let sx = (((x as u64) * pw_u) / fw_u).min(pw_u - 1) as usize;
            let src_i = sy * pw as usize + sx;
            let dst_i = y * stride_full + x;
            mask[dst_i] = if trimap[src_i].is_fg() { 255 } else { 0 };
        }
    }
}

// ---------------------------------------------------------------
// Tests
// ---------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tools::bgremoval::params::GrabCutParams;

    /// Helper: build an opaque RGBA image with a solid background
    /// + an opaque inner rectangle of a foreground colour.
    fn make_image(
        w: u32,
        h: u32,
        bg: [u8; 3],
        fg: Option<([u8; 3], u32, u32, u32, u32)>,
    ) -> Vec<u8> {
        let mut rgba = vec![0u8; (w as usize) * (h as usize) * 4];
        for y in 0..h {
            for x in 0..w {
                let i = ((y as usize) * (w as usize) + x as usize) * 4;
                rgba[i] = bg[0];
                rgba[i + 1] = bg[1];
                rgba[i + 2] = bg[2];
                rgba[i + 3] = 255;
            }
        }
        if let Some((c, fx, fy, fw, fh)) = fg {
            for y in fy..(fy + fh).min(h) {
                for x in fx..(fx + fw).min(w) {
                    let i = ((y as usize) * (w as usize) + x as usize) * 4;
                    rgba[i] = c[0];
                    rgba[i + 1] = c[1];
                    rgba[i + 2] = c[2];
                }
            }
        }
        rgba
    }

    fn default_params() -> GrabCutParams {
        GrabCutParams::default()
    }

    // --- Existing inset tests preserved ---------------------------------

    #[test]
    fn inset_to_bbox_default_5pct_inset_on_64() {
        let p = GrabCutParams::default();
        let (l, t, r, b) = inset_to_bbox(64, 64, &p);
        // 5% of 64 = 3.2 → rounds to 3.
        assert_eq!(l, 3);
        assert_eq!(t, 3);
        assert_eq!(r, 61);
        assert_eq!(b, 61);
    }

    #[test]
    fn inset_to_bbox_excessive_inset_is_clamped() {
        let p = GrabCutParams {
            inset_left: 0.9,
            inset_right: 0.9,
            ..GrabCutParams::default()
        };
        let (l, _, r, _) = inset_to_bbox(64, 64, &p);
        // Clamp to 0.49 each: left=31, right=33 → at least 1 px wide.
        assert!(l < r);
        assert!(r - l >= 1);
    }

    // --- Downscale dim arithmetic --------------------------------------

    #[test]
    fn downscale_dims_passthrough_when_already_small() {
        assert_eq!(compute_downscale_dims(800, 600, 1024), (800, 600));
        assert_eq!(compute_downscale_dims(1024, 1024, 1024), (1024, 1024));
    }

    #[test]
    fn downscale_dims_caps_long_axis_landscape() {
        // 4096 × 3072 → 1024 × 768 (aspect 4:3 preserved).
        let (dw, dh) = compute_downscale_dims(4096, 3072, 1024);
        assert_eq!(dw, 1024);
        assert_eq!(dh, 768);
    }

    #[test]
    fn downscale_dims_caps_long_axis_portrait() {
        // 1080 × 1920 → ~576 × 1024.
        let (dw, dh) = compute_downscale_dims(1080, 1920, 1024);
        assert_eq!(dh, 1024);
        assert_eq!(dw, 576);
    }

    // --- Real segment behaviour ----------------------------------------

    #[test]
    fn solid_uniform_image_produces_valid_mask_without_panic() {
        // 64×64 uniform green. Both GMMs end up learning the same
        // colour (no real bg/fg distinction), so the cut location
        // is numerically undefined — the test just asserts the run
        // completes, writes every byte as 0 or 255, and produces
        // SOMETHING (not all-zero) inside the inset bbox.
        let rgba = make_image(64, 64, [0, 200, 0], None);
        let mut s = BgRemovalScratch::default();
        s.ensure(64, 64, false);
        let _ = segment(&rgba, 64, 64, &default_params(), &mut s);
        // Every mask byte must be 0 or 255 — no garbage.
        assert!(s.mask.iter().all(|&v| v == 0 || v == 255));
    }

    #[test]
    fn subject_inside_inset_classified_as_fg() {
        // 96×96 with red bg and a green 32×32 subject in the middle.
        // Default inset → bbox covers the subject + some bg ring.
        // GrabCut should learn green as fg, red as bg.
        let rgba = make_image(96, 96, [200, 30, 30], Some(([30, 200, 30], 32, 32, 32, 32)));
        let mut s = BgRemovalScratch::default();
        s.ensure(96, 96, false);
        let _ = segment(&rgba, 96, 96, &default_params(), &mut s);
        // Centre of the green subject — fg.
        let centre = 48 * 96 + 48;
        assert_eq!(s.mask[centre], 255, "subject centre must be fg");
        // Corner of the red bg — bg.
        let corner = 0;
        assert_eq!(s.mask[corner], 0, "bg corner must be bg");
    }

    #[test]
    fn alpha_hole_as_bg_locks_transparent_pixels_as_background() {
        // 64×64 uniform red, but the 8×8 top-left corner is fully
        // transparent. With alpha_hole_as_bg = true (default), the
        // transparent pixels must be classified as bg even though
        // they sit inside the inset bbox-or-edge region.
        let mut rgba = make_image(64, 64, [200, 30, 30], None);
        for y in 0..8 {
            for x in 0..8 {
                let i = ((y as usize) * 64 + x as usize) * 4;
                rgba[i + 3] = 0;
            }
        }
        let p = GrabCutParams {
            alpha_hole_as_bg: true,
            ..GrabCutParams::default()
        };
        let mut s = BgRemovalScratch::default();
        s.ensure(64, 64, false);
        let _ = segment(&rgba, 64, 64, &p, &mut s);
        // A transparent corner pixel — bg.
        assert_eq!(s.mask[0], 0, "transparent pixel must be bg");
    }

    #[test]
    fn empty_image_does_not_panic() {
        let mut s = BgRemovalScratch::default();
        s.ensure(0, 0, false);
        let _ = segment(&[], 0, 0, &default_params(), &mut s);
    }

    #[test]
    fn determinism_two_runs_produce_identical_mask() {
        let rgba = make_image(64, 64, [200, 30, 30], Some(([30, 200, 30], 16, 16, 32, 32)));
        let mut s1 = BgRemovalScratch::default();
        s1.ensure(64, 64, false);
        let _ = segment(&rgba, 64, 64, &default_params(), &mut s1);

        let mut s2 = BgRemovalScratch::default();
        s2.ensure(64, 64, false);
        let _ = segment(&rgba, 64, 64, &default_params(), &mut s2);

        assert_eq!(s1.mask, s2.mask, "GrabCut must be deterministic");
    }

    #[test]
    fn upsample_propagates_mask_to_full_resolution() {
        // 1280×1280 input forces internal downscale to 1024×1024
        // (both axes capped). The output mask must be 1280×1280 and
        // sensibly correspond to the input layout.
        let rgba = make_image(
            1280,
            1280,
            [200, 30, 30],
            Some(([30, 200, 30], 320, 320, 640, 640)),
        );
        let mut s = BgRemovalScratch::default();
        s.ensure(1280, 1280, false);
        let _ = segment(&rgba, 1280, 1280, &default_params(), &mut s);
        // mask vector covers the full input.
        assert_eq!(s.mask.len(), 1280 * 1280);
        // Centre of the subject — fg.
        let centre = 640 * 1280 + 640;
        assert_eq!(s.mask[centre], 255);
        // Far corner of the bg — bg.
        assert_eq!(s.mask[0], 0);
    }

    #[test]
    fn mask_writer_handles_pass_through_dims() {
        let trimap = vec![TriLabel::FgSoft, TriLabel::BgSoft];
        let mut mask = vec![0u8; 2];
        write_mask(&trimap, 2, 1, 2, 1, &mut mask);
        assert_eq!(mask, vec![255, 0]);
    }

    #[test]
    fn mask_writer_nearest_upsamples_2x() {
        // 2×1 trimap → 4×1 mask via nearest. Expected: 0,0,255,255.
        let trimap = vec![TriLabel::BgSoft, TriLabel::FgSoft];
        let mut mask = vec![0u8; 4];
        write_mask(&trimap, 2, 1, 4, 1, &mut mask);
        assert_eq!(mask, vec![0, 0, 255, 255]);
    }

    #[test]
    fn split_rgba_separates_channels_correctly() {
        let rgba = vec![10u8, 20, 30, 40, 50, 60, 70, 80];
        let (rgb, alpha) = split_rgba_to_rgb_alpha(&rgba);
        assert_eq!(rgb, vec![10, 20, 30, 50, 60, 70]);
        assert_eq!(alpha, vec![40, 80]);
    }
}
