// ⭐ W5b (F4) — as MÁSCARAS dos sprites distantes: o alfa de cada um na própria moldura `[0, 1]²`,
// numa janela da atlas (`distantes.rs`). Cada texel da janela é a média do alfa em 4×4 subamostras
// da área dele (o alfa É a cobertura, como na C0); a C3 lê-a bilinear e diz «dentro» de ½ para cima.

struct Atlas {
    lado: vec2<f32>,
    _p: vec2<f32>,
}

struct Mascara {
    // O canto da janela (texels da atlas) e o lado dela.
    janela: vec4<f32>,
    // O UV em (s, t) = (0, 0) (xy) e (1, 1) (zw).
    uv: vec4<f32>,
}

@group(0) @binding(0) var<uniform> atlas: Atlas;
@group(0) @binding(1) var<storage, read> lista: array<Mascara>;
@group(1) @binding(0) var textura: texture_2d<f32>;
@group(1) @binding(1) var amostra: sampler;

const SUB_MASCARA: u32 = 4u;

struct Saida {
    @builtin(position) pos: vec4<f32>,
    @location(0) @interpolate(flat) mascara: u32,
}

@vertex
fn vs(@builtin(vertex_index) v: u32, @builtin(instance_index) i: u32) -> Saida {
    let j = lista[i].janela;
    let p = j.xy + j.z * vec2<f32>(f32(v & 1u), f32(v >> 1u));
    var o: Saida;
    o.pos = vec4<f32>(p.x / atlas.lado.x * 2.0 - 1.0, 1.0 - p.y / atlas.lado.y * 2.0, 0.0, 1.0);
    o.mascara = i;
    return o;
}

@fragment
fn fs(e: Saida) -> @location(0) vec4<f32> {
    let m = lista[e.mascara];
    let canto = e.pos.xy - 0.5 - m.janela.xy;
    var soma = 0.0;
    for (var j = 0u; j < SUB_MASCARA; j++) {
        for (var i = 0u; i < SUB_MASCARA; i++) {
            let st = (canto + (vec2<f32>(f32(i), f32(j)) + 0.5) / f32(SUB_MASCARA)) / m.janela.z;
            soma += textureSampleLevel(textura, amostra, mix(m.uv.xy, m.uv.zw, st), 0.0).a;
        }
    }
    return vec4<f32>(soma / f32(SUB_MASCARA * SUB_MASCARA), 0.0, 0.0, 1.0);
}
