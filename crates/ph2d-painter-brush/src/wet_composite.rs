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
//! The algorithm, per canvas pixel (scoped to the wet bbox by the caller), ADR-0080:
//!   1. bicubic (Catmull-Rom) upsample of the low-res [`PIG_CH`]-channel field → cell;
//!   2. REDUCE the cell to the mixed pigment ([`prepared_from_field`]): `ks_mix = ks_acc/mass`,
//!      `colour = reflectance_to_rgb(ks_to_refl(ks_mix)) + err_acc/mass`; bare paper
//!      (`mass < 1e-4`) writes the backdrop and skips;
//!   3. value-opacity (ADR-0079) from the MIXED colour: `alpha = 1 − exp(−(mass/color_sum)·K)`,
//!      `color_sum = 0.3 + 0.7·value`;
//!   4. straight-alpha glaze: K–M mix ([`mix_prepared_exact`]) over opaque backdrop, porter-duff
//!      "over" at a transparent edge, lerped by the backdrop's own alpha (no black fringe).

use crate::diffusion::{PIG_BANDS, PIG_ERR0, PIG_CH, PIG_MASS, WetCell};
use crate::pigment_mix::{PreparedPigment, mix_prepared_exact, prepared_from_field};
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

/// Bicubic (Catmull-Rom) sample of the low-res wet-field at fractional grid coords `(fx, fy)`,
/// edges clamped — over the raw [`PIG_CH`] channels (ADR-0080). The field channels are all
/// EXTENSIVE (mass-weighted K/S + err + mass), so bicubic over them is the correct
/// premultiplied interpolation; the per-pixel reduction ([`prepared_from_field`]) divides by
/// the interpolated mass afterwards. The cubic can overshoot, so the K/S bands + mass (which
/// are physically ≥0) are floored at 0; the signed `err` re-anchor passes through. The GPU
/// `composite.wgsl` mirrors this band-for-band.
#[inline]
#[must_use]
pub fn sample_pigment_bicubic(pig: &[WetCell], gw: u32, gh: u32, fx: f32, fy: f32) -> WetCell {
    let x0 = fx.floor() as i32;
    let y0 = fy.floor() as i32;
    let wx = catmull_rom_weights(fx - x0 as f32);
    let wy = catmull_rom_weights(fy - y0 as f32);
    let cx = |x: i32| x.clamp(0, gw as i32 - 1) as u32;
    let cy = |y: i32| y.clamp(0, gh as i32 - 1) as u32;
    let mut out = [0.0f32; PIG_CH];
    for (j, &wyj) in wy.iter().enumerate() {
        let gy = cy(y0 - 1 + j as i32);
        let mut row = [0.0f32; PIG_CH];
        for (i, &wxi) in wx.iter().enumerate() {
            let p = &pig[(gy * gw + cx(x0 - 1 + i as i32)) as usize];
            for k in 0..PIG_CH {
                row[k] += p[k] * wxi;
            }
        }
        for k in 0..PIG_CH {
            out[k] += row[k] * wyj;
        }
    }
    // Floor the physically-nonneg channels (K/S bands + mass); keep err signed.
    for v in &mut out[0..PIG_BANDS] {
        *v = v.max(0.0);
    }
    out[PIG_MASS] = out[PIG_MASS].max(0.0);
    out
}

/// Inclusive grid-cell bbox of cells carrying compositable pigment (coverage `mass`
/// ≥ the composite floor `1e-4`). `None` if the grid is bare. A cheap O(cells)
/// scan that scopes the composite to the wet region.
#[must_use]
pub fn wet_pigment_bbox(pig: &[WetCell], gw: u32, gh: u32) -> Option<(u32, u32, u32, u32)> {
    let (mut x0, mut y0, mut x1, mut y1) = (u32::MAX, u32::MAX, 0u32, 0u32);
    let mut any = false;
    for gy in 0..gh {
        let row = (gy * gw) as usize;
        for gx in 0..gw {
            if pig[row + gx as usize][PIG_MASS] >= 1.0e-4 {
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

/// Reduce a bicubic-sampled wet-field cell to its mixed [`PreparedPigment`] + coverage `mass`
/// (ADR-0080) — the per-pixel composite reduction. The K/S bands are `Σ mass·ks` (premult), so
/// `prepared_from_field` divides by the interpolated mass to get the mixed pigment; `mass` is
/// the coverage. A single pigment yields exactly its own `prepare_pigment`, so the single-colour
/// composite reproduces the pre-ADR-0080 path to float precision.
#[inline]
#[must_use]
fn reduce_sampled_cell(cell: &WetCell) -> (PreparedPigment, f32) {
    let mass = cell[PIG_MASS];
    let ks: [f32; PIG_BANDS] = std::array::from_fn(|i| cell[i]);
    let err = [cell[PIG_ERR0], cell[PIG_ERR0 + 1], cell[PIG_ERR0 + 2]];
    (prepared_from_field(&ks, err, mass), mass)
}

/// CPU reference for the wet-field composite (the parity ground truth the GPU port
/// mirrors). Composites `pig` (low-res `gw×gh`, [`PIG_CH`] channels) over `backdrop` into
/// `canvas` (both canvas-res `cw×ch` RGBA8), scoped to `grid_region`, with `WET_COMPOSITE_SS²`
/// coverage supersampling (premultiplied average) to antialias the edge. Per pixel the field
/// reduces to a mixed pigment (ADR-0080: blue+yellow→green), value-opacity per ADR-0079, then
/// a Kubelka–Munk glaze over the backdrop. `coverage_k` is the mass→alpha rate; `scale` =
/// canvas/grid ratio. Pixels outside the padded region are untouched.
#[allow(clippy::too_many_arguments)]
pub fn composite_wet_field_cpu(
    canvas: &mut [u8],
    backdrop: &[u8],
    pig: &[WetCell],
    gw: u32,
    gh: u32,
    cw: u32,
    ch: u32,
    scale: u32,
    coverage_k: f32,
    grid_region: (u32, u32, u32, u32),
) {
    let inv = 1.0 / scale as f32;
    let (px_lo, py_lo, px_hi, py_hi) = composite_canvas_region(grid_region, scale, cw, ch);
    let n = WET_COMPOSITE_SS.max(1);
    let inv_n = 1.0 / n as f32;
    // One glaze sub-sample over the (already-linear) backdrop. `None` ⇒ dry (backdrop).
    let glaze = |fx: f32, fy: f32, back: &[f32; 3], back_a: f32| -> Option<([f32; 3], f32)> {
        let cell = sample_pigment_bicubic(pig, gw, gh, fx, fy);
        let (prepared, mass) = reduce_sampled_cell(&cell);
        if mass < 1.0e-4 {
            return None;
        }
        let pcol = prepared.color();
        // ADR-0079 value-opacity: a deeper (lower-value) MIXED pigment covers more. `value` is
        // the mixed colour's max channel, so the subtractive mix's value drives the build-up.
        let value = pcol[0].max(pcol[1]).max(pcol[2]).clamp(0.0, 1.0);
        let color_sum = 0.3 + 0.7 * value;
        let amount = mass / color_sum;
        let alpha = 1.0 - (-amount * coverage_k).exp();
        let out_a = alpha + back_a * (1.0 - alpha);
        let km = mix_prepared_exact(&prepared, *back, alpha);
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
    use crate::diffusion::DiffusionGrid;
    use crate::pigment_mix::prepare_pigment; // legacy-formula parity helper

    /// Build a low-res wet-field of a single `color` with per-cell coverage `mass` (the
    /// single-stroke invariant) — the K/S accumulation each cell carries (ADR-0080).
    fn field_of(color: [f32; 3], mass: &[f32]) -> Vec<WetCell> {
        mass.iter()
            .map(|&m| DiffusionGrid::cell_from_color_mass(color, m))
            .collect()
    }

    /// Element-wise sum of two cells (two pigments co-present → they mix at the reduction).
    fn add_cells(a: WetCell, b: &WetCell) -> WetCell {
        let mut c = a;
        for k in 0..PIG_CH {
            c[k] += b[k];
        }
        c
    }

    /// The **legacy (pre-ADR-0080) single-colour composite**, recomputed inline: gray coverage
    /// `mass` bicubic-upsampled = `dens`, uniform `pcol = colour`, value-opacity
    /// `color_sum = 0.3+0.7·value`, K–M glaze via `prepare_pigment(colour)`. The parity target
    /// for [`single_color_composite_matches_legacy_formula`].
    #[allow(clippy::too_many_arguments)]
    fn legacy_single_color_composite(
        canvas: &mut [u8],
        backdrop: &[u8],
        mass: &[f32],
        col: [f32; 3],
        gw: u32,
        gh: u32,
        cw: u32,
        ch: u32,
        scale: u32,
        coverage_k: f32,
    ) {
        let prepared = prepare_pigment(col);
        let value = col[0].max(col[1]).max(col[2]).clamp(0.0, 1.0);
        let color_sum = 0.3 + 0.7 * value;
        let inv = 1.0 / scale as f32;
        let n = WET_COMPOSITE_SS.max(1);
        let inv_n = 1.0 / n as f32;
        let (px_lo, py_lo, px_hi, py_hi) =
            composite_canvas_region((0, 0, gw - 1, gh - 1), scale, cw, ch);
        // Bicubic of the scalar mass field (= the legacy `dens`, since old gray pigment summed
        // to the coverage and bicubic is linear).
        let sample_mass = |fx: f32, fy: f32| -> f32 {
            let x0 = fx.floor() as i32;
            let y0 = fy.floor() as i32;
            let wx = catmull_rom_weights(fx - x0 as f32);
            let wy = catmull_rom_weights(fy - y0 as f32);
            let cx = |x: i32| x.clamp(0, gw as i32 - 1) as u32;
            let cy = |y: i32| y.clamp(0, gh as i32 - 1) as u32;
            let mut out = 0.0f32;
            for (j, &wyj) in wy.iter().enumerate() {
                let gy = cy(y0 - 1 + j as i32);
                let mut row = 0.0f32;
                for (i, &wxi) in wx.iter().enumerate() {
                    row += mass[(gy * gw + cx(x0 - 1 + i as i32)) as usize] * wxi;
                }
                out += row * wyj;
            }
            out.max(0.0)
        };
        for cyp in py_lo..py_hi {
            for cxp in px_lo..px_hi {
                let i = ((cyp * cw + cxp) * 4) as usize;
                let back_a = backdrop[i + 3] as f32 / 255.0;
                let back = [
                    srgb_to_linear_byte(backdrop[i]),
                    srgb_to_linear_byte(backdrop[i + 1]),
                    srgb_to_linear_byte(backdrop[i + 2]),
                ];
                let mut acc_rgb = [0.0f32; 3];
                let mut acc_a = 0.0f32;
                let mut any_wet = false;
                for sy in 0..n {
                    let fy = ((cyp as f32 + (sy as f32 + 0.5) * inv_n) * inv - 0.5)
                        .clamp(0.0, gh as f32 - 1.0);
                    for sx in 0..n {
                        let fx = ((cxp as f32 + (sx as f32 + 0.5) * inv_n) * inv - 0.5)
                            .clamp(0.0, gw as f32 - 1.0);
                        let dens = sample_mass(fx, fy);
                        let (rgb_sub, a_sub) = if dens < 1.0e-4 {
                            (back, back_a)
                        } else {
                            any_wet = true;
                            let amount = dens / color_sum;
                            let alpha = 1.0 - (-amount * coverage_k).exp();
                            let out_a = alpha + back_a * (1.0 - alpha);
                            let km = mix_prepared_exact(&prepared, back, alpha);
                            let inv_a = if out_a > 1.0e-4 { 1.0 / out_a } else { 0.0 };
                            let mut rgb = [0.0f32; 3];
                            for k in 0..3 {
                                let straight =
                                    (col[k] * alpha + back[k] * back_a * (1.0 - alpha)) * inv_a;
                                rgb[k] = (straight + (km[k] - straight) * back_a).clamp(0.0, 1.0);
                            }
                            (rgb, out_a)
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

    /// **THE P2 single-colour parity gate (ADR-0080 §2.3):** a single-colour wash composites to
    /// within ≤1 LSB (RGBA8) of the pre-ADR-0080 formula — the ADR-0080 reduction (`ks_acc/mass`)
    /// differs from `prepare_pigment(colour)` only by ~1e-6 float reassociation, which the u8
    /// quantisation absorbs. This guards the validated single-colour look (value-opacity,
    /// edge-darkening, K–M glaze). Tested over BOTH an opaque + a transparent backdrop.
    #[test]
    fn single_color_composite_matches_legacy_formula() {
        let (gw, gh, scale) = (8u32, 8u32, 2u32);
        let (cw, ch) = (gw * scale, gh * scale);
        let col = [0.2f32, 0.45, 0.85];
        // A soft radial mass field (varying coverage including partial edges).
        let mass: Vec<f32> = (0..gw * gh)
            .map(|idx| {
                let (gx, gy) = ((idx % gw) as f32, (idx / gw) as f32);
                let d = (((gx - 3.5).powi(2) + (gy - 3.5).powi(2)).sqrt()) / 3.5;
                (1.0 - d).max(0.0) * 0.8
            })
            .collect();
        let pig = field_of(col, &mass);
        for backdrop_px in [[90u8, 110, 130, 255], [0, 0, 0, 0]] {
            let mut backdrop = vec![0u8; (cw * ch * 4) as usize];
            for px in backdrop.chunks_exact_mut(4) {
                px.copy_from_slice(&backdrop_px);
            }
            let mut new_canvas = backdrop.clone();
            composite_wet_field_cpu(
                &mut new_canvas,
                &backdrop,
                &pig,
                gw,
                gh,
                cw,
                ch,
                scale,
                1.06,
                (0, 0, gw - 1, gh - 1),
            );
            let mut old_canvas = backdrop.clone();
            legacy_single_color_composite(
                &mut old_canvas,
                &backdrop,
                &mass,
                col,
                gw,
                gh,
                cw,
                ch,
                scale,
                1.06,
            );
            let max_d = new_canvas
                .iter()
                .zip(&old_canvas)
                .map(|(&a, &b)| (a as i32 - b as i32).abs())
                .max()
                .unwrap_or(0);
            assert!(
                max_d <= 1,
                "single-colour composite must match the legacy formula within ≤1 LSB (got {max_d}, backdrop {backdrop_px:?})"
            );
        }
    }

    /// **THE P2 mix composite gate (ADR-0080):** a wet field carrying BOTH blue + yellow pigment
    /// composites to a GREEN-dominant pixel over a white backdrop — the subtractive wet-on-wet
    /// mix, end to end through the composite (not the muddy grey a coverage average gives).
    #[test]
    fn composite_mixes_blue_and_yellow_to_green() {
        let (gw, gh, scale) = (8u32, 8u32, 2u32);
        let (cw, ch) = (gw * scale, gh * scale);
        // Every cell carries equal blue + yellow pigment → reduces to green.
        let blue = DiffusionGrid::cell_from_color_mass([0.0, 0.0, 1.0], 1.0);
        let yellow = DiffusionGrid::cell_from_color_mass([1.0, 1.0, 0.0], 1.0);
        let cell = add_cells(blue, &yellow);
        let pig = vec![cell; (gw * gh) as usize];
        // Opaque white backdrop (so the wash reads as the pigment colour, not a backdrop tint).
        let mut backdrop = vec![0u8; (cw * ch * 4) as usize];
        for px in backdrop.chunks_exact_mut(4) {
            px.copy_from_slice(&[255, 255, 255, 255]);
        }
        let mut canvas = backdrop.clone();
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
            (0, 0, gw - 1, gh - 1),
        );
        let i = ((ch / 2 * cw + cw / 2) * 4) as usize;
        let (r, g, b) = (canvas[i] as i32, canvas[i + 1] as i32, canvas[i + 2] as i32);
        assert!(
            g > r && g > b,
            "blue+yellow field composites green-dominant (not mud): [{r},{g},{b}]"
        );
    }

    /// The composite region MUST cover all pigment, or it clips the round dab into a
    /// rectangle (Enio's "quinas retangulares"). The true pigment bbox (+ the 2-cell
    /// pad) reproduces the full-canvas composite EXACTLY; a too-small region clips.
    #[test]
    fn composite_region_must_cover_pigment_or_it_clips() {
        use crate::diffusion::DiffusionParams;
        let (gw, gh, scale) = (48u32, 48u32, 1u32);
        let (cw, ch) = (gw, gh);
        let mut grid = DiffusionGrid::new(gw, gh, scale as f32);
        grid.splat(24.0, 24.0, 12.0, 0.7, [0.3, 0.15, 0.05], 0.5);
        let p = DiffusionParams::default();
        for _ in 0..8 {
            grid.step(&p);
        }
        let pig = grid.pigment().to_vec();
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
            (0, 0, gw - 1, gh - 1),
        );

        // The true pigment bbox (+ pad) must reproduce the full composite EXACTLY.
        let pbb = wet_pigment_bbox(&pig, gw, gh).expect("pigment present");
        let mut tight = backdrop.clone();
        composite_wet_field_cpu(
            &mut tight, &backdrop, &pig, gw, gh, cw, ch, scale, 1.06, pbb,
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
            &mut clipped, &backdrop, &pig, gw, gh, cw, ch, scale, 1.06, small,
        );
        assert_ne!(
            clipped, full,
            "a too-small region MUST clip the dab (test sensitivity)"
        );
    }

    /// Yellow wash over an opaque BLUE backdrop must go GREEN-dominant — the Kubelka–Munk glaze
    /// signature (a linear "over" leaves R≈G). The discriminant the GPU parity test reuses.
    #[test]
    fn yellow_over_blue_is_green() {
        let (gw, gh, scale) = (8u32, 8u32, 2u32);
        let (cw, ch) = (gw * scale, gh * scale);
        // Uniform yellow pigment field, opaque blue backdrop.
        let pig = field_of([0.8, 0.6, 0.02], &vec![0.6f32; (gw * gh) as usize]);
        let mut backdrop = vec![0u8; (cw * ch * 4) as usize];
        for px in backdrop.chunks_exact_mut(4) {
            px.copy_from_slice(&[20, 40, 200, 255]);
        }
        let mut canvas = backdrop.clone();
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
        let mass: Vec<f32> = (0..gw * gh)
            .map(|idx| {
                let (gx, gy) = ((idx % gw) as f32, (idx / gw) as f32);
                let d = (((gx - 3.5).powi(2) + (gy - 3.5).powi(2)).sqrt()) / 3.5;
                (1.0 - d).max(0.0) * 0.5
            })
            .collect();
        let pig = field_of([0.8, 0.36, 0.32], &mass);
        let backdrop = vec![0u8; (cw * ch * 4) as usize]; // transparent
        let mut canvas = backdrop.clone();
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
