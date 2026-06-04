//! Chroma key in Oklab + corner-auto bg detection + connected
//! flood-fill from image borders.
//!
//! Pipeline:
//!
//! 1. **Sample** 16 dispersed `8×8` patches across the 4 image
//!    corners → 1024 sRGB pixels → 1024 Oklab points. Dispersed
//!    sub-patches dodge JPEG DCT block boundaries (a single
//!    `16×16` corner often lands on a block boundary and reads as
//!    a ringed average; four `8×8` sub-patches average that
//!    ringing out).
//! 2. **Cluster** the samples with k-means, k=2, serial Lloyd, max
//!    8 iters, deterministic init (min/max-luma centroid seeds).
//!    Pick the larger-membership cluster's centroid as `bg_oklab`.
//! 3. **Compute** Oklab Euclidean distance (ΔE) per pixel to
//!    `bg_oklab`. Cache squared distance in `scratch.delta_e`
//!    (compose step does the soft-band math).
//! 4. **Border-bg confidence** check: fraction of perimeter pixels
//!    with ΔE < tolerance. If < 60%, the subject likely touches an
//!    edge — fall back to a global threshold (no flood) so the
//!    subject is not erased.
//! 5. **Connected flood-fill** from every border pixel that passes
//!    `is_bg` — scanline BFS-on-spans, 4-connectivity. Only pixels
//!    reachable from a border are marked bg; interior pixels with
//!    the bg colour (e.g. a sky-blue eye on a sky-blue background)
//!    stay foreground.
//!
//! Override hook: if `params.reference_color = Some(rgb)`, step 1+2
//! are skipped and the given color is used directly as `bg_oklab`.
//!
//! Output:
//! - `scratch.delta_e[i]` = Oklab distance² (squared!) of pixel `i`
//!   to `bg_oklab`. The compose step compares against `tol²` and
//!   `(tol + feather)²` to derive the soft band — squaring avoids
//!   a `sqrt` per pixel.
//! - `scratch.mask[i]` = 0 (background) or 255 (foreground).
//!
//! Constraints respected:
//! - **HR-3** — all scratch space comes from `BgRemovalScratch`;
//!   inner loops do not allocate. Corner samples / assignments are
//!   `.clear()` + repushed into pre-grown vectors.
//! - **HR-5** — k-means runs serial with deterministic seeding; the
//!   same input gives bit-identical output across runs / platforms.

use super::super::params::ChromaParams;
use super::super::scratch::{BgRemovalScratch, FloodSpan};
use super::SegmentResult;

/// Side length of one corner sub-patch in pixels. 4 sub-patches per
/// corner × 4 corners = 16 patches × 64 px = 1024 sRGB samples.
const PATCH: u32 = 8;
/// Sub-patches per corner. Dispersed inside a `(2 * STRIDE)` square
/// at each corner.
const SUB_PER_CORNER: u32 = 4;
/// Inter-patch stride within a corner block. Picked so the 4 patches
/// don't share a JPEG 8×8 DCT block.
const STRIDE: u32 = PATCH + 4;
/// Largest `(x, y)` offset emitted by `sample_corner_patches` for
/// the corner-anchored arrangement (`(SUB_PER_CORNER/2) * STRIDE
/// + STRIDE/2`). Used to derive the minimum image dimension that
/// the big-image path can handle without u32 underflow on the
/// top-right / bottom-left corner offsets (`w - PATCH - off`).
const MAX_CORNER_OFFSET: u32 = STRIDE + STRIDE / 2;
/// Below this dimension on either axis, fall back to a single
/// top-left `PATCH × PATCH` tile. Set to `2 * (MAX_CORNER_OFFSET +
/// PATCH)` so that the top-left and top-right corners (and BL/BR)
/// sample **disjoint** pixels — at the minimum threshold, TL covers
/// columns `[ox, ox+PATCH)` and TR covers `[w - PATCH - ox, w - ox)`
/// which are non-overlapping iff `w >= 2 * (MAX_CORNER_OFFSET +
/// PATCH)`. Without this, very small big-path inputs would sample
/// the same pixels 4× and bias the k-means.
const MIN_BIG_DIM: u32 = 2 * (MAX_CORNER_OFFSET + PATCH);

/// Maximum Lloyd iterations for the k=2 corner-cluster k-means.
const KMEANS_MAX_ITERS: u32 = 8;

/// Minimum fraction of border pixels classified as background for
/// the connected flood-fill to be trusted. Below this, the subject
/// most likely touches an edge and we fall back to global threshold.
const BORDER_BG_CONFIDENCE_FLOOR: f32 = 0.60;

// =============================================================
// Public entry point
// =============================================================

/// Run the chroma+flood segmentation. Writes `scratch.delta_e`
/// (squared ΔE in Oklab) and `scratch.mask` (0/255). Returns the
/// detected/specified background in Oklab so the compose step can
/// despill against it.
///
/// # Panics
/// Panics if `rgba.len() != (w * h * 4) as usize`.
/// `extra_colors` are additional user-picked background references
/// (sRGB 8-bit, from the panel eyedropper). A pixel is treated as
/// background-similar when it is close to the auto/override
/// `bg_oklab` **OR** to any extra colour — so the user can knock out
/// multi-coloured backgrounds the corner-auto pass misses. Pass an
/// empty slice for auto-only behaviour.
pub fn segment(
    rgba: &[u8],
    w: u32,
    h: u32,
    params: &ChromaParams,
    extra_colors: &[[u8; 3]],
    scratch: &mut BgRemovalScratch,
) -> SegmentResult {
    let n = (w as usize) * (h as usize);
    assert_eq!(rgba.len(), n * 4);

    // Edge case: 0-sized image. Nothing to do; black bg.
    if n == 0 || w == 0 || h == 0 {
        return SegmentResult::Chroma {
            bg_oklab: [0.0, 0.0, 0.0],
        };
    }

    // 1 — detect bg (or accept the override).
    let bg_oklab = match params.reference_color {
        Some(rgb) => srgb_to_oklab(rgb[0], rgb[1], rgb[2]),
        None => detect_corner_bg(rgba, w, h, scratch),
    };

    // 2 — compute squared ΔE per pixel. Sanitize tolerance: NaN
    // would silently propagate through `delta_e <= tol_sq` and
    // mark every pixel as foreground (NaN compares false against
    // everything), so collapse non-finite values to 0.
    let tol = if params.tolerance.is_finite() {
        params.tolerance.max(0.0)
    } else {
        0.0
    };
    let tol_sq = tol * tol;
    fill_delta_e_sq(
        rgba,
        n,
        bg_oklab,
        &mut scratch.delta_e,
        &mut scratch.pixels_oklab,
    );

    // Build the "near any user-picked extra" predicate WITHOUT
    // touching `delta_e`. Critical bug fix (Enio 2026-05-26 audit):
    // the previous code did `delta_e[i] = min(d_bg, d_extra...)`,
    // which means a single extra colour lowered the global ΔE field
    // that drives `border_bg_confidence`, flood seeding, flood
    // PROPAGATION, and the interior-pocket sweep. The flood then
    // bridged the foreground via paths of pixels merely similar to
    // the extra, devouring the entire image. By computing extras as a
    // separate per-pixel bool and OR'ing it into the mask AFTER the
    // flood, extras can only knock out pixels directly close to a
    // pick — no cascading. Uses the cached `pixels_oklab` so cost is
    // K × N float-distance ops (no `srgb_to_oklab` in the inner
    // loop): for K=5 at 512² that's ~1.3 M ops instead of ~1.3 M
    // `srgb_to_oklab` calls (~5× cheaper).
    for v in scratch.is_near_extra[..n].iter_mut() {
        *v = 0;
    }
    if !extra_colors.is_empty() {
        // K ≤ MAX_EXTRA_BG_COLORS (12), so a stack array fits.
        let mut extra_oklabs: [[f32; 3]; crate::params::MAX_EXTRA_BG_COLORS] =
            [[0.0; 3]; crate::params::MAX_EXTRA_BG_COLORS];
        let k = extra_colors.len().min(crate::params::MAX_EXTRA_BG_COLORS);
        for (i, &rgb) in extra_colors.iter().take(k).enumerate() {
            extra_oklabs[i] = srgb_to_oklab(rgb[0], rgb[1], rgb[2]);
        }
        for i in 0..n {
            let base = i * 4;
            if rgba[base + 3] == 0 {
                continue;
            }
            let p = scratch.pixels_oklab[i];
            for &e in extra_oklabs.iter().take(k) {
                if oklab_dist_sq(p, e) <= tol_sq {
                    scratch.is_near_extra[i] = 1;
                    break;
                }
            }
        }
    }

    // 3 — decide flood vs threshold-only. Uses the UNCORRUPTED `delta_e`
    // (relative to the auto-detected bg) so border confidence reflects
    // the actual border similarity to that bg — not an extras-amplified
    // value that would force the flood path on every pick.
    let use_flood = params.use_flood && {
        let conf = border_bg_confidence(&scratch.delta_e, w, h, tol_sq);
        conf >= BORDER_BG_CONFIDENCE_FLOOR
    };

    // 4 — write the mask.
    if use_flood {
        // Start with every pixel as fg; the flood paints bg from
        // the border inward.
        for v in &mut scratch.mask[..n] {
            *v = 255;
        }
        flood_from_borders(
            &scratch.delta_e,
            &mut scratch.mask,
            &mut scratch.spans,
            w,
            h,
            tol_sq,
        );
        // Interior bg pockets: the border flood only removes bg-similar
        // regions CONNECTED to the image border, so ENCLOSED bg-similar
        // pockets (e.g. the gaps between a character's arm and torso)
        // stay foreground. Sweep them too — any still-fg pixel that is
        // HARD background (ΔE² ≤ tol²) is an interior hole, not subject.
        // Anti-aliased subject-edge pixels sit in the soft band
        // (ΔE² > tol², ≤ (tol+feather)²) so they survive this sweep and
        // are handled by the compose soft-edge.
        for i in 0..n {
            if scratch.mask[i] != 0 && scratch.delta_e[i] <= tol_sq {
                scratch.mask[i] = 0;
            }
        }
    } else {
        // Fallback: global threshold. No connectivity guard.
        for i in 0..n {
            scratch.mask[i] = if scratch.delta_e[i] <= tol_sq { 0 } else { 255 };
        }
    }

    // 5 — extras post-pass. Pixels directly near any user pick become
    // hard background WITHOUT participating in flood propagation. This
    // is what makes Pick Colors safe again: a single bad pick can only
    // remove pixels of that colour, not bridge through the subject.
    if !extra_colors.is_empty() {
        for i in 0..n {
            if scratch.is_near_extra[i] != 0 {
                scratch.mask[i] = 0;
            }
        }
    }

    SegmentResult::Chroma { bg_oklab }
}

// =============================================================
// sRGB → Oklab (inline, no external crate)
// =============================================================

// =============================================================
// Corner sampling + serial k-means
// =============================================================

/// Sample corner patches and k-means-cluster them; return the
/// larger cluster's centroid as the detected bg.
///
/// Reused by [`super::grabcut`] to seed its trimap with a real
/// background reference (the background colour is resolution-
/// independent, so GrabCut detects it on the full-res input and then
/// floods at its own processing resolution).
pub(crate) fn detect_corner_bg(
    rgba: &[u8],
    w: u32,
    h: u32,
    scratch: &mut BgRemovalScratch,
) -> [f32; 3] {
    scratch.corner_samples.clear();
    sample_corner_patches(rgba, w, h, &mut scratch.corner_samples);

    if scratch.corner_samples.is_empty() {
        // Pathological tiny image — fall back to top-left pixel.
        let r = rgba[0];
        let g = rgba[1];
        let b = rgba[2];
        return srgb_to_oklab(r, g, b);
    }

    let samples = scratch.corner_samples.as_slice();
    scratch.corner_assignments.clear();
    scratch.corner_assignments.resize(samples.len(), 0u8);
    kmeans_k2(samples, &mut scratch.corner_assignments)
}

/// Fill `out` with Oklab samples from 16 dispersed `8×8` patches
/// (4 per corner). Skips patches that don't fit (images smaller
/// than `2 * STRIDE`).
fn sample_corner_patches(rgba: &[u8], w: u32, h: u32, out: &mut Vec<[f32; 3]>) {
    // Tiny images: sample every pixel within a (PATCH × PATCH)
    // top-left tile. Avoids zero-sample case AND u32 underflow on
    // the corner-mirror offsets when either axis is below the size
    // that lets all 16 patches fit (`MIN_BIG_DIM`).
    if w < MIN_BIG_DIM || h < MIN_BIG_DIM {
        let pw = PATCH.min(w);
        let ph = PATCH.min(h);
        for py in 0..ph {
            for px in 0..pw {
                push_pixel_oklab(rgba, w, px, py, out);
            }
        }
        return;
    }

    // For each corner, place `SUB_PER_CORNER` patches dispersed
    // inside a `2 × STRIDE` block at each corner. Offsets are
    // computed inline (no `Vec::collect` — HR-3: zero heap alloc
    // in the hot path).
    for i in 0..SUB_PER_CORNER {
        let ox = (i / 2) * STRIDE + (i % 2) * STRIDE / 2;
        let oy = (i % 2) * STRIDE + (i / 2) * STRIDE / 2;
        // Top-left.
        push_patch_oklab(rgba, w, ox, oy, out);
        // Top-right.
        push_patch_oklab(rgba, w, w - PATCH - ox, oy, out);
        // Bottom-left.
        push_patch_oklab(rgba, w, ox, h - PATCH - oy, out);
        // Bottom-right.
        push_patch_oklab(rgba, w, w - PATCH - ox, h - PATCH - oy, out);
    }
}

/// Read one pixel as Oklab and push to `out` — **only if it is
/// sufficiently opaque** (alpha ≥ `ALPHA_SAMPLE_FLOOR`). Skipping
/// transparent pixels prevents their (formally undefined) RGB
/// payload from poisoning the corner-cluster k-means: a PNG sprite
/// with transparent borders / corners would otherwise let "alpha-
/// premultiplied-by-the-PNG-encoder zero RGB" cast votes for
/// pitch-black bg.
#[inline(always)]
fn push_pixel_oklab(rgba: &[u8], w: u32, x: u32, y: u32, out: &mut Vec<[f32; 3]>) {
    let idx = ((y as usize) * (w as usize) + x as usize) * 4;
    if rgba[idx + 3] < ALPHA_SAMPLE_FLOOR {
        return;
    }
    out.push(srgb_to_oklab(rgba[idx], rgba[idx + 1], rgba[idx + 2]));
}

/// Minimum source alpha for a pixel to contribute to corner
/// sampling. ~6% opacity — below this the pixel's RGB is mostly
/// noise / encoder garbage from anti-aliased edges.
const ALPHA_SAMPLE_FLOOR: u8 = 16;

/// Read a `PATCH × PATCH` region starting at `(ox, oy)` and push
/// every pixel's Oklab into `out`. No bounds clamp — caller is
/// responsible for ensuring the patch fits.
fn push_patch_oklab(rgba: &[u8], w: u32, ox: u32, oy: u32, out: &mut Vec<[f32; 3]>) {
    for dy in 0..PATCH {
        for dx in 0..PATCH {
            push_pixel_oklab(rgba, w, ox + dx, oy + dy, out);
        }
    }
}

/// Serial k=2 Lloyd k-means in Oklab. Deterministic init: centroid
/// A = sample with lowest L, centroid B = sample with highest L
/// (tie-broken by index → deterministic). Returns the centroid of
/// the cluster with the larger membership.
fn kmeans_k2(samples: &[[f32; 3]], assignments: &mut [u8]) -> [f32; 3] {
    debug_assert_eq!(samples.len(), assignments.len());
    if samples.is_empty() {
        return [0.0; 3];
    }
    if samples.len() == 1 {
        return samples[0];
    }

    // Init: pick min-L and max-L samples as initial centroids.
    let (mut ci_a, mut ci_b) = (0usize, 0usize);
    let (mut min_l, mut max_l) = (samples[0][0], samples[0][0]);
    for (i, s) in samples.iter().enumerate() {
        if s[0] < min_l {
            min_l = s[0];
            ci_a = i;
        }
        if s[0] > max_l {
            max_l = s[0];
            ci_b = i;
        }
    }
    if ci_a == ci_b {
        // Monochrome corners; pick any second seed.
        ci_b = (ci_a + 1) % samples.len();
    }
    let mut centroid_a = samples[ci_a];
    let mut centroid_b = samples[ci_b];

    let mut last_changes = u32::MAX;
    for _ in 0..KMEANS_MAX_ITERS {
        // Assign.
        let mut changes = 0u32;
        for (i, s) in samples.iter().enumerate() {
            let da = oklab_dist_sq(*s, centroid_a);
            let db = oklab_dist_sq(*s, centroid_b);
            let new_assign = if da <= db { 0u8 } else { 1u8 };
            if assignments[i] != new_assign {
                changes += 1;
                assignments[i] = new_assign;
            }
        }
        // Early-exit once no sample changes cluster. The first
        // pass starts from `assignments` initialised to `0` (so
        // it always records "changes"), guaranteeing centroid_b
        // gets at least one update — no risk of premature exit.
        if changes == 0 && last_changes != u32::MAX {
            break;
        }
        last_changes = changes;

        // Update centroids — serial sum, deterministic order.
        let (mut sa, mut sb, mut na, mut nb) = ([0.0f32; 3], [0.0f32; 3], 0u32, 0u32);
        for (i, s) in samples.iter().enumerate() {
            match assignments[i] {
                0 => {
                    sa[0] += s[0];
                    sa[1] += s[1];
                    sa[2] += s[2];
                    na += 1;
                }
                _ => {
                    sb[0] += s[0];
                    sb[1] += s[1];
                    sb[2] += s[2];
                    nb += 1;
                }
            }
        }
        if na > 0 {
            let inv = 1.0 / (na as f32);
            centroid_a = [sa[0] * inv, sa[1] * inv, sa[2] * inv];
        }
        if nb > 0 {
            let inv = 1.0 / (nb as f32);
            centroid_b = [sb[0] * inv, sb[1] * inv, sb[2] * inv];
        }
    }

    // Final membership tally. `>=` tie-breaks toward `centroid_a`
    // (the min-L seed) — deterministic regardless of sample order.
    let (mut na, mut nb) = (0u32, 0u32);
    for a in assignments.iter() {
        match a {
            0 => na += 1,
            _ => nb += 1,
        }
    }
    if na >= nb { centroid_a } else { centroid_b }
}

// =============================================================
// ΔE² fill + border-confidence + flood
// =============================================================

/// Fill `out[..n]` with squared Oklab distance of every pixel to
/// `bg_oklab`. Fully-transparent pixels (`a == 0`) get `ΔE² = 0`
/// so they classify as background without participating in the
/// colour comparison — their RGB is, per spec, undefined garbage
/// for source PNGs and would otherwise produce visible artefacts
/// at edges of sprites that already carry alpha holes.
///
/// Exact-match opaque pixels (input color equals `bg_oklab` to FP
/// noise precision) get a hard `0.0` instead of `~1e-14` — without
/// this floor, `delta_e <= tol_sq` would silently fail at `tolerance
/// = 0` because the k-means centroid accumulator can drift by a few
/// ULPs vs. a fresh round-trip through `srgb_to_oklab`.
///
/// Serial — fast enough at 4k (~10 ns/px). Caches each pixel's OkLab
/// into `oklab_out` so the extras-folding loop in [`segment`] can
/// reuse the conversion (was K×N before the cache — a 5-extra slider
/// drag at 512² did ~1.3 M `srgb_to_oklab` calls per pipeline run and
/// visibly froze the UI).
fn fill_delta_e_sq(
    rgba: &[u8],
    n: usize,
    bg_oklab: [f32; 3],
    delta_e_out: &mut [f32],
    oklab_out: &mut [[f32; 3]],
) {
    debug_assert!(delta_e_out.len() >= n);
    debug_assert!(oklab_out.len() >= n);
    // Below this, `oklab_dist_sq` is dominated by float noise from
    // the round-trip; floor to exact 0 so the `<=` test against a
    // user `tol_sq = 0` still treats matches as background.
    const FP_NOISE_FLOOR_SQ: f32 = 1.0e-6;
    for i in 0..n {
        let base = i * 4;
        if rgba[base + 3] == 0 {
            delta_e_out[i] = 0.0;
            // Sentinel for transparent — readers must guard via alpha.
            oklab_out[i] = [0.0, 0.0, 0.0];
            continue;
        }
        let p = srgb_to_oklab(rgba[base], rgba[base + 1], rgba[base + 2]);
        oklab_out[i] = p;
        let d = oklab_dist_sq(p, bg_oklab);
        delta_e_out[i] = if d < FP_NOISE_FLOOR_SQ { 0.0 } else { d };
    }
}

/// Fraction of perimeter pixels with ΔE² ≤ `tol_sq`. Returns 0 for
/// pathological 1-px-wide images (n_border == 0 only when both
/// w == 0 or h == 0, which we early-returned earlier).
pub(crate) fn border_bg_confidence(delta_e: &[f32], w: u32, h: u32, tol_sq: f32) -> f32 {
    let (w_us, h_us) = (w as usize, h as usize);
    if w_us == 0 || h_us == 0 {
        return 0.0;
    }

    let mut bg = 0u32;
    let mut total = 0u32;

    // Top + bottom rows.
    for x in 0..w_us {
        if delta_e[x] <= tol_sq {
            bg += 1;
        }
        total += 1;
        if h_us > 1 {
            let i = (h_us - 1) * w_us + x;
            if delta_e[i] <= tol_sq {
                bg += 1;
            }
            total += 1;
        }
    }
    // Left + right columns (excluding corners already counted).
    if h_us > 2 {
        for y in 1..h_us - 1 {
            let i_left = y * w_us;
            let i_right = i_left + w_us - 1;
            if delta_e[i_left] <= tol_sq {
                bg += 1;
            }
            total += 1;
            if w_us > 1 {
                if delta_e[i_right] <= tol_sq {
                    bg += 1;
                }
                total += 1;
            }
        }
    }

    if total == 0 {
        0.0
    } else {
        (bg as f32) / (total as f32)
    }
}

/// Connected flood-fill from every border pixel that passes
/// `delta_e <= tol_sq`. BFS-on-spans (scanline), 4-connectivity.
///
/// Invariant on entry: `mask[..n]` is all-255 (foreground).
/// On exit: pixels reachable from a "bg-similar" border pixel
/// via 4-connected paths of bg-similar pixels are set to 0.
pub(crate) fn flood_from_borders(
    delta_e: &[f32],
    mask: &mut [u8],
    spans: &mut Vec<FloodSpan>,
    w: u32,
    h: u32,
    tol_sq: f32,
) {
    let (w_us, h_us) = (w as usize, h as usize);
    if w_us == 0 || h_us == 0 {
        return;
    }
    spans.clear();

    let is_bg = |i: usize| delta_e[i] <= tol_sq;

    // Seed: every border pixel that is bg-similar AND still fg-marked.
    // We immediately convert to a span by extending to its full run.
    // The mask is set inside `seed_span_at` to 0 so the run is
    // marked visited.
    let seed_span_at = |x: u32, y: u32, mask: &mut [u8], spans: &mut Vec<FloodSpan>| {
        let row = (y as usize) * w_us;
        let i = row + x as usize;
        if mask[i] == 0 || !is_bg(i) {
            return;
        }
        // Extend left.
        let mut xl = x;
        while xl > 0 && is_bg(row + (xl - 1) as usize) && mask[row + (xl - 1) as usize] != 0 {
            xl -= 1;
        }
        // Extend right.
        let mut xr = x;
        while (xr as usize) < w_us - 1
            && is_bg(row + (xr + 1) as usize)
            && mask[row + (xr + 1) as usize] != 0
        {
            xr += 1;
        }
        for xi in xl..=xr {
            mask[row + xi as usize] = 0;
        }
        spans.push(FloodSpan {
            y,
            x_left: xl,
            x_right: xr,
        });
    };

    // Top + bottom rows.
    for x in 0..w {
        seed_span_at(x, 0, mask, spans);
        if h_us > 1 {
            seed_span_at(x, h - 1, mask, spans);
        }
    }
    // Left + right columns.
    if h_us > 2 {
        for y in 1..(h - 1) {
            seed_span_at(0, y, mask, spans);
            if w_us > 1 {
                seed_span_at(w - 1, y, mask, spans);
            }
        }
    }

    // BFS. Pop a span; for each of its two vertical neighbours,
    // scan the run for new bg-similar pixels and push them as spans.
    while let Some(s) = spans.pop() {
        if s.y > 0 {
            scan_row_neighbours(
                delta_e,
                mask,
                spans,
                w,
                s.y - 1,
                s.x_left,
                s.x_right,
                tol_sq,
            );
        }
        if (s.y as usize) < h_us - 1 {
            scan_row_neighbours(
                delta_e,
                mask,
                spans,
                w,
                s.y + 1,
                s.x_left,
                s.x_right,
                tol_sq,
            );
        }
    }
}

/// Scan `[x_left..=x_right]` on row `y` for runs of bg-similar
/// pixels still marked fg; turn each into a new span and push it.
///
/// Argument count is intentionally high — the alternative is a
/// borrow-restricted helper struct that buys nothing in clarity.
#[allow(clippy::too_many_arguments)]
fn scan_row_neighbours(
    delta_e: &[f32],
    mask: &mut [u8],
    spans: &mut Vec<FloodSpan>,
    w: u32,
    y: u32,
    x_left: u32,
    x_right: u32,
    tol_sq: f32,
) {
    let row = (y as usize) * (w as usize);
    let mut x = x_left;
    while x <= x_right {
        let i = row + x as usize;
        if mask[i] != 0 && delta_e[i] <= tol_sq {
            // Found a run start; extend left + right.
            let mut xl = x;
            while xl > 0
                && delta_e[row + (xl - 1) as usize] <= tol_sq
                && mask[row + (xl - 1) as usize] != 0
            {
                xl -= 1;
            }
            let mut xr = x;
            while (xr as usize) < (w as usize) - 1
                && delta_e[row + (xr + 1) as usize] <= tol_sq
                && mask[row + (xr + 1) as usize] != 0
            {
                xr += 1;
            }
            for xi in xl..=xr {
                mask[row + xi as usize] = 0;
            }
            spans.push(FloodSpan {
                y,
                x_left: xl,
                x_right: xr,
            });
            x = xr + 1;
        } else {
            x += 1;
        }
    }
}

// =============================================================
// Inline tests
// =============================================================

// ── Submodules (god-module split, 2026-06-04; pure move) ──
mod color;
pub(crate) use color::{oklab_dist_sq, oklab_to_srgb8, srgb_to_oklab};
#[cfg(test)]
mod tests;
