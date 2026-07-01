//! Canvas-fixed **Grain** smear — the Tiled / Stencil Grain Mapping path.
//!
//! Split from [`crate::smear`] for the workspace LOC cap. The scale-invariant [`crate::StampMask`]
//! (used by [`crate::smear_blit_stamp`]) only covers the View Grain Mapping; a canvas-fixed Grain
//! (Tiled / Stencil) depends on the canvas position, so it is sampled per pixel here — the smear
//! sibling of [`crate::blit_canvas_cached`].

use crate::dab::DirtyRect;
use crate::spec::BrushSpec;
use crate::texture::ImageMask;

/// Smear one dab whose **Grain Mapping is canvas-fixed** (Tiled / Stencil) — its value depends on the
/// canvas position, so it can't be baked into the scale-invariant [`crate::StampMask`] that
/// [`crate::smear_blit_stamp`] uses. Computes the per-pixel weight `silhouette × Grain` (matching
/// [`crate::blit_canvas_cached`], the paint path for these mappings) and drags with it, so the Grain
/// Mapping shapes the smear like it shapes a painted dab. The silhouette is the Shape image (when
/// active) or the falloff; both are dab-relative (View-static here — per-dab Rake/Random uses the
/// static frame, an accepted approximation). Grain sampled per pixel (no canvas cache — a perf
/// follow-up). Toroidal lift on `wrap` axes, like [`crate::smear_dab`].
#[allow(clippy::too_many_arguments)]
#[must_use]
pub fn smear_blit_grain(
    buf: &mut [u8],
    width: u32,
    height: u32,
    from: [f32; 2],
    to: [f32; 2],
    radius: f32,
    spec: &BrushSpec,
    grain_image: Option<&ImageMask>,
    shape_image: Option<&ImageMask>,
    shape_ramp_lut: Option<&[f32]>,
    strength: f32,
    wrap: [bool; 2],
) -> Option<DirtyRect> {
    let strength = strength.clamp(0.0, 1.0);
    if strength <= 0.0 || radius <= 0.0 {
        return None;
    }
    let step_x = (to[0].round() as i64) - (from[0].round() as i64);
    let step_y = (to[1].round() as i64) - (from[1].round() as i64);
    if step_x == 0 && step_y == 0 {
        return None;
    }
    let fw = width as i64;
    let fh = height as i64;
    let (cx, cy) = (to[0], to[1]);
    let min_x = (cx - radius).floor().max(0.0) as i64;
    let min_y = (cy - radius).floor().max(0.0) as i64;
    let max_x = ((cx + radius).ceil() as i64 + 1).min(fw);
    let max_y = ((cy + radius).ceil() as i64 + 1).min(fh);
    if max_x <= min_x || max_y <= min_y {
        return None;
    }
    let bw = (max_x - min_x) as usize;
    let bh = (max_y - min_y) as usize;

    let footprint = spec.footprint_deform();
    let grain_active = spec.texture.is_active();
    let depth = spec.grain_depth();
    // Canvas-fixed Grain frame (identity footprint — a canvas-locked texture isn't deformed by the dab
    // flatten); the Shape frame is dab-relative. Static bases (rng unused for the cached family).
    let basis = crate::texture::dab_basis(
        &spec.texture,
        [0.0, 0.0],
        &mut 0u64,
        [width as f32, height as f32],
        [1.0, 0.0],
        crate::footprint::FootprintDeform::identity(),
    );
    let shape_basis = spec
        .shape_silhouette_active(shape_image.is_some())
        .then(|| {
            crate::texture::dab_basis(
                &spec.shape,
                [0.0, 0.0],
                &mut 0u64,
                [1.0, 1.0],
                [1.0, 0.0],
                footprint,
            )
        });

    // Lift snapshot (toroidal on wrap axes — see [`crate::smear_dab`]).
    let mut lifted = vec![[0u8; 4]; bw * bh];
    for j in 0..bh {
        let sy = min_y + j as i64 - step_y;
        let sy = if wrap[1] {
            sy.rem_euclid(fh)
        } else if sy < 0 || sy >= fh {
            continue;
        } else {
            sy
        };
        for i in 0..bw {
            let sx = min_x + i as i64 - step_x;
            let sx = if wrap[0] {
                sx.rem_euclid(fw)
            } else if sx < 0 || sx >= fw {
                continue;
            } else {
                sx
            };
            let si = ((sy * fw + sx) * 4) as usize;
            lifted[j * bw + i] = [buf[si], buf[si + 1], buf[si + 2], buf[si + 3]];
        }
    }

    let inv_r = 1.0 / radius;
    for j in 0..bh {
        let py = min_y + j as i64;
        let dy = (py as f32 + 0.5) - cy;
        for i in 0..bw {
            let px = min_x + i as i64;
            let dx = (px as f32 + 0.5) - cx;
            // Silhouette: Shape image (dab-relative) or the falloff, with the dab footprint.
            let mut w = match &shape_basis {
                Some(sb) => {
                    let raw = crate::texture::sample_shape_silhouette(
                        &spec.shape,
                        sb,
                        px,
                        py,
                        [cx, cy],
                        radius,
                        shape_image,
                    );
                    let sv = crate::texture::remap_shape_value(raw, shape_ramp_lut);
                    let t = footprint.falloff_t(dx * inv_r, dy * inv_r);
                    spec.compose_shape_silhouette(sv, spec.falloff_weight(t))
                }
                None => {
                    let t = footprint.falloff_t(dx * inv_r, dy * inv_r);
                    spec.falloff_weight(t)
                }
            };
            if w <= 0.0 {
                continue;
            }
            if grain_active {
                let g = crate::texture::sample(
                    &spec.texture,
                    &basis,
                    px,
                    py,
                    [cx, cy],
                    radius,
                    grain_image,
                );
                w *= if depth >= 1.0 {
                    g
                } else {
                    1.0 + (g - 1.0) * depth
                };
            }
            let w = w * strength;
            if w <= 0.0 {
                continue;
            }
            let src = lifted[j * bw + i];
            let di = ((py * fw + px) * 4) as usize;
            for c in 0..4 {
                let d = f32::from(buf[di + c]);
                let s = f32::from(src[c]);
                buf[di + c] = (d + (s - d) * w).round().clamp(0.0, 255.0) as u8;
            }
        }
    }
    Some(DirtyRect {
        x: min_x as u32,
        y: min_y as u32,
        w: bw as u32,
        h: bh as u32,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::texture::{TextureKind, TextureMapping};

    #[test]
    fn canvas_fixed_grain_changes_the_smear() {
        // A Tiled (canvas-fixed) Checker Grain must shape the smear — proven by comparing against the
        // plain round smear on the same input: the Grain path yields a DIFFERENT result (it isn't
        // ignored, the earlier bug). Opaque left half, transparent right; smear drags across x=20.
        let (w, h) = (40u32, 12u32);
        let mut base = vec![255u8; (w * h * 4) as usize];
        for y in 0..h {
            for x in 20..w {
                let i = ((y * w + x) * 4) as usize;
                base[i..i + 4].copy_from_slice(&[0, 0, 0, 0]);
            }
        }
        let mut spec = BrushSpec {
            radius_px: 8.0,
            hardness: 1.0,
            ..Default::default()
        };
        spec.texture.kind = TextureKind::Checker;
        spec.texture.mapping = TextureMapping::Tiled;
        assert!(spec.texture.is_active(), "Checker Grain is active");

        let mut with_grain = base.clone();
        let dirty = smear_blit_grain(
            &mut with_grain,
            w,
            h,
            [18.0, 6.0],
            [22.0, 6.0],
            8.0,
            &spec,
            None,
            None,
            None,
            1.0,
            [false, false],
        );
        assert!(dirty.is_some(), "grain smear paints");
        // It dragged opaque pixels into the transparent half.
        let dragged = (20..28).any(|x| with_grain[((6 * w + x) * 4 + 3) as usize] > 0);
        assert!(dragged, "grain smear dragged content across the boundary");
        // And the Grain changed the outcome vs. the plain round smear (Grain applied, not ignored).
        let mut plain = base.clone();
        let _ = crate::smear_dab(
            &mut plain,
            w,
            h,
            [18.0, 6.0],
            [22.0, 6.0],
            &BrushSpec {
                radius_px: 8.0,
                hardness: 1.0,
                ..Default::default()
            },
            1.0,
            [false, false],
        );
        assert!(
            with_grain != plain,
            "the canvas-fixed Grain shapes the smear"
        );
    }
}
