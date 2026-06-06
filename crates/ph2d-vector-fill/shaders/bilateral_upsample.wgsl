// === ph2d-vector-fill bilateral_upsample.wgsl (ADR-0060 §2.5) ===
// Joint-bilateral upsample in 2 passes (the single 21x21 mega-kernel blew the
// mobile budget — §4.3): pass 1 denoises the noisy low-res WoS result with a 3x3
// bilateral filter; pass 2 guided-upscales it to display resolution. Both passes
// operate on linear-RGBA storage buffers (no texture-format capabilities needed;
// the renderer copies the final buffer into its fill texture).

struct JbuParams {
    low_w: u32,
    low_h: u32,
    high_w: u32,
    high_h: u32,
    // Bilateral range bandwidth (colour-distance falloff); larger = smoother.
    sigma_range: f32,
    _p0: f32,
    _p1: f32,
    _p2: f32,
};

@group(0) @binding(0) var<uniform> jp: JbuParams;
@group(0) @binding(1) var<storage, read> src: array<vec4<f32>>;
@group(0) @binding(2) var<storage, read_write> dst: array<vec4<f32>>;

// Clamp-to-edge fetch from the low-res source.
fn load_low(x: i32, y: i32) -> vec4<f32> {
    let cx = clamp(x, 0, i32(jp.low_w) - 1);
    let cy = clamp(y, 0, i32(jp.low_h) - 1);
    return src[u32(cy) * jp.low_w + u32(cx)];
}

// Pass 1 — 3x3 bilateral denoise at low res. Spatial weight is a fixed Gaussian;
// the range weight (joint term) falls off with colour distance, preserving the
// sharp curve boundaries the diffusion produced while killing WoS speckle.
@compute @workgroup_size(8, 8, 1)
fn jbu_denoise(@builtin(global_invocation_id) gid: vec3<u32>) {
    if (gid.x >= jp.low_w || gid.y >= jp.low_h) {
        return;
    }
    let center = load_low(i32(gid.x), i32(gid.y));
    var sum: vec4<f32> = vec4<f32>(0.0);
    var wsum: f32 = 0.0;
    for (var dy: i32 = -1; dy <= 1; dy = dy + 1) {
        for (var dx: i32 = -1; dx <= 1; dx = dx + 1) {
            let s = load_low(i32(gid.x) + dx, i32(gid.y) + dy);
            let spatial = exp(-0.5 * f32(dx * dx + dy * dy));
            let d = s.xyz - center.xyz;
            let range = exp(-dot(d, d) / max(2.0 * jp.sigma_range * jp.sigma_range, 1e-6));
            let w = spatial * range;
            sum = sum + s * w;
            wsum = wsum + w;
        }
    }
    dst[gid.y * jp.low_w + gid.x] = sum / max(wsum, 1e-6);
}

// Pass 2 — guided upscale low -> high. Bilinear reconstruction at the high-res
// sample position; the low-res field is already boundary-aware (its values jump
// at curves), so bilinear preserves edges without ringing.
@compute @workgroup_size(8, 8, 1)
fn jbu_upsample(@builtin(global_invocation_id) gid: vec3<u32>) {
    if (gid.x >= jp.high_w || gid.y >= jp.high_h) {
        return;
    }
    let u = (f32(gid.x) + 0.5) / f32(jp.high_w);
    let v = (f32(gid.y) + 0.5) / f32(jp.high_h);
    let lx = u * f32(jp.low_w) - 0.5;
    let ly = v * f32(jp.low_h) - 0.5;
    let x0 = i32(floor(lx));
    let y0 = i32(floor(ly));
    let fx = lx - floor(lx);
    let fy = ly - floor(ly);
    let c00 = load_low(x0, y0);
    let c10 = load_low(x0 + 1, y0);
    let c01 = load_low(x0, y0 + 1);
    let c11 = load_low(x0 + 1, y0 + 1);
    let top = mix(c00, c10, fx);
    let bot = mix(c01, c11, fx);
    dst[gid.y * jp.high_w + gid.x] = mix(top, bot, fy);
}
