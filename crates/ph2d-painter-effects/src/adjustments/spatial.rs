//! Window- and coordinate-aware adjustment kernels — the spatial (neighbourhood)
//! filters (Gaussian / Sharpen / Motion / Chromatic Aberration) and the
//! coordinate-dependent per-pixel kinds (Noise / Halftone). The flat per-pixel
//! [`apply_adjustment`](super::apply_adjustment) cannot express these: it sees a
//! 1-D `&mut [[f32; 4]]` window with no 2-D layout and no canvas position.
//!
//! ## Two roles
//! 1. **Canonical CPU reference** for the GPU pass-graph
//!    (`ph2d-render::LayerCompositor` `SpatialAdjustment`). The σ↔radius mapping,
//!    the unsharp coefficient, and the motion box live HERE (single source of
//!    truth) and ARE gated: `spatial_weights_parity` pins [`gaussian_weights`] /
//!    [`motion_weights`] against the GPU. Chromatic Aberration follows the GPU
//!    `cs_chroma` model (canvas centre, `half_diag`, inward gather; B19.2).
//! 2. **CPU fallback** when a stack is not GPU-representable (per-layer mask /
//!    clipping / reference layer): [`apply_adjustment_windowed`] runs the kernel
//!    on the compositor's f32 window.
//!
//! All inputs/outputs are **straight, ENCODED f32 RGBA** (the compositor's
//! accumulator, ADR-0177), matching [`apply_adjustment`](super::apply_adjustment).
//! The spatial blurs/gathers run internally in **premultiplied display tones**,
//! like Krita at 8 bits (P4, measured) — their boundary is [`premultiply`] /
//! [`unpremultiply`]; Bloom alone is optical and crosses into light
//! ([`premultiply_em_luz`]) —, so colour AND coverage
//! feather together and transparent texels contribute nothing — fixing both the
//! clipped-to-silhouette blur and the transparent-edge colour speckle. The
//! compositor combine then uses the kernel's feathered alpha for spatial kinds
//! (see `compositor::compose`). Noise / Halftone / Shadows-Highlights are defined
//! in display (sRGB) space — the values as they are — and DO preserve coverage.
//!
//! ### Documented limitations (mirror the GPU pass-graph's own, handoff §4)
//! - **Spatial kernels on a sub-window** sample with clamp-to-edge at the window
//!   border; the compositor grows a dirty window by the summed [`window_halo`]s
//!   (or to the canvas) and crops, so `composite_region` == crop of `composite`.
//! - **Sampling is nearest-tap** for Motion / Chromatic Aberration. Bilinear taps
//!   are a quality refinement to land jointly with the GPU (`cs_blur_dir` /
//!   `cs_chroma`).
//! - **Sharpen `mask_edges`** is a deferred follow-up (semantics noted to the
//!   Coord). The GPU pass-graph must premultiply the materialised below-composite
//!   to match (the visible path — handoff to Coord).

use super::compute::{em_luz, linear_to_srgb_f32};
use super::*;
use ph2d_det_math::DetMath as _;

// B19b: as faixas e o blur de `k` por fora = o caminho antigo ao bit.
#[cfg(test)]
#[path = "spatial_faixas_tests.rs"]
mod faixas_tests;
#[path = "spatial_tonal.rs"]
mod tonal; // Bloom + Shadows/Highlights (LOC cap: sibling module)
#[cfg(test)]
pub(super) use tonal::SH_LUMA_FLOOR;
pub use tonal::{apply_bloom, apply_shadows_highlights, separable_blur_scalar};
/// Largest separable-blur half-width (kernel reaches `±MAX_BLUR_HALF` texels).
/// Bounds the weights buffer + per-pixel tap count. **Mirrors
/// `ph2d_render::layer_compositor::MAX_BLUR_HALF`** (kept in lock-step; a 256-px
/// blur is already far past any interactive use).
pub const MAX_BLUR_HALF: u32 = 256;

/// Cap on the LOW-res bloom blur radius (`radius / factor`). The downsample factor
/// is chosen so the actual kernel never exceeds this → cost is radius-independent.
/// **Mirrors `ph2d_render::layer_compositor::BLOOM_MAX_LOW_RADIUS`.**
const BLOOM_MAX_LOW_RADIUS: f32 = 16.0;

/// Bloom downsample factor for `radius` — a **power-of-two** chosen so the low-res
/// blur (`radius / factor`) is ≤ [`BLOOM_MAX_LOW_RADIUS`]; `1` (no downsample)
/// below that. **Mirrors `ph2d_render::layer_compositor::bloom_downsample_factor`
/// EXACTLY** (kept in lock-step like [`gaussian_weights`]) so the CPU fallback and
/// the GPU pass-graph produce the same glow at every radius — the Coord's
/// reconciliation (commit `e5eb54c`: radius-independent GPU Bloom).
pub(super) fn bloom_downsample_factor(radius: f32) -> u32 {
    if radius <= BLOOM_MAX_LOW_RADIUS {
        return 1;
    }
    ((radius / BLOOM_MAX_LOW_RADIUS).ceil() as u32)
        .next_power_of_two()
        .clamp(1, 32)
}

/// Geometry of the compositor window an adjustment is applied to: `acc` is a
/// `width × height` row-major slice anchored at canvas `(origin_x, origin_y)`.
/// Per-pixel kinds ignore it; spatial/coordinate kinds need it (the 2-D layout
/// for neighbour access, the origin for the absolute canvas coordinate).
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct AdjustWindow {
    pub width: u32,
    pub height: u32,
    pub origin_x: u32,
    pub origin_y: u32,
    /// The whole canvas the window lives in (Chromatic Aberration is centred on it).
    pub canvas_width: u32,
    pub canvas_height: u32,
}

impl AdjustWindow {
    /// A whole-canvas window (origin `0,0`). The exact, halo-free case for the
    /// spatial blurs.
    #[must_use]
    pub fn full(width: u32, height: u32) -> Self {
        Self {
            width,
            height,
            origin_x: 0,
            origin_y: 0,
            canvas_width: width,
            canvas_height: height,
        }
    }
}

/// How far (texels, Chebyshev) an output pixel of `params` reads its input — a window grown by the
/// SUM of the halos of a stack's kernels and cropped back is the crop of the whole-canvas result, to
/// the byte (gate `a_janela_suja_sob_um_ajuste_espacial_e_o_recorte_da_tela`). `None` = the kernel
/// depends on the window's SIZE (Bloom past the low-res cap: its downsample grid and upsample ratio
/// are the canvas's), so only the whole canvas is exact. Per-pixel and absolute-coordinate kinds: 0.
#[must_use]
pub fn window_halo(params: &AdjustmentParams) -> Option<u32> {
    let half = |r: f32| if r > 0.0 { gaussian_weights(r).1 } else { 0 };
    Some(match params {
        AdjustmentParams::GaussianBlur(p) => half(p.radius),
        AdjustmentParams::Sharpen(p) if p.amount != 0.0 => half(p.radius),
        AdjustmentParams::MotionBlur(p) if p.distance > 0.0 => motion_weights(p.distance).1,
        AdjustmentParams::ChromaticAberration(p) => {
            let m = p
                .red_shift
                .abs()
                .max(p.green_shift.abs())
                .max(p.blue_shift.abs());
            if !m.is_finite() {
                return None;
            }
            // |dir · shift / half_diag| ≤ |shift| per axis; +2 for the rounding of the tap.
            if m == 0.0 {
                0
            } else {
                (m.ceil() as u32).saturating_add(2)
            }
        }
        AdjustmentParams::Bloom(p) if p.intensity > 0.0 && p.radius > 0.0 => {
            if bloom_downsample_factor(p.radius) != 1 {
                return None;
            }
            half(p.radius)
        }
        AdjustmentParams::ShadowsHighlights(p) => {
            half(p.shadows_radius).max(half(p.highlights_radius))
        }
        _ => 0,
    })
}

/// Apply any adjustment `kind` to the compositor's ENCODED f32 window, with the
/// `win` geometry available to the spatial / coordinate kinds. Per-pixel kinds
/// delegate to [`apply_adjustment`](super::apply_adjustment) verbatim (the flat
/// path stays the single dispatch for those + keeps `ph2d-render`'s GPU-parity
/// test, which calls `apply_adjustment` directly, untouched).
pub fn apply_adjustment_windowed(
    kind: &AdjustmentKind,
    params: &AdjustmentParams,
    acc: &mut [[f32; 4]],
    win: AdjustWindow,
) {
    debug_assert_eq!(
        params.kind(),
        *kind,
        "apply_adjustment_windowed: kind/params variant mismatch"
    );
    debug_assert_eq!(
        acc.len(),
        (win.width as usize) * (win.height as usize),
        "apply_adjustment_windowed: acc length must equal win.width * win.height"
    );
    match (kind, params) {
        (AdjustmentKind::GaussianBlur, AdjustmentParams::GaussianBlur(p)) => {
            apply_gaussian(p, acc, win)
        }
        (AdjustmentKind::Sharpen, AdjustmentParams::Sharpen(p)) => apply_sharpen(p, acc, win),
        (AdjustmentKind::MotionBlur, AdjustmentParams::MotionBlur(p)) => {
            apply_motion_blur(p, acc, win)
        }
        (AdjustmentKind::ChromaticAberration, AdjustmentParams::ChromaticAberration(p)) => {
            apply_chromatic_aberration(p, acc, win)
        }
        (AdjustmentKind::Noise, AdjustmentParams::Noise(p)) => apply_noise(p, acc, win),
        (AdjustmentKind::Halftone, AdjustmentParams::Halftone(p)) => apply_halftone(p, acc, win),
        (AdjustmentKind::Bloom, AdjustmentParams::Bloom(p)) => apply_bloom(p, acc, win),
        (AdjustmentKind::ShadowsHighlights, AdjustmentParams::ShadowsHighlights(p)) => {
            apply_shadows_highlights(p, acc, win)
        }
        // Every other kind is per-pixel and position-independent.
        _ => super::compute::apply_adjustment(kind, params, acc),
    }
}

// ───────────────────────────── canonical kernels ─────────────────────────────

/// CANONICAL separable-Gaussian half-kernel. Returns `(weights, half)` where
/// `weights[i]` is the symmetric weight for offset `±i` (`weights[0]` = centre),
/// normalised so the full `2·half+1`-tap kernel sums to 1.
///
/// **σ = radius / 3** (the textbook 3σ truncation captures ≈ 99.7 % of the
/// Gaussian mass, so the kernel reaches exactly `half = ceil(radius)` texels). The
/// GPU pass-graph's placeholder used the identical mapping, so adopting it here is
/// a **zero-diff reconciliation** (the parity gate stays green) while moving the
/// math to its single canonical home.
#[must_use]
pub fn gaussian_weights(radius: f32) -> (Vec<f32>, u32) {
    let r = radius.max(0.0);
    let half = (r.ceil() as u32).clamp(1, MAX_BLUR_HALF);
    let sigma = (r / 3.0).max(1e-3);
    let two_sigma_sq = 2.0 * sigma * sigma;
    let mut weights = Vec::with_capacity(half as usize + 1);
    let mut sum = 0.0f32;
    for i in 0..=half {
        let x = i as f32;
        let g = (-(x * x) / two_sigma_sq).det_exp();
        weights.push(g);
        // Centre tap counts once; each ±i flank tap counts twice.
        sum += if i == 0 { g } else { 2.0 * g };
    }
    if sum > 0.0 {
        for w in &mut weights {
            *w /= sum;
        }
    }
    (weights, half)
}

/// CANONICAL motion (directional) kernel — a uniform box along the motion line.
/// Constant-velocity linear motion exposes every position on the line equally, so
/// a box is the exact model. Returns `(weights, half)` for the `2·half+1`-tap
/// average (`weights[i] = 1/(2·half+1)`); `half = ceil(distance/2)` so the line
/// spans ≈ `distance` px. Zero-diff with the GPU spike's `motion_weights`.
#[must_use]
pub fn motion_weights(distance: f32) -> (Vec<f32>, u32) {
    let half = ((distance.max(0.0) / 2.0).ceil() as u32).clamp(1, MAX_BLUR_HALF);
    let w = 1.0 / (2 * half + 1) as f32;
    (vec![w; half as usize + 1], half)
}

// ───────────────────────────── spatial kernels ───────────────────────────────

/// Read `(x, y)` from a `w × h` linear-RGBA window with clamp-to-edge.
#[inline]
fn sample_clamp(buf: &[[f32; 4]], w: i32, h: i32, x: i32, y: i32) -> [f32; 4] {
    let xc = x.clamp(0, w - 1);
    let yc = y.clamp(0, h - 1);
    buf[(yc * w + xc) as usize]
}

/// The blurs' boundary (ADR-0177 P4): encoded straight → premultiplied ENCODED,
/// in place. The kernels run in display tones like Krita at 8 bits (its Gaussian
/// of radius R is ours to the byte — doc Painter 45 §8, P4). They MUST run
/// premultiplied: a transparent texel then contributes `(0,0,0,0)` — not a
/// stale/black colour — at an alpha edge (no dark halo), and **coverage (alpha)
/// feathers outward with the colour** instead of staying clipped to the original
/// opaque silhouette.
#[inline]
fn premultiply(buf: &mut [[f32; 4]]) {
    par_pixels(buf, |px| {
        let a = px[3].clamp(0.0, 1.0);
        *px = [px[0] * a, px[1] * a, px[2] * a, a];
    });
}

/// Inverse of [`premultiply`] (premultiplied encoded → encoded straight). A fully
/// feathered-out (near-zero alpha) texel collapses to transparent black.
#[inline]
fn unpremultiply(buf: &mut [[f32; 4]]) {
    unpremultiply_with(buf, |v| v.clamp(0.0, 1.0));
}

/// Bloom's boundary: it is optical (light added), so it premultiplies in LIGHT.
#[inline]
fn premultiply_em_luz(buf: &mut [[f32; 4]]) {
    par_pixels(buf, |px| {
        let a = px[3].clamp(0.0, 1.0);
        let l = em_luz(px);
        *px = [l[0] * a, l[1] * a, l[2] * a, a];
    });
}

/// Inverse of [`premultiply_em_luz`] (premultiplied light → encoded straight).
#[inline]
fn unpremultiply_em_luz(buf: &mut [[f32; 4]]) {
    unpremultiply_with(buf, linear_to_srgb_f32);
}

#[inline]
fn unpremultiply_with(buf: &mut [[f32; 4]], encode: impl Fn(f32) -> f32 + Sync) {
    par_pixels(buf, |px| {
        let a = px[3].clamp(0.0, 1.0);
        if a > 1e-6 {
            px[0] = encode(px[0] / a);
            px[1] = encode(px[1] / a);
            px[2] = encode(px[2] / a);
            px[3] = a;
        } else {
            *px = [0.0, 0.0, 0.0, 0.0];
        }
    });
}

/// Separable Gaussian blur in **premultiplied** display tones — colour AND coverage
/// feather together, so a blurred layer's transparency spreads outward (soft
/// edges) instead of staying clipped to the opaque silhouette. The canonical CPU
/// reference + the CPU-fallback production kernel.
pub fn apply_gaussian(p: &GaussianBlurParams, acc: &mut [[f32; 4]], win: AdjustWindow) {
    if p.radius <= 0.0 {
        return;
    }
    premultiply(acc);
    separable_blur_premul(p.radius, acc, win);
    unpremultiply(acc);
}

/// Separable Gaussian blur of ALL 4 channels in place. The buffer MUST already be
/// premultiplied ([`premultiply`]); the output stays premultiplied. Shared by
/// Gaussian + the blur stage of Sharpen.
/// Compute the `h` rows of a `w`-wide output buffer, splitting them across the
/// available CPUs on the rayon pool (B19.7, ADR-0202). `row(y, out_row)` fills one
/// output row from shared input it captures — each thread owns a disjoint
/// `&mut` band, so it is data-race-free, and the per-pixel work is independent of
/// thread count, so the result is BIT-IDENTICAL regardless of CPU count (keeps
/// determinism + GPU parity). Small buffers run serially (thread setup not worth
/// it). This is the lever that takes the CPU-fallback kernels to hardware max.
fn par_rows<T, F>(out: &mut [T], w: usize, h: usize, row: F)
where
    T: Send,
    F: Fn(usize, &mut [T]) + Sync,
{
    let threads = std::thread::available_parallelism()
        .map(|n| n.get())
        .unwrap_or(1)
        .min(h.max(1));
    if threads <= 1 || w * h < 1 << 14 {
        for (y, r) in out.chunks_mut(w.max(1)).enumerate() {
            row(y, r);
        }
        return;
    }
    use rayon::prelude::*;
    let rows_per = h.div_ceil(threads);
    out.par_chunks_mut(rows_per * w)
        .enumerate()
        .for_each(|(band_idx, band)| {
            let y0 = band_idx * rows_per;
            for (i, r) in band.chunks_mut(w).enumerate() {
                row(y0 + i, r);
            }
        });
}

/// Parallel in-place map over a flat buffer for COORDINATE-INDEPENDENT per-pixel
/// ops (each element transformed from itself only). On the rayon pool, bit-identical
/// (element order irrelevant), serial for small buffers. (Siblings — e.g. the
/// Color Lookup grade in `lut` — reuse this; hence `pub(super)`.)
pub(super) fn par_pixels<T, F>(buf: &mut [T], f: F)
where
    T: Send,
    F: Fn(&mut T) + Sync,
{
    let n = buf.len();
    let threads = std::thread::available_parallelism()
        .map(|t| t.get())
        .unwrap_or(1)
        .min(n.max(1));
    if threads <= 1 || n < 1 << 14 {
        buf.iter_mut().for_each(&f);
        return;
    }
    use rayon::prelude::*;
    buf.par_chunks_mut(n.div_ceil(threads))
        .for_each(|band| band.iter_mut().for_each(&f));
}

fn separable_blur_premul(radius: f32, acc: &mut [[f32; 4]], win: AdjustWindow) {
    let (w, h) = (win.width as i32, win.height as i32);
    if w == 0 || h == 0 {
        return;
    }
    let (weights, half) = gaussian_weights(radius);
    let half = half as i32;
    let (wu, hu) = (w as usize, h as usize);
    // Horizontal pass: read `acc` → write `tmp`; the interior reads without a clamp.
    let mut tmp = vec![[0.0f32; 4]; acc.len()];
    {
        let src: &[[f32; 4]] = acc;
        par_rows(&mut tmp, wu, hu, |y, out_row| {
            let row = &src[y * wu..(y + 1) * wu];
            for (x, o) in out_row.iter_mut().enumerate() {
                let xi = x as i32;
                let mut c = [0.0f32; 4];
                if xi >= half && xi + half < w {
                    let lo = x - half as usize;
                    for (j, s) in row[lo..=x + half as usize].iter().enumerate() {
                        let wgt = weights[(j as i32 - half).unsigned_abs() as usize];
                        for ch in 0..4 {
                            c[ch] += s[ch] * wgt;
                        }
                    }
                } else {
                    for k in -half..=half {
                        let s = row[(xi + k).clamp(0, w - 1) as usize];
                        let wgt = weights[k.unsigned_abs() as usize];
                        for ch in 0..4 {
                            c[ch] += s[ch] * wgt;
                        }
                    }
                }
                *o = c;
            }
        });
    }
    // Vertical pass: read `tmp` → write `acc`, tap `k` outside and `x` inside (a whole source
    // row per tap); each pixel still sums its taps in `k` order, so the bits are the per-pixel loop's.
    par_rows(acc, wu, hu, |y, out_row| {
        out_row.fill([0.0; 4]);
        for k in -half..=half {
            let r = (y as i32 + k).clamp(0, h - 1) as usize;
            let wgt = weights[k.unsigned_abs() as usize];
            for (o, s) in out_row.iter_mut().zip(&tmp[r * wu..(r + 1) * wu]) {
                for ch in 0..4 {
                    o[ch] += s[ch] * wgt;
                }
            }
        }
    });
}

/// Unsharp-mask sharpen in premultiplied display tones: `out = base + amount·(base −
/// blur(base))` across ALL 4 channels, so the alpha edge sharpens with the colour
/// (and stays alpha-correct). `mask_edges` is DEFERRED (a future high-gradient
/// gate — semantics noted to the Coord). Negatives clamp at 0; encode clamps top.
pub fn apply_sharpen(p: &SharpenParams, acc: &mut [[f32; 4]], win: AdjustWindow) {
    if p.amount == 0.0 || p.radius <= 0.0 {
        return;
    }
    premultiply(acc);
    let base = acc.to_vec(); // premultiplied base
    separable_blur_premul(p.radius, acc, win); // acc = blur(base), premultiplied
    for (out, b) in acc.iter_mut().zip(base.iter()) {
        for c in 0..4 {
            out[c] = (b[c] + p.amount * (b[c] - out[c])).max(0.0);
        }
        out[3] = out[3].min(1.0); // coverage stays in [0, 1] after the overshoot
    }
    unpremultiply(acc);
}

/// Directional (motion) blur — a uniform box of `2·half+1` nearest taps along
/// `angle` (radians), in **premultiplied** display tones (colour + coverage smear).
pub fn apply_motion_blur(p: &MotionBlurParams, acc: &mut [[f32; 4]], win: AdjustWindow) {
    if p.distance <= 0.0 {
        return;
    }
    let (w, h) = (win.width as i32, win.height as i32);
    if w == 0 || h == 0 {
        return;
    }
    premultiply(acc);
    let (weights, half) = motion_weights(p.distance);
    let wgt = weights[0]; // uniform box
    let half = half as i32;
    let (sin, cos) = p.angle.det_sin_cos(); // direction (cos, sin)
    let taps: Vec<(i32, i32)> = (-half..=half)
        .map(|k| {
            (
                (k as f32 * cos).round() as i32,
                (k as f32 * sin).round() as i32,
            )
        })
        .collect();
    let src = acc.to_vec(); // premultiplied
    let (wu, hu) = (w as usize, h as usize);
    // Each output row reads only `src`; taps outside, `x` inside, summed in `k` order per pixel.
    par_rows(acc, wu, hu, |y, out_row| {
        out_row.fill([0.0; 4]);
        for &(ox, oy) in &taps {
            let r = &src[(y as i32 + oy).clamp(0, h - 1) as usize * wu..][..wu];
            for (x, o) in out_row.iter_mut().enumerate() {
                let s = r[(x as i32 + ox).clamp(0, w - 1) as usize];
                for ch in 0..4 {
                    o[ch] += s[ch] * wgt;
                }
            }
        }
    });
    unpremultiply(acc);
}

/// Chromatic aberration — a per-channel radial GATHER, in **premultiplied** display tones, on the
/// GPU `cs_chroma` model: centre = the CANVAS centre `(cw/2, ch/2)`, `scale_c = shift_c / half_diag`
/// (half the canvas diagonal), channel `c` of canvas pixel `p` reads
/// `floor(p − (p − centre)·scale_c + ½)` (inward gather: a positive shift fringes outward), clamped to the window. Computed in
/// ABSOLUTE canvas coordinates, so a dirty window is the crop of the whole canvas (its halo is
/// [`window_halo`]). Alpha is gathered UNSHIFTED (the lens shifts colour, not coverage).
/// `falloff_center` is RESERVED. Premultiplied, a transparent texel contributes nothing (no
/// transparent-edge speckle).
pub fn apply_chromatic_aberration(
    p: &ChromaticAberrationParams,
    acc: &mut [[f32; 4]],
    win: AdjustWindow,
) {
    if p.red_shift == 0.0 && p.green_shift == 0.0 && p.blue_shift == 0.0 {
        return;
    }
    let (w, h) = (win.width as i32, win.height as i32);
    if w == 0 || h == 0 {
        return;
    }
    premultiply(acc);
    let (cw, ch) = (win.canvas_width as f32, win.canvas_height as f32);
    let half_diag = 0.5 * (cw * cw + ch * ch).sqrt();
    let inv = if half_diag > 0.0 {
        1.0 / half_diag
    } else {
        0.0
    };
    let (cx, cy) = (cw * 0.5, ch * 0.5);
    let scale = [p.red_shift * inv, p.green_shift * inv, p.blue_shift * inv];
    let (ox, oy) = (win.origin_x as i32, win.origin_y as i32);
    let src = acc.to_vec(); // premultiplied
    let src = &src[..];
    par_rows(acc, w as usize, h as usize, |y, out_row| {
        let y = y as i32;
        let gy = (oy + y) as f32;
        let dy = gy - cy;
        for (x, o) in out_row.iter_mut().enumerate() {
            let x = x as i32;
            let gx = (ox + x) as f32;
            let dx = gx - cx;
            let tap = |sc: f32| {
                let sx = (gx - dx * sc + 0.5).floor() as i32 - ox;
                let sy = (gy - dy * sc + 0.5).floor() as i32 - oy;
                sample_clamp(src, w, h, sx, sy)
            };
            let i = (y * w + x) as usize;
            *o = [
                tap(scale[0])[0],
                tap(scale[1])[1],
                tap(scale[2])[2],
                src[i][3],
            ];
        }
    });
    unpremultiply(acc);
}

// ──────────────────────── coordinate-dependent per-pixel ──────────────────────

/// Display-space amplitude of `amount = 1` noise (`±NOISE_SCALE` at the extreme).
const NOISE_SCALE: f32 = 0.5;

/// Integer avalanche hash (Murmur-style finaliser) — deterministic, well-mixed.
#[inline]
fn hash_u32(mut x: u32) -> u32 {
    x ^= x >> 16;
    x = x.wrapping_mul(0x7feb_352d);
    x ^= x >> 15;
    x = x.wrapping_mul(0x846c_a68b);
    x ^= x >> 16;
    x
}

/// Deterministic `0..=1` from an absolute canvas coordinate + a per-channel salt.
#[inline]
fn rand01(x: u32, y: u32, salt: u32) -> f32 {
    let h = hash_u32(x ^ hash_u32(y ^ hash_u32(salt.wrapping_add(0x9E37_79B9))));
    h as f32 / u32::MAX as f32
}

/// One noise sample in `[-1, 1]`, deterministic per `(x, y, channel)`. `Uniform`
/// is a single hash; `Gaussian` is an Irwin–Hall sum of 4 hashes (≈ normal,
/// rescaled to `[-1, 1]`).
#[inline]
fn noise_value(x: u32, y: u32, channel: u32, kind: NoiseKind) -> f32 {
    match kind {
        NoiseKind::Uniform => 2.0 * rand01(x, y, channel) - 1.0,
        NoiseKind::Gaussian => {
            let base = channel * 4;
            let sum = rand01(x, y, base)
                + rand01(x, y, base + 1)
                + rand01(x, y, base + 2)
                + rand01(x, y, base + 3);
            // Irwin–Hall(4) has mean 2, range 0..4 → centre & scale to [-1, 1].
            (sum - 2.0) * 0.5
        }
    }
}

/// Add film grain in display space (perceptually even across tones). Deterministic
/// per absolute canvas coordinate → dirty-rect exact. `monochromatic` adds the
/// same value to R/G/B; otherwise each channel gets independent noise.
pub fn apply_noise(p: &NoiseParams, acc: &mut [[f32; 4]], win: AdjustWindow) {
    if p.amount <= 0.0 {
        return;
    }
    let (wu, hu) = (win.width as usize, win.height as usize);
    par_rows(acc, wu, hu, |y, out_row| {
        let gy = win.origin_y + y as u32;
        for (x, px) in out_row.iter_mut().enumerate() {
            let gx = win.origin_x + x as u32;
            let n0 = noise_value(gx, gy, 0, p.kind);
            let (n1, n2) = if p.monochromatic {
                (n0, n0)
            } else {
                (
                    noise_value(gx, gy, 1, p.kind),
                    noise_value(gx, gy, 2, p.kind),
                )
            };
            for (c, n) in [n0, n1, n2].into_iter().enumerate() {
                px[c] = (px[c] + p.amount * n * NOISE_SCALE).clamp(0.0, 1.0);
            }
        }
    });
}

/// Rec.709 display luma of an encoded pixel (the values as they are).
#[inline]
fn display_luma(px: &[f32; 4]) -> f32 {
    0.2126 * px[0].clamp(0.0, 1.0) + 0.7152 * px[1].clamp(0.0, 1.0) + 0.0722 * px[2].clamp(0.0, 1.0)
}

#[inline]
fn fract(v: f32) -> f32 {
    v - v.floor()
}

/// Halftone screen — convert the layer to a black-on-white ink pattern whose dot
/// size encodes luma (dark → larger ink), on a `dot_size`-px grid rotated by
/// `angle`. Deterministic per absolute coordinate → dirty-rect exact. Alpha kept.
pub fn apply_halftone(p: &HalftoneParams, acc: &mut [[f32; 4]], win: AdjustWindow) {
    let cell = p.dot_size.max(1.0);
    let (sin, cos) = p.angle.det_sin_cos();
    let (white, black) = (1.0, 0.0);
    let (wu, hu) = (win.width as usize, win.height as usize);
    par_rows(acc, wu, hu, |y, out_row| {
        let gy = (win.origin_y + y as u32) as f32;
        for (x, px) in out_row.iter_mut().enumerate() {
            let gx = (win.origin_x + x as u32) as f32;
            let l = display_luma(px);
            // dark = more ink. A coverage below half-a-pixel of dot reads as clean
            // paper — this also absorbs the f32 luma of "pure white" not being
            // bit-exactly 1.0 (which would otherwise ink a 1-px dot at each cell
            // centre, since the centre sits exactly at `dist == 0`).
            let coverage = 1.0 - l;
            if coverage * cell < 0.5 {
                px[0] = white;
                px[1] = white;
                px[2] = white;
                continue;
            }
            // Rotate into screen space, fold into the cell, centre at the origin.
            let u = gx * cos + gy * sin;
            let v = -gx * sin + gy * cos;
            let fu = fract(u / cell) - 0.5;
            let fv = fract(v / cell) - 0.5;
            let ink = match p.shape {
                // Disc grows from the cell centre (corner reach = 0.5·√2).
                HalftoneShape::Dot => {
                    (fu * fu + fv * fv).sqrt() < coverage * 0.5 * std::f32::consts::SQRT_2
                }
                // Parallel bars thicken with darkness.
                HalftoneShape::Line => fu.abs() < coverage * 0.5,
                // Concentric rings.
                HalftoneShape::Circle => fract(2.0 * (fu * fu + fv * fv).sqrt()) < coverage,
            };
            let g = if ink { black } else { white };
            px[0] = g;
            px[1] = g;
            px[2] = g;
        }
    });
}
