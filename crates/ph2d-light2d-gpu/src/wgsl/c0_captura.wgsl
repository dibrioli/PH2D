// C0 — a captura da `CenaDeLuz` na grade de luz. A GEOMETRIA é a cobertura por texel; o MATERIAL é
// de quem possui o texel (cobertura própria ≥ ½):
//   alvo 0 `emissao`    (Rgba16Float, max): rgb = radiância do emissor dono, a = 1/alcance
//   alvo 1 `cobertura`  (R16Float, soma):   Σ cobertura de todas as formas
//   alvo 2 `tau_dentro` (R16Float, ×):      ∏ (1 − α) dos donos (emissor: α = 1, é opaco)
// A marcha (c2_rc.wgsl) diz «dentro» onde a cobertura INTERPOLADA ≥ ½: a iso-linha cai na borda
// verdadeira de uma aresta reta. Ler o texel mais próximo dilatava a silhueta (o raio rente recolhia
// o máximo da cobertura do caminho): +16 % de erro contra o gabarito no Alto; com o limiar, 1,8 %
// (modelo de força bruta, docs/Luz2D handoff W2).
// Cada forma é a sua caixa; o fragmento mede a cobertura do texel: longe da borda (|distância| ≥
// meia diagonal) ela é 0 ou 1; perto, 8×8 subamostras com a regra do gabarito (paridade de
// cruzamentos; disco: |p − c| < r).

struct Grade {
    tamanho: vec2<f32>,
    _pad: vec2<f32>,
}

struct Forma {
    caixa: vec4<f32>,
    // rgb = radiância (0 num oclusor), a = 1/alcance (0 = infinito).
    cor: vec4<f32>,
    // Disco: centro, raio, 1. Polígono: w = 0.
    disco: vec4<f32>,
    vertices: vec2<u32>,
    opacidade: f32,
    _pad: f32,
}

@group(0) @binding(0) var<uniform> grade: Grade;
@group(0) @binding(1) var<storage, read> formas: array<Forma>;
@group(0) @binding(2) var<storage, read> vertices: array<vec2<f32>>;

const MEIA_DIAGONAL: f32 = 0.70710678;
const SUB: u32 = 8u;

struct Saida {
    @builtin(position) pos: vec4<f32>,
    @location(0) @interpolate(flat) forma: u32,
}

@vertex
fn vs(@builtin(vertex_index) v: u32, @builtin(instance_index) i: u32) -> Saida {
    let c = formas[i].caixa;
    let p = mix(c.xy, c.zw, vec2<f32>(f32(v & 1u), f32(v >> 1u)));
    var o: Saida;
    // A linha j da grade é a linha j do alvo (a linha 0 no topo do NDC).
    o.pos = vec4<f32>(p.x / grade.tamanho.x * 2.0 - 1.0, 1.0 - p.y / grade.tamanho.y * 2.0, 0.0, 1.0);
    o.forma = i;
    return o;
}

fn no_disco(f: Forma, p: vec2<f32>) -> bool {
    let d = p - f.disco.xy;
    return dot(d, d) < f.disco.z * f.disco.z;
}

fn no_poligono(f: Forma, p: vec2<f32>) -> bool {
    var dentro = false;
    let o = f.vertices.x;
    let n = f.vertices.y;
    var j = n - 1u;
    for (var i = 0u; i < n; i++) {
        let a = vertices[o + i];
        let b = vertices[o + j];
        if (a.y > p.y) != (b.y > p.y) && p.x < (b.x - a.x) * (p.y - a.y) / (b.y - a.y) + a.x {
            dentro = !dentro;
        }
        j = i;
    }
    return dentro;
}

fn dentro(f: Forma, p: vec2<f32>) -> bool {
    if f.disco.w > 0.5 {
        return no_disco(f, p);
    }
    return no_poligono(f, p);
}

// A distância de `p` à borda da forma.
fn distancia_a_borda(f: Forma, p: vec2<f32>) -> f32 {
    if f.disco.w > 0.5 {
        return abs(length(p - f.disco.xy) - f.disco.z);
    }
    var d = 3.4e38;
    let o = f.vertices.x;
    let n = f.vertices.y;
    var j = n - 1u;
    for (var i = 0u; i < n; i++) {
        let a = vertices[o + j];
        let ab = vertices[o + i] - a;
        let t = clamp(dot(p - a, ab) / max(dot(ab, ab), 1e-30), 0.0, 1.0);
        d = min(d, length(p - a - ab * t));
        j = i;
    }
    return d;
}

fn cobertura(f: Forma, p: vec2<f32>) -> f32 {
    if distancia_a_borda(f, p) >= MEIA_DIAGONAL {
        return select(0.0, 1.0, dentro(f, p));
    }
    var n = 0u;
    for (var j = 0u; j < SUB; j++) {
        for (var i = 0u; i < SUB; i++) {
            let q = p - 0.5 + (vec2<f32>(f32(i), f32(j)) + 0.5) / f32(SUB);
            n += select(0u, 1u, dentro(f, q));
        }
    }
    return f32(n) / f32(SUB * SUB);
}

// A lei do dono (`c0_alvos`) vem de `c0_dono.wgsl`, concatenado à frente.
@fragment
fn fs(e: Saida) -> Alvos {
    let f = formas[e.forma];
    let c = cobertura(f, e.pos.xy);
    if c <= 0.0 {
        discard;
    }
    return c0_alvos(c, f.cor, f.opacidade);
}
