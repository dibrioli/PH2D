//! Behavioural tests for the multi-layer coloured stamp ([`super`]).

use super::*;
use crate::spec::BrushSpec;
use crate::texture::{ImageMask, TextureKind};

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
    let stamp = render_color_stamp_mask(&spec, &[mask(&l)], &[[1.0, 0.0, 0.0]], None, 16);
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
    let stamp = render_color_stamp_mask(&spec, &[], &[], None, 8);
    let (_, a) = center_rgba(&stamp);
    assert_eq!(a, 0.0, "no layers ⇒ nothing painted");
}

#[test]
fn blit_composites_the_stamp_colour_onto_the_canvas() {
    // A 1-layer blue stamp blitted opaquely onto a black canvas turns the centre blue.
    let spec = shape_image_spec();
    let l = layer([true, true, true, true]);
    let stamp = render_color_stamp_mask(&spec, &[mask(&l)], &[[0.0, 0.0, 1.0]], None, 32);
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
    let a = render_color_stamp_mask(&spec, &[mask(&l)], &[[0.3, 0.6, 0.9]], None, 24);
    let b = render_color_stamp_mask(&spec, &[mask(&l)], &[[0.3, 0.6, 0.9]], None, 24);
    assert_eq!(a.data, b.data, "same inputs ⇒ byte-identical stamp (HR-5)");
}
