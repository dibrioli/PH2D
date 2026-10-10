// A LENTE — as leis da `ph2d-lens` em WGSL, função a função e na MESMA ordem de operações
// (doc 129 §3.2). Só funções puras: o passe (`lens.wgsl`) e a sonda do hash dos testes
// compõem este texto. Prefixo `lz_`. Pede o `pe_*` da `ph2d_ease::wgsl_lib!` e o `pc_*` da
// `ph2d_color::wgsl_perceptual!` antes dele.

// `ph2d_color::srgb::srgb_to_linear_unit`.
fn lz_s2l(v: f32) -> f32 {
    if (v <= 0.04045) {
        return v / 12.92;
    }
    return pow((v + 0.055) / 1.055, 2.4);
}

// `ph2d_color::srgb::linear_to_srgb_unit` (corta em `[0, 1]`).
fn lz_l2s(x: f32) -> f32 {
    let v = clamp(x, 0.0, 1.0);
    if (v <= 0.0031308) {
        return v * 12.92;
    }
    return 1.055 * pow(v, 1.0 / 2.4) - 0.055;
}

fn lz_s2l3(c: vec3<f32>) -> vec3<f32> {
    return vec3<f32>(lz_s2l(c.x), lz_s2l(c.y), lz_s2l(c.z));
}

fn lz_l2s3(c: vec3<f32>) -> vec3<f32> {
    return vec3<f32>(lz_l2s(c.x), lz_l2s(c.y), lz_l2s(c.z));
}

// ── vinheta (`ph2d_lens::vinheta`) ───────────────────────────────────────────

// `Φ` de Abramowitz & Stegun 7.1.26.
fn lz_phi(t: f32) -> f32 {
    let z = abs(t) * bitcast<f32>(0x3f3504f3u);
    let k = 1.0 / (1.0 + 0.3275911 * z);
    let poli = k * (0.2548296 + k * (-0.28449674 + k * (1.4214137 + k * (-1.4531521 + k * 1.0614054))));
    let erf = 1.0 - poli * exp(-z * z);
    if (t >= 0.0) {
        return 0.5 + 0.5 * erf;
    }
    return 0.5 - 0.5 * erf;
}

fn lz_meia_largura(y: f32, semi: vec2<f32>, expoente: f32) -> f32 {
    let t = abs(y / semi.y);
    if (t >= 1.0) {
        return 0.0;
    }
    return semi.x * pow(1.0 - pow(t, expoente), 1.0 / expoente);
}

// A máscara: a superelipse convolvida com a gaussiana, quadratura do ponto médio de 16 nós.
fn lz_mascara(q: vec2<f32>, semi: vec2<f32>, expoente: f32, sigma: f32) -> f32 {
    if (semi.x <= 0.0 || semi.y <= 0.0) {
        return 0.0;
    }
    let lo = max(-semi.y, q.y - 5.0 * sigma);
    let hi = min(semi.y, q.y + 5.0 * sigma);
    if (hi <= lo) {
        return 0.0;
    }
    let passo = (hi - lo) / 16.0;
    var soma = 0.0;
    for (var k = 0u; k < 16u; k = k + 1u) {
        let yl = lo + (f32(k) + 0.5) * passo;
        let h = lz_meia_largura(yl, semi, expoente);
        let d = (q.y - yl) / sigma;
        soma = soma + exp(-0.5 * d * d) * (lz_phi((h - q.x) / sigma) - lz_phi((-h - q.x) / sigma));
    }
    // `√(2π)` com os bits do `(2.0 * PI).sqrt()` da CPU.
    return soma * passo / (sigma * bitcast<f32>(0x40206c99u));
}

// ── grão (`ph2d_lens::grao`) ─────────────────────────────────────────────────

// `pcg3d` de Jarzynski & Olano (JCGT 2020): só inteiros de 32 bits ⇒ o MESMO bit da CPU.
fn lz_pcg3d(v_in: vec3<u32>) -> vec3<u32> {
    var v = v_in * 1664525u + vec3<u32>(1013904223u);
    v.x = v.x + v.y * v.z;
    v.y = v.y + v.z * v.x;
    v.z = v.z + v.x * v.y;
    v = v ^ (v >> vec3<u32>(16u));
    v.x = v.x + v.y * v.z;
    v.y = v.y + v.z * v.x;
    v.z = v.z + v.x * v.y;
    return v;
}

// ~N(0,1) por célula: 4 uniformes de 23 bits somadas em INTEIRO (exacta num f32 depois de
// centrada), um só arredondamento — o da multiplicação por `√3 / 2²³`.
fn lz_gauss_celula(cel: vec2<i32>, semente: u32, canal: u32) -> f32 {
    let a = lz_pcg3d(vec3<u32>(bitcast<u32>(cel.x), bitcast<u32>(cel.y), semente ^ (canal * 0x9E3779B9u)));
    let b = lz_pcg3d(vec3<u32>(a.x, a.y, a.z ^ 0x85EBCA6Bu));
    let soma = (a.x >> 9u) + (a.y >> 9u) + (a.z >> 9u) + (b.x >> 9u);
    return f32(i32(soma) - 16777214) * bitcast<f32>(0x345db3d7u);
}

fn lz_ruido(p: vec2<f32>, size: f32, semente: u32, canal: u32) -> f32 {
    let s = max(size, 1.0);
    let q = vec2<f32>(p.x / s - 0.5, p.y / s - 0.5);
    let i = floor(q);
    let t = q - i;
    let f = t * t * (vec2<f32>(3.0) - 2.0 * t);
    let c = vec2<i32>(i);
    let g00 = lz_gauss_celula(c, semente, canal);
    let g10 = lz_gauss_celula(c + vec2<i32>(1, 0), semente, canal);
    let g01 = lz_gauss_celula(c + vec2<i32>(0, 1), semente, canal);
    let g11 = lz_gauss_celula(c + vec2<i32>(1, 1), semente, canal);
    let topo = g00 + (g10 - g00) * f.x;
    let base = g01 + (g11 - g01) * f.x;
    return topo + (base - topo) * f.y;
}

// O grão sobre uma cor CODIFICADA. `k0 = amount × σ por amount`.
fn lz_grao(cod: vec3<f32>, p: vec2<f32>, k0: f32, size: f32, response: f32, colorido: bool, semente: u32) -> vec3<f32> {
    let l = clamp(0.2126 * cod.x + 0.7152 * cod.y + 0.0722 * cod.z, 0.0, 1.0);
    let peso = 1.0 + (4.0 * l * (1.0 - l) - 1.0) * clamp(response, 0.0, 1.0);
    let k = k0 * peso;
    if (colorido) {
        return vec3<f32>(
            cod.x + k * lz_ruido(p, size, semente, 0u),
            cod.y + k * lz_ruido(p, size, semente, 1u),
            cod.z + k * lz_ruido(p, size, semente, 2u),
        );
    }
    let n = lz_ruido(p, size, semente, 0u);
    return cod + vec3<f32>(k * n);
}
