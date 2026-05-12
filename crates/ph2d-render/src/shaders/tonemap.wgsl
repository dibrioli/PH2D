// Tonemap pass (M14.5).
//
// Samples the HDR game RT (Rgba16Float, linear-light) and writes
// the tonemapped result into an LDR sRGB target. AgX LUT-based
// implementation: the 3D LUT (33³ Rgba8Unorm) is bound at
// @group(0) @binding(2). When no LUT is supplied the host can use
// `Tonemap::new_identity()` which binds a 2×2×2 identity volume —
// the shader is the same.
//
// Coordinate convention: the LUT input domain is AgX's "Log2"
// encoding remapped to [0, 1]. We map linear HDR → log2 → [0, 1]
// via the standard AgX log encoding (min EV = -12.47, max EV = +4.03,
// Blender 4.0+ reference values), sample the LUT trilinearly, then
// return the LDR result. The output texture format is *Srgb so wgpu
// re-encodes the linear output to sRGB on write — there is **no
// manual gamma** in this shader.
//
// ─── State of the AgX path (M14.5, audit 2026-05-11) ─────────────
//
// `BYPASS_LUT = true` is the **intentional** default until the editor
// canvas hosts genuine HDR content (e.g. additive lights > 1.0,
// particle emitters with bloom, HDR sprite imports). Reasons:
//   1. Current editor content is in-gamut sRGB. Tonemapping does
//      nothing perceptually useful for [0, 1] colors.
//   2. The bundled identity LUT stores raw RGB 0..1 in voxels, but
//      the shader samples it at log2-encoded coords. That mismatch
//      applies an unintended log curve — i.e. the path is NOT
//      mathematically identity. Routing through it would tint mid-
//      tones (the "dull look" reported during M14.5 round 7).
//
// ─── Migration trigger ──────────────────────────────────────────
//
// Flip `BYPASS_LUT = false` AND replace the identity LUT with a real
// AgX bake when ANY of the following ships:
//   • Light/particle component emitting linear values > 1.0 to game_rt
//   • HDR texture import path (EXR / HDR / 16-bit PNG sprites)
//   • Bloom or glow post-process pass that exceeds [0, 1] range
//
// Bake script lives at (TBD) `tools/bake_agx_lut/` — outputs
// `assets/luts/agx_default.cube` for `Tonemap::new()` to load. Until
// the trigger fires, this code path is intentionally cold but kept
// compile-clean so the wiring (bind groups, samplers, pipeline) stays
// validated each build.
const BYPASS_LUT: bool = true;

@group(0) @binding(0)
var game_rt: texture_2d<f32>;
@group(0) @binding(1)
var game_sampler: sampler;
@group(0) @binding(2)
var agx_lut: texture_3d<f32>;
@group(0) @binding(3)
var lut_sampler: sampler;

struct VsOut {
    @builtin(position) clip_pos: vec4<f32>,
    @location(0) uv: vec2<f32>,
};

// Fullscreen triangle — three verts cover the whole NDC quad with a
// single triangle, cheaper than two-triangle quads (no shared edge).
@vertex
fn vs_main(@builtin(vertex_index) vid: u32) -> VsOut {
    var out: VsOut;
    let x = f32((vid << 1u) & 2u);
    let y = f32(vid & 2u);
    out.uv = vec2<f32>(x, 1.0 - y);
    out.clip_pos = vec4<f32>(x * 2.0 - 1.0, y * 2.0 - 1.0, 0.0, 1.0);
    return out;
}

// AgX log-encoding constants (Troy Sobotka's reference values used
// by Blender 4.0+). Linear → Log2 → [0, 1].
const AGX_MIN_EV: f32 = -12.47393;
const AGX_MAX_EV: f32 = 4.026069;

fn agx_log2_encode(linear: vec3<f32>) -> vec3<f32> {
    // Guard against log2(0) → -inf.
    let safe = max(linear, vec3<f32>(1e-10));
    let log2v = log2(safe);
    return clamp(
        (log2v - vec3<f32>(AGX_MIN_EV)) / (AGX_MAX_EV - AGX_MIN_EV),
        vec3<f32>(0.0),
        vec3<f32>(1.0),
    );
}

@fragment
fn fs_main(in: VsOut) -> @location(0) vec4<f32> {
    let hdr = textureSample(game_rt, game_sampler, in.uv);

    if (BYPASS_LUT) {
        // Pure passthrough: clamp HDR to LDR, let the sRGB target encode.
        // Keeps premultiplied alpha invariant (rgb already scaled by a).
        return vec4<f32>(clamp(hdr.rgb, vec3<f32>(0.0), vec3<f32>(1.0)), hdr.a);
    }

    // Premultiplied alpha throughout: divide-out alpha BEFORE tonemap
    // so highlights of semi-transparent sprites don't collapse to
    // mid-grey.
    let safe_a = max(hdr.a, 1e-6);
    let straight_rgb = hdr.rgb / safe_a;
    let coord = agx_log2_encode(straight_rgb);
    // Trilinear LUT sample. The 33³ identity LUT produces hdr_clamped
    // (the log2-decoded straight color, clamped to [0,1]) — visually
    // identical to no-op for in-gamut content, and the place to swap
    // in AgX's full sigmoid + per-channel curves once the LUT is baked.
    let mapped_rgb = textureSample(agx_lut, lut_sampler, coord).rgb;
    // Re-premultiply so the compositor's blend math stays consistent.
    return vec4<f32>(mapped_rgb * hdr.a, hdr.a);
}
