//! Tests for the cached brush stamp: the mask shape, the scale-blit matching the per-pixel stamp,
//! the textured blit, and the cacheability predicate.

use super::*;
use crate::blend::BrushBlend;
use crate::dab::stamp_dab;
use crate::falloff::Falloff;
use crate::footprint::FootprintDeform;
use crate::spec::BrushSpec;
use crate::texture::{TextureKind, TextureMapping, TextureSettings};

fn solid(w: u32, h: u32, rgba: [u8; 4]) -> Vec<u8> {
    rgba.iter()
        .copied()
        .cycle()
        .take((w * h * 4) as usize)
        .collect()
}

#[test]
fn shape_image_cached_mask_matches_per_pixel_silhouette() {
    use crate::ImageMask;
    use crate::dab::{ShapeInput, stamp_dab_textured};
    use crate::texture::TexDabBasis;
    // A Shape-only brush (white 8×8 square image, no Grain): the baked StampMask must reproduce the
    // per-pixel silhouette closely (only the mask's u8 + bilinear resolution differ at the rim).
    let (w, h) = (96u32, 96u32);
    let white = vec![255u8; 64];
    let img = ImageMask {
        lum: &white,
        width: 8,
        height: 8,
    };
    let spec = BrushSpec {
        radius_px: 36.0,
        color: [0.0, 0.0, 0.0],
        blend: BrushBlend::Mix,
        falloff: Falloff::Smooth, // would be a round disc WITHOUT the Shape
        shape: TextureSettings {
            kind: TextureKind::Image,
            ..Default::default()
        },
        ..Default::default()
    };
    let center = [48.0, 48.0];
    let basis = TexDabBasis::identity();
    let shape_in = ShapeInput {
        basis: &basis,
        image: Some(&img),
        ramp_lut: None,
    };
    // Per-pixel reference.
    let mut a = solid(w, h, [255, 255, 255, 255]);
    let _ = stamp_dab_textured(
        &mut a,
        w,
        h,
        center,
        &spec,
        1.0,
        false,
        None,
        None,
        Some(shape_in),
    );
    // Cached: the Shape is baked into the mask; blit it.
    let mask = render_stamp_mask(&spec, None, Some(&img), None, 96);
    let mut b = solid(w, h, [255, 255, 255, 255]);
    let _ = blit_stamp(&mut b, w, h, center, 36.0, &mask, &spec, 1.0, false);
    let idx = |x: u32, y: u32| ((y * w + x) * 4) as usize;
    // Both paint a SQUARE: centre + a footprint corner are black; the count of painted pixels matches.
    assert_eq!(a[idx(48, 48)], 0, "per-pixel centre black");
    assert_eq!(b[idx(48, 48)], 0, "cached centre black");
    assert!(
        a[idx(20, 20)] < 80 && b[idx(20, 20)] < 80,
        "both paint the square corner"
    );
    let count = |buf: &[u8]| (0..(w * h) as usize).filter(|&i| buf[i * 4] < 128).count();
    let (ca, cb) = (count(&a), count(&b));
    assert!(
        (ca as i32 - cb as i32).abs() * 20 <= ca as i32,
        "cached square silhouette ≈ per-pixel within ~5%: per-pixel={ca} cached={cb}"
    );
}

#[test]
fn shape_with_tiled_grain_canvas_cached_matches_per_pixel() {
    use crate::ImageMask;
    use crate::dab::{ShapeInput, stamp_dab_textured};
    use crate::texture::dab_basis;
    // Shape image (square) + a canvas-fixed Tiled Grain: the canvas-cache path computes the silhouette
    // per-pixel and reads the Grain from the cache — it must match the per-pixel path bit-closely.
    let (w, h) = (96u32, 96u32);
    let white = vec![255u8; 64];
    let shimg = ImageMask {
        lum: &white,
        width: 8,
        height: 8,
    };
    let spec = BrushSpec {
        radius_px: 36.0,
        color: [0.0, 0.0, 0.0],
        blend: BrushBlend::Mix,
        falloff: Falloff::Smooth,
        shape: TextureSettings {
            kind: TextureKind::Image,
            ..Default::default()
        },
        texture: TextureSettings {
            kind: TextureKind::Voronoi,
            mapping: TextureMapping::Tiled,
            ..Default::default()
        },
        ..Default::default()
    };
    let center = [48.0, 48.0];
    // Per-pixel reference: resolve both frames as the cached paths do.
    let mut rng = 0u64;
    let gbasis = dab_basis(
        &spec.texture,
        &mut rng,
        [w as f32, h as f32],
        FootprintDeform::identity(),
    );
    let sbasis = crate::texture::shape_basis(
        &spec.shape,
        &mut 0u64,
        [1.0, 1.0],
        FootprintDeform::identity(),
        crate::texture::ShapeFrame::Static,
    );
    let shape_in = ShapeInput {
        basis: &sbasis,
        image: Some(&shimg),
        ramp_lut: None,
    };
    let mut a = solid(w, h, [255, 255, 255, 255]);
    let _ = stamp_dab_textured(
        &mut a,
        w,
        h,
        center,
        &spec,
        1.0,
        false,
        Some(&gbasis),
        None,
        Some(shape_in),
    );
    // Canvas-cached (fresh cache): the Grain is cached per canvas pixel, the silhouette is per-pixel.
    let mut b = solid(w, h, [255, 255, 255, 255]);
    let mut tex = vec![0u8; (w * h) as usize];
    let mut ready = vec![0u8; (w * h) as usize];
    let _ = blit_canvas_cached(
        &mut b,
        &mut tex,
        &mut ready,
        w,
        h,
        center,
        36.0,
        &spec,
        None,
        Some(&shimg),
        None,
        1.0,
        false,
    );
    let maxdiff = (0..(w * h * 4) as usize)
        .map(|i| i32::from(a[i]).abs_diff(i32::from(b[i])))
        .max()
        .unwrap();
    assert!(
        maxdiff <= 3,
        "canvas-cached Grain × per-pixel Shape ≈ per-pixel, maxdiff={maxdiff}"
    );
}

#[test]
fn mask_is_falloff_shaped() {
    let spec = BrushSpec {
        falloff: Falloff::Smooth,
        hardness: 0.0,
        ..Default::default()
    };
    let mask = render_stamp_mask(&spec, None, None, None, 64);
    assert_eq!(mask.size(), 64);
    let at = |i: u32, j: u32| mask.data[(j * 64 + i) as usize];
    assert!(at(32, 32) > 200, "centre near full, got {}", at(32, 32));
    assert_eq!(at(0, 0), 0, "the corner is outside the disk → 0");
}

#[test]
fn blit_matches_the_per_pixel_stamp_closely() {
    // A hard disk: the cached scale-blit must match the direct per-pixel stamp (centre + far corner
    // exactly; painted area within a few percent — the rim differs by the mask's bilinear resolution).
    let (w, h) = (128u32, 128u32);
    let spec = BrushSpec {
        radius_px: 40.0,
        color: [0.0, 0.0, 0.0],
        blend: BrushBlend::Mix,
        falloff: Falloff::Constant,
        hardness: 1.0,
        ..Default::default()
    };
    let mut a = solid(w, h, [255, 255, 255, 255]);
    let mut b = solid(w, h, [255, 255, 255, 255]);
    let _ = stamp_dab(&mut a, w, h, [64.0, 64.0], &spec, 1.0, false);
    let mask = render_stamp_mask(&spec, None, None, None, 96);
    let _ = blit_stamp(&mut b, w, h, [64.0, 64.0], 40.0, &mask, &spec, 1.0, false);
    let idx = |x: u32, y: u32| ((y * w + x) * 4) as usize;
    assert_eq!(a[idx(64, 64)], 0, "stamp centre black");
    assert_eq!(b[idx(64, 64)], 0, "blit centre black");
    assert_eq!(a[idx(2, 2)], 255, "stamp far corner white");
    assert_eq!(b[idx(2, 2)], 255, "blit far corner white");
    let count = |buf: &[u8]| (0..(w * h) as usize).filter(|&i| buf[i * 4] < 128).count();
    let (ca, cb) = (count(&a), count(&b));
    assert!(
        (ca as i32 - cb as i32).abs() * 20 <= ca as i32,
        "painted area matches within ~5%: stamp={ca} blit={cb}"
    );
}

#[test]
fn textured_mask_blit_shows_the_pattern() {
    // A Checker mask blitted over a hard dab leaves a mix of painted + untouched pixels.
    let (w, h) = (96u32, 96u32);
    let spec = BrushSpec {
        radius_px: 36.0,
        color: [0.0, 0.0, 0.0],
        blend: BrushBlend::Mix,
        falloff: Falloff::Constant,
        hardness: 1.0,
        texture: TextureSettings {
            kind: TextureKind::Checker,
            mapping: TextureMapping::ViewPlane,
            size: [0.25, 0.25],
            ..Default::default()
        },
        ..Default::default()
    };
    let mask = render_stamp_mask(&spec, None, None, None, 96);
    let mut buf = solid(w, h, [255, 255, 255, 255]);
    blit_stamp(&mut buf, w, h, [48.0, 48.0], 36.0, &mask, &spec, 1.0, false).expect("painted");
    let (mut black, mut white) = (0, 0);
    for y in 16..80 {
        for x in 16..80 {
            match buf[((y * w + x) * 4) as usize] {
                0 => black += 1,
                255 => white += 1,
                _ => {}
            }
        }
    }
    assert!(
        black > 0 && white > 0,
        "checker mask reads: black={black} white={white}"
    );
}

#[test]
fn canvas_cached_blit_matches_the_per_pixel_tiled_stamp() {
    use crate::dab::stamp_dab_textured;
    use crate::texture::dab_basis;
    let (w, h) = (96u32, 96u32);
    let spec = BrushSpec {
        radius_px: 36.0,
        color: [0.0, 0.0, 0.0],
        blend: BrushBlend::Mix,
        falloff: Falloff::Smooth,
        texture: TextureSettings {
            kind: TextureKind::Voronoi,
            mapping: TextureMapping::Tiled,
            ..Default::default()
        },
        ..Default::default()
    };
    // Per-pixel reference.
    let mut rng = 0u64;
    let basis = dab_basis(
        &spec.texture,
        &mut rng,
        [w as f32, h as f32],
        FootprintDeform::identity(),
    );
    let mut a = solid(w, h, [255, 255, 255, 255]);
    let _ = stamp_dab_textured(
        &mut a,
        w,
        h,
        [48.0, 48.0],
        &spec,
        1.0,
        false,
        Some(&basis),
        None,
        None,
    );
    // Canvas-cached blit (fresh cache).
    let mut b = solid(w, h, [255, 255, 255, 255]);
    let mut tex = vec![0u8; (w * h) as usize];
    let mut ready = vec![0u8; (w * h) as usize];
    let _ = blit_canvas_cached(
        &mut b,
        &mut tex,
        &mut ready,
        w,
        h,
        [48.0, 48.0],
        36.0,
        &spec,
        None,
        None,
        None,
        1.0,
        false,
    );
    // Bit-close: the only difference is the texture's u8 quantisation in the cache.
    let maxdiff = (0..(w * h * 4) as usize)
        .map(|i| i32::from(a[i]).abs_diff(i32::from(b[i])))
        .max()
        .unwrap();
    assert!(
        maxdiff <= 3,
        "canvas-cached Tiled ≈ per-pixel, maxdiff={maxdiff}"
    );
    assert!(
        ready.contains(&1),
        "the cache was filled where the dab painted"
    );
    // Reuse: a second blit at the same spot reads the cache (no recompute) and is identical.
    let mut c = solid(w, h, [255, 255, 255, 255]);
    let _ = blit_canvas_cached(
        &mut c,
        &mut tex,
        &mut ready,
        w,
        h,
        [48.0, 48.0],
        36.0,
        &spec,
        None,
        None,
        None,
        1.0,
        false,
    );
    assert_eq!(
        b, c,
        "a cache-hit blit reproduces the cache-fill blit exactly"
    );
}

#[test]
fn cacheability_predicate() {
    let none = TextureSettings::default();
    assert!(none.is_cacheable(), "no texture is cacheable");
    let view = TextureSettings {
        kind: TextureKind::Noise,
        mapping: TextureMapping::ViewPlane,
        ..Default::default()
    };
    assert!(view.is_cacheable(), "static View texture is cacheable");
    for not in [
        TextureSettings {
            mapping: TextureMapping::Tiled,
            ..view
        },
        TextureSettings {
            mapping: TextureMapping::Stencil,
            ..view
        },
        TextureSettings { rake: true, ..view },
    ] {
        assert!(
            !not.is_cacheable(),
            "canvas-relative / per-dab texture is NOT cacheable: {not:?}"
        );
    }
}

#[test]
fn dab_flatten_squishes_the_cached_mask_into_a_rotated_ellipse() {
    // A hard round disk (Constant falloff). Vertical flatten pushes points ABOVE the centre outside
    // the ellipse while points to the SIDE stay in; rotating 90° swaps the two axes (Enio 2026-06-26).
    let round = BrushSpec {
        falloff: Falloff::Constant,
        hardness: 1.0,
        ..Default::default()
    };
    let flat = BrushSpec {
        dab_flatten: 0.7,
        ..round
    };
    let flat_rot = BrushSpec {
        dab_angle_deg: 90,
        ..flat
    };
    let size = 64u32;
    let at = |m: &StampMask, i: u32, j: u32| m.data[(j * size + i) as usize];
    let m_round = render_stamp_mask(&round, None, None, None, size);
    let m_flat = render_stamp_mask(&flat, None, None, None, size);
    let m_rot = render_stamp_mask(&flat_rot, None, None, None, size);
    let c = size / 2; // centre column / row
    let up = size / 8; // a row well above centre (on the vertical axis)
    let side = size / 8; // a column well left of centre (on the horizontal axis)
    // Round: both the vertical and the horizontal sample are inside the disk.
    assert!(
        at(&m_round, c, up) > 0 && at(&m_round, side, c) > 0,
        "round disk paints both axes"
    );
    // Vertical flatten: the ABOVE sample is squished outside → 0; the SIDE sample stays in.
    assert_eq!(at(&m_flat, c, up), 0, "vertical flatten clears the top");
    assert!(at(&m_flat, side, c) > 0, "vertical flatten keeps the side");
    // Rotated 90°: the minor axis is now horizontal — the two are swapped.
    assert!(at(&m_rot, c, up) > 0, "rotated flatten keeps the top");
    assert_eq!(at(&m_rot, side, c), 0, "rotated flatten clears the side");
}

#[test]
fn dab_default_flatten_is_byte_identical_round() {
    // Flatten 0 / angle 0 must leave the dab byte-identical to the pre-feature round mask (HR-5).
    let spec = BrushSpec::default();
    let a = render_stamp_mask(&spec, None, None, None, 48);
    let b = render_stamp_mask(
        &BrushSpec {
            dab_flatten: 0.0,
            dab_angle_deg: 0,
            ..spec
        },
        None,
        None,
        None,
        48,
    );
    assert_eq!(
        a.data, b.data,
        "default flatten/angle is the identity round dab"
    );
}
