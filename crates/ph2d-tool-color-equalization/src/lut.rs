//! 3D LUT color grading (Phase 3).
//!
//! Trilinear-interpolated 3D lookup table over a regular RGB grid of
//! `size³` cells. Each cell stores an output triple `[r, g, b] ∈ [0, 1]`.
//! [`apply_lut3d`] reads the cell by mapping the input sRGB triple onto
//! grid coordinates, then interpolates the 8 surrounding cells.
//!
//! `.cube` (Resolve / Premiere / Filmlight format) load + save can be
//! layered on top later; the runtime contract here is whatever produces
//! a `LUT3D` of `size·size·size·3` floats in row-major (`R varies
//! fastest, then G, then B`) order. The procedural presets in
//! [`crate::lut_presets`] use this convention.
//!
//! **GPU note**: this stage is the canonical 3D-texture-sampler win — a
//! single `textureSampleLevel(lut3d, sampler, vec3<f32>(r,g,b))` call
//! does the trilinear interp in hardware. A `wgpu` follow-up replaces
//! this CPU loop with one compute shader pass; expect ~30× speedup at
//! 1024². Keep the CPU path as the parity / fallback.

use crate::color_utils::clamp01;

/// Default cube side for procedurally generated LUTs — matches the
/// legacy engine's `PRESET_LUT_SIZE`. 17 cells per axis is the
/// canonical Resolve / DaVinci default; 17³ = 4 913 samples per LUT,
/// ≈ 60 KB at f32 RGB.
pub const DEFAULT_LUT_SIZE: u32 = 17;

/// A trilinear-sampleable 3D color lookup table.
#[derive(Clone, Debug, PartialEq)]
pub struct LUT3D {
    /// Number of cells per axis (cube side). Total cells = `size³`.
    pub size: u32,
    /// Flattened `size³ × 3` array of RGB triples in `[0, 1]`. Index
    /// order: `R` varies fastest, then `G`, then `B`. Cell `(rI, gI, bI)`
    /// channel `ch` lives at `data[(bI·size² + gI·size + rI) · 3 + ch]`.
    pub data: Vec<f32>,
}

impl LUT3D {
    /// `data.len()` expected for a given cube side.
    pub fn expected_len(size: u32) -> usize {
        (size as usize).pow(3) * 3
    }

    /// True when the LUT has the canonical identity shape (each cell
    /// outputs its own grid coordinate — `apply_lut3d` is a no-op).
    /// Useful as a fast-path test.
    pub fn is_identity(&self, tol: f32) -> bool {
        let n = self.size as i32;
        for bi in 0..n {
            for gi in 0..n {
                for ri in 0..n {
                    let idx = ((bi * n + gi) * n + ri) as usize * 3;
                    let r = ri as f32 / (n - 1) as f32;
                    let g = gi as f32 / (n - 1) as f32;
                    let b = bi as f32 / (n - 1) as f32;
                    if (self.data[idx] - r).abs() > tol
                        || (self.data[idx + 1] - g).abs() > tol
                        || (self.data[idx + 2] - b).abs() > tol
                    {
                        return false;
                    }
                }
            }
        }
        true
    }
}

/// Construct the identity LUT of the given `size`. Each cell outputs its
/// own grid coordinates — applying it is a no-op (within rounding).
pub fn identity_lut(size: u32) -> LUT3D {
    let size = size.max(2);
    let n = size as usize;
    let mut data = vec![0.0_f32; LUT3D::expected_len(size)];
    let denom = (size - 1) as f32;
    for bi in 0..n {
        for gi in 0..n {
            for ri in 0..n {
                let idx = ((bi * n + gi) * n + ri) * 3;
                data[idx] = ri as f32 / denom;
                data[idx + 1] = gi as f32 / denom;
                data[idx + 2] = bi as f32 / denom;
            }
        }
    }
    LUT3D { size, data }
}

/// Linearly blend two LUTs of the same size, cell-by-cell: `out = a·(1 −
/// mix) + b·mix`. Used by the dual-LUT panel to combine two looks into
/// one before sampling, so [`apply_lut3d`] runs once per pixel instead
/// of twice.
pub fn blend_luts(a: &LUT3D, b: &LUT3D, mix: f32) -> LUT3D {
    assert_eq!(a.size, b.size, "blend_luts requires equal-sized LUTs");
    assert_eq!(a.data.len(), b.data.len());
    let mix = mix.clamp(0.0, 1.0);
    let inv = 1.0 - mix;
    let data = a
        .data
        .iter()
        .zip(b.data.iter())
        .map(|(av, bv)| av * inv + bv * mix)
        .collect();
    LUT3D { size: a.size, data }
}

/// Apply a 3D LUT to straight-alpha RGBA8 in place. Each pixel maps to a
/// fractional grid coordinate `(rF, gF, bF) = (sRGB · (size − 1))`; the
/// 8 surrounding cells are then interpolated trilinearly. `intensity ∈
/// [0, 1]` blends the result back over the original (`0` = identity, `1`
/// = full LUT). Skips fully transparent pixels.
pub fn apply_lut3d(rgba: &mut [u8], lut: &LUT3D, intensity: f32) {
    if intensity <= 0.0 || lut.size < 2 {
        return;
    }
    let intensity = intensity.clamp(0.0, 1.0);
    let inv_intensity = 1.0 - intensity;
    let n = lut.size as i32;
    let n_minus_1 = (n - 1) as f32;
    let stride_x = 3;
    let stride_y = (n as usize) * 3;
    let stride_z = (n as usize) * (n as usize) * 3;

    for px in rgba.chunks_exact_mut(4) {
        if px[3] == 0 {
            continue;
        }
        let r = px[0] as f32 / 255.0;
        let g = px[1] as f32 / 255.0;
        let b = px[2] as f32 / 255.0;

        let rf = (r * n_minus_1).clamp(0.0, n_minus_1);
        let gf = (g * n_minus_1).clamp(0.0, n_minus_1);
        let bf = (b * n_minus_1).clamp(0.0, n_minus_1);

        let r0 = (rf.floor() as i32).clamp(0, n - 1);
        let g0 = (gf.floor() as i32).clamp(0, n - 1);
        let b0 = (bf.floor() as i32).clamp(0, n - 1);
        let r1 = (r0 + 1).min(n - 1);
        let g1 = (g0 + 1).min(n - 1);
        let b1 = (b0 + 1).min(n - 1);
        let rd = rf - r0 as f32;
        let gd = gf - g0 as f32;
        let bd = bf - b0 as f32;

        let base000 =
            (b0 as usize) * stride_z + (g0 as usize) * stride_y + (r0 as usize) * stride_x;
        let base100 =
            (b0 as usize) * stride_z + (g0 as usize) * stride_y + (r1 as usize) * stride_x;
        let base010 =
            (b0 as usize) * stride_z + (g1 as usize) * stride_y + (r0 as usize) * stride_x;
        let base110 =
            (b0 as usize) * stride_z + (g1 as usize) * stride_y + (r1 as usize) * stride_x;
        let base001 =
            (b1 as usize) * stride_z + (g0 as usize) * stride_y + (r0 as usize) * stride_x;
        let base101 =
            (b1 as usize) * stride_z + (g0 as usize) * stride_y + (r1 as usize) * stride_x;
        let base011 =
            (b1 as usize) * stride_z + (g1 as usize) * stride_y + (r0 as usize) * stride_x;
        let base111 =
            (b1 as usize) * stride_z + (g1 as usize) * stride_y + (r1 as usize) * stride_x;

        let mut out = [0.0_f32; 3];
        for (ch, out_ch) in out.iter_mut().enumerate() {
            let c000 = lut.data[base000 + ch];
            let c100 = lut.data[base100 + ch];
            let c010 = lut.data[base010 + ch];
            let c110 = lut.data[base110 + ch];
            let c001 = lut.data[base001 + ch];
            let c101 = lut.data[base101 + ch];
            let c011 = lut.data[base011 + ch];
            let c111 = lut.data[base111 + ch];
            // Lerp along R.
            let c00 = c000 * (1.0 - rd) + c100 * rd;
            let c10 = c010 * (1.0 - rd) + c110 * rd;
            let c01 = c001 * (1.0 - rd) + c101 * rd;
            let c11 = c011 * (1.0 - rd) + c111 * rd;
            // Lerp along G.
            let c0 = c00 * (1.0 - gd) + c10 * gd;
            let c1 = c01 * (1.0 - gd) + c11 * gd;
            // Lerp along B.
            *out_ch = c0 * (1.0 - bd) + c1 * bd;
        }

        let final_r = r * inv_intensity + clamp01(out[0]) * intensity;
        let final_g = g * inv_intensity + clamp01(out[1]) * intensity;
        let final_b = b * inv_intensity + clamp01(out[2]) * intensity;
        px[0] = (final_r * 255.0 + 0.5).clamp(0.0, 255.0) as u8;
        px[1] = (final_g * 255.0 + 0.5).clamp(0.0, 255.0) as u8;
        px[2] = (final_b * 255.0 + 0.5).clamp(0.0, 255.0) as u8;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn solid(w: u32, h: u32, rgb: [u8; 3]) -> Vec<u8> {
        let mut v = Vec::with_capacity((w * h * 4) as usize);
        for _ in 0..(w * h) {
            v.extend_from_slice(&[rgb[0], rgb[1], rgb[2], 255]);
        }
        v
    }

    #[test]
    fn identity_lut_round_trips_uniform_input() {
        let lut = identity_lut(DEFAULT_LUT_SIZE);
        assert_eq!(lut.size, DEFAULT_LUT_SIZE);
        assert_eq!(lut.data.len(), LUT3D::expected_len(DEFAULT_LUT_SIZE));
        let mut buf = solid(4, 4, [120, 80, 200]);
        let before = buf.clone();
        apply_lut3d(&mut buf, &lut, 1.0);
        // Identity LUT + trilinear interp should reproduce the input
        // within a few LSB (rounding from f32 grid).
        for (a, b) in buf.iter().zip(before.iter()) {
            assert!(a.abs_diff(*b) <= 2, "identity LUT drifted: {a} vs {b}");
        }
    }

    #[test]
    fn identity_lut_is_identity_predicate() {
        let lut = identity_lut(9);
        assert!(lut.is_identity(1e-5));
        // Perturb one cell — predicate should now return false.
        let mut lut = lut;
        lut.data[0] += 0.5;
        assert!(!lut.is_identity(1e-5));
    }

    #[test]
    fn apply_lut3d_zero_intensity_is_noop() {
        let lut = identity_lut(DEFAULT_LUT_SIZE);
        let mut buf = solid(4, 4, [50, 100, 150]);
        let before = buf.clone();
        apply_lut3d(&mut buf, &lut, 0.0);
        assert_eq!(buf, before);
    }

    #[test]
    fn apply_lut3d_skips_transparent_pixels() {
        // Build a LUT that pins everything to white — non-transparent
        // pixels should turn white, transparent ones remain untouched.
        let mut white_lut = identity_lut(9);
        for cell in white_lut.data.chunks_exact_mut(3) {
            cell[0] = 1.0;
            cell[1] = 1.0;
            cell[2] = 1.0;
        }
        let mut buf = vec![100u8, 50, 75, 0, 100, 50, 75, 255];
        apply_lut3d(&mut buf, &white_lut, 1.0);
        assert_eq!(&buf[0..4], &[100, 50, 75, 0]);
        assert_eq!(&buf[4..8], &[255, 255, 255, 255]);
    }

    #[test]
    fn blend_luts_endpoints_match_inputs() {
        let mut a = identity_lut(9);
        // Tint a slightly red so blend has something to interpolate.
        for cell in a.data.chunks_exact_mut(3) {
            cell[0] = (cell[0] + 0.1).min(1.0);
        }
        let b = identity_lut(9);
        let blend_a = blend_luts(&a, &b, 0.0);
        let blend_b = blend_luts(&a, &b, 1.0);
        assert_eq!(blend_a.data, a.data);
        assert_eq!(blend_b.data, b.data);
    }

    #[test]
    fn blend_luts_midpoint_averages_cells() {
        let mut a = identity_lut(9);
        let b = identity_lut(9);
        for cell in a.data.chunks_exact_mut(3) {
            cell[0] = 1.0;
        }
        let mid = blend_luts(&a, &b, 0.5);
        // First cell: r should be (1.0 + 0.0) / 2 = 0.5.
        assert!((mid.data[0] - 0.5).abs() < 1e-5);
    }

    #[test]
    fn apply_lut3d_intensity_blends_back_toward_original() {
        // Build a "everything to black" LUT.
        let mut black_lut = identity_lut(9);
        for v in black_lut.data.iter_mut() {
            *v = 0.0;
        }
        let mut buf = solid(2, 2, [200, 100, 50]);
        // intensity 0.5 → halfway between original and black.
        apply_lut3d(&mut buf, &black_lut, 0.5);
        // R: 0.5·200/255 ≈ 100 ± 1.
        assert!((buf[0] as i32 - 100).abs() <= 2);
        assert!((buf[1] as i32 - 50).abs() <= 2);
        assert!((buf[2] as i32 - 25).abs() <= 2);
    }
}
