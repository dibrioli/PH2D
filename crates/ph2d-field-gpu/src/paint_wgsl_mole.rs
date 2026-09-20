//! ⭐⭐⭐ **A BORDA MOLE DA SOMBRA, em WGSL** — a terceira metade do corpo do pintor.
//!
//! # Por que um QUARTO ficheiro
//!
//! O [`super::paint`] monta, o [`super::paint_wgsl`] declara o que o pintor LÊ, o
//! [`super::paint_wgsl_sondas`] integra o hemisfério, e este assa **o canal que uma closure
//! TRANSLÚCIDA lê** (`docs/Render3d/10` §12 e §25).
//!
//! ⛔ **Split, nunca allowlist** (`CLAUDE.md` §5.0): a `W10` levou o ficheiro das sondas de `675`
//! para `760` linhas, e a cura é um corte por RESPONSABILIDADE — ⚠️ e a fronteira **não foi
//! escolhida**: o borrão da sombra não é o hemisfério, é o que vem antes de o pintor sombrear.
//!
//! ⚠️ **As três metades são UM shader**, concatenadas por quem monta — e esta vai por ÚLTIMO, logo
//! tudo o que ela usa (`BLUR_COS`, `passo_da_luz`, `base_do_mole`, `luz`, `centro`) já está
//! declarado acima.

/// ⭐⭐⭐ **As DUAS passagens separáveis do canal mole**, o gémeo do
/// [`ph2d_field_render::sss_shadow::blur_por_canal`].
///
/// ⚠️ **Ela é CONCATENADA depois da [`super::paint_wgsl_sondas::PINTOR_SONDAS`]** por quem monta.
pub(crate) const PINTOR_MOLE: &str = r"
// ⭐⭐⭐ **A BORDA MOLE DA SOMBRA — as DUAS passagens** (`docs/Render3d/10` §12), o gemeo do
// `ph2d_field_render::sss_shadow::blur_por_canal`.
//
// A visibilidade que uma closure TRANSLUCIDA le' e' a MEDIA da vizinhanca, sobre a distancia de
// espalhamento do material. ⭐ Ela e' SEPARAVEL: um quadrado de `(2r+1)²` toques por pixel e' `O(r²)`
// e duas passagens de `(2r+1)` sao `O(r)`, com a MESMA resposta para um nucleo de caixa — e a `r=48`
// isso e' `9 409` toques contra `194`.
//
// ⚠️ **A guarda da normal e' a MESMA do ceu e do ricochete** (`BLUR_COS`): a media alisa DENTRO de
// uma superficie e nao atravessa uma quina. ⛔ Ela nao e' separavel em rigor (um vizinho pode estar
// ligado na horizontal e nao na vertical) — a divergencia e' declarada do lado da CPU e vale o preco.
//
// ⚠️⚠️ **Os TRES canais correm no mesmo laco, com mascara por canal** — o raio e' por canal e o
// resultado e' identico a tres lacos separados (uma media de caixa sobre a mesma janela). *Tres
// passagens separadas triplicariam as leituras de vizinho para dar o mesmo numero.*
fn borra_mole(g: vec3<u32>, horizontal: bool) {
    if (s.mole == 0u) { return; }
    if (g.x >= s.w || g.y >= s.h) { return; }
    let i = g.y * s.w + g.x;
    let c = centro[i];
    // ⚠️ A MESMA truncagem da CPU (`raio_px.clamp(0, MAX) as i32`), que corta para zero.
    let r = vec3<i32>(clamp(s.mole_raio, vec3<f32>(0.0), vec3<f32>({MAX_RAIO_MOLE})));
    let rmax = max(r.x, max(r.y, r.z));
    let passo = passo_da_luz();
    for (var l: u32 = 0u; l < s.n_lamps; l = l + 1u) {
        var dst = base_do_mole(i, l);
        if (horizontal) { dst = base_do_mole_tmp(i, l); }
        var entrada = vec3<f32>(luz[i * passo + 1u + l]);
        if (!horizontal) {
            let b = base_do_mole_tmp(i, l);
            entrada = vec3<f32>(luz[b], luz[b + 1u], luz[b + 2u]);
        }
        // ⚠️ Um pixel que nao e' peca fica com a ENTRADA, como o `out = canal.to_vec()` da CPU.
        if (c.x < 0.0) {
            luz[dst] = entrada.x; luz[dst + 1u] = entrada.y; luz[dst + 2u] = entrada.z;
            continue;
        }
        var soma = vec3<f32>(0.0);
        var peso = vec3<f32>(0.0);
        for (var d = -rmax; d <= rmax; d = d + 1) {
            var xx = i32(g.x);
            var yy = i32(g.y) + d;
            if (horizontal) { xx = i32(g.x) + d; yy = i32(g.y); }
            if (xx < 0 || yy < 0 || xx >= i32(s.w) || yy >= i32(s.h)) { continue; }
            let j = u32(yy) * s.w + u32(xx);
            let cj = centro[j];
            if (cj.x < 0.0) { continue; }
            if (dot(c.yzw, cj.yzw) < BLUR_COS) { continue; }
            var amostra = vec3<f32>(luz[j * passo + 1u + l]);
            if (!horizontal) {
                let b = base_do_mole_tmp(j, l);
                amostra = vec3<f32>(luz[b], luz[b + 1u], luz[b + 2u]);
            }
            let dentro = vec3<f32>(
                f32(abs(d) <= r.x), f32(abs(d) <= r.y), f32(abs(d) <= r.z)
            );
            soma = soma + amostra * dentro;
            peso = peso + dentro;
        }
        let media = soma / max(peso, vec3<f32>(1.0));
        let res = select(entrada, media, peso > vec3<f32>(0.0));
        luz[dst] = res.x; luz[dst + 1u] = res.y; luz[dst + 2u] = res.z;
    }
}

@compute @workgroup_size(8, 8, 1)
fn borra_mole_h(@builtin(global_invocation_id) g: vec3<u32>) { borra_mole(g, true); }

@compute @workgroup_size(8, 8, 1)
fn borra_mole_v(@builtin(global_invocation_id) g: vec3<u32>) { borra_mole(g, false); }
";
