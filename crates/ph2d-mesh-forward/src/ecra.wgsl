// ── A CODIFICACAO: o `premultiplicado::para_ecra` do Render, pixel a pixel — e o BRILHO por cima ──
// O halo e' o ultimo degrau da cadeia (a tenda que sobe ao quadro e a cor), somado pela lei da casa
// (`bl_compoe`) DEPOIS do olhar, com a mesma regra sobre a peca e sobre o fundo.
struct Ecra {
    // x, y = o lado do degrau de origem (o acumulado do nivel 0) · z, w = o quadro
    dims: vec4<u32>,
    // a base da tenda (`BloomParams::upsample_basis`)
    base: vec4<f32>,
    // x = saturacao · y = intensidade · z = exposicao (stops) · w = ha' brilho (0/1)
    cor: vec4<f32>,
    // xyz = tinta · w = codigo da vista
    tinta: vec4<f32>,
};

@group(0) @binding(0) var resolvida: texture_2d<f32>;
@group(0) @binding(1) var acumulado: texture_2d<f32>;
@group(0) @binding(2) var<uniform> E: Ecra;

fn bl_le(i: u32) -> vec3<f32> {
    let w = E.dims.x;
    return textureLoad(acumulado, vec2<i32>(i32(i % w), i32(i / w)), 0).xyz;
}

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

fn byte(x: f32) -> u32 {
    return u32(floor(clamp(x, 0.0, 1.0) * 255.0 + 0.5));
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
    var px = vec4<u32>(byte(srgb(c.r * inv) * a), byte(srgb(c.g * inv) * a), byte(srgb(c.b * inv) * a), u32(a8));
    if (E.cor.w != 0.0) {
        let t = bl_sobe_tenda(E.dims.x, E.dims.y, E.dims.z, E.dims.w, u32(q.x), u32(q.y), E.base);
        let halo = bl_cor(t, E.cor.x, E.tinta.xyz, E.cor.y);
        px = bl_compoe(px, vt_to_display(halo, E.cor.z, u32(E.tinta.w)));
    }
    return vec4<f32>(px) / 255.0;
}
