// Straight → premultiplied alpha blit for the Painter GPU live preview
// (ADR-0045 Phase 3, step 2). The layer compositor writes a STRAIGHT
// sRGB8 (`rgba8unorm`) result; the sprite preview slot samples
// `Rgba8UnormSrgb` PREMULTIPLIED (so its bilinear `textureSample`
// composites edge texels as `rgb·a`, matching the CPU `premultiply_rgba8`
// upload path — see `crate::premul`). This compute pass bridges the two
// WITHOUT a CPU readback: it premultiplies each texel in place.
//
// `textureLoad` on the `rgba8unorm` source returns `byte / 255` with NO
// sRGB decode; the pass redoes the CPU's integer premultiply on those bytes,
// so the preview of the GPU is the CPU's byte, 0 levels apart (B20.8).

@group(0) @binding(0) var src: texture_2d<f32>;
@group(0) @binding(1) var dst: texture_storage_2d<rgba8unorm, write>;
// The rect to premultiply: origin xy, extent zw (the whole canvas for a full pass).
@group(0) @binding(2) var<uniform> rect: vec4<u32>;

@compute @workgroup_size(8, 8, 1)
fn cs_main(@builtin(global_invocation_id) gid: vec3<u32>) {
    if (gid.x >= rect.z || gid.y >= rect.w) {
        return;
    }
    let coord = vec2<i32>(i32(rect.x + gid.x), i32(rect.y + gid.y));
    // The bytes back (`byte / 255` is within 1e-5 of the integer), then the CPU's integer law
    // `(c·a + 127) / 255` (`premultiply_rgba8`): the store gets `k / 255` within 1e-5 of the byte, so the
    // texel is the CPU's byte on every driver, never the float product rounded by its tolerance.
    let b = vec4<u32>(round(textureLoad(src, coord, 0) * 255.0));
    let pm = (b.rgb * b.a + vec3<u32>(127u)) / 255u;
    textureStore(dst, coord, vec4<f32>(vec4<u32>(pm, b.a)) / 255.0);
}
