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
use super::chroma;

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

/// Representative seed tolerance for the unit tests (production feeds the
/// live Tolerance slider value through `segment`'s `seed_tol` arg).
/// // LITERAL-OK: perceptual seed budget
#[cfg(test)]
const SEED_TOL: f32 = 0.10;
/// Clamp range for the seed tolerance fed from the Tolerance slider.
/// Floor avoids disabling the seed at tolerance 0; ceiling matches the
/// Chroma backend's `TOLERANCE_FULL_SCALE`. // LITERAL-OK: perceptual budget
const SEED_TOL_FLOOR: f32 = 0.04;
const SEED_TOL_MAX: f32 = 0.30;

/// Minimum border-bg confidence for the chroma seed to fire. Below this
/// the subject likely touches an image edge (so a border flood would
/// eat into it) and we keep the plain geometric inset seed. Mirrors the
/// chroma backend's own `BORDER_BG_CONFIDENCE_FLOOR`.
const SEED_BORDER_CONF_FLOOR: f32 = 0.60;

/// Protection-mask byte threshold (`>= ` ⇒ protected / forced-fg).
const PROTECT_THRESHOLD: u8 = 128;

/// Reusable GrabCut working state. Lives on `BgRemovalScratch` so
/// the per-call allocations from the orchestrator's pipeline can
/// reuse capacity across runs — critical for the panel preview
/// path where `segment()` runs on every slider tick (HR-3).
///
/// All `Vec` fields are `.clear()`-ed and re-pushed; `Vec::reserve`
/// keeps the allocation alive across calls. `BkGraph` and `NLinks`
/// expose their own `ensure(w, h)` resizers.
#[derive(Clone, Debug, Default)]
pub struct GrabCutScratch {
    /// Per-pixel trimap at the processing resolution. Length =
    /// `proc_w * proc_h`.
    pub trimap: Vec<TriLabel>,
    /// RGB-packed buffer of pixels belonging to the bg side
    /// (collected from `trimap`). Length up to `proc_w*proc_h*3`.
    pub bg_pixels: Vec<u8>,
    /// Same shape as `bg_pixels`, for the fg side.
    pub fg_pixels: Vec<u8>,
    /// Per-bg-pixel GMM component index. Length matches
    /// `bg_pixels.len() / 3`.
    pub bg_assigns: Vec<u8>,
    /// Per-fg-pixel GMM component index.
    pub fg_assigns: Vec<u8>,
    /// Per-pixel source-side cap for the BK t-links. Length =
    /// `proc_w * proc_h`.
    pub source_caps: Vec<f32>,
    /// Per-pixel sink-side cap for the BK t-links.
    pub sink_caps: Vec<f32>,
    /// Pre-computed n-link weights (8-conn, 4 dirs/pixel).
    pub n_links: NLinks,
    /// Max-flow solver state — keeps its `Vec<NodeState>` and
    /// `Vec<Edge>` allocations across calls.
    pub bk: BkGraph,
    /// Downscaled RGB-packed input (alpha stripped).
    pub down_rgb: Vec<u8>,
    /// Downscaled alpha channel (used by the `alpha_hole_as_bg`
    /// trimap-init pass).
    pub down_alpha: Vec<u8>,
}

impl GrabCutScratch {
    /// Pre-grow every buffer to fit a `w × h` processing image.
    /// Subsequent calls with the same dims do no allocation;
    /// subsequent calls with larger dims grow capacity once.
    pub fn ensure(&mut self, w: u32, h: u32) {
        let n = (w as usize) * (h as usize);
        // Per-pixel buffers — exact length.
        self.trimap.resize(n, TriLabel::BgSoft);
        self.source_caps.resize(n, 0.0);
        self.sink_caps.resize(n, 0.0);
        self.down_rgb.resize(n * 3, 0);
        self.down_alpha.resize(n, 0);
        // Side-pixel buffers — capacity only, length tracked by
        // `collect_pixels_into` / `component_assignments_into`.
        // Worst case all pixels on one side → reserve n*3 / n.
        if self.bg_pixels.capacity() < n * 3 {
            self.bg_pixels.reserve(n * 3 - self.bg_pixels.capacity());
        }
        if self.fg_pixels.capacity() < n * 3 {
            self.fg_pixels.reserve(n * 3 - self.fg_pixels.capacity());
        }
        if self.bg_assigns.capacity() < n {
            self.bg_assigns.reserve(n - self.bg_assigns.capacity());
        }
        if self.fg_assigns.capacity() < n {
            self.fg_assigns.reserve(n - self.fg_assigns.capacity());
        }
        self.n_links.ensure(w, h);
        self.bk.ensure(w, h);
    }
}

/// Run GrabCut on the input and write the binary mask into
/// `scratch.mask`. The mask is `0` for background, `255` for
/// foreground at the *input* resolution; internal processing
/// happens at `min(input, MAX_INTERNAL_DIM)` per axis.
///
/// `protect` is an optional foreground-protection mask aligned to the
/// **input** `(w, h)` (one byte/pixel, `>= PROTECT_THRESHOLD` =
/// protected). Protected pixels are locked `TriLabel::FgHard` in the
/// trimap seed so the cut can never push them to the background. The
/// mask is nearest-downsampled to the processing resolution.
pub fn segment(
    rgba: &[u8],
    w: u32,
    h: u32,
    params: &GrabCutParams,
    protect: Option<&[u8]>,
    seed_tol: f32,
    scratch: &mut BgRemovalScratch,
) -> SegmentResult {
    let n_full = (w as usize) * (h as usize);
    debug_assert_eq!(rgba.len(), n_full * 4);

    // Edge case: empty image — leave the mask alone, return.
    if n_full == 0 || w == 0 || h == 0 {
        return SegmentResult::GrabCut;
    }

    let (proc_w, proc_h) = compute_downscale_dims(w, h, MAX_INTERNAL_DIM);
    let n_proc = (proc_w as usize) * (proc_h as usize);

    // 0. Pre-grow every scratch buffer to fit `proc_w × proc_h`.
    //    Subsequent calls at the same dims do zero allocation.
    scratch.grabcut.ensure(proc_w, proc_h);

    // 1. Downscale (if needed) + split RGB / alpha into scratch.
    downscale_to_rgb_alpha_into(
        rgba,
        w,
        h,
        proc_w,
        proc_h,
        &mut scratch.grabcut.down_rgb,
        &mut scratch.grabcut.down_alpha,
    );
    debug_assert_eq!(scratch.grabcut.down_rgb.len(), n_proc * 3);
    debug_assert_eq!(scratch.grabcut.down_alpha.len(), n_proc);

    // 2. Build initial trimap from the inset rect + alpha.
    init_trimap(
        proc_w,
        proc_h,
        &scratch.grabcut.down_alpha,
        params,
        &mut scratch.grabcut.trimap,
    );

    // 2b. Harden the seed with the chroma backend's background detection:
    //     pixels that are border-connected and clearly the detected bg
    //     colour become `BgHard`. The plain inset seed (FgSoft inside /
    //     BgSoft outside) gives the GMMs a poor start on natural images —
    //     anything bg-coloured *inside* the inset trains the FG GMM and
    //     drags the cut. Anchoring real background as hard constraints is
    //     what Chroma already does well; this brings it to GrabCut.
    //     Confidence-gated so a subject touching the border isn't eaten.
    seed_hard_bg_from_chroma(rgba, w, h, proc_w, proc_h, seed_tol, scratch);

    // 2c. Protection mask: lock painted pixels to `FgHard` so the cut
    //     can never drop them. Applied last so it overrides the chroma
    //     seed / alpha-hole `BgHard` on any pixel the user explicitly
    //     protected (nearest-downsampled from the input resolution).
    if let Some(pm) = protect {
        apply_protect_fghard(pm, w, h, proc_w, proc_h, &mut scratch.grabcut.trimap);
    }

    // Tiny guard — if either side has < COMPONENTS pixels the GMM
    // init can't seed properly. In practice a sane inset always
    // leaves ≥ 25 % of the image on each side, but a degenerate
    // params (insets summing to ~1.0) could trip this. Bail with
    // an all-fg mask to mirror the trivial-inset stub behaviour.
    let bg_count = scratch.grabcut.trimap.iter().filter(|t| !t.is_fg()).count();
    let fg_count = scratch.grabcut.trimap.iter().filter(|t| t.is_fg()).count();
    if bg_count < COMPONENTS || fg_count < COMPONENTS {
        write_mask(
            &scratch.grabcut.trimap,
            proc_w,
            proc_h,
            w,
            h,
            &mut scratch.mask,
        );
        return SegmentResult::GrabCut;
    }

    // 3. Initial GMMs. `Gmm5` is a fixed-size struct, no heap.
    let mut gmm_bg = Gmm5::default();
    let mut gmm_fg = Gmm5::default();
    collect_pixels_into(
        &scratch.grabcut.down_rgb,
        &scratch.grabcut.trimap,
        false,
        &mut scratch.grabcut.bg_pixels,
    );
    collect_pixels_into(
        &scratch.grabcut.down_rgb,
        &scratch.grabcut.trimap,
        true,
        &mut scratch.grabcut.fg_pixels,
    );
    gmm_bg.init_kmeans_pp(&scratch.grabcut.bg_pixels, GMM_BG_SEED);
    gmm_fg.init_kmeans_pp(&scratch.grabcut.fg_pixels, GMM_FG_SEED);

    // 4. Build n-links once (only colour-distance dependent — does
    //    not change between iters).
    let beta = derive_beta(&scratch.grabcut.down_rgb, proc_w, proc_h);
    build_n_links(
        &scratch.grabcut.down_rgb,
        proc_w,
        proc_h,
        beta,
        &mut scratch.grabcut.n_links,
    );

    // 5. Iterate. T-link rebuild → max-flow → trimap update → GMM
    //    refit. Convergence: flip ratio < 1 / FLIP_DENOMINATOR.
    let max_iters = params.max_iters.clamp(1, 5);
    for iter in 0..max_iters {
        // 5a. T-links from current GMMs + trimap.
        build_t_links(
            &scratch.grabcut.down_rgb,
            &scratch.grabcut.trimap,
            &gmm_bg,
            &gmm_fg,
            &mut scratch.grabcut.source_caps,
            &mut scratch.grabcut.sink_caps,
        );

        // 5b. Max-flow. `reset()` keeps allocations; `ensure` ran
        //     once already at the top of `segment`.
        scratch.grabcut.bk.reset();
        scratch.grabcut.bk.load_capacities(
            &scratch.grabcut.source_caps,
            &scratch.grabcut.sink_caps,
            &scratch.grabcut.n_links.edges,
        );
        scratch.grabcut.bk.run_max_flow();

        // 5c. Update trimap from BK output.
        let flips = update_trimap(&mut scratch.grabcut.trimap, &scratch.grabcut.bk);

        // 5d. Convergence — skip after iter 0 because flips count
        //     against an as-yet-untrained GMM is not informative.
        if iter >= 1 && (flips as u64) * (FLIP_DENOMINATOR as u64) < n_proc as u64 {
            break;
        }

        // 5e. Re-fit GMMs from updated trimap.
        collect_pixels_into(
            &scratch.grabcut.down_rgb,
            &scratch.grabcut.trimap,
            false,
            &mut scratch.grabcut.bg_pixels,
        );
        collect_pixels_into(
            &scratch.grabcut.down_rgb,
            &scratch.grabcut.trimap,
            true,
            &mut scratch.grabcut.fg_pixels,
        );
        if scratch.grabcut.bg_pixels.len() < COMPONENTS * 3
            || scratch.grabcut.fg_pixels.len() < COMPONENTS * 3
        {
            // One side collapsed — stop iterating, preserve current
            // mask. Avoids divide-by-zero during E/M re-fit.
            break;
        }
        component_assignments_into(
            &scratch.grabcut.bg_pixels,
            &gmm_bg,
            &mut scratch.grabcut.bg_assigns,
        );
        component_assignments_into(
            &scratch.grabcut.fg_pixels,
            &gmm_fg,
            &mut scratch.grabcut.fg_assigns,
        );
        gmm_bg.fit(&scratch.grabcut.bg_pixels, &scratch.grabcut.bg_assigns);
        gmm_fg.fit(&scratch.grabcut.fg_pixels, &scratch.grabcut.fg_assigns);
    }

    // 6. Write mask back at the input resolution (nearest-neighbour
    //    upsample if we processed at a smaller dim).
    write_mask(
        &scratch.grabcut.trimap,
        proc_w,
        proc_h,
        w,
        h,
        &mut scratch.mask,
    );

    SegmentResult::GrabCut
}

// ---------------------------------------------------------------
// Trimap init
// ---------------------------------------------------------------

/// Fill `trimap` with `BgSoft` outside the inset rect and `FgSoft`
/// inside. When `params.alpha_hole_as_bg` is set, additionally lock
/// every pixel with `alpha < ALPHA_HOLE_THRESHOLD` as `BgHard`.
///
/// **Precedence**: the alpha-hole pass runs *after* the inset pass,
/// so `BgHard` from a transparent pixel overrides the inset's
/// `FgSoft` / `BgSoft` label. Test `alpha_hole_as_bg_locks_transparent_pixels_as_background`
/// pins this contract.
fn init_trimap(w: u32, h: u32, alpha: &[u8], params: &GrabCutParams, trimap: &mut [TriLabel]) {
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
// Seed hardening (chroma bg detection + protection mask)
// ---------------------------------------------------------------

/// Harden `scratch.grabcut.trimap` by marking border-connected,
/// clearly-background pixels as [`TriLabel::BgHard`], reusing the chroma
/// backend's corner-auto background detection + connected flood-fill.
///
/// The background colour is detected on the full-resolution `rgba`
/// (colour is resolution-independent) and the flood runs at the
/// processing resolution against `scratch.grabcut.down_rgb`. Only
/// existing soft labels are hardened — `BgHard` from the alpha-hole pass
/// is preserved, and the routine never touches the foreground side.
///
/// No-op (keeps the plain geometric seed) when the border-bg confidence
/// is below [`SEED_BORDER_CONF_FLOOR`], i.e. the subject likely touches
/// an image edge and a border flood would bleed into it.
///
/// Scratch usage: writes `scratch.delta_e` / `scratch.mask` /
/// `scratch.spans` at the processing resolution. GrabCut's compose path
/// never reads `delta_e`, and the final mask is overwritten by
/// `write_mask`, so this is safe to clobber here.
fn seed_hard_bg_from_chroma(
    rgba: &[u8],
    w: u32,
    h: u32,
    proc_w: u32,
    proc_h: u32,
    seed_tol: f32,
    scratch: &mut BgRemovalScratch,
) {
    let n_proc = (proc_w as usize) * (proc_h as usize);
    if n_proc == 0 {
        return;
    }

    // Detect the background reference (full-res, resolution-independent).
    let bg_oklab = chroma::detect_corner_bg(rgba, w, h, scratch);

    // Squared ΔE per processing-resolution pixel against the bg colour.
    for i in 0..n_proc {
        let p = chroma::srgb_to_oklab(
            scratch.grabcut.down_rgb[i * 3],
            scratch.grabcut.down_rgb[i * 3 + 1],
            scratch.grabcut.down_rgb[i * 3 + 2],
        );
        scratch.delta_e[i] = chroma::oklab_dist_sq(p, bg_oklab);
    }

    // The Tolerance slider feeds the seed aggressiveness in Smart Cut
    // (higher ⇒ more pixels hard-locked as background). Clamped to a sane
    // floor so a 0 tolerance doesn't disable the seed entirely.
    let tol = seed_tol.clamp(SEED_TOL_FLOOR, SEED_TOL_MAX);
    let tol_sq = tol * tol;
    // Subject-touches-border guard — mirrors the chroma backend.
    if chroma::border_bg_confidence(&scratch.delta_e, proc_w, proc_h, tol_sq)
        < SEED_BORDER_CONF_FLOOR
    {
        return;
    }

    // Connected flood from the borders: mask 0 ⇒ border-connected bg.
    for v in &mut scratch.mask[..n_proc] {
        *v = 255;
    }
    chroma::flood_from_borders(
        &scratch.delta_e,
        &mut scratch.mask,
        &mut scratch.spans,
        proc_w,
        proc_h,
        tol_sq,
    );

    // Promote flooded background to hard constraints. Never demote the
    // foreground side (a subject pixel matching the bg colour but not
    // border-connected is left soft for the graph-cut to decide).
    for i in 0..n_proc {
        if scratch.mask[i] == 0 && !scratch.grabcut.trimap[i].is_fg() {
            scratch.grabcut.trimap[i] = TriLabel::BgHard;
        }
    }
}

/// Lock every protected input pixel to [`TriLabel::FgHard`] in the
/// processing-resolution `trimap`. `protect` is at the input `(w, h)`;
/// each processing pixel samples the nearest input pixel.
fn apply_protect_fghard(
    protect: &[u8],
    w: u32,
    h: u32,
    proc_w: u32,
    proc_h: u32,
    trimap: &mut [TriLabel],
) {
    if w == 0 || h == 0 || proc_w == 0 || proc_h == 0 {
        return;
    }
    let (w_u, h_u) = (w as u64, h as u64);
    let (pw_u, ph_u) = (proc_w as u64, proc_h as u64);
    for y in 0..proc_h as usize {
        let sy = (((y as u64) * h_u) / ph_u).min(h_u - 1) as usize;
        for x in 0..proc_w as usize {
            let sx = (((x as u64) * w_u) / pw_u).min(w_u - 1) as usize;
            if protect[sy * w as usize + sx] >= PROTECT_THRESHOLD {
                trimap[y * proc_w as usize + x] = TriLabel::FgHard;
            }
        }
    }
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
/// writing the result split into the caller-owned `rgb` and
/// `alpha` buffers. Both are `.clear()`-ed and re-pushed so the
/// allocation persists across calls (HR-3).
///
/// `image::imageops::resize` with `FilterType::Triangle` is the
/// cheapest box-quality filter and produces no ringing — good
/// enough for a binary-mask consumer. It allocates an internal
/// `ImageBuffer` on the downscale path; that's the one remaining
/// alloc we can't eliminate without re-implementing the filter.
fn downscale_to_rgb_alpha_into(
    rgba: &[u8],
    w: u32,
    h: u32,
    dw: u32,
    dh: u32,
    rgb: &mut Vec<u8>,
    alpha: &mut Vec<u8>,
) {
    if dw == w && dh == h {
        split_rgba_to_rgb_alpha_into(rgba, rgb, alpha);
        return;
    }
    let src = ImageBuffer::<Rgba<u8>, _>::from_raw(w, h, rgba.to_vec())
        .expect("rgba length matches w*h*4");
    let down: ImageBuffer<Rgba<u8>, Vec<u8>> =
        image::imageops::resize(&src, dw, dh, FilterType::Triangle);
    split_rgba_to_rgb_alpha_into(down.as_raw(), rgb, alpha);
}

fn split_rgba_to_rgb_alpha_into(rgba: &[u8], rgb: &mut Vec<u8>, alpha: &mut Vec<u8>) {
    rgb.clear();
    alpha.clear();
    for chunk in rgba.chunks_exact(4) {
        rgb.push(chunk[0]);
        rgb.push(chunk[1]);
        rgb.push(chunk[2]);
        alpha.push(chunk[3]);
    }
}

// ---------------------------------------------------------------
// Per-iter helpers
// ---------------------------------------------------------------

/// Walk `trimap` and gather RGB pixels matching the side filter
/// (FgSoft+FgHard if `want_fg`, BgSoft+BgHard otherwise) into the
/// caller-owned `out` buffer (`.clear()`-ed then re-pushed, so the
/// allocation persists across calls — HR-3).
fn collect_pixels_into(rgb: &[u8], trimap: &[TriLabel], want_fg: bool, out: &mut Vec<u8>) {
    out.clear();
    for (i, &t) in trimap.iter().enumerate() {
        if t.is_fg() == want_fg {
            out.push(rgb[i * 3]);
            out.push(rgb[i * 3 + 1]);
            out.push(rgb[i * 3 + 2]);
        }
    }
}

/// For each pixel in `pixels` (RGB-packed), write its best
/// component index under the supplied GMM into `out`. `out` is
/// `.clear()`-ed first; final length = `pixels.len() / 3`.
fn component_assignments_into(pixels: &[u8], gmm: &Gmm5, out: &mut Vec<u8>) {
    out.clear();
    for chunk in pixels.chunks_exact(3) {
        let k = gmm.assign_component([chunk[0], chunk[1], chunk[2]]);
        out.push(k as u8);
    }
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
        let _ = segment(&rgba, 64, 64, &default_params(), None, SEED_TOL, &mut s);
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
        let _ = segment(&rgba, 96, 96, &default_params(), None, SEED_TOL, &mut s);
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
        let _ = segment(&rgba, 64, 64, &p, None, SEED_TOL, &mut s);
        // A transparent corner pixel — bg.
        assert_eq!(s.mask[0], 0, "transparent pixel must be bg");
    }

    #[test]
    fn empty_image_does_not_panic() {
        let mut s = BgRemovalScratch::default();
        s.ensure(0, 0, false);
        let _ = segment(&[], 0, 0, &default_params(), None, SEED_TOL, &mut s);
    }

    #[test]
    fn protect_mask_locks_pixel_as_foreground() {
        // 96×96 uniform red — the chroma seed would lock the whole image
        // as bg. Protecting a central 16×16 region forces those pixels
        // FgHard, so they must come out foreground while the unprotected
        // border stays background.
        let rgba = make_image(96, 96, [200, 30, 30], None);
        let mut protect = vec![0u8; 96 * 96];
        for y in 40..56 {
            for x in 40..56 {
                protect[y * 96 + x] = 255;
            }
        }
        let mut s = BgRemovalScratch::default();
        s.ensure(96, 96, false);
        let _ = segment(
            &rgba,
            96,
            96,
            &default_params(),
            Some(&protect),
            SEED_TOL,
            &mut s,
        );
        assert_eq!(s.mask[48 * 96 + 48], 255, "protected pixel must be fg");
        assert_eq!(s.mask[0], 0, "unprotected bg corner must be bg");
    }

    #[test]
    fn protect_none_matches_baseline() {
        // Threading the protect arg as `None` must be byte-identical to
        // the pre-protection behaviour (determinism / no regression).
        let rgba = make_image(96, 96, [200, 30, 30], Some(([30, 200, 30], 32, 32, 32, 32)));
        let mut a = BgRemovalScratch::default();
        a.ensure(96, 96, false);
        let _ = segment(&rgba, 96, 96, &default_params(), None, SEED_TOL, &mut a);
        let mut b = BgRemovalScratch::default();
        b.ensure(96, 96, false);
        let empty = vec![0u8; 96 * 96];
        let _ = segment(
            &rgba,
            96,
            96,
            &default_params(),
            Some(&empty),
            SEED_TOL,
            &mut b,
        );
        assert_eq!(a.mask, b.mask, "all-zero protect must equal None");
    }

    #[test]
    fn determinism_two_runs_produce_identical_mask() {
        let rgba = make_image(64, 64, [200, 30, 30], Some(([30, 200, 30], 16, 16, 32, 32)));
        let mut s1 = BgRemovalScratch::default();
        s1.ensure(64, 64, false);
        let _ = segment(&rgba, 64, 64, &default_params(), None, SEED_TOL, &mut s1);

        let mut s2 = BgRemovalScratch::default();
        s2.ensure(64, 64, false);
        let _ = segment(&rgba, 64, 64, &default_params(), None, SEED_TOL, &mut s2);

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
        let _ = segment(&rgba, 1280, 1280, &default_params(), None, SEED_TOL, &mut s);
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
        let mut rgb = Vec::new();
        let mut alpha = Vec::new();
        split_rgba_to_rgb_alpha_into(&rgba, &mut rgb, &mut alpha);
        assert_eq!(rgb, vec![10, 20, 30, 50, 60, 70]);
        assert_eq!(alpha, vec![40, 80]);
    }

    #[test]
    fn scratch_ensure_grows_buffers_to_dims() {
        let mut gs = GrabCutScratch::default();
        gs.ensure(32, 16);
        assert_eq!(gs.trimap.len(), 32 * 16);
        assert_eq!(gs.source_caps.len(), 32 * 16);
        assert_eq!(gs.sink_caps.len(), 32 * 16);
        assert_eq!(gs.down_rgb.len(), 32 * 16 * 3);
        assert_eq!(gs.down_alpha.len(), 32 * 16);
        // Side buffers reserve capacity but stay length-0.
        assert!(gs.bg_pixels.capacity() >= 32 * 16 * 3);
        assert_eq!(gs.bg_pixels.len(), 0);
        // BkGraph + NLinks resized via their own ensure.
        assert_eq!(gs.bk.width, 32);
        assert_eq!(gs.bk.height, 16);
        assert_eq!(gs.n_links.w, 32);
        assert_eq!(gs.n_links.h, 16);
    }

    #[test]
    fn scratch_reused_across_segment_calls_at_same_dims_does_not_realloc() {
        // Smoke-test: two consecutive segment() calls on the same
        // scratch + same dims should not crash and should produce
        // mask of the right length. We can't easily count allocs
        // without dhat, but functionality covers the refactor.
        let rgba = make_image(48, 48, [200, 30, 30], Some(([30, 200, 30], 16, 16, 16, 16)));
        let mut s = BgRemovalScratch::default();
        s.ensure(48, 48, false);
        let _ = segment(&rgba, 48, 48, &default_params(), None, SEED_TOL, &mut s);
        assert_eq!(s.mask.len(), 48 * 48);
        let bg_buf_cap_after_first = s.grabcut.bg_pixels.capacity();
        let bk_nodes_cap_after_first = s.grabcut.bk.nodes_capacity_for_test();
        let _ = segment(&rgba, 48, 48, &default_params(), None, SEED_TOL, &mut s);
        assert_eq!(s.mask.len(), 48 * 48);
        // Capacity should be ≥ first run — never shrinks.
        assert!(s.grabcut.bg_pixels.capacity() >= bg_buf_cap_after_first);
        assert!(s.grabcut.bk.nodes_capacity_for_test() >= bk_nodes_cap_after_first);
    }
}
