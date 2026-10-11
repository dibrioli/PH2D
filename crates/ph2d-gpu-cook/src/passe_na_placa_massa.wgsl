    // `pesos` + `inv_inercias` + `atenuacao` + `repouso` (doc 130 §2), e o estado de partida.
    if (i >= u.n) { return; }
    let d0 = dec[4u * i];
    let kind = u32(d0.x);
    var w = 1.0;
    if ((u.tem & 32u) != 0u) {
        let x = c_w[i];
        w = select(0.0, max(x, 0.0), pnp_finito(x));
    }
    var ii = 0.0;
    var da_coluna = false;
    if ((u.tem & 64u) != 0u) {
        let x = c_ii[i];
        if (pnp_finito(x)) { ii = max(x, 0.0); da_coluna = true; }
    }
    if (!da_coluna && kind != 0u && pnp_finito(w) && w > 0.0) {
        var v = 0.0;
        if (kind == 1u) { v = 2.0 * w / (d0.y * d0.y); } else { v = 3.0 * w / (d0.y * d0.y + d0.z * d0.z); }
        if (pnp_finito(v)) { ii = v; }
    }
    var k = 1.0;
    if ((u.tem & 128u) != 0u) {
        let f = c_fall[i];
        if (pnp_finito(f)) { k = clamp(f, 0.0, 1.0); }
    }
    var body = 0.0;
    var rest = vec2<f32>(0.0);
    if ((u.tem & 256u) != 0u) { body = c_body[i]; rest = c_rest[i]; }
    dec[4u * i + 2u] = vec4<f32>(w, ii, k, body);
    dec[4u * i + 3u] = vec4<f32>(rest, 0.0, 0.0);
    est[i] = vec4<f32>(c_p[i], 0.0, 0.0);
