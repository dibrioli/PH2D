//! Tests for the procedural texture samplers: per-kind boundedness + determinism (HR-5), the shape
//! knobs, and the image-backed bilinear sampler. Moved to a sibling file to keep `patterns.rs` under
//! the LOC cap.

use super::*;

#[test]
fn checker_alternates_by_cell_parity() {
    // Hard checker (Softness 0) reproduces the integer-cell parity.
    let hard = &[0.0f32];
    assert_eq!(checker(0.5, 0.5, hard), 0.0);
    assert_eq!(checker(1.5, 0.5, hard), 1.0);
    assert_eq!(checker(1.5, 1.5, hard), 0.0);
    assert_eq!(checker(-0.5, 0.5, hard), 1.0); // floor (not truncation): cell -1 has parity 1
}

#[test]
fn stripes_width_controls_coverage() {
    // k = [Width, Frequency(0.35 ≈ 1×), Softness].
    let s = |u: f32, w: f32| stripes(u, 0.0, &[w, 0.35, 0.3]);
    assert!(s(0.5, 0.5) > 0.9, "band centre is covered");
    let cov = |w: f32| (0..100).map(|i| s(i as f32 / 100.0, w)).sum::<f32>();
    assert!(
        cov(0.8) > cov(0.3),
        "a wider Width paints more of the period"
    );
    assert!((s(0.25, 0.5) - s(1.25, 0.5)).abs() < 1e-6, "periodic");
}

#[test]
fn voronoi_randomness_and_edges_change_the_field() {
    let cells = &[1.0, 0.0, 0.0, 0.0]; // F1 cells
    let cracks = &[1.0, 0.0, 0.0, 1.0]; // F2−F1 edges
    let (mut diff, mut n) = (0.0f32, 0.0f32);
    for i in 0..60 {
        let (u, v) = (i as f32 * 0.41, i as f32 * 0.29);
        let a = voronoi(u, v, cells);
        let b = voronoi(u, v, cracks);
        assert!(
            (0.0..=1.0).contains(&a) && (0.0..=1.0).contains(&b),
            "bounded"
        );
        diff += (a - b).abs();
        n += 1.0;
    }
    assert!(diff / n > 0.05, "the Edges knob produces a different field");
}

#[test]
fn value_noise_is_bounded_and_deterministic() {
    for i in 0..50 {
        let (u, v) = (i as f32 * 0.37, i as f32 * -0.21);
        let a = value_noise(u, v);
        assert!((0.0..=1.0).contains(&a), "noise out of range: {a}");
        assert_eq!(a, value_noise(u, v), "noise must be a pure function");
    }
    assert_ne!(value_noise(0.5, 0.5), value_noise(10.5, 10.5));
}

#[test]
fn sample_image_is_bilinear_centre_coord_and_tiles() {
    let lum = [0u8, 255, 128, 64]; // 2×2: [0,255; 128,64]
    let img = ImageMask {
        lum: &lum,
        width: 2,
        height: 2,
    };
    assert!(
        (sample_image(&img, 0.25, 0.25) - 0.0).abs() < 1e-6,
        "texel (0,0)=0"
    );
    assert!(
        (sample_image(&img, 0.75, 0.25) - 1.0).abs() < 1e-6,
        "texel (1,0)=255"
    );
    assert!((sample_image(&img, 1.25, 0.25) - sample_image(&img, 0.25, 0.25)).abs() < 1e-6);
    for k in 0..30 {
        assert!((0.0..=1.0).contains(&sample_image(&img, k as f32 * 0.17, -(k as f32) * 0.17)));
    }
}

#[test]
fn every_kind_is_bounded_and_deterministic() {
    // Neutral params + two non-neutral sets (all six slots), across every kind.
    for params in [
        [0.5; 6],
        [0.9, 0.3, 0.8, 0.5, 0.2, 0.7],
        [0.1, 0.7, 0.2, 0.5, 0.9, 0.3],
    ] {
        for k in 0..TextureKind::COUNT {
            let kind = TextureKind::from_u8(k);
            for i in 0..40 {
                let (u, v) = (i as f32 * 0.31 - 3.0, i as f32 * -0.27 + 1.5);
                let a = sample_kind(kind, [u, v], params, None);
                assert!(
                    (0.0..=1.0).contains(&a),
                    "{} out of range: {a}",
                    kind.name()
                );
                assert_eq!(
                    a,
                    sample_kind(kind, [u, v], params, None),
                    "{} must be pure",
                    kind.name()
                );
            }
        }
    }
}

#[test]
fn every_procedural_kind_varies_across_the_plane() {
    // Each procedural (not None / Image) must produce a non-flat field at its defaults.
    for k in 1..TextureKind::COUNT {
        let kind = TextureKind::from_u8(k);
        if matches!(kind, TextureKind::Image) {
            continue;
        }
        // Seed each slot from the kind's spec defaults (slots beyond the specs stay neutral).
        let mut params = [0.5f32; 6];
        for (slot, s) in param_specs(kind).iter().enumerate() {
            params[slot] = s.default;
        }
        let (mut lo, mut hi) = (f32::INFINITY, f32::NEG_INFINITY);
        for i in 0..72 {
            for j in 0..72 {
                let a = sample_kind(kind, [i as f32 * 0.17, j as f32 * 0.19], params, None);
                lo = lo.min(a);
                hi = hi.max(a);
            }
        }
        assert!(
            hi - lo > 0.05,
            "{} should vary (lo={lo} hi={hi})",
            kind.name()
        );
    }
}

#[test]
fn knob_defaults_to_neutral_past_the_slice() {
    assert_eq!(knob(&[], 0), 0.5);
    assert_eq!(knob(&[0.2], 0), 0.2);
    assert_eq!(knob(&[0.2], 3), 0.5, "out-of-range slot is neutral");
}

#[test]
fn texture_preview_is_grayscale_opaque_and_varies() {
    let (w, h) = (24u32, 16u32);
    let mut buf = vec![0u8; (w * h * 4) as usize];
    let mut params = [0.5f32; 6];
    params[2] = 0.0; // hard checker → crisp dark/light cells
    render_texture_preview(
        TextureKind::Checker,
        params,
        [1.0, 1.0],
        [0.0, 0.0],
        None,
        None,
        &mut buf,
        w,
        h,
    );
    let (mut dark, mut light) = (0, 0);
    for px in buf.chunks_exact(4) {
        assert_eq!(px[3], 255, "preview pixels are opaque");
        assert!(px[0] == px[1] && px[1] == px[2], "preview is grayscale");
        if px[0] < 40 {
            dark += 1;
        } else if px[0] > 215 {
            light += 1;
        }
    }
    assert!(dark > 0 && light > 0, "checker preview shows both cells");
    // A too-small buffer is a no-op (must not panic or write OOB).
    let mut tiny = vec![7u8; 4];
    render_texture_preview(
        TextureKind::Checker,
        params,
        [1.0, 1.0],
        [0.0, 0.0],
        None,
        None,
        &mut tiny,
        w,
        h,
    );
    assert_eq!(tiny, [7u8; 4], "too-small buffer left untouched");
}

#[test]
fn texture_preview_colorizes_with_the_ramp_and_shows_alpha() {
    use crate::ramp_alpha::RampAlphaMode;
    let (w, h) = (32u32, 24u32);
    let mut params = [0.5f32; 6];
    params[2] = 0.0; // hard checker → s is exactly 0 or 1
    // LUT: opaque RED at s=0 … TRANSPARENT green at s=1 (sRGB-straight RGBA).
    let mut lut = [[0.0f32; 4]; 256];
    for (i, c) in lut.iter_mut().enumerate() {
        let t = i as f32 / 255.0;
        *c = [1.0 - t, t, 0.0, 1.0 - t]; // red→green, alpha 1→0
    }
    let render = |mode| {
        let mut buf = vec![0u8; (w * h * 4) as usize];
        render_texture_preview(
            TextureKind::Checker,
            params,
            [1.0, 1.0],
            [0.0, 0.0],
            None,
            Some((&lut[..], mode)),
            &mut buf,
            w,
            h,
        );
        buf
    };
    // Sprite-alpha mode: the s=1 (transparent) cells composite over the checker → NOT pure green;
    // the s=0 cells are opaque red.
    let sprite = render(RampAlphaMode::TextureAlpha);
    let mut reddish = 0;
    let mut pure_green = 0;
    for px in sprite.chunks_exact(4) {
        if px[0] > 150 && px[1] < 90 {
            reddish += 1;
        }
        if px[0] < 40 && px[1] > 215 {
            pure_green += 1;
        }
    }
    assert!(reddish > 0, "the opaque ramp end paints red");
    assert_eq!(
        pure_green, 0,
        "the transparent ramp end is composited over the checker, never pure green"
    );
    // Off mode: alpha ignored → the s=1 cells ARE pure-ish green (no checker bleed).
    let off = render(RampAlphaMode::None);
    let off_green = off
        .chunks_exact(4)
        .filter(|px| px[0] < 40 && px[1] > 215)
        .count();
    assert!(off_green > 0, "Off mode shows the opaque green end");
}
