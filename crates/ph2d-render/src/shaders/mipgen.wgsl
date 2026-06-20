// mipgen.wgsl — generate one mip level from the level above by a 2× box
// downsample. A fullscreen triangle covers the (half-size) target mip; each
// output texel's CENTRE uv lands exactly on the 2×2 boundary of the source mip,
// so a single bilinear `textureSample` averages that 2×2 block. The source is an
// `Rgba8UnormSrgb` view ⇒ the hardware decodes sRGB→linear BEFORE averaging and
// re-encodes on write, so the downsample is correct in LINEAR light (and alpha,
// being linear + premultiplied, averages correctly too).

@group(0) @binding(0) var src: texture_2d<f32>;
@group(0) @binding(1) var samp: sampler;

struct VsOut {
    @builtin(position) pos: vec4<f32>,
    @location(0) uv: vec2<f32>,
};

@vertex
fn vs_main(@builtin(vertex_index) vi: u32) -> VsOut {
    // Oversized fullscreen triangle (covers [-1,1]² with one primitive).
    var out: VsOut;
    let x = f32((vi << 1u) & 2u); // 0, 2, 0
    let y = f32(vi & 2u);         // 0, 0, 2
    out.uv = vec2<f32>(x, y);
    out.pos = vec4<f32>(x * 2.0 - 1.0, 1.0 - y * 2.0, 0.0, 1.0);
    return out;
}

@fragment
fn fs_main(in: VsOut) -> @location(0) vec4<f32> {
    return textureSample(src, samp, in.uv);
}
