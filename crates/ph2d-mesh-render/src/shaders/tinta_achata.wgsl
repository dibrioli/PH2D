// ⭐⭐⭐ O COMPOSTO DAS CAMADAS ACHATADO NO PLANO DE TINTA (docs/3D/30 §13, W1b).
//
// A mesma lei de `ph2d_app_sculpt3d::pilha_da_peca::achata`, amostra a amostra:
// onde o composto é opaco a cor é EXACTAMENTE `byte / 255` (a tabela vem da
// CPU); onde não é, a mistura é em LUZ sobre o fundo (que chega já linear, da
// CPU) e volta a sRGB pela curva IEC 61966 de `ph2d_color::srgb`.
//
// O composto é a dobra `largura × ⌈N/largura⌉` do compositor de camadas do
// Painter (Rgba8Unorm, sRGB direito); a amostra `i` mora em `(i % largura,
// i / largura)`.

struct Cfg {
    n: u32,
    largura: u32,
    // Quantas amostras uma linha de grupos cobre (`grupos_x · 256`).
    passo_y: u32,
    _p: u32,
}

@group(0) @binding(0) var composto: texture_2d<f32>;
@group(0) @binding(1) var<storage, read> fundo: array<f32>;
@group(0) @binding(2) var<storage, read_write> amostras: array<f32>;
// [0, 256): byte / 255 · [256, 512): srgb_to_linear_byte(byte).
@group(0) @binding(3) var<storage, read> tabela: array<f32>;
@group(0) @binding(4) var<uniform> cfg: Cfg;

// O byte de um texel Rgba8Unorm — recuperado como o compositor o recupera.
fn byte_de(v: f32) -> u32 {
    return u32(v * 255.0 + 0.5);
}

fn para_srgb(linear: f32) -> f32 {
    let v = clamp(linear, 0.0, 1.0);
    if v <= 0.0031308 {
        return v * 12.92;
    }
    return 1.055 * pow(v, 1.0 / 2.4) - 0.055;
}

@compute @workgroup_size(256)
fn cs_achata(@builtin(global_invocation_id) gid: vec3<u32>) {
    let i = gid.y * cfg.passo_y + gid.x;
    if i >= cfg.n {
        return;
    }
    let px = textureLoad(composto, vec2<i32>(i32(i % cfg.largura), i32(i / cfg.largura)), 0);
    let r = byte_de(px.r);
    let g = byte_de(px.g);
    let b = byte_de(px.b);
    let a = byte_de(px.a);
    var cor: vec3<f32>;
    if a == 255u {
        cor = vec3<f32>(tabela[r], tabela[g], tabela[b]);
    } else {
        let al = tabela[a];
        let f = vec3<f32>(fundo[3u * i], fundo[3u * i + 1u], fundo[3u * i + 2u]);
        let luz = vec3<f32>(tabela[256u + r], tabela[256u + g], tabela[256u + b]) * al
            + f * (1.0 - al);
        cor = vec3<f32>(para_srgb(luz.x), para_srgb(luz.y), para_srgb(luz.z));
    }
    amostras[3u * i] = cor.x;
    amostras[3u * i + 1u] = cor.y;
    amostras[3u * i + 2u] = cor.z;
}
