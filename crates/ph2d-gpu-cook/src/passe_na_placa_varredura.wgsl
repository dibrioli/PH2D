// UMA VARREDURA do passe (doc 130) -- as funcoes que os dois caminhos partilham: o de varios
// despachos (muitas pecas) e o de UM grupo (poucas pecas, todas as varreduras num despacho).
// O estado vive em `est` com DUAS metades (`bi`/`bo` = 0 ou `n`): le-se uma fotografia, escreve-se
// a outra -- o Jacobi da CPU.

const PNP_NINGUEM: u32 = 0xffffffffu;

// Toca em todas as ligacoes: todo modulo da varredura tem o MESMO leiaute (o automatico so' inclui
// as ligacoes usadas, e um grupo de ligacoes tem de casar com ele).
fn pnp_toca() {
    if (u.n == PNP_NINGUEM) {
        _ = dec[0];
        _ = ago[0];
        _ = est[0];
        _ = atomicLoad(&starts[0]);
        _ = atomicLoad(&cursor[0]);
        _ = sorted[0];
        _ = atomicLoad(&ctrl[0]);
    }
}

// O colisor DECLARADO de `x` (o do filtro do repouso) e o de AGORA (rodado pelo giro).
fn pnp_declarado(x: u32) -> Col {
    let d0 = dec[4u * x];
    let d1 = dec[4u * x + 1u];
    return Col(u32(d0.x), d0.yz, d1.xy, d1.zw);
}
fn pnp_agora(x: u32) -> Col {
    let d0 = dec[4u * x];
    let a = ago[x];
    return Col(u32(d0.x), d0.yz, a.xy, a.zw);
}

// `Colisor::girado`: a forma de AGORA, rodada pelo giro acumulado.
fn pnp_gira_so(i: u32, bi: u32) {
    let d1 = dec[4u * i + 1u];
    let s = est[bi + i];
    var a = d1;
    if (s.z != 0.0) {
        let e = pnp_eixo_de(s.z);
        a = vec4<f32>(pnp_gira(d1.xy, e), a.zw);
        if (!pnp_zero(d1.zw)) { a = vec4<f32>(a.xy, pnp_gira(d1.zw, e)); }
    }
    ago[i] = a;
}

// A forma de agora + a contagem na grelha.
fn pnp_gira_conta(i: u32, bi: u32, lado: f32) {
    if (u32(dec[4u * i].x) == 0u) { return; }
    pnp_gira_so(i, bi);
    atomicAdd(&starts[pnp_balde(pnp_celula(est[bi + i].xy, lado), u.num_buckets)], 1u);
}

fn pnp_espalha(i: u32, bi: u32, lado: f32) {
    if (u32(dec[4u * i].x) == 0u) { return; }
    let b = pnp_balde(pnp_celula(est[bi + i].xy, lado), u.num_buckets);
    sorted[atomicLoad(&starts[b]) + atomicAdd(&cursor[b], 1u)] = i;
}

// Cada balde por indice CRESCENTE (insercao: os baldes sao pequenos).
fn pnp_ordena(b: u32) {
    let lo = atomicLoad(&starts[b]);
    let hi = atomicLoad(&starts[b + 1u]);
    for (var a = lo + 1u; a < hi; a = a + 1u) {
        let x = sorted[a];
        var c = a;
        while (c > lo && sorted[c - 1u] > x) { sorted[c] = sorted[c - 1u]; c = c - 1u; }
        sorted[c] = x;
    }
}

// A cabeca de um cursor: o primeiro indice do balde que vive DE FACTO na celula `alvo` (duas
// celulas podem partilhar um balde), ou `PNP_NINGUEM`. Devolve `(cursor, cabeca)`.
fn pnp_cabeca(inicio: u32, fim: u32, alvo: vec2<i32>, lado: f32, bi: u32) -> vec2<u32> {
    var c0 = inicio;
    loop {
        if (c0 >= fim) { return vec2<u32>(c0, PNP_NINGUEM); }
        let j = sorted[c0];
        let c = pnp_celula(est[bi + j].xy, lado);
        if (c.x == alvo.x && c.y == alvo.y) { return vec2<u32>(c0, j); }
        c0 = c0 + 1u;
    }
    return vec2<u32>(fim, PNP_NINGUEM);
}

// UM PAR, do lado de `i` (`varredura::corrigida`, o corpo do laco): soma no acumulador `acc` na
// ORDEM da CPU (ponto a ponto) e devolve `(acc, tocou)`.
fn pnp_contribui(i: u32, j: u32, bi: u32, pk: vec2<f32>, mk: vec4<f32>, col_k: Col, ligados: bool, acc: vec3<f32>) -> vec4<f32> {
    var delta = acc.xy;
    var giro = acc.z;
    let lo = min(i, j);
    let hi = max(i, j);
    let mj = dec[4u * j + 2u];
    if (ligados && mk.w != 0.0 && mk.w == mj.w) {
        let r_lo = dec[4u * lo + 3u].xy;
        let r_hi = dec[4u * hi + 3u].xy;
        if (pnp_contato(pnp_declarado(lo), r_lo, pnp_declarado(hi), r_hi, true).ok) { return vec4<f32>(acc, 0.0); }
    }
    let col_j = pnp_agora(j);
    let pj = est[bi + j].xy;
    var m: Man;
    if (i == lo) {
        m = pnp_manifesto(col_k, pk, col_j, pj, ((lo + hi) % 2u) == 0u);
    } else {
        m = pnp_manifesto(col_j, pj, col_k, pk, ((lo + hi) % 2u) == 0u);
    }
    if (m.np == 0u) { return vec4<f32>(acc, 0.0); }
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
    return vec4<f32>(delta, giro, select(0.0, 1.0, tocou));
}

// O `pnp_corrige` (`varredura::corrigida` + `laco::aplica`) vem DEPOIS deste ficheiro: a fusão dos
// nove cursores tem duas formas no código, escolhidas na montagem do módulo (doc 132 §3).

// `laco::aplica` para a peca `i`: a media dos contactos, escrita na metade `bo`; devolve o quanto
// ela andou (`-1` = sem contacto).
fn pnp_aplica(i: u32, bi: u32, bo: u32, acc: vec3<f32>, contatos: u32, alc_max: f32) -> f32 {
    let s = est[bi + i];
    if (contatos == 0u) {
        est[bo + i] = s;
        return -1.0;
    }
    let pk = s.xy;
    let inv = 1.0 / f32(contatos);
    let q = vec2<f32>(pk.x + acc.x * inv, pk.y + acc.y * inv);
    let giro_novo = s.z + acc.z * inv;
    est[bo + i] = vec4<f32>(q, giro_novo, 0.0);
    let dx = q.x - pk.x;
    let dy = q.y - pk.y;
    return sqrt(dx * dx + dy * dy) + abs(giro_novo - s.z) * PNP_POR_GRAU * alc_max;
}

// O `andou` como chave do `atomicMax` (nao-finito = infinito, a lei do `aplica`).
fn pnp_bits(andou: f32) -> u32 {
    if (pnp_finito(andou)) { return bitcast<u32>(andou); }
    return 0x7f800000u;
}
