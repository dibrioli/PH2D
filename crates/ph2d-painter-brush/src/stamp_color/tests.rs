//! Behavioural tests for the multi-layer coloured stamp ([`super`]).

use super::*;
use crate::spec::BrushSpec;
use crate::texture::{ImageMask, ImageRgb, TextureKind};

/// A BrushSpec whose Shape is an Image (so each layer's silhouette REPLACES the falloff), no Grain.
fn shape_image_spec() -> BrushSpec {
    let mut spec = BrushSpec::default();
    spec.shape.kind = TextureKind::Image;
    spec
}

/// A 2×2 luminance layer (row-major), full white where `on`, transparent (0) elsewhere.
fn layer(on: [bool; 4]) -> Vec<u8> {
    on.iter().map(|&b| if b { 255 } else { 0 }).collect()
}

fn mask(lum: &[u8]) -> ImageMask<'_> {
    ImageMask {
        lum,
        width: 2,
        height: 2,
    }
}

/// Read the straight RGBA at the stamp centre (the most-covered texel) by sampling the blit's source.
fn center_rgba(stamp: &ColorStampMask) -> ([f32; 3], f32) {
    super::sample_color_mask(stamp, 0.0, 0.0)
}

#[test]
fn single_full_layer_paints_its_colour_everywhere() {
    let spec = shape_image_spec();
    let l = layer([true, true, true, true]);
    let stamp = render_color_stamp_mask(
        &spec,
        &[mask(&l)],
        &[[1.0, 0.0, 0.0]],
        &[],
        &[],
        None,
        None,
        16,
    );
    let (rgb, a) = center_rgba(&stamp);
    assert!(a > 0.95, "full layer ⇒ opaque: {a}");
    assert!(
        (rgb[0] - 1.0).abs() < 0.02 && rgb[1] < 0.02 && rgb[2] < 0.02,
        "centre is the layer colour (red): {rgb:?}"
    );
}

#[test]
fn higher_layer_paints_above_the_lower_one() {
    // Bottom = red full-cover, top = green full-cover. Top is last ⇒ on top ⇒ centre reads green.
    let spec = shape_image_spec();
    let bottom = layer([true, true, true, true]);
    let top = layer([true, true, true, true]);
    let stamp = render_color_stamp_mask(
        &spec,
        &[mask(&bottom), mask(&top)],
        &[[1.0, 0.0, 0.0], [0.0, 1.0, 0.0]],
        &[],
        &[],
        None,
        None,
        16,
    );
    let (rgb, a) = center_rgba(&stamp);
    assert!(a > 0.95, "fully covered: {a}");
    assert!(
        rgb[1] > 0.95 && rgb[0] < 0.05,
        "the higher (top) layer's green wins over the lower red: {rgb:?}"
    );
}

#[test]
fn a_transparent_top_lets_the_lower_layer_show_through() {
    // Top layer is empty (no coverage) ⇒ the bottom red shows.
    let spec = shape_image_spec();
    let bottom = layer([true, true, true, true]);
    let top = layer([false, false, false, false]);
    let stamp = render_color_stamp_mask(
        &spec,
        &[mask(&bottom), mask(&top)],
        &[[1.0, 0.0, 0.0], [0.0, 1.0, 0.0]],
        &[],
        &[],
        None,
        None,
        16,
    );
    let (rgb, _) = center_rgba(&stamp);
    assert!(
        rgb[0] > 0.95 && rgb[1] < 0.05,
        "empty top ⇒ bottom red shows: {rgb:?}"
    );
}

#[test]
fn no_layers_is_fully_transparent() {
    let spec = shape_image_spec();
    let stamp = render_color_stamp_mask(&spec, &[], &[], &[], &[], None, None, 8);
    let (_, a) = center_rgba(&stamp);
    assert_eq!(a, 0.0, "no layers ⇒ nothing painted");
}

#[test]
fn blit_composites_the_stamp_colour_onto_the_canvas() {
    // A 1-layer blue stamp blitted opaquely onto a black canvas turns the centre blue.
    let spec = shape_image_spec();
    let l = layer([true, true, true, true]);
    let stamp = render_color_stamp_mask(
        &spec,
        &[mask(&l)],
        &[[0.0, 0.0, 1.0]],
        &[],
        &[],
        None,
        None,
        32,
    );
    let (w, h) = (16u32, 16u32);
    let mut buf = vec![0u8; (w * h * 4) as usize];
    let rect = blit_color_stamp(&mut buf, w, h, [8.0, 8.0], 6.0, &stamp, &spec, 1.0, false);
    assert!(rect.is_some(), "the blit touched the canvas");
    let i = ((8 * w + 8) * 4) as usize;
    assert!(
        buf[i] < 20 && buf[i + 1] < 20 && buf[i + 2] > 230 && buf[i + 3] > 230,
        "centre pixel is opaque blue: {:?}",
        &buf[i..i + 4]
    );
}

#[test]
fn deterministic_render() {
    let spec = shape_image_spec();
    let l = layer([true, false, true, true]);
    let a = render_color_stamp_mask(
        &spec,
        &[mask(&l)],
        &[[0.3, 0.6, 0.9]],
        &[],
        &[],
        None,
        None,
        24,
    );
    let b = render_color_stamp_mask(
        &spec,
        &[mask(&l)],
        &[[0.3, 0.6, 0.9]],
        &[],
        &[],
        None,
        None,
        24,
    );
    assert_eq!(a.data, b.data, "same inputs ⇒ byte-identical stamp (HR-5)");
}

#[test]
fn grain_colour_ramp_tints_the_composite() {
    // With a Grain Colour Ramp, the Grain value indexes a COLOUR that multiplies the tip. A constant
    // GREEN ramp over a WHITE layer ⇒ the composite reads green (white × green), not merely darkened.
    let mut spec = shape_image_spec();
    spec.texture.kind = TextureKind::Checker; // an active Grain (the constant ramp ignores its value)
    let l = layer([true, true, true, true]);
    let ramp = [[0.0f32, 1.0, 0.0, 1.0]; 256]; // constant green, full alpha
    let stamp = render_color_stamp_mask(
        &spec,
        &[mask(&l)],
        &[[1.0, 1.0, 1.0]],
        &[],
        &[],
        None,
        Some(&ramp),
        16,
    );
    let (rgb, a) = center_rgba(&stamp);
    assert!(a > 0.9, "covered: {a}");
    assert!(
        rgb[1] > 0.9 && rgb[0] < 0.1 && rgb[2] < 0.1,
        "the Grain ramp tints the white tip green: {rgb:?}"
    );
}

#[test]
fn texture_color_layer_paints_its_own_rgb_not_the_flat_tint() {
    // A full-cover layer whose flat tint is RED but whose captured RGB image is BLUE. With Texture
    // Color (the `layer_rgb` slot present) the stamp must read the layer's OWN blue, not the red tint.
    let spec = shape_image_spec();
    let l = layer([true, true, true, true]);
    let blue_rgb: Vec<u8> = [0u8, 0, 255].repeat(4); // 2×2 texels, all blue
    let rgb = ImageRgb {
        rgb: &blue_rgb,
        width: 2,
        height: 2,
    };
    let stamp = render_color_stamp_mask(
        &spec,
        &[mask(&l)],
        &[[1.0, 0.0, 0.0]], // flat tint = red (must be ignored)
        &[Some(rgb)],
        &[],
        None,
        None,
        16,
    );
    let (c, a) = center_rgba(&stamp);
    assert!(a > 0.95, "covered: {a}");
    assert!(
        c[2] > 0.95 && c[0] < 0.05 && c[1] < 0.05,
        "Texture Color samples the layer's own blue RGB, not the red flat tint: {c:?}"
    );
}

/// PARITY GATE — `accumulate_color_stamps_fused` (the per-Move bottleneck fast path) must produce
/// BYTE-IDENTICAL coverage maps + the same touched rect as N sequential `accumulate_color_stamp_coverage`
/// calls (the reference oracle). This is the executable assertion that the perf opt changed nothing
/// observable: it guards HR-5 (cross-platform replay hash) and the `ph2d-tool-painter` per-layer-colour
/// correctness tests, which both rest on this kernel. If it ever goes RED, the fused path diverged.
#[test]
fn fused_per_layer_accumulate_is_bit_identical_to_sequential() {
    let spec = shape_image_spec();
    // 4 layers with distinct silhouettes (so the per-layer alpha — hence coverage — genuinely differs).
    let l0 = layer([true, true, true, true]); // full
    let l1 = layer([false, true, true, false]); // anti-diagonal-ish
    let l2 = layer([true, false, false, true]); // diagonal
    let l3 = layer([false, false, true, true]); // bottom row
    let cols = [
        [1.0, 0.0, 0.0],
        [0.0, 1.0, 0.0],
        [0.0, 0.0, 1.0],
        [1.0, 1.0, 0.0],
    ];
    let stamps: Vec<ColorStampMask> = [&l0, &l1, &l2, &l3]
        .iter()
        .zip(cols.iter())
        .map(|(l, c)| render_color_stamp_mask(&spec, &[mask(l)], &[*c], &[], &[], None, None, 24))
        .collect();
    let n = stamps.len();

    let (w, h) = (40u32, 40u32);
    let len = (w as usize) * (h as usize);
    // A handful of OVERLAPPING dabs at fractional centres + varied radii + partial coverage — exercises
    // bilinear taps, the `over` build-up across dabs, and clamped edges.
    let dabs: [([f32; 2], f32, f32); 5] = [
        ([18.3, 19.7], 9.0, 0.8),
        ([22.6, 21.1], 7.5, 1.0),
        ([15.0, 24.4], 11.2, 0.5),
        ([20.0, 20.0], 6.0, 0.35),
        ([2.4, 3.9], 8.0, 0.9), // clipped against the canvas origin
    ];

    let mut seq: Vec<Vec<u8>> = (0..n).map(|_| vec![0u8; len]).collect();
    let mut fused: Vec<Vec<u8>> = (0..n).map(|_| vec![0u8; len]).collect();
    for &(center, radius, cov) in &dabs {
        // Reference: union the per-layer rects exactly as the old call site did.
        let mut seq_rect: Option<crate::dab::DirtyRect> = None;
        for (i, stamp) in stamps.iter().enumerate() {
            if let Some(r) =
                accumulate_color_stamp_coverage(&mut seq[i], w, h, center, radius, stamp, cov)
            {
                seq_rect = Some(match seq_rect {
                    None => r,
                    Some(a) => {
                        let x0 = a.x.min(r.x);
                        let y0 = a.y.min(r.y);
                        let x1 = (a.x + a.w).max(r.x + r.w);
                        let y1 = (a.y + a.h).max(r.y + r.h);
                        crate::dab::DirtyRect {
                            x: x0,
                            y: y0,
                            w: x1 - x0,
                            h: y1 - y0,
                        }
                    }
                });
            }
        }
        let fused_rect =
            accumulate_color_stamps_fused(&mut fused, w, h, center, radius, &stamps, cov);
        assert_eq!(
            fused_rect.map(|r| (r.x, r.y, r.w, r.h)),
            seq_rect.map(|r| (r.x, r.y, r.w, r.h)),
            "fused touched-rect must equal the union of the sequential per-layer rects"
        );
    }
    for i in 0..n {
        assert_eq!(
            fused[i], seq[i],
            "layer {i}: fused coverage map must be byte-identical to the sequential kernel"
        );
    }
}

/// GATE (perf, `accumulate_color_stamps_fused_batch`): the batched row-banded accumulate must produce
/// BYTE-IDENTICAL coverage maps + the same union rect as the per-dab fused loop (itself gated against
/// the sequential oracle above). Runs the batch at BOTH sizes: small (serial fallback) and large enough
/// to cross `PARALLEL_MIN_AREA` (the banded threads) — so the parallel path itself is what's verified.
/// If this goes RED, the banding broke the per-pixel dab order (HR-5).
#[test]
fn batched_fused_accumulate_is_bit_identical_to_sequential() {
    let spec = shape_image_spec();
    let l0 = layer([true, true, true, true]);
    let l1 = layer([false, true, true, false]);
    let l2 = layer([true, false, false, true]);
    let cols = [[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]];
    let stamps: Vec<ColorStampMask> = [&l0, &l1, &l2]
        .iter()
        .zip(cols.iter())
        .map(|(l, c)| render_color_stamp_mask(&spec, &[mask(l)], &[*c], &[], &[], None, None, 24))
        .collect();
    let n = stamps.len();
    // (label, canvas side, dab radius scale): small stays under PARALLEL_MIN_AREA (serial fallback);
    // large crosses it (Σ dab boxes ≈ 4·(2·90)² ≈ 130k+ px → the banded threads run).
    for (label, side, rs) in [("serial", 40u32, 1.0f32), ("banded", 640u32, 10.0)] {
        let (w, h) = (side, side);
        let len = (w as usize) * (h as usize);
        let k = side as f32 / 40.0;
        let dabs: [([f32; 2], f32, f32); 5] = [
            ([18.3 * k, 19.7 * k], 9.0 * rs, 0.8),
            ([22.6 * k, 21.1 * k], 7.5 * rs, 1.0),
            ([15.0 * k, 24.4 * k], 11.2 * rs, 0.5),
            ([20.0 * k, 20.0 * k], 6.0 * rs, 0.35),
            ([2.4, 3.9], 8.0 * rs, 0.9), // clipped against the canvas origin
        ];
        let mut per_dab: Vec<Vec<u8>> = (0..n).map(|_| vec![0u8; len]).collect();
        let mut ref_rect: Option<crate::dab::DirtyRect> = None;
        for &(center, radius, cov) in &dabs {
            if let Some(r) =
                accumulate_color_stamps_fused(&mut per_dab, w, h, center, radius, &stamps, cov)
            {
                ref_rect = Some(match ref_rect {
                    None => r,
                    Some(a) => {
                        let x0 = a.x.min(r.x);
                        let y0 = a.y.min(r.y);
                        let x1 = (a.x + a.w).max(r.x + r.w);
                        let y1 = (a.y + a.h).max(r.y + r.h);
                        crate::dab::DirtyRect {
                            x: x0,
                            y: y0,
                            w: x1 - x0,
                            h: y1 - y0,
                        }
                    }
                });
            }
        }
        let mut batched: Vec<Vec<u8>> = (0..n).map(|_| vec![0u8; len]).collect();
        let fd: Vec<FusedDab> = dabs
            .iter()
            .map(|&(center, radius, coverage)| FusedDab {
                center,
                radius,
                coverage,
            })
            .collect();
        let batch_rect = accumulate_color_stamps_fused_batch(&mut batched, w, h, &fd, &stamps);
        assert_eq!(
            batch_rect.map(|r| (r.x, r.y, r.w, r.h)),
            ref_rect.map(|r| (r.x, r.y, r.w, r.h)),
            "{label}: batch union rect must equal the per-dab union"
        );
        for i in 0..n {
            assert_eq!(
                batched[i], per_dab[i],
                "{label}: layer {i} batched map must be byte-identical to the per-dab fused loop"
            );
        }
    }
}
