// Motion glow (bloom) — the Call of Duty / Jimenez mip-chain (SIGGRAPH 2014),
// the technique Unity/Unreal ship and the reference LearnOpenGL "Physically Based
// Bloom" documents. A SINGLE wide box blur keeps the SQUARE of the source quad;
// this doesn't. The bright-passed image is progressively DOWNSAMPLED (13-tap) into
// a mip chain, then UPSAMPLED back (9-tap tent) with additive accumulation — the
// repeated bilinear halving dissolves the source's corners into a ROUND, smooth
// falloff with energy at every scale (a tight core halo AND a soft wide glow).
//
// HDR throughout (Rgba16Float): highlights above 1.0 survive the chain instead of
// clipping to white the way an 8-bit round-trip would. No transcendentals (HR-5):
// every pass is weighted texture taps.

struct VsOut {
    @builtin(position) pos: vec4<f32>,
    @location(0) uv: vec2<f32>,
};

// Fullscreen triangle from the vertex index — no vertex buffer (matches tonemap).
@vertex
fn vs_main(@builtin(vertex_index) vid: u32) -> VsOut {
    var out: VsOut;
    let x = f32((vid << 1u) & 2u);
    let y = f32(vid & 2u);
    out.uv = vec2<f32>(x, 1.0 - y);
    out.pos = vec4<f32>(x * 2.0 - 1.0, y * 2.0 - 1.0, 0.0, 1.0);
    return out;
}

@group(0) @binding(0) var src: texture_2d<f32>;
@group(0) @binding(1) var samp: sampler;

struct Params {
    // Meaning is per-pass; see each fragment:
    //  prefilter  → v = (threshold, threshold-knee, 2·knee, 0.25/knee)
    //  downsample → v = (srcTexel.x, srcTexel.y, _, _)
    //  upsample   → v = (filterRadius.x, filterRadius.y, _, _)  (UV; y aspect-scaled)
    //  composite  → v = (intensity, saturation, _, _), v2 = tint rgba
    v: vec4<f32>,
    v2: vec4<f32>,
};
@group(0) @binding(2) var<uniform> P: Params;

// Bright-pass with a soft knee (COD/Karis): below `threshold-knee` nothing
// contributes, above `threshold` full, quadratic ramp between — no hard edge.
// Premultiplied HDR in (Motion is over transparent black), so max(r,g,b) is a
// premult-safe brightness. Also downsamples (mip0 is half-res, bilinear).
@fragment
fn fs_prefilter(in: VsOut) -> @location(0) vec4<f32> {
    let c = textureSample(src, samp, in.uv).rgb;
    let brightness = max(c.r, max(c.g, c.b));
    var soft = brightness - P.v.y;
    soft = clamp(soft, 0.0, P.v.z);
    soft = soft * soft * P.v.w;
    let contribution = max(soft, brightness - P.v.x) / max(brightness, 1e-4);
    return vec4<f32>(c * contribution, 1.0);
}

// 13-tap downsample (COD): centre + inner 2×2 + outer ring, weighted so the
// filter is a smooth wide tent — this is where the corners get rounded off. `x`,
// `y` are the SOURCE texel size, so the taps land on the source's texel grid.
@fragment
fn fs_downsample(in: VsOut) -> @location(0) vec4<f32> {
    let x = P.v.x;
    let y = P.v.y;
    let uv = in.uv;
    let a = textureSample(src, samp, uv + vec2<f32>(-2.0 * x, 2.0 * y)).rgb;
    let b = textureSample(src, samp, uv + vec2<f32>(0.0, 2.0 * y)).rgb;
    let c = textureSample(src, samp, uv + vec2<f32>(2.0 * x, 2.0 * y)).rgb;
    let d = textureSample(src, samp, uv + vec2<f32>(-2.0 * x, 0.0)).rgb;
    let e = textureSample(src, samp, uv).rgb;
    let f = textureSample(src, samp, uv + vec2<f32>(2.0 * x, 0.0)).rgb;
    let g = textureSample(src, samp, uv + vec2<f32>(-2.0 * x, -2.0 * y)).rgb;
    let h = textureSample(src, samp, uv + vec2<f32>(0.0, -2.0 * y)).rgb;
    let i = textureSample(src, samp, uv + vec2<f32>(2.0 * x, -2.0 * y)).rgb;
    let j = textureSample(src, samp, uv + vec2<f32>(-x, y)).rgb;
    let k = textureSample(src, samp, uv + vec2<f32>(x, y)).rgb;
    let l = textureSample(src, samp, uv + vec2<f32>(-x, -y)).rgb;
    let m = textureSample(src, samp, uv + vec2<f32>(x, -y)).rgb;
    var o = e * 0.125;
    o += (a + c + g + i) * 0.03125;
    o += (b + d + f + h) * 0.0625;
    o += (j + k + l + m) * 0.125;
    return vec4<f32>(o, 1.0);
}

// 9-tap tent upsample (COD): a 3×3 [1 2 1; 2 4 2; 1 2 1]/16 kernel at
// `filterRadius`. Blended ADDITIVELY onto the finer mip (which already holds its
// own downsample), so the accumulation across the chain is what builds the round
// multi-scale glow. `filterRadius` (P.v.xy) is UV, y aspect-scaled for a round
// spread in pixels.
@fragment
fn fs_upsample(in: VsOut) -> @location(0) vec4<f32> {
    let x = P.v.x;
    let y = P.v.y;
    let uv = in.uv;
    let a = textureSample(src, samp, uv + vec2<f32>(-x, y)).rgb;
    let b = textureSample(src, samp, uv + vec2<f32>(0.0, y)).rgb;
    let c = textureSample(src, samp, uv + vec2<f32>(x, y)).rgb;
    let d = textureSample(src, samp, uv + vec2<f32>(-x, 0.0)).rgb;
    let e = textureSample(src, samp, uv).rgb;
    let f = textureSample(src, samp, uv + vec2<f32>(x, 0.0)).rgb;
    let g = textureSample(src, samp, uv + vec2<f32>(-x, -y)).rgb;
    let h = textureSample(src, samp, uv + vec2<f32>(0.0, -y)).rgb;
    let i = textureSample(src, samp, uv + vec2<f32>(x, -y)).rgb;
    var o = e * 4.0;
    o += (b + d + f + h) * 2.0;
    o += (a + c + g + i);
    o *= 1.0 / 16.0;
    return vec4<f32>(o, 1.0);
}

// The accumulated glow (mip0), coloured then scaled: desaturate toward luminance
// by `saturation` (0 = a white bloom), multiply by the tint (default white =
// no-op), then by intensity. The pipeline blends this ADDITIVELY over the scene
// (color One/One) — emitted light only brightens, so it bleeds over whatever is
// in front. Alpha 0 + the pipeline keeps the destination alpha, so the opaque
// scene stays opaque for the compositor.
@fragment
fn fs_composite(in: VsOut) -> @location(0) vec4<f32> {
    let intensity = P.v.x;
    let saturation = P.v.y;
    let tint = P.v2;
    var glow = textureSample(src, samp, in.uv).rgb;
    // Rec.709 luminance; mix toward grey for a white bloom at saturation 0.
    let lum = dot(glow, vec3<f32>(0.2126, 0.7152, 0.0722));
    glow = mix(vec3<f32>(lum), glow, saturation);
    glow = glow * tint.rgb * (intensity * tint.a);
    return vec4<f32>(glow, 0.0);
}
