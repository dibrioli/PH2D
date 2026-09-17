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

/// ⏱️⏱️ **A W2 DA F9 — QUANTOS OSSOS UM VÉRTICE DE FACTO USA?**
///
/// O número que DIMENSIONA o formato de vértice do *vertex shader* de pele: se um vértice for
/// influenciado por `K` ossos, o vértice carrega `K` índices e `K` pesos, e o resto do buffer é
/// desperdício pago em TODA sprite do app (o [`ph2d_render::QuadVertex`] é partilhado com o quad
/// simples).
///
/// ⚠️ **A pergunta é sobre pesos SIGNIFICATIVOS, não sobre não-zeros:** os pesos BBW são a solução
/// de um problema variacional sobre a arte inteira, logo quase todo vértice tem um resíduo minúsculo
/// em quase todo osso. O que interessa é quantos são precisos para reproduzir a pose **dentro da
/// barra que o produto já promete**.
///
/// `cargo test -p ph2d-app-vec --lib -- --ignored --nocapture quantos_ossos_um_vertice_usa`
#[test]
#[ignore = "MEDICAO, nao gate"]
fn quantos_ossos_um_vertice_usa() {
    let (sim, e) = cena(super::super::super::ALTURA_PX, None);
    let (sm, _p2l, _pele) = campo_da_cena(&sim, e);
    let ossos = sm.ossos();
    println!(
        "\n  bind: {} vertices x {ossos} ossos\n",
        sm.mesh.rest.len()
    );
    for corte in [0.0_f64, 1e-6, 1e-4, 1e-3, 1e-2] {
        let mut hist = vec![0usize; ossos + 1];
        let mut pior_resto = 0.0_f64;
        for v in 0..sm.mesh.rest.len() {
            let w = sm.pesos_de(v);
            let n = w.iter().filter(|x| x.abs() > corte).count();
            hist[n] += 1;
            let resto: f64 = w.iter().filter(|x| x.abs() <= corte).map(|x| x.abs()).sum();
            pior_resto = pior_resto.max(resto);
        }
        let dist: Vec<String> = hist
            .iter()
            .enumerate()
            .filter(|(_, c)| **c > 0)
            .map(|(n, c)| format!("{n}:{c}"))
            .collect();
        println!(
            "  corte {corte:>8.0e} | ossos por vertice {:<28} | pior peso descartado {pior_resto:.2e}",
            dist.join(" ")
        );
    }
    println!();
}

/// ⏱️ **A ESCADA τ ↔ PEÇAS NA ARTE REAL** — o número que decide se a W1 tem sujeito.
///
/// `cargo test -p ph2d-app-vec --lib -- --ignored --nocapture escada_da_assadura_na_arte_real`
#[test]
#[ignore = "MEDICAO, nao gate"]
fn escada_da_assadura_na_arte_real() {
    let (sim, e) = cena(super::super::super::ALTURA_PX, None);
    let (sm, _p2l, _pele) = campo_da_cena(&sim, e);
    let ossos = sm.ossos();
    let attrs = ph2d_poly2d::hermite_attrs(&sm.mesh, &sm.pesos, ossos);
    let lei = ph2d_poly2d::AttrLaw::Hermite { values: ossos };
    println!(
        "\n  bind: {} vertices x {ossos} ossos, {} pecas, arte {}x{} px (diagonal {:.0})\n",
        sm.mesh.rest.len(),
        sm.mesh.tris.len(),
        sm.mesh.size[0],
        sm.mesh.size[1],
        f64::from(sm.mesh.size[0]).hypot(f64::from(sm.mesh.size[1]))
    );
    println!("       tau |    pecas | x entrada |   desvio");
    println!("  ---------+----------+-----------+---------");
    for tau in [0.05_f64, 0.02, 0.01, 0.005, 0.002, 0.001] {
        let (m, _, r) = ph2d_poly2d::refine_rest_by_attrs(
            &sm.mesh,
            &attrs,
            ossos * 3,
            lei,
            ph2d_poly2d::RefineOptions {
                tolerance_px: tau,
                max_pieces: sm.mesh.tris.len() * 8,
                adaptativo: true,
            },
        );
        println!(
            "  {tau:>8.4} | {:>8} | {:>8.2}x | {:>8.5}",
            m.tris.len(),
            m.tris.len() as f64 / sm.mesh.tris.len() as f64,
            r.desvio.unwrap_or(f64::NAN)
        );
    }
    println!();
}

/// ⭐⭐⭐ **A W1 DA F9 NA ARTE REAL — a malha ASSADA no bind erra o campo menos do que o `Fast`, e
/// dentro da barra que as duas leis prometem.**
///
/// Corrido sobre a arte da cena do dono (`512 × 320`) e sobre os pesos BBW que o bind dela de facto
/// resolve — ⛔ não sobre um campo sintético.
///
/// # ⛔⛔ A régua desta wave NÃO é a silhueta, e a medição é que o disse
///
/// A fila pedia *«a régua da silhueta a mesma de hoje»*, e a 1.ª redacção deste gate obedeceu à
/// letra. Medido:
///
/// | desenho | peças | nós no topo | vai-e-volta | **desvio ao CAMPO** |
/// |---|---:|---:|---:|---:|
/// | `Fast` | `2 430` | `46` | `26,60°` | `0,4143 px` |
/// | `Smooth` (do quadro, zoom `8×`) | — | `64` | `26,71°` | `0,0881 px` |
/// | **assada** | `13 996` | `85` | `29,44°` | **`0,1781 px`** |
///
/// ⚠️⚠️ **O «vai-e-volta» CRESCE COM A DENSIDADE por construção** — ele soma a viragem absoluta ao
/// longo da polilinha da silhueta, e uma polilinha com mais nós segue melhor a curva verdadeira e
/// portanto acumula mais viragem. *Uma régua cuja janela segue o número que se está a variar não
/// pode ser a testemunha da variação* — a mesma lei que o espaçamento do pincel afiado já pagou.
///
/// ⇒ a régua com UNIDADE e com barra declarada é o **desvio ao campo** (`skinned_deviation`, em
/// pixels da arte), e a barra é a que as duas leis já prometem: [`TOLERANCIA_PX`] (`0,5 px`).
///
/// # O que os números dizem
///
/// A assada erra **`2,3×` menos** que o `Fast` e **`2,0×` mais** que o `Smooth` do quadro — e as
/// duas estão **dentro** de `0,5 px`. ⭐ E a diferença é esperada e não é um defeito: o `Smooth`
/// refina onde o erro está **naquela pose e naquele zoom**, e a assada é **independente da pose**
/// por construção. *Uma aproximação que serve todas as poses nunca bate, peça a peça, uma feita
/// para uma só* — e é precisamente por servir todas que ela se paga uma vez.
///
/// ⚠️ **O CONTROLO é o `Fast`:** ele tem de errar MAIS, senão a cena não contém o fenómeno e um
/// empate a três não afirmaria nada.
#[test]
fn a_malha_assada_no_bind_desenha_como_o_smooth_do_quadro() {
    let (sim, e) = cena(super::super::super::ALTURA_PX, None);
    let (sm, p2l, pele) = campo_da_cena(&sim, e);
    let ossos = sm.ossos();
    let mut ws = pele.scratch();
    let mut campo =
        |q: [f64; 2], pesos: &[f64]| ponto_do_produto(&pele, p2l.apply(q), pesos, &mut ws);

    // A ASSADURA — a lei, não a porta (ela lê uma env var que um teste não pode fixar).
    let (assada, pesos_assados) = ph2d_skeleton_live::skin_bake::assar(&sm.mesh, &sm.pesos, ossos)
        .expect("o campo de pesos BBW desta arte curva na articulacao");
    println!(
        "  bind {} -> assada {} pecas ({:.2}x)",
        sm.mesh.tris.len(),
        assada.tris.len(),
        assada.tris.len() as f64 / sm.mesh.tris.len() as f64
    );

    let zoom = 8.0_f64;
    let posa = |m: &ph2d_poly2d::Mesh2d,
                pesos: &[f64],
                campo: &mut dyn FnMut([f64; 2], &[f64]) -> [f64; 2]| {
        m.rest
            .iter()
            .enumerate()
            .map(|(v, &q)| campo(q, &pesos[v * ossos..(v + 1) * ossos]))
            .collect::<Vec<_>>()
    };
    let p_fast = posa(&sm.mesh, &sm.pesos, &mut campo);
    let p_assada = posa(&assada, &pesos_assados, &mut campo);
    let smooth = ph2d_skeleton_live::skin_refine::refine_skinned(
        &sm.mesh,
        &sm.pesos,
        ossos,
        &mut campo,
        opcoes(true, zoom),
    );

    // ⭐⭐⭐ **O DISCRIMINADOR: quanto cada malha erra o CAMPO VERDADEIRO** (a régua do
    // `skinned_deviation`, em pixels da arte). A silhueta mede ONDULAÇÃO, que cresce com a
    // densidade por construção — *uma régua cuja janela segue o número que se está a variar não
    // pode ser a única testemunha da variação*, a lei que o espaçamento do pincel afiado já pagou.
    let desvio = |m: &ph2d_poly2d::Mesh2d,
                  pesos: &[f64],
                  posed: &[[f64; 2]],
                  campo: &mut dyn FnMut([f64; 2], &[f64]) -> [f64; 2]| {
        let lei = ph2d_skeleton_live::skin_refine::weight_law(ossos, true);
        let attrs = ph2d_skeleton_live::skin_refine::weight_attrs(m, pesos, lei);
        ph2d_skeleton_live::skin_refine::skinned_deviation(m, posed, &attrs, lei, campo)
            * PX_POR_METRO
    };
    let d_fast = desvio(&sm.mesh, &sm.pesos, &p_fast, &mut campo);
    let d_assada = desvio(&assada, &pesos_assados, &p_assada, &mut campo);
    let d_smooth = ph2d_skeleton_live::skin_refine::skinned_deviation(
        &smooth.mesh,
        &smooth.posed,
        &smooth.attrs,
        smooth.law,
        &mut campo,
    ) * PX_POR_METRO;
    println!(
        "  desvio ao CAMPO (px da arte): Fast {d_fast:.4} | Smooth {d_smooth:.4} | ASSADA {d_assada:.4}"
    );

    let l_fast = silhueta(&sm.mesh, &p_fast, zoom);
    let l_smooth = silhueta(&smooth.mesh, &smooth.posed, zoom);
    let l_assada = silhueta(&assada, &p_assada, zoom);
    for ((nome, f), ((_, s), (_, a))) in l_fast.iter().zip(l_smooth.iter().zip(&l_assada)) {
        println!(
            "  {nome:>4} | Fast {:>6.2} ({} nos) | Smooth {:>6.2} ({} nos) | ASSADA {:>6.2} ({} nos)",
            {
                let (n, t, _) = vai_e_volta(f);
                t - n.abs()
            },
            f.len(),
            {
                let (n, t, _) = vai_e_volta(s);
                t - n.abs()
            },
            s.len(),
            {
                let (n, t, _) = vai_e_volta(a);
                t - n.abs()
            },
            a.len(),
        );
    }
    assert!(
        d_assada <= ph2d_skeleton_live::skin_bake::TOLERANCIA_PX,
        "a malha ASSADA erra {d_assada:.4} px do campo, acima da barra de {} px que a lei promete",
        ph2d_skeleton_live::skin_bake::TOLERANCIA_PX
    );
    assert!(
        d_assada < d_fast,
        "a malha ASSADA ({d_assada:.4} px) nao bate a malha do bind sem assar ({d_fast:.4} px) — \
         a assadura nao esta' a comprar nada"
    );
    // ⛔ **O CONTROLO:** sem uma malha crua que erre MAIS do que a barra, a asserção de cima
    // passaria sobre qualquer coisa.
    assert!(
        d_fast > d_assada * 2.0,
        "o `Fast` erra {d_fast:.4} px contra {d_assada:.4} da assada — a cena deixou de conter o \
         fenomeno, e um empate nao afirma nada"
    );
    // ⚠️ E o `Smooth` do quadro fica NOMEADO: ele erra menos por ser feito para ESTA pose.
    assert!(
        d_smooth <= d_assada,
        "o Smooth do quadro ({d_smooth:.4}) passou a errar MAIS que a assada ({d_assada:.4}) — \
         a leitura escrita no doc deste gate deixou de valer"
    );
}

/// A instância que o extract emite para a sprite `s` — o quad DELA, com a âncora resolvida.
///
/// ⚠️ Escrita aqui porque a sonda precisa de um `present` e a gémea dela vive na `ph2d-skeleton-live`
/// dentro de `#[cfg(test)]`, logo é invisível daqui (a armadilha §2.7 do HOWTO). As duas descrevem a
/// mesma coisa; o que esta sonda MEDE não depende de nenhum dos campos que elas pudessem divergir.
fn instancia_da_sonda(s: &ph2d_render::Sprite) -> ph2d_render::RenderInstance {
    ph2d_render::RenderInstance {
        world_pos: [0.0, 0.0],
        size: s.size,
        atlas_uv: [0.0, 0.0, 1.0, 1.0],
        tint: [1.0; 4],
        basis: ph2d_render::RenderInstance::IDENTITY_BASIS,
        texture_id: 0,
        premultiplied: 0.0,
        anchor: s.resolve_anchor(PPM),
        per_corner_tint: [[1.0; 4]; 4],
        opacity: 1.0,
        flip_uv: ph2d_render::RenderInstance::pack_flip_flags(s.flip_x, s.flip_y, false),
        z_order: 0,
        sampling: 0,
        uv_xform: ph2d_render::RenderInstance::IDENTITY_UV_XFORM,
        clip_group: ph2d_render::RenderInstance::CLIP_GROUP_NONE,
        clip_meta: 0,
        sub_order: 0,
    }
}

/// `(mínimo, mediana)` de amostras de relógio, em ms — o MÍNIMO é a leitura que sobrevive à carga
/// de fundo desta máquina (`CLAUDE.md` §5.0); a mediana ao lado diz quanto ela estava a roubar.
fn melhor_ms(mut ms: Vec<f64>) -> (f64, f64) {
    ms.sort_by(f64::total_cmp);
    (ms[0], ms[ms.len() / 2])
}

/// ⏱️⏱️⏱️ **O QUE UM QUADRO CUSTA COM A MALHA ASSADA — a medição que decide se a W2 (a placa) é
/// PRÉ-REQUISITO ou OPTIMIZAÇÃO.**
///
/// A W1 tirou o REFINAMENTO do quadro e pô-lo no bind. O que o quadro ainda paga, por imagem, é
/// **descodificar a malha guardada, deformar cada vértice e montar o `SpriteMesh`** — o `Fast`, a
/// `0,024 µs` por peça (F6-t). A assada é `5,76×` mais densa, logo esse custo multiplica por `5,76`
/// — e é essa multiplicação que esta sonda mede, na ARTE REAL e por número de imagens.
///
/// # ⚠️ A pergunta é de PRODUTO, e nunca foi medida
///
/// O cabeçalho do [`ph2d_skeleton_live::skin_bake`] afirma que a porta tem de nascer DESLIGADA
/// porque *«um interruptor que nasce ligado antes de a W2 pagar por ele poria mais vértices para a
/// CPU deformar por quadro»*. Isso é verdade sobre a DIRECÇÃO e não diz o NÚMERO: se o quadro
/// couber, o dono ganha *«o alisamento em qualquer cena»* HOJE e a W2 passa a ser o que tira esse
/// custo da CPU; se não couber, a W2 é pré-requisito e a porta fica fechada. *§0.0: meça antes de
/// limitar.*
///
/// ⚠️ **As `n` imagens são CÓPIAS do mesmo bind**, componente a componente — a mesma arte, os
/// mesmos ossos, a mesma tabela de pesos. É o modelo honesto de *«um personagem de muitas partes»*
/// para esta medição, porque o custo por imagem é função das peças dela e de mais nada.
///
/// `cargo test -p ph2d-app-vec --lib --profile smoke -- --ignored --nocapture o_que_um_quadro_custa`
#[test]
#[ignore = "MEDICAO, nao gate"]
fn o_que_um_quadro_custa_com_a_malha_assada() {
    use ph2d_ecs::{PresentWorld, SimRef, Transform};
    use ph2d_render::{Sprite, SpriteMesh};
    use ph2d_skeleton_live::skin_image::{SKIN_FRAME_PIECES, attach_skin_meshes};
    use std::time::Instant;

    const ZOOM: f64 = 8.0;
    const QUADRO_MS: f64 = 16.667;
    const RONDAS: usize = 30;

    let carga = std::fs::read_to_string("/proc/loadavg").unwrap_or_default();
    let porta = std::env::var("PH2D_SKIN_BAKE").unwrap_or_else(|_| "<fechada>".to_owned());
    println!(
        "\n  carga: {} · PH2D_SKIN_BAKE={porta}",
        carga
            .split_whitespace()
            .take(3)
            .collect::<Vec<_>>()
            .join(" ")
    );

    let (sim0, e0) = cena(super::super::super::ALTURA_PX, None);
    let (sm, _p2l, _pele) = campo_da_cena(&sim0, e0);
    let entrada_pecas = sm.mesh.tris.len();
    // ⏱️ **O que a ASSADURA custa, uma vez por bind** — ela não entra no quadro, e é por isso que
    // tem de ser medida à parte: um custo pago uma vez não se compara com um pago 60 vezes por
    // segundo, e confundi-los é o que faz um número parecer proibitivo ou barato ao contrário.
    let mut assaduras = Vec::with_capacity(5);
    for _ in 0..5 {
        let t = Instant::now();
        let _ = ph2d_skeleton_live::skin_bake::assar(&sm.mesh, &sm.pesos, sm.ossos());
        assaduras.push(t.elapsed().as_secs_f64() * 1e3);
    }
    let (assadura_min, assadura_med) = melhor_ms(assaduras);
    let assada_pecas = ph2d_skeleton_live::skin_bake::assar(&sm.mesh, &sm.pesos, sm.ossos())
        .expect("a assadura parte alguma coisa nesta arte")
        .0
        .tris
        .len();
    println!(
        "  arte {}x{} px · bind {entrada_pecas} pecas · assada {assada_pecas} pecas ({:.2}x) · \
         orcamento do quadro {SKIN_FRAME_PIECES} pecas",
        sm.mesh.size[0],
        sm.mesh.size[1],
        assada_pecas as f64 / entrada_pecas as f64
    );
    println!(
        "  assar UMA vez (por bind, fora do quadro): {assadura_min:.3}/{assadura_med:.3} ms\n"
    );

    println!(
        "  {:>3} | {:>6} | {:>13} | {:>7} | {:>8}",
        "img", "lei", "ms (min/med)", "pecas", "% quadro"
    );
    println!("  ----+--------+---------------+---------+---------");

    for n in [1_usize, 4, 8] {
        for (lei, smooth) in [
            ("Fast", None),
            (
                "Smooth",
                Some(ph2d_poly2d::RefineOptions {
                    tolerance_px: 0.5,
                    max_pieces: SKIN_FRAME_PIECES,
                    adaptativo: true,
                }),
            ),
        ] {
            // ⚠️ **A fonte é a do BIND, sempre** — a assadura chega pelo caminho do PRODUTO (o memo
            // do `Smooth`), e não escrita à mão aqui. *Uma sonda que planta o resultado na entrada
            // mede a lei que ela própria escreveu, nunca a que o quadro corre.*
            let (mut sim, e0) = cena(super::super::super::ALTURA_PX, None);
            let bind = sim
                .world()
                .get::<ph2d_skeleton_ecs::SkinBind>(e0)
                .expect("pele do bind")
                .clone();
            let sprite = *sim.world().get::<Sprite>(e0).expect("sprite da arte");
            let mut artes = vec![e0];
            for _ in 1..n {
                let e = sim.world_mut().spawn((Transform::IDENTITY, sprite)).id();
                sim.world_mut().entity_mut(e).insert(bind.clone());
                artes.push(e);
            }
            let mut present = PresentWorld::new();
            let instancias: Vec<_> = artes
                .iter()
                .map(|&e| {
                    present
                        .world_mut()
                        .spawn((SimRef(e), instancia_da_sonda(&sprite)))
                        .id()
                })
                .collect();

            // ⚠️ **Uma passagem a MORNO antes do relógio:** a assadura é paga uma vez por bind e já
            // tem coluna própria acima — deixá-la cair dentro da 1.ª ronda mediria as duas coisas
            // somadas e chamaria isso de custo por quadro.
            attach_skin_meshes(&sim, &mut present, PPM, smooth, PX_POR_METRO * ZOOM, &[]);

            let mut ms = Vec::with_capacity(RONDAS);
            for _ in 0..RONDAS {
                for p in &instancias {
                    present.world_mut().entity_mut(*p).remove::<SpriteMesh>();
                }
                let t = Instant::now();
                attach_skin_meshes(&sim, &mut present, PPM, smooth, PX_POR_METRO * ZOOM, &[]);
                ms.push(t.elapsed().as_secs_f64() * 1e3);
            }
            let saiu: usize = instancias
                .iter()
                .map(|p| {
                    present
                        .world()
                        .get::<SpriteMesh>(*p)
                        .map_or(0, |m| m.tris.len())
                })
                .sum();
            let (min, med) = melhor_ms(ms);
            println!(
                "  {n:>3} | {lei:>6} | {min:>6.3}/{med:<6.3} | {saiu:>7} | {:>6.1} %",
                min / QUADRO_MS * 100.0
            );
        }
    }
    println!();
}
