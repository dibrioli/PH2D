//! Canvas-fixed **Grain** clone — the Tiled / Stencil Grain Mapping path (and per-dab rotation).
//!
//! Split from [`crate::clone`] for the workspace LOC cap. The scale-invariant [`crate::StampMask`]
//! (used by [`crate::clone_blit_stamp`]) only covers the View Grain Mapping and a constant orientation;
//! a canvas-fixed Grain (Tiled / Stencil) depends on the canvas position, and Rake / Random / Jitter
//! Rotate change the frame per dab — so the weight is sampled per pixel here, the clone sibling of
//! [`crate::smear_blit_grain`] / [`crate::blit_canvas_cached`].

use crate::blur::footprint_bbox;
use crate::clone::lift_offset;
use crate::dab::DirtyRect;
use crate::footprint::FootprintDeform;
use crate::spec::BrushSpec;
use crate::texture::{ImageMask, TexDabBasis};

/// Clone one dab whose **Grain Mapping is canvas-fixed** (Tiled / Stencil) or whose Shape/Grain rotates
/// per dab (Rake / Random / Jitter Rotate) — cases the scale-invariant [`crate::StampMask`] can't bake.
/// Lifts the source region at `center + offset` and blends it with the per-pixel weight
/// `silhouette × Grain × strength` (matching [`crate::blit_canvas_cached`], the paint path for these
/// mappings), so the Grain Mapping and per-dab rotation shape the clone like they shape a painted dab.
/// The silhouette is the Shape image (when active) or the falloff. The caller supplies the per-dab
/// `footprint` (Jitter-Rotate) and the Grain / Shape `TexDabBasis` (Rake heading + Random draw).
/// `grain_basis` `None` ⇒ no Grain modulation; `shape_basis` `None` ⇒ the falloff is the silhouette.
/// Toroidal source lift on `wrap` axes, like [`crate::clone_dab`].
#[allow(clippy::too_many_arguments)]
#[must_use]
pub fn clone_blit_grain(
    buf: &mut [u8],
    width: u32,
    height: u32,
    center: [f32; 2],
    offset: [f32; 2],
    radius: f32,
    spec: &BrushSpec,
    footprint: FootprintDeform,
    grain_basis: Option<&TexDabBasis>,
    shape_basis: Option<&TexDabBasis>,
    grain_image: Option<&ImageMask>,
    shape_image: Option<&ImageMask>,
    shape_ramp_lut: Option<&[f32]>,
    grain_ramp_lut: Option<&[f32]>,
    strength: f32,
    wrap: [bool; 2],
) -> Option<DirtyRect> {
    let strength = strength.clamp(0.0, 1.0);
    if strength <= 0.0 || radius <= 0.0 {
        return None;
    }
    let (fw, fh) = (width as i64, height as i64);
    let (min_x, min_y, bw, bh) = footprint_bbox(center, radius, fw, fh, 1)?;
    let off = [offset[0].round() as i64, offset[1].round() as i64];
    let lifted = lift_offset(buf, fw, fh, min_x, min_y, bw, bh, off, wrap);

    let depth = spec.grain_depth();
    let (cx, cy) = (center[0], center[1]);
    let inv_r = 1.0 / radius;
    for j in 0..bh {
        let py = min_y + j as i64;
        let dy = (py as f32 + 0.5) - cy;
        for i in 0..bw {
            let px = min_x + i as i64;
            let dx = (px as f32 + 0.5) - cx;
            // Silhouette: Shape image (dab-relative) or the falloff, with the dab footprint.
            let mut w = match shape_basis {
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
            if let Some(gb) = grain_basis {
                let raw = crate::texture::sample(
                    &spec.texture,
                    gb,
                    px,
                    py,
                    [cx, cy],
                    radius,
                    grain_image,
                );
                // A B&W Grain ramp (Smear/Blur/Clone) remaps the grain into a coverage tone.
                let g = crate::texture::remap_shape_value(raw, grain_ramp_lut);
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
    fn canvas_fixed_grain_changes_the_clone() {
        // A Tiled (canvas-fixed) Checker Grain must shape the clone — proven by comparing against the
        // plain round clone on the same input: the Grain path yields a DIFFERENT result. Left red,
        // right blue; clone at a blue pixel with a −offset into the red half.
        let (w, h) = (40u32, 12u32);
        let mut base = vec![0u8; (w * h * 4) as usize];
        for y in 0..h {
            for x in 0..w {
                let i = ((y * w + x) * 4) as usize;
                let c = if x < 20 {
                    [255, 0, 0, 255]
                } else {
                    [0, 0, 255, 255]
                };
                base[i..i + 4].copy_from_slice(&c);
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

        let fp = spec.footprint_deform();
        let gb = crate::texture::dab_basis(
            &spec.texture,
            &mut 0u64,
            [w as f32, h as f32],
            crate::footprint::FootprintDeform::identity(),
        );
        let mut with_grain = base.clone();
        let dirty = clone_blit_grain(
            &mut with_grain,
            w,
            h,
            [28.0, 6.0],
            [-16.0, 0.0],
            8.0,
            &spec,
            fp,
            Some(&gb),
            None,
            None,
            None,
            None,
            None,
            1.0,
            [false, false],
        );
        assert!(dirty.is_some(), "grain clone paints");

        let mut plain = base.clone();
        let _ = crate::clone_dab(
            &mut plain,
            w,
            h,
            [28.0, 6.0],
            [-16.0, 0.0],
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
            "the canvas-fixed Grain shapes the clone"
        );
    }
}
