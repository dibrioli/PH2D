//! **Bloom e Sombras/Realces** — os dois kernels de tom e brilho, com os ajudantes só deles
//! (`smoothstep`, o blur escalar, a redução em caixa e a amostra bilinear) —, irmão de `spatial.rs`
//! por tecto de LOC. O caminho não muda: `spatial.rs` re-exporta os dois `apply_*`.
//!
//! Corte mecânico: a secção saiu inteira, verbatim.

use super::*;

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
