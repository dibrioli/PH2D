// C3 REBOTE (W5c, a lei do plano §2.4) — a FONTE do rebote do quadro seguinte: em cada texel das
// cascatas junto a uma borda, a luz que a superfície recebe ali, `luz_em(x⁺) = F(x⁺)/2π` (as cascatas
// mais a luz direta das lâmpadas), no EXTERIOR mais próximo `x⁺` (o F3: o que a câmera mostra). Dentro
// de um corpo (W5k: também o translúcido, que devolve `α·a`), `x⁺` sai da janela 5×5 pela bissecção do
// `meio`; fora, a faixa de `BANDA_DO_REBOTE`
// texels em volta de toda borda guarda a luz do próprio ponto (um objeto que anda acha a fonte onde
// chega). O resto fica com alfa 0 (sem fonte).
//
// Concatenado depois de `c3_direta.wgsl` (usa `meio`, `livre`, `c0_em`, `sem_sombra`, `visibilidade`,
// `na_fronteira`); grupo 3, binding 3 (os 0–2 são do preenchimento).

@group(3) @binding(3) var fonte_saida: texture_storage_2d<rgba16float, write>;

override BANDA_DO_REBOTE: f32 = 2.0;

fn luz_do_rebote(P: vec2<f32>) -> vec3<f32> {
    var f = c0_em(P);
    for (var k = 0u; k < lampadas.n; k++) {
        let s = sem_sombra(P, lampadas.itens[k]);
        if any(s > vec3<f32>(0.0)) {
            f += s * visibilidade(P, k, 0.0);
        }
    }
    return f / TAU;
}

@compute @workgroup_size(8, 8)
fn c3_rebote(@builtin(global_invocation_id) gid: vec3<u32>) {
    if any(gid.xy >= cas.tamanho) {
        return;
    }
    let P = vec2<f32>(gid.xy) + 0.5;
    var Q = P;
    borda_aqui = true;
    let dentro = num_corpo(P);
    borda_aqui = dentro;
    if dentro {
        var d_melhor = 3.4e38;
        for (var dy = -2; dy <= 2; dy++) {
            for (var dx = -2; dx <= 2; dx++) {
                let Pt = P + vec2<f32>(f32(dx), f32(dy));
                if any(Pt < vec2<f32>(0.0)) || any(Pt >= vec2<f32>(cas.tamanho)) {
                    continue;
                }
                if num_corpo(Pt) {
                    continue;
                }
                let b = na_fronteira(P, Pt);
                let d = distance(b, P);
                if d < d_melhor {
                    d_melhor = d;
                    Q = b;
                }
            }
        }
        if d_melhor >= 3.4e38 {
            textureStore(fonte_saida, gid.xy, vec4<f32>(0.0));
            return;
        }
    } else if livre(P) > BANDA_DO_REBOTE {
        textureStore(fonte_saida, gid.xy, vec4<f32>(0.0));
        return;
    }
    textureStore(fonte_saida, gid.xy, vec4<f32>(luz_do_rebote(Q), 1.0));
}
