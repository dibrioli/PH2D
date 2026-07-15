// Motion glow (bloom) — three fullscreen passes sharing one bind-group layout
// (source texture @0, sampler @1, a single vec4 of params @2). HDR throughout:
// input and every intermediate is Rgba16Float, so highlights above 1.0 survive
// the blur instead of clipping to white the way an 8-bit round-trip would.
//
// The chain is prefilter → Kawase blur (half-res ping-pong) → additive
// composite. No transcendentals (HR-5): every pass is weighted texture taps.

struct VsOut {
    @builtin(position) pos: vec4<f32>,
    @location(0) uv: vec2<f32>,
};

// Fullscreen triangle from the vertex index — no vertex buffer (matches the
// house tonemap pass). vid 0,1,2 → a triangle covering the [0,1] UV square.
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
    //  prefilter → (threshold, threshold-knee, 2·knee, 0.25/knee)
    //  blur      → (offset.x, offset.y, _, _) in UV units
    //  composite → (intensity, _, _, _)
    v: vec4<f32>,
};
@group(0) @binding(2) var<uniform> P: Params;

// Bright-pass with a soft knee (Call of Duty / Karis "Next-Gen Post", 2014):
// pixels below `threshold-knee` contribute nothing, pixels above `threshold`
// pass in full, and the band between ramps quadratically so the glow has no
// hard edge. Operates on premultiplied HDR (Motion is composited over
// transparent black), so `max(r,g,b)` is a premult-safe brightness.
@fragment
fn fs_prefilter(in: VsOut) -> @location(0) vec4<f32> {
    let c = textureSample(src, samp, in.uv);
    let brightness = max(c.r, max(c.g, c.b));
    // curve = (threshold, threshold-knee, 2·knee, 0.25/knee), packed CPU-side.
    var soft = brightness - P.v.y;
    soft = clamp(soft, 0.0, P.v.z);
    soft = soft * soft * P.v.w;
    let contribution = max(soft, brightness - P.v.x) / max(brightness, 1e-4);
    return vec4<f32>(c.rgb * contribution, c.a);
}

// One Kawase blur iteration: four bilinear taps at the diagonal corners of a
// box whose half-extent is `P.v.xy` (UV units, already scaled by the radius
// knob). Repeated with growing offsets it approximates a wide Gaussian at a
// fraction of the tap count.
@fragment
fn fs_blur(in: VsOut) -> @location(0) vec4<f32> {
    let o = P.v.xy;
    var sum = textureSample(src, samp, in.uv + vec2<f32>(o.x, o.y));
    sum += textureSample(src, samp, in.uv + vec2<f32>(-o.x, o.y));
    sum += textureSample(src, samp, in.uv + vec2<f32>(o.x, -o.y));
    sum += textureSample(src, samp, in.uv + vec2<f32>(-o.x, -o.y));
    return sum * 0.25;
}

// Scale the blurred glow by intensity. The pipeline blends this ADDITIVELY over
// the scene (color One/One), so glow only ever brightens — emitted light bleeds
// over whatever is in front, which is what makes the sparks look lit rather than
// pasted. Alpha is written 0 and the pipeline keeps the destination alpha, so
// the opaque scene stays opaque for the compositor.
@fragment
fn fs_composite(in: VsOut) -> @location(0) vec4<f32> {
    let glow = textureSample(src, samp, in.uv).rgb * P.v.x;
    return vec4<f32>(glow, 0.0);
}
