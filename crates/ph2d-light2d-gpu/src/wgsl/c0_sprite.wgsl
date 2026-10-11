// C0 — os SPRITES por texel (W3). Mesmo molde das formas: a CAIXA do sprite na grade, e o fragmento
// mede a cobertura do texel nas MESMAS 8×8 subamostras — aqui a média do alfa da textura em cada
// subamostra que cai dentro do quad (o alfa do texel É a cobertura, handoff W2). A emissão do dono é
// a cor média (ponderada pelo alfa) × o multiplicador do sprite. A lei do dono vem de `c0_dono.wgsl`.

struct Grade {
    tamanho: vec2<f32>,
    _pad: vec2<f32>,
}

struct FormaSprite {
    caixa: vec4<f32>,
    // Texel → quad: `s = s_.x·px + s_.y·py + s_.z`, idem `t` (o quad é `[0, 1]²`, `(0,0)` em cima à
    // esquerda, como o `quad_uv` do sprite).
    s_: vec4<f32>,
    t_: vec4<f32>,
    // O UV em `(s, t) = (0, 0)` (xy) e `(1, 1)` (zw), com as inversões já aplicadas.
    uv: vec4<f32>,
    // rgb = multiplicador da cor do texel na emissão (0 = não emite), a = 1/alcance.
    cor: vec4<f32>,
    opacidade: f32,
    premultiplicada: f32,
    // ⭐ W5c: a textura guarda a cor em sRGB (a camada do vetor, `Rgba8Unorm`): o albedo linear sai
    // pela curva; a de um sprite (`Rgba8UnormSrgb`) já chega linear.
    srgb: f32,
    // ⭐ W5d: o alfa já é ÁREA (lida exata, a camada do vetor): a cobertura é a média dele.
    alfa_e_area: f32,
    // ⭐ W5c: o multiplicador da cor do texel no albedo do rebote (zero = não reflete).
    albedo: vec4<f32>,
}

@group(0) @binding(0) var<uniform> grade: Grade;
@group(0) @binding(1) var<storage, read> sprites: array<FormaSprite>;
@group(1) @binding(0) var textura: texture_2d<f32>;
@group(1) @binding(1) var amostra: sampler;

const SUB: u32 = 8u;
// ⭐ W5d — a COBERTURA é a fração das subamostras com alfa ≥ ½ (a área da forma alfa ≥ ½), não a média
// do alfa: a borda exata da C3 (`c3_borda.wgsl`) lê a cobertura como ÁREA, e a média do alfa de uma
// textura de borda macia (a rampa do disco do atlas, ampliada) não é área de nada — a borda saía em
// zigue-zague (o disco de imagem: o excesso de variação +2 → +47). A borda «dentro» (cobertura ≥ ½)
// fica onde estava; a cor continua ponderada pelo alfa. A camada do vetor (lida exata, um píxel por
// texel) já traz a ÁREA no alfa: nela, o limiar faria da cobertura 0 ou 1 (a escada dos texels).
override COBERTURA_PELO_MEIO: bool = false;

struct Saida {
    @builtin(position) pos: vec4<f32>,
    @location(0) @interpolate(flat) forma: u32,
}

@vertex
fn vs(@builtin(vertex_index) v: u32, @builtin(instance_index) i: u32) -> Saida {
    let c = sprites[i].caixa;
    let p = mix(c.xy, c.zw, vec2<f32>(f32(v & 1u), f32(v >> 1u)));
    var o: Saida;
    o.pos = vec4<f32>(p.x / grade.tamanho.x * 2.0 - 1.0, 1.0 - p.y / grade.tamanho.y * 2.0, 0.0, 1.0);
    o.forma = i;
    return o;
}

@fragment
fn fs(e: Saida) -> Alvos {
    let f = sprites[e.forma];
    var soma_a = 0.0;
    var soma_meio = 0.0;
    var soma_rgb = vec3<f32>(0.0);
    for (var j = 0u; j < SUB; j++) {
        for (var i = 0u; i < SUB; i++) {
            let q = e.pos.xy - 0.5 + (vec2<f32>(f32(i), f32(j)) + 0.5) / f32(SUB);
            let st = vec2<f32>(dot(f.s_.xy, q) + f.s_.z, dot(f.t_.xy, q) + f.t_.z);
            if all(st >= vec2<f32>(0.0)) && all(st < vec2<f32>(1.0)) {
                let c = textureSampleLevel(textura, amostra, mix(f.uv.xy, f.uv.zw, st), 0.0);
                let reta = select(c.rgb, c.rgb / max(c.a, 1e-6), f.premultiplicada > 0.5);
                soma_a += c.a;
                soma_meio += select(0.0, 1.0, c.a >= 0.5);
                soma_rgb += reta * c.a;
            }
        }
    }
    let c = select(soma_a, soma_meio, COBERTURA_PELO_MEIO && f.alfa_e_area < 0.5) / f32(SUB * SUB);
    if c <= 0.0 {
        discard;
    }
    let reta = soma_rgb / max(soma_a, 1e-6);
    let cor = vec4<f32>(reta * f.cor.rgb, f.cor.a);
    var linear = reta;
    if f.srgb > 0.5 {
        linear = select(pow((reta + 0.055) / 1.055, vec3<f32>(2.4)), reta / 12.92, reta <= vec3<f32>(0.04045));
    }
    return c0_alvos(c, cor, f.opacidade, linear * f.albedo.rgb);
}
