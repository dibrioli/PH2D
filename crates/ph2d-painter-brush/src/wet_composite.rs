//! Wet-field composite — the CPU reference (parity ground truth) for compositing
//! the low-res live diffusion field over a backdrop into the canvas, with a
//! **Kubelka–Munk subtractive glaze** (ADR-0049 / ADR-0077 D12).
//!
//! This is the home of the per-pixel composite math that shipped inline in
//! `ph2d_tool_painter` (W15.2). It is hoisted here so it has ONE definition that
//! three callers share:
//!   1. the painter tool's CPU composite (`PainterTool::composite_wet_field`),
//!   2. the GPU port's parity test (`ph2d-painter-fluid`, which can't depend on
//!      the tool — it is a leaf the tool consumes), and
//!   3. the GPU shader itself, mirrored band-for-band.
//!
//! ## Why the EXACT (no-LUT) spectral mix here
//! [`mix_prepared_exact`] reconstructs the backdrop reflectance directly instead of
//! via the per-pixel trilinear LUT (the LUT is a CPU hot-path cache the GPU has no
//! use for — ADR-0049 §3). Using the exact mix on BOTH sides makes the CPU
//! reference and the GPU port agree to float precision, so the parity gate is tight
//! (the LUT's ~1% sub-grid residual would otherwise swamp it).
//!
//! The algorithm, per canvas pixel (scoped to the wet bbox by the caller):
//!   1. bicubic (Catmull-Rom) upsample of the low-res pigment → `p`;
//!   2. `dens = Σp`; bare paper (`< 1e-4`) writes the backdrop and skips;
//!   3. colour-independent coverage `alpha = 1 − exp(−(dens/color_sum)·K)`;
//!   4. straight-alpha glaze: K–M mix over opaque backdrop, porter-duff "over" at a
//!      transparent edge, lerped by the backdrop's own alpha (no black fringe).

use crate::pigment_mix::{PreparedPigment, mix_prepared_exact, prepare_pigment};
use ph2d_color::srgb::{linear_to_srgb_byte, srgb_to_linear_byte};

/// Catmull-Rom cubic weights for the four taps around a fractional position
/// `t ∈ [0,1)` (taps at offsets −1, 0, +1, +2). Sum to 1; interpolating + C1, so
/// upsampling a coarse field reads smooth instead of faceted.
#[inline]
#[must_use]
pub fn catmull_rom_weights(t: f32) -> [f32; 4] {
    let t2 = t * t;
    let t3 = t2 * t;
    [
        -0.5 * t3 + t2 - 0.5 * t,
        1.5 * t3 - 2.5 * t2 + 1.0,
        -1.5 * t3 + 2.0 * t2 + 0.5 * t,
        0.5 * t3 - 0.5 * t2,
    ]
}

/// Bicubic (Catmull-Rom) sample of the low-res pigment field at fractional grid
/// coords `(fx, fy)`, edges clamped. The cubic can overshoot, so pigment mass is
/// floored at 0 (negative mass is unphysical).
#[inline]
#[must_use]
pub fn sample_pigment_bicubic(pig: &[[f32; 3]], gw: u32, gh: u32, fx: f32, fy: f32) -> [f32; 3] {
    let x0 = fx.floor() as i32;
    let y0 = fy.floor() as i32;
    let wx = catmull_rom_weights(fx - x0 as f32);
    let wy = catmull_rom_weights(fy - y0 as f32);
    let cx = |x: i32| x.clamp(0, gw as i32 - 1) as u32;
    let cy = |y: i32| y.clamp(0, gh as i32 - 1) as u32;
    let mut out = [0.0f32; 3];
    for (j, &wyj) in wy.iter().enumerate() {
        let gy = cy(y0 - 1 + j as i32);
        let mut row = [0.0f32; 3];
        for (i, &wxi) in wx.iter().enumerate() {
            let p = pig[(gy * gw + cx(x0 - 1 + i as i32)) as usize];
            row[0] += p[0] * wxi;
            row[1] += p[1] * wxi;
            row[2] += p[2] * wxi;
        }
        out[0] += row[0] * wyj;
        out[1] += row[1] * wyj;
        out[2] += row[2] * wyj;
    }
    [out[0].max(0.0), out[1].max(0.0), out[2].max(0.0)]
}

/// Inclusive grid-cell bbox of cells carrying compositable pigment (total mass
/// ≥ the composite floor `1e-4`). `None` if the grid is bare. A cheap O(cells)
/// scan that scopes the composite to the wet region.
#[must_use]
pub fn wet_pigment_bbox(pig: &[[f32; 3]], gw: u32, gh: u32) -> Option<(u32, u32, u32, u32)> {
    let (mut x0, mut y0, mut x1, mut y1) = (u32::MAX, u32::MAX, 0u32, 0u32);
    let mut any = false;
    for gy in 0..gh {
        let row = (gy * gw) as usize;
        for gx in 0..gw {
            let p = pig[row + gx as usize];
            if p[0] + p[1] + p[2] >= 1.0e-4 {
                any = true;
                x0 = x0.min(gx);
                y0 = y0.min(gy);
                x1 = x1.max(gx);
                y1 = y1.max(gy);
            }
        }
    }
    any.then_some((x0, y0, x1, y1))
}

/// The amortised composite brush — computed ONCE per composite (a fluid stroke is
/// one colour). Ships straight to the GPU as a uniform.
pub struct WetCompositeBrush {
    /// K–M brush side (`color` + per-band `ks` + round-trip `err`).
    pub prepared: PreparedPigment,
    /// Pigment chromaticity = brush colour (linear sRGB). Equals `prepared.color()`.
    pub pcol: [f32; 3],
    /// Coverage normaliser = Σ(stroke colour linear), floored — recovers the
    /// colour-independent pigment `amount` from the grid's colour×amount mass.
    pub color_sum: f32,
}

/// Derive the amortised composite brush from the grid pigment + the stroke colour
/// (linear sRGB). The chromaticity comes from the grid's total pigment mass; the
/// coverage normaliser comes from the stroke colour (ADR-0077 D12 fix — bright
/// pigments no longer read fully opaque).
#[must_use]
pub fn prepare_wet_composite(
    _pigment: &[[f32; 3]],
    stroke_color_linear: [f32; 3],
) -> WetCompositeBrush {
    // **ADR-0079:** `pcol` now preserves the picked colour's VALUE (see
    // [`prepare_wet_composite_from_stroke`]). A fluid stroke is one colour, so the grid
    // pigment's chromaticity equals the stroke's — the grid is no longer needed to derive
    // the brush, and deriving `pcol` from the grid total (chromaticity-only) would DROP the
    // value, diverging from the live GPU path. So this delegates: both paths produce the
    // identical value-preserving brush (`stroke_derived_brush_matches_grid_derived`).
    prepare_wet_composite_from_stroke(stroke_color_linear)
}

/// Derive the composite brush from ONLY the stroke colour (linear sRGB) — for the
/// GPU resident path, where the bloomed pigment lives on the GPU and the CPU grid
/// holds just this frame's deposit. A fluid stroke is one colour, and the grid
/// pigment is `stroke_colour × amount` per cell (splat + the linear, per-channel
/// diffuse/advect keep the chromaticity uniform), so the chromaticity `pcol` the
/// grid-total derivation in [`prepare_wet_composite`] computes is exactly
/// `normalize(stroke_colour)` — the `Σamount` cancels. This computes the identical
/// `pcol`/`color_sum` without needing the pigment field.
#[must_use]
pub fn prepare_wet_composite_from_stroke(stroke_color_linear: [f32; 3]) -> WetCompositeBrush {
    // **ADR-0079 — preserve the picked colour's VALUE.** `pcol` is now the picked colour
    // itself (linear, clamped to the K–M reflectance range), NOT just its chromaticity, so a
    // dark red and a light red yield DIFFERENT washes (the wash converges to the picked
    // colour at full coverage). Opacity stays COVERAGE-driven — `color_sum` still normalises
    // the value out of the shader's `amount` (the ADR-0077 D12 fix: brightness must not read
    // as opacity), so only the pigment *colour* gains value, not the build-up rate. Earlier
    // this normalised the chromaticity (`/ssum·3`), discarding value (Enio 2026-06-08:
    // "tanto faz vermelho claro ou escuro").
    let pcol = [
        stroke_color_linear[0].clamp(0.0, 1.0),
        stroke_color_linear[1].clamp(0.0, 1.0),
        stroke_color_linear[2].clamp(0.0, 1.0),
    ];
    let prepared = prepare_pigment(pcol);
    // **ADR-0079:** the deposited pigment is now a colour-INDEPENDENT gray coverage mass
    // (the lifecycle splat deposits `[dep/3; 3]`, channel-sum = `dep`), so the normaliser is
    // the constant `1` — `amount = Σdens = Σdep` (coverage) for EVERY colour, so black/dark
    // deposit real coverage (else `(0,0,0)` gave `amount = 0` → black painted nothing). For
    // non-black colours this matches the old `Σ(stroke)` normaliser's coverage exactly (the
    // old colour-scaled deposit cancelled to `Σdep` too).
    let color_sum = 1.0;
    WetCompositeBrush {
        prepared,
        pcol,
        color_sum,
    }
}

/// Canvas-pixel bbox `(px_lo, py_lo, px_hi, py_hi)` (exclusive hi) covered by a
/// grid region at `scale`, padded **2 grid cells each side** and clamped to the
/// canvas. The dispatch/loop bounds for the composite — shared by the CPU loop and
/// the GPU dispatch so they cover the same pixels.
///
/// **Pad = 2 cells (not 1):** the Catmull-Rom bicubic reads ±1.5 cells, the 2×2
/// coverage supersample adds a ±0.25-px sub-position offset, and the gated diffusion
/// leaks pigment ~1 cell past the wet gate (`diffuse`'s face-conductance). A 1-cell
/// pad under-covered the soft falloff → the round dab was hard-cut to the rectangle
/// (Enio's "quinas retangulares"). Caller MUST also feed a region that already
/// contains all pigment (the all-time wet envelope, not the receding water bbox).
#[must_use]
pub fn composite_canvas_region(
    grid_region: (u32, u32, u32, u32),
    scale: u32,
    cw: u32,
    ch: u32,
) -> (u32, u32, u32, u32) {
    let (gx0, gy0, gx1, gy1) = grid_region;
    let px_lo = gx0.saturating_sub(2) * scale;
    let py_lo = gy0.saturating_sub(2) * scale;
    let px_hi = ((gx1 + 3) * scale).min(cw);
    let py_hi = ((gy1 + 3) * scale).min(ch);
    (px_lo, py_lo, px_hi, py_hi)
}

/// Composite supersampling factor — `N×N` coverage samples per canvas pixel,
/// premultiplied-averaged. The antialiasing that smooths an OPAQUE stroke's
/// silhouette: the pigment field is bicubic-smooth, but a steep coverage edge is
/// under-sampled at pixel centers → jaggies ("baixa resolução nas bordas"). The GPU
/// `composite.wgsl` mirrors this `N` exactly (parity). `N=1` ⇒ the original
/// single-sample composite (no AA), bit-identical to pre-W15.3.
pub const WET_COMPOSITE_SS: u32 = 2;

/// CPU reference for the wet-field composite (the parity ground truth the GPU port
/// mirrors). Composites `pig` (low-res `gw×gh`) over `backdrop` into `canvas` (both
/// canvas-res `cw×ch` RGBA8), scoped to `grid_region`, with `WET_COMPOSITE_SS²`
/// coverage supersampling (premultiplied average) to antialias the edge.
/// `coverage_k` is the density→alpha rate; `scale` = canvas/grid ratio. Pixels
/// outside the padded region are untouched (the caller's backdrop invariant).
#[allow(clippy::too_many_arguments)]
pub fn composite_wet_field_cpu(
    canvas: &mut [u8],
    backdrop: &[u8],
    pig: &[[f32; 3]],
    gw: u32,
    gh: u32,
    cw: u32,
    ch: u32,
    scale: u32,
    coverage_k: f32,
    brush: &WetCompositeBrush,
    grid_region: (u32, u32, u32, u32),
) {
    let inv = 1.0 / scale as f32;
    let (px_lo, py_lo, px_hi, py_hi) = composite_canvas_region(grid_region, scale, cw, ch);
    let pcol = brush.pcol;
    let n = WET_COMPOSITE_SS.max(1);
    let inv_n = 1.0 / n as f32;
    // One glaze sub-sample over the (already-linear) backdrop. `None` ⇒ dry (backdrop).
    let glaze = |fx: f32, fy: f32, back: &[f32; 3], back_a: f32| -> Option<([f32; 3], f32)> {
        let p = sample_pigment_bicubic(pig, gw, gh, fx, fy);
        let dens = (p[0] + p[1] + p[2]).max(0.0);
        if dens < 1.0e-4 {
            return None;
        }
        let amount = dens / brush.color_sum;
        let alpha = 1.0 - (-amount * coverage_k).exp();
        let out_a = alpha + back_a * (1.0 - alpha);
        let km = mix_prepared_exact(&brush.prepared, *back, alpha);
        let inv_a = if out_a > 1.0e-4 { 1.0 / out_a } else { 0.0 };
        let mut rgb = [0.0f32; 3];
        for k in 0..3 {
            let straight = (pcol[k] * alpha + back[k] * back_a * (1.0 - alpha)) * inv_a;
            rgb[k] = (straight + (km[k] - straight) * back_a).clamp(0.0, 1.0);
        }
        Some((rgb, out_a))
    };
    for cy in py_lo..py_hi {
        for cx in px_lo..px_hi {
            let i = ((cy * cw + cx) * 4) as usize;
            let back_a = backdrop[i + 3] as f32 / 255.0;
            let back = [
                srgb_to_linear_byte(backdrop[i]),
                srgb_to_linear_byte(backdrop[i + 1]),
                srgb_to_linear_byte(backdrop[i + 2]),
            ];
            // Premultiplied average of N×N coverage sub-samples (dry → backdrop).
            let mut acc_rgb = [0.0f32; 3];
            let mut acc_a = 0.0f32;
            let mut any_wet = false;
            for sy in 0..n {
                let fy = ((cy as f32 + (sy as f32 + 0.5) * inv_n) * inv - 0.5)
                    .clamp(0.0, gh as f32 - 1.0);
                for sx in 0..n {
                    let fx = ((cx as f32 + (sx as f32 + 0.5) * inv_n) * inv - 0.5)
                        .clamp(0.0, gw as f32 - 1.0);
                    let (rgb_sub, a_sub) = match glaze(fx, fy, &back, back_a) {
                        Some(s) => {
                            any_wet = true;
                            s
                        }
                        None => (back, back_a),
                    };
                    acc_rgb[0] += rgb_sub[0] * a_sub;
                    acc_rgb[1] += rgb_sub[1] * a_sub;
                    acc_rgb[2] += rgb_sub[2] * a_sub;
                    acc_a += a_sub;
                }
            }
            if !any_wet {
                canvas[i..i + 4].copy_from_slice(&backdrop[i..i + 4]);
                continue;
            }
            let final_a = acc_a * inv_n * inv_n;
            let rgb = if acc_a > 1.0e-6 {
                [acc_rgb[0] / acc_a, acc_rgb[1] / acc_a, acc_rgb[2] / acc_a]
            } else {
                [0.0; 3]
            };
            canvas[i] = linear_to_srgb_byte(rgb[0]);
            canvas[i + 1] = linear_to_srgb_byte(rgb[1]);
            canvas[i + 2] = linear_to_srgb_byte(rgb[2]);
            canvas[i + 3] = (final_a * 255.0).round().clamp(0.0, 255.0) as u8;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The stroke-derived brush (GPU resident path) computes the SAME `pcol` +
    /// `color_sum` as the grid-total derivation, for a uniform-chromaticity field.
    #[test]
    fn stroke_derived_brush_matches_grid_derived() {
        let scol = [0.8f32, 0.6, 0.02];
        // A field of `scol × amount` (varying amount), the single-stroke invariant.
        let pig: Vec<[f32; 3]> = (0..64)
            .map(|i| {
                let a = (i as f32 + 1.0) * 0.01;
                [scol[0] * a, scol[1] * a, scol[2] * a]
            })
            .collect();
        let grid = prepare_wet_composite(&pig, scol);
        let stroke = prepare_wet_composite_from_stroke(scol);
        for k in 0..3 {
            assert!(
                (grid.pcol[k] - stroke.pcol[k]).abs() < 1e-5,
                "pcol[{k}] mismatch: {} vs {}",
                grid.pcol[k],
                stroke.pcol[k]
            );
        }
        assert!((grid.color_sum - stroke.color_sum).abs() < 1e-5);
    }

    /// The composite region MUST cover all pigment, or it clips the round dab into a
    /// rectangle (Enio's "quinas retangulares"). The true pigment bbox (+ the 2-cell
    /// pad) reproduces the full-canvas composite EXACTLY; a too-small region clips.
    /// The GPU path's monotonic wet envelope is a superset of this pigment bbox.
    #[test]
    fn composite_region_must_cover_pigment_or_it_clips() {
        use crate::diffusion::{DiffusionGrid, DiffusionParams};
        let (gw, gh, scale) = (48u32, 48u32, 1u32);
        let (cw, ch) = (gw, gh);
        let mut grid = DiffusionGrid::new(gw, gh, scale as f32);
        grid.splat(24.0, 24.0, 12.0, 0.7, [0.3, 0.15, 0.05]);
        let p = DiffusionParams::default();
        for _ in 0..8 {
            grid.step(&p);
        }
        let pig = grid.pigment().to_vec();
        let brush = prepare_wet_composite(&pig, [0.3, 0.15, 0.05]);
        let backdrop = vec![0u8; (cw * ch * 4) as usize];

        // Ground truth: composite over the FULL grid.
        let mut full = backdrop.clone();
        composite_wet_field_cpu(
            &mut full,
            &backdrop,
            &pig,
            gw,
            gh,
            cw,
            ch,
            scale,
            1.06,
            &brush,
            (0, 0, gw - 1, gh - 1),
        );

        // The true pigment bbox (+ pad) must reproduce the full composite EXACTLY.
        let pbb = wet_pigment_bbox(&pig, gw, gh).expect("pigment present");
        let mut tight = backdrop.clone();
        composite_wet_field_cpu(
            &mut tight, &backdrop, &pig, gw, gh, cw, ch, scale, 1.06, &brush, pbb,
        );
        assert_eq!(
            tight, full,
            "pigment-bbox region must reproduce the full composite (no clip)"
        );

        // Sensitivity: a deliberately TOO-SMALL region MUST clip (differ from full).
        let small = (
            pbb.0 + 5,
            pbb.1 + 5,
            pbb.2.saturating_sub(5),
            pbb.3.saturating_sub(5),
        );
        let mut clipped = backdrop.clone();
        composite_wet_field_cpu(
            &mut clipped,
            &backdrop,
            &pig,
            gw,
            gh,
            cw,
            ch,
            scale,
            1.06,
            &brush,
            small,
        );
        assert_ne!(
            clipped, full,
            "a too-small region MUST clip the dab (test sensitivity)"
        );
    }

    /// Yellow wash over an opaque BLUE backdrop must go GREEN-dominant — the
    /// Kubelka–Munk signature (a linear "over" leaves R≈G). The discriminant the
    /// GPU parity test reuses.
    #[test]
    fn yellow_over_blue_is_green() {
        let (gw, gh, scale) = (8u32, 8u32, 2u32);
        let (cw, ch) = (gw * scale, gh * scale);
        // Uniform yellow pigment field, opaque blue backdrop.
        let pig = vec![[0.6f32, 0.6, 0.0]; (gw * gh) as usize];
        let mut backdrop = vec![0u8; (cw * ch * 4) as usize];
        for px in backdrop.chunks_exact_mut(4) {
            px.copy_from_slice(&[20, 40, 200, 255]);
        }
        let mut canvas = backdrop.clone();
        // Stroke colour ~ yellow (linear).
        let brush = prepare_wet_composite(&pig, [0.8, 0.6, 0.02]);
        composite_wet_field_cpu(
            &mut canvas,
            &backdrop,
            &pig,
            gw,
            gh,
            cw,
            ch,
            scale,
            1.06,
            &brush,
            (0, 0, gw - 1, gh - 1),
        );
        let i = ((ch / 2 * cw + cw / 2) * 4) as usize;
        let (r, g, b) = (canvas[i] as i32, canvas[i + 1] as i32, canvas[i + 2] as i32);
        assert!(
            g > r && g > b,
            "K–M wash green-dominant over blue: [{r},{g},{b}]"
        );
    }

    /// A wash over a fully TRANSPARENT backdrop must read as the pigment colour with
    /// NO black fringe (straight-alpha "over", not a mix toward black).
    #[test]
    fn edge_over_transparent_has_no_black_fringe() {
        let (gw, gh, scale) = (8u32, 8u32, 2u32);
        let (cw, ch) = (gw * scale, gh * scale);
        // A soft coral dab, low mass at the rim → partial coverage there.
        let mut pig = vec![[0.0f32; 3]; (gw * gh) as usize];
        for gy in 0..gh {
            for gx in 0..gw {
                let d = (((gx as f32 - 3.5).powi(2) + (gy as f32 - 3.5).powi(2)).sqrt()) / 3.5;
                let m = (1.0 - d).max(0.0) * 0.5;
                pig[(gy * gw + gx) as usize] = [m, m * 0.45, m * 0.4];
            }
        }
        let backdrop = vec![0u8; (cw * ch * 4) as usize]; // transparent
        let mut canvas = backdrop.clone();
        let brush = prepare_wet_composite(&pig, [0.8, 0.36, 0.32]);
        composite_wet_field_cpu(
            &mut canvas,
            &backdrop,
            &pig,
            gw,
            gh,
            cw,
            ch,
            scale,
            1.06,
            &brush,
            (0, 0, gw - 1, gh - 1),
        );
        // Every painted (alpha>0) pixel must keep a warm coral hue — red leads, and
        // no pixel collapses to near-black-with-alpha (the fringe bug).
        for px in canvas.chunks_exact(4) {
            if px[3] > 8 {
                assert!(
                    px[0] >= px[2],
                    "coral keeps red≥blue (no muddy fringe): {px:?}"
                );
                assert!(
                    px[0] as u32 + px[1] as u32 + px[2] as u32 > 24,
                    "partial-coverage edge is not a black fringe: {px:?}"
                );
            }
        }
    }
}
