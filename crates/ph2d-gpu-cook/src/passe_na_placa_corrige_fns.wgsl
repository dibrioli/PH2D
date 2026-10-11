const PNP_NINGUEM: u32 = 0xffffffffu;

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

// A cabeca de um cursor: o primeiro indice do balde que vive DE FACTO na celula `alvo` (dois
// celulas podem partilhar um balde), ou `PNP_NINGUEM`.
// Devolve `(cursor, cabeca)`.
fn pnp_cabeca(inicio: u32, fim: u32, alvo: vec2<i32>, lado: f32) -> vec2<u32> {
    var c0 = inicio;
    loop {
        if (c0 >= fim) { return vec2<u32>(c0, PNP_NINGUEM); }
        let j = sorted[c0];
        let c = pnp_celula(est_in[j].xy, lado);
        if (c.x == alvo.x && c.y == alvo.y) { return vec2<u32>(c0, j); }
        c0 = c0 + 1u;
    }
    return vec2<u32>(fim, PNP_NINGUEM);
}
