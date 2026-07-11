// T1.7 — as duas passagens de espaço-de-cor que ligam o rasterizador do Flip
// (premult, linear, 16F) ao `LayerCompositor` do Painter (straight, sRGB8):
//
//   fs_resolve : 16F premult linear  →  Rgba8Unorm straight sRGB8   (fatia da camada)
//   fs_blit    : Rgba8Unorm straight sRGB8 (saída do compositor)  →  16F premult linear
//
// Ambas são 1:1 em pixel (fullscreen triângulo + `textureLoad` pela coord de
// fragmento), então não há filtragem nem flip de Y — a fatia da camada, o slot
// do compositor e o `game_rt` compartilham a MESMA grade de framebuffer.
//
// As transferências sRGB são **byte-idênticas** às de `ph2d-render`
// (`layer_composite.wgsl` / `ph2d_color::srgb`): a fatia que escrevo aqui é
// re-decodificada pela LUT do compositor, então os literais precisam casar.

struct VsOut {
    @builtin(position) pos: vec4<f32>,
};

// Triângulo único cobrindo o viewport: (-1,-1), (3,-1), (-1,3).
@vertex
fn vs_fullscreen(@builtin(vertex_index) vid: u32) -> VsOut {
    var out: VsOut;
    let x = f32((vid << 1u) & 2u) * 2.0 - 1.0;
    let y = f32(vid & 2u) * 2.0 - 1.0;
    out.pos = vec4<f32>(x, y, 0.0, 1.0);
    return out;
}

@group(0) @binding(0) var src: texture_2d<f32>;

// Literais idênticos a `ph2d-render::layer_composite.wgsl`.
fn linear_to_srgb(v: f32) -> f32 {
    let c = clamp(v, 0.0, 1.0);
    if c <= 0.0031308 {
        return c * 12.92;
    }
    return 1.055 * pow(c, 1.0 / 2.4) - 0.055;
}

fn srgb_to_linear(v: f32) -> f32 {
    let x = clamp(v, 0.0, 1.0);
    if x <= 0.04045 {
        return x / 12.92;
    }
    return pow((x + 0.055) / 1.055, 2.4);
}

// A camada rasterizada (premult linear, 16F) → straight sRGB8, a fatia que o
// compositor espera (rgb straight sRGB-encoded, a = cobertura).
@fragment
fn fs_resolve(in: VsOut) -> @location(0) vec4<f32> {
    let p = textureLoad(src, vec2<i32>(in.pos.xy), 0); // premult linear
    let a = p.a;
    var lin = vec3<f32>(0.0, 0.0, 0.0);
    if a > 0.0 {
        lin = p.rgb / a; // un-premultiply
    }
    return vec4<f32>(
        linear_to_srgb(lin.r),
        linear_to_srgb(lin.g),
        linear_to_srgb(lin.b),
        a,
    );
}

// A saída do compositor (straight sRGB8) → 16F premult linear, componível sobre
// o `game_rt` com o blend premult-over (One, OneMinusSrcAlpha) do pipeline.
@fragment
fn fs_blit(in: VsOut) -> @location(0) vec4<f32> {
    let s = textureLoad(src, vec2<i32>(in.pos.xy), 0); // straight sRGB
    let lin = vec3<f32>(
        srgb_to_linear(s.r),
        srgb_to_linear(s.g),
        srgb_to_linear(s.b),
    );
    let a = s.a;
    return vec4<f32>(lin * a, a); // premult
}
