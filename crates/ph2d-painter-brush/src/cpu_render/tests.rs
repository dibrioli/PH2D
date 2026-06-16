use super::*;
use crate::stamp::Stamp;

fn empty_canvas(w: u32, h: u32) -> Vec<u8> {
    vec![0u8; (w * h * 4) as usize]
}

fn red_stamp(x: f32, y: f32, size: f32) -> Stamp {
    let mut s = Stamp::zeroed();
    s.position_world = [x, y];
    s.size_px = size;
    // OKLab approximation of opaque red. Doesn't have to be perceptually
    // accurate — we just need a finite, non-trivially-zero color.
    s.color_oklab = [0.6, 0.25, 0.1, 1.0];
    s.opacity = 1.0;
    s.flow = 1.0;
    s.rendering_mode = RenderingMode::UniformGlaze as u32;
    s
}

#[test]
fn mixbox_pigment_paints_blue_plus_yellow_as_green_in_the_live_path() {
    // The live painter renders strokes through THIS cpu_render path. Paint a
    // half-coverage YELLOW stamp (pigment_mode=Mixbox) over a BLUE canvas: the
    // result must be green-dominant AND clearly greener than the Linear blend of
    // the same stroke (which is a muddy grey). Locks the W5 smoke end-to-end.
    let (w, h) = (8u32, 8u32);
    let blue_canvas = || {
        let mut c = vec![0u8; (w * h * 4) as usize];
        for px in c.chunks_exact_mut(4) {
            px.copy_from_slice(&[0, 0, 255, 255]); // opaque sRGB blue
        }
        c
    };
    // Yellow stamp (OKLab of linear-sRGB (1,1,0)), half opacity, full footprint.
    let mut s = Stamp::zeroed();
    s.position_world = [3.5, 3.5];
    s.size_px = 8.0;
    s.color_oklab = [0.968, -0.0713, 0.1984, 1.0];
    s.opacity = 0.5;
    s.flow = 1.0;
    s.rendering_mode = RenderingMode::UniformGlaze as u32;

    let center = ((3 * w + 3) * 4) as usize;
    let mut linear = blue_canvas();
    s.pigment_mode = 0;
    apply_stamps(&mut linear, w, h, &[s]);

    let mut mix = blue_canvas();
    s.pigment_mode = 1;
    apply_stamps(&mut mix, w, h, &[s]);

    let (mr, mg, mb) = (
        mix[center] as i32,
        mix[center + 1] as i32,
        mix[center + 2] as i32,
    );
    let (lr, lg, lb) = (
        linear[center] as i32,
        linear[center + 1] as i32,
        linear[center + 2] as i32,
    );
    // Mixbox: green CLEARLY leads (a real green).
    assert!(
        mg > mr + 15 && mg > mb + 15,
        "Mixbox blue+yellow is green-dominant: mix=({mr},{mg},{mb}) linear=({lr},{lg},{lb})"
    );
    // Linear: a flat grey — green does NOT lead. This is the contrast the smoke shows.
    assert!(
        (lg - lr).abs() < 12 && (lg - lb).abs() < 12,
        "Linear blend is grey (green doesn't lead): linear=({lr},{lg},{lb})"
    );
}

#[test]
fn procedural_grain_varies_coverage_vs_plain() {
    // A grainy brush paints PAPER TEXTURE: coverage varies across the footprint
    // (some pixels less inked → paper shows through), and grain only ever REMOVES
    // coverage vs the plain stamp (Multiply blend). The default brush (no grain)
    // is unaffected — proven by the rest of the suite staying green.
    let (w, h) = (40u32, 40u32);
    let mk = |grain: bool| {
        let mut canvas = vec![0u8; (w * h * 4) as usize];
        let mut s = Stamp::zeroed();
        s.position_world = [19.5, 19.5];
        s.size_px = 36.0;
        s.color_oklab = [0.7, 0.0, 0.0, 1.0];
        s.opacity = 1.0;
        s.flow = 1.0;
        s.rendering_mode = RenderingMode::UniformGlaze as u32;
        if grain {
            s.flags |= crate::stamp::FLAG_GRAIN_PROCEDURAL;
            s.grain_layer = crate::grain_noise::GRAIN_SIMPLEX | (255u32 << 8); // full depth
            s.grain_scale = 0.5; // finer grain → more variation within the footprint
        } else {
            s.grain_layer = u32::MAX;
        }
        apply_stamps(&mut canvas, w, h, &[s]);
        canvas
    };
    let plain = mk(false);
    let grainy = mk(true);
    let alpha = |c: &[u8], x: u32, y: u32| c[((y * w + x) * 4 + 3) as usize];
    let (mut gmin, mut gmax) = (255u8, 0u8);
    let mut grain_never_exceeds_plain = true;
    for y in 14..26 {
        for x in 14..26 {
            let p = alpha(&plain, x, y);
            let g = alpha(&grainy, x, y);
            gmin = gmin.min(g);
            gmax = gmax.max(g);
            if g > p.saturating_add(1) {
                grain_never_exceeds_plain = false;
            }
        }
    }
    assert!(
        grain_never_exceeds_plain,
        "grain only removes coverage (grainy ≤ plain)"
    );
    assert!(
        (gmax as i32 - gmin as i32) > 40,
        "grain produces visible texture variation across the footprint: {gmin}..{gmax}"
    );
}

#[test]
fn alpha_lock_skips_transparent_and_locks_alpha() {
    // T3.7 §2.10: with alpha_lock, paint lands only where the canvas already
    // has alpha, and the alpha is held constant everywhere (only color blends).
    let (w, h) = (16u32, 16u32);
    // Left half opaque white, right half fully transparent.
    let mut canvas = vec![0u8; (w * h * 4) as usize];
    for y in 0..h {
        for x in 0..(w / 2) {
            let i = ((y * w + x) * 4) as usize;
            canvas[i..i + 4].copy_from_slice(&[255, 255, 255, 255]);
        }
    }
    let before = canvas.clone();
    // A large opaque-red stamp over the whole canvas, alpha-locked.
    let s = red_stamp((w / 2) as f32, (h / 2) as f32, (w * 2) as f32);
    apply_stamps_with_options(&mut canvas, w, h, &[s], true);
    // Alpha is locked at EVERY pixel (transparent stays transparent, opaque
    // stays opaque).
    for i in (0..canvas.len()).step_by(4) {
        assert_eq!(
            canvas[i + 3],
            before[i + 3],
            "alpha locked at pixel {}",
            i / 4
        );
    }
    // Transparent (right) half is byte-for-byte untouched (no color leak).
    for y in 0..h {
        for x in (w / 2)..w {
            let i = ((y * w + x) * 4) as usize;
            assert_eq!(
                &canvas[i..i + 4],
                &before[i..i + 4],
                "transparent untouched"
            );
        }
    }
    // Some opaque (left) pixel had its color blended toward the paint.
    let any_color_changed = (0..h).any(|y| {
        (0..(w / 2)).any(|x| {
            let i = ((y * w + x) * 4) as usize;
            canvas[i..i + 3] != before[i..i + 3]
        })
    });
    assert!(
        any_color_changed,
        "alpha-locked paint still blends color on opaque pixels"
    );
}

#[test]
fn single_stamp_writes_pixels() {
    let (w, h) = (32, 32);
    let mut canvas = empty_canvas(w, h);
    let s = red_stamp(16.0, 16.0, 16.0);
    apply_stamps(&mut canvas, w, h, &[s]);
    // Center pixel must have non-zero alpha.
    let idx = (16 * w as usize + 16) * 4;
    assert!(
        canvas[idx + 3] > 0,
        "center pixel must be painted (got alpha {})",
        canvas[idx + 3]
    );
    // Top-left corner of the canvas must be untouched.
    assert_eq!(canvas[0..4], [0, 0, 0, 0]);
}

#[test]
fn painted_byte_is_srgb_encoded_color_not_raw_linear() {
    // GAMMA FIX (2026-05-31): a stamp must write the sRGB-ENCODED color —
    // the value the `Rgba8UnormSrgb` sprite + the sidebar swatch + the
    // replay path show — NOT the raw linear value. Before the fix the
    // compositor wrote `linear * 255`, so a stroke came out dark/
    // desaturated and mismatched its swatch (Enio W2 smoke). A
    // UniformGlaze-over on a TRANSPARENT canvas yields straight RGB =
    // the source color exactly (un-premul cancels coverage alpha), so the
    // painted byte must equal `linear_to_srgb_byte(linear)`.
    let (w, h) = (8, 8);
    let mut canvas = empty_canvas(w, h);
    let mut s = red_stamp(4.0, 4.0, 6.0);
    s.shape_layer = 2; // square_hard → shape α = 1 in the interior.
    apply_stamps(&mut canvas, w, h, &[s]);
    let idx = (4 * w as usize + 4) * 4;
    assert!(canvas[idx + 3] > 0, "center pixel must be painted");
    let linear = oklab_to_linear_srgb(s.color_oklab[0], s.color_oklab[1], s.color_oklab[2]);
    for c in 0..3 {
        let lin_c = linear[c].clamp(0.0, 1.0);
        let want_srgb = linear_to_srgb_byte(lin_c);
        let old_raw_linear = (lin_c * 255.0 + 0.5) as u8;
        assert_eq!(
            canvas[idx + c],
            want_srgb,
            "channel {c}: painted byte {} must equal the sRGB encoding {} (= swatch), \
                 not the raw-linear {}",
            canvas[idx + c],
            want_srgb,
            old_raw_linear,
        );
        // Fix proof: for a mid-range channel, sRGB-encode is strictly
        // brighter than the old raw-linear byte (the visible bug symptom).
        if (0.02..0.98).contains(&lin_c) {
            assert!(
                want_srgb > old_raw_linear,
                "channel {c}: sRGB {want_srgb} must lift mid linear above raw {old_raw_linear}",
            );
        }
    }
}

#[test]
fn nan_position_is_noop() {
    let (w, h) = (8, 8);
    let mut canvas = empty_canvas(w, h);
    let mut s = red_stamp(f32::NAN, 4.0, 8.0);
    s.position_world[0] = f32::NAN;
    apply_stamps(&mut canvas, w, h, &[s]);
    assert!(
        canvas.iter().all(|&b| b == 0),
        "NaN position must produce zero writes"
    );
}

#[test]
fn zero_size_is_noop() {
    let (w, h) = (8, 8);
    let mut canvas = empty_canvas(w, h);
    let s = red_stamp(4.0, 4.0, 0.0);
    apply_stamps(&mut canvas, w, h, &[s]);
    assert!(canvas.iter().all(|&b| b == 0));
}

#[test]
fn off_canvas_clips() {
    let (w, h) = (8, 8);
    let mut canvas = empty_canvas(w, h);
    // Stamp entirely outside the canvas.
    let s = red_stamp(-100.0, -100.0, 8.0);
    apply_stamps(&mut canvas, w, h, &[s]);
    assert!(canvas.iter().all(|&b| b == 0));
}

#[test]
fn two_overlapping_stamps_accumulate_alpha() {
    // Two opaque UniformGlaze stamps on the same pixel should each
    // increase alpha (Porter-Duff "over" reaches 1.0 after a single
    // fully-opaque stamp; the second is a no-op on alpha but proves
    // the dst-read works (would crash / produce NaN otherwise).
    let (w, h) = (8, 8);
    let mut canvas = empty_canvas(w, h);
    let s = red_stamp(4.0, 4.0, 6.0);
    apply_stamps(&mut canvas, w, h, &[s, s]);
    let idx = (4 * w as usize + 4) * 4;
    assert!(
        canvas[idx + 3] > 200,
        "should be near-opaque after 2 stamps"
    );
    // No NaN-induced zero, no garbage. RGB channels should be in
    // the red region (R > G, R > B).
    assert!(
        canvas[idx] >= canvas[idx + 1] && canvas[idx] >= canvas[idx + 2],
        "Red channel should dominate ({}, {}, {})",
        canvas[idx],
        canvas[idx + 1],
        canvas[idx + 2]
    );
}

#[test]
fn determinism_same_stamps_same_canvas() {
    // HR-5 — same input → same output bytes.
    let (w, h) = (32, 32);
    let mut a = empty_canvas(w, h);
    let mut b = empty_canvas(w, h);
    let stamps = vec![
        red_stamp(10.0, 10.0, 12.0),
        red_stamp(20.0, 10.0, 12.0),
        red_stamp(15.0, 20.0, 12.0),
    ];
    apply_stamps(&mut a, w, h, &stamps);
    apply_stamps(&mut b, w, h, &stamps);
    assert_eq!(a, b, "deterministic byte output required");
}

#[test]
fn hover_preview_flag_skips() {
    let (w, h) = (8, 8);
    let mut canvas = empty_canvas(w, h);
    let mut s = red_stamp(4.0, 4.0, 6.0);
    s.flags = crate::stamp::FLAG_HOVER_PREVIEW;
    apply_stamps(&mut canvas, w, h, &[s]);
    assert!(
        canvas.iter().all(|&b| b == 0),
        "hover preview stamps must not write to canvas"
    );
}

// ──────────────────────────────────────────────────────────────────────
// Rendering-mode parity gates (audit T1.5 round 1 A-H2). Algebraic
// invariants the per-mode functions must satisfy — ANY change to the
// CPU OR shader rendering math that breaks one of these is silently
// visible to the user.
// ──────────────────────────────────────────────────────────────────────

fn approx_eq(a: [f32; 4], b: [f32; 4], eps: f32) -> bool {
    (a[0] - b[0]).abs() < eps
        && (a[1] - b[1]).abs() < eps
        && (a[2] - b[2]).abs() < eps
        && (a[3] - b[3]).abs() < eps
}

#[test]
fn rendering_mode_zero_src_is_identity_on_dst() {
    // For every glaze mode: src_premul = [0,0,0,0] → result = dst.
    // (Blending modes can be undefined at result_a < 1e-6; we test
    // their non-trivial cases below.)
    let dst = [0.4, 0.2, 0.1, 0.5];
    for mode in [
        RenderingMode::LightGlaze,
        RenderingMode::UniformGlaze,
        RenderingMode::HeavyGlaze,
    ] {
        let r = apply_rendering_mode(mode, [0.0; 4], dst, 0.0);
        assert!(
            approx_eq(r, dst, 1e-6),
            "mode {:?} with zero src must equal dst; got {:?}",
            mode,
            r
        );
    }
}

#[test]
fn uniform_glaze_full_opacity_replaces_dst() {
    // Porter-Duff over with α_s = 1 → result = src.
    let src = [0.6 * 1.0, 0.3 * 1.0, 0.1 * 1.0, 1.0];
    let dst = [0.2, 0.5, 0.7, 0.5];
    let r = apply_rendering_mode(RenderingMode::UniformGlaze, src, dst, 0.0);
    assert!(approx_eq(r, src, 1e-6));
}

#[test]
fn rendering_mode_premul_invariant_preserved() {
    // For UniformGlaze: result must satisfy `result.rgb ≤ result.a`
    // (premul invariant — straight RGB after un-premul must be in
    // [0,1]). C2-round-3 regression guard.
    let src = [0.4 * 0.7, 0.2 * 0.7, 0.1 * 0.7, 0.7];
    let dst = [0.3 * 0.5, 0.1 * 0.5, 0.05 * 0.5, 0.5];
    let r = apply_rendering_mode(RenderingMode::UniformGlaze, src, dst, 0.0);
    assert!(
        r[0] <= r[3] + 1e-6 && r[1] <= r[3] + 1e-6 && r[2] <= r[3] + 1e-6,
        "premul invariant violated: rgb {:?} > alpha {}",
        &r[..3],
        r[3]
    );
}

#[test]
fn uniform_blending_at_full_alpha_equals_src_color() {
    // alpha_s = 1 → result alpha = 1, result rgb = src_rgb (mix
    // collapses to src). Used to ensure unmul→lerp→re-premul cycle
    // didn't introduce a divisor bug.
    let src_straight_rgb = [0.6, 0.3, 0.1];
    let src = [
        src_straight_rgb[0],
        src_straight_rgb[1],
        src_straight_rgb[2],
        1.0,
    ];
    let dst = [0.2 * 0.4, 0.5 * 0.4, 0.7 * 0.4, 0.4];
    let r = apply_rendering_mode(RenderingMode::UniformBlending, src, dst, 0.0);
    // r is premul; result_a = 1, so r.rgb should equal src_rgb.
    assert!(
        approx_eq(r, src, 1e-5),
        "uniform_blending α_s=1 collapse: got {:?}, expected {:?}",
        r,
        src
    );
}

#[test]
fn intense_glaze_precision_at_small_alpha() {
    // Regression for D-3.M15: src.rgb / aa = src_straight * sqrt(α_s)
    // avoids divisor blow-up at α_s → 0. Test at α_s = 0.01 (small
    // but finite): result should be finite and have alpha = sqrt(0.01) = 0.1.
    let alpha_s: f32 = 0.01;
    let src_straight_rgb = [0.8, 0.4, 0.2];
    let src = [
        src_straight_rgb[0] * alpha_s,
        src_straight_rgb[1] * alpha_s,
        src_straight_rgb[2] * alpha_s,
        alpha_s,
    ];
    let dst = [0.0, 0.0, 0.0, 0.0];
    let r = apply_rendering_mode(RenderingMode::IntenseGlaze, src, dst, 0.0);
    assert!(r[0].is_finite() && r[1].is_finite() && r[2].is_finite());
    assert!((r[3] - alpha_s.sqrt()).abs() < 1e-5);
}

#[test]
fn round_half_up_negative_inputs() {
    // D-2.F3 regression: `floor(x + 0.5) as i32` must produce
    // shader-identical results at negative inputs. Test that the
    // CPU formula matches expected reference for `x ∈ [-1.5, 1.5]`
    // in 0.25 steps.
    let cases: &[(f32, i32)] = &[
        (-1.5, -1),
        (-1.0, -1),
        (-0.51, -1),
        (-0.5, 0),
        (-0.25, 0),
        (0.0, 0),
        (0.25, 0),
        (0.49, 0),
        (0.5, 1),
        (1.0, 1),
        (1.49, 1),
        (1.5, 2),
    ];
    for &(x, expected) in cases {
        let got = (x + 0.5).floor() as i32;
        assert_eq!(
            got, expected,
            "round_half_up({}) = {} (expected {})",
            x, got, expected
        );
    }
}

#[test]
fn cpu_shader_textual_parity_all_six_modes() {
    // Audit T1.5 round 2 F3 + round 6 R6-LN-2: textual parity gate
    // for ALL 6 rendering modes. Catches a future shader rewrite
    // that preserves invariants (alpha collapse / over composition)
    // but drifts the formula structure — CPU + shader would silently
    // disagree on intermediate pixel values.
    //
    // Method: whitespace-normalize STAMP_WGSL, then assert each
    // mode's canonical formula entry-line appears verbatim.
    let shader = crate::stamp_pipeline::STAMP_WGSL;
    let norm = |s: &str| s.split_whitespace().collect::<Vec<_>>().join(" ");
    let shader_norm = norm(shader);

    // ── Glaze modes (Porter-Duff over with mode-specific α curve) ─
    // light_glaze: `src + dst * (1.0 - src.a * 0.6)`.
    assert!(
        shader_norm.contains("return src + dst * (1.0 - src.a * 0.6);"),
        "shader light_glaze formula drifted (R2-F3)"
    );
    // uniform_glaze: Porter-Duff "over".
    assert!(
        shader_norm.contains("return src + dst * (1.0 - src.a);"),
        "shader uniform_glaze formula drifted (R2-F3)"
    );
    // intense_glaze: alpha curve via `aa = sqrt(α_s)` precision-safe.
    assert!(
        shader_norm.contains("let aa = sqrt(alpha_s);"),
        "shader intense_glaze sqrt-alpha curve drifted (R6-LN-2)"
    );
    // heavy_glaze: aggressive over with clamp.
    assert!(
        shader_norm.contains("let r = src + dst * (1.0 - src.a * 0.85);"),
        "shader heavy_glaze formula drifted (R2-F3)"
    );

    // ── Blending modes (continuous mix in straight space + Porter-Duff α) ─
    // Both blending modes share the `result_a = α_s + α_d * (1 - α_s)`
    // Porter-Duff "over" alpha accumulation (premul invariant fix —
    // round 1 D-3.C2).
    assert!(
        shader_norm.contains("let result_a = alpha_s + alpha_d * (1.0 - alpha_s);"),
        "shader blending Porter-Duff alpha-over drifted (R6-LN-2 / C2)"
    );
    // intense_blending wet-mix smudge factor.
    assert!(
        shader_norm.contains("let pull = clamp(wet, 0.0, 1.0);"),
        "shader intense_blending wet-pull factor drifted (R6-LN-2)"
    );
}

#[test]
fn shader_flag_constants_match_rust_bit_values() {
    // Audit T1.5 round 1 A-M7: parity gate between Rust FLAG_* and
    // shader FLAG_* declarations. If a new flag bit is added in
    // Rust without the parallel WGSL line, this test catches it
    // before the shader silently treats the bit as zero.
    //
    // Whitespace-normalized contains() so the test survives
    // alignment churn in the shader source (e.g. tabs vs spaces,
    // extra padding to align `=` columns).
    let shader = crate::stamp_pipeline::STAMP_WGSL;
    let normalize = |s: &str| s.split_whitespace().collect::<Vec<_>>().join(" ");
    let shader_norm = normalize(shader);
    let pairs: &[(&str, u32)] = &[
        (
            "FLAG_SHAPE_FLIP_X: u32 = 1u",
            crate::stamp::FLAG_SHAPE_FLIP_X,
        ),
        (
            "FLAG_SHAPE_FLIP_Y: u32 = 2u",
            crate::stamp::FLAG_SHAPE_FLIP_Y,
        ),
        (
            "FLAG_GRAIN_BEHAVIOR_MOVING: u32 = 4u",
            crate::stamp::FLAG_GRAIN_BEHAVIOR_MOVING,
        ),
        ("FLAG_BURNT_EDGES: u32 = 8u", crate::stamp::FLAG_BURNT_EDGES),
        ("FLAG_WET_EDGES: u32 = 16u", crate::stamp::FLAG_WET_EDGES),
        (
            "FLAG_LUMINANCE_BLENDING: u32 = 32u",
            crate::stamp::FLAG_LUMINANCE_BLENDING,
        ),
        (
            "FLAG_GRAIN_PROCEDURAL: u32 = 64u",
            crate::stamp::FLAG_GRAIN_PROCEDURAL,
        ),
        (
            "FLAG_HOVER_PREVIEW: u32 = 256u",
            crate::stamp::FLAG_HOVER_PREVIEW,
        ),
        (
            "FLAG_PREDICTED_SAMPLE: u32 = 512u",
            crate::stamp::FLAG_PREDICTED_SAMPLE,
        ),
    ];
    for (needle, _rust_value) in pairs {
        assert!(
            shader_norm.contains(needle),
            "shader FLAG declaration missing or drifted: `{}`",
            needle
        );
    }
    // Bit values themselves (paranoia — also covered by stamp::tests::
    // flag_constants_are_distinct_bits).
    assert_eq!(crate::stamp::FLAG_HOVER_PREVIEW, 256);
    assert_eq!(crate::stamp::FLAG_PREDICTED_SAMPLE, 512);
}

// ──────────────────────────────────────────────────────────────────────
// T1.6 extension tests — shape dispatch / rotation / flip + shader
// textual parity for the three procedural shape kernels.
// ──────────────────────────────────────────────────────────────────────

#[test]
fn cpu_shader_shape_kernels_textual_parity() {
    // Audit T1.6 — extended after lens O-1: shape kernels must stay
    // byte-equivalent CPU ↔ shader OR a future shader rewrite that
    // preserves an analytic curve but drifts the formula structure
    // would silently diverge. This gate covers EACH kernel's
    // distance line + edge-band line + smoothstep line + return
    // line. The CPU side uses scalar `(dx*dx+dy*dy).sqrt()` (not
    // WGSL `length()`) — audit T1.6 O-3 — and this test pins the
    // shader's scalar form to match.
    //
    // **Limitation (audit O-2):** this is a shader↔hardcoded-string
    // parity, NOT a CPU↔shader runtime parity. A future CPU edit
    // that drifts from these strings stays green. A real CPU↔shader
    // numerical parity test (run the GPU dispatch headless + compare
    // pixels) requires a wgpu device in CI and is tracked as a
    // T-numerical-parity follow-up (W2+).
    let shader = crate::stamp_pipeline::STAMP_WGSL;
    let norm = |s: &str| s.split_whitespace().collect::<Vec<_>>().join(" ");
    let shader_norm = norm(shader);

    // round_hard — pixel-relative analytic coverage (clamp of the signed
    // distance / aa). dx/dy scalar form (NOT WGSL `length()`).
    assert!(
        shader_norm.contains("let dx = uv.x - 0.5;"),
        "shader round_hard dx scalar drifted"
    );
    assert!(
        shader_norm.contains("let dy = uv.y - 0.5;"),
        "shader round_hard dy scalar drifted"
    );
    assert!(
        shader_norm.contains("let r = sqrt(dx * dx + dy * dy);"),
        "shader round_hard scalar sqrt-distance drifted (O-3 regression)"
    );
    assert!(
        shader_norm.contains("return clamp((0.5 - r) / aa + 0.5, 0.0, 1.0);"),
        "shader round_hard pixel-relative coverage drifted"
    );

    // round_soft — full body with explicit d² >= 1 early-out (unchanged profile).
    assert!(
        shader_norm.contains("let d_sq = (dx * dx + dy * dy) / 0.25;"),
        "shader round_soft d² normalization drifted"
    );
    assert!(
        shader_norm.contains("if d_sq >= 1.0 {"),
        "shader round_soft d² >= 1 early-out drifted (O-1 extension)"
    );
    assert!(
        shader_norm.contains("let one_minus_d_sq = 1.0 - d_sq;"),
        "shader round_soft (1-d²) intermediate drifted (O-1 extension)"
    );
    assert!(
        shader_norm.contains("return one_minus_d_sq * one_minus_d_sq;"),
        "shader round_soft (1-d²)² return drifted"
    );

    // square_hard — Chebyshev distance + pixel-relative coverage.
    assert!(
        shader_norm.contains("let d = max(dx, dy);"),
        "shader square_hard Chebyshev distance drifted"
    );
    assert!(
        shader_norm.contains("return clamp((0.5 - d) / aa + 0.5, 0.0, 1.0);"),
        "shader square_hard pixel-relative coverage drifted"
    );

    // oval_hard — Van Verth gradient-normalized coverage (2:1 oblong).
    assert!(
        shader_norm.contains("let ex = (uv.x - 0.5) / 0.5;"),
        "shader oval_hard ex stretched drifted"
    );
    assert!(
        shader_norm.contains("let ey = (uv.y - 0.5) / 0.25;"),
        "shader oval_hard ey stretched drifted"
    );
    assert!(
        shader_norm.contains("let f = ex * ex + ey * ey - 1.0;"),
        "shader oval_hard implicit field drifted"
    );
    assert!(
        shader_norm.contains("let grad = max(sqrt(gx * gx + gy * gy), 1e-6);"),
        "shader oval_hard gradient magnitude drifted"
    );
    assert!(
        shader_norm.contains("return clamp(0.5 - dist / aa, 0.0, 1.0);"),
        "shader oval_hard gradient-normalized coverage drifted"
    );

    // shape_alpha_for_slot dispatch (now threads `aa`) + 0 → round_hard /
    // 1 → round_soft / 2 → square_hard / 3 → oval_hard / default → round_hard.
    assert!(
        shader_norm.contains("case 0u: { return round_hard_shape(uv, aa); }"),
        "shader shape dispatch slot 0 drifted"
    );
    assert!(
        shader_norm.contains("case 1u: { return round_soft_shape(uv, aa); }"),
        "shader shape dispatch slot 1 drifted"
    );
    assert!(
        shader_norm.contains("case 2u: { return square_hard_shape(uv, aa); }"),
        "shader shape dispatch slot 2 drifted"
    );
    assert!(
        shader_norm.contains("case 3u: { return oval_hard_shape(uv, aa); }"),
        "shader shape dispatch slot 3 drifted (V-2 R4 addition)"
    );
    assert!(
        shader_norm.contains("default: { return round_hard_shape(uv, aa); }"),
        "shader shape dispatch fallback drifted"
    );
}

#[test]
fn cpu_shader_rotation_pipeline_textual_parity() {
    // Audit T1.6: the rotation/flip uv transform is the most error-prone
    // part of the shader edit — gate it textually so a future rewrite
    // that breaks parity (e.g. rotates by +rotation_rad instead of
    // -rotation_rad) doesn't slip through.
    let shader = crate::stamp_pipeline::STAMP_WGSL;
    let norm = |s: &str| s.split_whitespace().collect::<Vec<_>>().join(" ");
    let shader_norm = norm(shader);

    // Sub-pixel dab: aa + uv scale derive from `1/size_px` (NOT `footprint`),
    // bit-identical to `cpu_render::apply_one_stamp`.
    assert!(
        shader_norm.contains("let inv_size = 1.0 / stamp.size_px;"),
        "shader inv_size (sub-pixel scale) drifted"
    );
    assert!(
        shader_norm.contains("let aa = SHAPE_AA_PX * inv_size;"),
        "shader aa must derive from size_px (sub-pixel parity), not footprint"
    );
    // Centered uv = each pixel centre's offset to the dab's TRUE fractional
    // centre (sub-pixel anti-wobble) — NOT a `(i+0.5)/footprint` grid.
    assert!(
        shader_norm.contains(
            "var uv_centered = vec2<f32>( (f32(world_x) - stamp.position_world_x) * inv_size,"
        ),
        "shader sub-pixel uv centering drifted"
    );
    // Flip bits applied BEFORE rotation (audit T1.6).
    assert!(
        shader_norm.contains("if (stamp.flags & FLAG_SHAPE_FLIP_X) != 0u {"),
        "shader flip_x check drifted"
    );
    assert!(
        shader_norm.contains("uv_centered.x = -uv_centered.x;"),
        "shader flip_x negate drifted"
    );
    assert!(
        shader_norm.contains("if (stamp.flags & FLAG_SHAPE_FLIP_Y) != 0u {"),
        "shader flip_y check drifted"
    );
    // Rotation by -rotation_rad via trig identity (audit T1.6 U-9):
    // `cos(-x) = cos(x)`, `sin(-x) = -sin(x)` — avoids feeding a
    // negated argument to the backend intrinsic + eliminates ±0
    // sign-bit drift at rotation_rad = 0.
    assert!(
        shader_norm.contains("let cos_r = cos(stamp.rotation_rad);"),
        "shader rotation matrix uses cos(stamp.rotation_rad) per U-9 identity"
    );
    assert!(
        shader_norm.contains("let sin_r = -sin(stamp.rotation_rad);"),
        "shader rotation matrix uses -sin(stamp.rotation_rad) per U-9 identity"
    );
}

fn red_stamp_with_slot(x: f32, y: f32, size: f32, slot: u32) -> Stamp {
    let mut s = Stamp::zeroed();
    s.position_world = [x, y];
    s.size_px = size;
    s.color_oklab = [0.6, 0.25, 0.1, 1.0];
    s.opacity = 1.0;
    s.flow = 1.0;
    s.rendering_mode = RenderingMode::UniformGlaze as u32;
    s.shape_layer = slot;
    s
}

#[test]
fn shape_slot_dispatch_round_soft_writes_pixels() {
    let (w, h) = (32, 32);
    let mut canvas = empty_canvas(w, h);
    let s = red_stamp_with_slot(16.0, 16.0, 16.0, crate::library::ROUND_SOFT_SLOT);
    apply_stamps(&mut canvas, w, h, &[s]);
    let center_idx = (16 * w as usize + 16) * 4;
    assert!(
        canvas[center_idx + 3] > 0,
        "round_soft stamp must paint center pixel"
    );
}

#[test]
fn shape_slot_dispatch_square_hard_writes_corner_pixels() {
    // Distinguishing feature of square_hard vs round_*: corners INSIDE
    // the bounding box have non-zero alpha (a circle has empty corners).
    let (w, h) = (32, 32);
    let mut canvas_sq = empty_canvas(w, h);
    let mut canvas_rh = empty_canvas(w, h);
    let stamp_size = 20.0;
    apply_stamps(
        &mut canvas_sq,
        w,
        h,
        &[red_stamp_with_slot(
            16.0,
            16.0,
            stamp_size,
            crate::library::SQUARE_HARD_SLOT,
        )],
    );
    apply_stamps(
        &mut canvas_rh,
        w,
        h,
        &[red_stamp_with_slot(
            16.0,
            16.0,
            stamp_size,
            crate::library::ROUND_HARD_SLOT,
        )],
    );
    // Inspect bounding-box corner (around pixel 7,7 — top-left of the
    // 20px footprint centered at 16,16). Square fills; round leaves it 0.
    let corner_x = 7;
    let corner_y = 7;
    let idx = (corner_y * w as usize + corner_x) * 4;
    let sq_alpha = canvas_sq[idx + 3];
    let rh_alpha = canvas_rh[idx + 3];
    assert!(
        sq_alpha > rh_alpha,
        "square_hard corner alpha ({sq_alpha}) must exceed round_hard corner alpha ({rh_alpha})"
    );
}

#[test]
fn rotation_pipeline_round_hard_unaffected() {
    // round_hard is radial-symmetric: rotation must not change pixels.
    // Render at rotation=0 and rotation=π/3, compare byte-equal.
    let (w, h) = (32, 32);
    let mut a = empty_canvas(w, h);
    let mut b = empty_canvas(w, h);
    let mut s0 = red_stamp_with_slot(16.0, 16.0, 12.0, crate::library::ROUND_HARD_SLOT);
    let mut s1 = s0;
    s0.rotation_rad = 0.0;
    s1.rotation_rad = core::f32::consts::FRAC_PI_3;
    apply_stamps(&mut a, w, h, &[s0]);
    apply_stamps(&mut b, w, h, &[s1]);
    assert_eq!(a, b, "round_hard pixels must be invariant under rotation");
}

// FOLLOW-UP-W6: pixel-level flip gate that PROVES flip changes
// output (rather than being a no-op on symmetric shapes) requires
// an asymmetric shape kernel. Today's T1.6 shapes (round_hard /
// round_soft / square_hard) are doubly-symmetric → flip is
// visually a no-op by construction. When the first asymmetric
// shape ships in W6+ (`flat_chisel`, `bristle_spread`,
// `splatter_spread` per spec §1.6.7), add a parallel test:
//   `flip_x_mirrors_asymmetric_shape_pixels` — apply same stamp
//   with/without FLAG_SHAPE_FLIP_X on an asymmetric shape; assert
//   the canvas pixels differ in a mirror-image pattern.
// The current test (`flip_preserves_output_for_symmetric_shapes`)
// is TAUTOLOGICAL for T1.6 shapes but remains useful as a
// regression sentinel for the symmetric set.
// Audit T1.6 R-8 + W-2 + W-3 — search grep "FOLLOW-UP-W6" to find.
#[test]
fn flip_preserves_output_for_symmetric_shapes() {
    // All three T1.6 shape kernels (round_hard, round_soft, square_hard)
    // are symmetric in **both** x and y around the centerpoint, so
    // applying flip_x and/or flip_y MUST be a no-op visually (the
    // sample-coord set is permuted but each pixel reads the same
    // alpha). This test pins that invariant — if a future shape with
    // axis-asymmetry is added (W6+ flat_chisel etc.), the test gets
    // SHAPE_SLOT updated and the asymmetric shape gets its own gate.
    //
    // **Follow-up:** pixel-level flip gate that PROVES the flip
    // changes output (rather than no-op) requires an asymmetric shape
    // (W6+). T1.6 ships the flip code path + the FLAG bits + the
    // scheduler wiring; the asymmetric-shape acceptance test lands
    // when the first asymmetric shape arrives.
    let (w, h) = (32, 32);
    for slot in [
        crate::library::ROUND_HARD_SLOT,
        crate::library::ROUND_SOFT_SLOT,
        crate::library::SQUARE_HARD_SLOT,
    ] {
        for rotation in [0.0_f32, core::f32::consts::FRAC_PI_4, 0.7] {
            let mut a = empty_canvas(w, h);
            let mut b = empty_canvas(w, h);
            let mut s0 = red_stamp_with_slot(16.0, 16.0, 12.0, slot);
            s0.rotation_rad = rotation;
            let mut s1 = s0;
            s1.flags |= crate::stamp::FLAG_SHAPE_FLIP_X | crate::stamp::FLAG_SHAPE_FLIP_Y;
            apply_stamps(&mut a, w, h, &[s0]);
            apply_stamps(&mut b, w, h, &[s1]);
            // Flip of a doubly-symmetric shape is VISUALLY invariant. Post sub-pixel
            // sampling (the dab is centred on its true position, not the footprint
            // centre — more correct), an integer position with an even footprint
            // samples a slightly asymmetric uv, so under rotation the AA edge can
            // quantize ±1 between the flipped/unflipped pass. Allow ≤1/255 per
            // channel (imperceptible) instead of bit-exact equality.
            let max_diff = a
                .iter()
                .zip(b.iter())
                .map(|(x, y)| (*x as i32 - *y as i32).unsigned_abs())
                .max()
                .unwrap_or(0);
            assert!(
                max_diff <= 1,
                "slot {slot} + rotation {rotation} must be flip_x|flip_y invariant \
                 (≤1/255); got max diff {max_diff}"
            );
        }
    }
}

#[test]
fn flip_flag_changes_internal_sample_coord_for_offcenter_pixel() {
    // Unit-level proof that the flip code path **does** alter the
    // sample uv. We can't observe shape sampling directly without
    // exposing internals, so we use a shape-agnostic invariant: when
    // `rotation_rad != 0` and `flip_x` is set, the position of the
    // SAMPLE point for off-center pixel (px=0, py=0) differs from
    // the no-flip case. Computed via the documented analytic formula
    // (matches the inner loop of `apply_one_stamp` exactly).
    let footprint = 16.0_f32;
    let center = (footprint - 1.0) * 0.5;
    // pixel (0, 0)
    let u_axis = 0.5 / footprint;
    let v_axis = 0.5 / footprint;
    let uc = u_axis - 0.5;
    let vc = v_axis - 0.5;
    let rotation = 0.7_f32;
    let cos_r = (-rotation).cos();
    let sin_r = (-rotation).sin();

    // Without flip
    let ur_noflip = uc * cos_r - vc * sin_r;
    let vr_noflip = uc * sin_r + vc * cos_r;
    // With flip_x
    let uc_flipped = -uc;
    let ur_flip = uc_flipped * cos_r - vc * sin_r;
    let vr_flip = uc_flipped * sin_r + vc * cos_r;

    assert!(
        (ur_noflip - ur_flip).abs() > 1e-3 || (vr_noflip - vr_flip).abs() > 1e-3,
        "flip_x must change the sample coord for off-center pixel under rotation; \
             noflip=({ur_noflip}, {vr_noflip}) flip=({ur_flip}, {vr_flip})"
    );
    let _ = center; // silence "unused" warning if test moves
}

#[test]
fn cpu_deterministic_under_rotation() {
    // HR-5: same rotation + same stamp → byte-identical canvas.
    let (w, h) = (32, 32);
    let mut a = empty_canvas(w, h);
    let mut b = empty_canvas(w, h);
    let mut s = red_stamp_with_slot(16.0, 16.0, 14.0, crate::library::SQUARE_HARD_SLOT);
    s.rotation_rad = 0.7;
    apply_stamps(&mut a, w, h, &[s]);
    apply_stamps(&mut b, w, h, &[s]);
    assert_eq!(a, b, "rotated stamp must be bit-deterministic");
}

#[test]
fn unknown_shape_slot_falls_back_to_round_hard() {
    // CPU path falls back to round_hard for unknown slots, matching
    // shader `default:` arm. Compare against explicit slot 0.
    let (w, h) = (32, 32);
    let mut a = empty_canvas(w, h);
    let mut b = empty_canvas(w, h);
    let s_unknown = red_stamp_with_slot(16.0, 16.0, 12.0, 99);
    let s_round = red_stamp_with_slot(16.0, 16.0, 12.0, 0);
    apply_stamps(&mut a, w, h, &[s_unknown]);
    apply_stamps(&mut b, w, h, &[s_round]);
    assert_eq!(a, b, "unknown slot must fall back to round_hard");
}

#[test]
fn falloff_taper_fades_the_wash_stroke() {
    // The wash path (the DEFAULT brush) must honor the per-stamp falloff taper
    // (`stamp.opacity`): a tapered dab deposits less coverage than a full dab, so
    // the painted alpha where the stroke has faded is lower. Regression guard —
    // the wash path previously dropped `stamp.opacity`, so falloff did nothing.
    let (w, h) = (64u32, 16u32);
    let mut canvas = empty_canvas(w, h);
    let backdrop = empty_canvas(w, h); // transparent pre-stroke backdrop
    let mut coverage = vec![0.0f32; (w * h) as usize];
    let mut wc = vec![[0.0f32; 3]; (w * h) as usize];

    // Two isolated dabs: full opacity (left) vs heavily tapered (right).
    let mut full = red_stamp(10.0, 8.0, 8.0);
    full.opacity = 1.0;
    let mut tapered = red_stamp(50.0, 8.0, 8.0);
    tapered.opacity = 0.2;

    apply_stamps_wash(
        &mut canvas,
        &backdrop,
        &mut coverage,
        &mut wc,
        w,
        h,
        &[full, tapered],
        1.0,   // opacity_cap (brush opacity)
        false, // pigment
        false, // alpha_lock
        0.0,   // paper_grain (off in this test)
        None,  // wet (legacy wash path)
    );

    let (mut left_max, mut right_max) = (0u8, 0u8);
    for y in 0..h {
        for x in 0..w {
            let a = canvas[((y * w + x) * 4 + 3) as usize];
            if x < 32 {
                left_max = left_max.max(a);
            } else {
                right_max = right_max.max(a);
            }
        }
    }
    assert!(left_max > 0, "the full dab must paint visible coverage");
    assert!(
        right_max < left_max,
        "falloff taper must fade the wash stroke: tapered max alpha {right_max} \
         should be below the full dab's {left_max}"
    );
}
#[test]
fn wash_color_accumulation_blends_overlapping_dab_colors() {
    // Color Dynamics smoothness: when an earlier dab has only PARTIALLY covered a
    // pixel (its feather, or partial flow — the real moving-stroke case), a later
    // dab of a DIFFERENT colour must BLEND with it (coverage-weighted average),
    // not overwrite it (the old "discrete coloured discs" / last-wins behaviour).
    // Partial flow (0.5) keeps each dab sub-full so the colours can mix.
    let (w, h) = (40u32, 40u32);
    let backdrop = vec![0u8; (w * h * 4) as usize];
    let paint = |stamps: &[Stamp]| {
        let mut canvas = backdrop.clone();
        let mut cov = vec![0.0f32; (w * h) as usize];
        let mut wc = vec![[0.0f32; 3]; (w * h) as usize];
        apply_stamps_wash(
            &mut canvas,
            &backdrop,
            &mut cov,
            &mut wc,
            w,
            h,
            stamps,
            1.0,
            false,
            false,
            0.0,
            None,
        );
        canvas
    };
    let mut a = red_stamp(20.0, 20.0, 24.0);
    a.color_oklab = [0.6, 0.22, 0.13, 1.0]; // reddish
    a.flow = 0.5;
    let mut b = red_stamp(20.0, 20.0, 24.0);
    b.color_oklab = [0.7, -0.18, 0.12, 1.0]; // greenish
    b.flow = 0.5;
    let i = ((20 * w + 20) * 4) as usize;
    let rgb = |c: &[u8]| (c[i], c[i + 1], c[i + 2]);
    let both = rgb(&paint(&[a, b]));
    let b_only = rgb(&paint(&[b]));
    // Under last-wins the earlier (red) dab would leave no trace → both == b_only.
    // Under the blend the red persists → the centre colour differs from b alone.
    assert_ne!(
        both, b_only,
        "the earlier dab's colour must persist in the blend, not be overwritten (last-wins)"
    );
    // And the blend keeps red present (the reddish dab contributed).
    assert!(
        both.0 > b_only.0,
        "blended centre must be redder than b-alone: blend={both:?} b_only={b_only:?}"
    );
}

#[test]
fn wash_later_opaque_dab_is_on_top_not_inverted_z() {
    // Z-ORDER: two FULLY opaque overlapping dabs in one stroke — red THEN blue —
    // at the same spot must read BLUE (the later dab on top). The old coverage-
    // weighted average let the EARLIER (red) dab dominate as coverage saturated
    // (in fact the later dab became invisible once the earlier one fully covered),
    // so the earlier-painted colour appeared on top — inverted z.
    let (w, h) = (40u32, 40u32);
    let backdrop = vec![255u8; (w * h * 4) as usize]; // white paper
    let mut canvas = backdrop.clone();
    let mut cov = vec![0.0f32; (w * h) as usize];
    let mut wc = vec![[0.0f32; 3]; (w * h) as usize];
    let a = red_stamp(20.0, 20.0, 24.0); // reddish, full flow + opacity
    let mut b = red_stamp(20.0, 20.0, 24.0);
    b.color_oklab = [0.45, -0.03, -0.31, 1.0]; // bluish
    apply_stamps_wash(
        &mut canvas,
        &backdrop,
        &mut cov,
        &mut wc,
        w,
        h,
        &[a, b],
        1.0,
        false,
        false,
        0.0,
        None,
    );
    let i = ((20 * w + 20) * 4) as usize;
    let (r, g, bl) = (canvas[i] as i32, canvas[i + 1] as i32, canvas[i + 2] as i32);
    assert!(
        bl > r + 20,
        "later (blue) dab must be on top, not the earlier red: rgb=({r},{g},{bl})"
    );
}

#[test]
fn wash_honors_rendering_mode_option_a() {
    // Option (a): the wash now composites via the brush's rendering mode. Two
    // different modes on the same wash stroke must produce DIFFERENT output, in
    // BOTH the Linear (full mode) and Pigment (glaze coverage curve) paths.
    let (w, h) = (8u32, 8u32);
    let backdrop = vec![200u8; (w * h * 4) as usize]; // light grey
    let paint = |mode: u32, pigment: bool| -> (u8, u8, u8) {
        let mut canvas = backdrop.clone();
        let mut cov = vec![0.0f32; (w * h) as usize];
        let mut wc = vec![[0.0f32; 3]; (w * h) as usize];
        let mut s = red_stamp(4.0, 4.0, 8.0);
        s.flow = 0.5; // semi-transparent so the modes' attenuation shows
        s.rendering_mode = mode;
        apply_stamps_wash(
            &mut canvas,
            &backdrop,
            &mut cov,
            &mut wc,
            w,
            h,
            &[s],
            0.6,
            pigment,
            false,
            0.0,
            None,
        );
        let i = ((4 * w + 4) * 4) as usize;
        (canvas[i], canvas[i + 1], canvas[i + 2])
    };
    // Linear: glaze modes are fully distinct via apply_rendering_mode.
    assert_ne!(
        paint(0, false),
        paint(1, false),
        "Linear wash: LightGlaze must differ from UniformGlaze"
    );
    assert_ne!(
        paint(3, false),
        paint(1, false),
        "Linear wash: HeavyGlaze must differ from UniformGlaze"
    );
    // Pigment: the glaze coverage curve makes Light/Heavy differ from Uniform
    // (UniformGlaze stays the K-M identity → no regression).
    assert_ne!(
        paint(0, true),
        paint(1, true),
        "Pigment wash: LightGlaze must differ from UniformGlaze"
    );
    assert_ne!(
        paint(3, true),
        paint(1, true),
        "Pigment wash: HeavyGlaze must differ from UniformGlaze"
    );
}

// ───────────────────────── W7 Wet Mix (mixer-brush) ─────────────────────────
// Design: docs/Novo Painter/04_design_W7_wet_mix.md. Reservoir pickup/deposit on
// the live wash path; `wet = None` is the legacy default brush.

fn solid_canvas(w: u32, h: u32, rgba: [u8; 4]) -> Vec<u8> {
    let mut c = vec![0u8; (w * h * 4) as usize];
    for px in c.chunks_exact_mut(4) {
        px.copy_from_slice(&rgba);
    }
    c
}

fn clamp3(c: [f32; 3]) -> [f32; 3] {
    [c[0].clamp(0.0, 1.0), c[1].clamp(0.0, 1.0), c[2].clamp(0.0, 1.0)]
}

/// Neutral Wet Mix (full charge, full attack, no pull, no dilution) on a SINGLE
/// dab must reproduce the legacy wash (`wet = None`) byte-for-byte — the mixer
/// path reduces to the default brush when nothing is asked of it. (Depletion only
/// affects the NEXT dab, so a single dab is the clean equivalence point.)
#[test]
fn wet_neutral_single_dab_equals_legacy() {
    let (w, h) = (24u32, 24u32);
    let backdrop = solid_canvas(w, h, [255, 255, 255, 255]);
    let s = red_stamp(12.0, 12.0, 10.0);
    let seed = clamp3(oklab_to_linear_srgb(0.6, 0.25, 0.1)); // == the stamp's rgb_clamped
    let run = |wet: bool| {
        let mut canvas = backdrop.clone();
        let mut cov = vec![0.0f32; (w * h) as usize];
        let mut wc = vec![[0.0f32; 3]; (w * h) as usize];
        let mut st = WetState { color: seed, load: 1.0 };
        let cfg = WetMixConfig { dilution: 0.0, attack: 1.0, pull: 0.0, wetness_jitter: 0.0 };
        let wetarg = if wet { Some((&mut st, cfg)) } else { None };
        apply_stamps_wash(
            &mut canvas, &backdrop, &mut cov, &mut wc, w, h, &[s], 1.0, false, false, 0.0, wetarg,
        );
        canvas
    };
    assert_eq!(
        run(true),
        run(false),
        "neutral Wet Mix on a single dab must equal the legacy wash byte-for-byte"
    );
}

/// Charge depletes along the trail: with no pull/dilution and a partial Charge,
/// a straight line of dabs must be MORE opaque at the start than at the fading end.
#[test]
fn wet_charge_depletes_along_trail() {
    let (w, h) = (96u32, 8u32);
    let backdrop = empty_canvas(w, h); // transparent pre-stroke
    let mut canvas = empty_canvas(w, h);
    let mut cov = vec![0.0f32; (w * h) as usize];
    let mut wc = vec![[0.0f32; 3]; (w * h) as usize];
    let mut st = WetState {
        color: clamp3(oklab_to_linear_srgb(0.6, 0.25, 0.1)),
        load: 0.6,
    };
    let cfg = WetMixConfig { dilution: 0.0, attack: 1.0, pull: 0.0, wetness_jitter: 0.0 };
    let stamps: Vec<Stamp> = (6..90).step_by(3).map(|x| red_stamp(x as f32, 4.0, 5.0)).collect();
    apply_stamps_wash(
        &mut canvas, &backdrop, &mut cov, &mut wc, w, h, &stamps, 1.0, false, false, 0.0,
        Some((&mut st, cfg)),
    );
    let alpha = |x: u32| canvas[((4 * w + x) * 4 + 3) as usize];
    assert!(
        alpha(8) > alpha(86),
        "charge depletes → trail fades: start alpha {} should exceed end alpha {}",
        alpha(8),
        alpha(86)
    );
    assert!(st.load < 0.6, "load must have depleted (was 0.6, now {})", st.load);
}

/// Pull picks up the canvas colour: a BLUE brush over a RED backdrop deposits red
/// with `pull = 1`, blue with `pull = 0`.
#[test]
fn wet_pull_picks_up_backdrop_color() {
    let (w, h) = (16u32, 16u32);
    let backdrop = solid_canvas(w, h, [220, 20, 20, 255]); // opaque red canvas
    let blue = {
        let mut s = red_stamp(8.0, 8.0, 10.0);
        s.color_oklab = [0.55, -0.05, -0.18, 1.0]; // bluish
        s
    };
    let seed_blue = clamp3(oklab_to_linear_srgb(0.55, -0.05, -0.18));
    let run = |pull: f32| {
        let mut canvas = backdrop.clone();
        let mut cov = vec![0.0f32; (w * h) as usize];
        let mut wc = vec![[0.0f32; 3]; (w * h) as usize];
        let mut st = WetState { color: seed_blue, load: 1.0 };
        let cfg = WetMixConfig { dilution: 0.0, attack: 1.0, pull, wetness_jitter: 0.0 };
        apply_stamps_wash(
            &mut canvas, &backdrop, &mut cov, &mut wc, w, h, &[blue], 1.0, false, false, 0.0,
            Some((&mut st, cfg)),
        );
        let i = ((8 * w + 8) * 4) as usize;
        (canvas[i] as i32, canvas[i + 2] as i32) // (r, b)
    };
    let (r_pull, b_pull) = run(1.0);
    let (r_none, b_none) = run(0.0);
    assert!(
        r_pull > r_none,
        "pull picks up the red backdrop: r(pull)={r_pull} > r(no pull)={r_none}"
    );
    assert!(
        b_pull < b_none,
        "pull replaces the brush blue with red: b(pull)={b_pull} < b(no pull)={b_none}"
    );
}

/// Dilution thins the deposit: more water → less coverage on the same dab.
#[test]
fn wet_dilution_reduces_coverage() {
    let (w, h) = (24u32, 24u32);
    let backdrop = empty_canvas(w, h);
    let seed = clamp3(oklab_to_linear_srgb(0.6, 0.25, 0.1));
    let run = |dilution: f32| {
        let mut canvas = empty_canvas(w, h);
        let mut cov = vec![0.0f32; (w * h) as usize];
        let mut wc = vec![[0.0f32; 3]; (w * h) as usize];
        let mut st = WetState { color: seed, load: 1.0 };
        let cfg = WetMixConfig { dilution, attack: 1.0, pull: 0.0, wetness_jitter: 0.0 };
        apply_stamps_wash(
            &mut canvas, &backdrop, &mut cov, &mut wc, w, h, &[red_stamp(12.0, 12.0, 10.0)], 1.0,
            false, false, 0.0, Some((&mut st, cfg)),
        );
        canvas[((12 * w + 12) * 4 + 3) as usize] // centre alpha
    };
    assert!(
        run(0.0) > run(0.8),
        "more dilution → less coverage: dry alpha {} should exceed diluted alpha {}",
        run(0.0),
        run(0.8)
    );
}
