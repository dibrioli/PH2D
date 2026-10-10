// C1 — o campo de distância por JFA (Rong & Tan, 2006), em resolução reduzida (`fator` texels
// finos por célula), mais um passo de 1 a fechar (JFA+1). Cada célula guarda a semente mais
// próxima empacotada em r32uint (`x | y << 16`); VAZIO = nenhuma.
// Sólido = cobertura > 0 (emissor ou oclusor, inteiro ou em parte): o que a marcha não salta — na
// captura das cascatas OU na captura FINA dos opacos (W5a: o campo serve as duas marchas, e um
// oclusor mais fino que o texel grosso só existe na fina). A fina chega pela MARCA: `zera` e
// `marca` (um fio por texel fino) deixam 1 em cada célula com cobertura fina; a `semente` lê-a.

struct Jfa {
    fino: vec2<u32>,
    tamanho: vec2<u32>,
    fator: u32,
    salto: u32,
    // Texels da captura fina por texel das cascatas; 0 = sem ela (a semente não lê a marca).
    escala: u32,
    _pad: u32,
}

@group(0) @binding(0) var<uniform> jfa: Jfa;
@group(0) @binding(1) var cobertura: texture_2d<f32>;
@group(0) @binding(2) var entrada: texture_2d<u32>;
@group(0) @binding(3) var saida: texture_storage_2d<r32uint, write>;

const VAZIO: u32 = 0xffffffffu;

@compute @workgroup_size(8, 8)
fn semente(@builtin(global_invocation_id) id: vec3<u32>) {
    if id.x >= jfa.tamanho.x || id.y >= jfa.tamanho.y {
        return;
    }
    var s = VAZIO;
    let base = id.xy * jfa.fator;
    for (var dy = 0u; dy < jfa.fator; dy++) {
        for (var dx = 0u; dx < jfa.fator; dx++) {
            let p = base + vec2<u32>(dx, dy);
            if p.x < jfa.fino.x && p.y < jfa.fino.y && textureLoad(cobertura, p, 0).r > 0.0 {
                s = id.x | (id.y << 16u);
            }
        }
    }
    // A marca da captura fina (ligada em `entrada`).
    if jfa.escala > 0u && textureLoad(entrada, id.xy, 0).x != 0u {
        s = id.x | (id.y << 16u);
    }
    textureStore(saida, id.xy, vec4<u32>(s, 0u, 0u, 0u));
}

// ⭐ W5a: a marca a zeros (`saida` = a marca), por célula.
@compute @workgroup_size(8, 8)
fn zera(@builtin(global_invocation_id) id: vec3<u32>) {
    if id.x >= jfa.tamanho.x || id.y >= jfa.tamanho.y {
        return;
    }
    textureStore(saida, id.xy, vec4<u32>(0u));
}

// ⭐ W5a: um fio por texel da captura fina (`cobertura` = a fina, `saida` = a marca): 1 na célula de
// quem tem cobertura.
@compute @workgroup_size(8, 8)
fn marca(@builtin(global_invocation_id) id: vec3<u32>) {
    if any(id.xy >= jfa.fino * jfa.escala) {
        return;
    }
    if textureLoad(cobertura, id.xy, 0).r > 0.0 {
        textureStore(saida, id.xy / (jfa.fator * jfa.escala), vec4<u32>(1u));
    }
}

@compute @workgroup_size(8, 8)
fn salto(@builtin(global_invocation_id) id: vec3<u32>) {
    if id.x >= jfa.tamanho.x || id.y >= jfa.tamanho.y {
        return;
    }
    let aqui = vec2<i32>(id.xy);
    let k = i32(jfa.salto);
    var melhor = VAZIO;
    var d_melhor = 3.4e38;
    for (var dy = -1; dy <= 1; dy++) {
        for (var dx = -1; dx <= 1; dx++) {
            let q = aqui + vec2<i32>(dx, dy) * k;
            if q.x < 0 || q.y < 0 || q.x >= i32(jfa.tamanho.x) || q.y >= i32(jfa.tamanho.y) {
                continue;
            }
            let s = textureLoad(entrada, q, 0).x;
            if s == VAZIO {
                continue;
            }
            let v = vec2<f32>(f32(s & 0xffffu), f32(s >> 16u)) - vec2<f32>(aqui);
            let d = dot(v, v);
            if d < d_melhor {
                d_melhor = d;
                melhor = s;
            }
        }
    }
    textureStore(saida, id.xy, vec4<u32>(melhor, 0u, 0u, 0u));
}
