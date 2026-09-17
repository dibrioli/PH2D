//! ⏱️⏱️ **A W0 DA F9 — UMA MALHA ASSADA UMA VEZ ALISA A DOBRA COMO O REFINAMENTO POR QUADRO?**
//!
//! Filha da [`super`] (a silhueta) para herdar as fixturas e as duas réguas; a pergunta é a que a
//! fila do módulo manda medir **antes de qualquer código** (`docs/Skeleton/01_a_fila.md`, F9 W0):
//!
//! > *uma malha fixa assada em repouso alisa a dobra FORTE como o refinamento por quadro alisa?*
//!
//! # Porque esta é a pergunta DECISIVA, e não uma entre várias
//!
//! A direcção inteira da F9 — a densidade sai do QUADRO e vai para o BIND, e a deformação passa
//! para o *vertex shader* — assenta numa única propriedade: **uma topologia só tem de servir todas
//! as poses**. Se ela servir, o resto é engenharia (enviar poses, costurar os leitores da malha).
//! Se não servir, a direcção cai e as ondas W1–W4 medem outra coisa.
//!
//! ⚠️ **E ela mede-se sem inventar lei nenhuma.** Assar *no pior caso* (a dobra mais forte, no zoom
//! mais fino) usa o refinamento que já existe e responde à pergunta da topologia. O critério
//! proposto na fila — refinar onde o CAMPO DE PESOS curva — é uma forma de escolher ONDE partir com
//! menos peças, e só faz sentido medir depois de se saber que uma topologia fixa chega.
//!
//! # ⛔ O CONTROLO que torna a comparação válida
//!
//! Re-posar uma malha assada noutra cena só significa alguma coisa se o BIND das duas cenas for o
//! mesmo — mesmas posições de repouso, mesmos pesos. Se a `cena_dobrada` mexesse no bind ao dobrar,
//! a tabela compararia duas artes diferentes e leria isso como qualidade. ⇒ a sonda **afirma-o
//! primeiro**, e pára se não for verdade.
//!
//! # O que cada coluna é
//!
//! * **peças** — triângulos entregues. Na assada é um número só, o mesmo em toda a linha: é ele que
//!   vira memória de GPU (a W4 da fila).
//! * **faceta** — `skinned_deviation`, o desvio da malha contra o campo que ela segue, em pixels de
//!   ECRÃ (já multiplicado pelo zoom).
//! * **canto** — o maior canto da silhueta desenhada (`vai_e_volta`), que é a régua do OLHO e a que
//!   apanhou o report de 2026-09-16. ⚠️ Ela não sabe que campo existe, de propósito.

use super::*;

/// Os pesos de um vértice DENTRO dos atributos refinados — os `ossos` primeiros do passo.
///
/// ⚠️ **O passo não é `ossos`**: a lei de Hermite guarda gradientes ao lado dos pesos, e é por isso
/// que o produto faz `w.get(..ossos)` antes de chamar o campo (`ph2d_skeleton_live::skin_refine`).
fn pesos_no_passo(attrs: &[f64], passo: usize, v: usize, ossos: usize) -> &[f64] {
    &attrs[v * passo..v * passo + ossos]
}

/// ⭐⭐⭐ **O QUE O *VERTEX SHADER* FARIA** — cada vértice da malha assada posado a partir do
/// **repouso dele e dos pesos dele**, na cena de agora.
///
/// ⚠️ É exactamente a conta da F9: por vértice, repouso + pesos (enviados quando a malha muda); por
/// quadro, só as poses dos ossos. Nada aqui olha para a pose em que a malha foi assada.
fn posa_a_assada(
    rest: &[[f64; 2]],
    attrs: &[f64],
    passo: usize,
    ossos: usize,
    campo: &mut dyn FnMut([f64; 2], &[f64]) -> [f64; 2],
) -> Vec<[f64; 2]> {
    (0..rest.len())
        .map(|v| campo(rest[v], pesos_no_passo(attrs, passo, v, ossos)))
        .collect()
}

/// O maior canto da silhueta desenhada de uma malha posada.
fn maior_canto(m: &ph2d_poly2d::Mesh2d, posed: &[[f64; 2]], zoom: f64) -> f64 {
    silhueta(m, posed, zoom)
        .iter()
        .map(|(_, p)| vai_e_volta(p).2)
        .fold(0.0_f64, f64::max)
}

/// ⏱️ **SONDA (`--ignored`) — a tabela da W0.**
///
/// ```text
/// bash scripts/ph2d-run.sh cargo test -p ph2d-app-vec --lib -- --ignored --nocapture sonda_a_malha_assada
/// ```
#[test]
#[ignore = "sonda: imprime a tabela, sem barra"]
fn sonda_a_malha_assada_contra_o_refinamento_por_quadro() {
    const DOBRAS: [f32; 5] = [25.0, 60.0, 90.0, 120.0, 150.0];
    const ZOOMS: [f64; 4] = [1.0, 4.0, 8.0, 16.0];
    // A pose e o zoom em que a malha é ASSADA: o pior caso desta varredura.
    const DOBRA_DA_ASSADURA: f32 = 150.0;
    const ZOOM_DA_ASSADURA: f64 = 16.0;

    let cena_de = |graus: f32| {
        cena_dobrada(
            super::super::super::ALTURA_PX,
            None,
            ph2d_poly2d::GridOptions::default(),
            graus,
        )
    };

    // ─── O CONTROLO: o bind é o MESMO em todas as dobras ──────────────────────────────────────
    let (sim0, e0) = cena_de(DOBRAS[0]);
    let (sm0, _, _) = campo_da_cena(&sim0, e0);
    let (rest0, pesos0, tris0) = (
        sm0.mesh.rest.clone(),
        sm0.pesos.clone(),
        sm0.mesh.tris.clone(),
    );
    for &g in &DOBRAS[1..] {
        let (sim, e) = cena_de(g);
        let (sm, _, _) = campo_da_cena(&sim, e);
        assert_eq!(
            (sm.mesh.rest.len(), sm.pesos.len(), sm.mesh.tris.len()),
            (rest0.len(), pesos0.len(), tris0.len()),
            "a {g} graus o BIND tem outro tamanho — a sonda compararia duas artes diferentes"
        );
        let pior_r = sm
            .mesh
            .rest
            .iter()
            .zip(&rest0)
            .map(|(a, b)| (a[0] - b[0]).hypot(a[1] - b[1]))
            .fold(0.0_f64, f64::max);
        let pior_w = sm
            .pesos
            .iter()
            .zip(&pesos0)
            .map(|(a, b)| (a - b).abs())
            .fold(0.0_f64, f64::max);
        assert!(
            pior_r < 1e-12 && pior_w < 1e-12,
            "a {g} graus o BIND mudou (repouso {pior_r:.3e}, pesos {pior_w:.3e}) — dobrar a cena \
             tem de mexer so' nas POSES dos ossos, senao esta sonda nao compara a mesma arte"
        );
    }
    println!(
        "CONTROLO: o bind e' o mesmo nas {} dobras ({} vertices, {} pecas)\n",
        DOBRAS.len(),
        rest0.len(),
        tris0.len()
    );

    // ─── A ASSADURA: uma vez, no pior caso ────────────────────────────────────────────────────
    let (sim_a, e_a) = cena_de(DOBRA_DA_ASSADURA);
    let (sm_a, p2l_a, pele_a) = campo_da_cena(&sim_a, e_a);
    let ossos = sm_a.ossos();
    let mut ws_a = pele_a.scratch();
    let mut campo_a =
        |q: [f64; 2], pesos: &[f64]| ponto_do_produto(&pele_a, p2l_a.apply(q), pesos, &mut ws_a);
    let assada = ph2d_skeleton_live::skin_refine::refine_skinned(
        &sm_a.mesh,
        &sm_a.pesos,
        ossos,
        &mut campo_a,
        opcoes(true, ZOOM_DA_ASSADURA),
    );
    let passo = assada
        .attrs
        .len()
        .checked_div(assada.mesh.rest.len())
        .unwrap_or(0);
    println!(
        "ASSADA a {DOBRA_DA_ASSADURA} graus, zoom {ZOOM_DA_ASSADURA}x: {} pecas, {} vertices \
         (bind: {} pecas) | tecto do quadro: {} pecas",
        assada.mesh.tris.len(),
        assada.mesh.rest.len(),
        tris0.len(),
        ph2d_skeleton_live::skin_image::SKIN_FRAME_PIECES
    );
    println!(
        "\n{:>5} {:>5} | {:>8} {:>9} {:>8} | {:>8} {:>9} {:>8}",
        "dobra", "zoom", "pf.pecas", "pf.faceta", "pf.canto", "as.pecas", "as.faceta", "as.canto"
    );

    for &graus in &DOBRAS {
        for &zoom in &ZOOMS {
            let (sim, e) = cena_de(graus);
            let (sm, p2l, pele) = campo_da_cena(&sim, e);
            let mut ws = pele.scratch();
            let mut campo =
                |q: [f64; 2], pesos: &[f64]| ponto_do_produto(&pele, p2l.apply(q), pesos, &mut ws);

            // O produto de hoje: refina a malha do bind NESTA pose, neste zoom.
            let pf = ph2d_skeleton_live::skin_refine::refine_skinned(
                &sm.mesh,
                &sm.pesos,
                ossos,
                &mut campo,
                opcoes(true, zoom),
            );
            let pf_faceta = ph2d_skeleton_live::skin_refine::skinned_deviation(
                &pf.mesh, &pf.posed, &pf.attrs, pf.law, &mut campo,
            ) * PX_POR_METRO
                * zoom;
            let pf_canto = maior_canto(&pf.mesh, &pf.posed, zoom);

            // A assada: a MESMA topologia, posada do repouso e dos pesos.
            let posed = posa_a_assada(&assada.mesh.rest, &assada.attrs, passo, ossos, &mut campo);
            let as_faceta = ph2d_skeleton_live::skin_refine::skinned_deviation(
                &assada.mesh,
                &posed,
                &assada.attrs,
                assada.law,
                &mut campo,
            ) * PX_POR_METRO
                * zoom;
            let as_canto = maior_canto(&assada.mesh, &posed, zoom);

            println!(
                "{graus:>5} {zoom:>5} | {:>8} {pf_faceta:>9.3} {pf_canto:>8.2} | {:>8} \
                 {as_faceta:>9.3} {as_canto:>8.2}",
                pf.mesh.tris.len(),
                assada.mesh.tris.len(),
            );
        }
    }
}
