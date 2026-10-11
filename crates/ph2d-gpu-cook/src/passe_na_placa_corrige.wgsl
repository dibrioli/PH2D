    // `varredura::corrigida` + `laco::aplica` (doc 130 §2): Jacobi com a media dos contactos.
    if (i >= u.n) { return; }
    let s = est_in[i];
    if (atomicLoad(&ctrl[2]) != 0u || u32(dec[4u * i].x) == 0u) {
        est_out[i] = s;
        return;
    }
    let alc_max = bitcast<f32>(atomicLoad(&ctrl[0]));
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
        cur[k] = starts[b];
        fim[k] = starts[b + 1u];
        cel[k] = c;
        let h = pnp_cabeca(cur[k], fim[k], c, lado);
        cur[k] = h.x;
        cab[k] = h.y;
    }
    let pk = s.xy;
    let mk = dec[4u * i + 2u];
    let col_k = pnp_agora(i);
    let ligados = (u.tem & 256u) != 0u;
    var delta = vec2<f32>(0.0, 0.0);
    var giro = 0.0;
    var contatos = 0u;
    loop {
        var j = PNP_NINGUEM;
        var kk = 9u;
        for (var k = 0u; k < 9u; k = k + 1u) {
            if (cab[k] < j) { j = cab[k]; kk = k; }
        }
        if (kk == 9u) { break; }
        let h = pnp_cabeca(cur[kk] + 1u, fim[kk], cel[kk], lado);
        cur[kk] = h.x;
        cab[kk] = h.y;
        if (j == i) { continue; }
        let lo = min(i, j);
        let hi = max(i, j);
        let mj = dec[4u * j + 2u];
        if (ligados && mk.w != 0.0 && mk.w == mj.w) {
            let r_lo = dec[4u * lo + 3u].xy;
            let r_hi = dec[4u * hi + 3u].xy;
            if (pnp_contato(pnp_declarado(lo), r_lo, pnp_declarado(hi), r_hi, true).ok) { continue; }
        }
        let col_j = pnp_agora(j);
        let pj = est_in[j].xy;
        var m: Man;
        if (i == lo) {
            m = pnp_manifesto(col_k, pk, col_j, pj, ((lo + hi) % 2u) == 0u);
        } else {
            m = pnp_manifesto(col_j, pj, col_k, pk, ((lo + hi) % 2u) == 0u);
        }
        if (m.np == 0u) { continue; }
        let ck = pnp_centro(col_k, pk);
        let cj = pnp_centro(col_j, pj);
        var sinal = 1.0;
        if (i == lo) { sinal = -1.0; }
        let quota = 1.0 / f32(m.np);
        var tocou = false;
        for (var q = 0u; q < m.np; q = q + 1u) {
            var c = m.a;
            if (q == 1u) { c = m.b; }
            let bk = pnp_cruz(vec2<f32>(c.ponto.x - ck.x, c.ponto.y - ck.y), c.n);
            let bj = pnp_cruz(vec2<f32>(c.ponto.x - cj.x, c.ponto.y - cj.y), c.n);
            let soma = (mk.x + mk.y * bk * bk) + (mj.x + mj.y * bj * bj);
            if (soma <= 0.0) { continue; }
            tocou = true;
            let lambda = c.pen / soma;
            let empurra = lambda * mk.x * sinal * quota;
            delta = vec2<f32>(delta.x + c.n.x * empurra, delta.y + c.n.y * empurra);
            giro = giro + bk * lambda * mk.y * sinal * PNP_GRAUS * quota;
        }
        if (tocou) { contatos = contatos + 1u; }
    }
    if (contatos == 0u) {
        est_out[i] = s;
        return;
    }
    let inv = 1.0 / f32(contatos);
    let q = vec2<f32>(pk.x + delta.x * inv, pk.y + delta.y * inv);
    let giro_novo = s.z + giro * inv;
    est_out[i] = vec4<f32>(q, giro_novo, 0.0);
    let dx = q.x - pk.x;
    let dy = q.y - pk.y;
    let andou = sqrt(dx * dx + dy * dy) + abs(giro_novo - s.z) * PNP_POR_GRAU * alc_max;
    var bits = 0x7f800000u;
    if (pnp_finito(andou)) { bits = bitcast<u32>(andou); }
    atomicMax(&ctrl[1], bits);
