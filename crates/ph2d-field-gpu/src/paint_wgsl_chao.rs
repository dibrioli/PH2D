//! ⭐⭐⭐ **O CHÃO QUE SÓ RECEBE, em WGSL** — o factor da sombra e a luz que a peça devolve ao chão.
//!
//! ⛔ **Split, nunca allowlist** (`CLAUDE.md` §5.0): a leitura da grelha do chão passou de bilinear a
//! B-spline cúbica em 2026-09-24 (o report das *«áreas retangulares ruins»*, `docs/Render3d/09`
//! §10) e levou o [`super::paint_wgsl_sondas`] de `693` para `709` linhas. ⚠️ E a fronteira **não
//! foi escolhida**: aquele ficheiro integra o HEMISFÉRIO da peça, e isto é o que o CHÃO recebe dela
//! — as duas metades da `W4` (a sombra, `docs/Render3d/07`) e da cor devolvida (`09`).
//!
//! ⚠️ **As metades são UM shader**, concatenadas por quem monta ([`super::paint_fonte`]): as
//! ranhuras (`{FADE}`, `{LUMA_R}`…) são preenchidas no texto JUNTO, e tudo o que isto usa
//! (`mat_do_chao`, `chao_em`, `mx_indirect`…) está declarado nas outras metades.

/// ⭐⭐⭐ **O chão, em WGSL** — o gémeo do `shade_render::catcher` e da `GroundBounce::sample` da CPU.
pub(crate) const PINTOR_CHAO: &str = r"
// ⭐⭐⭐ **A LEI DO CHÃO QUE SÓ RECEBE** — `luz que chega com a peça / luz que chegaria sem ela`, em
// luminância, sobre a difusa branca. É o `shade_render::catcher` da CPU, linha a linha.
//
// ⚠️ **As duas somas correm as mesmas contas na mesma ordem**, e onde nada tapa elas são o MESMO
// número: a razão sai exactamente `1` e o pixel fica com os bytes do fundo.
fn fator_do_chao(i: u32, q: vec3<f32>, v: vec3<f32>) -> f32 {
    let m = mat_do_chao();
    let n = mundo_para_vista(vec3<f32>(0.0, 1.0, 0.0));
    let ceu = mx_indirect(m, n, v);
    let base = i * passo_da_luz();
    var livre = ceu;
    var chega = ceu * luz[base];
    let piso = PISO_LUZ * PISO_LUZ;
    for (var l: u32 = 0u; l < s.n_lamps; l = l + 1u) {
        let d = s.lamps[l].xyz - q;
        let cru = dot(d, d);
        var to_light = n;
        if (cru > piso) { to_light = mundo_para_vista(d * (1.0 / sqrt(cru))); }
        let rad = pintor.lamp[l].rgb / max(cru, piso);
        livre = livre + mx_direct(m, n, v, to_light, rad);
        chega = chega + mx_direct(m, n, v, to_light, rad * luz[base + 1u + l]);
    }
    let luma = vec3<f32>({LUMA_R}, {LUMA_G}, {LUMA_B});
    let a = luma.x * chega.x + luma.y * chega.y + luma.z * chega.z;
    let b = luma.x * livre.x + luma.y * livre.y + luma.z * livre.z;
    if (!(b > 0.0)) { return 1.0; }
    return clamp(a / b, 0.0, 1.0);
}

// O fundo com a sombra do chão por cima, em linear pré-multiplicado — o `shadowed_background` da CPU.
fn fundo_sombreado(f: f32) -> vec4<f32> {
    return vec4<f32>(pintor.fundo.rgb * f, (1.0 - f) + pintor.fundo.a * f);
}

// ⭐⭐⭐ **A LUZ QUE A PEÇA DEVOLVE AO PONTO `q` DO CHÃO** — a B-spline cúbica sobre os `4×4` nós,
// vezes o esmorecimento da orla. É o `GroundBounce::sample` da CPU, linha a linha (e os pesos são o
// `bspline_pesos` dela).
//
// ⚠️ **Fora do campo devolve ZERO**, e a orla já lá pôs zero antes da borda: a saída antecipada é
// guarda de índice, e a LEI é a orla (ver o doc da irmã de CPU).
fn bspline_pesos(t: f32) -> vec4<f32> {
    let s = 1.0 - t;
    let t2 = t * t;
    let t3 = t * t * t;
    return vec4<f32>(
        s * s * s / 6.0,
        (3.0 * t3 - 6.0 * t2 + 4.0) / 6.0,
        (-3.0 * t3 + 3.0 * t2 + 3.0 * t + 1.0) / 6.0,
        t3 / 6.0,
    );
}

fn ricochete_do_chao(q: vec3<f32>) -> vec3<f32> {
    let n = pintor.modo2.y;
    if (n < 2u) { return vec3<f32>(0.0); }
    let passo = pintor.chao_campo.z;
    if (!(passo > 0.0)) { return vec3<f32>(0.0); }
    let lado = f32(n - 1u);
    let u = vec2<f32>(
        (q.x - pintor.chao_campo.x) / passo,
        (q.z - pintor.chao_campo.y) / passo,
    );
    if (u.x < 0.0 || u.y < 0.0 || u.x > lado || u.y > lado) { return vec3<f32>(0.0); }
    let c0 = vec2<u32>(
        min(u32(floor(u.x)), n - 2u),
        min(u32(floor(u.y)), n - 2u),
    );
    let f = vec2<f32>(
        clamp(u.x - f32(c0.x), 0.0, 1.0),
        clamp(u.y - f32(c0.y), 0.0, 1.0),
    );
    let wx = bspline_pesos(f.x);
    let wz = bspline_pesos(f.y);
    let ultimo = n - 1u;
    var soma = vec3<f32>(0.0);
    for (var dz = 0u; dz < 4u; dz = dz + 1u) {
        // O nó `c0 - 1 + dz`, preso à borda.
        let iz = min(max(c0.y + dz, 1u) - 1u, ultimo);
        for (var dx = 0u; dx < 4u; dx = dx + 1u) {
            let ix = min(max(c0.x + dx, 1u) - 1u, ultimo);
            let w = wx[dx] * wz[dz];
            let k = (iz * n + ix) * 3u;
            soma = soma + w * vec3<f32>(chao_luz[k], chao_luz[k + 1u], chao_luz[k + 2u]);
        }
    }
    // A orla: medida no QUADRADO do campo (Chebyshev), como na CPU.
    let meia = passo * lado * 0.5;
    if (!(meia > 0.0)) { return vec3<f32>(0.0); }
    let centro = vec2<f32>(pintor.chao_campo.x + meia, pintor.chao_campo.y + meia);
    let d = max(abs(q.x - centro.x), abs(q.z - centro.y)) / meia;
    return soma * clamp((1.0 - d) / {FADE}, 0.0, 1.0);
}

// ⭐ **A luz devolvida ao chão DESTE pixel** — `[0,0,0]` quando ele não vê chão nenhum.
fn chao_no_pixel(x: u32, y: u32) -> vec3<f32> {
    if (s.chao == 0u) { return vec3<f32>(0.0); }
    let r = ray_at_plane(raio(f32(x) + 0.5, f32(y) + 0.5));
    let q = chao_em(r);
    if (q.w == 0.0) { return vec3<f32>(0.0); }
    let e = ricochete_do_chao(q.xyz);
    if (e.x <= 0.0 && e.y <= 0.0 && e.z <= 0.0) { return vec3<f32>(0.0); }
    // ⭐⭐ **A MESMA lei indirecta com o ambiente trocado** que a peça usa para o ricochete dela —
    // o espelho exacto do `SoIrradiancia` da CPU. ⚠️ Escrever aqui uma segunda expressão para «a
    // resposta do material a uma irradiância» seria a segunda cópia de uma lei que já tem porta.
    ricochete = e;
    ambiente_e_ricochete = true;
    let saiu = mx_indirect(
        mat_do_chao(),
        mundo_para_vista(vec3<f32>(0.0, 1.0, 0.0)),
        direccao_de_vista(r.d),
    );
    ambiente_e_ricochete = false;
    return saiu;
}

// ⭐⭐ **A luz devolvida de uma BORDA** — a média dos vizinhos de cruz que falham a peça, na MESMA
// ordem do `edge_ground_bounce` da CPU. ⚠️ **Sem vizinho de fundo devolve ZERO** (e o irmão do
// factor devolve `1,0`): ausência de LUZ, nunca luz inventada.
fn chao_da_borda(i: u32, x: u32, y: u32) -> vec3<f32> {
    if (s.chao == 0u) { return vec3<f32>(0.0); }
    if (centro[i].x < 0.0) { return chao_no_pixel(x, y); }
    var soma = vec3<f32>(0.0);
    var n = 0u;
    if (x > 0u && centro[i - 1u].x < 0.0) { soma = soma + chao_no_pixel(x - 1u, y); n = n + 1u; }
    if (x + 1u < s.w && centro[i + 1u].x < 0.0) { soma = soma + chao_no_pixel(x + 1u, y); n = n + 1u; }
    if (y > 0u && centro[i - s.w].x < 0.0) { soma = soma + chao_no_pixel(x, y - 1u); n = n + 1u; }
    if (y + 1u < s.h && centro[i + s.w].x < 0.0) { soma = soma + chao_no_pixel(x, y + 1u); n = n + 1u; }
    if (n == 0u) { return vec3<f32>(0.0); }
    return soma / f32(n);
}

// O factor do chão DESTE pixel — `1,0` quando ele não vê chão nenhum.
fn fator_no_pixel(j: u32, x: u32, y: u32) -> f32 {
    let r = ray_at_plane(raio(f32(x) + 0.5, f32(y) + 0.5));
    let q = chao_em(r);
    if (q.w == 0.0) { return 1.0; }
    return fator_do_chao(j, q.xyz, direccao_de_vista(r.d));
}

// ⭐⭐ **O factor de uma BORDA** — o do próprio pixel quando ele falha a peça; senão, a média dos
// vizinhos de cruz que a falham (esquerda, direita, cima, baixo — a ordem da CPU).
fn fator_da_borda(i: u32, x: u32, y: u32) -> f32 {
    if (s.chao == 0u) { return 1.0; }
    if (centro[i].x < 0.0) { return fator_no_pixel(i, x, y); }
    var soma = 0.0;
    var n = 0u;
    if (x > 0u && centro[i - 1u].x < 0.0) { soma = soma + fator_no_pixel(i - 1u, x - 1u, y); n = n + 1u; }
    if (x + 1u < s.w && centro[i + 1u].x < 0.0) { soma = soma + fator_no_pixel(i + 1u, x + 1u, y); n = n + 1u; }
    if (y > 0u && centro[i - s.w].x < 0.0) { soma = soma + fator_no_pixel(i - s.w, x, y - 1u); n = n + 1u; }
    if (y + 1u < s.h && centro[i + s.w].x < 0.0) { soma = soma + fator_no_pixel(i + s.w, x, y + 1u); n = n + 1u; }
    if (n == 0u) { return 1.0; }
    return soma / f32(n);
}
";
