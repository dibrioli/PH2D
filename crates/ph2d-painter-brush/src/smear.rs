//! Smear one dab — drag the canvas content along the stroke.
//!
//! **Clean-room** behaviour of Blender's 2D image-paint **Smear** brush: it lifts the footprint
//! region from the *previous* dab position, then blends it at the *current* position by linear
//! interpolation, masked by the brush falloff × strength. This is the community-accepted
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
///
/// `wrap` (Tiling) makes the LIFT toroidal per axis: a source pixel past a wrapped edge reads from the
/// opposite edge (the image is one seamless tile). Without it the source is CLAMPED to the canvas —
/// a pixel past the edge reads the edge pixel — so an off-canvas lift never drags transparency in.
/// The wrapped WRITE is the caller's job (it stamps the dab at the wrapped positions).
#[allow(clippy::too_many_arguments)]
#[must_use]
pub fn smear_dab(
    buf: &mut [u8],
    width: u32,
    height: u32,
    from: [f32; 2],
    to: [f32; 2],
    spec: &BrushSpec,
    strength: f32,
    wrap: [bool; 2],
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

    // Lift: snapshot the source pixel (canvas at `dest - step`) for every dest cell. A `wrap` axis
    // reads toroidally (`rem_euclid` → opposite edge) so a tiled seam is seamless; a non-wrap axis
    // whose source is off-canvas reads the EDGE pixel (`clamp` — "extend, never a hole", the law the
    // warp-field Smear already follows in `bilinear_clamped`).
    //
    // ⚠️ It used to stay transparent-zero, on the premise *«that is only the dab rim, falloff ~0 —
    // nil effect»*. True mid-canvas, **false whenever the dab CENTRE sits on the edge**, where the
    // weight is full: dragging from the edge inward painted transparency into an opaque canvas
    // (report 2026-09-24 — measured through the watercolor Smudge at `r = 24`: `4 845` texels with
    // alpha `< 255`, min `178`).
    let mut lifted = vec![[0u8; 4]; bw * bh];
    for j in 0..bh {
        let sy = min_y + j as i64 - step_y;
        let sy = if wrap[1] {
            sy.rem_euclid(fh)
        } else {
            sy.clamp(0, fh - 1)
        };
        for i in 0..bw {
            let sx = min_x + i as i64 - step_x;
            let sx = if wrap[0] {
                sx.rem_euclid(fw)
            } else {
                sx.clamp(0, fw - 1)
            };
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
        assert!(
            smear_dab(
                &mut buf,
                w,
                h,
                [8.0, 8.0],
                [8.0, 8.0],
                &spec,
                1.0,
                [false, false]
            )
            .is_none()
        );
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
        let dirty = smear_dab(
            &mut buf,
            w,
            h,
            [8.0, 8.0],
            [11.0, 8.0],
            &spec,
            1.0,
            [false, false],
        )
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
        let _ = smear_dab(
            &mut lo,
            w,
            h,
            [8.0, 8.0],
            [11.0, 8.0],
            &spec,
            0.25,
            [false, false],
        );
        let (_, _, mut hi) = make();
        let _ = smear_dab(
            &mut hi,
            w,
            h,
            [8.0, 8.0],
            [11.0, 8.0],
            &spec,
            1.0,
            [false, false],
        );
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
        assert!(
            smear_dab(
                &mut buf,
                w,
                h,
                [-50.0, -50.0],
                [-47.0, -50.0],
                &spec,
                1.0,
                [false, false]
            )
            .is_none()
        );
    }

    #[test]
    fn an_off_canvas_lift_reads_the_edge_and_a_wrapped_one_reads_the_opposite_edge() {
        // Report 2026-09-24: a dab CENTRED on the edge, dragging inward, lifts from past the edge. The
        // lift used to stay transparent-zero there («only the rim, nil effect» — false at the
        // centre, where the weight is full), and painted alpha holes into an opaque canvas.
        //
        // Now the two axes answer different questions, and the fixture tells them apart by COLOUR
        // (alpha alone cannot: both keep the canvas opaque): the left half is A, the right half B.
        // * wrap  ⇒ the source past the LEFT edge is the RIGHT edge ⇒ the seam pixel becomes B;
        // * clamp ⇒ the source past the left edge is the left EDGE pixel ⇒ the seam stays A, opaque.
        const A: [u8; 4] = [200, 20, 20, 255];
        const B: [u8; 4] = [20, 20, 200, 255];
        let (w, h) = (16u32, 8u32);
        let mut buf = canvas(w, h);
        for y in 0..h {
            for x in 0..w {
                set(&mut buf, w, x, y, if x < 8 { A } else { B });
            }
        }
        let spec = BrushSpec {
            radius_px: 3.0,
            hardness: 1.0,
            ..Default::default()
        };
        let mut wrapped = buf.clone();
        let _ = smear_dab(
            &mut wrapped,
            w,
            h,
            [-1.0, 4.0],
            [1.0, 4.0],
            &spec,
            1.0,
            [true, false],
        );
        assert_eq!(
            px(&wrapped, w, 0, 4),
            B,
            "wrapped smear lifts from the OPPOSITE edge"
        );
        let _ = smear_dab(
            &mut buf,
            w,
            h,
            [-1.0, 4.0],
            [1.0, 4.0],
            &spec,
            1.0,
            [false, false],
        );
        assert_eq!(
            px(&buf, w, 0, 4),
            A,
            "an un-wrapped lift past the edge reads the EDGE pixel, not the far side"
        );
        assert!(
            buf.as_chunks::<4>().0.iter().all(|p| p[3] == 255),
            "an off-canvas lift never drags transparency into an opaque canvas"
        );
        // The same law on the OTHER axis: a dab centred on the TOP edge dragging down. (A mutation
        // that put the old skip back on `y` alone survived the horizontal half above.)
        let _ = smear_dab(
            &mut buf,
            w,
            h,
            [4.0, -1.0],
            [4.0, 1.0],
            &spec,
            1.0,
            [false, false],
        );
        assert!(
            buf.as_chunks::<4>().0.iter().all(|p| p[3] == 255),
            "a lift past the TOP edge reads the edge row, never transparent"
        );
    }
}
