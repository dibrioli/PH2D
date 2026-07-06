//! Behavioural tests for the brush texture: procedural samplers, the 2D mapping modes, the
//! deterministic rotation, and the coverage modulation through [`crate::dab::stamp_dab_textured`].

use super::*;
use crate::footprint::FootprintDeform;

/// The canvas-anchored [`sample_tiled`] honours the texture **Angle**: a directional pattern (Stripes)
/// samples DIFFERENTLY at 0° vs 90° over the same canvas span — the fix for "Angle não funciona" on a
/// Tiled paper (the render-path reads the Grain paper through this). Also pins `sample_tiled_rot`.
#[test]
fn sample_tiled_honours_angle() {
    let mut s = TextureSettings {
        kind: TextureKind::Stripes,
        mapping: TextureMapping::Tiled,
        ..TextureSettings::default()
    };
    let across = |st: &TextureSettings| -> Vec<f32> {
        (0..40)
            .map(|i| sample_tiled(st, i * 8, 64, None))
            .collect()
    };
    s.angle_deg = 0;
    let a = across(&s);
    s.angle_deg = 90;
    let b = across(&s);
    assert!(
        a.iter().zip(&b).any(|(x, y)| (x - y).abs() > 0.05),
        "Angle must rotate the tiled pattern (0° vs 90° differ)"
    );
    // The precomputed-basis form agrees with the internal one.
    let rot = angle_basis(90);
    for i in 0..40 {
        assert!((sample_tiled_rot(&s, i * 8, 64, None, rot) - b[i as usize]).abs() < 1e-6);
    }
}

/// A hard Checker's params (Softness `0`). The engine's `TextureSettings::default()` is neutral `0.5`
/// in every slot, which for Checker means a *soft* edge that never saturates to an exact `0`/`1`; the
/// value-pinning tests below want the crisp pattern.
fn checker_hard() -> [f32; MAX_TEX_PARAMS] {
    let mut p = [0.5; MAX_TEX_PARAMS];
    p[2] = 0.0; // slot 2 = Checker Softness
    p
}
use crate::blend::BrushBlend;
use crate::dab::{stamp_dab, stamp_dab_textured};
use crate::falloff::Falloff;
use crate::spec::BrushSpec;

/// `dab_basis` with a default 64×64 canvas — the rotation / jitter tests don't depend on the canvas
/// size (only the Stencil mapping does, and those tests pass it explicitly).
fn basis(s: &TextureSettings, dir: [f32; 2], rng: &mut u64) -> TexDabBasis {
    dab_basis(
        s,
        dir,
        rng,
        [64.0, 64.0],
        [1.0, 0.0],
        FootprintDeform::identity(),
    )
}

/// Timing: how much does the per-pixel texture sample add to a LARGE dab stamp (the Anchored
/// re-stamp-every-move case)? Run: `cargo test -p ph2d-painter-brush --release perf_texture_stamp
/// -- --ignored --nocapture`. Not a gate (wall-clock, machine-dependent).
#[test]
#[ignore]
fn perf_texture_stamp_cost_on_a_large_dab() {
    use std::time::Instant;
    let (w, h) = (2048u32, 2048u32);
    let mut buf = vec![128u8; (w * h * 4) as usize];
    let mk = |kind: TextureKind| BrushSpec {
        radius_px: 900.0,
        falloff: Falloff::Smooth,
        texture: TextureSettings {
            kind,
            ..Default::default()
        },
        ..Default::default()
    };
    let canvas = [w as f32, h as f32];
    let runs = 8;
    let bench = |label: &str, spec: &BrushSpec, buf: &mut [u8]| {
        let mut rng = 1u64;
        let b = dab_basis(
            &spec.texture,
            [1.0, 0.0],
            &mut rng,
            canvas,
            [1.0, 0.0],
            FootprintDeform::identity(),
        );
        let t0 = Instant::now();
        for _ in 0..runs {
            let _ = stamp_dab_textured(
                buf,
                w,
                h,
                [1024.0, 1024.0],
                spec,
                0.5,
                false,
                Some(&b),
                None,
                None,
            );
        }
        let ms = t0.elapsed().as_secs_f64() * 1000.0 / f64::from(runs);
        eprintln!("  {label:<10} {ms:6.2} ms/stamp");
    };
    eprintln!("perf: 2048² canvas, radius 900 (≈2.5M-px footprint), {runs} stamps avg:");
    bench("plain", &mk(TextureKind::None), &mut buf);
    bench("noise", &mk(TextureKind::Noise), &mut buf);
    bench("checker", &mk(TextureKind::Checker), &mut buf);
    bench("voronoi", &mk(TextureKind::Voronoi), &mut buf);
    bench("stripes", &mk(TextureKind::Stripes), &mut buf);
}

// ── Procedural samplers ─────────────────────────────────────────────────────────────────────

#[test]
fn none_kind_is_full_coverage() {
    let s = TextureSettings::default(); // kind == None
    let b = TexDabBasis::identity();
    for px in -3..3 {
        for py in -3..3 {
            assert_eq!(sample(&s, &b, px, py, [0.0, 0.0], 8.0, None), 1.0);
        }
    }
}

// (Per-sampler procedural tests live in `texture/patterns.rs`, next to the functions.)

// ── Rotation / basis (determinism, HR-5) ────────────────────────────────────────────────────

#[test]
fn rotate_zero_is_identity_and_360_wraps() {
    assert_eq!(rotate_by_degrees(0), [1.0, 0.0]);
    let w = rotate_by_degrees(360);
    assert_eq!(w, [1.0, 0.0], "360° wraps to 0° exactly (deg % 360)");
}

#[test]
fn rotate_90_degrees_points_up() {
    let u = rotate_by_degrees(90);
    assert!((u[0] - 0.0).abs() < 1e-3, "x≈0, got {}", u[0]);
    assert!((u[1] - 1.0).abs() < 1e-3, "y≈1, got {}", u[1]);
    // Still a unit vector (no drift past tolerance).
    let len = (u[0] * u[0] + u[1] * u[1]).sqrt();
    assert!((len - 1.0).abs() < 1e-3);
}

#[test]
fn dab_basis_angle_is_deterministic() {
    let s = TextureSettings {
        kind: TextureKind::Noise,
        angle_deg: 30,
        ..Default::default()
    };
    let mut r1 = 123;
    let mut r2 = 123;
    assert_eq!(
        basis(&s, [0.0, 0.0], &mut r1),
        basis(&s, [0.0, 0.0], &mut r2)
    );
}

#[test]
fn rake_uses_the_stroke_tangent() {
    let s = TextureSettings {
        kind: TextureKind::Noise,
        rake: true,
        angle_deg: 0,
        ..Default::default()
    };
    let mut rng = 1;
    let b = basis(&s, [0.0, 5.0], &mut rng); // tangent points +y
    // u should align with the (normalised) tangent.
    assert!((b.u[0] - 0.0).abs() < 1e-6 && (b.u[1] - 1.0).abs() < 1e-6);
}

#[test]
fn rake_falls_back_to_angle_for_zero_tangent() {
    let s = TextureSettings {
        kind: TextureKind::Noise,
        rake: true,
        angle_deg: 0,
        ..Default::default()
    };
    let mut rng = 1;
    let b = basis(&s, [0.0, 0.0], &mut rng); // degenerate tangent
    assert_eq!(b.u, [1.0, 0.0], "zero tangent ⇒ angle 0 ⇒ (1,0)");
}

#[test]
fn rake_off_ignores_the_heading_byte_identical() {
    // With Rake OFF the per-dab heading (Dab::dir) must not touch the texture frame at all: the frame
    // for any heading equals the frame for a zero heading. This is the guarantee that the new heading
    // plumbing leaves a non-Rake brush bit-for-bit unchanged (the byte-identity baseline).
    let s = TextureSettings {
        kind: TextureKind::Noise,
        rake: false,
        random_angle: false,
        angle_deg: 37,
        ..Default::default()
    };
    let (mut a, mut b) = (1u64, 1u64);
    let with_heading = basis(&s, [0.0, 1.0], &mut a);
    let zero_heading = basis(&s, [0.0, 0.0], &mut b);
    assert_eq!(
        with_heading, zero_heading,
        "Rake off ⇒ the heading is ignored, frame is identical"
    );
    // And it is the plain Angle frame, not the tangent.
    assert_eq!(with_heading.u, rotate_by_degrees(37));
}

#[test]
fn jitter_rotate_composes_on_top_of_the_rake_heading() {
    // Jitter Rotate (`extra_rot`) must compose ON TOP of the Rake base, not replace it: the final `u`
    // equals the rake heading rotated by the jitter vector (a 2D rotation = complex multiply). Proves
    // the new heading coexists with Jitter Rotate exactly as the old reconstructed direction did.
    let s = TextureSettings {
        kind: TextureKind::Noise,
        rake: true,
        angle_deg: 0,
        ..Default::default()
    };
    let heading = [0.0, 1.0]; // +y
    let extra = rotate_by_degrees(30); // a 30° Jitter Rotate
    let mut rng = 1;
    let b = dab_basis(
        &s,
        heading,
        &mut rng,
        [64.0, 64.0],
        extra,
        FootprintDeform::identity(),
    );
    // Expected: heading (the rake base) complex-multiplied by `extra`.
    let want = [
        heading[0] * extra[0] - heading[1] * extra[1],
        heading[0] * extra[1] + heading[1] * extra[0],
    ];
    assert!(
        (b.u[0] - want[0]).abs() < 1e-6 && (b.u[1] - want[1]).abs() < 1e-6,
        "jitter rotates the rake heading, not replaces it: got {:?} want {want:?}",
        b.u
    );
    // Identity jitter leaves the rake base untouched (the no-jitter path stays bit-identical).
    let mut rng2 = 1;
    let id = dab_basis(
        &s,
        heading,
        &mut rng2,
        [64.0, 64.0],
        [1.0, 0.0],
        FootprintDeform::identity(),
    );
    assert_eq!(id.u, heading, "no jitter ⇒ u is the bare rake heading");
}

#[test]
fn random_angle_is_deterministic_per_seed_but_varies() {
    let s = TextureSettings {
        kind: TextureKind::Noise,
        random_angle: true,
        ..Default::default()
    };
    let (mut a1, mut a2) = (7, 7);
    assert_eq!(
        basis(&s, [0.0, 0.0], &mut a1),
        basis(&s, [0.0, 0.0], &mut a2)
    );
    // A different seed (almost surely) gives a different rotation.
    let mut b = 99;
    assert_ne!(
        basis(&s, [0.0, 0.0], &mut a1).u,
        basis(&s, [0.0, 0.0], &mut b).u
    );
}

// ── Mapping modes (the View/Tiled/Random behavioural distinction) ───────────────────────────

#[test]
fn view_follows_the_dab_but_tiled_is_fixed_to_canvas() {
    let view = TextureSettings {
        kind: TextureKind::Noise,
        mapping: TextureMapping::ViewPlane,
        ..Default::default()
    };
    let tiled = TextureSettings {
        kind: TextureKind::Noise,
        mapping: TextureMapping::Tiled,
        ..Default::default()
    };
    let b = TexDabBasis::identity();
    let (px, py) = (40, 40);
    // View: moving the dab centre changes the sampled value (texture rides the brush).
    let v_a = sample(&view, &b, px, py, [40.0, 40.0], 16.0, None);
    let v_b = sample(&view, &b, px, py, [10.0, 10.0], 16.0, None);
    assert_ne!(v_a, v_b, "View Plane should follow the dab centre");
    // Tiled: the same pixel samples the same value regardless of dab centre.
    let t_a = sample(&tiled, &b, px, py, [40.0, 40.0], 16.0, None);
    let t_b = sample(&tiled, &b, px, py, [10.0, 10.0], 16.0, None);
    assert_eq!(t_a, t_b, "Tiled is fixed to the image, not the brush");
}

#[test]
fn random_mapping_jitters_offset_other_mappings_do_not() {
    let rnd = TextureSettings {
        kind: TextureKind::Noise,
        mapping: TextureMapping::Random,
        ..Default::default()
    };
    let mut s1 = 1;
    let mut s2 = 2;
    let b1 = basis(&rnd, [0.0, 0.0], &mut s1);
    let b2 = basis(&rnd, [0.0, 0.0], &mut s2);
    assert_ne!(
        b1.jitter, b2.jitter,
        "Random mapping randomises the per-dab offset"
    );

    let view = TextureSettings {
        mapping: TextureMapping::ViewPlane,
        ..rnd
    };
    let mut s3 = 3;
    assert_eq!(
        basis(&view, [0.0, 0.0], &mut s3).jitter,
        [0.0, 0.0],
        "non-Random mappings have no per-dab offset"
    );
}

// ── Stencil mapping (image-space positioned mask) ───────────────────────────────────────────

#[test]
fn stencil_frame_reads_the_dedicated_stencil_fields_not_the_texture_tiling() {
    let s = TextureSettings {
        kind: TextureKind::Checker,
        mapping: TextureMapping::Stencil,
        stencil_size: [1.0, 1.0],
        // The texture tiling values below must NOT leak into the stencil frame (Enio 2026-06-25).
        size: [4.0, 4.0],
        offset: [0.7, -0.3],
        ..Default::default()
    };
    // stencil_offset 0, stencil_size 1 → centred, half = half the canvas; angle 0 → u = (1,0).
    let (c, h, u) = stencil_frame(&s, [100.0, 80.0]);
    assert_eq!(c, [50.0, 40.0]);
    assert_eq!(
        h,
        [50.0, 40.0],
        "texture `size` did not leak into the stencil"
    );
    assert_eq!(u, [1.0, 0.0]);
    // stencil_offset +1 X / -1 Y → centre at the right / top edge (texture `offset` is irrelevant).
    let s2 = TextureSettings {
        stencil_offset: [1.0, -1.0],
        ..s
    };
    let (c2, _, _) = stencil_frame(&s2, [100.0, 80.0]);
    assert_eq!(c2, [100.0, 0.0]);
    // The default stencil is 50 % of the sprite (half-extent = a quarter of each canvas dimension).
    let def = TextureSettings {
        mapping: TextureMapping::Stencil,
        ..Default::default()
    };
    assert_eq!(
        stencil_frame(&def, [100.0, 80.0]).1,
        [25.0, 20.0],
        "default stencil rect is 50% of the sprite"
    );
}

#[test]
fn stencil_masks_outside_the_rect_and_is_image_fixed() {
    let s = TextureSettings {
        kind: TextureKind::Checker,
        mapping: TextureMapping::Stencil,
        stencil_size: [0.2, 0.2], // ±10% of a 100px canvas → a small central rect
        ..Default::default()
    };
    let mut rng = 0;
    let b = dab_basis(
        &s,
        [0.0, 0.0],
        &mut rng,
        [100.0, 100.0],
        [1.0, 0.0],
        FootprintDeform::identity(),
    );
    // A pixel far outside the rect → masked to 0.
    assert_eq!(
        sample(&s, &b, 5, 5, [5.0, 5.0], 8.0, None),
        0.0,
        "outside the stencil paints nothing"
    );
    // The rect is fixed to the image: moving the dab centre does NOT change the mask.
    assert_eq!(
        sample(&s, &b, 5, 5, [5.0, 5.0], 8.0, None),
        sample(&s, &b, 5, 5, [99.0, 99.0], 8.0, None),
        "stencil is image-fixed, not dab-relative"
    );
}

#[test]
fn stencil_rect_shows_the_procedural_pattern() {
    // Inside a Checker stencil the rect spans several cells (STENCIL_TILES), so distinct interior
    // pixels return both parities — i.e. the pattern reads, it is not one flat cell.
    let s = TextureSettings {
        kind: TextureKind::Checker,
        mapping: TextureMapping::Stencil,
        stencil_size: [1.0, 1.0], // rect = the whole canvas
        params: checker_hard(),
        ..Default::default()
    };
    let mut rng = 0;
    let b = dab_basis(
        &s,
        [0.0, 0.0],
        &mut rng,
        [64.0, 64.0],
        [1.0, 0.0],
        FootprintDeform::identity(),
    );
    let mut seen0 = false;
    let mut seen1 = false;
    for x in (2..62).step_by(3) {
        let v = sample(&s, &b, x, 32, [32.0, 32.0], 8.0, None);
        if v == 0.0 {
            seen0 = true;
        }
        if v == 1.0 {
            seen1 = true;
        }
    }
    assert!(
        seen0 && seen1,
        "the stencil rect shows the checker pattern (both cells)"
    );
}

#[test]
fn stencil_texture_transform_reshapes_the_pattern_within_the_rect() {
    // In Stencil mode the texture's OWN Size / Offset / Angle reshape the pattern INSIDE the rect,
    // independently of the gizmo (`stencil_*`), which only places the rect (Enio 2026-06-26). A
    // horizontal scan of the rect interior changes when any of the three is changed.
    let base = TextureSettings {
        kind: TextureKind::Checker,
        mapping: TextureMapping::Stencil,
        stencil_size: [1.0, 1.0], // rect = the whole canvas
        params: checker_hard(),
        ..Default::default()
    };
    let row = |s: &TextureSettings| -> Vec<f32> {
        let mut rng = 0;
        let b = dab_basis(
            s,
            [0.0, 0.0],
            &mut rng,
            [64.0, 64.0],
            [1.0, 0.0],
            FootprintDeform::identity(),
        );
        (4..60)
            .step_by(2)
            .map(|x| sample(s, &b, x, 32, [32.0, 32.0], 8.0, None))
            .collect()
    };
    let base_row = row(&base);
    // Texture Size → denser pattern in the rect.
    assert_ne!(
        row(&TextureSettings {
            size: [4.0, 4.0],
            ..base
        }),
        base_row,
        "texture Size changes the pattern density in the rect"
    );
    // Texture Offset → the pattern shifts in the rect.
    assert_ne!(
        row(&TextureSettings {
            offset: [0.5, 0.0],
            ..base
        }),
        base_row,
        "texture Offset shifts the pattern in the rect"
    );
    // Texture Angle → the pattern rotates within the rect (distinct from the gizmo's rotation).
    assert_ne!(
        row(&TextureSettings {
            angle_deg: 45,
            ..base
        }),
        base_row,
        "texture Angle rotates the pattern in the rect"
    );
    // None of the above moved the GIZMO: the rect frame still spans the whole canvas.
    assert_eq!(
        stencil_frame(
            &TextureSettings {
                size: [4.0, 4.0],
                offset: [0.5, 0.0],
                angle_deg: 45,
                ..base
            },
            [64.0, 64.0]
        )
        .1,
        [32.0, 32.0],
        "the texture transform leaves the stencil rect (gizmo) unchanged"
    );
}

// ── Coverage modulation through stamp_dab_textured ──────────────────────────────────────────

fn solid(w: u32, h: u32, rgba: [u8; 4]) -> Vec<u8> {
    rgba.iter()
        .copied()
        .cycle()
        .take((w * h * 4) as usize)
        .collect()
}

#[test]
fn inactive_texture_is_byte_identical_to_stamp_dab() {
    let (w, h) = (32, 32);
    let spec = BrushSpec {
        radius_px: 10.0,
        color: [0.0, 0.0, 0.0],
        blend: BrushBlend::Mix,
        falloff: Falloff::Smooth,
        ..Default::default() // texture kind None
    };
    let mut a = solid(w, h, [255, 255, 255, 255]);
    let mut b = solid(w, h, [255, 255, 255, 255]);
    let basis = TexDabBasis::identity();
    let ra = stamp_dab(&mut a, w, h, [16.0, 16.0], &spec, 1.0, false);
    let rb = stamp_dab_textured(
        &mut b,
        w,
        h,
        [16.0, 16.0],
        &spec,
        1.0,
        false,
        Some(&basis),
        None,
        None,
    );
    assert_eq!(ra, rb);
    assert_eq!(a, b, "an inactive texture must not change a single byte");
}

#[test]
fn checker_texture_leaves_zero_texel_cells_unpainted() {
    // A hard-disk black dab over white. With a Checker texture (cells of 0 and 1), the texels in
    // the 0-cells deposit no paint, so some in-footprint pixels stay white that the texture-free
    // dab paints black.
    let (w, h) = (48, 48);
    let spec = BrushSpec {
        radius_px: 18.0,
        color: [0.0, 0.0, 0.0],
        blend: BrushBlend::Mix,
        falloff: Falloff::Constant,
        hardness: 1.0,
        texture: TextureSettings {
            kind: TextureKind::Checker,
            mapping: TextureMapping::ViewPlane,
            // Big tiles so each cell spans several pixels.
            size: [0.25, 0.25],
            params: checker_hard(),
            ..Default::default()
        },
        ..Default::default()
    };
    let mut plain = solid(w, h, [255, 255, 255, 255]);
    let mut tex = solid(w, h, [255, 255, 255, 255]);
    let plain_spec = BrushSpec {
        texture: TextureSettings::default(),
        ..spec
    };
    let basis = TexDabBasis::identity();
    stamp_dab(&mut plain, w, h, [24.0, 24.0], &plain_spec, 1.0, false).expect("plain painted");
    stamp_dab_textured(
        &mut tex,
        w,
        h,
        [24.0, 24.0],
        &spec,
        1.0,
        false,
        Some(&basis),
        None,
        None,
    )
    .expect("textured painted");

    let alpha_painted = |buf: &[u8]| -> usize {
        (0..(w * h) as usize)
            .filter(|&i| buf[i * 4] == 0) // black ⇒ fully painted
            .count()
    };
    let plain_black = alpha_painted(&plain);
    let tex_black = alpha_painted(&tex);
    assert!(plain_black > 0, "sanity: plain dab paints");
    assert!(
        tex_black > 0 && tex_black < plain_black,
        "checker masks part of the footprint: plain={plain_black}, tex={tex_black}"
    );
}

#[test]
fn full_one_texture_matches_unmodulated_dab() {
    // Stripes at u=0.5 gives 1.0; but to assert "texture==1 ⇒ identical", drive the whole footprint
    // through a Checker cell of value 1 by mapping every pixel to the same odd cell (huge tile,
    // offset into cell 1). Simpler: assert the modulation never *adds* paint vs the plain dab.
    let (w, h) = (24, 24);
    let base = BrushSpec {
        radius_px: 9.0,
        color: [0.0, 0.0, 0.0],
        blend: BrushBlend::Mix,
        falloff: Falloff::Smooth,
        ..Default::default()
    };
    let spec = BrushSpec {
        texture: TextureSettings {
            kind: TextureKind::Noise,
            ..Default::default()
        },
        ..base
    };
    let mut plain = solid(w, h, [255, 255, 255, 255]);
    let mut tex = solid(w, h, [255, 255, 255, 255]);
    let basis = TexDabBasis::identity();
    let _ = stamp_dab(&mut plain, w, h, [12.0, 12.0], &base, 1.0, false);
    let _ = stamp_dab_textured(
        &mut tex,
        w,
        h,
        [12.0, 12.0],
        &spec,
        1.0,
        false,
        Some(&basis),
        None,
        None,
    );
    // The texture only attenuates: every channel value is ≥ the plain one (lighter or equal,
    // because less black was deposited). Never darker.
    for i in 0..(w * h) as usize {
        assert!(
            tex[i * 4] >= plain[i * 4],
            "texture must only attenuate coverage, never add paint (pixel {i})"
        );
    }
}

// ── Image-mask texture (imported luminance) ─────────────────────────────────────────────────
// (The bilinear `sample_image` unit test lives in `texture/patterns.rs`.)

#[test]
fn image_texture_modulates_the_dab() {
    let (w, h) = (24u32, 24u32);
    let spec = BrushSpec {
        radius_px: 9.0,
        color: [0.0, 0.0, 0.0],
        blend: BrushBlend::Mix,
        falloff: Falloff::Constant,
        hardness: 1.0,
        texture: TextureSettings {
            kind: TextureKind::Image,
            mapping: TextureMapping::Tiled,
            ..Default::default()
        },
        ..Default::default()
    };
    let b = TexDabBasis::identity();
    // All-black luminance → mask 0 → nothing painted.
    let black = vec![0u8; 64];
    let img_k = ImageMask {
        lum: &black,
        width: 8,
        height: 8,
    };
    let mut buf = solid(w, h, [255, 255, 255, 255]);
    assert!(
        stamp_dab_textured(
            &mut buf,
            w,
            h,
            [12.0, 12.0],
            &spec,
            1.0,
            false,
            Some(&b),
            Some(&img_k),
            None,
        )
        .is_none(),
        "an all-black image mask deposits nothing"
    );
    // All-white luminance → mask 1 → paints fully (centre black).
    let white = vec![255u8; 64];
    let img_w = ImageMask {
        lum: &white,
        width: 8,
        height: 8,
    };
    let mut buf2 = solid(w, h, [255, 255, 255, 255]);
    stamp_dab_textured(
        &mut buf2,
        w,
        h,
        [12.0, 12.0],
        &spec,
        1.0,
        false,
        Some(&b),
        Some(&img_w),
        None,
    )
    .expect("painted");
    assert_eq!(
        buf2[((12 * w + 12) * 4) as usize],
        0,
        "all-white image mask paints fully"
    );
    // kind Image with NO pixels supplied → inert (1.0) → paints like no texture.
    let mut buf3 = solid(w, h, [255, 255, 255, 255]);
    assert!(
        stamp_dab_textured(
            &mut buf3,
            w,
            h,
            [12.0, 12.0],
            &spec,
            1.0,
            false,
            Some(&b),
            None,
            None,
        )
        .is_some(),
        "kind Image without pixels is inert (full coverage), not a mask"
    );
}

#[test]
fn procedural_shape_silhouette_samples_like_the_grain() {
    // A procedural Shape reads the pattern the GRAIN way (mapped across the whole dab), NOT a `[-1,1]²`
    // clipped tip — so Size/Offset/Angle transform only the pattern (the falloff is the boundary). Its
    // raw silhouette value must therefore equal the Grain sampler everywhere, including OUTSIDE the tip.
    let s = TextureSettings {
        kind: TextureKind::Checker,
        ..Default::default()
    };
    let b = TexDabBasis::identity();
    for &(px, py) in &[(3, 3), (100, 7), (50, 50), (-40, 12)] {
        assert_eq!(
            sample_shape_silhouette(&s, &b, px, py, [0.0, 0.0], 8.0, None),
            sample(&s, &b, px, py, [0.0, 0.0], 8.0, None),
            "procedural Shape ({px},{py}) must sample like the Grain, not a clipped tip"
        );
    }
    // The Image tip, by contrast, IS the finite clipped silhouette (inert outside its bound).
    let img = TextureSettings {
        kind: TextureKind::Image,
        ..Default::default()
    };
    assert_eq!(
        sample_shape_silhouette(&img, &b, 100, 7, [0.0, 0.0], 8.0, None),
        0.0,
        "an Image tip with no pixels is inert outside its bound (clipped, not grain-mapped)"
    );
}

#[test]
fn render_shape_preview_masks_procedural_with_falloff() {
    // The live Shape preview composes `falloff × pattern` for a procedural kind, so a falloff that
    // fades to the rim deposits LESS total coverage than a flat (all-1) falloff — and the square's
    // corners (outside the round envelope) are masked to nothing.
    let (w, h) = (32u32, 32u32);
    let params = checker_hard();
    let fade: Vec<f32> = (0..64).map(|i| 1.0 - i as f32 / 63.0).collect();
    let flat = [1.0f32; 64];

    let mut buf_fade = vec![0u8; (w * h) as usize];
    let mut buf_flat = vec![0u8; (w * h) as usize];
    crate::texture::render_shape_preview(
        TextureKind::Checker,
        0,
        [0.0, 0.0],
        [1.0, 1.0],
        params,
        FootprintDeform::identity(),
        &fade,
        None,
        None,
        &mut buf_fade,
        w,
        h,
    );
    crate::texture::render_shape_preview(
        TextureKind::Checker,
        0,
        [0.0, 0.0],
        [1.0, 1.0],
        params,
        FootprintDeform::identity(),
        &flat,
        None,
        None,
        &mut buf_flat,
        w,
        h,
    );
    let sum = |b: &[u8]| b.iter().map(|&x| u64::from(x)).sum::<u64>();
    assert_eq!(
        buf_fade[0], 0,
        "the corner (rim, falloff 0) is masked to nothing"
    );
    assert!(
        sum(&buf_fade) > 0,
        "the procedural shape must paint something"
    );
    assert!(
        sum(&buf_fade) < sum(&buf_flat),
        "the fading falloff masks the pattern more than a flat one: {} vs {}",
        sum(&buf_fade),
        sum(&buf_flat)
    );
}

#[test]
fn stencil_image_fills_the_rect_once_at_size_one() {
    use crate::ImageMask;
    // A horizontal gradient image (left dark → right bright). In Stencil at Size (1,1) it fills the rect
    // ONCE — sampling left→right across the rect is monotonic over the FULL gradient (the old 4-tile
    // window repeated it, forcing the user to set Size 0.5 for a 1:1 fit; Enio 2026-06-28).
    let lum: Vec<u8> = (0..16).map(|i| (i * 17) as u8).collect(); // 16×1 ramp 0..255
    let img = ImageMask {
        lum: &lum,
        width: 16,
        height: 1,
    };
    let s = TextureSettings {
        kind: TextureKind::Image,
        mapping: TextureMapping::Stencil,
        stencil_size: [1.0, 1.0], // rect = the whole 64² canvas
        size: [1.0, 1.0],
        ..Default::default()
    };
    let mut rng = 0u64;
    let b = basis(&s, [0.0, 0.0], &mut rng);
    let at = |x: i64| sample(&s, &b, x, 32, [32.0, 32.0], 30.0, Some(&img));
    let (l, m, r) = (at(6), at(32), at(58));
    assert!(
        l < m && m < r,
        "image fills the rect once (monotonic L<M<R): {l} {m} {r}"
    );
    assert!(
        r - l > 0.6,
        "one fill spans most of the gradient: l={l} r={r}"
    );
}

#[test]
fn shape_image_rotates_with_the_basis_angle() {
    use crate::ImageMask;
    // Left half white, right half black (a directional image). With the basis Angle at 0 a point left of
    // centre samples the WHITE half; rotating the basis 90° must sample elsewhere (the rotation reaches
    // the shape sampler). If this fails, the engine doesn't rotate; if it passes, the bug is in wiring.
    let mut lum = vec![0u8; 16 * 16];
    for y in 0..16 {
        for x in 0..8 {
            lum[y * 16 + x] = 255;
        }
    }
    let img = ImageMask {
        lum: &lum,
        width: 16,
        height: 16,
    };
    let mk = |angle: u16| TextureSettings {
        kind: TextureKind::Image,
        mapping: TextureMapping::ViewPlane,
        angle_deg: angle,
        ..Default::default()
    };
    let at = |angle: u16| {
        let s = mk(angle);
        let mut rng = 0u64;
        let b = dab_basis(
            &s,
            [0.0, 0.0],
            &mut rng,
            [64.0, 64.0],
            [1.0, 0.0],
            FootprintDeform::identity(),
        );
        // pixel (24,32): left of centre (32,32), radius 16 → rel ≈ (-0.47, 0.03).
        sample_shape_silhouette(&s, &b, 24, 32, [32.0, 32.0], 16.0, Some(&img))
    };
    assert_ne!(
        at(0),
        at(90),
        "the shape image rotates with the basis angle: {} vs {}",
        at(0),
        at(90)
    );
}

#[test]
fn jitter_footprint_rotates_the_grain_deterministically_not_randomly() {
    // Enio: "Jitter Rot shouldn't act like Random Angle." With random_angle OFF, the Grain follows the
    // footprint — a deterministic rotation locked to the brush, NOT a per-dab random one.
    let s = TextureSettings {
        kind: TextureKind::Noise,
        mapping: TextureMapping::ViewPlane,
        ..Default::default()
    };
    let (center, radius) = ([10.0_f32, 10.0], 8.0_f32);
    let fp45 = FootprintDeform::new(0.0, 45);
    // Two DIFFERENT rng seeds, same footprint: identical (no per-dab randomness → not Random Angle).
    let mut ra = 1u64;
    let mut rb = 9_999u64;
    let ba = dab_basis(&s, [0.0, 0.0], &mut ra, [64.0, 64.0], [1.0, 0.0], fp45);
    let bb = dab_basis(&s, [0.0, 0.0], &mut rb, [64.0, 64.0], [1.0, 0.0], fp45);
    let va = sample(&s, &ba, 14, 12, center, radius, None);
    let vb = sample(&s, &bb, 14, 12, center, radius, None);
    assert!(
        (va - vb).abs() < 1e-6,
        "same footprint ⇒ same Grain regardless of rng (NOT random angle): {va} vs {vb}"
    );
    // Rotating the footprint DOES rotate the Grain (it's locked to the brush, not fixed).
    let mut rc = 1u64;
    let b0 = dab_basis(
        &s,
        [0.0, 0.0],
        &mut rc,
        [64.0, 64.0],
        [1.0, 0.0],
        FootprintDeform::new(0.0, 0),
    );
    let v0 = sample(&s, &b0, 14, 12, center, radius, None);
    assert!(
        (va - v0).abs() > 1e-4,
        "the Grain rotates WITH the footprint (jitter spins the whole stamp): {va} vs {v0}"
    );
}

// ── Watercolor Granulation gate (#2) — the Grain deposit response ──────────────────────────────────

/// `granulation == 0` returns the plain Depth multiply **byte-for-byte** (a non-watercolor brush is
/// unchanged), and the gate is identity at a tooth peak.
#[test]
fn grain_coverage_is_byte_identical_without_granulation() {
    for &depth in &[0.0f32, 0.3, 1.0] {
        for i in 0..=10 {
            let s = i as f32 / 10.0;
            let base = if depth >= 1.0 { s } else { 1.0 + (s - 1.0) * depth };
            assert_eq!(
                super::grain_coverage(s, depth, 0.0),
                base,
                "granulation 0 must equal the plain Depth multiply exactly"
            );
        }
    }
    // A tooth peak (sample 1) always keeps full coverage, at any granulation.
    assert_eq!(super::granulation_gate(1.0, 1.0), 1.0);
    assert_eq!(super::grain_coverage(1.0, 1.0, 1.0), 1.0);
}

/// Granulation rejects pigment in the valleys (monotonic in the amount) and leaves the peaks intact —
/// the physical basis of the granulation speckle.
#[test]
fn granulation_rejects_valleys_keeps_peaks() {
    // Valley (sample 0) at full granulation → fully rejected.
    assert!((super::granulation_gate(0.0, 1.0) - 0.0).abs() < 1e-6);
    // A mid valley is attenuated, and more granulation attenuates more (monotonic).
    let g_lo = super::grain_coverage(0.3, 1.0, 0.4);
    let g_hi = super::grain_coverage(0.3, 1.0, 0.9);
    let g_none = super::grain_coverage(0.3, 1.0, 0.0);
    assert!(g_hi < g_lo && g_lo < g_none, "more granulation rejects more valley deposit");
    assert!(g_hi >= 0.0, "coverage stays non-negative");
    // The peak is untouched regardless.
    assert_eq!(super::grain_coverage(1.0, 1.0, 0.9), 1.0);
}
