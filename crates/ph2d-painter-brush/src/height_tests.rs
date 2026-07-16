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
    let mut radius = vec![0.0f32; (W * W) as usize];
    let mut fields = HeightFields {
        height: &mut dst,
        paint: &mut paint,
        grain: &mut grain,
        film: &mut film,
        radius: &mut radius,
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

/// Drive a plough (uniform 1-load ground, a straight run of overlapping dabs with `impasto_push`)
/// through the REAL kernel sequence the deposit runs — un-paint the standing wave lobe, bite, bank
/// with `share`, re-lay the lobe at the new tip — and return the displacement plane.
fn ploughed_plane(share: f32, count: u32) -> (Vec<f32>, f32, f32, f32) {
    use crate::height::{HeightDab, HeightFields, accumulate_dab_height};
    use crate::height_push::{PushBite, bank_dab_push, wave_lobe};
    const W: u32 = 200;
    let n = (W * W) as usize;
    let radius = 12.0f32;
    let ground = vec![1.0f32; n];
    let mut plane = vec![0.0f32; n];
    let mut height = vec![0.0f32; n];
    let mut paint = vec![0.0f32; n];
    let mut grain = vec![0u8; n];
    let mut film = vec![0u8; n];
    let mut radius_pl = vec![0.0f32; n];
    let mut scratch = Vec::new();
    let spec = crate::BrushSpec {
        radius_px: radius,
        impasto: true,
        impasto_depth: 0.5,
        impasto_push: 1.0,
        space_attenuation: false,
        ..Default::default()
    };
    let (y, x0, step) = (100.0f32, 60.0f32, 6.0f32);
    let mut wave = 0.0f32;
    let mut last_tip: Option<HeightDab> = None;
    for k in 0..count {
        let cx = x0 + step * k as f32;
        let dab = HeightDab {
            center: [cx, y],
            radius,
            coverage: 1.0,
            footprint: spec.footprint_deform(),
            prev_center: (k > 0).then_some([cx - step, y]),
            shape: None,
            grain: None,
            grain_image: None,
        };
        // The tool's order: un-paint the standing lobe FIRST (paint unchanged since it was laid,
        // so the (1 − paint) weights recompute to the exact numbers that laid it).
        if let (Some(tip), true) = (last_tip.take(), wave > 0.0) {
            let _ = wave_lobe(&mut plane, &paint, &mut scratch, W, W, &tip, wave, -1.0);
        }
        let mut fields = HeightFields {
            height: &mut height,
            paint: &mut paint,
            grain: &mut grain,
            film: &mut film,
            radius: &mut radius_pl,
        };
        let mut bite = PushBite {
            ground: &ground,
            plane: &mut plane,
            displaced: 0.0,
        };
        let _ = accumulate_dab_height(&mut fields, W, W, &spec, &dab, Some(&mut bite));
        let displaced = bite.displaced;
        let (_, carried) = bank_dab_push(
            &mut plane,
            &paint,
            &mut scratch,
            W,
            W,
            &dab,
            displaced,
            share,
        );
        wave += carried;
        if wave > 0.0
            && wave_lobe(&mut plane, &paint, &mut scratch, W, W, &dab, wave, 1.0).is_some()
        {
            last_tip = Some(HeightDab { ..dab });
        }
        // The ledger closes at EVERY step, not just at the end — a lobe that lays more than it
        // un-lays would still sum to zero eventually if a later error cancelled it.
        let ledger: f64 = plane.iter().map(|&v| f64::from(v)).sum();
        assert!(
            ledger.abs() < 0.5,
            "dab {k}: the plane's ledger drifted to {ledger:+.3} loads*px^2 — displacement must \
             move paint, never create or destroy it"
        );
    }
    (plane, x0, step * (count - 1) as f32 + x0, radius)
}

/// Zone split of a plough's banked (positive) plane: (ahead of the end, behind the start,
/// lateral, inside the swath), as fractions of the banked total.
fn plough_zones(plane: &[f32], first_x: f32, last_x: f32, radius: f32) -> (f64, f64, f64, f64) {
    const W: usize = 200;
    let y = 100.0f32;
    let (mut ahead, mut behind, mut lateral, mut total) = (0.0f64, 0.0f64, 0.0f64, 0.0f64);
    for py in 0..W {
        for px in 0..W {
            let v = plane[py * W + px];
            if v <= 0.0 {
                continue;
            }
            total += f64::from(v);
            let (fx, fy) = (px as f32 + 0.5, py as f32 + 0.5);
            if fx > last_x + radius {
                ahead += f64::from(v);
            } else if fx < first_x - radius {
                behind += f64::from(v);
            } else if (fy - y).abs() > radius {
                lateral += f64::from(v);
            }
        }
    }
    let swath = total - ahead - behind - lateral;
    (
        ahead / total,
        behind / total,
        lateral / total,
        swath / total,
    )
}

/// **The ploughed paint waits at the stroke's FRONTIER — the bow wave** (Enio's fila-última item:
/// *"o desenho da tinta deslocada não convence"*; diagnosed by rendering: the displaced paint
/// ringed the whole footprint like a stamped cookie-cutter — as tall behind the START as anywhere
/// — because every dab banked around itself; a blade pushes a WAVE ahead of its tip, IMPaSTo NPAR
/// 2004, `v_p = −c∇p`).
///
/// Per-dab forward *banking* cannot make that wave: the next dab paints over it and the envelope
/// bite (a one-shot `ground × Δm`) cannot re-transport it — measured: 0.4% ever landed ahead and
/// 16% fossilised inside the swath, which is MUT S quantified. So the wave is a session SCALAR
/// ([`crate::height_push::DEPOSIT_FORWARD_SHARE`] of each bite joins it, the rest sheds into the
/// lateral wake) painted as an exactly-removable lobe at the CURRENT tip ([`wave_lobe`]) — it
/// travels with the brush and rests at the frontier when the stroke ends.
///
/// Measured on this fixture: share 0.6 ⇒ ahead 42.8% · lateral 45.8% · behind 4.5% · swath 6.9%;
/// share 0 (the old drawing) ⇒ ahead 0.7%. The bars are an order of magnitude on both sides.
///
/// **Mutations that must bleed:** (a) `DEPOSIT_FORWARD_SHARE → 0.0` — no wave, `ahead` collapses;
/// (b) drop the un-paint of the standing lobe — a fossil lobe trail down the whole swath, the
/// swath bound explodes; (c) un-negate `sweep_axis` in [`wave_lobe`] — the "wave" paints backward
/// into the swath.
#[test]
fn the_ploughed_paint_waits_at_the_strokes_frontier() {
    use crate::height_push::DEPOSIT_FORWARD_SHARE;
    let (plane, first_x, last_x, radius) = ploughed_plane(DEPOSIT_FORWARD_SHARE, 12);
    let (ahead, behind, lateral, swath) = plough_zones(&plane, first_x, last_x, radius);
    assert!(
        ahead >= 0.30,
        "only {:.1}% of the banked paint waits ahead of the stroke's end — the wave never made it \
         to the frontier (measured healthy: 42.8%)",
        ahead * 100.0
    );
    assert!(
        ahead >= 5.0 * behind,
        "the frontier ({:.1}%) must dwarf the start ({:.1}%) — a blade leaves nothing standing \
         where it set off; symmetry here is the stamped-cookie-cutter drawing this gate exists to \
         refuse",
        ahead * 100.0,
        behind * 100.0
    );
    assert!(
        lateral >= 0.25,
        "the lateral wake vanished ({:.1}%) — the wave should SHED as it rolls, leaving ridges \
         along the channel's sides",
        lateral * 100.0
    );
    assert!(
        swath <= 0.15,
        "{:.1}% of the displaced paint sits INSIDE the channel — either the standing lobe is not \
         being un-painted before the tip moves (a fossil trail), or the lobe points backward",
        swath * 100.0
    );

    // The presence sibling: with the share at 0 the old, purely lateral drawing must come back —
    // this pins that the wave is OPT-IN plumbing, and keeps the baseline the Conserve relies on.
    let (plane0, f0, l0, r0) = ploughed_plane(0.0, 12);
    let (ahead0, _, lateral0, _) = plough_zones(&plane0, f0, l0, r0);
    assert!(
        ahead0 < 0.05 && lateral0 > 0.75,
        "share 0 must reproduce the purely lateral bank (ahead {:.1}%, lateral {:.1}%)",
        ahead0 * 100.0,
        lateral0 * 100.0
    );
}

/// **The wave TRAVELS — it stands at the current tip, not where it used to be.** Real-time was
/// the first complaint the Push ever drew (*"não em tempo real"*, 2026-07-12), and a wave that
/// only appears at pen-up would be the same failure in new clothes. Mid-plough, the standing
/// positive plane ahead of the CURRENT tip must dwarf whatever remains around the tip of several
/// dabs ago (bit-for-bit zero, in fact — the un-paint is an exact negation; the tolerance here is
/// only the lateral wake the old tip legitimately keeps beside itself).
#[test]
fn the_wave_travels_with_the_tip() {
    use crate::height_push::DEPOSIT_FORWARD_SHARE;
    const W: usize = 200;
    let (plane, _, last_x, radius) = ploughed_plane(DEPOSIT_FORWARD_SHARE, 12);
    let y = 100.0f32;
    // Positive plane in the forward half-disc of a tip position.
    let probe = |cx: f32| -> f64 {
        let mut sum = 0.0f64;
        for py in 0..W {
            for px in 0..W {
                let v = plane[py * W + px];
                if v <= 0.0 {
                    continue;
                }
                let (dx, dy) = (px as f32 + 0.5 - cx, py as f32 + 0.5 - y);
                if dx > 0.0 && (dx * dx + dy * dy) <= (radius * 2.2) * (radius * 2.2) {
                    sum += f64::from(v);
                }
            }
        }
        sum
    };
    let at_tip = probe(last_x);
    let mid_x = last_x - 6.0 * 6.0; // the tip six dabs ago
    let at_old_tip = probe(mid_x) - probe(last_x).min(0.0);
    assert!(
        at_tip > 0.0,
        "no wave stands ahead of the final tip — the lobe never landed"
    );
    assert!(
        at_tip >= 2.0 * at_old_tip,
        "the wave left as much standing at an OLD tip ({at_old_tip:.1}) as at the current one \
         ({at_tip:.1}) — it is being laid but not taken back up: a trail, not a travelling wave"
    );
}
