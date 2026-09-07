// ⭐⭐⭐ **O VIDRO JATEADO** — o borrão que fica ENTRE o mundo e a receita aberta (Enio,
// 2026-09-07: *«crie a feature de borrar discretamente o que está por trás do prefab como um
// vidro jateado»*).
//
// Três estágios, uma responsabilidade cada:
//
//   mundo (tela cheia) --down--> a (metade) --blur H--> b --blur V--> a --up--> mundo
//
// ⚠️ **Tudo isto acontece no espaço do DESENHISTA** (valores já codificados em sRGB), porque é
// nele que o acumulador do mundo guarda e é nele que o compositor deste app mistura. Um borrão
// feito em luz linear no meio de uma cadeia que compõe em sRGB mudaria a cor de TODAS as bordas —
// a mesma razão pela qual o `WorldRt` é `Bgra8Unorm` e não a variante sRGB.
//
// ⚠️ **A meia resolução não é uma economia, é metade do borrão.** Ela duplica o alcance efectivo
// do mesmo kernel de 5 taps e apaga o padrão de amostragem que um kernel curto deixa em texto
// fino. O custo cai a um quarto ao mesmo tempo, o que é consequência, não motivo.
//
// ⚠️ **Sem transcendentais no laço** (HR-5): os pesos do kernel são constantes.

struct Frost {
    // O passo de amostragem, em UV. No `down` é o texel da fonte; no `blur` é a DIRECÇÃO do
    // percurso (só um dos dois componentes é != 0).
    step: vec2<f32>,
    _pad: vec2<f32>,
    // O véu, em LUZ LINEAR com a força no alfa — convertido aqui, como o `WorldRt::clear_linear`.
    veil: vec4<f32>,
};

@group(0) @binding(0) var src: texture_2d<f32>;
@group(0) @binding(1) var src_sampler: sampler;
@group(0) @binding(2) var<uniform> u: Frost;

struct VsOut {
    @builtin(position) pos: vec4<f32>,
    @location(0) uv: vec2<f32>,
};

@vertex
fn vs_main(@builtin(vertex_index) vi: u32) -> VsOut {
    var out: VsOut;
    let x = f32((vi << 1u) & 2u);
    let y = f32(vi & 2u);
    out.uv = vec2<f32>(x, y);
    out.pos = vec4<f32>(x * 2.0 - 1.0, 1.0 - y * 2.0, 0.0, 1.0);
    return out;
}

fn linear_to_srgb(c: vec3<f32>) -> vec3<f32> {
    let safe = clamp(c, vec3<f32>(0.0), vec3<f32>(1.0));
    let lo = safe * 12.92;
    let hi = 1.055 * pow(safe, vec3<f32>(1.0 / 2.4)) - vec3<f32>(0.055);
    let cutoff = step(vec3<f32>(0.0031308), safe);
    return mix(lo, hi, cutoff);
}

// **Meia resolução, com a média das QUATRO células** que cada texel novo cobre. Um tap bilinear
// só acertaria a média se caísse exactamente no canto partilhado — e meio texel de erro num
// downsample vira cintilação quando a câmera anda.
@fragment
fn fs_down(in: VsOut) -> @location(0) vec4<f32> {
    let o = u.step * 0.5;
    var c = textureSample(src, src_sampler, in.uv + vec2<f32>(-o.x, -o.y));
    c += textureSample(src, src_sampler, in.uv + vec2<f32>(o.x, -o.y));
    c += textureSample(src, src_sampler, in.uv + vec2<f32>(-o.x, o.y));
    c += textureSample(src, src_sampler, in.uv + vec2<f32>(o.x, o.y));
    return c * 0.25;
}

// **A Gaussiana separável, em 5 taps com amostragem LINEAR.** Os dois pares de fora caem ENTRE
// texels de propósito: o filtro bilinear soma dois vizinhos de graça, e um kernel de 9 pesos sai
// em 5 leituras. Os offsets e os pesos são os canónicos desta técnica (σ ≈ 2,6 texels).
const W0: f32 = 0.2270270270;
const W1: f32 = 0.3162162162;
const W2: f32 = 0.0702702703;
const O1: f32 = 1.3846153846;
const O2: f32 = 3.2307692308;

@fragment
fn fs_blur(in: VsOut) -> @location(0) vec4<f32> {
    var c = textureSample(src, src_sampler, in.uv) * W0;
    c += textureSample(src, src_sampler, in.uv + u.step * O1) * W1;
    c += textureSample(src, src_sampler, in.uv - u.step * O1) * W1;
    c += textureSample(src, src_sampler, in.uv + u.step * O2) * W2;
    c += textureSample(src, src_sampler, in.uv - u.step * O2) * W2;
    return c;
}

// **De volta à tela cheia, com o VÉU.** O véu é o que separa *«está desfocado»* de *«está atrás do
// vidro»*: sem ele um fundo de contraste baixo fica só ligeiramente mole, e a receita não ganha o
// degrau de leitura que a põe à frente.
@fragment
fn fs_up(in: VsOut) -> @location(0) vec4<f32> {
    let blurred = textureSample(src, src_sampler, in.uv);
    let veil = linear_to_srgb(u.veil.rgb);
    return vec4<f32>(mix(blurred.rgb, veil, u.veil.a), 1.0);
}
