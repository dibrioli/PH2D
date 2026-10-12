// C3 — o PREENCHIMENTO (W5a, F3 da fila do dono): o JFA do COMPLEMENTO dos opacos sobre a grade
// fina, em células de `celula` texels finos. A semente de uma célula é o texel fino EXTERIOR dela
// mais perto do centro (fora ⟺ o `meio` da captura ligada tem τ > 0); uma célula toda dentro de um
// opaco nasce sem semente, e os saltos dão-lhe a mais próxima. Só a REGIÃO (em células) se calcula.
// A `c3_fina` lê o resultado (`preenchido`): o texel de dentro calcula-se no exterior mais próximo.
//
// Concatenado depois de `c3_direta.wgsl` (usa `meio` e `SEM_SEMENTE`).

struct Preenche {
    celulas: vec2<u32>,
    inicio: vec2<u32>,
    fim: vec2<u32>,
    celula: u32,
    salto: u32,
    // Texels finos por texel das cascatas.
    fina: u32,
    _p0: u32,
    _p1: u32,
    _p2: u32,
}

@group(3) @binding(0) var<uniform> pre: Preenche;
@group(3) @binding(1) var pre_entrada: texture_2d<u32>;
@group(3) @binding(2) var pre_saida: texture_storage_2d<r32uint, write>;

fn fora_do_opaco(t: vec2<u32>) -> bool {
    return !num_corpo((vec2<f32>(t) + 0.5) / f32(pre.fina));
}

@compute @workgroup_size(8, 8)
fn c3_preenche_semente(@builtin(global_invocation_id) gid: vec3<u32>) {
    let c = gid.xy + pre.inicio;
    if any(c >= pre.fim) {
        return;
    }
    let n = cas.tamanho * pre.fina;
    let centro = (vec2<f32>(c) + 0.5) * f32(pre.celula);
    var melhor = SEM_SEMENTE;
    var d_melhor = 3.4e38;
    for (var dy = 0u; dy < pre.celula; dy++) {
        for (var dx = 0u; dx < pre.celula; dx++) {
            let t = c * pre.celula + vec2<u32>(dx, dy);
            if any(t >= n) || !fora_do_opaco(t) {
                continue;
            }
            let v = vec2<f32>(t) + 0.5 - centro;
            let d = dot(v, v);
            if d < d_melhor {
                d_melhor = d;
                melhor = t.x | (t.y << 16u);
            }
        }
    }
    textureStore(pre_saida, c, vec4<u32>(melhor, 0u, 0u, 0u));
}

@compute @workgroup_size(8, 8)
fn c3_preenche_salto(@builtin(global_invocation_id) gid: vec3<u32>) {
    let c = gid.xy + pre.inicio;
    if any(c >= pre.fim) {
        return;
    }
    let aqui = (vec2<f32>(c) + 0.5) * f32(pre.celula);
    let k = i32(pre.salto);
    var melhor = SEM_SEMENTE;
    var d_melhor = 3.4e38;
    for (var dy = -1; dy <= 1; dy++) {
        for (var dx = -1; dx <= 1; dx++) {
            let q = vec2<i32>(c) + vec2<i32>(dx, dy) * k;
            if any(q < vec2<i32>(pre.inicio)) || any(q >= vec2<i32>(pre.fim)) {
                continue;
            }
            let s = textureLoad(pre_entrada, q, 0).x;
            if s == SEM_SEMENTE {
                continue;
            }
            let v = vec2<f32>(f32(s & 0xffffu), f32(s >> 16u)) + 0.5 - aqui;
            let d = dot(v, v);
            if d < d_melhor {
                d_melhor = d;
                melhor = s;
            }
        }
    }
    textureStore(pre_saida, c, vec4<u32>(melhor, 0u, 0u, 0u));
}
