//! Tests for the cached brush stamp: the mask shape, the scale-blit matching the per-pixel stamp,
//! the textured blit, and the cacheability predicate.

use super::*;
use crate::blend::BrushBlend;
use crate::dab::stamp_dab;
use crate::falloff::Falloff;
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
fn mask_is_falloff_shaped() {
    let spec = BrushSpec {
        falloff: Falloff::Smooth,
        hardness: 0.0,
        ..Default::default()
    };
    let mask = render_stamp_mask(&spec, None, 64);
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
    let mask = render_stamp_mask(&spec, None, 96);
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
    let mask = render_stamp_mask(&spec, None, 96);
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
        TextureSettings {
            random_angle: true,
            ..view
        },
    ] {
        assert!(
            !not.is_cacheable(),
            "canvas-relative / per-dab texture is NOT cacheable: {not:?}"
        );
    }
}
