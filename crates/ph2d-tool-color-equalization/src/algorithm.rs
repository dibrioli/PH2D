//! Color Equalization — pure CPU pipeline.
//!
//! `std`-only, no editor / ECS / external image deps. Operates on
//! straight-alpha RGBA8 (`w*h*4` bytes, row-major) and produces a fresh
//! RGBA8 buffer of the same dimensions. Stages, in pipeline order:
//!
//! 1. [`clahe`] — Contrast-Limited Adaptive Histogram Equalization on the
//!    BT.709 luminance channel (Zuiderveld 1994, *Graphics Gems IV*
//!    pp. 474-485).
//! 2. [`adjust_tonal`] — combined Phase 1 tonal pipeline batched in a
//!    single sRGB ↔ linear ↔ OKLab round-trip per pixel. Stages in order:
//!    Exposure → Temperature (Bradford) → Tint → Brightness → Contrast →
//!    Vibrance (OKLab) → Saturation (OKLab). Each stage is also exposed
//!    as a pure primitive (`apply_*_linear` / `apply_*_oklab`) for
//!    standalone tests and future WGSL parity.
//!    Phase 3 LUT color grading then runs in-line (procedural presets
//!    via [`crate::lut_presets`] → [`crate::lut::apply_lut3d`]; dual-slot
//!    blend by `lut_mix` + intensity attenuation; skipped when both
//!    slots are `None`).
//! 3. [`sharpen_laplacian`] (radius ≤ 1) or [`sharpen_unsharp`] (radius
//!    > 1) — Phase 2 detail enhancement. Denoise stage was evaluated
//!    (Bilateral, NLM, Guided Filter, À-Trous, Domain Transform,
//!    Anisotropic Diffusion, TV-Chambolle, Wavelet Shrinkage) and
//!    removed 2026-05-27 — none met the visual bar.
//! 4. [`auto_levels`] / [`auto_contrast`] / [`auto_colors`] — optional
//!    post-tonal normalization toggles (Phase 2).
//! 5. [`auto_white_balance`] — Gray-World channel gains in sRGB.
//!
//! [`compute_histogram`] returns the per-channel + luma distribution and
//! powers both the panel's visual overlay and the auto-* percentile
//! analysis. [`run_pipeline`] threads everything together; each stage is
//! also usable standalone for tests / future GPU parity work.
//!
//! ## GPU port plan (follow-up)
//!
//! Every per-pixel stage in this module is embarrassingly parallel. A
//! single WGSL compute pass can fuse the Phase 1 tonal pipeline (the
//! seven stages + OKLab smart-sat pair) plus the CLAHE LUT apply step
//! plus auto-WB into one shader, doing exactly one sRGB → linear and
//! one OKLab round-trip per pixel. The legacy engine demonstrates the pattern in 799 LOC of
//! WebGL2 (`ceq-webgl.ts`). Histogram + Bradford-matrix precompute stay
//! CPU (atomic contention vs. sequential setup); the per-pixel apply is
//! GPU. Parity test (ε = 0.5 / 255) compares this CPU path against the
//! shader output on the same input.

use crate::color_utils::{
    bradford_matrix_for_kelvin, linear_rgb_to_oklab, linear_to_srgb_u8, mat3_mul_vec,
    oklab_to_linear_rgb, srgb_to_linear_u8,
};
use crate::lut::{DEFAULT_LUT_SIZE, apply_lut3d, blend_luts};
use crate::lut_presets::generate_preset_lut;
use crate::params::ColorEqualizationParams;

/// Number of bins in the per-tile luminance histogram. 8-bit luminance
/// means a one-to-one mapping; the LUT step turns this into a per-tile
/// 256-byte table.
const HISTOGRAM_BINS: usize = 256;

/// Run the full Color Equalization pipeline against `rgba` (straight-alpha
/// RGBA8, `w * h * 4` bytes) with `params`, writing into `out` (resized
/// to match). Stages run in order: CLAHE → brightness/contrast/saturation
/// → optional auto-WB.
///
/// The output buffer is reused across runs (HR-3) — the caller owns it.
/// Caller may pass an empty `Vec<u8>` on first call; subsequent calls
/// reuse the allocation.
pub fn run_pipeline(
    pixels: &[ph2d_color::SrgbRgba],
    w: u32,
    h: u32,
    params: &ColorEqualizationParams,
    out: &mut Vec<u8>,
) {
    let rgba: &[u8] = bytemuck::cast_slice(pixels);
    let expected = (w as usize) * (h as usize) * 4;
    assert_eq!(rgba.len(), expected, "rgba length must match w*h*4");
    out.clear();
    out.resize(expected, 0);
    if w == 0 || h == 0 {
        return;
    }

    // Fast-path: identity params → copy source through, skip every
    // stage. Mirrors the GPU chain's zero-dispatch shortcut
    // (`chain_identity_params_short_circuits_to_no_dispatches`).
    if params.is_noop() {
        out.copy_from_slice(rgba);
        return;
    }

    // Stage 1 — CLAHE (writes through into `out`). Skipped at the
    // identity clip limit so the per-tile CDF reconstruction can't
    // tint the image when CLAHE is effectively off.
    if params.clip_limit > crate::params::CLIP_LIMIT_MIN {
        clahe(rgba, w, h, params.clip_limit, params.tile_grid_size, out);
    } else {
        out.copy_from_slice(rgba);
    }

    // Stage 2 — combined Phase 1 tonal pipeline in a single sRGB↔linear
    // (and optional OKLab) round-trip per pixel. Skipped when ALL params
    // are at identity to keep the no-op cheap.
    if !params.tonal_is_identity() {
        adjust_tonal(out, params);
    }

    // Stage 2.5 — Phase 3 LUT color grading. Procedural presets are
    // materialised here on-demand (≈ 5-15 ms per active preset at the
    // default 17³ size; bypassed entirely when both slots are `None`
    // or `lut_intensity` is `0`). Dual-LUT case pre-blends the two
    // LUTs by `lut_mix` so the per-pixel apply pass only samples one
    // cube. A wgpu compute follow-up replaces this CPU loop with one
    // `textureSampleLevel(lut3d, ...)` per pixel.
    if !params.lut_is_identity() {
        let lut1 = generate_preset_lut(params.lut_preset_1, DEFAULT_LUT_SIZE);
        let lut2 = generate_preset_lut(params.lut_preset_2, DEFAULT_LUT_SIZE);
        match (lut1, lut2) {
            (Some(a), Some(b)) => {
                let blended = blend_luts(&a, &b, params.lut_mix);
                apply_lut3d(out, &blended, params.lut_intensity);
            }
            (Some(a), None) => apply_lut3d(out, &a, params.lut_intensity),
            (None, Some(b)) => apply_lut3d(out, &b, params.lut_intensity),
            (None, None) => {}
        }
    }

    // Stage 3 — Phase 2 sharpen. Small radius (≤ 1) takes the fast
    // Laplacian 3×3; larger radius takes Unsharp Mask (Gaussian blur).
    if params.sharpen_amount > 0.0 {
        if params.sharpen_radius <= 1.0 {
            sharpen_laplacian(out, w, h, params.sharpen_amount);
        } else {
            sharpen_unsharp(out, w, h, params.sharpen_amount, params.sharpen_radius);
        }
    }

    // Stage 4 — Phase 2 optional automatic adjustments. Each is a toggle
    // applied AFTER tonal so it normalises the user's adjustments rather
    // than fighting them.
    if params.auto_levels {
        auto_levels(out);
    }
    if params.auto_contrast {
        auto_contrast(out);
    }
    if params.auto_colors {
        auto_colors(out);
    }

    // Stage 5 — Gray-World auto white balance (also in place over `out`).
    if params.auto_wb {
        auto_white_balance(out);
    }

    // Stage 6 — Posterize (Floyd-Steinberg dithering optional). Always
    // CPU — the error-diffusion sweep is strict raster-scan. Runs after
    // all colour-shift stages so it operates on the final palette.
    if params.posterize_levels >= POSTERIZE_LEVELS_MIN {
        posterize(
            out,
            w,
            h,
            params.posterize_levels,
            params.posterize_dithering,
            params.posterize_dither_strength,
            params.posterize_dither_grain,
        );
    }

    // Stage 7 — Quantize (K-Means++ in OKLab). Always CPU. Runs LAST —
    // every prior stage feeds into the colour set that gets clustered.
    if params.quantize_colors >= QUANTIZE_COLORS_MIN {
        quantize(out, w, h, params.quantize_colors);
    }
}

// ── Stage 1 — CLAHE ───────────────────────────────────────────────────────

/// Apply Contrast-Limited Adaptive Histogram Equalization to the luminance
/// of a straight-alpha RGBA8 image.
///
/// `clip_limit` is the Zuiderveld redistribution ceiling in units of "mean
/// bin count" (`1.0` = uniform histogram → no contrast boost; `2.0` = the
/// canonical default; values above `4` tend to amplify noise harshly).
///
/// `tile_grid_size` is the number of tiles per side (`N×N` partition). The
/// per-tile histograms are 256-bin; per-pixel the LUTs of the four
/// surrounding tile centres are bilinearly interpolated. Boundary pixels
/// fall back to fewer tiles (corner = 1, edge = 2, interior = 4) by
/// clamping the tile-centre indices.
///
/// Hue / chroma preservation runs in **BT.709 YCbCr (full range, sRGB
/// gamma-encoded)** — mirrors OpenCV `cvtColor BGR2YCrCb + equalize Y +
/// merge` / Krita's CLAHE pipeline. The histogram + LUT are built over
/// `Y` only; `Cb` and `Cr` (cached at original-pixel precision) are
/// reattached after the equalized `Y` is interpolated. The legacy
/// `scale = new_L / max(old_L, 1)` reconstruction shifted hue in dark
/// pixels (`l_in ∈ {1, 2}` → scale > 50× → one channel saturating
/// before the others) and bled tile-CDF differences into chroma in soft
/// areas — the YCbCr split removes both. Alpha passes through unchanged.
pub fn clahe(src: &[u8], w: u32, h: u32, clip_limit: f32, tile_grid_size: u32, dst: &mut [u8]) {
    let n_px = (w as usize) * (h as usize);
    assert_eq!(src.len(), n_px * 4);
    assert_eq!(dst.len(), n_px * 4);
    if w == 0 || h == 0 {
        return;
    }

    // Degenerate tile grid → fall through (preserves source). The panel
    // clamps to ≥4 already, but a tool consumer might bypass that.
    let n_tiles = tile_grid_size.max(1);

    // 1. Y / Cb / Cr buffers (BT.709 full-range, sRGB gamma).
    //    Y as u8 because the histogram + LUT live in [0, 255]; Cb / Cr
    //    as f32 to retain sub-LSB chroma precision through reattach.
    let mut luma = vec![0u8; n_px];
    let mut cb = vec![0.0_f32; n_px];
    let mut cr = vec![0.0_f32; n_px];
    for (i, px) in src.chunks_exact(4).enumerate() {
        let r = px[0] as f32;
        let g = px[1] as f32;
        let b = px[2] as f32;
        let y = 0.2126 * r + 0.7152 * g + 0.0722 * b;
        luma[i] = clamp8(y);
        cb[i] = (b - y) / 1.8556;
        cr[i] = (r - y) / 1.5748;
    }

    // 2. Per-tile LUT table: `n_tiles * n_tiles` LUTs of 256 bytes each.
    //    Index `(ty, tx)` → `lut[ty * n_tiles + tx]`.
    let mut luts: Vec<[u8; HISTOGRAM_BINS]> = Vec::with_capacity((n_tiles * n_tiles) as usize);
    let mut histogram = [0u32; HISTOGRAM_BINS];
    for ty in 0..n_tiles {
        for tx in 0..n_tiles {
            let (x0, x1) = tile_span(tx, n_tiles, w);
            let (y0, y1) = tile_span(ty, n_tiles, h);
            histogram.fill(0);
            for y in y0..y1 {
                let row = (y as usize) * (w as usize);
                for x in x0..x1 {
                    histogram[luma[row + x as usize] as usize] += 1;
                }
            }
            let tile_area = (x1 - x0) * (y1 - y0);
            luts.push(build_clahe_lut(&histogram, tile_area, clip_limit));
        }
    }

    // 3. Per-pixel bilinear interp of the 4 nearest tile centres.
    //    Tile centre `(tx, ty)` sits at `((tx+0.5)*tile_w, (ty+0.5)*tile_h)`.
    let tile_w = (w as f32) / (n_tiles as f32);
    let tile_h = (h as f32) / (n_tiles as f32);
    for y in 0..h {
        let fy = (y as f32 + 0.5) / tile_h - 0.5;
        let ty_lo = fy.floor() as i32;
        let ty_hi = ty_lo + 1;
        let wy = fy - ty_lo as f32; // 0..1 weight toward `ty_hi`.
        let ty_lo_c = ty_lo.clamp(0, n_tiles as i32 - 1) as u32;
        let ty_hi_c = ty_hi.clamp(0, n_tiles as i32 - 1) as u32;
        for x in 0..w {
            let fx = (x as f32 + 0.5) / tile_w - 0.5;
            let tx_lo = fx.floor() as i32;
            let tx_hi = tx_lo + 1;
            let wx = fx - tx_lo as f32;
            let tx_lo_c = tx_lo.clamp(0, n_tiles as i32 - 1) as u32;
            let tx_hi_c = tx_hi.clamp(0, n_tiles as i32 - 1) as u32;

            let idx = (y as usize) * (w as usize) + x as usize;
            let l_in = luma[idx] as usize;
            let l00 = luts[(ty_lo_c * n_tiles + tx_lo_c) as usize][l_in] as f32;
            let l10 = luts[(ty_lo_c * n_tiles + tx_hi_c) as usize][l_in] as f32;
            let l01 = luts[(ty_hi_c * n_tiles + tx_lo_c) as usize][l_in] as f32;
            let l11 = luts[(ty_hi_c * n_tiles + tx_hi_c) as usize][l_in] as f32;
            let top = l00 + wx * (l10 - l00);
            let bot = l01 + wx * (l11 - l01);
            let new_y = top + wy * (bot - top);

            // BT.709 full-range inverse: reattach the original Cb / Cr.
            // Tile-CDF differences live only in `new_y`; hue stays put.
            let cb_v = cb[idx];
            let cr_v = cr[idx];
            let r_out = new_y + 1.5748 * cr_v;
            let g_out = new_y - 0.187_324 * cb_v - 0.468_124 * cr_v;
            let b_out = new_y + 1.8556 * cb_v;

            let src_off = idx * 4;
            dst[src_off] = clamp8(r_out);
            dst[src_off + 1] = clamp8(g_out);
            dst[src_off + 2] = clamp8(b_out);
            dst[src_off + 3] = src[src_off + 3];
        }
    }
}

/// `[start, end)` tile span across `total` pixels for `idx`-th tile of
/// `n_tiles` (integer split with the trailing tile absorbing the remainder
/// so total coverage stays exact).
fn tile_span(idx: u32, n_tiles: u32, total: u32) -> (u32, u32) {
    let start = idx * total / n_tiles;
    let end = if idx + 1 == n_tiles {
        total
    } else {
        (idx + 1) * total / n_tiles
    };
    (start, end)
}

/// Per-tile CLAHE LUT (Zuiderveld redistribution + CDF normalization).
///
/// 1. `clip_count = round(clip_limit * tile_area / 256)`.
/// 2. Truncate each bin at `clip_count`; sum the total excess.
/// 3. Redistribute `excess / 256` uniformly to every bin; any residual
///    excess is doled out one-per-bin cyclically until exhausted (the
///    Zuiderveld "even-out the leftovers" step, mandatory when
///    `excess % 256 != 0`).
/// 4. CDF + normalize so `lut[255] = 255`.
fn build_clahe_lut(
    histogram: &[u32; HISTOGRAM_BINS],
    tile_area: u32,
    clip_limit: f32,
) -> [u8; HISTOGRAM_BINS] {
    let mean = (tile_area as f32 / HISTOGRAM_BINS as f32).max(1.0);
    let clip = (clip_limit.max(1.0) * mean).round() as u32;
    let mut clipped = *histogram;
    let mut excess: u32 = 0;
    for bin in clipped.iter_mut() {
        if *bin > clip {
            excess += *bin - clip;
            *bin = clip;
        }
    }
    // Redistribute. Split excess into the uniform per-bin share + the
    // leftover (Zuiderveld dual loop). The leftover is sprayed in order so
    // the LUT stays deterministic (HR-5).
    let per_bin = excess / HISTOGRAM_BINS as u32;
    let mut residual = excess - per_bin * HISTOGRAM_BINS as u32;
    for bin in clipped.iter_mut() {
        *bin += per_bin;
    }
    if residual > 0 {
        // Spray the leftover one count at a time, walking the bins.
        let mut i = 0;
        while residual > 0 {
            clipped[i] += 1;
            i = (i + 1) % HISTOGRAM_BINS;
            residual -= 1;
        }
    }

    // 4. Build the CDF and normalize to `[0, 255]`. `tile_area` is the
    //    total post-redistribution count (the bumps net to zero since we
    //    only moved counts within the histogram).
    let mut lut = [0u8; HISTOGRAM_BINS];
    if tile_area == 0 {
        // Identity LUT for an empty tile (shouldn't happen with the
        // `max(1)` guard on `n_tiles`, but kept for robustness).
        for (i, v) in lut.iter_mut().enumerate() {
            *v = i as u8;
        }
        return lut;
    }
    let scale = 255.0 / tile_area as f32;
    let mut acc: u32 = 0;
    for (i, count) in clipped.iter().enumerate() {
        acc += count;
        lut[i] = clamp8(acc as f32 * scale);
    }
    lut
}

// ── Stage 2 — tonal pipeline (Phase 1) ────────────────────────────────────
//
// Each stage primitive operates on a **linear-sRGB triple `[R, G, B]`** (or
// **OKLab triple `[L, a, b]`** for the Vibrance / Saturation pair). They are
// `pub` so tests + future WGSL parity work can call them in isolation; the
// production path is [`adjust_tonal`], which fuses them inside one
// `sRGB → linear → … → linear → sRGB` round-trip per pixel.

/// Apply exposure (EV stops) in linear-light sRGB. `m = pow(2, ev)` followed
/// by a soft-knee highlight compression above `0.8` that gradually rolls
/// values off toward `1.0` instead of hard-clipping. `ev` is the EV stop
/// count (`-3..+3`); `0` is identity.
pub fn apply_exposure_linear(rgb: &mut [f32; 3], ev: f32) {
    if ev == 0.0 {
        return;
    }
    let m = (2.0_f32).powf(ev);
    for c in rgb.iter_mut() {
        let v = *c * m;
        *c = soft_knee(v);
    }
}

/// Soft-knee compression above `0.8`: linear identity below the knee, then
/// `0.8 + 0.2 · (1 − exp(-(v - 0.8) · 2))` above. Asymptotic to `1.0` —
/// prevents harsh hard-clip on bright values. Mirrors the legacy `softKnee`.
fn soft_knee(v: f32) -> f32 {
    if v <= 0.8 {
        v
    } else {
        0.8 + 0.2 * (1.0 - (-(v - 0.8) * 2.0).exp())
    }
}

/// Apply pre-computed Bradford temperature matrix in linear sRGB. Build the
/// matrix once outside the per-pixel loop via
/// [`crate::color_utils::bradford_matrix_for_kelvin`] using
/// [`temperature01_to_kelvin`] to project a `-1..+1` slider value onto the
/// `2000K..10000K` target range (photographer convention: positive = warm).
pub fn apply_temperature_linear(rgb: &mut [f32; 3], matrix: &[f32; 9]) {
    let out = mat3_mul_vec(matrix, *rgb);
    *rgb = out;
}

/// Map the `-1..+1` slider value onto a target Kelvin for the Bradford
/// adaptation. Photographer convention: positive = warm (low Kelvin / orange
/// cast), negative = cool (high Kelvin / blue cast); `0` = D65 neutral.
pub fn temperature01_to_kelvin(t: f32) -> f32 {
    let t = t.clamp(-1.0, 1.0);
    if t >= 0.0 {
        // 0 → 6500K (D65); +1 → 2000K (tungsten, warm).
        6500.0 - (6500.0 - 2000.0) * t
    } else {
        // 0 → 6500K; -1 → 10000K (overcast / blue).
        6500.0 + (10000.0 - 6500.0) * (-t)
    }
}

/// Apply tint (green ↔ magenta) in linear-light sRGB. `tint ∈ [-1, +1]`:
/// positive shifts toward magenta (drops G, lifts R/B in luminance-preserving
/// proportions); negative shifts toward green. Mirrors the legacy `applyTint`.
pub fn apply_tint_linear(rgb: &mut [f32; 3], tint: f32) {
    if tint == 0.0 {
        return;
    }
    let t = tint.clamp(-1.0, 1.0);
    // Green shifts; R/B counter-shift weighted by their luminance
    // contribution so the overall Y stays roughly constant (BT.709).
    let g_shift = -t * 0.05;
    let r_comp = t * 0.05 * 0.7152 / 0.2126;
    let b_comp = t * 0.05 * 0.7152 / 0.0722;
    rgb[0] *= 1.0 + r_comp;
    rgb[1] *= 1.0 + g_shift;
    rgb[2] *= 1.0 + b_comp;
}

/// Apply brightness in linear-light sRGB. `brightness ∈ [-1, +1]` — applied
/// multiplicatively as `m = 1 + brightness`, so `0` is identity, `-1`
/// collapses to black, `+1` doubles. Multiplicative (not additive) preserves
/// blacks: a pure-black pixel stays black instead of being lifted to grey.
/// Mirrors the legacy `applyBrightness`.
pub fn apply_brightness_linear(rgb: &mut [f32; 3], brightness: f32) {
    if brightness == 0.0 {
        return;
    }
    let m = 1.0 + brightness.clamp(-1.0, 1.0);
    for c in rgb.iter_mut() {
        *c *= m;
    }
}

/// Apply contrast in linear-light sRGB with an S-curve around the
/// perceptual midpoint (`0.18`, "18 % grey"). `contrast ∈ [0.5, 2.0]`,
/// `1.0` is identity. Above `1.0` steepens midtones (S-curve); below `1.0`
/// flattens them. Mirrors the legacy `applyContrast` (more nuanced than a
/// simple multiply around 0.5 — preserves shadows).
pub fn apply_contrast_linear(rgb: &mut [f32; 3], contrast: f32) {
    if (contrast - 1.0).abs() < f32::EPSILON {
        return;
    }
    let strength = (contrast.clamp(0.5, 2.0) - 1.0) * 2.0;
    let pivot = 0.18;
    for c in rgb.iter_mut() {
        let centered = *c - pivot;
        let sign = if centered >= 0.0 { 1.0 } else { -1.0 };
        let abs = centered.abs();
        let curved = if contrast > 1.0 {
            abs * (1.0 + strength * (1.0 - abs))
        } else {
            abs * (1.0 + strength * abs)
        };
        *c = (pivot + sign * curved).clamp(0.0, 1.0);
    }
}

/// Apply vibrance (smart saturation) in OKLab. `vibrance ∈ [-1, +1]`:
/// boosts chroma INVERSELY proportional to current chroma, so already-vivid
/// regions (skin tones, sky) get less boost than muted regions. The
/// chroma-norm threshold `0.15` matches the legacy reference. Mirrors
/// `applyVibrance` (without explicit `cos(hue)` / `sin(hue)` — chroma
/// scaling preserves hue trivially when both `a` and `b` are scaled).
pub fn apply_vibrance_oklab(lab: &mut [f32; 3], vibrance: f32) {
    if vibrance == 0.0 {
        return;
    }
    let vn = vibrance.clamp(-1.0, 1.0);
    let chroma = (lab[1] * lab[1] + lab[2] * lab[2]).sqrt();
    if chroma <= 0.0 {
        return;
    }
    // Skin-tone protection: less boost when chroma is already high.
    let chroma_norm = (chroma / 0.15).min(1.0);
    let boost = vn * (1.0 - chroma_norm * chroma_norm);
    let factor = (1.0 + boost).max(0.0);
    lab[1] *= factor;
    lab[2] *= factor;
}

/// Apply uniform saturation in OKLab. `saturation ∈ [-1, +1]`, `0` is
/// identity; `-1` desaturates fully (grayscale), `+1` doubles chroma.
/// Mirrors the legacy `applySaturation` in OKLab. Scales `a` and `b`
/// directly (= scaling chroma while keeping hue, since `a + ib = chroma · e^(iθ)`).
pub fn apply_saturation_oklab(lab: &mut [f32; 3], saturation: f32) {
    if saturation == 0.0 {
        return;
    }
    let sat_mult = (1.0 + saturation.clamp(-1.0, 1.0)).max(0.0);
    lab[1] *= sat_mult;
    lab[2] *= sat_mult;
}

/// Apply the full Phase 1 tonal pipeline in place over straight-alpha
/// RGBA8. Performs ONE sRGB → linear and (when Vibrance/Saturation
/// non-identity) ONE OKLab round-trip per pixel — instead of cascading
/// each stage with its own conversion. Transparent pixels (`alpha == 0`)
/// are skipped (RGB undefined per straight-alpha convention).
///
/// Order matches the legacy reference: Exposure → Temperature → Tint →
/// Brightness → Contrast → Vibrance → Saturation. Stage primitives are
/// also exposed `pub` for standalone tests.
pub fn adjust_tonal(rgba: &mut [u8], params: &ColorEqualizationParams) {
    // Precompute the Bradford temperature matrix once outside the per-pixel
    // loop — it depends only on the target Kelvin.
    let temp_matrix: Option<[f32; 9]> = if params.temperature != 0.0 {
        Some(bradford_matrix_for_kelvin(temperature01_to_kelvin(
            params.temperature,
        )))
    } else {
        None
    };
    let needs_oklab = params.vibrance != 0.0 || params.saturation != 0.0;

    for px in rgba.chunks_exact_mut(4) {
        if px[3] == 0 {
            continue;
        }
        let mut rgb = [
            srgb_to_linear_u8(px[0]),
            srgb_to_linear_u8(px[1]),
            srgb_to_linear_u8(px[2]),
        ];

        // Linear-sRGB stages.
        apply_exposure_linear(&mut rgb, params.exposure);
        if let Some(ref m) = temp_matrix {
            apply_temperature_linear(&mut rgb, m);
        }
        apply_tint_linear(&mut rgb, params.tint);
        apply_brightness_linear(&mut rgb, params.brightness);
        apply_contrast_linear(&mut rgb, params.contrast);

        // OKLab stages — single conversion for the pair.
        if needs_oklab {
            let mut lab = linear_rgb_to_oklab(rgb[0], rgb[1], rgb[2]);
            apply_vibrance_oklab(&mut lab, params.vibrance);
            apply_saturation_oklab(&mut lab, params.saturation);
            rgb = oklab_to_linear_rgb(lab[0], lab[1], lab[2]);
        }

        px[0] = linear_to_srgb_u8(rgb[0]);
        px[1] = linear_to_srgb_u8(rgb[1]);
        px[2] = linear_to_srgb_u8(rgb[2]);
    }
}

// ── Stage 3 — auto-WB (Gray-World) ────────────────────────────────────────

/// Gray-World auto white balance applied in place over straight-alpha
/// RGBA8 — runs in **linear sRGB**, not gamma-encoded.
///
/// Why linear: averaging luminance is a physical operation (mean of
/// light), and sRGB is a perceptual encoding that compresses shadows.
/// Averaging gamma-encoded values pulls the mean toward the dark end and
/// biases the gains — visible as drifted WB in high-contrast scenes
/// (sun + shadow). Decoding to linear before averaging restores the
/// photon-space invariant gray-world depends on.
///
/// Averages linear R / G / B independently over opaque pixels (`alpha >
/// 0`), then computes `gain = mean_gray / mean_channel` per channel and
/// rescales every pixel in linear space before re-encoding sRGB.
/// Transparent pixels are skipped (no contribution to the mean, no
/// rescale).
///
/// Falls back to a no-op when there are no opaque pixels or any channel
/// mean is zero (a fully black or single-channel image — no information to
/// balance against).
pub fn auto_white_balance(rgba: &mut [u8]) {
    use crate::color_utils::{linear_to_srgb_u8, srgb_to_linear_u8};

    let mut sum_r = 0.0_f64;
    let mut sum_g = 0.0_f64;
    let mut sum_b = 0.0_f64;
    let mut count: u64 = 0;
    for px in rgba.chunks_exact(4) {
        if px[3] == 0 {
            continue;
        }
        sum_r += srgb_to_linear_u8(px[0]) as f64;
        sum_g += srgb_to_linear_u8(px[1]) as f64;
        sum_b += srgb_to_linear_u8(px[2]) as f64;
        count += 1;
    }
    if count == 0 {
        return;
    }
    let mean_r = (sum_r / count as f64) as f32;
    let mean_g = (sum_g / count as f64) as f32;
    let mean_b = (sum_b / count as f64) as f32;
    if mean_r == 0.0 || mean_g == 0.0 || mean_b == 0.0 {
        return;
    }
    let mean_gray = (mean_r + mean_g + mean_b) / 3.0;
    let gain_r = mean_gray / mean_r;
    let gain_g = mean_gray / mean_g;
    let gain_b = mean_gray / mean_b;
    for px in rgba.chunks_exact_mut(4) {
        if px[3] == 0 {
            continue;
        }
        let r_lin = srgb_to_linear_u8(px[0]) * gain_r;
        let g_lin = srgb_to_linear_u8(px[1]) * gain_g;
        let b_lin = srgb_to_linear_u8(px[2]) * gain_b;
        px[0] = linear_to_srgb_u8(r_lin);
        px[1] = linear_to_srgb_u8(g_lin);
        px[2] = linear_to_srgb_u8(b_lin);
    }
}

// ── Stage 4 — Phase 2 ─────────────────────────────────────────────────────
//
// Histogram + automatic adjustments + sharpen. Each is pure /
// deterministic / `std`-only. CPU implementations target the Apply
// path (one-shot per sprite); a WGSL compute follow-up is annotated
// per stage where the GPU win is large (Unsharp Mask).

/// Per-channel 256-bin histogram (R, G, B, and BT.709 luma) plus the count
/// of opaque pixels that contributed. Built by [`compute_histogram`] from
/// a straight-alpha RGBA8 buffer; consumed by [`auto_levels`] /
/// [`auto_contrast`] / [`auto_colors`] and by the panel's overlay
/// visualizer. Skips fully transparent pixels.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HistogramData {
    pub r: [u32; 256],
    pub g: [u32; 256],
    pub b: [u32; 256],
    /// BT.709 luma — `Y = 0.2126·R + 0.7152·G + 0.0722·B`. Matches the
    /// sRGB primaries CLAHE / `luma_srgb` rely on; the older BT.601
    /// (`0.299, 0.587, 0.114`) constants were for analog NTSC encoding
    /// and don't reflect modern sRGB luminance.
    pub l: [u32; 256],
    /// Pixels with `alpha > 0` counted across all channels.
    pub opaque_count: u32,
}

impl Default for HistogramData {
    fn default() -> Self {
        Self {
            r: [0; 256],
            g: [0; 256],
            b: [0; 256],
            l: [0; 256],
            opaque_count: 0,
        }
    }
}

/// Compute per-channel + luma histograms from a straight-alpha RGBA8
/// buffer. Skips fully transparent pixels.
///
/// CPU-only by design: atomic histogram updates on GPU suffer contention
/// and the analytical passes already dominate the per-image cost in CLAHE
/// when the histogram is needed there. Linear-scan CPU is ~4 ns per pixel
/// in release — under 5 ms for 1024².
pub fn compute_histogram(pixels: &[ph2d_color::SrgbRgba]) -> HistogramData {
    let rgba: &[u8] = bytemuck::cast_slice(pixels);
    let mut h = HistogramData::default();
    for px in rgba.chunks_exact(4) {
        if px[3] == 0 {
            continue;
        }
        h.r[px[0] as usize] += 1;
        h.g[px[1] as usize] += 1;
        h.b[px[2] as usize] += 1;
        // BT.709 luma — sRGB primaries. Coefficients applied to sRGB
        // gamma-encoded values (matches the inline Y in `clahe`), not
        // linear — fine for histogram bucketing.
        let luma = (0.2126 * px[0] as f32 + 0.7152 * px[1] as f32 + 0.0722 * px[2] as f32) as usize;
        h.l[luma.min(255)] += 1;
        h.opaque_count += 1;
    }
    h
}

/// Build a 256-byte LUT that stretches `[min, max]` linearly onto
/// `[0, 255]`. Pixels at the extremes saturate. Used by the auto-* stages.
fn stretch_lut(min: u8, max: u8) -> [u8; 256] {
    let mut lut = [0u8; 256];
    let range = (max as i32 - min as i32).max(1) as f32;
    for (i, v) in lut.iter_mut().enumerate() {
        let stretched = ((i as i32 - min as i32) as f32 / range) * 255.0;
        *v = clamp8(stretched);
    }
    lut
}

/// Find the `[min, max]` channel range that excludes the bottom and top
/// `cutoff_fraction` percentiles. `cutoff_fraction` is in `[0, 1]` —
/// typical values are `0.005` (Auto Levels) or `0.01` (Auto Colors).
fn percentile_range(hist: &[u32; 256], total: u32, cutoff_fraction: f32) -> (u8, u8) {
    if total == 0 {
        return (0, 255);
    }
    let cutoff = (total as f32 * cutoff_fraction).floor() as u32;
    let mut count: u32 = 0;
    let mut lo = 0u8;
    for (v, &c) in hist.iter().enumerate() {
        count += c;
        if count > cutoff {
            lo = v as u8;
            break;
        }
    }
    count = 0;
    let mut hi = 255u8;
    for (v, &c) in hist.iter().enumerate().rev() {
        count += c;
        if count > cutoff {
            hi = v as u8;
            break;
        }
    }
    (lo, hi)
}

/// Auto Levels — per-channel histogram stretching with 0.5 % outlier
/// trimming. Same `findRange` shape as the legacy `autoLevels`.
pub fn auto_levels(rgba: &mut [u8]) {
    let hist = compute_histogram(bytemuck::cast_slice(rgba));
    if hist.opaque_count == 0 {
        return;
    }
    let (r_lo, r_hi) = percentile_range(&hist.r, hist.opaque_count, 0.005);
    let (g_lo, g_hi) = percentile_range(&hist.g, hist.opaque_count, 0.005);
    let (b_lo, b_hi) = percentile_range(&hist.b, hist.opaque_count, 0.005);
    let lut_r = stretch_lut(r_lo, r_hi);
    let lut_g = stretch_lut(g_lo, g_hi);
    let lut_b = stretch_lut(b_lo, b_hi);
    for px in rgba.chunks_exact_mut(4) {
        if px[3] == 0 {
            continue;
        }
        px[0] = lut_r[px[0] as usize];
        px[1] = lut_g[px[1] as usize];
        px[2] = lut_b[px[2] as usize];
    }
}

/// Auto Colors — per-channel stretching with 1 % outlier trimming.
/// Softer than Auto Levels; matches the legacy `autoColors`.
pub fn auto_colors(rgba: &mut [u8]) {
    let hist = compute_histogram(bytemuck::cast_slice(rgba));
    if hist.opaque_count == 0 {
        return;
    }
    let (r_lo, r_hi) = percentile_range(&hist.r, hist.opaque_count, 0.01);
    let (g_lo, g_hi) = percentile_range(&hist.g, hist.opaque_count, 0.01);
    let (b_lo, b_hi) = percentile_range(&hist.b, hist.opaque_count, 0.01);
    let lut_r = stretch_lut(r_lo, r_hi);
    let lut_g = stretch_lut(g_lo, g_hi);
    let lut_b = stretch_lut(b_lo, b_hi);
    for px in rgba.chunks_exact_mut(4) {
        if px[3] == 0 {
            continue;
        }
        px[0] = lut_r[px[0] as usize];
        px[1] = lut_g[px[1] as usize];
        px[2] = lut_b[px[2] as usize];
    }
}

/// Auto Contrast — stretches BT.709 **linear-light** luminance via a
/// uniform ratio scale on linear-sRGB RGB (preserves hue). Uses 5 %/95 %
/// percentile cut.
///
/// Linear-light Y is the correct lightness measure here: pure red
/// (255,0,0) and pure blue (0,0,255) have HSL L = 0.5 each, but their
/// linear luminances differ by ~3× (`Y_red ≈ 0.21`, `Y_blue ≈ 0.07`).
/// HSL L would treat them as equivalent and the per-channel scale by
/// `new_L/L` would push saturated pixels past 1.0 in one channel before
/// the others — manifest as hue drift. BT.709 linear keeps the scale
/// physically meaningful.
pub fn auto_contrast(rgba: &mut [u8]) {
    use crate::color_utils::{linear_to_srgb_u8, srgb_to_linear_u8};

    // 1. Linear-luma histogram (256 bins over `[0, 1]`).
    let mut hist_l = [0u32; 256];
    let mut total: u32 = 0;
    for px in rgba.chunks_exact(4) {
        if px[3] == 0 {
            continue;
        }
        let rl = srgb_to_linear_u8(px[0]);
        let gl = srgb_to_linear_u8(px[1]);
        let bl = srgb_to_linear_u8(px[2]);
        let y = 0.2126 * rl + 0.7152 * gl + 0.0722 * bl;
        let bin = (y.clamp(0.0, 1.0) * 255.0).round() as usize;
        hist_l[bin.min(255)] += 1;
        total += 1;
    }
    if total == 0 {
        return;
    }
    let (lo, hi) = percentile_range(&hist_l, total, 0.05);
    let lo_n = lo as f32 / 255.0;
    let range = ((hi as f32 - lo as f32) / 255.0).max(f32::EPSILON);

    // 2. Per-pixel: stretch linear Y, scale linear RGB by the ratio, encode.
    for px in rgba.chunks_exact_mut(4) {
        if px[3] == 0 {
            continue;
        }
        let rl = srgb_to_linear_u8(px[0]);
        let gl = srgb_to_linear_u8(px[1]);
        let bl = srgb_to_linear_u8(px[2]);
        let y = 0.2126 * rl + 0.7152 * gl + 0.0722 * bl;
        if y <= 0.0 || y >= 1.0 {
            continue;
        }
        let new_y = ((y - lo_n) / range).clamp(0.0, 1.0);
        let ratio = new_y / y;
        px[0] = linear_to_srgb_u8(rl * ratio);
        px[1] = linear_to_srgb_u8(gl * ratio);
        px[2] = linear_to_srgb_u8(bl * ratio);
    }
}

/// Sharpen via the 3×3 Laplacian kernel `[0,-1,0; -1,5,-1; 0,-1,0]` in
/// **linear sRGB**. `amount` in `[0, 2]`:
/// `result = center + (laplacian − center) · amount`. Fast and CPU-
/// friendly; use this when `radius ≤ 1`.
///
/// Linear-space sharpening avoids the gamma-space asymmetry of the old
/// path (sRGB compresses shadows → equal-amplitude ringing was perceived
/// stronger in dark areas than in highlights). Linear ringing is
/// uniformly visible across the tonal range.
pub fn sharpen_laplacian(rgba: &mut [u8], w: u32, h: u32, amount: f32) {
    use crate::color_utils::{linear_to_srgb_u8, srgb_to_linear_u8};

    if amount <= 0.0 {
        return;
    }
    let w_i = w as i32;
    let h_i = h as i32;
    let stride = w as usize;
    let n_px = (w as usize) * (h as usize);

    // Pre-linearize once; reused for 4 neighbour lookups per pixel × 3
    // channels — amortises the sRGB transfer.
    let mut src_lin: Vec<[f32; 3]> = Vec::with_capacity(n_px);
    for px in rgba.chunks_exact(4) {
        src_lin.push([
            srgb_to_linear_u8(px[0]),
            srgb_to_linear_u8(px[1]),
            srgb_to_linear_u8(px[2]),
        ]);
    }

    for y in 0..h_i {
        for x in 0..w_i {
            let cidx = (y as usize) * stride + (x as usize);
            let ci = cidx * 4;
            if rgba[ci + 3] == 0 {
                continue;
            }
            let center = src_lin[cidx];
            let top = if y > 0 {
                src_lin[cidx - stride]
            } else {
                center
            };
            let bottom = if y < h_i - 1 {
                src_lin[cidx + stride]
            } else {
                center
            };
            let left = if x > 0 { src_lin[cidx - 1] } else { center };
            let right = if x < w_i - 1 {
                src_lin[cidx + 1]
            } else {
                center
            };
            for ch in 0..3 {
                let laplacian = 5.0 * center[ch] - top[ch] - bottom[ch] - left[ch] - right[ch];
                let result = center[ch] + (laplacian - center[ch]) * amount;
                rgba[ci + ch] = linear_to_srgb_u8(result);
            }
        }
    }
}

/// Sharpen via unsharp masking (Gaussian blur → subtract → add scaled
/// difference) in **linear sRGB**. Use this when `radius > 1`. `radius`
/// typically `1.5..3`; `amount` in `[0, 2]`.
///
/// Why linear: Gaussian blur of gamma-encoded values darkens edges
/// (mean(sRGB_dark, sRGB_light) ≠ sRGB(mean(linear_dark, linear_light))).
/// In sharpen the visible effect is asymmetric ringing — undershoots
/// in shadows are exaggerated, overshoots in highlights are muted. The
/// linear-space pipeline keeps both edges symmetric.
///
/// **GPU note**: separable Gaussian blur is the canonical GPU win — two
/// horizontal + vertical passes scale linearly with radius on CPU but
/// stay near-constant on GPU. CPU path here is fine for radius ≤ 5 in
/// 1024² previews; large-radius production sharpen should use WGSL.
pub fn sharpen_unsharp(rgba: &mut [u8], w: u32, h: u32, amount: f32, radius: f32) {
    use crate::color_utils::{linear_to_srgb_u8, srgb_to_linear_u8};

    if amount <= 0.0 || radius <= 0.0 {
        return;
    }
    let kernel = gaussian_kernel_1d(radius);
    let size = kernel.len();
    let half = (size / 2) as i32;
    let total = (w as usize) * (h as usize);
    let w_i = w as i32;
    let h_i = h as i32;

    for ch in 0..3 {
        // Extract channel as **linear** sRGB into f32 buffer.
        let mut channel: Vec<f32> = (0..total)
            .map(|i| srgb_to_linear_u8(rgba[i * 4 + ch]))
            .collect();
        let original_lin: Vec<f32> = channel.clone();

        // Horizontal pass (separable).
        let mut h_pass = vec![0.0_f32; total];
        for y in 0..h_i {
            for x in 0..w_i {
                let mut sum = 0.0;
                let mut wt = 0.0;
                for (k, &kw) in kernel.iter().enumerate() {
                    let sx = (x + k as i32 - half).clamp(0, w_i - 1);
                    sum += channel[y as usize * w as usize + sx as usize] * kw;
                    wt += kw;
                }
                h_pass[y as usize * w as usize + x as usize] = sum / wt;
            }
        }

        // Vertical pass into `channel` (reused as blur output, in linear).
        for y in 0..h_i {
            for x in 0..w_i {
                let mut sum = 0.0;
                let mut wt = 0.0;
                for (k, &kw) in kernel.iter().enumerate() {
                    let sy = (y + k as i32 - half).clamp(0, h_i - 1);
                    sum += h_pass[sy as usize * w as usize + x as usize] * kw;
                    wt += kw;
                }
                channel[y as usize * w as usize + x as usize] = sum / wt;
            }
        }

        // Unsharp combine in linear: `original + amount · (original − blur)`.
        // Encode back to sRGB on write.
        for i in 0..total {
            if rgba[i * 4 + 3] == 0 {
                continue;
            }
            let orig = original_lin[i];
            let blur = channel[i];
            let diff = orig - blur;
            rgba[i * 4 + ch] = linear_to_srgb_u8(orig + amount * diff);
        }
    }
}

/// Normalised 1D Gaussian kernel of odd length `⌈radius·2⌉·2+1`, with
/// `σ = radius / 2`. Centred so index `size / 2` is the peak. `pub`
/// so the WGSL Unsharp port ([`crate::gpu::sharpen`]) can share the
/// exact same kernel — single source of truth.
pub fn gaussian_kernel_1d(radius: f32) -> Vec<f32> {
    let size = ((radius * 2.0).ceil() as usize) * 2 + 1;
    let sigma = (radius / 2.0).max(f32::EPSILON);
    let half = (size / 2) as f32;
    let mut kernel = vec![0.0_f32; size];
    let mut sum = 0.0_f32;
    for (i, k) in kernel.iter_mut().enumerate() {
        let d = i as f32 - half;
        *k = (-(d * d) / (2.0 * sigma * sigma)).exp();
        sum += *k;
    }
    for k in kernel.iter_mut() {
        *k /= sum;
    }
    kernel
}

// ── Stage 7 — Posterize + Quantize ───────────────────────────────────────
//
// Both are sequential CPU stages by construction:
// - Posterize w/ Floyd-Steinberg propagates per-pixel error to four
//   forward neighbours — strict raster-scan order, no SIMD/GPU port.
// - K-Means++ Quantize iterates a population-wide cluster fit then
//   re-maps every pixel — sample-bounded, deterministic seed for stable
//   palettes on identical inputs.
//
// Pipeline runs them AFTER all GPU-amenable stages (auto-WB) so the
// chained shader path can read back once before this section.

/// Smallest value of `levels` that activates posterize. `0`/`1` are
/// reserved "off" sentinels (a 1-level posterize is meaningless: every
/// pixel would map to the same value).
pub const POSTERIZE_LEVELS_MIN: u32 = 2;
/// Cap matching the legacy panel's discrete option list (`2, 3, 4, 6, 8,
/// 16`). Higher values would round-trip nearly unchanged.
pub const POSTERIZE_LEVELS_MAX: u32 = 16;

/// Smallest k for K-Means++ quantization (mirror of the legacy panel's
/// `4, 8, 16, 32, 64, 128, 256` list). Below 2 the algorithm collapses
/// to a single colour, which is the "off" sentinel.
pub const QUANTIZE_COLORS_MIN: u32 = 2;
/// Hard cap — 256 colours is the indexed-image standard and matches the
/// legacy panel's top option. Above it the sample budget (30k pixels)
/// no longer covers the centroid space cleanly.
pub const QUANTIZE_COLORS_MAX: u32 = 256;

/// K-Means++ sample cap (legacy parity). Quantize on very large images
/// would otherwise pay O(N · k) per iteration; the sampled subset
/// already covers cluster space well at this size.
const QUANTIZE_SAMPLE_CAP: usize = 30_000;

/// Max K-Means iterations (legacy parity). Convergence usually trips the
/// `QUANTIZE_CONVERGE_EPS` early-exit by iter 4-6 on natural images.
const QUANTIZE_MAX_ITER: usize = 10;

/// Centroid Δ threshold (OKLab units). When every centroid moves less
/// than this between iterations we stop early — palette already stable.
const QUANTIZE_CONVERGE_EPS: f32 = 0.001;

/// Deterministic xorshift seed for the K-Means++ initial-centroid draw.
/// Hard-coded so the same input + `num_colors` always produces the same
/// palette across runs (important for snapshot tests + a user re-running
/// Quantize getting the same result, not a new palette every time).
const QUANTIZE_SEED: u64 = 0x517c_c1b7_2722_0a95;

/// Reduce each RGB channel to `levels` discrete steps. When `dithering`
/// is `true`, Floyd-Steinberg error diffusion (7/16 right, 3/16 bottom-
/// left, 5/16 bottom, 1/16 bottom-right) carries the per-channel
/// quantization residue forward through the raster — the legacy
/// pattern, smoother on gradients than the plain map.
///
/// `levels < 2` is the off-sentinel (no-op). Alpha is preserved.
/// In-place on straight-alpha RGBA8.
///
/// **Color space: sRGB gamma (intentional).** The Tier 3 audit considered
/// migrating to linear sRGB for theoretical consistency with the other
/// Phase 2 stages, but FS dithering in linear preserves *physical* light
/// average rather than *perceptual* brightness — a uniform mid-grey 128
/// would dither to ~21% white pixels (linear mean 0.214 = sRGB 128) and
/// the perceived brightness would shift drastically. Pixel-art workflows
/// expect the dithered mosaic to read as the same grey, so the
/// quantization step + error diffusion stay in sRGB. This is also what
/// every reference implementation (legacy engine, Aseprite, GIMP)
/// expects, so palette outputs stay byte-compatible.
pub fn posterize(
    rgba: &mut [u8],
    w: u32,
    h: u32,
    levels: u32,
    dithering: bool,
    dither_strength: f32,
    dither_grain: u32,
) {
    if levels < POSTERIZE_LEVELS_MIN || w == 0 || h == 0 {
        return;
    }
    let levels = levels.min(POSTERIZE_LEVELS_MAX);
    let step = 255.0 / ((levels - 1) as f32);
    let total = (w as usize) * (h as usize);
    debug_assert_eq!(rgba.len(), total * 4);

    let strength = if !dithering {
        0.0
    } else {
        dither_strength.clamp(0.0, 1.0)
    };
    let grain = dither_grain.clamp(1, 8);

    if strength <= f32::EPSILON {
        for px in rgba.chunks_exact_mut(4) {
            for c in &mut px[..3] {
                *c = posterize_value(*c as f32, step);
            }
        }
        return;
    }

    // Floyd-Steinberg path. Grain>1 downsamples to a (w/grain × h/grain)
    // working buffer (block average), runs FS on that grid, then re-
    // upsamples (nearest) into the output. Grain=1 is per-pixel FS.
    let gw = w.div_ceil(grain);
    let gh = h.div_ceil(grain);
    let gtotal = (gw as usize) * (gh as usize);
    let mut buf = vec![0.0_f32; gtotal * 3];

    if grain == 1 {
        for (i, px) in rgba.chunks_exact(4).enumerate() {
            buf[i * 3] = px[0] as f32;
            buf[i * 3 + 1] = px[1] as f32;
            buf[i * 3 + 2] = px[2] as f32;
        }
    } else {
        for by in 0..gh {
            for bx in 0..gw {
                let x0 = (bx * grain) as usize;
                let y0 = (by * grain) as usize;
                let x1 = (x0 + grain as usize).min(w as usize);
                let y1 = (y0 + grain as usize).min(h as usize);
                let mut acc = [0.0_f32; 3];
                let mut count = 0u32;
                for y in y0..y1 {
                    for x in x0..x1 {
                        let pi = (y * w as usize + x) * 4;
                        acc[0] += rgba[pi] as f32;
                        acc[1] += rgba[pi + 1] as f32;
                        acc[2] += rgba[pi + 2] as f32;
                        count += 1;
                    }
                }
                let bi = ((by * gw + bx) as usize) * 3;
                let inv = if count == 0 { 0.0 } else { 1.0 / count as f32 };
                buf[bi] = acc[0] * inv;
                buf[bi + 1] = acc[1] * inv;
                buf[bi + 2] = acc[2] * inv;
            }
        }
    }

    let w_i = gw as isize;
    let h_i = gh as isize;
    for y in 0..h_i {
        for x in 0..w_i {
            let bi = ((y * w_i + x) * 3) as usize;
            for ch in 0..3 {
                let old = buf[bi + ch];
                let new_v = posterize_value(old, step);
                buf[bi + ch] = new_v as f32;
                let err = old - new_v as f32;
                if x + 1 < w_i {
                    buf[bi + 3 + ch] += err * (7.0 / 16.0);
                }
                if y + 1 < h_i {
                    let below = (((y + 1) * w_i + x) * 3) as usize;
                    if x > 0 {
                        buf[below - 3 + ch] += err * (3.0 / 16.0);
                    }
                    buf[below + ch] += err * (5.0 / 16.0);
                    if x + 1 < w_i {
                        buf[below + 3 + ch] += err * (1.0 / 16.0);
                    }
                }
            }
        }
    }

    // Sample (downsampled) buffer back to full res; lerp with the
    // per-pixel plain posterize result by `strength`.
    for y in 0..h as usize {
        for x in 0..w as usize {
            let bx = (x / grain as usize).min(gw as usize - 1);
            let by = (y / grain as usize).min(gh as usize - 1);
            let bi = (by * gw as usize + bx) * 3;
            let pi = (y * w as usize + x) * 4;
            for ch in 0..3 {
                let dith = buf[bi + ch];
                let plain = posterize_value(rgba[pi + ch] as f32, step) as f32;
                let out = plain + (dith - plain) * strength;
                rgba[pi + ch] = clamp8(out);
            }
        }
    }
}

fn posterize_value(v: f32, step: f32) -> u8 {
    let clamped = v.clamp(0.0, 255.0);
    let quantized = (clamped / step).round() * step;
    clamp8(quantized)
}

/// Reduce an image to `num_colors` perceptually balanced colours via
/// K-Means++ clustering in OKLab. Sampling caps the per-iteration cost
/// at [`QUANTIZE_SAMPLE_CAP`] pixels; the resulting palette is mapped
/// back across every opaque pixel (alpha = 0 pixels are skipped — a
/// transparent pixel has no colour to assign).
///
/// `num_colors < 2` is the off-sentinel (no-op). Reproducibility: the
/// K-Means++ seeding RNG is fixed ([`QUANTIZE_SEED`]) so re-quantizing
/// the same image with the same `num_colors` yields the same palette,
/// not a new one each invocation.
pub fn quantize(rgba: &mut [u8], w: u32, h: u32, num_colors: u32) {
    if num_colors < QUANTIZE_COLORS_MIN || w == 0 || h == 0 {
        return;
    }
    let k = num_colors.min(QUANTIZE_COLORS_MAX) as usize;
    let total = (w as usize) * (h as usize);
    debug_assert_eq!(rgba.len(), total * 4);

    // ── Sample opaque pixels into OKLab ─────────────────────────────
    let sample_stride = (total / QUANTIZE_SAMPLE_CAP).max(1);
    let mut samples: Vec<[f32; 3]> = Vec::with_capacity(total / sample_stride + 1);
    for i in (0..total).step_by(sample_stride) {
        let px = &rgba[i * 4..i * 4 + 4];
        if px[3] == 0 {
            continue;
        }
        let lr = crate::color_utils::srgb_to_linear_u8(px[0]);
        let lg = crate::color_utils::srgb_to_linear_u8(px[1]);
        let lb = crate::color_utils::srgb_to_linear_u8(px[2]);
        samples.push(linear_rgb_to_oklab(lr, lg, lb));
    }
    if samples.is_empty() {
        return;
    }

    // ── K-Means++ palette in OKLab ──────────────────────────────────
    let centroids = kmeans_pp_oklab(&samples, k);

    // ── Materialise palette in sRGB (one round-trip per centroid) ───
    let palette_srgb: Vec<[u8; 3]> = centroids
        .iter()
        .map(|c| {
            let lin = oklab_to_linear_rgb(c[0], c[1], c[2]);
            [
                linear_to_srgb_u8(lin[0].max(0.0)),
                linear_to_srgb_u8(lin[1].max(0.0)),
                linear_to_srgb_u8(lin[2].max(0.0)),
            ]
        })
        .collect();

    // Re-encode the palette to OKLab too — we round-tripped through
    // sRGB8 quantization, so OKLab distance against the SHIPPED palette
    // colours (not the raw centroids) is what matches the visual swap.
    let palette_lab: Vec<[f32; 3]> = palette_srgb
        .iter()
        .map(|p| {
            let lr = crate::color_utils::srgb_to_linear_u8(p[0]);
            let lg = crate::color_utils::srgb_to_linear_u8(p[1]);
            let lb = crate::color_utils::srgb_to_linear_u8(p[2]);
            linear_rgb_to_oklab(lr, lg, lb)
        })
        .collect();

    // ── Map every opaque pixel to its nearest palette colour ─────────
    for i in 0..total {
        let px_off = i * 4;
        if rgba[px_off + 3] == 0 {
            continue;
        }
        let lr = crate::color_utils::srgb_to_linear_u8(rgba[px_off]);
        let lg = crate::color_utils::srgb_to_linear_u8(rgba[px_off + 1]);
        let lb = crate::color_utils::srgb_to_linear_u8(rgba[px_off + 2]);
        let lab = linear_rgb_to_oklab(lr, lg, lb);
        let mut best = 0usize;
        let mut best_d = f32::INFINITY;
        for (j, c) in palette_lab.iter().enumerate() {
            let dl = lab[0] - c[0];
            let da = lab[1] - c[1];
            let db = lab[2] - c[2];
            let d = dl * dl + da * da + db * db;
            if d < best_d {
                best_d = d;
                best = j;
            }
        }
        let pal = palette_srgb[best];
        rgba[px_off] = pal[0];
        rgba[px_off + 1] = pal[1];
        rgba[px_off + 2] = pal[2];
    }
}

/// xorshift64 — minimal deterministic RNG seeded from [`QUANTIZE_SEED`].
/// Used only by the K-Means++ initialisation; quality requirements are
/// modest (uniform draws over a small sample set) and we want zero
/// external deps.
fn xorshift64(state: &mut u64) -> u64 {
    let mut x = *state;
    x ^= x << 13;
    x ^= x >> 7;
    x ^= x << 17;
    *state = x;
    x
}

/// K-Means++ in OKLab. Returns `k` centroids (or fewer when the sample
/// set already has ≤ `k` points). The implementation is a 1:1 port of
/// the legacy [`quantize.ts`] — D²-weighted seeding then up to
/// [`QUANTIZE_MAX_ITER`] Lloyd iterations with the
/// [`QUANTIZE_CONVERGE_EPS`] early-exit.
fn kmeans_pp_oklab(samples: &[[f32; 3]], k: usize) -> Vec<[f32; 3]> {
    if samples.is_empty() {
        return vec![[0.5, 0.0, 0.0]];
    }
    if samples.len() <= k {
        return samples.to_vec();
    }
    let mut rng = QUANTIZE_SEED;
    let mut centroids: Vec<[f32; 3]> = Vec::with_capacity(k);
    centroids.push(samples[(xorshift64(&mut rng) as usize) % samples.len()]);

    // D²-weighted random selection for the remaining (k-1) centroids.
    while centroids.len() < k {
        let mut distances = Vec::with_capacity(samples.len());
        let mut total_d = 0.0_f32;
        for s in samples {
            let mut min_d = f32::INFINITY;
            for c in &centroids {
                let dl = s[0] - c[0];
                let da = s[1] - c[1];
                let db = s[2] - c[2];
                let d = dl * dl + da * da + db * db;
                if d < min_d {
                    min_d = d;
                }
            }
            distances.push(min_d);
            total_d += min_d;
        }
        if total_d <= 0.0 {
            centroids.push(samples[(xorshift64(&mut rng) as usize) % samples.len()]);
            continue;
        }
        let threshold = (xorshift64(&mut rng) as f32 / u64::MAX as f32) * total_d;
        let mut cumulative = 0.0_f32;
        let mut picked = false;
        for (i, d) in distances.iter().enumerate() {
            cumulative += d;
            if cumulative >= threshold {
                centroids.push(samples[i]);
                picked = true;
                break;
            }
        }
        if !picked {
            centroids.push(samples[(xorshift64(&mut rng) as usize) % samples.len()]);
        }
    }

    // Lloyd iterations: assign → average → repeat until stable.
    let mut assignments = vec![0_usize; samples.len()];
    for _ in 0..QUANTIZE_MAX_ITER {
        for (i, s) in samples.iter().enumerate() {
            let mut best = 0usize;
            let mut best_d = f32::INFINITY;
            for (j, c) in centroids.iter().enumerate() {
                let dl = s[0] - c[0];
                let da = s[1] - c[1];
                let db = s[2] - c[2];
                let d = dl * dl + da * da + db * db;
                if d < best_d {
                    best_d = d;
                    best = j;
                }
            }
            assignments[i] = best;
        }
        let mut sums = vec![[0.0_f32; 3]; k];
        let mut counts = vec![0_u32; k];
        for (i, s) in samples.iter().enumerate() {
            let j = assignments[i];
            sums[j][0] += s[0];
            sums[j][1] += s[1];
            sums[j][2] += s[2];
            counts[j] += 1;
        }
        let mut moved = false;
        for j in 0..k {
            if counts[j] == 0 {
                continue;
            }
            let inv = 1.0 / counts[j] as f32;
            let new_c = [sums[j][0] * inv, sums[j][1] * inv, sums[j][2] * inv];
            if (centroids[j][0] - new_c[0]).abs() > QUANTIZE_CONVERGE_EPS
                || (centroids[j][1] - new_c[1]).abs() > QUANTIZE_CONVERGE_EPS
                || (centroids[j][2] - new_c[2]).abs() > QUANTIZE_CONVERGE_EPS
            {
                moved = true;
            }
            centroids[j] = new_c;
        }
        if !moved {
            break;
        }
    }
    centroids
}

// ── Utility ──────────────────────────────────────────────────────────────

fn clamp8(v: f32) -> u8 {
    if v < 0.0 {
        0
    } else if v >= 255.0 {
        255
    } else {
        (v + 0.5) as u8
    }
}

/// Aspect-fit `(sw, sh)` inside a `max_dim × max_dim` box without
/// upscaling. The preview cap uses this to bound CLAHE work per slider
/// drag (briefing PREVIEW cap 512²).
pub fn aspect_fit_within(sw: u32, sh: u32, max_dim: u32) -> (u32, u32) {
    if sw == 0 || sh == 0 || max_dim == 0 {
        return (sw.max(1), sh.max(1));
    }
    if sw <= max_dim && sh <= max_dim {
        return (sw, sh);
    }
    if sw >= sh {
        let dh = ((sh as u64 * max_dim as u64) / sw as u64).max(1) as u32;
        (max_dim, dh)
    } else {
        let dw = ((sw as u64 * max_dim as u64) / sh as u64).max(1) as u32;
        (dw, max_dim)
    }
}

/// Bilinear-interpolating RGBA8 resize, own implementation (no `image`
/// dep). Maps each destination pixel back to a fractional source position
/// and bilinearly samples the four neighbours per channel (alpha
/// included).
pub fn resize_bilinear_rgba(
    src_pixels: &[ph2d_color::SrgbRgba],
    sw: u32,
    sh: u32,
    dw: u32,
    dh: u32,
) -> Vec<u8> {
    let src: &[u8] = bytemuck::cast_slice(src_pixels);
    let mut dst = vec![0u8; (dw as usize) * (dh as usize) * 4];
    if sw == 0 || sh == 0 || dw == 0 || dh == 0 {
        return dst;
    }
    let sx_scale = sw as f32 / dw as f32;
    let sy_scale = sh as f32 / dh as f32;
    for y in 0..dh {
        let sy = (y as f32 + 0.5) * sy_scale - 0.5;
        let sy0 = sy.floor().max(0.0) as i32;
        let sy1 = (sy0 + 1).min(sh as i32 - 1);
        let sy0_c = sy0.clamp(0, sh as i32 - 1);
        let wy = (sy - sy0 as f32).clamp(0.0, 1.0);
        for x in 0..dw {
            let sx = (x as f32 + 0.5) * sx_scale - 0.5;
            let sx0 = sx.floor().max(0.0) as i32;
            let sx1 = (sx0 + 1).min(sw as i32 - 1);
            let sx0_c = sx0.clamp(0, sw as i32 - 1);
            let wx = (sx - sx0 as f32).clamp(0.0, 1.0);
            let dst_off = ((y as usize) * (dw as usize) + x as usize) * 4;
            for c in 0..4 {
                let p00 = src[((sy0_c as usize) * (sw as usize) + sx0_c as usize) * 4 + c] as f32;
                let p10 = src[((sy0_c as usize) * (sw as usize) + sx1 as usize) * 4 + c] as f32;
                let p01 = src[((sy1 as usize) * (sw as usize) + sx0_c as usize) * 4 + c] as f32;
                let p11 = src[((sy1 as usize) * (sw as usize) + sx1 as usize) * 4 + c] as f32;
                let top = p00 + wx * (p10 - p00);
                let bot = p01 + wx * (p11 - p01);
                dst[dst_off + c] = clamp8(top + wy * (bot - top));
            }
        }
    }
    dst
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 4×4 RGBA8 with a single solid colour + opaque alpha.
    fn solid(w: u32, h: u32, rgb: [u8; 3]) -> Vec<u8> {
        let mut v = Vec::with_capacity((w * h * 4) as usize);
        for _ in 0..(w * h) {
            v.extend_from_slice(&[rgb[0], rgb[1], rgb[2], 255]);
        }
        v
    }

    /// Read the RGBA pixel at `(x, y)` from a `w`-wide RGBA8 buffer.
    fn px(buf: &[u8], w: u32, x: u32, y: u32) -> [u8; 4] {
        let i = ((y * w + x) * 4) as usize;
        [buf[i], buf[i + 1], buf[i + 2], buf[i + 3]]
    }

    #[test]
    fn clamp8_handles_overflow_and_underflow() {
        assert_eq!(clamp8(-10.0), 0);
        assert_eq!(clamp8(0.0), 0);
        assert_eq!(clamp8(127.4), 127);
        assert_eq!(clamp8(127.6), 128);
        assert_eq!(clamp8(255.0), 255);
        assert_eq!(clamp8(999.0), 255);
    }

    #[test]
    fn clahe_is_noop_on_uniform_image() {
        // Single-colour input has a Dirac-delta histogram per tile; the
        // CDF maps that bin to 255 and every other bin to 0. So the
        // output is still uniform — but maximally bright. To get a true
        // pass-through, the image must already have luminance 255.
        // Verify the always-255-luma case (solid white).
        let src = solid(16, 16, [255, 255, 255]);
        let mut dst = vec![0u8; src.len()];
        clahe(&src, 16, 16, 2.0, 4, &mut dst);
        for y in 0..16 {
            for x in 0..16 {
                assert_eq!(px(&dst, 16, x, y), [255, 255, 255, 255]);
            }
        }
    }

    #[test]
    fn clahe_preserves_alpha() {
        let mut src = solid(8, 8, [128, 64, 200]);
        // Make a few pixels semi-transparent.
        src[3] = 50;
        src[(7 + 7 * 8) * 4 + 3] = 0;
        let mut dst = vec![0u8; src.len()];
        clahe(&src, 8, 8, 2.0, 4, &mut dst);
        assert_eq!(dst[3], 50);
        assert_eq!(dst[(7 + 7 * 8) * 4 + 3], 0);
    }

    #[test]
    fn clahe_clip_limit_min_does_not_panic_and_still_normalizes_to_255() {
        // clip_limit=1.0 is the slider MIN. `build_clahe_lut` line 311
        // computes `clip = max(1.0, 1.0) * mean = mean`, so bins at or
        // below the mean don't get clipped — excess only accumulates from
        // bins above the mean. The CDF normalization still maps the top
        // bin to 255 regardless of clip strength. Regression cover
        // (Agent C audit, §6 BAIXA): no test gated this floor before.
        let src = solid(8, 8, [128, 128, 128]);
        let mut dst = vec![0u8; src.len()];
        // Should not panic; uniform input maps the (now single) populated
        // bin to 255 just like the larger clip_limit cases.
        clahe(&src, 8, 8, 1.0, 4, &mut dst);
        for px in dst.chunks_exact(4) {
            assert_eq!(px[0], 255);
            assert_eq!(px[3], 255);
        }
    }

    #[test]
    fn clahe_increases_dynamic_range_on_low_contrast_input() {
        // Build a 32×32 grayscale ramp clamped to mid-range [100, 150]
        // (low contrast). CLAHE should stretch the dynamic range so the
        // output spans a wider min/max gap.
        let mut src = Vec::with_capacity(32 * 32 * 4);
        for y in 0..32u32 {
            for x in 0..32u32 {
                let v = 100u8 + (((x + y) % 50) as u8);
                src.extend_from_slice(&[v, v, v, 255]);
            }
        }
        let mut dst = vec![0u8; src.len()];
        clahe(&src, 32, 32, 2.0, 4, &mut dst);
        let mut lo = 255u8;
        let mut hi = 0u8;
        for px in dst.chunks_exact(4) {
            lo = lo.min(px[0]);
            hi = hi.max(px[0]);
        }
        // Source spans 50 luma units; CLAHE should grow that.
        assert!(hi as i32 - lo as i32 >= 50, "CLAHE did not stretch range");
    }

    // ── Phase 1 — tonal stage primitives ─────────────────────────────

    #[test]
    fn exposure_zero_is_identity() {
        let mut rgb = [0.3_f32, 0.5, 0.7];
        let before = rgb;
        apply_exposure_linear(&mut rgb, 0.0);
        assert_eq!(rgb, before);
    }

    #[test]
    fn exposure_plus_one_ev_doubles_below_knee() {
        // 0.3 doubles to 0.6 (below soft-knee threshold of 0.8 → no
        // compression).
        let mut rgb = [0.3_f32, 0.3, 0.3];
        apply_exposure_linear(&mut rgb, 1.0);
        for c in rgb {
            assert!((c - 0.6).abs() < 1e-5, "got {c}");
        }
    }

    #[test]
    fn exposure_soft_knee_caps_below_one() {
        // 0.6 × 2^2 = 2.4 — would clip hard to 1.0 without soft knee.
        // Soft knee: 0.8 + 0.2·(1 - exp(-3.2)) ≈ 0.8 + 0.2·0.959 ≈ 0.99.
        let mut rgb = [0.6_f32, 0.6, 0.6];
        apply_exposure_linear(&mut rgb, 2.0);
        for c in rgb {
            assert!(
                c < 1.0 && c > 0.97,
                "soft knee should approach 1 from below: got {c}"
            );
        }
    }

    #[test]
    fn temperature_zero_is_identity_within_floats() {
        // Bradford D65→D65 is near-identity; with input == output passing
        // through `apply_temperature_linear` with the identity matrix
        // should hardly move the pixel.
        let m = bradford_matrix_for_kelvin(6500.0);
        let mut rgb = [0.4_f32, 0.55, 0.7];
        let before = rgb;
        apply_temperature_linear(&mut rgb, &m);
        for i in 0..3 {
            assert!(
                (rgb[i] - before[i]).abs() < 0.02,
                "channel {i} drifted: before {} after {}",
                before[i],
                rgb[i]
            );
        }
    }

    #[test]
    fn temperature_warm_lifts_red() {
        // Positive temperature (warm) — apply on neutral grey, R should
        // rise, B should drop.
        let m = bradford_matrix_for_kelvin(temperature01_to_kelvin(0.7));
        let mut rgb = [0.5_f32, 0.5, 0.5];
        apply_temperature_linear(&mut rgb, &m);
        assert!(rgb[0] > 0.5, "warm should boost R: got {}", rgb[0]);
        assert!(rgb[2] < 0.5, "warm should drop B: got {}", rgb[2]);
    }

    #[test]
    fn temperature_cool_lifts_blue() {
        let m = bradford_matrix_for_kelvin(temperature01_to_kelvin(-0.7));
        let mut rgb = [0.5_f32, 0.5, 0.5];
        apply_temperature_linear(&mut rgb, &m);
        assert!(rgb[0] < 0.5, "cool should drop R: got {}", rgb[0]);
        assert!(rgb[2] > 0.5, "cool should lift B: got {}", rgb[2]);
    }

    #[test]
    fn tint_positive_shifts_toward_magenta() {
        // Tint > 0 drops G and lifts R/B in compensation.
        let mut rgb = [0.5_f32, 0.5, 0.5];
        apply_tint_linear(&mut rgb, 0.5);
        assert!(rgb[1] < 0.5, "G should drop with magenta tint");
        assert!(rgb[0] > 0.5, "R should rise with magenta tint");
        assert!(rgb[2] > 0.5, "B should rise with magenta tint");
    }

    #[test]
    fn brightness_multiplicative_preserves_black() {
        // Critical legacy semantic: brightness is multiplicative, so pure
        // black stays pure black (no lift to mid-grey).
        let mut rgb = [0.0_f32, 0.0, 0.0];
        apply_brightness_linear(&mut rgb, 0.8);
        assert_eq!(rgb, [0.0, 0.0, 0.0]);
    }

    #[test]
    fn brightness_lifts_midtones() {
        let mut rgb = [0.3_f32, 0.3, 0.3];
        apply_brightness_linear(&mut rgb, 0.5);
        // m = 1.5 → 0.3 × 1.5 = 0.45.
        for c in rgb {
            assert!((c - 0.45).abs() < 1e-5);
        }
    }

    #[test]
    fn contrast_one_is_identity() {
        let mut rgb = [0.3_f32, 0.5, 0.7];
        let before = rgb;
        apply_contrast_linear(&mut rgb, 1.0);
        assert_eq!(rgb, before);
    }

    #[test]
    fn contrast_above_one_pushes_pixels_away_from_pivot() {
        // Pivot is 0.18; contrast > 1 pushes pixels above pivot UP and
        // pixels below pivot DOWN.
        let mut above = [0.5_f32, 0.5, 0.5];
        apply_contrast_linear(&mut above, 1.5);
        assert!(above[0] > 0.5, "pixel above pivot should rise: {above:?}");

        let mut below = [0.1_f32, 0.1, 0.1];
        apply_contrast_linear(&mut below, 1.5);
        assert!(below[0] < 0.1, "pixel below pivot should drop: {below:?}");
    }

    #[test]
    fn vibrance_zero_is_identity() {
        let mut lab = [0.5_f32, 0.1, 0.05];
        let before = lab;
        apply_vibrance_oklab(&mut lab, 0.0);
        assert_eq!(lab, before);
    }

    #[test]
    fn vibrance_boosts_low_chroma_more_than_high_chroma() {
        // Two pixels: low chroma (a=0.02, b=0.01) and high chroma
        // (a=0.20, b=0.10). Same vibrance value (+0.5). The low-chroma
        // pixel should gain proportionally more.
        let mut low = [0.5_f32, 0.02, 0.01];
        let mut hi = [0.5_f32, 0.20, 0.10];
        let chroma_low_before = (low[1].powi(2) + low[2].powi(2)).sqrt();
        let chroma_hi_before = (hi[1].powi(2) + hi[2].powi(2)).sqrt();
        apply_vibrance_oklab(&mut low, 0.5);
        apply_vibrance_oklab(&mut hi, 0.5);
        let chroma_low_after = (low[1].powi(2) + low[2].powi(2)).sqrt();
        let chroma_hi_after = (hi[1].powi(2) + hi[2].powi(2)).sqrt();
        let ratio_low = chroma_low_after / chroma_low_before;
        let ratio_hi = chroma_hi_after / chroma_hi_before;
        assert!(
            ratio_low > ratio_hi,
            "low-chroma pixel should grow more (got {ratio_low} vs {ratio_hi})"
        );
    }

    #[test]
    fn saturation_minus_one_zeroes_chroma() {
        let mut lab = [0.5_f32, 0.2, -0.1];
        apply_saturation_oklab(&mut lab, -1.0);
        // sat_mult = 0 → chroma collapses to 0.
        assert!(lab[1].abs() < 1e-6);
        assert!(lab[2].abs() < 1e-6);
    }

    #[test]
    fn saturation_plus_one_doubles_chroma() {
        let mut lab = [0.5_f32, 0.1, -0.05];
        apply_saturation_oklab(&mut lab, 1.0);
        // sat_mult = 2 → chroma doubles.
        assert!((lab[1] - 0.2).abs() < 1e-6);
        assert!((lab[2] - -0.1).abs() < 1e-6);
    }

    #[test]
    fn temperature01_to_kelvin_endpoints() {
        assert_eq!(temperature01_to_kelvin(0.0), 6500.0);
        // Photographer convention: +1 → warm = low Kelvin.
        assert_eq!(temperature01_to_kelvin(1.0), 2000.0);
        assert_eq!(temperature01_to_kelvin(-1.0), 10000.0);
        // Out-of-range clamps.
        assert_eq!(temperature01_to_kelvin(99.0), 2000.0);
        assert_eq!(temperature01_to_kelvin(-99.0), 10000.0);
    }

    // ── adjust_tonal (combined batch) ────────────────────────────────

    #[test]
    fn adjust_tonal_identity_leaves_pixels_within_one_lsb() {
        let mut buf = solid(4, 4, [60, 130, 200]);
        let before = buf.clone();
        adjust_tonal(&mut buf, &ColorEqualizationParams::default());
        for (a, b) in buf.iter().zip(before.iter()) {
            assert!(a.abs_diff(*b) <= 1, "drift {a} vs {b}");
        }
    }

    #[test]
    fn adjust_tonal_brightness_preserves_black() {
        // Critical: multiplicative brightness MUST keep pure black at 0.
        let mut buf = vec![0u8, 0, 0, 255];
        let p = ColorEqualizationParams {
            brightness: 0.8,
            ..ColorEqualizationParams::default()
        };
        adjust_tonal(&mut buf, &p);
        assert_eq!(&buf[..3], &[0, 0, 0]);
    }

    #[test]
    fn adjust_tonal_saturation_minus_one_grayscales() {
        let mut buf = solid(4, 4, [200, 50, 50]);
        let p = ColorEqualizationParams {
            saturation: -1.0,
            ..ColorEqualizationParams::default()
        };
        adjust_tonal(&mut buf, &p);
        let r = buf[0];
        let g = buf[1];
        let b = buf[2];
        // OKLab's perceptual luma differs from BT.709, so the grey value
        // won't exactly match input R; just assert channels collapsed to
        // the same value (within 2 LSB given OKLab cube-root rounding).
        assert!(r.abs_diff(g) <= 2, "R/G drift after desat: {r} vs {g}");
        assert!(g.abs_diff(b) <= 2, "G/B drift after desat: {g} vs {b}");
    }

    #[test]
    fn adjust_tonal_skips_transparent_pixels() {
        let mut buf = vec![100u8, 150, 200, 0, 100, 150, 200, 255];
        let p = ColorEqualizationParams {
            brightness: 0.5,
            ..ColorEqualizationParams::default()
        };
        adjust_tonal(&mut buf, &p);
        assert_eq!(&buf[0..4], &[100, 150, 200, 0]);
        assert!(buf[4] != 100, "opaque pixel should have been adjusted");
    }

    #[test]
    fn adjust_tonal_exposure_brightens() {
        let mut buf = solid(4, 4, [80, 80, 80]);
        let p = ColorEqualizationParams {
            exposure: 1.0, // +1 EV stop
            ..ColorEqualizationParams::default()
        };
        adjust_tonal(&mut buf, &p);
        assert!(buf[0] > 80, "+1 EV should brighten (got {})", buf[0]);
    }

    #[test]
    fn adjust_tonal_vibrance_increases_chroma() {
        let mut buf = solid(4, 4, [120, 100, 100]); // very mild red cast
        let before_r = buf[0] as i32;
        let p = ColorEqualizationParams {
            vibrance: 1.0,
            ..ColorEqualizationParams::default()
        };
        adjust_tonal(&mut buf, &p);
        // Low-chroma input → vibrance pumps it up — R should now be
        // visibly higher than G/B.
        assert!(
            (buf[0] as i32 - before_r) > 5,
            "vibrance did not pump low chroma: got delta {}",
            buf[0] as i32 - before_r
        );
    }

    #[test]
    fn auto_wb_balances_red_cast() {
        // Average sample: 200 R, 100 G, 100 B → mean grey ≈ 133. Gains
        // should drop R and lift G/B toward grey.
        let mut buf = solid(4, 4, [200, 100, 100]);
        auto_white_balance(&mut buf);
        let r = buf[0];
        let g = buf[1];
        let b = buf[2];
        assert!(r < 200, "R should drop after gray-world (got {r})");
        assert!(g > 100, "G should rise after gray-world (got {g})");
        assert!(b > 100, "B should rise after gray-world (got {b})");
    }

    #[test]
    fn auto_wb_skips_transparent_pixels() {
        // Single transparent pixel + one opaque pixel.
        let mut buf = vec![200u8, 100, 100, 0, 200, 100, 100, 255];
        auto_white_balance(&mut buf);
        assert_eq!(&buf[0..4], &[200, 100, 100, 0]);
        // Opaque pixel was rebalanced; it now sits closer to grey than
        // the input (200, 100, 100).
        let r = buf[4];
        let g = buf[5];
        let b = buf[6];
        assert!(r < 200 && g > 100 && b > 100);
    }

    #[test]
    fn auto_wb_noop_on_pure_grey() {
        let mut buf = solid(4, 4, [128, 128, 128]);
        let before = buf.clone();
        auto_white_balance(&mut buf);
        for (a, b) in buf.iter().zip(before.iter()) {
            assert!(a.abs_diff(*b) <= 1);
        }
    }

    #[test]
    fn run_pipeline_preserves_dimensions() {
        let src = solid(8, 8, [120, 80, 200]);
        let p = ColorEqualizationParams::default();
        let mut out = Vec::new();
        run_pipeline(bytemuck::cast_slice(&src), 8, 8, &p, &mut out);
        assert_eq!(out.len(), src.len());
    }

    #[test]
    fn run_pipeline_identity_round_trip_exact() {
        // Phase 1 audit (2026-05): with `ColorEqualizationParams::default()`
        // the pipeline must produce the source byte-for-byte. Defaults are
        // engineered identity (`CLIP_LIMIT_DEFAULT = CLIP_LIMIT_MIN`,
        // every tonal knob at its identity value, every Phase 2 toggle
        // off, no LUT preset, no posterize / quantize). The fast-path
        // guard in `run_pipeline` short-circuits on `is_noop()`; this
        // test pins the guarantee so a future stage author can't break
        // the "activating the tool with no edits is a no-op" invariant.
        //
        // Source spans every alpha state (opaque + semi-transparent +
        // fully transparent) and four primary hues so any stage that
        // sneaks in a unilateral mutation would diverge here.
        let p = ColorEqualizationParams::default();
        assert!(
            p.is_noop(),
            "test precondition: default params must be is_noop()"
        );
        let mut src = Vec::with_capacity(16 * 16 * 4);
        for y in 0..16u8 {
            for x in 0..16u8 {
                let r = x.wrapping_mul(17);
                let g = y.wrapping_mul(17);
                let b = (x ^ y).wrapping_mul(17);
                let a = match (x % 4, y % 4) {
                    (0, _) => 0,
                    (1, _) => 64,
                    (2, _) => 128,
                    _ => 255,
                };
                src.extend_from_slice(&[r, g, b, a]);
            }
        }
        let mut out = Vec::new();
        run_pipeline(bytemuck::cast_slice(&src), 16, 16, &p, &mut out);
        assert_eq!(out, src, "default params must round-trip identity");
    }

    #[test]
    fn run_pipeline_auto_wb_toggle_changes_output() {
        // Compose-level verification: with a red-cast input, toggling
        // auto-WB on must change the pipeline output relative to the same
        // pipeline with auto-WB off. (The pure auto-WB stage is exercised
        // by `auto_wb_balances_red_cast`; this test just confirms the
        // pipeline threads the flag through.)
        let mut src = Vec::with_capacity(8 * 8 * 4);
        for y in 0..8u32 {
            for x in 0..8u32 {
                let jitter = ((x + y) % 16) as u8;
                src.extend_from_slice(&[200 - jitter / 2, 100 + jitter, 100 + jitter, 255]);
            }
        }
        let p_off = ColorEqualizationParams {
            auto_wb: false,
            ..ColorEqualizationParams::default()
        };
        let p_on = ColorEqualizationParams {
            auto_wb: true,
            ..ColorEqualizationParams::default()
        };
        let mut out_off = Vec::new();
        let mut out_on = Vec::new();
        run_pipeline(bytemuck::cast_slice(&src), 8, 8, &p_off, &mut out_off);
        run_pipeline(bytemuck::cast_slice(&src), 8, 8, &p_on, &mut out_on);
        assert_ne!(
            out_off, out_on,
            "auto-wb flag did not affect pipeline output"
        );
    }

    #[test]
    fn resize_bilinear_identity() {
        let src = solid(4, 4, [100, 150, 200]);
        let dst = resize_bilinear_rgba(bytemuck::cast_slice(&src), 4, 4, 4, 4);
        for (a, b) in dst.iter().zip(src.iter()) {
            assert!(a.abs_diff(*b) <= 1);
        }
    }

    #[test]
    fn resize_bilinear_halves_dims() {
        let src = solid(8, 8, [100, 150, 200]);
        let dst = resize_bilinear_rgba(bytemuck::cast_slice(&src), 8, 8, 4, 4);
        assert_eq!(dst.len(), 4 * 4 * 4);
        // Solid colour → bilinear is identity-coloured.
        assert_eq!(&dst[..4], &[100, 150, 200, 255]);
    }

    #[test]
    fn aspect_fit_within_caps_larger_dimensions() {
        assert_eq!(aspect_fit_within(1024, 512, 512), (512, 256));
        assert_eq!(aspect_fit_within(512, 1024, 512), (256, 512));
        assert_eq!(aspect_fit_within(400, 300, 512), (400, 300));
    }

    // ── Phase 2 ──────────────────────────────────────────────────────

    #[test]
    fn histogram_skips_transparent_and_counts_opaque() {
        let buf = vec![
            10u8, 20, 30, 0, // transparent — skipped
            10, 20, 30, 255, 200, 100, 50, 255,
        ];
        let h = compute_histogram(bytemuck::cast_slice(&buf));
        assert_eq!(h.opaque_count, 2);
        assert_eq!(h.r[10], 1);
        assert_eq!(h.r[200], 1);
        assert_eq!(h.r[20], 0, "alpha=0 should not contribute");
    }

    #[test]
    fn histogram_total_equals_opaque_count() {
        let mut buf = Vec::with_capacity(8 * 8 * 4);
        for i in 0..(8 * 8) {
            buf.extend_from_slice(&[
                (i % 256) as u8,
                ((i * 2) % 256) as u8,
                ((i * 3) % 256) as u8,
                255,
            ]);
        }
        let h = compute_histogram(bytemuck::cast_slice(&buf));
        assert_eq!(h.opaque_count, 64);
        let r_total: u32 = h.r.iter().sum();
        let g_total: u32 = h.g.iter().sum();
        let b_total: u32 = h.b.iter().sum();
        let l_total: u32 = h.l.iter().sum();
        assert_eq!(r_total, 64);
        assert_eq!(g_total, 64);
        assert_eq!(b_total, 64);
        assert_eq!(l_total, 64);
    }

    #[test]
    fn auto_levels_stretches_compressed_range() {
        // Build a buffer whose R channel only occupies [80, 180].
        let mut buf = Vec::with_capacity(32 * 32 * 4);
        for y in 0..32u32 {
            for x in 0..32u32 {
                let r = 80u8 + (((x + y) % 100) as u8);
                buf.extend_from_slice(&[r, 128, 128, 255]);
            }
        }
        auto_levels(&mut buf);
        let mut lo = 255u8;
        let mut hi = 0u8;
        for px in buf.chunks_exact(4) {
            lo = lo.min(px[0]);
            hi = hi.max(px[0]);
        }
        // R channel now spans close to full range.
        assert!(lo <= 10, "auto_levels did not pull min down (got {lo})");
        assert!(hi >= 245, "auto_levels did not push max up (got {hi})");
    }

    #[test]
    fn auto_colors_preserves_uniform_distribution() {
        // Build a 64×64 image where each channel hits every value 0..255
        // multiple times (uniform distribution). 1 % cutoff (40 pixels at
        // each tail) won't shift min/max past 0 / 255, so auto_colors is
        // effectively identity.
        let mut buf = Vec::with_capacity(64 * 64 * 4);
        for i in 0..(64 * 64) {
            let v = (i % 256) as u8;
            buf.extend_from_slice(&[v, v, v, 255]);
        }
        let before = buf.clone();
        auto_colors(&mut buf);
        let mut max_drift = 0u8;
        for (a, b) in buf.iter().zip(before.iter()) {
            max_drift = max_drift.max(a.abs_diff(*b));
        }
        assert!(
            max_drift <= 2,
            "auto_colors on uniform-distribution buffer drifted by {max_drift}"
        );
    }

    #[test]
    fn auto_contrast_lifts_compressed_lightness() {
        // All pixels at L ≈ 0.4..0.6 (compressed mid-range).
        let mut buf = Vec::with_capacity(16 * 16 * 4);
        for y in 0..16u32 {
            for x in 0..16u32 {
                let v = 102u8 + (((x + y) % 50) as u8); // 102..152 ≈ L 0.4..0.6
                buf.extend_from_slice(&[v, v, v, 255]);
            }
        }
        auto_contrast(&mut buf);
        let mut lo = 255u8;
        let mut hi = 0u8;
        for px in buf.chunks_exact(4) {
            lo = lo.min(px[0]);
            hi = hi.max(px[0]);
        }
        assert!(
            hi as i32 - lo as i32 >= 100,
            "auto_contrast did not stretch"
        );
    }

    #[test]
    fn auto_levels_skips_transparent_pixels() {
        let mut buf = vec![10u8, 10, 10, 0, 50, 80, 120, 255, 200, 220, 240, 255];
        auto_levels(&mut buf);
        assert_eq!(&buf[0..4], &[10, 10, 10, 0]); // untouched
    }

    #[test]
    fn sharpen_laplacian_zero_amount_is_noop() {
        let mut buf = solid(8, 8, [120, 130, 140]);
        let before = buf.clone();
        sharpen_laplacian(&mut buf, 8, 8, 0.0);
        assert_eq!(buf, before);
    }

    #[test]
    fn sharpen_laplacian_increases_edge_contrast() {
        // 4×4 with a clean vertical edge at x=2: left=80, right=180.
        let mut buf = Vec::with_capacity(4 * 4 * 4);
        for _y in 0..4u32 {
            for x in 0..4u32 {
                let v = if x < 2 { 80u8 } else { 180u8 };
                buf.extend_from_slice(&[v, v, v, 255]);
            }
        }
        // Row 1 (`y = 1`), columns 1 (just-left-of-edge) and 2 (just-right).
        let idx_left = ((4 + 1) * 4) as usize;
        let idx_right = ((4 + 2) * 4) as usize;
        let edge_left_before = buf[idx_left];
        let edge_right_before = buf[idx_right];
        sharpen_laplacian(&mut buf, 4, 4, 1.0);
        let edge_left_after = buf[idx_left];
        let edge_right_after = buf[idx_right];
        let contrast_before = (edge_right_before as i32 - edge_left_before as i32).abs();
        let contrast_after = (edge_right_after as i32 - edge_left_after as i32).abs();
        assert!(
            contrast_after >= contrast_before,
            "Laplacian did not enhance edge: before {contrast_before}, after {contrast_after}"
        );
    }

    #[test]
    fn sharpen_unsharp_zero_amount_is_noop() {
        let mut buf = solid(8, 8, [120, 130, 140]);
        let before = buf.clone();
        sharpen_unsharp(&mut buf, 8, 8, 0.0, 2.0);
        assert_eq!(buf, before);
    }

    #[test]
    fn sharpen_unsharp_returns_non_empty_on_radius_above_one() {
        let mut buf = solid(16, 16, [120, 130, 140]);
        sharpen_unsharp(&mut buf, 16, 16, 0.5, 2.0);
        // Solid colour input → Gaussian blur returns the same value → diff
        // is zero → output equals input. So a basic smoke is: the function
        // ran without panicking + output remains a valid buffer.
        assert_eq!(buf.len(), 16 * 16 * 4);
    }

    #[test]
    fn gaussian_kernel_normalises_to_one() {
        for radius in [1.0_f32, 2.0, 3.0, 5.0] {
            let k = gaussian_kernel_1d(radius);
            let sum: f32 = k.iter().sum();
            assert!(
                (sum - 1.0).abs() < 1e-5,
                "kernel for radius {radius} did not normalize: sum {sum}"
            );
            // Centre is the peak.
            let mid = k.len() / 2;
            for (i, v) in k.iter().enumerate() {
                if i != mid {
                    assert!(*v <= k[mid] + 1e-6, "non-monotonic at radius {radius}");
                }
            }
        }
    }

    #[test]
    fn run_pipeline_lut_preset_toggle_changes_output() {
        // Activate Sepia in slot 1 — output should diverge from the
        // neutral CLAHE baseline (warm cast collapses chroma toward
        // sepia tones).
        let mut src = Vec::with_capacity(8 * 8 * 4);
        for i in 0..(8 * 8) {
            src.extend_from_slice(&[
                (i * 3 % 256) as u8,
                (i * 5 % 256) as u8,
                (i * 7 % 256) as u8,
                255,
            ]);
        }
        let p_off = ColorEqualizationParams {
            clip_limit: crate::params::CLIP_LIMIT_MIN,
            ..ColorEqualizationParams::default()
        };
        let p_on = ColorEqualizationParams {
            lut_preset_1: crate::lut_presets::LutPreset::Sepia,
            ..p_off
        };
        let mut out_off = Vec::new();
        let mut out_on = Vec::new();
        run_pipeline(bytemuck::cast_slice(&src), 8, 8, &p_off, &mut out_off);
        run_pipeline(bytemuck::cast_slice(&src), 8, 8, &p_on, &mut out_on);
        assert_ne!(out_off, out_on, "LUT preset toggle did not change output");
    }

    #[test]
    fn run_pipeline_dual_lut_blend_changes_output_at_midpoint() {
        // With slot 1 = Warm + slot 2 = Cool + mix = 0.5, the output
        // should sit between the two preset extremes (neither pure warm
        // nor pure cool).
        let mut src = Vec::with_capacity(8 * 8 * 4);
        for i in 0..(8 * 8) {
            src.extend_from_slice(&[128 + (i * 2 % 64) as u8, 128, 128 + (i * 3 % 64) as u8, 255]);
        }
        let base = ColorEqualizationParams {
            clip_limit: crate::params::CLIP_LIMIT_MIN,
            ..ColorEqualizationParams::default()
        };
        let p_warm = ColorEqualizationParams {
            lut_preset_1: crate::lut_presets::LutPreset::Warm,
            ..base
        };
        let p_cool = ColorEqualizationParams {
            lut_preset_1: crate::lut_presets::LutPreset::Cool,
            ..base
        };
        let p_blend = ColorEqualizationParams {
            lut_preset_1: crate::lut_presets::LutPreset::Warm,
            lut_preset_2: crate::lut_presets::LutPreset::Cool,
            lut_mix: 0.5,
            ..base
        };
        let mut warm = Vec::new();
        let mut cool = Vec::new();
        let mut blend = Vec::new();
        run_pipeline(bytemuck::cast_slice(&src), 8, 8, &p_warm, &mut warm);
        run_pipeline(bytemuck::cast_slice(&src), 8, 8, &p_cool, &mut cool);
        run_pipeline(bytemuck::cast_slice(&src), 8, 8, &p_blend, &mut blend);
        assert_ne!(blend, warm, "blend should not equal pure-warm");
        assert_ne!(blend, cool, "blend should not equal pure-cool");
    }

    #[test]
    fn run_pipeline_lut_intensity_zero_is_noop_relative_to_baseline() {
        let mut src = Vec::with_capacity(8 * 8 * 4);
        for i in 0..(8 * 8) {
            src.extend_from_slice(&[
                (i * 3 % 256) as u8,
                (i * 5 % 256) as u8,
                (i * 7 % 256) as u8,
                255,
            ]);
        }
        let base = ColorEqualizationParams {
            clip_limit: crate::params::CLIP_LIMIT_MIN,
            ..ColorEqualizationParams::default()
        };
        let p_zero = ColorEqualizationParams {
            lut_preset_1: crate::lut_presets::LutPreset::Cinematic,
            lut_intensity: 0.0,
            ..base
        };
        let mut baseline = Vec::new();
        let mut zero = Vec::new();
        run_pipeline(bytemuck::cast_slice(&src), 8, 8, &base, &mut baseline);
        run_pipeline(bytemuck::cast_slice(&src), 8, 8, &p_zero, &mut zero);
        assert_eq!(
            baseline, zero,
            "intensity=0 should short-circuit the LUT stage entirely"
        );
    }

    #[test]
    fn run_pipeline_auto_levels_toggle_changes_output() {
        // Build a low-range input (R ∈ [80, 180]); Auto Levels should
        // stretch it noticeably.
        let mut src = Vec::with_capacity(16 * 16 * 4);
        for y in 0..16u32 {
            for x in 0..16u32 {
                let r = 80u8 + (((x + y) % 100) as u8);
                src.extend_from_slice(&[r, 128, 128, 255]);
            }
        }
        let p_off = ColorEqualizationParams {
            clip_limit: crate::params::CLIP_LIMIT_MIN, // neutral CLAHE
            ..ColorEqualizationParams::default()
        };
        let p_on = ColorEqualizationParams {
            auto_levels: true,
            ..p_off
        };
        let mut out_off = Vec::new();
        let mut out_on = Vec::new();
        run_pipeline(bytemuck::cast_slice(&src), 16, 16, &p_off, &mut out_off);
        run_pipeline(bytemuck::cast_slice(&src), 16, 16, &p_on, &mut out_on);
        assert_ne!(out_off, out_on, "auto_levels toggle did not change output");
    }

    #[test]
    fn percentile_range_finds_endpoints() {
        let mut hist = [0u32; 256];
        hist[10] = 100;
        hist[200] = 100;
        let (lo, hi) = percentile_range(&hist, 200, 0.005);
        // 0.5 % of 200 = 1, so cutoff lifts past index 10's 100 → lo = 10.
        assert_eq!(lo, 10);
        assert_eq!(hi, 200);
    }

    // ── Posterize + Quantize ─────────────────────────────────────────

    #[test]
    fn posterize_off_is_noop() {
        let mut rgba = vec![17, 41, 89, 255, 33, 200, 7, 128];
        let original = rgba.clone();
        posterize(&mut rgba, 2, 1, 0, false, 1.0, 1);
        assert_eq!(rgba, original, "level 0 must be a no-op");
        posterize(&mut rgba, 2, 1, 1, false, 1.0, 1);
        assert_eq!(rgba, original, "level 1 is below MIN, must be a no-op");
    }

    #[test]
    fn posterize_plain_produces_levels_minus_one_steps() {
        // 2 levels → snap to {0, 255}. A mid-range pixel rounds toward
        // the nearest endpoint; alpha untouched.
        let mut rgba = vec![100, 200, 50, 200];
        posterize(&mut rgba, 1, 1, 2, false, 1.0, 1);
        assert!(rgba[0] == 0 || rgba[0] == 255);
        assert!(rgba[1] == 0 || rgba[1] == 255);
        assert!(rgba[2] == 0 || rgba[2] == 255);
        assert_eq!(rgba[3], 200, "alpha must pass through");
    }

    #[test]
    fn posterize_dithered_preserves_average_brightness() {
        // A uniform mid-grey field with FS dithering should land on a
        // mix of the two surrounding palette entries (0 and 255 for
        // levels=2), with the average within a few LSB of the input.
        let w = 32_u32;
        let h = 32_u32;
        let total = (w * h) as usize;
        let mut rgba = vec![128_u8; total * 4];
        for i in 0..total {
            rgba[i * 4 + 3] = 255;
        }
        let avg_before =
            rgba.chunks_exact(4).map(|p| p[0] as u32).sum::<u32>() as f32 / total as f32;
        posterize(&mut rgba, w, h, 2, true, 1.0, 1);
        let avg_after =
            rgba.chunks_exact(4).map(|p| p[0] as u32).sum::<u32>() as f32 / total as f32;
        assert!(
            (avg_before - avg_after).abs() < 6.0,
            "FS dithering must preserve global mean (before={avg_before}, after={avg_after})"
        );
    }

    #[test]
    fn quantize_off_is_noop() {
        let mut rgba = vec![17, 41, 89, 255, 33, 200, 7, 128];
        let original = rgba.clone();
        quantize(&mut rgba, 2, 1, 0);
        assert_eq!(rgba, original, "color count 0 must be a no-op");
        quantize(&mut rgba, 2, 1, 1);
        assert_eq!(
            rgba, original,
            "color count 1 is below MIN, must be a no-op"
        );
    }

    #[test]
    fn quantize_reduces_distinct_colours() {
        // 4×4 image with 16 distinct gradient colours, quantize to 4.
        // After mapping, distinct (R, G, B) triples must be ≤ 4.
        let w = 4_u32;
        let h = 4_u32;
        let mut rgba = Vec::with_capacity(64);
        for y in 0..h {
            for x in 0..w {
                rgba.extend_from_slice(&[
                    (x.saturating_mul(80) as u8),
                    (y.saturating_mul(80) as u8),
                    ((x + y).saturating_mul(40) as u8),
                    255,
                ]);
            }
        }
        quantize(&mut rgba, w, h, 4);
        let mut palette: std::collections::BTreeSet<(u8, u8, u8)> = Default::default();
        for px in rgba.chunks_exact(4) {
            palette.insert((px[0], px[1], px[2]));
        }
        assert!(
            palette.len() <= 4,
            "quantize(k=4) produced {} colours",
            palette.len()
        );
    }

    #[test]
    fn quantize_skips_fully_transparent_pixels() {
        // A transparent pixel's RGB must NOT be replaced by a palette
        // entry — the palette is derived from opaque pixels only and
        // remapping a transparent pixel would silently shift alpha-
        // composited content.
        let mut rgba = vec![10, 20, 30, 0, 200, 200, 200, 255];
        quantize(&mut rgba, 2, 1, 2);
        assert_eq!(
            &rgba[0..3],
            &[10, 20, 30],
            "transparent RGB must pass through"
        );
    }

    #[test]
    fn quantize_is_deterministic_for_same_input() {
        // Same input + k must produce IDENTICAL output across calls —
        // the K-Means++ RNG is fixed-seeded ([`QUANTIZE_SEED`]) so
        // re-quantizing the same image gives the same palette, not a
        // new one each time.
        let w = 8_u32;
        let h = 8_u32;
        let mut rgba_a = Vec::with_capacity(256);
        for i in 0..64 {
            rgba_a.extend_from_slice(&[
                (i * 4) as u8,
                (255 - i * 4) as u8,
                ((i * 7) % 255) as u8,
                255,
            ]);
        }
        let mut rgba_b = rgba_a.clone();
        quantize(&mut rgba_a, w, h, 4);
        quantize(&mut rgba_b, w, h, 4);
        assert_eq!(
            rgba_a, rgba_b,
            "K-Means++ must be deterministic per fixed seed"
        );
    }
}
