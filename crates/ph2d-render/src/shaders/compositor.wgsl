// Compositor pass (M14.5 round 7 — designer-space blending).
//
// Straight-alpha "over" (Porter-Duff) in **sRGB-as-linear** (designer)
// space, NOT scene-linear. This matches Figma, browsers, and legacy
// 2D engines — design tokens curated against those tools render with
// the same perceptual hue/contrast they were calibrated against.
//
// Pipeline:
//   1. Sample `game_rt` (Bgra8UnormSrgb → wgpu auto-decodes to linear).
//   2. Re-encode game to sRGB so the blend happens in the same space
//      as Vello's raw bytes (which are sRGB-encoded designer values).
//   3. Blend straight-alpha "over" in designer space.
//   4. Decode the blended result back to linear before write — the
//      swap chain is Bgra8UnormSrgb and will sRGB-encode on store.
//      Decoding first cancels that encode → bytes land as designed.
//
//     game_srgb = linear_to_srgb(textureSample(game_rt).rgb)
//     out_srgb  = vello.rgb * vello.a + game_srgb * (1 - vello.a)
//     return srgb_to_linear(out_srgb)
//
// `vello_rt` is `Rgba8Unorm` (storage-binding requirement; sRGB view
// sibling forbidden by wgpu validation). Vello v0.8 writes
// sRGB-encoded straight-alpha bytes — we deliberately do NOT decode
// them: that *is* the designer space we want to blend in.
//
// Vello straight-alpha confirmation:
// `vello_shaders/shader/fine.wgsl:1271-1281` (v0.8.0) — the fine
// kernel divides RGB by alpha before `textureStore`, so the bytes
// in the intermediate are straight alpha. We premultiply on the fly
// in the `over` math above.

@group(0) @binding(0)
var game_rt: texture_2d<f32>;
@group(0) @binding(1)
var game_sampler: sampler;
@group(0) @binding(2)
var vello_rt: texture_2d<f32>;
@group(0) @binding(3)
var vello_sampler: sampler;

struct VsOut {
    @builtin(position) clip_pos: vec4<f32>,
    @location(0) uv: vec2<f32>,
};

@vertex
fn vs_main(@builtin(vertex_index) vid: u32) -> VsOut {
    var out: VsOut;
    let x = f32((vid << 1u) & 2u);
    let y = f32(vid & 2u);
    out.uv = vec2<f32>(x, 1.0 - y);
    out.clip_pos = vec4<f32>(x * 2.0 - 1.0, y * 2.0 - 1.0, 0.0, 1.0);
    return out;
}

// sRGB → linear (IEC 61966-2-1), per-channel.
fn srgb_to_linear(c: vec3<f32>) -> vec3<f32> {
    let lo = c / 12.92;
    let hi = pow((c + vec3<f32>(0.055)) / 1.055, vec3<f32>(2.4));
    let cutoff = step(vec3<f32>(0.04045), c);
    return mix(lo, hi, cutoff);
}

// linear → sRGB (IEC 61966-2-1), per-channel.
fn linear_to_srgb(c: vec3<f32>) -> vec3<f32> {
    let safe = clamp(c, vec3<f32>(0.0), vec3<f32>(1.0));
    let lo = safe * 12.92;
    let hi = 1.055 * pow(safe, vec3<f32>(1.0 / 2.4)) - vec3<f32>(0.055);
    let cutoff = step(vec3<f32>(0.0031308), safe);
    return mix(lo, hi, cutoff);
}

@fragment
fn fs_main(in: VsOut) -> @location(0) vec4<f32> {
    // game_rt is Bgra8UnormSrgb: wgpu auto-decodes to linear on sample.
    // Re-encode to sRGB to enter designer space for the blend.
    let game_lin = textureSample(game_rt, game_sampler, in.uv).rgb;
    let game_srgb = linear_to_srgb(game_lin);

    // vello_rt bytes are already sRGB-encoded straight-alpha — that's
    // the designer space, so we DO NOT decode.
    let vello = textureSample(vello_rt, vello_sampler, in.uv);

    // Straight-alpha "over" in designer space.
    let out_srgb = vello.rgb * vello.a + game_srgb * (1.0 - vello.a);

    // Swap chain is Bgra8UnormSrgb → wgpu will sRGB-encode on store.
    // Pre-decode so the encode cancels and bytes land as composed.
    return vec4<f32>(srgb_to_linear(out_srgb), 1.0);
}
