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
    fill_delta_e_sq(rgba, n, bg_oklab, &mut scratch.delta_e);

    // Fold in every extra user-picked background colour: a pixel's
    // ΔE² becomes the MIN distance to {bg_oklab} ∪ {extra colours}, so
    // being close to ANY reference classifies it as background.
    // Everything downstream (flood / threshold / interior sweep / soft
    // band) then works unchanged. Transparent pixels keep their forced
    // 0.0 (already handled by `fill_delta_e_sq`); we skip them here too.
    for &rgb in extra_colors {
        let extra_oklab = srgb_to_oklab(rgb[0], rgb[1], rgb[2]);
        for (i, d) in scratch.delta_e.iter_mut().enumerate().take(n) {
            let base = i * 4;
            if rgba[base + 3] == 0 {
                continue;
            }
            let p = srgb_to_oklab(rgba[base], rgba[base + 1], rgba[base + 2]);
            *d = d.min(oklab_dist_sq(p, extra_oklab));
        }
    }

    // 3 — decide flood vs threshold-only.
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

    SegmentResult::Chroma { bg_oklab }
}

// =============================================================
// sRGB → Oklab (inline, no external crate)
// =============================================================

/// Convert one sRGB 8-bit triplet to Oklab. Reference:
/// <https://bottosson.github.io/posts/oklab/>.
///
/// Steps: sRGB byte → linear sRGB ([0,1]) → LMS (cube-root-warped)
/// → Oklab (L, a, b) ∈ approx [0,1] × [-0.4, 0.4] × [-0.4, 0.4].
///
/// The matrix constants come straight from the Oklab paper and
/// must NOT be truncated — `clippy::excessive_precision` is
/// silenced here because changing the digits drifts the colour
/// space.
#[allow(clippy::excessive_precision)]
pub(crate) fn srgb_to_oklab(r: u8, g: u8, b: u8) -> [f32; 3] {
    #[inline(always)]
    fn srgb_to_linear(c: u8) -> f32 {
        let cf = (c as f32) / 255.0;
        if cf <= 0.04045 {
            cf / 12.92
        } else {
            ((cf + 0.055) / 1.055).powf(2.4)
        }
    }

    let rl = srgb_to_linear(r);
    let gl = srgb_to_linear(g);
    let bl = srgb_to_linear(b);

    // Linear sRGB → LMS (Oklab paper, Eq. 1).
    let lm = 0.4122214708 * rl + 0.5363325363 * gl + 0.0514459929 * bl;
    let mm = 0.2119034982 * rl + 0.6806995451 * gl + 0.1073969566 * bl;
    let sm = 0.0883024619 * rl + 0.2817188376 * gl + 0.6299787005 * bl;

    let l_ = lm.cbrt();
    let m_ = mm.cbrt();
    let s_ = sm.cbrt();

    // LMS' → Oklab (Eq. 3).
    [
        0.2104542553 * l_ + 0.7936177850 * m_ - 0.0040720468 * s_,
        1.9779984951 * l_ - 2.4285922050 * m_ + 0.4505937099 * s_,
        0.0259040371 * l_ + 0.7827717662 * m_ - 0.8086757660 * s_,
    ]
}

/// Inverse of [`srgb_to_oklab`]: Oklab → sRGB 8-bit. Used by the
/// compose step's despill / foreground-decontamination pass to know
/// the detected background colour in sRGB. Matrices are the published
/// inverses from Björn Ottosson's Oklab reference (must NOT be
/// truncated). Out-of-gamut results are clamped to `[0, 255]`.
#[allow(clippy::excessive_precision)]
pub(crate) fn oklab_to_srgb8(lab: [f32; 3]) -> [u8; 3] {
    let (l, a, b) = (lab[0], lab[1], lab[2]);
    let l_ = l + 0.3963377774 * a + 0.2158037573 * b;
    let m_ = l - 0.1055613458 * a - 0.0638541728 * b;
    let s_ = l - 0.0894841775 * a - 1.2914855480 * b;
    let lc = l_ * l_ * l_;
    let mc = m_ * m_ * m_;
    let sc = s_ * s_ * s_;
    let rl = 4.0767416621 * lc - 3.3077115913 * mc + 0.2309699292 * sc;
    let gl = -1.2684380046 * lc + 2.6097574011 * mc - 0.3413193965 * sc;
    let bl = -0.0041960863 * lc - 0.7034186147 * mc + 1.7076147010 * sc;

    #[inline(always)]
    fn linear_to_srgb8(c: f32) -> u8 {
        let c = c.clamp(0.0, 1.0);
        let s = if c <= 0.0031308 {
            c * 12.92
        } else {
            1.055 * c.powf(1.0 / 2.4) - 0.055
        };
        (s * 255.0 + 0.5).clamp(0.0, 255.0) as u8
    }
    [
        linear_to_srgb8(rl),
        linear_to_srgb8(gl),
        linear_to_srgb8(bl),
    ]
}

/// Squared Euclidean distance between two Oklab points.
#[inline(always)]
pub(crate) fn oklab_dist_sq(a: [f32; 3], b: [f32; 3]) -> f32 {
    let dl = a[0] - b[0];
    let da = a[1] - b[1];
    let db = a[2] - b[2];
    dl * dl + da * da + db * db
}

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
/// Serial — fast enough at 4k (~10 ns/px).
fn fill_delta_e_sq(rgba: &[u8], n: usize, bg_oklab: [f32; 3], out: &mut [f32]) {
    debug_assert!(out.len() >= n);
    // Below this, `oklab_dist_sq` is dominated by float noise from
    // the round-trip; floor to exact 0 so the `<=` test against a
    // user `tol_sq = 0` still treats matches as background.
    const FP_NOISE_FLOOR_SQ: f32 = 1.0e-6;
    for (i, out_i) in out.iter_mut().enumerate().take(n) {
        let base = i * 4;
        if rgba[base + 3] == 0 {
            *out_i = 0.0;
            continue;
        }
        let p = srgb_to_oklab(rgba[base], rgba[base + 1], rgba[base + 2]);
        let d = oklab_dist_sq(p, bg_oklab);
        *out_i = if d < FP_NOISE_FLOOR_SQ { 0.0 } else { d };
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

#[cfg(test)]
mod tests {
    use super::*;

    /// Helper: build an RGBA8 buffer of size `w × h` with a solid
    /// background colour and an optional inner rectangle of a
    /// foreground colour.
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

    fn default_params() -> ChromaParams {
        ChromaParams::default()
    }

    #[test]
    fn srgb_to_oklab_white_is_near_unit_luma() {
        let lab = srgb_to_oklab(255, 255, 255);
        // Oklab L for pure white ≈ 1.0 (paper).
        assert!((lab[0] - 1.0).abs() < 0.01, "white L = {}", lab[0]);
        assert!(lab[1].abs() < 0.01);
        assert!(lab[2].abs() < 0.01);
    }

    #[test]
    fn srgb_to_oklab_black_is_zero() {
        let lab = srgb_to_oklab(0, 0, 0);
        assert!(lab[0].abs() < 1e-3);
        assert!(lab[1].abs() < 1e-3);
        assert!(lab[2].abs() < 1e-3);
    }

    #[test]
    fn srgb_to_oklab_red_has_positive_a() {
        let lab = srgb_to_oklab(255, 0, 0);
        assert!(lab[1] > 0.1, "red a* should be positive, got {}", lab[1]);
    }

    #[test]
    fn solid_bg_with_no_subject_is_entirely_background() {
        let rgba = make_image(32, 32, [0, 200, 0], None);
        let mut s = BgRemovalScratch::default();
        s.ensure(32, 32, false);
        let r = segment(&rgba, 32, 32, &default_params(), &[], &mut s);
        match r {
            SegmentResult::Chroma { bg_oklab } => {
                // bg detected — the centroid should be very close to
                // green oklab.
                let green = srgb_to_oklab(0, 200, 0);
                assert!(oklab_dist_sq(bg_oklab, green) < 0.01);
            }
            _ => panic!("expected Chroma"),
        }
        let mask = &s.mask[..32 * 32];
        let bg_count = mask.iter().filter(|&&v| v == 0).count();
        assert!(
            bg_count > 32 * 32 * 99 / 100,
            "≥99% should be bg, got {bg_count}/{} ",
            32 * 32
        );
    }

    #[test]
    fn subject_on_uniform_bg_is_preserved() {
        // 32×32 green bg, 8×8 red subject in the middle.
        let rgba = make_image(32, 32, [0, 200, 0], Some(([200, 30, 30], 12, 12, 8, 8)));
        let mut s = BgRemovalScratch::default();
        s.ensure(32, 32, false);
        let _ = segment(&rgba, 32, 32, &default_params(), &[], &mut s);

        // Subject pixels (12..20 × 12..20) should be fg.
        for y in 12..20 {
            for x in 12..20 {
                let i = (y as usize) * 32 + x as usize;
                assert_eq!(s.mask[i], 255, "subject pixel ({x},{y}) should be fg");
            }
        }
        // Border pixels should be bg.
        for x in 0..32 {
            let top = x as usize;
            let bot = 31 * 32 + x as usize;
            assert_eq!(s.mask[top], 0, "top row pixel {x} should be bg");
            assert_eq!(s.mask[bot], 0, "bottom row pixel {x} should be bg");
        }
    }

    #[test]
    fn interior_bg_color_pocket_is_removed() {
        // 32×32 green bg, 12×12 red ring with an interior green
        // patch (an enclosed bg-coloured "hole" — like the gap between
        // a character's arm and torso). It must be removed: the border
        // flood can't reach it, but the interior-pocket sweep does
        // (hard bg, ΔE² ≤ tol²). Was previously protected as fg, which
        // left bg colour stuck inside the drawing.
        let mut rgba = make_image(32, 32, [0, 200, 0], Some(([200, 30, 30], 10, 10, 12, 12)));
        // Punch a 2×2 green hole at (15,15) inside the red ring.
        for y in 15..17 {
            for x in 15..17 {
                let i = ((y as usize) * 32 + x as usize) * 4;
                rgba[i] = 0;
                rgba[i + 1] = 200;
                rgba[i + 2] = 0;
            }
        }

        let mut s = BgRemovalScratch::default();
        s.ensure(32, 32, false);
        let _ = segment(&rgba, 32, 32, &default_params(), &[], &mut s);

        // The 2×2 interior green patch is bg-coloured → removed.
        for y in 15..17 {
            for x in 15..17 {
                let i = (y as usize) * 32 + x as usize;
                assert_eq!(
                    s.mask[i], 0,
                    "interior bg-colour pocket ({x},{y}) should be removed"
                );
            }
        }
        // The red ring (subject) stays fg.
        let ring_i = 11 * 32 + 11;
        assert_eq!(s.mask[ring_i], 255, "red ring pixel should stay fg");
        // Exterior green is still bg.
        let edge_i = 0;
        assert_eq!(s.mask[edge_i], 0, "exterior bg pixel should be bg");
    }

    #[test]
    fn reference_color_override_skips_corner_detection() {
        // Make an image where 3 corners are red and 1 is green; if
        // we let corner-auto run, it picks the majority (red). But
        // we override with green, so the green corner should be the
        // one masked.
        let mut rgba = vec![255u8; 32 * 32 * 4]; // all opaque white
        for y in 0..32 {
            for x in 0..32 {
                let i = ((y as usize) * 32 + x as usize) * 4;
                let is_tl = x < 16 && y < 16;
                let (r, g, b) = if is_tl { (0, 200, 0) } else { (200, 0, 0) };
                rgba[i] = r;
                rgba[i + 1] = g;
                rgba[i + 2] = b;
                rgba[i + 3] = 255;
            }
        }
        let mut p = default_params();
        // Override: target the *green* corner as bg.
        p.reference_color = Some([0, 200, 0]);
        // Disable flood so we see a pure threshold result.
        p.use_flood = false;
        let mut s = BgRemovalScratch::default();
        s.ensure(32, 32, false);
        let _ = segment(&rgba, 32, 32, &p, &[], &mut s);

        // TL (green) should be bg; BR (red) should be fg.
        let tl = 0;
        let br = 31 * 32 + 31;
        assert_eq!(s.mask[tl], 0, "TL (green-overridden bg) should be bg");
        assert_eq!(s.mask[br], 255, "BR (red, not bg) should be fg");
    }

    #[test]
    fn delta_e_sq_is_zero_on_bg_and_positive_elsewhere() {
        let rgba = make_image(16, 16, [0, 200, 0], Some(([200, 0, 0], 6, 6, 4, 4)));
        let mut s = BgRemovalScratch::default();
        s.ensure(16, 16, false);
        let _ = segment(&rgba, 16, 16, &default_params(), &[], &mut s);

        // A border pixel should have ΔE² ≈ 0.
        assert!(
            s.delta_e[0] < 0.001,
            "border ΔE² should be ~0, got {}",
            s.delta_e[0]
        );
        // The very centre (subject) should have ΔE² >> 0.
        let centre = 7 * 16 + 7;
        assert!(
            s.delta_e[centre] > 0.05,
            "subject ΔE² should be >0.05, got {}",
            s.delta_e[centre]
        );
    }

    #[test]
    fn border_bg_confidence_drops_when_subject_touches_edge() {
        // 32×32 green bg, but the subject (red) is glued to the
        // entire top row. Border-bg-confidence < 60%, so flood
        // should be auto-disabled and the threshold-only path runs
        // (interior subject still ends up fg). Image must clear
        // MIN_BIG_DIM so corner sampling sees 3 green corners and
        // 1 red corner — k-means picks green as the majority bg.
        let rgba = make_image(64, 64, [0, 200, 0], Some(([200, 30, 30], 0, 0, 64, 24)));

        let mut p = default_params();
        p.tolerance = 0.10;
        let mut s = BgRemovalScratch::default();
        s.ensure(64, 64, false);
        let _ = segment(&rgba, 64, 64, &p, &[], &mut s);

        // Top-left pixel is red → fg.
        assert_eq!(
            s.mask[0], 255,
            "top-left red pixel should be fg under fallback"
        );
        // Bottom-left pixel is green → bg.
        let bl = 63 * 64;
        assert_eq!(
            s.mask[bl], 0,
            "bottom-left green pixel should be bg under fallback"
        );
    }

    #[test]
    fn empty_image_returns_default_bg_no_panic() {
        let mut s = BgRemovalScratch::default();
        s.ensure(0, 0, false);
        let r = segment(&[], 0, 0, &default_params(), &[], &mut s);
        match r {
            SegmentResult::Chroma { bg_oklab } => assert_eq!(bg_oklab, [0.0, 0.0, 0.0]),
            _ => panic!("expected Chroma"),
        }
    }

    #[test]
    fn determinism_same_input_produces_same_mask() {
        let rgba = make_image(48, 48, [10, 180, 30], Some(([220, 40, 40], 18, 18, 12, 12)));

        let mut s1 = BgRemovalScratch::default();
        s1.ensure(48, 48, false);
        let r1 = segment(&rgba, 48, 48, &default_params(), &[], &mut s1);

        let mut s2 = BgRemovalScratch::default();
        s2.ensure(48, 48, false);
        let r2 = segment(&rgba, 48, 48, &default_params(), &[], &mut s2);

        assert_eq!(s1.mask, s2.mask);
        match (r1, r2) {
            (SegmentResult::Chroma { bg_oklab: a }, SegmentResult::Chroma { bg_oklab: b }) => {
                assert_eq!(a, b);
            }
            _ => panic!("expected Chroma in both"),
        }
    }

    #[test]
    fn scratch_is_reusable_across_calls() {
        let rgba_a = make_image(24, 24, [200, 0, 0], None);
        let rgba_b = make_image(24, 24, [0, 0, 200], None);

        let mut s = BgRemovalScratch::default();
        s.ensure(24, 24, false);

        let _ = segment(&rgba_a, 24, 24, &default_params(), &[], &mut s);
        let bg_count_a = s.mask.iter().filter(|&&v| v == 0).count();

        // No re-ensure between calls — reuse same allocation.
        let _ = segment(&rgba_b, 24, 24, &default_params(), &[], &mut s);
        let bg_count_b = s.mask.iter().filter(|&&v| v == 0).count();

        // Both runs should produce mostly-bg masks (different bg
        // colours but each input is uniform).
        assert!(bg_count_a > 24 * 24 * 99 / 100);
        assert!(bg_count_b > 24 * 24 * 99 / 100);
    }

    // --- Audit-driven coverage gap tests ---------------------------------

    #[test]
    fn transparent_border_does_not_poison_corner_bg_detection() {
        // 64×64 light-grey opaque bg with 4-px transparent corners
        // (alpha-rounded sprite) and a 16×16 opaque red subject in
        // the middle. Without the alpha-aware skip, the transparent
        // corner pixels' garbage RGB would poison k-means; with it,
        // k-means samples the actual light-grey bg.
        let dim = 64usize;
        let mut rgba = vec![0u8; dim * dim * 4];
        for i in 0..(dim * dim) {
            let b = i * 4;
            rgba[b] = 200;
            rgba[b + 1] = 200;
            rgba[b + 2] = 200;
            rgba[b + 3] = 255;
        }
        // Punch 4×4 transparent corners.
        for cy in 0..4 {
            for cx in 0..4 {
                for &(x, y) in &[
                    (cx, cy),
                    (dim - 1 - cx, cy),
                    (cx, dim - 1 - cy),
                    (dim - 1 - cx, dim - 1 - cy),
                ] {
                    rgba[(y * dim + x) * 4 + 3] = 0;
                }
            }
        }
        // 16×16 red subject in the middle.
        for y in 24..40 {
            for x in 24..40 {
                let b = (y * dim + x) * 4;
                rgba[b] = 220;
                rgba[b + 1] = 30;
                rgba[b + 2] = 30;
                rgba[b + 3] = 255;
            }
        }
        let mut s = BgRemovalScratch::default();
        s.ensure(dim as u32, dim as u32, false);
        let _ = segment(
            &rgba,
            dim as u32,
            dim as u32,
            &default_params(),
            &[],
            &mut s,
        );

        // Centre red subject must be fg.
        let centre = 32 * dim + 32;
        assert_eq!(s.mask[centre], 255, "opaque red centre must be fg");
        // Off-centre grey bg pixel must be bg.
        let bg_pixel = 10 * dim + 30;
        assert_eq!(s.mask[bg_pixel], 0, "light-grey bg pixel must be bg");
        // Transparent corner pixel must be bg (alpha-driven via
        // delta_e=0 forced for `a == 0`).
        assert_eq!(s.mask[0], 0, "transparent corner must be bg");
    }

    #[test]
    fn nan_tolerance_does_not_make_everything_foreground() {
        // Without the NaN sanitisation, `delta_e <= NaN` is always
        // false and every pixel would be marked fg silently. Image
        // size must clear MIN_BIG_DIM so corner sampling sees the
        // bg colour.
        let rgba = make_image(64, 64, [0, 200, 0], None);
        let mut p = default_params();
        p.tolerance = f32::NAN;
        let mut s = BgRemovalScratch::default();
        s.ensure(64, 64, false);
        let _ = segment(&rgba, 64, 64, &p, &[], &mut s);

        // Sanitised → tol_sq = 0 + FP_NOISE_FLOOR → uniform bg
        // (exact match within FP noise) all classify as bg.
        let bg = s.mask.iter().filter(|&&v| v == 0).count();
        assert_eq!(
            bg,
            64 * 64,
            "NaN tolerance must collapse to 0, classifying uniform bg as bg"
        );
    }

    #[test]
    fn tolerance_zero_only_marks_exact_match_as_bg() {
        // Pure red 64×64 with a single off-by-one pixel (R=254 vs
        // 255). With tolerance=0 the off-by-one pixel falls outside
        // the threshold → fg; the corner pixel matches exactly
        // within FP noise → bg. Image must clear MIN_BIG_DIM.
        let mut rgba = make_image(64, 64, [255, 0, 0], None);
        let probe = (5 * 64 + 5) * 4;
        rgba[probe] = 254; // R 255→254
        let mut p = default_params();
        p.tolerance = 0.0;
        // Disable flood so the probe pixel is classified on its
        // own colour, not via flood-propagation from neighbours.
        p.use_flood = false;
        let mut s = BgRemovalScratch::default();
        s.ensure(64, 64, false);
        let _ = segment(&rgba, 64, 64, &p, &[], &mut s);

        assert_eq!(
            s.mask[5 * 64 + 5],
            255,
            "off-by-one pixel must be fg at tol=0"
        );
        assert_eq!(s.mask[0], 0, "exact-match corner pixel must be bg at tol=0");
    }

    #[test]
    fn very_tall_image_falls_back_to_small_path_without_panic() {
        // 4 × 200 image — width below MIN_BIG_DIM. The sample path
        // must take the small-image fallback (no panic, sensible
        // mask).
        let rgba = make_image(4, 200, [50, 150, 250], None);
        let mut s = BgRemovalScratch::default();
        s.ensure(4, 200, false);
        let _ = segment(&rgba, 4, 200, &default_params(), &[], &mut s);
        // Uniform input → entire mask should be bg.
        let bg = s.mask.iter().filter(|&&v| v == 0).count();
        assert!(
            bg > 4 * 200 * 95 / 100,
            "≥95% should be bg, got {bg}/{}",
            4 * 200
        );
    }

    #[test]
    fn min_big_dim_boundary_does_not_overflow_or_overlap() {
        // Exactly MIN_BIG_DIM (52×52) is the first size that takes
        // the big-image dispersion path. The top-left and top-right
        // patches must sample disjoint columns by construction.
        // Functional check: a uniform image yields a fully-bg mask
        // and no panic.
        let dim = MIN_BIG_DIM;
        let rgba = make_image(dim, dim, [100, 100, 100], None);
        let mut s = BgRemovalScratch::default();
        s.ensure(dim, dim, false);
        let _ = segment(&rgba, dim, dim, &default_params(), &[], &mut s);
        let n = (dim * dim) as usize;
        let bg = s.mask.iter().filter(|&&v| v == 0).count();
        assert_eq!(bg, n, "uniform MIN_BIG_DIM image must be entirely bg");

        // One below — falls into small-image path.
        let small = dim - 1;
        let rgba2 = make_image(small, small, [100, 100, 100], None);
        let mut s2 = BgRemovalScratch::default();
        s2.ensure(small, small, false);
        let _ = segment(&rgba2, small, small, &default_params(), &[], &mut s2);
        let n2 = (small * small) as usize;
        let bg2 = s2.mask.iter().filter(|&&v| v == 0).count();
        assert_eq!(bg2, n2, "uniform small-fallback image must be entirely bg");
    }

    #[test]
    fn extra_color_masks_a_second_background_region() {
        // 64×64 image: green bg (auto-detected from the corners) with a
        // BLUE 16×16 block in the centre. Blue is far from green, so
        // with auto-only the blue block stays foreground. Adding blue as
        // an extra background colour must knock it out — without
        // disturbing the green-bg classification. Disable flood so each
        // pixel is judged on its own colour (the blue block is interior,
        // not border-connected).
        let blue = [30u8, 30, 220];
        let rgba = make_image(64, 64, [0, 200, 0], Some((blue, 24, 24, 16, 16)));
        let mut p = default_params();
        p.use_flood = false;
        let centre = 32 * 64 + 32;

        // Auto-only: blue centre is foreground.
        let mut s = BgRemovalScratch::default();
        s.ensure(64, 64, false);
        let _ = segment(&rgba, 64, 64, &p, &[], &mut s);
        assert_eq!(
            s.mask[centre], 255,
            "without the extra colour the blue block must stay foreground"
        );

        // With blue as an extra bg colour: the blue centre is removed.
        let mut s2 = BgRemovalScratch::default();
        s2.ensure(64, 64, false);
        let _ = segment(&rgba, 64, 64, &p, &[blue], &mut s2);
        assert_eq!(
            s2.mask[centre], 0,
            "the extra blue colour must mask the blue block as background"
        );
        // The green border is still background.
        assert_eq!(s2.mask[0], 0, "green border must remain background");
    }
}
