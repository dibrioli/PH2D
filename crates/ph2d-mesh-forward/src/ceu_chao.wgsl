// O CEU QUE O CHAO VE, uma vez por quadro, no enquadramento da cobertura (nao depende da camara).
// Em cada texel do chao: a fraccao do ceu, ponderada pelo cosseno, que as pecas tapam — em cada fatia
// de azimute, o INTERVALO de elevacoes tapado, do fundo das pecas (`b`, a cobertura vista de baixo) ao
// topo (`g`, vista de cima). So' o topo seria um campo de alturas: enche de peca o vao debaixo de uma
// esfera e tapa o ceu que passa por baixo dela.
// Amostras ENTRELACADAS (azimute e passo deslocados por texel numa grelha 4x4) e um borrao 5x5 que as
// junta NO MESMO quadro: nada se acumula entre quadros.

struct P {
    // x = profundidade do chao na cobertura · y = aresta do texel da cobertura (mundo) · z = fundo
    // (mundo) · w = ultimo nivel da cobertura
    a: vec4<f32>,
    // x = fatias por texel · y = passos por fatia · z = razao entre passos · w = lado da cobertura
    b: vec4<f32>,
    // x = a fraccao da borda do quadro onde o escurecimento cai a zero · y = lado desta textura ·
    // z = a pegada lateral de uma amostra, em fraccoes da distancia
    c: vec4<f32>,
};

@group(0) @binding(0) var<uniform> p: P;
// No 1.o passe a cobertura; no borrao, o ceu cru.
@group(0) @binding(1) var fonte: texture_2d<f32>;
@group(0) @binding(2) var liso: sampler;

// Abaixo disto (em degraus da profundidade de 8 bits da cobertura) o fundo de uma peca coberta esta'
// no chao: o texel e' DENTRO dela.
const SOLIDO: f32 = 3.0 / 255.0;

@vertex
fn vs_ceu(@builtin(vertex_index) k: u32) -> @builtin(position) vec4<f32> {
    let q = array<vec2<f32>, 3>(vec2<f32>(-1.0, -1.0), vec2<f32>(3.0, -1.0), vec2<f32>(-1.0, 3.0));
    return vec4<f32>(q[k], 0.0, 1.0);
}

fn bayer(ij: vec2<u32>) -> u32 {
    let m = array<u32, 16>(0u, 8u, 2u, 10u, 12u, 4u, 14u, 6u, 3u, 11u, 1u, 9u, 15u, 7u, 13u, 5u);
    return m[(ij.y & 3u) * 4u + (ij.x & 3u)];
}

fn sen2(h: f32, t: f32) -> f32 {
    return h * h / (h * h + t * t);
}

@fragment
fn fs_ceu(@builtin(position) q: vec4<f32>) -> @location(0) vec4<f32> {
    let uv = q.xy / p.c.y;
    let ij = vec2<u32>(q.xy);
    let zc = p.a.x;
    let texel = p.a.y;
    let fundo = p.a.z;
    let topo = p.a.w;
    let fatias = u32(p.b.x);
    let passos = u32(p.b.y);
    let razao = p.b.z;
    let lado = p.b.w;
    let a0 = textureSampleLevel(fonte, liso, uv, 0.0);
    if (a0.r > 0.5 && zc - a0.b / a0.r < SOLIDO) {
        return vec4<f32>(0.0);
    }
    let dphi = 6.2831853 / f32(fatias);
    let desvio = (f32(bayer(ij)) + 0.5) / 16.0;
    let t0 = texel * pow(razao, (f32(bayer(ij.yx)) + 0.5) / 16.0);
    var occ = 0.0;
    for (var d = 0u; d < fatias; d = d + 1u) {
        let ang = (f32(d) + desvio) * dphi;
        let dir = vec2<f32>(cos(ang), sin(ang));
        var lo = 1.0;
        var hi = 0.0;
        var w = 0.0;
        var t = t0;
        for (var k = 0u; k < passos; k = k + 1u) {
            let s = uv + dir * (t / (texel * lado));
            let lod = clamp(log2(max(t * p.c.z / texel, 1.0)), 0.0, topo);
            let a = textureSampleLevel(fonte, liso, s, lod);
            if (a.r > 1.0e-3) {
                hi = max(hi, sen2(max(zc - a.g / a.r, 0.0) * fundo, t));
                lo = min(lo, sen2(max(zc - a.b / a.r, 0.0) * fundo, t));
                w = max(w, clamp(a.r, 0.0, 1.0));
            }
            t = t * razao;
        }
        occ = occ + w * max(hi - lo, 0.0);
    }
    // A borda do quadro: o que falta da cauda cai a zero, sem degrau.
    let borda = 1.0 - max(abs(2.0 * uv.x - 1.0), abs(2.0 * uv.y - 1.0));
    occ = occ * clamp(borda / p.c.x, 0.0, 1.0);
    return vec4<f32>(1.0 - occ / f32(fatias), 1.0, 0.0, 1.0);
}

// O borrao 5x5 (pesos 1/2 nas bordas: cada uma das 16 posicoes da grelha pesa o mesmo), so' com os
// texels de fora das pecas. Sai PRE-MULTIPLICADO pela validade: o chao filtra (r, g) e divide.
@fragment
fn fs_borra(@builtin(position) q: vec4<f32>) -> @location(0) vec4<f32> {
    let c = vec2<i32>(q.xy);
    let lim = vec2<i32>(textureDimensions(fonte)) - vec2<i32>(1);
    let pesos = array<f32, 5>(0.5, 1.0, 1.0, 1.0, 0.5);
    var s = vec2<f32>(0.0);
    for (var j = 0; j < 5; j = j + 1) {
        for (var i = 0; i < 5; i = i + 1) {
            let v = textureLoad(fonte, clamp(c + vec2<i32>(i - 2, j - 2), vec2<i32>(0), lim), 0);
            s = s + pesos[i] * pesos[j] * v.g * vec2<f32>(v.r, 1.0);
        }
    }
    return vec4<f32>(s / 16.0, 0.0, 1.0);
}
