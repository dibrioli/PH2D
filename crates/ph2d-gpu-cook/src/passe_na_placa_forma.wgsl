    // `declarado` + `ativo` (doc 130 §2): o colisor de cada peca, lido UMA vez.
    if (i >= u.n) { return; }
    let p = c_p[i];
    var sz = vec2<f32>(1.0, 1.0);
    if ((u.tem & 8u) != 0u) { sz = c_size[i]; }
    var graus = 0.0;
    if ((u.tem & 16u) != 0u) { graus = c_rot[i]; }
    let eixo = pnp_eixo_de(graus);
    var c = Col(0u, vec2<f32>(0.0), eixo, vec2<f32>(0.0));
    if ((u.tem & 2u) != 0u) {
        let m = c_caixa[i];
        if (pnp_finito(m.x) && pnp_finito(m.y) && m.x >= 0.0 && m.y >= 0.0 && max(m.x, m.y) > 0.0) {
            c.kind = 2u;
            c.m = vec2<f32>(m.x * abs(sz.x), m.y * abs(sz.y));
        }
    }
    if (c.kind == 0u && (u.tem & 1u) != 0u) {
        let r = c_raio[i];
        if (pnp_finito(r) && r > 0.0) {
            c.kind = 1u;
            c.m = vec2<f32>(r * max(abs(sz.x), abs(sz.y)), 0.0);
        }
    }
    if ((u.tem & 4u) != 0u) {
        let d0 = c_desvio[i];
        if (!pnp_zero(d0)) {
            let d = vec2<f32>(d0.x * sz.x, d0.y * sz.y);
            c.desvio = vec2<f32>(d.x * eixo.x - d.y * eixo.y, d.x * eixo.y + d.y * eixo.x);
        }
    }
    var ok = c.kind != 0u && pnp_finito(c.desvio.x) && pnp_finito(c.desvio.y)
        && pnp_finito(p.x) && pnp_finito(p.y);
    if (c.kind == 1u) { ok = ok && pnp_finito(c.m.x) && c.m.x > 0.0; }
    if (c.kind == 2u) {
        ok = ok && pnp_finito(c.m.x) && pnp_finito(c.m.y) && pnp_finito(eixo.x)
            && pnp_finito(eixo.y) && max(c.m.x, c.m.y) > 0.0;
    }
    if (!ok) { c.kind = 0u; }
    var alc = 0.0;
    if (c.kind != 0u) { alc = pnp_alcance(c); }
    dec[4u * i] = vec4<f32>(f32(c.kind), c.m.x, c.m.y, alc);
    dec[4u * i + 1u] = vec4<f32>(c.eixo, c.desvio);
    if (c.kind != 0u) { atomicMax(&ctrl[0], bitcast<u32>(alc)); }
