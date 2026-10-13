// O `corrige` com os nove cursores em MATRIZES privadas indexadas dinamicamente (a forma de
// sempre, doc 130). Concatenado depois de `passe_na_placa_varredura.wgsl`.

// `varredura::corrigida` + `laco::aplica`: escreve a peca `i` na metade `bo` e devolve o quanto ela
// andou (`-1` = sem contacto).
fn pnp_corrige(i: u32, bi: u32, bo: u32, alc_max: f32) -> f32 {
    let s = est[bi + i];
    if (u32(dec[4u * i].x) == 0u) {
        est[bo + i] = s;
        return -1.0;
    }
    let lado = 2.0 * alc_max;
    let ci = pnp_celula(s.xy, lado);
    // Os 9 baldes a' volta, fundidos por indice CRESCENTE -- a ordem de soma da CPU.
    var cur: array<u32, 9>;
    var fim: array<u32, 9>;
    var cel: array<vec2<i32>, 9>;
    var cab: array<u32, 9>;
    for (var k = 0u; k < 9u; k = k + 1u) {
        let c = ci + vec2<i32>(i32(k % 3u) - 1, i32(k / 3u) - 1);
        let b = pnp_balde(c, u.num_buckets);
        fim[k] = atomicLoad(&starts[b + 1u]);
        cel[k] = c;
        let h = pnp_cabeca(atomicLoad(&starts[b]), fim[k], c, lado, bi);
        cur[k] = h.x;
        cab[k] = h.y;
    }
    let pk = s.xy;
    let mk = dec[4u * i + 2u];
    let col_k = pnp_agora(i);
    let ligados = (u.tem & 256u) != 0u;
    var acc = vec3<f32>(0.0);
    var contatos = 0u;
    var candidatos = 0u;
    loop {
        var j = PNP_NINGUEM;
        var kk = 9u;
        for (var k = 0u; k < 9u; k = k + 1u) {
            if (cab[k] < j) { j = cab[k]; kk = k; }
        }
        if (kk == 9u) { break; }
        let h = pnp_cabeca(cur[kk] + 1u, fim[kk], cel[kk], lado, bi);
        cur[kk] = h.x;
        cab[kk] = h.y;
        if (j == i) { continue; }
        candidatos = candidatos + 1u;
        let c = pnp_contribui(i, j, bi, pk, mk, col_k, ligados, acc);
        acc = c.xyz;
        if (c.w > 0.0) { contatos = contatos + 1u; }
    }
    // O `Relatorio` (doc 130 §5): os candidatos da 1.ª varredura, somados sobre as peças.
    if (atomicLoad(&ctrl[3]) == 0u) { atomicAdd(&ctrl[4], candidatos); }
    return pnp_aplica(i, bi, bo, acc, contatos, alc_max);
}
