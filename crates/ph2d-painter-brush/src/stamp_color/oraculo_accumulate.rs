//! ORÁCULO `#[cfg(test)]` (B29a, B52.R26): `accumulate.rs` VERBATIM do HEAD 937a504585 (só os comentários tirados e os
//! caminhos apontados para os irmãos-oráculo). A paridade ao bit do esqueleto de bandas mede-se contra isto.
#![allow(dead_code, clippy::all)]

use super::{ColorStampMask, grain_modulation, sample_color_mask};
use crate::dab::{DirtyRect, encode};
use crate::spec::BrushSpec;
use crate::texture::{ImageMask, ImageRgb, sample_shape_rgb_unit};

#[must_use]
pub fn accumulate_color_stamp_coverage(
    cov: &mut [u8],
    width: u32,
    height: u32,
    center: [f32; 2],
    radius: f32,
    stamp: &ColorStampMask,
    coverage: f32,
) -> Option<DirtyRect> {
    let coverage = coverage.clamp(0.0, 1.0);
    if coverage <= 0.0 || width == 0 || height == 0 || radius <= 0.0 {
        return None;
    }
    let radius = radius * stamp.reach;
    let stamp = stamp.nivel_para(2.0 * radius);
    let (cx, cy) = (center[0], center[1]);
    let x0 = (cx - radius).floor().max(0.0) as i64;
    let y0 = (cy - radius).floor().max(0.0) as i64;
    let x1 = ((cx + radius).ceil() as i64 + 1).min(width as i64);
    let y1 = ((cy + radius).ceil() as i64 + 1).min(height as i64);
    if x0 >= x1 || y0 >= y1 {
        return None;
    }
    let inv_radius = 1.0 / radius;
    let mut touched = false;
    for py in y0..y1 {
        let v = ((py as f32 + 0.5) - cy) * inv_radius;
        let row = (py as usize) * (width as usize);
        for px in x0..x1 {
            let u = ((px as f32 + 0.5) - cx) * inv_radius;
            let (_, a_tex) = sample_color_mask(stamp, u, v);
            let a = a_tex * coverage;
            if a <= 0.0 {
                continue;
            }
            let idx = row + px as usize;
            let prev = f32::from(cov[idx]) / 255.0;
            cov[idx] = encode(a + prev * (1.0 - a)); // over-accumulate
            touched = true;
        }
    }
    if !touched {
        return None;
    }
    Some(DirtyRect {
        x: x0 as u32,
        y: y0 as u32,
        w: (x1 - x0) as u32,
        h: (y1 - y0) as u32,
    })
}

#[must_use]
pub fn accumulate_color_stamps_fused(
    covs: &mut [Vec<u8>],
    width: u32,
    height: u32,
    center: [f32; 2],
    radius: f32,
    stamps: &[ColorStampMask],
    coverage: f32,
) -> Option<DirtyRect> {
    let coverage = coverage.clamp(0.0, 1.0);
    let n = stamps.len();
    if coverage <= 0.0 || width == 0 || height == 0 || radius <= 0.0 || n == 0 {
        return None;
    }
    debug_assert_eq!(covs.len(), n, "one coverage map per layer");
    debug_assert!(
        stamps.iter().all(|s| s.size == stamps[0].size),
        "fused accumulate requires all stamps baked at the same size"
    );
    let radius = radius * stamps[0].reach;
    let k = stamps[0].indice_do_nivel(2.0 * radius);
    let size = stamps[0].nivel(k).size;
    let (cx, cy) = (center[0], center[1]);
    let x0 = (cx - radius).floor().max(0.0) as i64;
    let y0 = (cy - radius).floor().max(0.0) as i64;
    let x1 = ((cx + radius).ceil() as i64 + 1).min(width as i64);
    let y1 = ((cy + radius).ceil() as i64 + 1).min(height as i64);
    if x0 >= x1 || y0 >= y1 {
        return None;
    }
    let inv_radius = 1.0 / radius;
    let sz = size as usize;
    let mut touched = false;
    for py in y0..y1 {
        let v = ((py as f32 + 0.5) - cy) * inv_radius;
        let Some((ty0, ty1, ty)) = crate::tip::axis_taps(v, size) else {
            continue;
        };
        let (yr0, yr1) = (ty0 * sz, ty1 * sz);
        let row = (py as usize) * (width as usize);
        for px in x0..x1 {
            let u = ((px as f32 + 0.5) - cx) * inv_radius;
            let Some((xc0, xc1, tx)) = crate::tip::axis_taps(u, size) else {
                continue;
            };
            let o00 = (yr0 + xc0) * 4 + 3;
            let o10 = (yr0 + xc1) * 4 + 3;
            let o01 = (yr1 + xc0) * 4 + 3;
            let o11 = (yr1 + xc1) * 4 + 3;
            let idx = row + px as usize;
            for (i, stamp) in stamps.iter().enumerate() {
                let d = &stamp.nivel(k).data;
                let c = |o: usize| f32::from(d[o]) / 255.0;
                let top = c(o00) + (c(o10) - c(o00)) * tx;
                let bot = c(o01) + (c(o11) - c(o01)) * tx;
                let a_tex = top + (bot - top) * ty;
                let a = a_tex * coverage;
                if a <= 0.0 {
                    continue;
                }
                let cov = &mut covs[i];
                let prev = f32::from(cov[idx]) / 255.0;
                cov[idx] = encode(a + prev * (1.0 - a)); // over-accumulate
                touched = true;
            }
        }
    }
    if !touched {
        return None;
    }
    Some(DirtyRect {
        x: x0 as u32,
        y: y0 as u32,
        w: (x1 - x0) as u32,
        h: (y1 - y0) as u32,
    })
}

#[allow(clippy::too_many_arguments)]
#[must_use]
pub fn accumulate_shape_layer_rgba(
    acc: &mut [u8],
    width: u32,
    height: u32,
    center: [f32; 2],
    radius: f32,
    spec: &BrushSpec,
    shape_basis: &crate::texture::TexDabBasis,
    layer: &ImageMask,
    grain: Option<(&crate::texture::TexDabBasis, Option<&ImageMask>)>,
    grain_ramp: Option<&[[f32; 4]]>,
    color: [f32; 3],
    rgb: Option<&ImageRgb>,
    color_shift: [f32; 3],
    coverage: f32,
) -> Option<DirtyRect> {
    let coverage = coverage.clamp(0.0, 1.0);
    if coverage <= 0.0 || width == 0 || height == 0 || radius <= 0.0 {
        return None;
    }
    let depth = spec.grain_depth();
    let (cx, cy) = (center[0], center[1]);
    let [ex, ey] = crate::tip::dab_extent([1.0, 1.0], &spec.shape, Some(shape_basis));
    let (rx, ry) = (radius * ex, radius * ey);
    let x0 = (cx - rx).floor().max(0.0) as i64;
    let y0 = (cy - ry).floor().max(0.0) as i64;
    let x1 = ((cx + rx).ceil() as i64 + 1).min(width as i64);
    let y1 = ((cy + ry).ceil() as i64 + 1).min(height as i64);
    if x0 >= x1 || y0 >= y1 {
        return None;
    }
    let inv_radius = 1.0 / radius;
    let mut touched = false;
    for py in y0..y1 {
        let v = ((py as f32 + 0.5) - cy) * inv_radius;
        let row = (py as usize) * (width as usize);
        for px in x0..x1 {
            let u = ((px as f32 + 0.5) - cx) * inv_radius;
            let mut a = crate::texture::sample_shape_silhouette_unit_px(
                &spec.shape,
                shape_basis,
                u,
                v,
                inv_radius,
                Some(layer),
            );
            if a <= 0.0 {
                continue;
            }
            let base = match rgb {
                Some(img) => super::texture_times_tint(
                    sample_shape_rgb_unit(&spec.shape, shape_basis, u, v, img),
                    color,
                    color_shift,
                ),
                None => color,
            };
            let mut tint = base;
            if let Some((gb, gi)) = grain {
                let g = if spec.texture.mapping.is_canvas_fixed() {
                    crate::texture::sample(&spec.texture, gb, px, py, center, radius, gi)
                } else {
                    crate::texture::sample_unit_px(&spec.texture, gb, u, v, inv_radius, gi)
                };
                let m = grain_modulation(
                    g,
                    depth,
                    spec.effective_granulation(),
                    grain_ramp,
                    &spec.texture.lei,
                );
                tint = [base[0] * m[0], base[1] * m[1], base[2] * m[2]];
                a *= m[3];
                a *= crate::texture::stencil_gate(&spec.texture, gb, px, py);
            }
            a *= crate::dente::dente_at(spec, a, px, py);
            a *= coverage;
            if a <= 0.0 {
                continue;
            }
            let o = (row + px as usize) * 4;
            let inv = 1.0 - a;
            acc[o] = encode(tint[0] * a + f32::from(acc[o]) / 255.0 * inv);
            acc[o + 1] = encode(tint[1] * a + f32::from(acc[o + 1]) / 255.0 * inv);
            acc[o + 2] = encode(tint[2] * a + f32::from(acc[o + 2]) / 255.0 * inv);
            acc[o + 3] = encode(a + f32::from(acc[o + 3]) / 255.0 * inv);
            touched = true;
        }
    }
    if !touched {
        return None;
    }
    Some(DirtyRect {
        x: x0 as u32,
        y: y0 as u32,
        w: (x1 - x0) as u32,
        h: (y1 - y0) as u32,
    })
}
