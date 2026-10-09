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
    _p0: f32,
    _p1: f32,
}

@group(0) @binding(0) var<uniform> grade: Grade;
@group(0) @binding(1) var<storage, read> sprites: array<FormaSprite>;
@group(1) @binding(0) var textura: texture_2d<f32>;
@group(1) @binding(1) var amostra: sampler;

const SUB: u32 = 8u;

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
    var soma_rgb = vec3<f32>(0.0);
    for (var j = 0u; j < SUB; j++) {
        for (var i = 0u; i < SUB; i++) {
            let q = e.pos.xy - 0.5 + (vec2<f32>(f32(i), f32(j)) + 0.5) / f32(SUB);
            let st = vec2<f32>(dot(f.s_.xy, q) + f.s_.z, dot(f.t_.xy, q) + f.t_.z);
            if all(st >= vec2<f32>(0.0)) && all(st < vec2<f32>(1.0)) {
                let c = textureSampleLevel(textura, amostra, mix(f.uv.xy, f.uv.zw, st), 0.0);
                let reta = select(c.rgb, c.rgb / max(c.a, 1e-6), f.premultiplicada > 0.5);
                soma_a += c.a;
                soma_rgb += reta * c.a;
            }
        }
    }
    let c = soma_a / f32(SUB * SUB);
    if c <= 0.0 {
        discard;
    }
    let cor = vec4<f32>(soma_rgb / max(soma_a, 1e-6) * f.cor.rgb, f.cor.a);
    return c0_alvos(c, cor, f.opacidade);
}
