//! WGSL kernel sources for the sharpen compute pipelines. Inline raw
//! strings (no `include_str!`) so they ship in the binary with no
//! runtime file dependency. `pub(super)` — consumed by `laplacian` /
//! `unsharp` siblings via `super::wgsl::*`.

/// 3×3 Laplacian in **linear sRGB** (Tier 3 audit parity). Decodes
/// center + 4 neighbours from sRGB to linear, applies the 5-cross kernel
/// `5·center − top − bottom − left − right`, blends
/// `center + (lap − center) · amount`, and encodes back to sRGB. Edge
/// pixels clamp the neighbour index (mirrors CPU `if y > 0 { ... } else
/// { center }`).
pub(super) const LAPLACIAN_WGSL: &str = r#"
struct Uniforms {
    amount: f32,
    _pad0: f32,
    _pad1: f32,
    _pad2: f32,
};

@group(0) @binding(0) var input_tex: texture_2d<f32>;
@group(0) @binding(1) var output_tex: texture_storage_2d<rgba8unorm, write>;
@group(0) @binding(2) var<uniform> u: Uniforms;

fn srgb_to_linear_c(c: f32) -> f32 {
    if (c <= 0.04045) {
        return c / 12.92;
    }
    return pow((c + 0.055) / 1.055, 2.4);
}

fn linear_to_srgb_c(c: f32) -> f32 {
    let c_clamped = max(c, 0.0);
    if (c_clamped <= 0.0031308) {
        return c_clamped * 12.92;
    }
    return 1.055 * pow(c_clamped, 1.0 / 2.4) - 0.055;
}

fn s2l(rgb: vec3<f32>) -> vec3<f32> {
    return vec3<f32>(srgb_to_linear_c(rgb.r), srgb_to_linear_c(rgb.g), srgb_to_linear_c(rgb.b));
}

fn l2s(rgb: vec3<f32>) -> vec3<f32> {
    return vec3<f32>(linear_to_srgb_c(rgb.r), linear_to_srgb_c(rgb.g), linear_to_srgb_c(rgb.b));
}

@compute @workgroup_size(8, 8, 1)
fn main(@builtin(global_invocation_id) id: vec3<u32>) {
    let dims = textureDimensions(output_tex);
    if (id.x >= dims.x || id.y >= dims.y) {
        return;
    }
    let coord = vec2<i32>(i32(id.x), i32(id.y));
    let dims_i = vec2<i32>(i32(dims.x), i32(dims.y));
    let center_srgb = textureLoad(input_tex, coord, 0);
    if (center_srgb.a == 0.0) {
        textureStore(output_tex, coord, center_srgb);
        return;
    }
    let top_coord    = vec2<i32>(coord.x, max(coord.y - 1, 0));
    let bottom_coord = vec2<i32>(coord.x, min(coord.y + 1, dims_i.y - 1));
    let left_coord   = vec2<i32>(max(coord.x - 1, 0), coord.y);
    let right_coord  = vec2<i32>(min(coord.x + 1, dims_i.x - 1), coord.y);
    let center = s2l(center_srgb.rgb);
    let top    = s2l(textureLoad(input_tex, top_coord, 0).rgb);
    let bottom = s2l(textureLoad(input_tex, bottom_coord, 0).rgb);
    let left   = s2l(textureLoad(input_tex, left_coord, 0).rgb);
    let right  = s2l(textureLoad(input_tex, right_coord, 0).rgb);

    let laplacian = 5.0 * center - top - bottom - left - right;
    let result_lin = center + (laplacian - center) * u.amount;
    let result_srgb = clamp(l2s(result_lin), vec3<f32>(0.0), vec3<f32>(1.0));
    textureStore(output_tex, coord, vec4<f32>(result_srgb, center_srgb.a));
}
"#;

/// Unsharp H pass in **linear sRGB**: decodes each sampled sRGB pixel to
/// linear, blurs in linear with the Gaussian kernel, writes the linear
/// blur into the `rgba16float` intermediate (which has the headroom to
/// store unclamped linear values). The V pass also reads linear and
/// finishes with the combine + sRGB encode.
pub(super) const UNSHARP_H_WGSL: &str = r#"
struct Uniforms {
    amount: f32,
    half: i32,
    kernel_size: i32,
    _pad: i32,
};

@group(0) @binding(0) var input_tex: texture_2d<f32>;
@group(0) @binding(1) var h_pass: texture_storage_2d<rgba16float, write>;
@group(0) @binding(2) var<uniform> u: Uniforms;
@group(0) @binding(3) var<storage, read> kernel: array<f32>;

fn srgb_to_linear_c(c: f32) -> f32 {
    if (c <= 0.04045) {
        return c / 12.92;
    }
    return pow((c + 0.055) / 1.055, 2.4);
}

fn s2l(rgb: vec3<f32>) -> vec3<f32> {
    return vec3<f32>(srgb_to_linear_c(rgb.r), srgb_to_linear_c(rgb.g), srgb_to_linear_c(rgb.b));
}

@compute @workgroup_size(8, 8, 1)
fn main(@builtin(global_invocation_id) id: vec3<u32>) {
    let dims = textureDimensions(h_pass);
    if (id.x >= dims.x || id.y >= dims.y) {
        return;
    }
    let coord = vec2<i32>(i32(id.x), i32(id.y));
    let dims_i = vec2<i32>(i32(dims.x), i32(dims.y));
    let center = textureLoad(input_tex, coord, 0);

    var sum_rgb = vec3<f32>(0.0);
    var wt = 0.0;
    for (var k: i32 = 0; k < u.kernel_size; k = k + 1) {
        let sx = clamp(coord.x + k - u.half, 0, dims_i.x - 1);
        let sample_lin = s2l(textureLoad(input_tex, vec2<i32>(sx, coord.y), 0).rgb);
        let kw = kernel[k];
        sum_rgb = sum_rgb + sample_lin * kw;
        wt = wt + kw;
    }
    let blur_lin = sum_rgb / wt;
    textureStore(h_pass, coord, vec4<f32>(blur_lin, center.a));
}
"#;

/// Unsharp V + combine pass in **linear sRGB**: blurs the H-pass
/// intermediate (already linear) along the Y axis, decodes the original
/// sRGB sample to linear, combines `orig_lin + amount·(orig_lin −
/// blur_lin)`, encodes the result back to sRGB. Transparent pixels pass
/// through untouched.
pub(super) const UNSHARP_V_WGSL: &str = r#"
struct Uniforms {
    amount: f32,
    half: i32,
    kernel_size: i32,
    _pad: i32,
};

@group(0) @binding(0) var input_tex: texture_2d<f32>;
@group(0) @binding(1) var h_pass: texture_2d<f32>;
@group(0) @binding(2) var output_tex: texture_storage_2d<rgba8unorm, write>;
@group(0) @binding(3) var<uniform> u: Uniforms;
@group(0) @binding(4) var<storage, read> kernel: array<f32>;

fn srgb_to_linear_c(c: f32) -> f32 {
    if (c <= 0.04045) {
        return c / 12.92;
    }
    return pow((c + 0.055) / 1.055, 2.4);
}

fn linear_to_srgb_c(c: f32) -> f32 {
    let c_clamped = max(c, 0.0);
    if (c_clamped <= 0.0031308) {
        return c_clamped * 12.92;
    }
    return 1.055 * pow(c_clamped, 1.0 / 2.4) - 0.055;
}

fn s2l(rgb: vec3<f32>) -> vec3<f32> {
    return vec3<f32>(srgb_to_linear_c(rgb.r), srgb_to_linear_c(rgb.g), srgb_to_linear_c(rgb.b));
}

fn l2s(rgb: vec3<f32>) -> vec3<f32> {
    return vec3<f32>(linear_to_srgb_c(rgb.r), linear_to_srgb_c(rgb.g), linear_to_srgb_c(rgb.b));
}

@compute @workgroup_size(8, 8, 1)
fn main(@builtin(global_invocation_id) id: vec3<u32>) {
    let dims = textureDimensions(output_tex);
    if (id.x >= dims.x || id.y >= dims.y) {
        return;
    }
    let coord = vec2<i32>(i32(id.x), i32(id.y));
    let dims_i = vec2<i32>(i32(dims.x), i32(dims.y));
    let orig_srgb = textureLoad(input_tex, coord, 0);
    if (orig_srgb.a == 0.0) {
        textureStore(output_tex, coord, orig_srgb);
        return;
    }

    var sum_rgb = vec3<f32>(0.0);
    var wt = 0.0;
    for (var k: i32 = 0; k < u.kernel_size; k = k + 1) {
        let sy = clamp(coord.y + k - u.half, 0, dims_i.y - 1);
        let sample = textureLoad(h_pass, vec2<i32>(coord.x, sy), 0).rgb;
        let kw = kernel[k];
        sum_rgb = sum_rgb + sample * kw;
        wt = wt + kw;
    }
    let blur_lin = sum_rgb / wt;
    let orig_lin = s2l(orig_srgb.rgb);
    let diff = orig_lin - blur_lin;
    let result_lin = orig_lin + u.amount * diff;
    let result_srgb = clamp(l2s(result_lin), vec3<f32>(0.0), vec3<f32>(1.0));
    textureStore(output_tex, coord, vec4<f32>(result_srgb, orig_srgb.a));
}
"#;
