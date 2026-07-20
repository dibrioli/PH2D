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
//!    [`motion_weights`] against the GPU. **Exception — Chromatic Aberration is
//!    NOT a single source of truth (audit 2026-06-18):** the CPU
//!    [`apply_chromatic_aberration`] fallback DIVERGES from the GPU `cs_chroma` in
//!    the displacement SIGN (outward vs inward), the centre (`(w-1)/2` vs `w/2`)
//!    and the normaliser (`corner` vs `half_diag`), and NO gate reconciles them.
//!    It is latent only because that CPU fallback has no production caller yet — a
//!    W-spatial enable must pick ONE model and gate it before trusting either.
//! 2. **CPU fallback** when a stack is not GPU-representable (per-layer mask /
//!    clipping / reference layer): [`apply_adjustment_windowed`] runs the kernel
//!    on the compositor's linear f32 window.
//!
//! All inputs/outputs are **straight, LINEAR f32 RGBA** (the compositor's blend
//! space), matching [`apply_adjustment`](super::apply_adjustment). The spatial
//! blurs/gathers run internally in **premultiplied** space (see [`premultiply`]),
//! so colour AND coverage feather together and transparent texels contribute
//! nothing — fixing both the clipped-to-silhouette blur and the transparent-edge
//! colour speckle. The compositor combine then uses the kernel's feathered alpha
//! for spatial kinds (see `compositor::compose`). Noise / Halftone are defined in
//! display (sRGB) space, round-trip internally, and DO preserve coverage.
//!
//! ### Documented limitations (mirror the GPU pass-graph's own, handoff §4)
//! - **Spatial blur on a sub-window:** the kernels sample with clamp-to-edge at
//!   the window border, so a blur run on a dirty-rect *sub*-window is approximate
//!   at that border (the halo outside the rect is unavailable on the CPU). The
//!   real-time GPU path expands the work region by the blur halo and is exact; the
//!   common CPU case — a full-canvas recompose on an adjustment-param drag
//!   (`composite_with_cache`) — is also exact (window == canvas). Noise / Halftone
//!   are pure functions of the *absolute* canvas coordinate, so they are dirty-rect
//!   exact (`composite_region` == crop of `composite`).
//! - **Sampling is nearest-tap** for Motion / Chromatic Aberration. Bilinear taps
//!   are a quality refinement to land jointly with the GPU (`cs_blur_dir` /
//!   `cs_chroma`).
//! - **Sharpen `mask_edges`** is a deferred follow-up (semantics noted to the
//!   Coord). The GPU pass-graph must premultiply the materialised below-composite
//!   to match (the visible path — handoff to Coord).

use super::compute::{linear_to_srgb_f32, srgb_to_linear_f32};
use super::*;

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
        }
    }
}

/// Apply any adjustment `kind` to the compositor's linear f32 window, with the
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
        let g = (-(x * x) / two_sigma_sq).exp();
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

/// Premultiply RGB by alpha in place (straight → premultiplied linear). The blurs
/// MUST run premultiplied: a transparent texel then contributes `(0,0,0,0)` — not
/// a stale/black colour — at an alpha edge (no dark halo), and **coverage (alpha)
/// feathers outward with the colour** instead of staying clipped to the original
/// opaque silhouette. (The straight-RGB blur the spike shipped left the alpha
/// untouched, so a blurred transparent layer kept a hard edge — Enio's report.)
#[inline]
fn premultiply(buf: &mut [[f32; 4]]) {
    for px in buf.iter_mut() {
        let a = px[3].clamp(0.0, 1.0);
        px[0] *= a;
        px[1] *= a;
        px[2] *= a;
        px[3] = a;
    }
}

/// Inverse of [`premultiply`] (premultiplied → straight). A fully feathered-out
/// (near-zero alpha) texel collapses to transparent black.
#[inline]
fn unpremultiply(buf: &mut [[f32; 4]]) {
    for px in buf.iter_mut() {
        let a = px[3].clamp(0.0, 1.0);
        if a > 1e-6 {
            px[0] /= a;
            px[1] /= a;
            px[2] /= a;
            px[3] = a;
        } else {
            *px = [0.0, 0.0, 0.0, 0.0];
        }
    }
}

/// Separable Gaussian blur in **premultiplied** linear RGBA — colour AND coverage
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
/// available CPUs via SCOPED THREADS (no new deps). `row(y, out_row)` fills one
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
    let rows_per = h.div_ceil(threads);
    std::thread::scope(|s| {
        for (band_idx, band) in out.chunks_mut(rows_per * w).enumerate() {
            let row = &row;
            let y0 = band_idx * rows_per;
            s.spawn(move || {
                for (i, r) in band.chunks_mut(w).enumerate() {
                    row(y0 + i, r);
                }
            });
        }
    });
}

/// Parallel in-place map over a flat buffer for COORDINATE-INDEPENDENT per-pixel
/// ops (each element transformed from itself only). Scoped threads, bit-identical
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
    let chunk = n.div_ceil(threads);
    std::thread::scope(|s| {
        for band in buf.chunks_mut(chunk) {
            let f = &f;
            s.spawn(move || band.iter_mut().for_each(f));
        }
    });
}

fn separable_blur_premul(radius: f32, acc: &mut [[f32; 4]], win: AdjustWindow) {
    let (w, h) = (win.width as i32, win.height as i32);
    if w == 0 || h == 0 {
        return;
    }
    let (weights, half) = gaussian_weights(radius);
    let half = half as i32;
    let (wu, hu) = (w as usize, h as usize);
    // Horizontal pass: read `acc` → write `tmp` (rows in parallel).
    let mut tmp = vec![[0.0f32; 4]; acc.len()];
    {
        let src: &[[f32; 4]] = acc;
        par_rows(&mut tmp, wu, hu, |y, out_row| {
            let y = y as i32;
            for (x, o) in out_row.iter_mut().enumerate() {
                let mut c = [0.0f32; 4];
                for k in -half..=half {
                    let s = sample_clamp(src, w, h, x as i32 + k, y);
                    let wgt = weights[k.unsigned_abs() as usize];
                    for ch in 0..4 {
                        c[ch] += s[ch] * wgt;
                    }
                }
                *o = c;
            }
        });
    }
    // Vertical pass: read `tmp` → write `acc` (rows in parallel).
    par_rows(acc, wu, hu, |y, out_row| {
        let y = y as i32;
        for (x, o) in out_row.iter_mut().enumerate() {
            let mut c = [0.0f32; 4];
            for k in -half..=half {
                let s = sample_clamp(&tmp, w, h, x as i32, y + k);
                let wgt = weights[k.unsigned_abs() as usize];
                for ch in 0..4 {
                    c[ch] += s[ch] * wgt;
                }
            }
            *o = c;
        }
    });
}

/// Unsharp-mask sharpen in premultiplied linear: `out = base + amount·(base −
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
/// `angle` (radians), in **premultiplied** linear (colour + coverage smear).
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
    let (sin, cos) = p.angle.sin_cos(); // direction (cos, sin)
    let src = acc.to_vec(); // premultiplied
    for y in 0..h {
        for x in 0..w {
            let mut c = [0.0f32; 4];
            for k in -half..=half {
                let ox = (k as f32 * cos).round() as i32;
                let oy = (k as f32 * sin).round() as i32;
                let s = sample_clamp(&src, w, h, x + ox, y + oy);
                for ch in 0..4 {
                    c[ch] += s[ch] * wgt;
                }
            }
            acc[(y * w + x) as usize] = c;
        }
    }
    unpremultiply(acc);
}

/// Chromatic aberration — a per-channel radial GATHER, in **premultiplied** linear
/// RGBA. Each output channel `c` samples the below-composite at `centre + (p −
/// centre)·scale_c`, where `scale_c = 1 + shift_c / corner_dist` (so `shift_c` px
/// of fringe at the canvas corner, 0 at the centre — the linear-radial model).
/// Alpha is gathered UNSHIFTED (the lens shifts colour channels, not the overall
/// coverage). The per-channel scales + centre are computed once (no per-pixel
/// `sqrt`). `falloff_center` is RESERVED. Nearest-tap (bilinear is a follow-up).
///
/// Premultiplication is what kills the **transparent-edge speckle** (Enio's blue
/// dots): a transparent texel carries arbitrary stale RGB in straight space, so a
/// shifted blue gather sprayed that colour into visible pixels. Premultiplied,
/// every transparent texel is `(0,0,0,0)` → it contributes nothing to the gather.
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
    let cx = (w as f32 - 1.0) * 0.5;
    let cy = (h as f32 - 1.0) * 0.5;
    let corner = (cx * cx + cy * cy).sqrt().max(1e-3);
    let scale = |shift: f32| 1.0 + shift / corner;
    let (sr, sg, sb) = (
        scale(p.red_shift),
        scale(p.green_shift),
        scale(p.blue_shift),
    );
    let src = acc.to_vec(); // premultiplied
    let gather = |dx: f32, dy: f32, s: f32, ch: usize| -> f32 {
        let sx = (cx + dx * s).round() as i32;
        let sy = (cy + dy * s).round() as i32;
        sample_clamp(&src, w, h, sx, sy)[ch]
    };
    for y in 0..h {
        for x in 0..w {
            let dx = x as f32 - cx;
            let dy = y as f32 - cy;
            let o = &mut acc[(y * w + x) as usize];
            o[0] = gather(dx, dy, sr, 0);
            o[1] = gather(dx, dy, sg, 1);
            o[2] = gather(dx, dy, sb, 2);
            o[3] = gather(dx, dy, 1.0, 3); // coverage: no shift
        }
    }
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
                let d = linear_to_srgb_f32(px[c]);
                let v = (d + p.amount * n * NOISE_SCALE).clamp(0.0, 1.0);
                px[c] = srgb_to_linear_f32(v);
            }
        }
    });
}

/// Rec.709 display luma of a linear pixel.
#[inline]
fn display_luma(px: &[f32; 4]) -> f32 {
    let r = linear_to_srgb_f32(px[0]);
    let g = linear_to_srgb_f32(px[1]);
    let b = linear_to_srgb_f32(px[2]);
    0.2126 * r + 0.7152 * g + 0.0722 * b
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
    let (sin, cos) = p.angle.sin_cos();
    let white = srgb_to_linear_f32(1.0);
    let black = srgb_to_linear_f32(0.0);
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

// ──────────────────────────── Bloom + Shadows/Highlights ──────────────────────

/// Smooth Hermite interpolation: 0 below `e0`, 1 above `e1`, an S-curve between.
#[inline]
fn smoothstep(e0: f32, e1: f32, x: f32) -> f32 {
    let t = ((x - e0) / (e1 - e0)).clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

/// Separable Gaussian blur of a SCALAR field (a tone map) in place, clamp-to-edge.
/// Used by Shadows/Highlights to build the local-average luma. `radius ≤ 0` leaves
/// the field unblurred (a global, per-pixel tone reference).
fn separable_blur_scalar(radius: f32, field: &mut [f32], win: AdjustWindow) {
    if radius <= 0.0 {
        return;
    }
    let (w, h) = (win.width as i32, win.height as i32);
    if w == 0 || h == 0 {
        return;
    }
    let (weights, half) = gaussian_weights(radius);
    let half = half as i32;
    let (wu, hu) = (w as usize, h as usize);
    let tap = |buf: &[f32], x: i32, y: i32| -> f32 {
        buf[(y.clamp(0, h - 1) * w + x.clamp(0, w - 1)) as usize]
    };
    // Horizontal pass: read `field` → write `tmp` (rows in parallel).
    let mut tmp = vec![0.0f32; field.len()];
    {
        let src: &[f32] = field;
        par_rows(&mut tmp, wu, hu, |y, out_row| {
            let y = y as i32;
            for (x, o) in out_row.iter_mut().enumerate() {
                let mut s = 0.0;
                for k in -half..=half {
                    s += tap(src, x as i32 + k, y) * weights[k.unsigned_abs() as usize];
                }
                *o = s;
            }
        });
    }
    // Vertical pass: read `tmp` → write `field`.
    par_rows(field, wu, hu, |y, out_row| {
        let y = y as i32;
        for (x, o) in out_row.iter_mut().enumerate() {
            let mut s = 0.0;
            for k in -half..=half {
                s += tap(&tmp, x as i32, y + k) * weights[k.unsigned_abs() as usize];
            }
            *o = s;
        }
    });
}

/// Bloom — bright-pass → blur → additive glow, in **premultiplied** linear. Pixels
/// whose display luma is above `threshold` (softened by the `falloff` knee) are
/// extracted as the bright EXCESS, blurred by `radius`, and added back scaled by
/// `intensity`. The glow is premultiplied, so it carries coverage and **haloes
/// outward** past the bright source into transparency (the bloom look). Bloom is a
/// coverage-feathering kind (see `AdjustmentKind::feathers_coverage`).
pub fn apply_bloom(p: &BloomParams, acc: &mut [[f32; 4]], win: AdjustWindow) {
    if p.intensity <= 0.0 || p.radius <= 0.0 {
        return;
    }
    let (w, h) = (win.width as i32, win.height as i32);
    if w == 0 || h == 0 {
        return;
    }
    let knee = p.falloff.max(1e-3);
    let (wu, hu) = (w as usize, h as usize);
    // Bright-pass at full res (premultiplied: `color·alpha·weight`; transparent → 0).
    let mut bright = vec![[0.0f32; 4]; acc.len()];
    {
        let src: &[[f32; 4]] = acc;
        par_rows(&mut bright, wu, hu, |y, out_row| {
            for (x, g) in out_row.iter_mut().enumerate() {
                let base = src[y * wu + x];
                let w_bright = smoothstep(p.threshold, p.threshold + knee, display_luma(&base));
                let k = base[3].clamp(0.0, 1.0) * w_bright;
                *g = [base[0] * k, base[1] * k, base[2] * k, k];
            }
        });
    }
    // PERF: the glow is low-frequency, so blur it at REDUCED resolution — the
    // standard bloom trick + a mirror of the GPU mip pyramid. Downsample the
    // bright-pass by `factor`, blur the small buffer (radius scaled down → far
    // fewer taps over far fewer pixels: ≈ factor⁴ less work), bilinear-upsample on
    // the add. Large canvases pick a bigger factor; small ones stay full-res so the
    // glow shape is unchanged. This is what fixes the slider-drag FPS on the CPU
    // fallback path (the GPU pass-graph is the Coord's follow-up).
    // RADIUS-based factor (mirror of the GPU) → the low-res blur is bounded, so the
    // cost is radius-independent AND the CPU fallback matches the GPU at every radius.
    let factor = bloom_downsample_factor(p.radius) as i32;
    let glow = if factor == 1 {
        let mut g = bright;
        separable_blur_premul(p.radius, &mut g, win);
        g
    } else {
        let (sw, sh) = ((w + factor - 1) / factor, (h + factor - 1) / factor);
        let mut small = downsample_box(&bright, w, h, sw, sh, factor);
        separable_blur_premul(
            p.radius / factor as f32,
            &mut small,
            AdjustWindow::full(sw as u32, sh as u32),
        );
        small // upsampled on read below
    };
    // Add the glow onto the premultiplied base, then back to straight (parallel).
    premultiply(acc);
    let intensity = p.intensity;
    let glow = &glow;
    if factor == 1 {
        par_rows(acc, wu, hu, |y, out_row| {
            for (x, o) in out_row.iter_mut().enumerate() {
                let g = glow[y * wu + x];
                o[0] += intensity * g[0];
                o[1] += intensity * g[1];
                o[2] += intensity * g[2];
                o[3] = (o[3] + intensity * g[3]).clamp(0.0, 1.0);
            }
        });
    } else {
        let (sw, sh) = ((w + factor - 1) / factor, (h + factor - 1) / factor);
        // Centre-aligned map full → small using the actual dim ratio (sw/w, not
        // 1/factor) — matches the GPU `cs_bloom_up` exactly when w isn't a clean
        // multiple of `factor`.
        let inv_x = sw as f32 / w as f32;
        let inv_y = sh as f32 / h as f32;
        par_rows(acc, wu, hu, |y, out_row| {
            let fy = (y as f32 + 0.5) * inv_y - 0.5;
            for (x, o) in out_row.iter_mut().enumerate() {
                let fx = (x as f32 + 0.5) * inv_x - 0.5;
                let g = bilinear(glow, sw, sh, fx, fy);
                o[0] += intensity * g[0];
                o[1] += intensity * g[1];
                o[2] += intensity * g[2];
                o[3] = (o[3] + intensity * g[3]).clamp(0.0, 1.0);
            }
        });
    }
    unpremultiply(acc);
}

/// Box-downsample a `w×h` premultiplied buffer to `dw×dh` by averaging each
/// `factor×factor` source block (clamped at the edges). Premultiplied RGBA is
/// linear-combinable, so a plain average is correct.
fn downsample_box(
    src: &[[f32; 4]],
    w: i32,
    h: i32,
    dw: i32,
    dh: i32,
    factor: i32,
) -> Vec<[f32; 4]> {
    let mut dst = vec![[0.0f32; 4]; (dw * dh) as usize];
    for dy in 0..dh {
        for dx in 0..dw {
            let mut acc = [0.0f32; 4];
            let mut n = 0.0f32;
            for sy in 0..factor {
                for sx in 0..factor {
                    let x = (dx * factor + sx).min(w - 1);
                    let y = (dy * factor + sy).min(h - 1);
                    let s = src[(y * w + x) as usize];
                    for c in 0..4 {
                        acc[c] += s[c];
                    }
                    n += 1.0;
                }
            }
            let inv = 1.0 / n;
            dst[(dy * dw + dx) as usize] = [acc[0] * inv, acc[1] * inv, acc[2] * inv, acc[3] * inv];
        }
    }
    dst
}

/// Bilinear sample of a `w×h` buffer at fractional `(fx, fy)` with clamp-to-edge.
fn bilinear(buf: &[[f32; 4]], w: i32, h: i32, fx: f32, fy: f32) -> [f32; 4] {
    let x0 = fx.floor() as i32;
    let y0 = fy.floor() as i32;
    let tx = fx - x0 as f32;
    let ty = fy - y0 as f32;
    let at = |x: i32, y: i32| buf[(y.clamp(0, h - 1) * w + x.clamp(0, w - 1)) as usize];
    let c00 = at(x0, y0);
    let c10 = at(x0 + 1, y0);
    let c01 = at(x0, y0 + 1);
    let c11 = at(x0 + 1, y0 + 1);
    let mut out = [0.0f32; 4];
    for c in 0..4 {
        let a = c00[c] + (c10[c] - c00[c]) * tx;
        let b = c01[c] + (c11[c] - c01[c]) * tx;
        out[c] = a + (b - a) * ty;
    }
    out
}

/// Shadows/Highlights — LOCAL tonal correction. The display luma is blurred into a
/// local-average tone map (separate `*_radius` for each), so shadows lift /
/// highlights recover based on the NEIGHBOURHOOD tone — preserving local contrast,
/// unlike a global curve. `*_tonal_width` set how far into the range each reaches;
/// `midtone_contrast` is an S-curve around mid-grey; `color_correction` scales
/// saturation in the corrected regions. Coverage is PRESERVED (a tonal op, not an
/// image blur — `feathers_coverage` is false).
pub fn apply_shadows_highlights(
    p: &ShadowsHighlightsParams,
    acc: &mut [[f32; 4]],
    win: AdjustWindow,
) {
    if p.shadows_amount == 0.0 && p.highlights_amount == 0.0 && p.midtone_contrast == 0.0 {
        return;
    }
    // Local-average luma maps (one per correction radius).
    let mut local_lo: Vec<f32> = acc.iter().map(display_luma).collect();
    let mut local_hi = local_lo.clone();
    separable_blur_scalar(p.shadows_radius, &mut local_lo, win);
    separable_blur_scalar(p.highlights_radius, &mut local_hi, win);
    let tw_s = p.shadows_tonal_width.max(1e-3);
    let tw_h = p.highlights_tonal_width.max(1e-3);
    let (wu, hu) = (win.width as usize, win.height as usize);
    let (lo, hi) = (&local_lo, &local_hi);
    par_rows(acc, wu, hu, |y, out_row| {
        for (x, px) in out_row.iter_mut().enumerate() {
            let i = y * wu + x;
            let l = display_luma(px);
            // Membership from the LOCAL tone: deep-shadow / bright-highlight weights.
            let ws = 1.0 - smoothstep(0.0, tw_s, lo[i]);
            let wh = smoothstep(1.0 - tw_h, 1.0, hi[i]);
            let mut new_l = l + p.shadows_amount * ws - p.highlights_amount * wh;
            new_l = (0.5 + (new_l - 0.5) * (1.0 + p.midtone_contrast)).clamp(0.0, 1.0);
            // Re-tone in display space, preserving hue (scale toward new_l).
            let mut d = [
                linear_to_srgb_f32(px[0]),
                linear_to_srgb_f32(px[1]),
                linear_to_srgb_f32(px[2]),
            ];
            if l > 1e-4 {
                let ratio = new_l / l;
                for c in &mut d {
                    *c = (*c * ratio).clamp(0.0, 1.0);
                }
            } else {
                d = [new_l, new_l, new_l];
            }
            // Saturation tweak in the corrected regions.
            let cc = p.color_correction * (ws + wh).min(1.0);
            if cc != 0.0 {
                for c in &mut d {
                    *c = (new_l + (*c - new_l) * (1.0 + cc)).clamp(0.0, 1.0);
                }
            }
            px[0] = srgb_to_linear_f32(d[0]);
            px[1] = srgb_to_linear_f32(d[1]);
            px[2] = srgb_to_linear_f32(d[2]);
        }
    });
}
