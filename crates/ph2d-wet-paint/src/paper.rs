//! Procedural paper (port of `paper.js`, SPEC §4).
//!
//! A 512x512 height tile in [0,1], tiled across the canvas with integer wrap
//! and baked into the padded grid (pad ring included — border cells must read
//! a real tooth or the edge capillary term diverges).
//!
//! The relief the solver needs is a fine quasi-periodic FELT: narrow grooves
//! 1-2 px wide broken into short segments, densely packed, with one dominant
//! orientation; nearly all height variance below 8 px wavelength. Long smooth
//! valleys are a defect — they make drip fronts descend as fused wedges
//! instead of fingering into rivulets that follow the grain.
//!
//! Construction: (lattice undulation + fine tooth + hash sparkle) + thousands
//! of short fiber strokes, then a high-pass (subtract 0.55 x 8-px wrap box
//! blur), then a contrast stretch to the preset's target mean/std with a
//! final clamp to [0,1] (the clamp mints the sparse 0/1 tails real scans have).

use crate::grid::Grid;
use crate::jsmath::wrap_mask;
use crate::rng::{Rng, hash2};

pub const TILE_SIZE: usize = 512;
const TS: usize = TILE_SIZE;
const TMASK: i32 = TS as i32 - 1;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PaperPreset {
    /// Cold press (default): isotropic-ish felt.
    Cold,
    /// Rough: deep tooth, strongly vertical grain.
    Rough,
    /// Hot press: smooth, shallow, fine.
    Hot,
}

/// Target statistics match scans of real sheets (acceptance-tested, §18.6).
/// Amplitudes are calibration values: only their RELATIVE size matters (the
/// contrast stretch renormalizes); they were tuned so the measured
/// mean-gradient lands on the target for each preset.
pub struct PaperParams {
    pub mean: f64,
    pub std: f64,
    pub grad: f64,
    pub fibres: f64,
    pub angle: f64,
    pub jitter: f64,
    pub tooth_amp: f64,
    pub sparkle_amp: f64,
    pub fibre_depth: f64,
    seed_base: u32,
}

impl PaperPreset {
    pub fn params(self) -> &'static PaperParams {
        match self {
            PaperPreset::Cold => &PaperParams {
                mean: 0.543,
                std: 0.150,
                grad: 0.121,
                fibres: 9000.0,
                angle: 0.35,
                jitter: 1.0,
                tooth_amp: 0.28,
                sparkle_amp: 0.062,
                fibre_depth: 0.16,
                seed_base: 11,
            },
            PaperPreset::Rough => &PaperParams {
                mean: 0.517,
                std: 0.192,
                grad: 0.144,
                fibres: 11000.0,
                angle: std::f64::consts::FRAC_PI_2,
                jitter: 0.25,
                tooth_amp: 0.34,
                sparkle_amp: 0.052,
                fibre_depth: 0.20,
            seed_base: 23,
            },
            PaperPreset::Hot => &PaperParams {
                mean: 0.497,
                std: 0.106,
                grad: 0.098,
                fibres: 5000.0,
                angle: 0.35,
                jitter: 1.1,
                tooth_amp: 0.20,
                sparkle_amp: 0.125,
                fibre_depth: 0.10,
                seed_base: 37,
            },
        }
    }
}

/// Live paper-group multipliers (all default 1).
#[derive(Clone, Copy)]
pub struct PaperKnobs {
    pub contrast: f64,
    pub fibres: f64,
    pub grooves: f64,
}

impl Default for PaperKnobs {
    fn default() -> Self {
        PaperKnobs { contrast: 1.0, fibres: 1.0, grooves: 1.0 }
    }
}

/// Seamless bilinear value-noise from an n x n lattice, sampled at tile (x,y).
/// The lattice is f32 (the JS Float32Array); the blend runs in f64.
fn lattice_noise(values: &[f32], n: usize, x: usize, y: usize) -> f64 {
    let fx = (x as f64 / TS as f64) * n as f64;
    let fy = (y as f64 / TS as f64) * n as f64;
    let x0 = fx as usize;
    let y0 = fy as usize;
    let tx = fx - x0 as f64;
    let ty = fy - y0 as f64;
    let x1 = (x0 + 1) % n;
    let y1 = (y0 + 1) % n;
    let v00 = values[y0 * n + x0] as f64;
    let v10 = values[y0 * n + x1] as f64;
    let v01 = values[y1 * n + x0] as f64;
    let v11 = values[y1 * n + x1] as f64;
    let a = v00 + (v10 - v00) * tx;
    let b = v01 + (v11 - v01) * tx;
    a + (b - a) * ty
}

/// Generate one paper tile. Deterministic per (preset, sheet, knobs).
pub fn generate_paper_tile(preset: PaperPreset, sheet_index: u32, knobs: PaperKnobs) -> Vec<f32> {
    let p = preset.params();
    let seed = p
        .seed_base
        .wrapping_mul(1_000_003)
        .wrapping_add(sheet_index.wrapping_mul(7919))
        .wrapping_add(5);
    let mut rng = Rng::new(seed);
    let mut t = vec![0.0f32; TS * TS];

    // -- component 1+2: a faint large-scale undulation (6x6 lattice) and a
    //    fine 2-px tooth (256x256 lattice), both seamless.
    // The JS keeps both lattices in Float32Array: every rng draw ROUNDS to
    // f32 before the noise sampler reads it (port-verify finding — keeping
    // f64 here shifted every tile texel).
    let mut big = [0.0f32; 36];
    for v in big.iter_mut() {
        *v = rng.next() as f32;
    }
    const FINE: usize = 256;
    let mut fine = vec![0.0f32; FINE * FINE];
    for v in fine.iter_mut() {
        *v = rng.next() as f32;
    }
    let sparkle_seed = seed ^ 0x5f37_59df;
    for y in 0..TS {
        for x in 0..TS {
            let mut v = 0.5;
            v += (lattice_noise(&big, 6, x, y) - 0.5) * 0.02; // undulation
            v += (lattice_noise(&fine, FINE, x, y) - 0.5) * p.tooth_amp; // fine tooth
            v += (hash2(x as i32, y as i32, sparkle_seed) - 0.5) * p.sparkle_amp; // sparkle
            t[y * TS + x] = v as f32;
        }
    }

    // -- component 3: thousands of short fiber strokes. Each walks 5-13 steps
    //    of ~0.7 px along the grain angle (+- jitter), stamping a soft round
    //    cross-section, mostly grooves (negative) with ~38% bright ridges.
    let count = (p.fibres * knobs.fibres).round().max(0.0) as i64;
    for _ in 0..count {
        let mut fx = rng.next() * TS as f64;
        let mut fy = rng.next() * TS as f64;
        let ang = p.angle + (rng.next() * 2.0 - 1.0) * p.jitter;
        let dx = libm::cos(ang) * 0.7;
        let dy = libm::sin(ang) * 0.7;
        // JS: `5 + (rng() * 9) | 0` — the addition binds first, so this is
        // trunc(5 + rng*9), i.e. 5..=13 steps.
        let steps = (5.0 + rng.next() * 9.0) as i32;
        let half_w = 0.4 + rng.next() * 0.7;
        let sign = if rng.next() < 0.38 { 1.0 } else { -1.0 };
        let depth = sign * (0.5 + rng.next() * 0.5) * p.fibre_depth * knobs.grooves;
        let reach = (half_w + 1.0).ceil() as i32;
        let inv_hw = 1.0 / (half_w + 0.5);
        for _s in 0..steps {
            let cx = fx;
            let cy = fy;
            let x0 = cx.floor() as i32 - reach;
            let x1 = cx.floor() as i32 + reach;
            let y0 = cy.floor() as i32 - reach;
            let y1 = cy.floor() as i32 + reach;
            for yy in y0..=y1 {
                for xx in x0..=x1 {
                    let ddx = xx as f64 - cx;
                    let ddy = yy as f64 - cy;
                    let d = (ddx * ddx + ddy * ddy).sqrt() * inv_hw;
                    if d >= 1.0 {
                        continue;
                    }
                    let w = 1.0 - d * d * (3.0 - 2.0 * d); // soft round cross-section
                    let idx = wrap_mask(yy, TMASK) * TS + wrap_mask(xx, TMASK);
                    t[idx] = (t[idx] as f64 + depth * w) as f32;
                }
            }
            fx += dx;
            fy += dy;
        }
    }

    // -- high-pass: subtract 0.55 x an 8-px wrap-around box blur (separable),
    //    pushing ~95% of the variance below 8 px wavelength.
    let blurred = box_blur_wrap(&t, 4);
    for i in 0..t.len() {
        t[i] = (t[i] as f64 - 0.55 * blurred[i] as f64) as f32;
    }

    // -- contrast stretch to the target statistics, then clamp.
    let mut mean = 0.0f64;
    for &v in t.iter() {
        mean += v as f64;
    }
    mean /= t.len() as f64;
    let mut variance = 0.0f64;
    for &v in t.iter() {
        let d = v as f64 - mean;
        variance += d * d;
    }
    let std = (variance / t.len() as f64).sqrt();
    // JS `|| 1`: falsy remap — both 0 AND NaN become 1 (port-verify finding).
    let std = if std == 0.0 || std.is_nan() { 1.0 } else { std };
    let gain = (p.std * knobs.contrast) / std;
    for v in t.iter_mut() {
        let nv = p.mean + (*v as f64 - mean) * gain;
        *v = nv.clamp(0.0, 1.0) as f32;
    }
    t
}

/// Separable wrap-around box blur, radius r (window 2r+1).
fn box_blur_wrap(src: &[f32], r: i32) -> Vec<f32> {
    let mut tmp = vec![0.0f32; TS * TS];
    let mut out = vec![0.0f32; TS * TS];
    let inv = 1.0 / (2 * r + 1) as f64;
    for y in 0..TS {
        // horizontal
        let row = y * TS;
        let mut acc = 0.0f64;
        for k in -r..=r {
            acc += src[row + wrap_mask(k, TMASK)] as f64;
        }
        for x in 0..TS as i32 {
            tmp[row + x as usize] = (acc * inv) as f32;
            acc += src[row + wrap_mask(x + r + 1, TMASK)] as f64
                - src[row + wrap_mask(x - r, TMASK)] as f64;
        }
    }
    for x in 0..TS {
        // vertical
        let mut acc = 0.0f64;
        for k in -r..=r {
            acc += tmp[wrap_mask(k, TMASK) * TS + x] as f64;
        }
        for y in 0..TS as i32 {
            out[y as usize * TS + x] = (acc * inv) as f32;
            acc += tmp[wrap_mask(y + r + 1, TMASK) * TS + x] as f64
                - tmp[wrap_mask(y - r, TMASK) * TS + x] as f64;
        }
    }
    out
}

/// Bake a tile into the padded grid paper array, pad ring included, with
/// integer wrap (nearest texel). Cell (x,y) maps to tile texel (x-1, y-1).
pub fn bake_paper(grid: &mut Grid, tile: &[f32]) {
    let s = grid.s;
    for y in 0..grid.rows {
        let ty = wrap_mask(y as i32 - 1, TMASK) * TS;
        let mut i = y * s;
        for x in 0..s {
            grid.paper[i] = tile[ty + wrap_mask(x as i32 - 1, TMASK)];
            i += 1;
        }
    }
}

/// Measured statistics of a tile (acceptance test §18.6 + calibration).
pub struct TileStats {
    pub mean: f64,
    pub std: f64,
    pub grad: f64,
}

pub fn measure_tile_stats(t: &[f32]) -> TileStats {
    let mut mean = 0.0f64;
    for &v in t.iter() {
        mean += v as f64;
    }
    mean /= t.len() as f64;
    let mut variance = 0.0f64;
    let mut grad_sum = 0.0f64;
    for y in 0..TS as i32 {
        for x in 0..TS as i32 {
            let v = t[y as usize * TS + x as usize] as f64;
            let d = v - mean;
            variance += d * d;
            let gx = (t[y as usize * TS + wrap_mask(x + 1, TMASK)] as f64
                - t[y as usize * TS + wrap_mask(x - 1, TMASK)] as f64)
                * 0.5;
            let gy = (t[wrap_mask(y + 1, TMASK) * TS + x as usize] as f64
                - t[wrap_mask(y - 1, TMASK) * TS + x as usize] as f64)
                * 0.5;
            grad_sum += (gx * gx + gy * gy).sqrt();
        }
    }
    TileStats {
        mean,
        std: (variance / t.len() as f64).sqrt(),
        grad: grad_sum / t.len() as f64,
    }
}
