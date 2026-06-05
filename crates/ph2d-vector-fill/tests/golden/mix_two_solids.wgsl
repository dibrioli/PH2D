// === ph2d-vector-fill generated shader (ADR-0060) ===
struct FillParams {
    colors: array<vec4<f32>, 16>,
    scalars: array<vec4<f32>, 16>,
    ucontrol: array<vec4<u32>, 16>,
    stop_colors: array<array<vec4<f32>, 8>, 16>,
    stop_pos: array<array<vec4<f32>, 2>, 16>,
    time: f32,
};
@group(0) @binding(0) var<uniform> params: FillParams;

fn ph2d_blend(a: vec4<f32>, b: vec4<f32>, mode: u32) -> vec4<f32> {
    switch mode {
        case 0u: {
            let oa = b.a + a.a * (1.0 - b.a);
            let rgb = (b.rgb * b.a + a.rgb * a.a * (1.0 - b.a)) / max(oa, 1e-5);
            return vec4<f32>(rgb, oa);
        }
        case 1u: { return a * b; }
        case 2u: { return vec4<f32>(1.0) - (vec4<f32>(1.0) - a) * (vec4<f32>(1.0) - b); }
        case 3u: {
            let lo = 2.0 * a * b;
            let hi = vec4<f32>(1.0) - 2.0 * (vec4<f32>(1.0) - a) * (vec4<f32>(1.0) - b);
            return select(lo, hi, a > vec4<f32>(0.5));
        }
        case 4u: { return min(a + b, vec4<f32>(1.0)); }
        case 5u: { return b; }
        default: { return b; }
    }
}
fn ph2d_mix_blend(a: vec4<f32>, b: vec4<f32>, factor: f32, mode: u32) -> vec4<f32> {
    return mix(a, ph2d_blend(a, b, mode), clamp(factor, 0.0, 1.0));
}

fn fill_main(coord: vec2<f32>) -> vec4<f32> {
    let v0: vec4<f32> = params.colors[0];
    let v1: vec4<f32> = params.colors[1];
    let v2: vec4<f32> = ph2d_mix_blend(v0, v1, params.scalars[2].x, params.ucontrol[2].x);
    return v2;
}
