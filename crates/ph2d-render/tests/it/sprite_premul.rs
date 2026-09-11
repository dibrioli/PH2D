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

/// Software model of the `in.premultiplied > 0.5` branch added for the
/// BG-Removal fringe fix:
/// ```wgsl
/// let color = tex * in.tint;
/// if (in.premultiplied > 0.5) { return color; }
/// ```
/// `tex` is sampled from an ALREADY-premultiplied texture, so the
/// shader returns it (× tint) directly without a second premultiply.
fn sprite_fragment_premul(tex: [f32; 4], tint: [f32; 4]) -> [f32; 4] {
    [
        tex[0] * tint[0],
        tex[1] * tint[1],
        tex[2] * tint[2],
        tex[3] * tint[3],
    ]
}

/// 1-D bilinear lerp of two RGBA texels at parameter `t` — the core of
/// what the GPU sampler does on an edge between two texels. Used to
/// contrast the two compositing models (premultiply-before vs -after
/// sampling) that drive the fringe.
fn lerp(a: [f32; 4], b: [f32; 4], t: f32) -> [f32; 4] {
    [
        a[0] + (b[0] - a[0]) * t,
        a[1] + (b[1] - a[1]) * t,
        a[2] + (b[2] - a[2]) * t,
        a[3] + (b[3] - a[3]) * t,
    ]
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

// ---- BG-Removal fringe fix: premultiplied-texture branch ----

#[test]
fn premul_branch_passes_premultiplied_texel_through() {
    // A valid premultiplied texel (rgb already ≤ a) with opaque white
    // tint must come out unchanged AND already satisfy the premultiplied
    // invariant the blend equation expects.
    let tex = [0.4, 0.2, 0.1, 0.5]; // premultiplied: rgb ≤ a
    let out = sprite_fragment_premul(tex, [1.0; 4]);
    approx(out, tex);
    assert!(out[0] <= out[3] && out[1] <= out[3] && out[2] <= out[3]);
}

#[test]
fn premul_branch_eliminates_edge_fringe_vs_straight_branch() {
    // The fringe scenario: an edge between a fully-transparent texel and
    // an opaque RED texel, sampled at the bilinear midpoint (t = 0.5).
    //
    // Reference = what the Vello preview does (premultiply BEFORE
    // sampling). For a straight RED opaque texel and a transparent
    // texel, premultiplied forms are (1,0,0,1) and (0,0,0,0); their
    // bilinear midpoint is (0.5, 0, 0, 0.5).
    let premul_opaque_red = [1.0, 0.0, 0.0, 1.0];
    let premul_transparent = [0.0, 0.0, 0.0, 0.0];
    let reference = lerp(premul_transparent, premul_opaque_red, 0.5);
    approx(reference, [0.5, 0.0, 0.0, 0.5]);

    // APPLY path WITH the fix: texture stores premultiplied data, the
    // sampler lerps it, and the premul branch passes it through.
    let sampled_premul = lerp(premul_transparent, premul_opaque_red, 0.5);
    let fixed = sprite_fragment_premul(sampled_premul, [1.0; 4]);
    approx(fixed, reference);

    // OLD (buggy) APPLY path: texture stores STRAIGHT data, so the
    // transparent texel keeps whatever RGB it carried. Background
    // Removal zeroes alpha but the edge texel's straight RGB is NOT
    // zero — model it as a contaminating dark/purple straight value
    // (here a stray blue) at a=0. The straight sampler lerps colour at
    // full weight, then premultiplies after.
    let straight_opaque_red = [1.0, 0.0, 0.0, 1.0];
    let straight_transparent_contaminated = [0.0, 0.0, 0.6, 0.0]; // a=0 but rgb≠0
    let sampled_straight = lerp(straight_transparent_contaminated, straight_opaque_red, 0.5);
    let buggy = sprite_fragment(sampled_straight, [1.0; 4]);

    // The buggy path leaks blue into the edge (the fringe); the fixed
    // path matches the reference exactly and carries no blue.
    assert!(
        buggy[2] > 1e-3,
        "expected the straight-alpha path to leak a blue fringe, got {buggy:?}"
    );
    assert!(
        fixed[2].abs() < 1e-6,
        "fixed path must carry no fringe colour, got {fixed:?}"
    );
}

#[test]
fn premul_branch_keeps_invariant_for_blend() {
    // A premultiplied texel passed through the branch then blended over
    // any opaque background never overshoots 1.0 (no halo), same
    // guarantee the straight branch gives.
    let texels = [
        [0.0, 0.0, 0.0, 0.0],
        [0.5, 0.0, 0.0, 0.5],
        [1.0, 1.0, 1.0, 1.0],
        [0.2, 0.4, 0.6, 0.8],
    ];
    let bgs = [[0.0; 4], [1.0, 1.0, 1.0, 1.0], [0.5, 0.5, 0.5, 1.0]];
    for tex in texels {
        let src = sprite_fragment_premul(tex, [1.0; 4]);
        for dst in bgs {
            let out = premul_blend(src, dst);
            for (i, c) in out.iter().enumerate() {
                assert!(
                    *c <= 1.0 + 1e-6,
                    "premul-branch halo: channel {i}={c} for tex={tex:?} over {dst:?}"
                );
            }
        }
    }
}
