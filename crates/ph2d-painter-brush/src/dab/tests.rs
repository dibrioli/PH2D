//! Tests for the dab stamp: coverage, falloff, strength, alpha-lock, the textured path, and
//! the Color Ramp colour path. Moved to a sibling file to keep `dab.rs` under the LOC cap.

use super::*;
use crate::blend::BrushBlend;
use crate::falloff::Falloff;

fn solid(width: u32, height: u32, rgba: [u8; 4]) -> Vec<u8> {
    rgba.iter()
        .copied()
        .cycle()
        .take((width * height * 4) as usize)
        .collect()
}

#[test]
fn ramped_stamp_paints_the_ramp_colours_not_the_brush_colour() {
    use crate::texture::{TexDabBasis, TextureKind, TextureMapping, TextureSettings};
    let (w, h) = (48u32, 48u32);
    let spec = BrushSpec {
        radius_px: 18.0,
        color: [0.0, 1.0, 0.0], // brush GREEN — must NOT appear (the ramp drives colour)
        blend: BrushBlend::Mix,
        falloff: Falloff::Constant,
        hardness: 1.0,
        texture: TextureSettings {
            kind: TextureKind::Checker,
            mapping: TextureMapping::ViewPlane,
            size: [0.25, 0.25], // big cells → both checker parities land in the footprint
            ..Default::default()
        },
        ..Default::default()
    };
    // LUT: red at the checker 0-cells (texture value 0), blue at the 1-cells (value 1).
    let lut = [[1.0, 0.0, 0.0, 1.0], [0.0, 0.0, 1.0, 1.0]];
    let mut buf = solid(w, h, [255, 255, 255, 255]);
    let basis = TexDabBasis::identity();
    stamp_dab_ramped(
        &mut buf,
        w,
        h,
        [24.0, 24.0],
        &spec,
        1.0,
        false,
        Some(&basis),
        None,
        &lut,
    )
    .expect("ramped dab painted");
    let (mut red, mut blue, mut green) = (0, 0, 0);
    for i in 0..(w * h) as usize {
        let (r, g, b) = (buf[i * 4], buf[i * 4 + 1], buf[i * 4 + 2]);
        if r > 200 && g < 60 && b < 60 {
            red += 1;
        } else if b > 200 && r < 60 && g < 60 {
            blue += 1;
        } else if g > 200 && r < 60 && b < 60 {
            green += 1;
        }
    }
    assert!(
        red > 0 && blue > 0,
        "ramp paints both stops' colours: red={red} blue={blue}"
    );
    assert_eq!(
        green, 0,
        "the brush's own colour must NOT appear — the ramp drives colour"
    );
}

#[test]
fn dab_paints_center_and_reports_rect() {
    let (w, h) = (32, 32);
    let mut buf = solid(w, h, [255, 255, 255, 255]); // white, opaque
    let spec = BrushSpec {
        radius_px: 6.0,
        color: [0.0, 0.0, 0.0], // black
        blend: BrushBlend::Mix,
        falloff: Falloff::Constant, // hard so the centre is fully painted
        ..Default::default()
    };
    let rect = stamp_dab(&mut buf, w, h, [16.0, 16.0], &spec, 1.0, false).expect("painted");
    // Centre pixel went black.
    let i = ((16 * w + 16) * 4) as usize;
    assert_eq!(&buf[i..i + 4], &[0, 0, 0, 255]);
    // Rect contains the centre and is within the canvas.
    assert!(rect.x <= 16 && rect.y <= 16);
    assert!(rect.x + rect.w <= w && rect.y + rect.h <= h);
    // A pixel well outside the radius is untouched (still white).
    let far = ((2 * w + 2) * 4) as usize;
    assert_eq!(&buf[far..far + 4], &[255, 255, 255, 255]);
}

#[test]
fn dab_off_canvas_is_none() {
    let (w, h) = (16, 16);
    let mut buf = solid(w, h, [0, 0, 0, 0]);
    let spec = BrushSpec {
        radius_px: 3.0,
        ..Default::default()
    };
    assert!(stamp_dab(&mut buf, w, h, [-100.0, -100.0], &spec, 1.0, false).is_none());
}

#[test]
fn zero_coverage_is_none() {
    let (w, h) = (16, 16);
    let mut buf = solid(w, h, [10, 20, 30, 255]);
    let spec = BrushSpec::default();
    assert!(stamp_dab(&mut buf, w, h, [8.0, 8.0], &spec, 0.0, false).is_none());
    // Buffer untouched.
    assert_eq!(buf[0..4], [10, 20, 30, 255]);
}

#[test]
fn soft_falloff_fades_from_center_to_rim() {
    let (w, h) = (40, 40);
    let mut buf = solid(w, h, [255, 255, 255, 255]);
    let spec = BrushSpec {
        radius_px: 14.0,
        color: [0.0, 0.0, 0.0],
        blend: BrushBlend::Mix,
        falloff: Falloff::Smooth,
        ..Default::default()
    };
    stamp_dab(&mut buf, w, h, [20.0, 20.0], &spec, 1.0, false).expect("painted");
    let val = |x: u32, y: u32| buf[((y * w + x) * 4) as usize];
    // Centre darker than a point near the rim.
    assert!(
        val(20, 20) < val(20, 32),
        "centre should be darker than the rim"
    );
}

#[test]
fn strength_scales_dab_opacity() {
    // Full strength → black; half strength → mid-grey over white (Mix at a=0.5).
    let (w, h) = (16, 16);
    let hard = |strength: f32| BrushSpec {
        radius_px: 5.0,
        color: [0.0, 0.0, 0.0],
        blend: BrushBlend::Mix,
        falloff: Falloff::Constant,
        hardness: 1.0,
        strength,
        ..Default::default()
    };
    let mut full = solid(w, h, [255, 255, 255, 255]);
    stamp_dab(&mut full, w, h, [8.0, 8.0], &hard(1.0), 1.0, false).expect("painted");
    assert_eq!(full[((8 * w + 8) * 4) as usize], 0, "full strength = black");

    let mut half = solid(w, h, [255, 255, 255, 255]);
    stamp_dab(&mut half, w, h, [8.0, 8.0], &hard(0.5), 1.0, false).expect("painted");
    let v = half[((8 * w + 8) * 4) as usize];
    assert!(
        (120..=136).contains(&v),
        "half strength ≈ mid-grey, got {v}"
    );
}

#[test]
fn preserve_alpha_paints_only_into_existing_alpha() {
    let (w, h) = (8, 8);
    // Left half opaque white, right half fully transparent.
    let mut buf = vec![0u8; (w * h * 4) as usize];
    for y in 0..h {
        for x in 0..4 {
            let i = ((y * w + x) * 4) as usize;
            buf[i..i + 4].copy_from_slice(&[255, 255, 255, 255]);
        }
    }
    let spec = BrushSpec {
        radius_px: 6.0,
        color: [0.0, 0.0, 0.0],
        blend: BrushBlend::Mix,
        falloff: Falloff::Constant,
        ..Default::default()
    };
    // Cover the whole buffer with alpha-lock on.
    stamp_dab(&mut buf, w, h, [4.0, 4.0], &spec, 1.0, true).expect("painted");
    let alpha = |x: u32, y: u32| buf[((y * w + x) * 4 + 3) as usize];
    let rgb = |x: u32, y: u32| {
        let i = ((y * w + x) * 4) as usize;
        [buf[i], buf[i + 1], buf[i + 2]]
    };
    // Opaque side recoloured to black, alpha preserved.
    assert_eq!(rgb(1, 4), [0, 0, 0], "opaque side recoloured");
    assert_eq!(alpha(1, 4), 255, "opaque alpha preserved");
    // Transparent side untouched — no alpha created.
    assert_eq!(alpha(6, 4), 0, "alpha-lock did not paint into transparency");
}

#[test]
fn large_dab_takes_the_parallel_path_and_stamps_correctly() {
    // A dab past `PARALLEL_MIN_AREA` splits across row bands; the result must match the serial
    // expectation (centre + interior painted; a far corner outside the radius untouched). Guards
    // a band-offset bug — the small-canvas tests above all take the serial path.
    let (w, h) = (600u32, 600u32);
    let mut buf = solid(w, h, [255, 255, 255, 255]);
    let spec = BrushSpec {
        radius_px: 250.0,
        color: [0.0, 0.0, 0.0],
        blend: BrushBlend::Mix,
        falloff: Falloff::Constant,
        hardness: 1.0, // hard disk
        ..Default::default()
    };
    let rect = stamp_dab(&mut buf, w, h, [300.0, 300.0], &spec, 1.0, false).expect("painted");
    assert!(
        rect.w as usize * rect.h as usize >= PARALLEL_MIN_AREA,
        "this dab must cross the parallel threshold"
    );
    let at = |x: u32, y: u32| buf[((y * w + x) * 4) as usize];
    assert_eq!(at(300, 300), 0, "centre painted black");
    assert_eq!(
        at(300, 80),
        0,
        "an interior point (dist 220 < 250) is painted"
    );
    assert_eq!(
        at(5, 5),
        255,
        "a far corner outside the radius is untouched"
    );
}
