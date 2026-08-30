// Colagem de UMA faixa sobre o acumulador do mundo (ADR-0154 Fase 2).
//
// A mistura é de HARDWARE (`One / OneMinusSrcAlpha`) sobre um alvo de formato CRU, o que faz o
// `over` acontecer no espaço do DESENHISTA — a convenção que o compositor deste app já usa.
// ⇒ este shader tem UMA responsabilidade: entregar a cor da faixa em **sRGB pré-multiplicado**,
// venha ela de onde vier.
//
// As duas fontes têm convenções DIFERENTES, e é por isso que os dois interruptores existem:
//   · a faixa de SPRITE vem do tonemap, numa vista `Bgra8UnormSrgb` (a amostragem descodifica ⇒
//     é preciso re-codificar) e já **pré-multiplicada**;
//   · a faixa de VETOR vem do intermediário do Vello, `Rgba8Unorm` (sem descodificação) e com
//     alfa **directa**.
// Ler uma com a convenção da outra dá uma borda escura ou uma cor lavada — nenhuma das duas
// falha alto.

struct Flags {
    // 1 = a amostragem descodificou de sRGB ⇒ re-codificar.
    decode_srgb: u32,
    // 1 = a fonte já é pré-multiplicada.
    premultiplied: u32,
    _pad0: u32,
    _pad1: u32,
};

@group(0) @binding(0) var src: texture_2d<f32>;
@group(0) @binding(1) var src_sampler: sampler;
@group(0) @binding(2) var<uniform> flags: Flags;

struct VsOut {
    @builtin(position) pos: vec4<f32>,
    @location(0) uv: vec2<f32>,
};

@vertex
fn vs_main(@builtin(vertex_index) vi: u32) -> VsOut {
    // Triângulo de tela cheia.
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

@fragment
fn fs_main(in: VsOut) -> @location(0) vec4<f32> {
    var c = textureSample(src, src_sampler, in.uv);
    // ⚠️ A des-pré-multiplicação vem PRIMEIRO: a conversão de espaço é por-canal sobre a cor
    // DIRECTA. Re-codificar uma cor já escalada pelo alfa daria uma curva errada nas bordas.
    if (flags.premultiplied == 1u) {
        let a = max(c.a, 1e-6);
        c = vec4<f32>(c.rgb / a, c.a);
    }
    if (flags.decode_srgb == 1u) {
        c = vec4<f32>(linear_to_srgb(c.rgb), c.a);
    }
    // Sai sempre pré-multiplicado — é o que a mistura `One / OneMinusSrcAlpha` espera.
    return vec4<f32>(c.rgb * c.a, c.a);
}
