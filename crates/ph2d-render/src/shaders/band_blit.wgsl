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
//
// ⭐ A LUZ 2D (W3): com a luz acesa e uma fonte do MUNDO (vetor, formas do Motion), a cor do
// desenhista passa pela MESMA conta do sprite — `linear(albedo) × luz_em → tonemap_mapear → sRGB`
// (`receber.wgsl` e `tonemap_conta.wgsl`, concatenados à frente). Sem luz, o ramo não é tomado.

struct Flags {
    // 1 = a amostragem descodificou de sRGB ⇒ re-codificar.
    decode_srgb: u32,
    // 1 = a fonte já é pré-multiplicada.
    premultiplied: u32,
    // 1 = a fonte é conteúdo do MUNDO e recebe a luz 2D quando acesa (a grade do chrome não).
    recebe_luz: u32,
    _pad1: u32,
};

@group(0) @binding(0) var src: texture_2d<f32>;
@group(0) @binding(1) var src_sampler: sampler;
@group(0) @binding(2) var<uniform> flags: Flags;
// O LUT do tonemap — o MESMO que o passe de tonemap liga (o grupo da luz o carrega).
@group(3) @binding(3) var luz_lut: texture_3d<f32>;
@group(3) @binding(4) var luz_lut_amostra: sampler;

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

fn srgb_to_linear(c: vec3<f32>) -> vec3<f32> {
    let lo = c / 12.92;
    let hi = pow((c + vec3<f32>(0.055)) / 1.055, vec3<f32>(2.4));
    let cutoff = step(vec3<f32>(0.04045), c);
    return mix(lo, hi, cutoff);
}

// A cor do desenhista (sRGB codificado) acesa pela luz no píxel `px` do alvo: a conta do sprite.
fn acender(c: vec4<f32>, px: vec2<f32>) -> vec4<f32> {
    var reta = c;
    if (flags.premultiplied == 1u) {
        reta = vec4<f32>(c.rgb / max(c.a, 1e-6), c.a);
    }
    let lin = srgb_to_linear(reta.rgb) * luz_em(luz_tela_para_mundo(px));
    let m = tonemap_mapear(vec4<f32>(lin * reta.a, reta.a), luz_lut, luz_lut_amostra);
    let s = linear_to_srgb(m.rgb / max(m.a, 1e-6));
    return vec4<f32>(s * m.a, m.a);
}

@fragment
fn fs_main(in: VsOut) -> @location(0) vec4<f32> {
    var c = textureSample(src, src_sampler, in.uv);
    if (flags.recebe_luz == 1u && luz.ligada == 1u) {
        return acender(c, in.pos.xy);
    }
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
