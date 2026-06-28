//! Tests for the dab stamp: coverage, falloff, strength, alpha-lock, the textured path, and
//! the Color Ramp colour path. Moved to a sibling file to keep `dab.rs` under the LOC cap.

use super::*;
use crate::blend::BrushBlend;
use crate::falloff::Falloff;
use crate::footprint::FootprintDeform;

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
        None,
        &lut,
        crate::ramp_alpha::RampAlphaMode::None,
        None,
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

/// Shared fixture for the ramp-alpha-mode tests: a hard checker dab whose ramp is opaque red at the
/// 0-cells and `a0`-alpha blue at the 1-cells.
#[cfg(test)]
fn ramp_alpha_dab(buf: &mut [u8], w: u32, h: u32, a1: f32, mode: crate::ramp_alpha::RampAlphaMode) {
    use crate::texture::{TexDabBasis, TextureKind, TextureMapping, TextureSettings};
    let spec = BrushSpec {
        radius_px: 18.0,
        color: [0.0, 1.0, 0.0],
        blend: BrushBlend::Mix,
        falloff: Falloff::Constant, // hard disk → coverage ≈ 1 across the footprint
        hardness: 1.0,
        texture: TextureSettings {
            kind: TextureKind::Checker,
            mapping: TextureMapping::ViewPlane,
            size: [0.25, 0.25], // big cells → both parities land in the footprint
            ..Default::default()
        },
        ..Default::default()
    };
    let lut = [[1.0, 0.0, 0.0, 1.0], [0.0, 0.0, 1.0, a1]];
    let basis = TexDabBasis::identity();
    stamp_dab_ramped(
        buf,
        w,
        h,
        [24.0, 24.0],
        &spec,
        1.0,
        false,
        Some(&basis),
        None,
        None,
        &lut,
        mode,
        None,
    )
    .expect("ramped dab painted");
}

#[test]
fn ramp_alpha_texture_mode_punches_the_sprite_transparent() {
    use crate::ramp_alpha::RampAlphaMode;
    let (w, h) = (48u32, 48u32);
    // s=1 stop is fully transparent → mode TextureAlpha makes those cells see-through.
    let mut buf = solid(w, h, [255, 255, 255, 255]);
    ramp_alpha_dab(&mut buf, w, h, 0.0, RampAlphaMode::TextureAlpha);
    let (mut transparent, mut opaque) = (0, 0);
    for i in 0..(w * h) as usize {
        match buf[i * 4 + 3] {
            a if a < 30 => transparent += 1,
            a if a > 220 => opaque += 1,
            _ => {}
        }
    }
    assert!(
        transparent > 0,
        "transparent ramp cells punch the sprite see-through: {transparent}"
    );
    assert!(opaque > 0, "opaque ramp cells stay solid: {opaque}");
}

#[test]
fn ramp_alpha_none_vs_strength_on_a_transparent_canvas() {
    use crate::ramp_alpha::RampAlphaMode;
    let (w, h) = (48u32, 48u32);
    let opaque_px = |buf: &[u8]| {
        (0..(w * h) as usize)
            .filter(|&i| buf[i * 4 + 3] > 220)
            .count()
    };
    // Onto a transparent canvas, the s=1 (alpha-0) cells: None ignores alpha → still deposits full
    // coverage; Strength → zero coverage there → those cells stay transparent. So None covers more.
    let mut none = solid(w, h, [0, 0, 0, 0]);
    ramp_alpha_dab(&mut none, w, h, 0.0, RampAlphaMode::None);
    let mut strength = solid(w, h, [0, 0, 0, 0]);
    ramp_alpha_dab(&mut strength, w, h, 0.0, RampAlphaMode::Strength);
    assert!(
        opaque_px(&none) > opaque_px(&strength),
        "Strength drops coverage at the translucent cells: none={} strength={}",
        opaque_px(&none),
        opaque_px(&strength)
    );
    assert!(
        opaque_px(&strength) > 0,
        "the opaque (s=0) cells still paint under Strength"
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

#[test]
fn accumulate_off_caps_a_stroke_at_strength_while_on_builds_up() {
    // Accumulate OFF (a per-stroke coverage mask) caps overlapping dabs at Strength; ON (no mask)
    // lets them build past it. Constant falloff + hard disk ⇒ the centre weight is exactly 1.
    let (w, h) = (16u32, 16u32);
    let spec = BrushSpec {
        radius_px: 5.0,
        color: [0.0, 0.0, 0.0], // black over white
        blend: BrushBlend::Mix,
        falloff: Falloff::Constant,
        hardness: 1.0,
        strength: 0.5,
        ..Default::default()
    };
    let center = [8.0, 8.0];
    let i = (8 * w as usize + 8) * 4; // centre pixel
    // OFF: five overlapping dabs in ONE stroke leave the centre at 50 % coverage (white→grey ~128).
    let mut off = solid(w, h, [255, 255, 255, 255]);
    let mut mask = vec![0u8; (w * h) as usize];
    for _ in 0..5 {
        // After the first dab caps the (constant-falloff) disk, later dabs paint nothing → `None`.
        let _ = stamp_dab_textured_masked(
            &mut off,
            w,
            h,
            center,
            &spec,
            1.0,
            false,
            None,
            None,
            None,
            Some(&mut mask),
        );
    }
    assert!(
        (i32::from(off[i]) - 128).abs() <= 2,
        "Accumulate OFF caps at Strength: {} (want ~128)",
        off[i]
    );
    // ON (no mask): the same five dabs build past Strength toward black.
    let mut on = solid(w, h, [255, 255, 255, 255]);
    for _ in 0..5 {
        stamp_dab_textured_masked(
            &mut on, w, h, center, &spec, 1.0, false, None, None, None, None,
        )
        .expect("dab painted");
    }
    assert!(
        on[i] < 40 && off[i] > on[i],
        "Accumulate ON builds past Strength: on={} off={} (want on dark, off lighter)",
        on[i],
        off[i]
    );
}

#[test]
fn accumulate_off_grain_caps_each_texel_at_its_weighted_coverage() {
    use crate::ImageMask;
    use crate::texture::{TexDabBasis, TextureKind, TextureSettings};
    // A flat 50 %-grey Grain image ⇒ grain ≈ 0.5 everywhere. With Accumulate OFF, a texel's coverage must
    // top out at grain × Strength (~0.5) no matter how many overlapping dabs cross it — re-passing must NOT
    // climb the semi-transparent texture to full opacity (the "textured areas fill in" bug, Enio 2026-06-27).
    let (w, h) = (16u32, 16u32);
    let grey = vec![128u8; 64];
    let img = ImageMask {
        lum: &grey,
        width: 8,
        height: 8,
    };
    let spec = BrushSpec {
        radius_px: 5.0,
        color: [0.0, 0.0, 0.0], // black over white
        blend: BrushBlend::Mix,
        falloff: Falloff::Constant,
        hardness: 1.0,
        strength: 1.0,
        texture: TextureSettings {
            kind: TextureKind::Image,
            ..Default::default()
        },
        ..Default::default()
    };
    let basis = TexDabBasis::identity();
    let center = [8.0, 8.0];
    let i = (8 * w as usize + 8) * 4;
    let mut buf = solid(w, h, [255, 255, 255, 255]);
    let mut mask = vec![0u8; (w * h) as usize];
    for _ in 0..10 {
        let _ = stamp_dab_textured_masked(
            &mut buf,
            w,
            h,
            center,
            &spec,
            1.0,
            false,
            Some(&basis),
            Some(&img),
            None,
            Some(&mut mask),
        );
    }
    // grain ≈ 0.502 → final coverage ≈ 0.502 → white→~127, NOT driven toward black by the re-passes.
    assert!(
        (i32::from(buf[i]) - 127).abs() <= 8,
        "grain caps the texel at ~0.5 coverage even after 10 passes: {} (want ~127, not black)",
        buf[i]
    );
}

// ── W0: dual-slot (Shape × Grain) composition ───────────────────────────────────────────────

#[test]
fn grain_depth_one_is_default_and_zero_disables_the_grain() {
    use crate::texture::{TexDabBasis, TextureKind, TextureMapping, TextureSettings};
    let (w, h) = (48u32, 48u32);
    let base = BrushSpec {
        radius_px: 18.0,
        color: [0.0, 0.0, 0.0],
        blend: BrushBlend::Mix,
        falloff: Falloff::Smooth,
        texture: TextureSettings {
            kind: TextureKind::Checker,
            mapping: TextureMapping::ViewPlane,
            size: [0.3, 0.3],
            ..Default::default()
        },
        ..Default::default()
    };
    let basis = TexDabBasis::identity();
    let center = [24.0, 24.0];

    // Default grain (depth = 1.0) applies the checker → differs from a bare (no-grain) dab.
    let mut g1 = solid(w, h, [255, 255, 255, 255]);
    stamp_dab_textured(
        &mut g1,
        w,
        h,
        center,
        &base,
        1.0,
        false,
        Some(&basis),
        None,
        None,
    )
    .expect("grain dab painted");

    // depth = 0.0 disables the grain: byte-for-byte identical to the same dab with NO grain.
    let mut d0 = solid(w, h, [255, 255, 255, 255]);
    let spec0 = BrushSpec {
        grain_depth: 0.0,
        ..base
    };
    stamp_dab_textured(
        &mut d0,
        w,
        h,
        center,
        &spec0,
        1.0,
        false,
        Some(&basis),
        None,
        None,
    )
    .expect("depth-0 dab painted");

    let mut notex = solid(w, h, [255, 255, 255, 255]);
    let spec_nt = BrushSpec {
        texture: TextureSettings::default(),
        ..base
    };
    stamp_dab(&mut notex, w, h, center, &spec_nt, 1.0, false).expect("plain dab painted");

    assert_eq!(
        d0, notex,
        "grain_depth 0 ⇒ grain has no effect (silhouette only)"
    );
    assert_ne!(
        g1, notex,
        "grain_depth 1 (default) ⇒ the grain actually modulates"
    );
}

#[test]
fn shape_image_replaces_the_falloff_with_a_crisp_silhouette() {
    use crate::ImageMask;
    use crate::texture::{TexDabBasis, TextureKind, TextureSettings};
    let (w, h) = (48u32, 48u32);
    // An all-white 8×8 Shape image ⇒ the silhouette is the full footprint SQUARE (crisp to the edge),
    // not a round falloff disc. A footprint corner that a Smooth disc leaves blank is now painted.
    let white = vec![255u8; 64];
    let img = ImageMask {
        lum: &white,
        width: 8,
        height: 8,
    };
    let spec = BrushSpec {
        radius_px: 18.0,
        color: [0.0, 0.0, 0.0],
        blend: BrushBlend::Mix,
        falloff: Falloff::Smooth,
        shape: TextureSettings {
            kind: TextureKind::Image,
            ..Default::default()
        },
        ..Default::default()
    };
    let basis = TexDabBasis::identity();
    let shape_in = ShapeInput {
        basis: &basis,
        image: Some(&img),
        ramp_lut: None,
    };
    let center = [24.0, 24.0];
    let idx = |x: u32, y: u32| ((y * w + x) * 4) as usize;

    let mut sq = solid(w, h, [255, 255, 255, 255]);
    stamp_dab_textured(
        &mut sq,
        w,
        h,
        center,
        &spec,
        1.0,
        false,
        None,
        None,
        Some(shape_in),
    )
    .expect("shape dab painted");
    // Pixel (8,8): footprint coord ≈ (−0.86, −0.86) — inside the square (|u|,|v|<1) ⇒ painted black;
    // a Smooth falloff there is at t ≈ 1.2 > 1 ⇒ zero.
    assert!(
        sq[idx(8, 8)] < 80,
        "square shape paints the footprint corner (got {})",
        sq[idx(8, 8)]
    );

    // A falloff-only brush (Shape inactive) leaves that same corner unpainted (round disc).
    let mut disc = solid(w, h, [255, 255, 255, 255]);
    let spec_disc = BrushSpec {
        shape: TextureSettings::default(),
        ..spec
    };
    stamp_dab(&mut disc, w, h, center, &spec_disc, 1.0, false).expect("disc dab painted");
    assert!(
        disc[idx(8, 8)] > 240,
        "round falloff leaves the corner blank (got {})",
        disc[idx(8, 8)]
    );

    // An Image shape with NO pixels is INACTIVE (the tool then uses the falloff — no blank brush).
    assert!(
        !spec.shape_silhouette_active(false),
        "an Image shape with no pixels is inactive ⇒ falloff silhouette"
    );
    assert!(
        spec.shape_silhouette_active(true),
        "an Image shape with pixels is the active silhouette"
    );
}

#[test]
fn shape_angle_rotates_the_silhouette() {
    use crate::ImageMask;
    use crate::texture::{TextureKind, TextureSettings, dab_basis};
    let (w, h) = (48u32, 48u32);
    // A vertical-bar Shape: the central columns (x = 3,4 of 8) are white, the rest black.
    let mut lum = vec![0u8; 64];
    for y in 0..8 {
        lum[y * 8 + 3] = 255;
        lum[y * 8 + 4] = 255;
    }
    let img = ImageMask {
        lum: &lum,
        width: 8,
        height: 8,
    };
    let mk = |angle: u16| BrushSpec {
        radius_px: 18.0,
        color: [0.0, 0.0, 0.0],
        blend: BrushBlend::Mix,
        falloff: Falloff::Smooth,
        shape: TextureSettings {
            kind: TextureKind::Image,
            angle_deg: angle,
            ..Default::default()
        },
        ..Default::default()
    };
    let center = [24.0, 24.0];
    let idx = |x: u32, y: u32| ((y * w + x) * 4) as usize;

    // Angle 0: the bar runs vertically — painted ABOVE the centre, blank to the RIGHT.
    let s0 = mk(0);
    let b0 = dab_basis(
        &s0.shape,
        [0.0, 0.0],
        &mut 0u64,
        [1.0, 1.0],
        [1.0, 0.0],
        FootprintDeform::identity(),
    );
    let mut a = solid(w, h, [255, 255, 255, 255]);
    stamp_dab_textured(
        &mut a,
        w,
        h,
        center,
        &s0,
        1.0,
        false,
        None,
        None,
        Some(ShapeInput {
            basis: &b0,
            image: Some(&img),
            ramp_lut: None,
        }),
    )
    .expect("angle-0 shape painted");
    assert!(
        a[idx(24, 12)] < 80,
        "vertical bar paints above centre (got {})",
        a[idx(24, 12)]
    );
    assert!(
        a[idx(36, 24)] > 200,
        "vertical bar is blank to the right (got {})",
        a[idx(36, 24)]
    );

    // Angle 90: the SAME shape now runs horizontally — the orientation flips.
    let s90 = mk(90);
    let b90 = dab_basis(
        &s90.shape,
        [0.0, 0.0],
        &mut 0u64,
        [1.0, 1.0],
        [1.0, 0.0],
        FootprintDeform::identity(),
    );
    let mut c = solid(w, h, [255, 255, 255, 255]);
    stamp_dab_textured(
        &mut c,
        w,
        h,
        center,
        &s90,
        1.0,
        false,
        None,
        None,
        Some(ShapeInput {
            basis: &b90,
            image: Some(&img),
            ramp_lut: None,
        }),
    )
    .expect("angle-90 shape painted");
    assert!(
        c[idx(24, 12)] > 200,
        "rotated bar no longer paints above centre (got {})",
        c[idx(24, 12)]
    );
    assert!(
        c[idx(36, 24)] < 80,
        "rotated bar now paints to the right (got {})",
        c[idx(36, 24)]
    );
}
