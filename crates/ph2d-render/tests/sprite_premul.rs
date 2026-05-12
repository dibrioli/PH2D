//! Sprite shader: premultiplied-alpha output invariants.
//!
//! Mirrors the fragment shader in
//! [`src/shaders/sprite.wgsl`](../src/shaders/sprite.wgsl) as a pure
//! Rust function so the math can be unit-tested without a GPU. The
//! tests pin two contracts:
//!
//! 1. The fragment emits **premultiplied** RGBA — `rgb ≤ a` for every
//!    in-gamut input (this is the invariant
//!    [`pipeline.rs`](../src/pipeline.rs)'s
//!    `BlendState::PREMULTIPLIED_ALPHA_BLENDING` assumes).
//! 2. The resulting blend (`out = src + dst * (1 - src.a)`) stays in
//!    `[0, 1]` per channel for any opaque destination — i.e. **no
//!    "halo" bug** where a straight-alpha source paired with a
//!    premultiplied blend equation would overshoot 1.0 on AA edges.
//!
//! Both contracts had real regression cost during M14.5: the original
//! Vello/compositor handoff treated straight-alpha bytes as if they
//! were premultiplied and produced visible white halos around chrome.
//! These tests guard against that class of bug coming back via the
//! sprite path.

/// Software model of [`src/shaders/sprite.wgsl`](../src/shaders/sprite.wgsl)
/// `fs_main`. The shader reads:
/// ```wgsl
/// let tex = textureSample(atlas_tex, atlas_sampler, in.uv);
/// let color = tex * in.tint;
/// return vec4<f32>(color.rgb * color.a, color.a);
/// ```
/// Inputs are straight-alpha (atlas bytes and the tint uniform are
/// not premultiplied); the output is premultiplied.
fn sprite_fragment(atlas: [f32; 4], tint: [f32; 4]) -> [f32; 4] {
    let r = atlas[0] * tint[0];
    let g = atlas[1] * tint[1];
    let b = atlas[2] * tint[2];
    let a = atlas[3] * tint[3];
    [r * a, g * a, b * a, a]
}

/// Software model of wgpu's `BlendState::PREMULTIPLIED_ALPHA_BLENDING`
/// — the blend equation declared in [`src/pipeline.rs`](../src/pipeline.rs).
/// `out = src + dst * (1 - src.a)`, per channel including alpha.
/// `src` MUST be premultiplied (which is what `sprite_fragment`
/// produces). `dst` is whatever pixel was previously in the target.
fn premul_blend(src: [f32; 4], dst: [f32; 4]) -> [f32; 4] {
    let inv_a = 1.0 - src[3];
    [
        src[0] + dst[0] * inv_a,
        src[1] + dst[1] * inv_a,
        src[2] + dst[2] * inv_a,
        src[3] + dst[3] * inv_a,
    ]
}

fn approx(a: [f32; 4], b: [f32; 4]) {
    for (i, (x, y)) in a.iter().zip(b.iter()).enumerate() {
        assert!(
            (x - y).abs() < 1e-6,
            "channel {i}: got {x}, expected {y} (full: {a:?} vs {b:?})"
        );
    }
}

#[test]
fn opaque_white_atlas_with_opaque_white_tint_is_identity() {
    // Both inputs at full intensity → output is opaque premul white.
    let out = sprite_fragment([1.0; 4], [1.0; 4]);
    approx(out, [1.0, 1.0, 1.0, 1.0]);
}

#[test]
fn tint_alpha_scales_premultiplied_rgb() {
    // Tint α=0.5 over opaque atlas: pre-multiply collapses RGB by α,
    // so a fully-saturated white reads as 0.5 mid-gray with α=0.5.
    let out = sprite_fragment([1.0; 4], [1.0, 1.0, 1.0, 0.5]);
    approx(out, [0.5, 0.5, 0.5, 0.5]);
}

#[test]
fn atlas_alpha_scales_premultiplied_rgb() {
    // Atlas α=0.5 over opaque tint: same shape — final α=0.5, rgb
    // already pre-multiplied by it.
    let out = sprite_fragment([1.0, 1.0, 1.0, 0.5], [1.0; 4]);
    approx(out, [0.5, 0.5, 0.5, 0.5]);
}

#[test]
fn atlas_and_tint_alphas_multiply() {
    // Both at 0.5 → combined α = 0.25 → rgb collapses to 0.25.
    let out = sprite_fragment([1.0, 1.0, 1.0, 0.5], [1.0, 1.0, 1.0, 0.5]);
    approx(out, [0.25, 0.25, 0.25, 0.25]);
}

#[test]
fn colored_tint_multiplies_rgb_before_premultiply() {
    // Atlas opaque white, tint = red(0.8) opaque → color = (0.8,0,0,1)
    // → output premul = (0.8, 0, 0, 1).
    let out = sprite_fragment([1.0; 4], [0.8, 0.0, 0.0, 1.0]);
    approx(out, [0.8, 0.0, 0.0, 1.0]);
}

#[test]
fn premultiplied_invariant_rgb_le_alpha() {
    // For ANY combination of inputs in [0, 1], the output must
    // satisfy rgb ≤ a per channel. This is the defining property
    // of premultiplied alpha; if it breaks, the blend equation in
    // pipeline.rs will overshoot 1.0 and produce halos.
    let samples = [
        ([0.0; 4], [0.0; 4]),
        ([1.0; 4], [1.0; 4]),
        ([1.0; 4], [1.0, 0.5, 0.25, 0.5]),
        ([0.3, 0.6, 0.9, 0.5], [1.0; 4]),
        ([1.0, 1.0, 1.0, 0.001], [1.0; 4]),
        ([1.0, 1.0, 1.0, 1.0], [0.001; 4]),
    ];
    for (atlas, tint) in samples {
        let out = sprite_fragment(atlas, tint);
        let a = out[3];
        for (i, c) in out[..3].iter().enumerate() {
            assert!(
                *c <= a + 1e-6,
                "premul invariant broken (channel {i}={c} > a={a}) for atlas={atlas:?} tint={tint:?}"
            );
        }
    }
}

#[test]
fn blend_over_opaque_black_yields_premultiplied_rgb() {
    // Opaque black bg + half-alpha white sprite → result = src + 0 = src.
    // Result's alpha is src.a + 1*(1-src.a) = 1 (opaque).
    let src = sprite_fragment([1.0; 4], [1.0, 1.0, 1.0, 0.5]); // (0.5,0.5,0.5,0.5)
    let dst = [0.0, 0.0, 0.0, 1.0];
    let out = premul_blend(src, dst);
    approx(out, [0.5, 0.5, 0.5, 1.0]);
}

#[test]
fn blend_over_opaque_white_lerps_correctly() {
    // Opaque white bg + half-alpha black sprite → result = (0.5, ...).
    // With straight-alpha bug (where src.rgb stayed at 0 instead of
    // being premultiplied), result would be 1*(1-0.5) = 0.5 — same
    // number by accident but for the wrong reason.
    // With *colored* half-alpha sprite, the bug surfaces.
    let src = sprite_fragment([1.0; 4], [0.0, 0.0, 0.0, 0.5]); // (0,0,0,0.5)
    let dst = [1.0, 1.0, 1.0, 1.0];
    let out = premul_blend(src, dst);
    approx(out, [0.5, 0.5, 0.5, 1.0]);
}

#[test]
fn halo_regression_no_channel_exceeds_one() {
    // The original M14.5 halo bug: a *straight*-alpha source
    // (rgb=1, a=0.5) put through PREMULTIPLIED blend equation
    // becomes src.rgb + dst.rgb*(1-a) = 1 + 1*0.5 = 1.5 → halos.
    //
    // With the correct fragment shader (this test's `sprite_fragment`),
    // src.rgb is pre-multiplied so the equivalent input is
    // (0.5, 0.5, 0.5, 0.5), and the blend yields 1.0 max. Sweep a
    // grid of edge cases and confirm no channel ever exceeds 1.0.
    let alphas = [0.0_f32, 0.25, 0.5, 0.75, 1.0];
    let rgbs = [0.0_f32, 0.5, 1.0];
    let bg_colors = [[0.0_f32; 4], [0.5, 0.5, 0.5, 1.0], [1.0, 1.0, 1.0, 1.0]];
    for a in alphas {
        for r in rgbs {
            for g in rgbs {
                for b in rgbs {
                    let src = sprite_fragment([r, g, b, 1.0], [1.0, 1.0, 1.0, a]);
                    for dst in bg_colors {
                        let out = premul_blend(src, dst);
                        for (i, c) in out.iter().enumerate() {
                            assert!(
                                *c <= 1.0 + 1e-6,
                                "halo regression: channel {i}={c} > 1.0 for src=({r},{g},{b},a={a}) over dst={dst:?}",
                            );
                        }
                    }
                }
            }
        }
    }
}

#[test]
fn fully_transparent_sprite_preserves_destination() {
    // Sprite α=0 → src.rgb premultiplied to 0 → blend = dst.
    let src = sprite_fragment([1.0; 4], [1.0, 1.0, 1.0, 0.0]);
    approx(src, [0.0, 0.0, 0.0, 0.0]);
    let dst = [0.3, 0.6, 0.9, 1.0];
    let out = premul_blend(src, dst);
    approx(out, dst);
}

#[test]
fn fully_opaque_sprite_replaces_destination() {
    // Sprite α=1 → blend = src + dst*0 = src.
    let src = sprite_fragment([0.3, 0.6, 0.9, 1.0], [1.0; 4]);
    let dst = [1.0, 1.0, 1.0, 1.0];
    let out = premul_blend(src, dst);
    approx(out, src);
}
