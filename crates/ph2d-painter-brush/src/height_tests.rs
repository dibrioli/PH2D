//! Tests for the impasto height kernel ([`super`]) — split out for the workspace file-LOC cap.

use super::*;

#[test]
fn wire_discriminants_round_trip() {
    for v in 0..DepthSource::COUNT {
        assert_eq!(DepthSource::from_u8(v).to_u8(), v);
    }
    for v in 0..DrawTo::COUNT {
        assert_eq!(DrawTo::from_u8(v).to_u8(), v);
    }
    // Unknown wire values fall back to the default, never panic.
    assert_eq!(DepthSource::from_u8(200), DepthSource::default());
    assert_eq!(DrawTo::from_u8(200), DrawTo::default());
}

#[test]
fn default_draw_to_writes_both_channels() {
    let d = DrawTo::default();
    assert!(d.writes_color() && d.writes_depth());
    assert!(DrawTo::Color.writes_color() && !DrawTo::Color.writes_depth());
    assert!(!DrawTo::Depth.writes_color() && DrawTo::Depth.writes_depth());
}

use crate::texture::{TextureKind, TextureMapping};
use crate::{BrushSpec, Falloff};

const W: u32 = 33;

/// A dab at the canvas centre with the given spec; returns the deposited height field.
fn deposit(spec: &BrushSpec) -> Vec<f32> {
    deposit_fields(spec).0
}

/// The same dab, returning all three planes (relief + the ingredients it was derived from).
fn deposit_fields(spec: &BrushSpec) -> (Vec<f32>, Vec<f32>, Vec<u8>) {
    let mut dst = vec![0.0f32; (W * W) as usize];
    let footprint = spec.footprint_deform();
    let basis = crate::texture::dab_basis(
        &spec.texture,
        [1.0, 0.0],
        &mut 0u64,
        [W as f32, W as f32],
        [1.0, 0.0],
        footprint,
    );
    let dab = HeightDab {
        center: [16.5, 16.5],
        radius: 12.0,
        coverage: 1.0,
        footprint,
        shape: None,
        grain: Some(&basis),
        grain_image: None,
        prev_center: None, // a lone stamped dab — nothing to sweep back to
    };
    let mut paint = vec![0.0f32; (W * W) as usize];
    let mut grain = vec![0u8; (W * W) as usize];
    let mut film = vec![0u8; (W * W) as usize];
    let mut fields = HeightFields {
        height: &mut dst,
        paint: &mut paint,
        grain: &mut grain,
        film: &mut film,
    };
    let _ = accumulate_dab_height(&mut fields, W, W, spec, &dab, None);
    (dst, paint, grain)
}

/// The heights on the falloff's PLATEAU — the dab's flat interior, where Uniform must be level and
/// Grain must not be. `hardness = 0.6` ⇒ `w == 1` for `t < 0.6`, i.e. within 7 px of the centre.
fn plateau(h: &[f32]) -> Vec<f32> {
    let mut out = Vec::new();
    for y in 0..W as i32 {
        for x in 0..W as i32 {
            let (dx, dy) = (f64::from(x) + 0.5 - 16.5, f64::from(y) + 0.5 - 16.5);
            if (dx * dx + dy * dy).sqrt() < 6.0 {
                out.push(h[(y as usize) * (W as usize) + x as usize]);
            }
        }
    }
    out
}

#[test]
fn depth_source_uniform_is_level_and_grain_is_not() {
    // The gate the plan froze: Uniform lays a LEVEL plateau (the grain textures the pigment, not
    // the body); Grain lets the grain's striations into the relief, so the height VARIES inside the
    // dab. This is the whole difference between a smooth loaded brush and a bristle brush — if the
    // two ever produced the same field, `DepthSource` would be a dead knob.
    let mut base = BrushSpec {
        impasto: true,
        impasto_depth: 1.0,
        hardness: 0.6,
        falloff: Falloff::Smooth,
        radius_px: 10.0, // the impasto REFERENCE radius: size-scaling is off (scale 1) so these test the profile math
        ..Default::default()
    };
    base.texture.kind = TextureKind::Noise; // a Grain with real per-texel variation
    base.texture.mapping = TextureMapping::ViewPlane;

    let uniform = deposit(&BrushSpec {
        impasto_source: DepthSource::Uniform,
        ..base
    });
    let grain = deposit(&BrushSpec {
        impasto_source: DepthSource::Grain,
        ..base
    });

    let u_plateau = plateau(&uniform);
    let g_plateau = plateau(&grain);
    assert!(!u_plateau.is_empty(), "the fixture sampled a plateau");

    let spread = |v: &[f32]| {
        let (lo, hi) = v
            .iter()
            .fold((f32::MAX, f32::MIN), |(lo, hi), &x| (lo.min(x), hi.max(x)));
        hi - lo
    };
    assert!(
        spread(&u_plateau) < 1e-6,
        "Uniform: the plateau must be LEVEL — the Grain shapes pigment, not body (spread {})",
        spread(&u_plateau)
    );
    assert!(
        spread(&g_plateau) > 0.05,
        "Grain: the striations must reach the relief (spread {})",
        spread(&g_plateau)
    );
    // And the plateau really is at full depth (nothing silently scaled it away).
    assert!(
        (u_plateau[0] - 1.0).abs() < 1e-6,
        "full depth on the plateau"
    );
}

#[test]
fn every_body_knob_is_a_pure_function_of_the_stored_ingredients() {
    // Enio, 2026-07-12: "coloque todos os parâmetros vivos em tempo real para ajustes depois do
    // traço." That is only possible if the stroke stores what it DEPOSITED (paint + grain) rather
    // than the height it happened to compute — so this pins the property the whole live-edit rests
    // on: re-deriving from the stored planes at a new setting equals depositing at that setting.
    // Break it (bake anything into the height that `derive_height` cannot see) and the panel's
    // sliders start lying about the last stroke.
    let mut base = BrushSpec {
        impasto: true,
        impasto_depth: 0.8,
        hardness: 0.3,
        falloff: Falloff::Smooth,
        radius_px: 10.0, // the impasto REFERENCE radius: size-scaling is off (scale 1) so these test the profile math
        ..Default::default()
    };
    base.texture.kind = TextureKind::Noise;
    base.texture.mapping = TextureMapping::ViewPlane;

    // Deposit ONCE, with one arbitrary setting; keep its ingredients.
    let (_, paint, grain) = deposit_fields(&BrushSpec {
        impasto_body: 1.0,
        impasto_source: DepthSource::Grain,
        ..base
    });

    // Now every other setting must be reachable from those ingredients alone.
    for body in [0.0f32, 0.35, 1.0] {
        for source in [DepthSource::Uniform, DepthSource::Grain] {
            for depth in [-0.9f32, 0.2, 1.0] {
                let spec = BrushSpec {
                    impasto_body: body,
                    impasto_source: source,
                    impasto_depth: depth,
                    ..base
                };
                let fresh = deposit(&spec);
                for i in 0..(W * W) as usize {
                    let live = derive_height(&spec, paint[i], f32::from(grain[i]) / 255.0);
                    assert!(
                        (live - fresh[i]).abs() < 1e-6,
                        "re-derived relief must equal a fresh deposit at the same settings \
                         (body {body}, source {source:?}, depth {depth}, px {i}: {live} vs {})",
                        fresh[i]
                    );
                }
            }
        }
    }
}

#[test]
fn one_pass_is_one_thickness_and_carving_deepens() {
    // The envelope, now taken on the PAINT: a heavier bid (more paint) owns the pixel, a lighter
    // one changes nothing — so scrubbing back and forth inside a stroke leaves ONE thickness. And
    // the sign rides Depth, so a carving brush deepens with more paint instead of being cancelled.
    let spec = BrushSpec {
        impasto: true,
        impasto_depth: 1.0,
        impasto_body: 0.0, // the identity profile — isolate the envelope, not the curve
        ..Default::default()
    };
    let carve = BrushSpec {
        impasto_depth: -1.0,
        ..spec
    };
    assert!(derive_height(&spec, 0.7, 1.0) > derive_height(&spec, 0.3, 1.0));
    assert_eq!(derive_height(&spec, 0.0, 1.0), 0.0, "no paint, no body");
    assert!(
        derive_height(&carve, 0.7, 1.0) < derive_height(&carve, 0.3, 1.0),
        "carving deepens with more paint"
    );
    assert!(
        derive_height(&carve, 0.7, 1.0) < 0.0,
        "a carving brush digs in"
    );
}

#[test]
fn eraser_scrubs_the_relief_it_finds() {
    // Erasing must remove the RELIEF, not carve a hole — otherwise the pigment goes and the light
    // still reports a ridge (ghost relief). Full coverage erases to flat.
    let spec = BrushSpec {
        impasto: true,
        impasto_depth: 1.0,
        hardness: 1.0, // hard disk → deterministic full coverage inside
        falloff: Falloff::Constant,
        radius_px: 10.0, // the impasto REFERENCE radius: size-scaling is off (scale 1) so these test the profile math
        ..Default::default()
    };
    let mut field = vec![0.8f32; (W * W) as usize];
    let footprint = spec.footprint_deform();
    let dab = HeightDab {
        center: [16.5, 16.5],
        radius: 12.0,
        coverage: 1.0,
        footprint,
        shape: None,
        grain: None,
        grain_image: None,
        prev_center: None,
    };
    let mut cov = vec![255u8; (W * W) as usize];
    let rect = erase_dab_height(&mut field, &mut cov, W, W, &spec, &dab)
        .expect("the eraser touched relief");
    // (The eraser scrubs the COMMITTED layer — height + coverage. The ingredient planes are the
    // live stroke's, and an erase drops that stroke: there is nothing left to re-derive.)
    assert!(rect.w > 0 && rect.h > 0);
    let centre = field[(16 * W + 16) as usize];
    assert!(
        centre.abs() < 1e-6,
        "under the dab the relief is gone ({centre})"
    );
    let corner = field[0];
    assert_eq!(corner, 0.8, "outside the dab the relief is untouched");
}
