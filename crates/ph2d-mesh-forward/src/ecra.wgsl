// ── A CODIFICACAO: o `premultiplicado::para_ecra` do Render, pixel a pixel ───────────────────────
@group(0) @binding(0) var resolvida: texture_2d<f32>;

@vertex
fn vs_ecra(@builtin(vertex_index) k: u32) -> @builtin(position) vec4<f32> {
    let p = array<vec2<f32>, 3>(vec2<f32>(-1.0, -1.0), vec2<f32>(3.0, -1.0), vec2<f32>(-1.0, 3.0));
    return vec4<f32>(p[k], 0.0, 1.0);
}

fn srgb(x: f32) -> f32 {
    let c = clamp(x, 0.0, 1.0);
    if (c <= 0.0031308) {
        return c * 12.92;
    }
    return 1.055 * pow(c, 1.0 / 2.4) - 0.055;
}

@fragment
fn fs_ecra(@builtin(position) q: vec4<f32>) -> @location(0) vec4<f32> {
    let c = textureLoad(resolvida, vec2<i32>(q.xy), 0);
    let a8 = floor(clamp(c.a, 0.0, 1.0) * 255.0 + 0.5);
    let a = a8 / 255.0;
    var inv = 0.0;
    if (a8 > 0.0) {
        inv = 1.0 / a;
    }
    return vec4<f32>(srgb(c.r * inv) * a, srgb(c.g * inv) * a, srgb(c.b * inv) * a, a);
}
