//! Smear one dab — drag the canvas content along the stroke.
//!
//! **Clean-room** behaviour of Blender's 2D image-paint Smear
//! (`editors/sculpt_paint/mesh/paint_image_2d.cc`: `paint_2d_lift_smear` lifts the footprint region
//! from the *previous* dab position, then blends it at the *current* position with
//! `IMB_BLEND_INTERPOLATE`, masked by the brush falloff × strength). This is the community-accepted
//! "Smear"/Smearing algorithm — Krita's Color Smudge "Smearing" mode is the same idea ("copies the
//! area underneath the previous position of the brush onto the new position, taking opacity into
//! account"). Only the algorithm is ported, never the code.
//!
//! It drags the real pixels (preserves texture), unlike the MyPaint/Krita-"Dulling" running-colour
//! smudge (a colour accumulator) — that is a separate flavour we could add as a mode later.

use crate::dab::DirtyRect;
use crate::spec::BrushSpec;

/// Smear one dab: pull the canvas content from `from` toward `to`, inside the dab footprint centred
/// at `to`, weighted by the brush falloff (with `spec.hardness`) × `strength`.
///
/// `buf` is straight-alpha RGBA8, row-major, `width * height * 4` bytes (the layer's native space,
/// matching [`crate::stamp_dab`]). `from`/`to` are image-space pixel coords — the previous and current
/// dab centres. `strength` in `[0, 1]` is the smear amount (Blender's `mask_max`): `1` fully drags,
/// lower is softer. The pixel displacement is the integer `round(to) - round(from)` (Blender walks in
/// integer brush coords). Returns `None` when there is no movement, zero strength/radius, or the
/// footprint is fully off-canvas — mirroring Blender's early-out.
///
/// The source region overlaps the destination (the two footprints are one step apart), so the source
/// pixels are **lifted into a snapshot first**; writes never feed back into later reads within a dab.
/// Channels are interpolated straight (per Blender/Krita Smearing); on a mostly-opaque canvas this is
/// exact, and it drags alpha too.
#[must_use]
pub fn smear_dab(
    buf: &mut [u8],
    width: u32,
    height: u32,
    from: [f32; 2],
    to: [f32; 2],
    spec: &BrushSpec,
    strength: f32,
) -> Option<DirtyRect> {
    let radius = spec.clamped_radius();
    let strength = strength.clamp(0.0, 1.0);
    if strength <= 0.0 || radius <= 0.0 {
        return None;
    }
    // Integer drag vector (Blender lifts/stamps in integer brush coords). No move ⇒ no smear.
    let step_x = (to[0].round() as i64) - (from[0].round() as i64);
    let step_y = (to[1].round() as i64) - (from[1].round() as i64);
    if step_x == 0 && step_y == 0 {
        return None;
    }

    let fw = width as i64;
    let fh = height as i64;
    // Footprint bbox at the destination (`to`), clamped to the canvas. Half-open [min, max).
    let min_x = (to[0] - radius).floor().max(0.0) as i64;
    let min_y = (to[1] - radius).floor().max(0.0) as i64;
    let max_x = ((to[0] + radius).ceil() as i64).min(fw);
    let max_y = ((to[1] + radius).ceil() as i64).min(fh);
    if max_x <= min_x || max_y <= min_y {
        return None;
    }
    let bw = (max_x - min_x) as usize;
    let bh = (max_y - min_y) as usize;

    // Lift: snapshot the source pixel (canvas at `dest - step`) for every dest cell; cells whose
    // source is off-canvas stay transparent-zero and never contribute (weight applied below still
    // lerps toward zero there, matching Blender's zeroed out-of-image regions — but those are at the
    // dab rim where the falloff is ~0, so the visible effect is nil).
    let mut lifted = vec![[0u8; 4]; bw * bh];
    for j in 0..bh {
        let sy = min_y + j as i64 - step_y;
        if sy < 0 || sy >= fh {
            continue;
        }
        for i in 0..bw {
            let sx = min_x + i as i64 - step_x;
            if sx < 0 || sx >= fw {
                continue;
            }
            let si = ((sy * fw + sx) * 4) as usize;
            lifted[j * bw + i] = [buf[si], buf[si + 1], buf[si + 2], buf[si + 3]];
        }
    }

    // Blend: dest = lerp(dest, lifted, w), w = falloff(dist/radius) × strength.
    let inv_r = 1.0 / radius;
    for j in 0..bh {
        let y = min_y + j as i64;
        let dy = y as f32 + 0.5 - to[1];
        for i in 0..bw {
            let x = min_x + i as i64;
            let dx = x as f32 + 0.5 - to[0];
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
                let d = buf[di + c] as f32;
                let s = src[c] as f32;
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

    fn canvas(w: u32, h: u32) -> Vec<u8> {
        vec![0u8; (w * h * 4) as usize]
    }
    fn px(buf: &[u8], w: u32, x: u32, y: u32) -> [u8; 4] {
        let i = ((y * w + x) * 4) as usize;
        [buf[i], buf[i + 1], buf[i + 2], buf[i + 3]]
    }
    fn set(buf: &mut [u8], w: u32, x: u32, y: u32, c: [u8; 4]) {
        let i = ((y * w + x) * 4) as usize;
        buf[i..i + 4].copy_from_slice(&c);
    }

    #[test]
    fn no_movement_is_noop() {
        let (w, h) = (16, 16);
        let mut buf = canvas(w, h);
        set(&mut buf, w, 8, 8, [255, 0, 0, 255]);
        let spec = BrushSpec {
            radius_px: 4.0,
            ..Default::default()
        };
        assert!(smear_dab(&mut buf, w, h, [8.0, 8.0], [8.0, 8.0], &spec, 1.0).is_none());
        assert_eq!(px(&buf, w, 8, 8), [255, 0, 0, 255], "buffer untouched");
    }

    #[test]
    fn drags_colour_in_the_direction_of_motion() {
        // Left half opaque white, right half transparent. Smear rightward across the boundary at
        // x=8 must pull white INTO the right (transparent) half.
        let (w, h) = (32, 16);
        let mut buf = canvas(w, h);
        for y in 0..h {
            for x in 0..8 {
                set(&mut buf, w, x, y, [255, 255, 255, 255]);
            }
        }
        let spec = BrushSpec {
            radius_px: 6.0,
            hardness: 1.0, // hard disk so the rim weight is full up to the edge
            ..Default::default()
        };
        // Step +3px to the right, footprint centred at the boundary.
        let dirty = smear_dab(&mut buf, w, h, [8.0, 8.0], [11.0, 8.0], &spec, 1.0)
            .expect("in-bounds moving smear paints");
        // A pixel just right of the old boundary gained white (was transparent).
        let p = px(&buf, w, 9, 8);
        assert!(p[3] > 0 && p[0] > 0, "white dragged rightward: {p:?}");
        assert!(dirty.w > 0 && dirty.h > 0);
    }

    #[test]
    fn strength_scales_the_pull() {
        // Same setup; a low strength pulls less alpha than a high one at the same pixel.
        let make = || {
            let (w, h) = (32u32, 16u32);
            let mut buf = canvas(w, h);
            for y in 0..h {
                for x in 0..8 {
                    set(&mut buf, w, x, y, [255, 255, 255, 255]);
                }
            }
            (w, h, buf)
        };
        let spec = BrushSpec {
            radius_px: 6.0,
            hardness: 1.0,
            ..Default::default()
        };
        let (w, h, mut lo) = make();
        let _ = smear_dab(&mut lo, w, h, [8.0, 8.0], [11.0, 8.0], &spec, 0.25);
        let (_, _, mut hi) = make();
        let _ = smear_dab(&mut hi, w, h, [8.0, 8.0], [11.0, 8.0], &spec, 1.0);
        assert!(
            px(&hi, w, 9, 8)[3] > px(&lo, w, 9, 8)[3],
            "higher strength drags more alpha"
        );
    }

    #[test]
    fn fully_offscreen_is_none() {
        let (w, h) = (16, 16);
        let mut buf = canvas(w, h);
        let spec = BrushSpec {
            radius_px: 3.0,
            ..Default::default()
        };
        assert!(smear_dab(&mut buf, w, h, [-50.0, -50.0], [-47.0, -50.0], &spec, 1.0).is_none());
    }
}
