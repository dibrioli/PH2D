//! ⭐⭐⭐ **O QUE O OLHO VÊ, EM PIXELS DE ECRÃ** — filho da [`super`] (a assadura) para herdar as
//! fixturas dela.
//!
//! ⛔⛔⛔ **Este módulo nasceu do report do dono de 2026-09-17** (*«Como eu já havia dito muitas
//! vezes: Fast e Smooth estão sempre idênticos. Nada mudou»*), e a primeira coisa que ele mediu foi
//! que **o dono tinha razão**.
//!
//! ⚠️ **A régua de toda esta linha era o desvio ao CAMPO, em pixels da ARTE** — uma propriedade da
//! APROXIMAÇÃO. Ninguém vê uma aproximação: vê-se a tinta no ecrã. Medido aqui, as duas leis punham
//! cada texel a `0,04 px` uma da outra na mediana e `0,34 px` no pior ponto, na dobra que a cena
//! ship. *Uma régua que mede a fidelidade ao modelo não responde «isto muda alguma coisa aos
//! olhos?».*

use super::*;

/// A distância, em pixels de ecrã, do ponto `p` à polilinha `linha`.
fn dist_a_polilinha(p: [f64; 2], linha: &[[f64; 2]]) -> f64 {
    linha
        .windows(2)
        .map(|w| {
            let (a, b) = (w[0], w[1]);
            let (dx, dy) = (b[0] - a[0], b[1] - a[1]);
            let l2 = dx * dx + dy * dy;
            let t = if l2 > 0.0 {
                (((p[0] - a[0]) * dx + (p[1] - a[1]) * dy) / l2).clamp(0.0, 1.0)
            } else {
                0.0
            };
            (p[0] - (a[0] + t * dx)).hypot(p[1] - (a[1] + t * dy))
        })
        .fold(f64::INFINITY, f64::min)
}

/// ⏱️⏱️⏱️ **QUANTO É QUE O `Smooth` MUDA NA TELA, EM PIXELS** — a sonda do report do dono
/// (*«Fast e Smooth estão sempre idênticos»*, 2026-09-17).
///
/// ⚠️⚠️ **Todas as réguas desta linha mediam o desvio ao CAMPO**, que é uma propriedade da
/// APROXIMAÇÃO — e o dono não vê aproximação nenhuma: ele vê a SILHUETA desenhada. *Uma régua que
/// mede a fidelidade ao modelo não responde «isto muda alguma coisa aos olhos?».*
///
/// Esta mede a distância máxima, em **pixels de ecrã**, entre o contorno que o `Fast` desenha e o
/// que o `Smooth` desenha, por zoom.
///
/// `cargo test -p ph2d-app-vec --lib --profile smoke -- --ignored --nocapture quanto_o_smooth_muda`
#[test]
#[ignore = "MEDICAO, nao gate"]
fn quanto_o_smooth_muda_na_tela() {
    let (sim, e) = cena(super::super::super::ALTURA_PX, None);
    let (sm, p2l, pele) = campo_da_cena(&sim, e);
    let ossos = sm.ossos();
    let mut ws = pele.scratch();
    let mut campo =
        |q: [f64; 2], pesos: &[f64]| ponto_do_produto(&pele, p2l.apply(q), pesos, &mut ws);

    let p_fast: Vec<[f64; 2]> = (0..sm.mesh.rest.len())
        .map(|v| campo(sm.mesh.rest[v], sm.pesos_de(v)))
        .collect();

    let attrs = ph2d_poly2d::hermite_attrs(&sm.mesh, &sm.pesos, ossos);
    let lei = ph2d_poly2d::AttrLaw::Hermite { values: ossos };
    let (assada, attrs_assados, _) = ph2d_poly2d::refine_rest_by_attrs(
        &sm.mesh,
        &attrs,
        ossos * 3,
        lei,
        ph2d_poly2d::RefineOptions {
            tolerance_px: ph2d_skeleton_live::skin_bake::tolerancia_do_bind(&sm.mesh),
            max_pieces: sm.mesh.tris.len() * 8,
            adaptativo: true,
        },
    );
    let p_assada = posa_a_assada(&assada.rest, &attrs_assados, ossos * 3, ossos, &mut campo);

    println!(
        "\n  arte {}x{} px · Fast {} pecas · ASSADA {} pecas",
        sm.mesh.size[0],
        sm.mesh.size[1],
        sm.mesh.tris.len(),
        assada.tris.len()
    );
    println!("  a barra do OLHO e' 1 px de ecra~: abaixo dela as duas desenham o mesmo.\n");
    println!("  {:>5} | {:>10} | {:>12}", "zoom", "pior px", "mediana px");
    println!("  ------+------------+-------------");
    for zoom in [1.0_f64, 2.0, 4.0, 8.0, 16.0] {
        let a = silhueta(&sm.mesh, &p_fast, zoom);
        let b = silhueta(&assada, &p_assada, zoom);
        let mut todas = Vec::new();
        for ((_, fa), (_, as_)) in a.iter().zip(&b) {
            for p in as_ {
                todas.push(dist_a_polilinha(*p, fa));
            }
        }
        todas.sort_by(f64::total_cmp);
        let pior = todas.last().copied().unwrap_or(0.0);
        let mediana = todas[todas.len() / 2];
        println!("  {zoom:>5.0} | {pior:>10.3} | {mediana:>12.4}");
    }
    println!();
}

/// Onde a malha `m` (posada em `posed`) põe o ponto de REPOUSO `q`, em metros de mundo.
///
/// ⚠️ **É a lei do desenho, não uma aproximação dela:** cada triângulo aplica o afim que leva os
/// três cantos de repouso aos três posados, que é exactamente o que o rasterizador faz.
fn onde_a_malha_poe(m: &ph2d_poly2d::Mesh2d, posed: &[[f64; 2]], q: [f64; 2]) -> Option<[f64; 2]> {
    for t in &m.tris {
        let (i, j, k) = (t[0] as usize, t[1] as usize, t[2] as usize);
        let (a, b, c) = (m.rest[i], m.rest[j], m.rest[k]);
        let area = (b[0] - a[0]) * (c[1] - a[1]) - (b[1] - a[1]) * (c[0] - a[0]);
        if area.abs() < 1e-12 {
            continue;
        }
        let w1 = ((q[0] - a[0]) * (c[1] - a[1]) - (q[1] - a[1]) * (c[0] - a[0])) / area;
        let w2 = ((b[0] - a[0]) * (q[1] - a[1]) - (b[1] - a[1]) * (q[0] - a[0])) / area;
        let w0 = 1.0 - w1 - w2;
        if w0 < -1e-9 || w1 < -1e-9 || w2 < -1e-9 {
            continue;
        }
        return Some([
            w0 * posed[i][0] + w1 * posed[j][0] + w2 * posed[k][0],
            w0 * posed[i][1] + w1 * posed[j][1] + w2 * posed[k][1],
        ]);
    }
    None
}

/// ⏱️⏱️⏱️ **E QUANTO É QUE O `Smooth` MOVE A TINTA DE DENTRO** — a outra metade do report.
///
/// ⛔⛔ **A sonda da silhueta não responde a isto, e a razão é a FIXTURA:** o canvas desta cena é
/// **branco chapado**, logo a única coisa visível nele é o contorno. O report original do dono
/// (2026-09-10) era sobre um braço **PINTADO** — ali o que se vê é a tinta de dentro a esticar.
///
/// ⇒ esta mede, para uma grelha de pontos da imagem, quantos **pixels de ecrã** separam o sítio
/// onde o `Fast` põe aquele texel do sítio onde o `Smooth` o põe.
#[test]
#[ignore = "MEDICAO, nao gate"]
fn quanto_o_smooth_move_a_tinta_de_dentro() {
    for graus in [25.0_f32, 60.0, 90.0, 150.0] {
        println!("\n  ===== dobra {graus}° por junta =====");
        a_tinta_de_dentro(graus);
    }
}

fn a_tinta_de_dentro(graus: f32) {
    let (sim, e) = cena_dobrada(
        super::super::super::ALTURA_PX,
        None,
        ph2d_poly2d::GridOptions::default(),
        graus,
    );
    let (sm, p2l, pele) = campo_da_cena(&sim, e);
    let ossos = sm.ossos();
    let mut ws = pele.scratch();
    let mut campo =
        |q: [f64; 2], pesos: &[f64]| ponto_do_produto(&pele, p2l.apply(q), pesos, &mut ws);

    let p_fast: Vec<[f64; 2]> = (0..sm.mesh.rest.len())
        .map(|v| campo(sm.mesh.rest[v], sm.pesos_de(v)))
        .collect();
    let attrs = ph2d_poly2d::hermite_attrs(&sm.mesh, &sm.pesos, ossos);
    let lei = ph2d_poly2d::AttrLaw::Hermite { values: ossos };
    let (assada, attrs_assados, _) = ph2d_poly2d::refine_rest_by_attrs(
        &sm.mesh,
        &attrs,
        ossos * 3,
        lei,
        ph2d_poly2d::RefineOptions {
            tolerance_px: ph2d_skeleton_live::skin_bake::tolerancia_do_bind(&sm.mesh),
            max_pieces: sm.mesh.tris.len() * 8,
            adaptativo: true,
        },
    );
    let p_assada = posa_a_assada(&assada.rest, &attrs_assados, ossos * 3, ossos, &mut campo);

    // ⚠️⚠️ **E o CONTROLO: uma malha MUITO mais fina**, que é a melhor aproximação disponível do
    // campo verdadeiro. Sem ele não se distingue *«as duas erram o mesmo»* de *«as duas acertam»* —
    // e a segunda é a conclusão que este report obriga a considerar.
    let (fina, attrs_finos, _) = ph2d_poly2d::refine_rest_by_attrs(
        &sm.mesh,
        &attrs,
        ossos * 3,
        lei,
        ph2d_poly2d::RefineOptions {
            tolerance_px: ph2d_skeleton_live::skin_bake::tolerancia_do_bind(&sm.mesh) / 16.0,
            max_pieces: sm.mesh.tris.len() * 64,
            adaptativo: true,
        },
    );
    let p_fina = posa_a_assada(&fina.rest, &attrs_finos, ossos * 3, ossos, &mut campo);
    println!("  (o controlo e' uma malha de {} pecas)", fina.tris.len());
    let (w, h) = (f64::from(sm.mesh.size[0]), f64::from(sm.mesh.size[1]));
    const N: usize = 60;
    println!("\n  grelha de {N}x{N} pontos da imagem, em pixels de ECRA~ (zoom 1)\n");
    println!(
        "  {:>14} | {:>9} | {:>10}",
        "contra", "pior px", "mediana px"
    );
    println!("  ---------------+-----------+-----------");
    let mut d_entre = Vec::new();
    let mut d_fast = Vec::new();
    let mut d_assada = Vec::new();
    for j in 0..=N {
        for i in 0..=N {
            let q = [w * i as f64 / N as f64, h * j as f64 / N as f64];
            let (Some(a), Some(b)) = (
                onde_a_malha_poe(&sm.mesh, &p_fast, q),
                onde_a_malha_poe(&assada, &p_assada, q),
            ) else {
                continue;
            };
            let px = |u: [f64; 2], v: [f64; 2]| {
                ((u[0] - v[0]) * PX_POR_METRO).hypot((u[1] - v[1]) * PX_POR_METRO)
            };
            d_entre.push(px(a, b));
            if let Some(v) = onde_a_malha_poe(&fina, &p_fina, q) {
                d_fast.push(px(a, v));
                d_assada.push(px(b, v));
            }
        }
    }
    let resumo = |mut v: Vec<f64>| {
        v.sort_by(f64::total_cmp);
        (v.last().copied().unwrap_or(0.0), v[v.len() / 2])
    };
    for (nome, v) in [
        ("Fast x Smooth", d_entre),
        ("Fast x campo", d_fast),
        ("Smooth x campo", d_assada),
    ] {
        if v.is_empty() {
            continue;
        }
        let (pior, med) = resumo(v);
        println!("  {nome:>14} | {pior:>9.3} | {med:>10.4}");
    }
    println!();
}

/// ⛔⛔⛔ **O `Fast` JÁ DESENHA O CAMPO A MENOS DE MEIO PIXEL — e é por isso que o dono reporta
/// «Fast e Smooth estão sempre idênticos»** (2026-09-17, e ele disse que já o tinha dito muitas
/// vezes).
///
/// # ⚠️ Todas as réguas desta linha mediam a grandeza errada
///
/// Elas mediam o desvio ao CAMPO em pixels da ARTE (`0,4143` contra `0,1781`), que é uma
/// propriedade da **aproximação**. O dono não vê aproximação nenhuma: ele vê **pixels de ecrã**. E
/// medido ali, na dobra que a cena ship (`25°`) e no zoom `1`, a tinta muda de sítio **`0,04 px` na
/// mediana e `0,34 px` no pior ponto**. *Nenhum olho distingue um terço de pixel.*
///
/// ⛔⛔ **A premissa do botão MORREU e ninguém reconferiu.** Ele nasceu do report de 2026-09-10
/// (*«arestas retas ao dobrar»*), quando a malha do bind era uma grelha uniforme e os pesos eram
/// euclidianos. As duas waves que vieram a seguir — a **grelha graduada pelas articulações** e os
/// pesos do **padrão-ouro** com a lei de Hermite — curaram a faceta na própria malha do bind. ⇒ o
/// `Fast` passou a estar certo, e o `Smooth` ficou sem nada para corrigir. *§0.0: quem move o número
/// que tornava algo inalcançável tem de reconferir a nota — e aqui o número moveu-se por baixo de
/// uma feature inteira.*
///
/// # O que este gate afirma, e porque tem DUAS metades
///
/// 1. **O `Fast` está certo** — ele fica a menos de `MEIO PIXEL` do campo verdadeiro na dobra que a
///    cena ship. É esta metade que explica o report, e é ela que reprova no dia em que alguém
///    piorar a malha do bind.
/// 2. **E o `Smooth` NÃO é inútil em princípio** — numa dobra forte e com zoom, a diferença passa de
///    um pixel. Sem esta metade, alguém leria a primeira como *«apague o botão»*, que é uma decisão
///    de produto que este gate não tem autoridade para tomar.
///
/// ⚠️ **O «campo verdadeiro» é uma malha `64×` mais fina**, e não uma fórmula: é a melhor
/// aproximação disponível, e a barra é grosseira o bastante para a diferença entre ela e o limite
/// não contar.
#[test]
fn o_fast_ja_desenha_o_campo_a_menos_de_meio_pixel() {
    /// A dobra que a cena de smoke ship.
    const DOBRA_DO_PRODUTO: f32 = 25.0;
    /// Meio pixel de ecrã — a barra que as duas leis já prometem, agora na unidade do OLHO.
    const MEIO_PIXEL: f64 = 0.5;

    let (pior_fast, pior_entre) = separacao_na_tela(DOBRA_DO_PRODUTO, 1.0);
    assert!(
        pior_fast <= MEIO_PIXEL,
        "o `Fast` erra {pior_fast:.3} px de ecra~ contra o campo, acima de {MEIO_PIXEL} — a malha \
         do bind piorou, e o report «Fast e Smooth sao identicos» deixou de ter esta causa"
    );
    assert!(
        pior_entre <= 1.0,
        "as duas leis ja' se separam {pior_entre:.3} px no zoom de trabalho — a premissa deste gate \
         mudou e o report do dono passou a ter outra causa"
    );

    // ⛔ A METADE QUE IMPEDE A LEITURA ERRADA: numa dobra forte e com zoom, elas SEPARAM-SE.
    let (_, forte) = separacao_na_tela(150.0, 8.0);
    assert!(
        forte > 1.0,
        "nem a `150°` e zoom `8` as duas se separam mais de um pixel ({forte:.3}) — aí o botao nao \
         tem regime nenhum, e isso e' uma decisao de produto e nao um gate verde"
    );
}

/// `(pior desvio do `Fast` ao campo, pior separação entre `Fast` e `Smooth`)`, em pixels de ECRÃ.
fn separacao_na_tela(graus: f32, zoom: f64) -> (f64, f64) {
    let (sim, e) = cena_dobrada(
        super::super::super::ALTURA_PX,
        None,
        ph2d_poly2d::GridOptions::default(),
        graus,
    );
    let (sm, p2l, pele) = campo_da_cena(&sim, e);
    let ossos = sm.ossos();
    let mut ws = pele.scratch();
    let mut campo =
        |q: [f64; 2], pesos: &[f64]| ponto_do_produto(&pele, p2l.apply(q), pesos, &mut ws);
    let p_fast: Vec<[f64; 2]> = (0..sm.mesh.rest.len())
        .map(|v| campo(sm.mesh.rest[v], sm.pesos_de(v)))
        .collect();
    let attrs = ph2d_poly2d::hermite_attrs(&sm.mesh, &sm.pesos, ossos);
    let lei = ph2d_poly2d::AttrLaw::Hermite { values: ossos };
    let tau = ph2d_skeleton_live::skin_bake::tolerancia_do_bind(&sm.mesh);
    let refina = |t: f64, tecto: usize| {
        ph2d_poly2d::refine_rest_by_attrs(
            &sm.mesh,
            &attrs,
            ossos * 3,
            lei,
            ph2d_poly2d::RefineOptions {
                tolerance_px: t,
                max_pieces: sm.mesh.tris.len() * tecto,
                adaptativo: true,
            },
        )
    };
    let (assada, a_at, _) = refina(tau, 8);
    let p_assada = posa_a_assada(&assada.rest, &a_at, ossos * 3, ossos, &mut campo);
    let (fina, f_at, _) = refina(tau / 16.0, 64);
    let p_fina = posa_a_assada(&fina.rest, &f_at, ossos * 3, ossos, &mut campo);

    let (w, h) = (f64::from(sm.mesh.size[0]), f64::from(sm.mesh.size[1]));
    const N: usize = 40;
    let (mut pior_fast, mut pior_entre) = (0.0_f64, 0.0_f64);
    for j in 0..=N {
        for i in 0..=N {
            let q = [w * i as f64 / N as f64, h * j as f64 / N as f64];
            let (Some(a), Some(b), Some(v)) = (
                onde_a_malha_poe(&sm.mesh, &p_fast, q),
                onde_a_malha_poe(&assada, &p_assada, q),
                onde_a_malha_poe(&fina, &p_fina, q),
            ) else {
                continue;
            };
            let px = |u: [f64; 2], t: [f64; 2]| {
                ((u[0] - t[0]) * PX_POR_METRO * zoom).hypot((u[1] - t[1]) * PX_POR_METRO * zoom)
            };
            pior_fast = pior_fast.max(px(a, v));
            pior_entre = pior_entre.max(px(a, b));
        }
    }
    (pior_fast, pior_entre)
}
