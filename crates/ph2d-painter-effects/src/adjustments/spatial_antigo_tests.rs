//! B19b oráculo: os kernels de `spatial.rs`/`spatial_tonal.rs` de `0550d538a3` À LETRA (blocos
//! copiados por `sed -n`, sha256 dos dois ficheiros `e329c0e6…`/`59ddaedf…`): blur separável com
//! clamp por tap, Motion e CA seriais, `downsample_box` serial, S/H com dois borrões.
#![allow(dead_code, clippy::all)]

use super::super::*;
use ph2d_det_math::DetMath as _;

/// A única porta para o que era privado (o resto do ficheiro é a cópia).
pub(super) fn downsample_box_antigo(
    s: &[[f32; 4]],
    w: i32,
    h: i32,
    dw: i32,
    dh: i32,
    f: i32,
) -> Vec<[f32; 4]> {
    downsample_box(s, w, h, dw, dh, f)
}

/// Read `(x, y)` from a `w × h` linear-RGBA window with clamp-to-edge.
#[inline]
fn sample_clamp(buf: &[[f32; 4]], w: i32, h: i32, x: i32, y: i32) -> [f32; 4] {
    let xc = x.clamp(0, w - 1);
    let yc = y.clamp(0, h - 1);
    buf[(yc * w + xc) as usize]
}
#[inline]
fn premultiply(buf: &mut [[f32; 4]]) {
    for px in buf.iter_mut() {
        let a = px[3].clamp(0.0, 1.0);
        *px = [px[0] * a, px[1] * a, px[2] * a, a];
    }
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
    for px in buf.iter_mut() {
        let a = px[3].clamp(0.0, 1.0);
        let l = em_luz(px);
        *px = [l[0] * a, l[1] * a, l[2] * a, a];
    }
}

/// Inverse of [`premultiply_em_luz`] (premultiplied light → encoded straight).
#[inline]
fn unpremultiply_em_luz(buf: &mut [[f32; 4]]) {
    unpremultiply_with(buf, linear_to_srgb_f32);
}

#[inline]
fn unpremultiply_with(buf: &mut [[f32; 4]], encode: impl Fn(f32) -> f32) {
    for px in buf.iter_mut() {
        let a = px[3].clamp(0.0, 1.0);
        if a > 1e-6 {
            px[0] = encode(px[0] / a);
            px[1] = encode(px[1] / a);
            px[2] = encode(px[2] / a);
            px[3] = a;
        } else {
            *px = [0.0, 0.0, 0.0, 0.0];
        }
    }
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

/// Chromatic aberration — a per-channel radial GATHER, in **premultiplied** display tones, on the
/// GPU `cs_chroma` model: centre = the CANVAS centre `(cw/2, ch/2)`, `scale_c = shift_c / half_diag`
/// (half the canvas diagonal), channel `c` of canvas pixel `p` reads `floor(p − (p − centre)·scale_c
/// + ½)` (inward gather: a positive shift fringes outward), clamped to the window. Computed in
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
    for y in 0..h {
        let gy = (oy + y) as f32;
        let dy = gy - cy;
        for x in 0..w {
            let gx = (ox + x) as f32;
            let dx = gx - cx;
            let tap = |sc: f32| {
                let sx = (gx - dx * sc + 0.5).floor() as i32 - ox;
                let sy = (gy - dy * sc + 0.5).floor() as i32 - oy;
                sample_clamp(&src, w, h, sx, sy)
            };
            let i = (y * w + x) as usize;
            acc[i] = [
                tap(scale[0])[0],
                tap(scale[1])[1],
                tap(scale[2])[2],
                src[i][3],
            ];
        }
    }
    unpremultiply(acc);
}
/// Smooth Hermite interpolation: 0 below `e0`, 1 above `e1`, an S-curve between.
#[inline]
fn smoothstep(e0: f32, e1: f32, x: f32) -> f32 {
    let t = ((x - e0) / (e1 - e0)).clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

/// Separable Gaussian blur of a SCALAR field (a tone map) in place, clamp-to-edge.
/// Used by Shadows/Highlights to build the local-average luma. `radius ≤ 0` leaves
/// the field unblurred (a global, per-pixel tone reference).
pub fn separable_blur_scalar(radius: f32, field: &mut [f32], win: AdjustWindow) {
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
/// Bloom — bright-pass → blur → additive glow, in **premultiplied** light. Pixels
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
                let l = em_luz(&base); // the glow is light
                *g = [l[0] * k, l[1] * k, l[2] * k, k];
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
    premultiply_em_luz(acc);
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
    unpremultiply_em_luz(acc);
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

/// Display luma below which Shadows/Highlights stops scaling the colour (≈ 8 codes of 8-bit):
/// `new_l / l` on near-black noise turned it into a saturated primary (DF-50). Mirrored by the
/// WGSL `SH_LUMA_FLOOR` (`cs_combine_sh`).
pub const SH_LUMA_FLOOR: f32 = 1.0 / 32.0;

/// Shadows/Highlights — LOCAL tonal correction. The display luma is blurred into a
/// local-average tone map (separate `*_radius` for each), so shadows lift /
/// highlights recover based on the NEIGHBOURHOOD tone — preserving local contrast,
/// unlike a global curve. `*_tonal_width` set how far into the range each reaches;
/// `midtone_contrast` is an S-curve around mid-grey; `color_correction` scales
/// saturation in the corrected regions. Defined in display tones end to end (the
/// encoded values as they are). Coverage is PRESERVED (a tonal op, not an image
/// blur — `feathers_coverage` is false).
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
                px[0].clamp(0.0, 1.0),
                px[1].clamp(0.0, 1.0),
                px[2].clamp(0.0, 1.0),
            ];
            if l >= SH_LUMA_FLOOR {
                let ratio = new_l / l;
                for c in &mut d {
                    *c = (*c * ratio).clamp(0.0, 1.0);
                }
            } else {
                // B19.3: abaixo do piso a cor é ruído — escala até ao piso e soma o resto em cinza
                // (a luma chega a `new_l`; `l → 0` é o cinza de antes, `l → piso` a lei de cima).
                let ratio = new_l / SH_LUMA_FLOOR;
                let add = new_l - l * ratio;
                for c in &mut d {
                    *c = (*c * ratio + add).clamp(0.0, 1.0);
                }
            }
            // Saturation tweak in the corrected regions.
            let cc = p.color_correction * (ws + wh).min(1.0);
            if cc != 0.0 {
                for c in &mut d {
                    *c = (new_l + (*c - new_l) * (1.0 + cc)).clamp(0.0, 1.0);
                }
            }
            px[0] = d[0];
            px[1] = d[1];
            px[2] = d[2];
        }
    });
}
