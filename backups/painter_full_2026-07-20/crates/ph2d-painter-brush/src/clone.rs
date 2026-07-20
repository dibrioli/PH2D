//! Clone one dab — copy canvas pixels from a fixed source OFFSET to the dab position (the clone stamp).
//!
//! **Clean-room** behaviour of Blender's 2D image-paint Clone
//! (`paint_image_2d.cc`: `paint_2d_clone` lifts the footprint region from `dest − clone_offset·size`
//! and composites it with `IMB_BLEND_INTERPOLATE`, masked by the brush falloff × strength) ≡ the
//! Photoshop/GIMP/Krita **Clone Stamp**: sample a source point, then paint the source region at the
//! cursor, keeping a fixed source→dest offset. Only the algorithm is ported, never the code.
//!
//! The lift-and-blend sibling of [`crate::smear_dab`]: Smear lifts from the *previous dab* (a per-step
//! displacement); Clone lifts from a **fixed offset** `source = dest + offset` (offset established when
//! the stroke begins, from the sampled source anchor). Unlike Smear it needs no motion — a stationary
//! dab clones its footprint. The source region overlaps the destination, so it is snapshotted first;
//! writes never feed back into later reads within a dab. `wrap` (Tiling) makes the lift toroidal.

use crate::blur::footprint_bbox;
use crate::dab::DirtyRect;
use crate::spec::BrushSpec;
use crate::stamp::{StampMask, sample_mask};

/// Snapshot the source region for a dab: for each destination cell `(min_x+i, min_y+j)` read the canvas
/// at `dest + off` (integer offset). A `wrap` axis reads toroidally (`rem_euclid`); a non-wrap axis
/// whose source is off-canvas stays transparent-zero (the dab rim, falloff ~0 — nil effect), mirroring
/// [`crate::smear_dab`]'s lift. Returns `bw·bh` straight-RGBA8 cells.
#[allow(clippy::too_many_arguments)]
pub(crate) fn lift_offset(
    buf: &[u8],
    fw: i64,
    fh: i64,
    min_x: i64,
    min_y: i64,
    bw: usize,
    bh: usize,
    off: [i64; 2],
    wrap: [bool; 2],
) -> Vec<[u8; 4]> {
    let mut lifted = vec![[0u8; 4]; bw * bh];
    for j in 0..bh {
        let sy = min_y + j as i64 + off[1];
        let sy = if wrap[1] {
            sy.rem_euclid(fh)
        } else if sy < 0 || sy >= fh {
            continue;
        } else {
            sy
        };
        for i in 0..bw {
            let sx = min_x + i as i64 + off[0];
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
    lifted
}

/// Clone one dab centred at `center`, sampling the canvas at `center + offset` (per pixel), weighted by
/// the brush falloff (with `spec.hardness`) × `strength`. The round-falloff path (no Shape / Grain /
/// flatten). `offset` is the source→dest vector in image px (the sampled source minus the stroke's
/// start). `strength` in `[0, 1]` is the blend-back amount. Returns `None` for zero strength/radius or
/// a fully off-canvas footprint. Channels interpolated straight, like [`crate::smear_dab`].
#[allow(clippy::too_many_arguments)]
#[must_use]
pub fn clone_dab(
    buf: &mut [u8],
    width: u32,
    height: u32,
    center: [f32; 2],
    offset: [f32; 2],
    spec: &BrushSpec,
    strength: f32,
    wrap: [bool; 2],
) -> Option<DirtyRect> {
    let radius = spec.clamped_radius();
    let strength = strength.clamp(0.0, 1.0);
    if strength <= 0.0 || radius <= 0.0 {
        return None;
    }
    let (fw, fh) = (width as i64, height as i64);
    let (min_x, min_y, bw, bh) = footprint_bbox(center, radius, fw, fh, 0)?;
    let off = [offset[0].round() as i64, offset[1].round() as i64];
    let lifted = lift_offset(buf, fw, fh, min_x, min_y, bw, bh, off, wrap);
    let inv_r = 1.0 / radius;
    for j in 0..bh {
        let y = min_y + j as i64;
        let dy = y as f32 + 0.5 - center[1];
        for i in 0..bw {
            let x = min_x + i as i64;
            let dx = x as f32 + 0.5 - center[0];
            let t = (dx * dx + dy * dy).sqrt() * inv_r;
            if t >= 1.0 {
                continue;
            }
            let w = spec.falloff_weight(t) * strength;
            if w <= 0.0 {
                continue;
            }
            let src = lifted[j * bw + i];
            let di = ((y * fw + x) * 4) as usize;
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

/// Clone one dab using a pre-rendered [`StampMask`] as the per-pixel weight — so the brush **Shape**
/// (silhouette), **Grain**, and **dab flatten/rotate** all shape the clone exactly as they shape a
/// painted dab (the mask bakes silhouette × Grain × flatten/rotate). The [`crate::blit_stamp`] sibling
/// for cloning: `radius` is the dab radius (the mask spans `[-1, 1]²`), `offset` the source→dest vector,
/// `strength` scales the blend-back. Snapshots the source region first (no read/write feedback).
#[allow(clippy::too_many_arguments)]
#[must_use]
pub fn clone_blit_stamp(
    buf: &mut [u8],
    width: u32,
    height: u32,
    center: [f32; 2],
    offset: [f32; 2],
    radius: f32,
    mask: &StampMask,
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
    let inv_r = 1.0 / radius;
    for j in 0..bh {
        let y = min_y + j as i64;
        let v = (y as f32 + 0.5 - center[1]) * inv_r;
        for i in 0..bw {
            let x = min_x + i as i64;
            let u = (x as f32 + 0.5 - center[0]) * inv_r;
            let w = sample_mask(mask, u, v) * strength;
            if w <= 0.0 {
                continue;
            }
            let src = lifted[j * bw + i];
            let di = ((y * fw + x) * 4) as usize;
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

    fn px(buf: &[u8], w: u32, x: u32, y: u32) -> [u8; 4] {
        let i = ((y * w + x) * 4) as usize;
        [buf[i], buf[i + 1], buf[i + 2], buf[i + 3]]
    }

    /// Left half red, right half blue (opaque). Cloning at a blue pixel with an offset pointing LEFT
    /// into the red half copies red onto blue — the clone stamp.
    fn red_blue(w: u32, h: u32) -> Vec<u8> {
        let mut buf = vec![0u8; (w * h * 4) as usize];
        for y in 0..h {
            for x in 0..w {
                let i = ((y * w + x) * 4) as usize;
                let c = if x < w / 2 {
                    [255, 0, 0, 255]
                } else {
                    [0, 0, 255, 255]
                };
                buf[i..i + 4].copy_from_slice(&c);
            }
        }
        buf
    }

    #[test]
    fn clones_source_pixels_at_the_offset() {
        let (w, h) = (32u32, 16u32);
        let mut buf = red_blue(w, h);
        let spec = BrushSpec {
            radius_px: 4.0,
            hardness: 1.0,
            ..Default::default()
        };
        // Dab at x=24 (blue), offset −16 → source at x=8 (red). Blue must gain red.
        let dirty = clone_dab(
            &mut buf,
            w,
            h,
            [24.0, 8.0],
            [-16.0, 0.0],
            &spec,
            1.0,
            [false, false],
        )
        .expect("in-bounds clone paints");
        let p = px(&buf, w, 24, 8);
        assert!(p[0] > 0, "red cloned into the blue half: {p:?}");
        assert!(dirty.w > 0 && dirty.h > 0);
    }

    #[test]
    fn strength_scales_the_clone() {
        let (w, h) = (32u32, 16u32);
        let spec = BrushSpec {
            radius_px: 4.0,
            hardness: 1.0,
            ..Default::default()
        };
        let mut lo = red_blue(w, h);
        let _ = clone_dab(
            &mut lo,
            w,
            h,
            [24.0, 8.0],
            [-16.0, 0.0],
            &spec,
            0.25,
            [false, false],
        );
        let mut hi = red_blue(w, h);
        let _ = clone_dab(
            &mut hi,
            w,
            h,
            [24.0, 8.0],
            [-16.0, 0.0],
            &spec,
            1.0,
            [false, false],
        );
        assert!(
            px(&hi, w, 24, 8)[0] > px(&lo, w, 24, 8)[0],
            "higher strength clones more red"
        );
    }

    #[test]
    fn fully_offscreen_is_none() {
        let (w, h) = (16u32, 16u32);
        let mut buf = vec![255u8; (w * h * 4) as usize];
        let spec = BrushSpec {
            radius_px: 3.0,
            ..Default::default()
        };
        assert!(
            clone_dab(
                &mut buf,
                w,
                h,
                [-50.0, -50.0],
                [4.0, 0.0],
                &spec,
                1.0,
                [false, false]
            )
            .is_none()
        );
    }

    #[test]
    fn masked_clone_uses_the_stamp_mask() {
        let (w, h) = (32u32, 16u32);
        let mut buf = red_blue(w, h);
        let spec = BrushSpec {
            radius_px: 4.0,
            hardness: 1.0,
            ..Default::default()
        };
        let mask = crate::render_stamp_mask(&spec, None, None, None, 64);
        let dirty = clone_blit_stamp(
            &mut buf,
            w,
            h,
            [24.0, 8.0],
            [-16.0, 0.0],
            4.0,
            &mask,
            1.0,
            [false, false],
        )
        .expect("masked clone paints");
        assert!(px(&buf, w, 24, 8)[0] > 0, "red cloned via the mask");
        assert!(dirty.w > 0 && dirty.h > 0);
    }

    #[test]
    fn wrapping_clone_reads_across_the_seam() {
        // A source offset past the wrapped edge reads from the opposite edge (seamless tile), not
        // transparent. Left column red on an otherwise-blue opaque canvas; a clone near the RIGHT edge
        // with a +offset that runs past the right edge wraps to the left column → picks up red.
        let (w, h) = (16u32, 8u32);
        let buf = red_blue(w, h);
        let spec = BrushSpec {
            radius_px: 2.0,
            hardness: 1.0,
            ..Default::default()
        };
        // Dab at x=15 (right edge, blue), offset +2 → source x=17 → wraps to x=1 (red half).
        let mut wrapped = buf.clone();
        let _ = clone_dab(
            &mut wrapped,
            w,
            h,
            [15.0, 4.0],
            [2.0, 0.0],
            &spec,
            1.0,
            [true, false],
        );
        let mut plain = buf.clone();
        let _ = clone_dab(
            &mut plain,
            w,
            h,
            [15.0, 4.0],
            [2.0, 0.0],
            &spec,
            1.0,
            [false, false],
        );
        assert_ne!(
            px(&wrapped, w, 15, 4),
            px(&plain, w, 15, 4),
            "the wrapped clone reads across the seam, the clamped one skips off-canvas"
        );
    }
}
