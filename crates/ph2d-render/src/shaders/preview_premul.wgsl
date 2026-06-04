// Straight → premultiplied alpha blit for the Painter GPU live preview
// (ADR-0045 Phase 3, step 2). The layer compositor writes a STRAIGHT
// sRGB8 (`rgba8unorm`) result; the sprite preview slot samples
// `Rgba8UnormSrgb` PREMULTIPLIED (so its bilinear `textureSample`
// composites edge texels as `rgb·a`, matching the CPU `premultiply_rgba8`
// upload path — see `crate::premul`). This compute pass bridges the two
// WITHOUT a CPU readback: it premultiplies each texel in place.
//
// `textureLoad` on the `rgba8unorm` source returns `byte / 255` with NO
// sRGB decode, so `rgb * a` in this normalized space equals the byte-space
// `rgb * a / 255` of `premultiply_rgba8` (the store rounds to nearest,
// so the result is bit-identical to ±1 — the same ULP bound the layer
// compositor's GPU↔CPU parity gate already accepts).

@group(0) @binding(0) var src: texture_2d<f32>;
@group(0) @binding(1) var dst: texture_storage_2d<rgba8unorm, write>;

@compute @workgroup_size(8, 8, 1)
fn cs_main(@builtin(global_invocation_id) gid: vec3<u32>) {
    let dims = textureDimensions(dst);
    if (gid.x >= dims.x || gid.y >= dims.y) {
        return;
    }
    let coord = vec2<i32>(i32(gid.x), i32(gid.y));
    let c = textureLoad(src, coord, 0);
    // Straight-alpha premultiply: rgb' = rgb * a, alpha unchanged.
    let pm = vec4<f32>(c.rgb * c.a, c.a);
    textureStore(dst, coord, pm);
}
