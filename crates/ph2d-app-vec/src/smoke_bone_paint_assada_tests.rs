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

/// O CAMPO de deformação desta cena: `(repouso, pesos) → posado`.
///
/// ⚠️ Ele é um `dyn` e não um genérico porque as duas sondas o passam ADIANTE (a
/// [`posa_a_assada`] recebe-o já construído por quem tem a pele na mão), e um `impl FnMut` não
/// atravessa essa fronteira sem monomorfizar cada chamador.
type Campo<'a> = dyn FnMut([f64; 2], &[f64]) -> [f64; 2] + 'a;

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
    campo: &mut Campo<'_>,
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
    let posa = |m: &ph2d_poly2d::Mesh2d, pesos: &[f64], campo: &mut Campo<'_>| {
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
    let desvio =
        |m: &ph2d_poly2d::Mesh2d, pesos: &[f64], posed: &[[f64; 2]], campo: &mut Campo<'_>| {
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

/// ⭐⭐⭐ **A CENA LOTADA CONTÉM O FENÓMENO QUE ELA EXISTE PARA MOSTRAR.**
///
/// O report que abriu a F9 é *«uma cena com muita arte presa fica com o `Smooth` igual ao `Fast`»*,
/// e a causa é a soma das malhas presas passar o orçamento de refinamento do quadro. ⚠️ **Uma cena
/// que não lá chegue ensina o contrário do que diz** — o dono carregaria em `Smooth`, veria a
/// diferença e concluiria que o defeito nunca existiu.
///
/// ⚠️ **As duas metades:** a cena de UM canvas fica ABAIXO do orçamento (é a que o dono já aprovou,
/// e ela não é o sujeito desta pergunta) · e o TECTO do roteador tem de a levar ACIMA dele.
///
/// ⚠️ A contagem por canvas sai do BIND do produto, nunca de um número escrito aqui.
#[test]
fn a_cena_lotada_passa_o_orcamento_do_quadro() {
    use ph2d_skeleton_live::skin_image::SKIN_FRAME_PIECES;
    let (sim, e) = cena(super::super::super::ALTURA_PX, None);
    let (sm, _p2l, _pele) = campo_da_cena(&sim, e);
    let por_canvas = sm.mesh.tris.len();
    assert!(
        por_canvas <= SKIN_FRAME_PIECES,
        "um canvas sozinho ({por_canvas} pecas) ja' passa o orcamento ({SKIN_FRAME_PIECES}) — a \
         cena que o dono aprovou mudou de regime, e o roteiro de 8 passos dela deixou de valer"
    );
    let tecto = por_canvas * super::super::super::NIVEIS as usize;
    assert!(
        tecto > SKIN_FRAME_PIECES,
        "com o tecto do roteador ({} canvas) a cena guarda {tecto} pecas e o orcamento e' \
         {SKIN_FRAME_PIECES} — ela NAO chega ao regime do report, e o smoke nao mostra nada",
        super::super::super::NIVEIS
    );
}

/// ⛔ **O NÍVEL É UMA CONTAGEM, e o caminho de omissão é a cena de UM canvas.**
///
/// ⚠️ **O `=` vazio é o caso que morde:** `env VAR=` **define** a variável com a string vazia (a
/// armadilha que esta casa já pagou num arnês de mutação), e ali a resposta certa é `1` e não zero
/// — *uma cena com zero canvas monta e não demonstra nada.*
#[test]
fn o_nivel_e_uma_contagem_e_o_omisso_e_um_canvas() {
    use super::super::super::{NIVEIS, quantos_de};
    assert_eq!(quantos_de(None), 1, "sem env a cena e' a de sempre");
    assert_eq!(
        quantos_de(Some("")),
        1,
        "`env VAR=` define a variavel VAZIA"
    );
    assert_eq!(quantos_de(Some("1")), 1);
    assert_eq!(quantos_de(Some(" 4 ")), 4);
    assert_eq!(quantos_de(Some("0")), 1, "zero canvas nao e' uma cena");
    assert_eq!(quantos_de(Some("99")), NIVEIS, "o tecto e' o do roteador");
    assert_eq!(
        quantos_de(Some("sim")),
        1,
        "um valor ilegivel cai no omisso"
    );
}

/// ⛔⛔ **COM UM CANVAS A CENA É A DE SEMPRE, AO BIT** — é isto que mantém o roteiro de 8 passos que
/// o dono aprovou.
///
/// ⚠️ A régua é a POSIÇÃO, que é a única coisa que a contagem podia mover: a coluna centra-se na
/// origem, logo com `n = 1` o único canvas tem de ficar exactamente onde ele ficava.
#[test]
fn com_um_canvas_a_cena_fica_onde_sempre_esteve() {
    use super::super::super::centro_do_canvas;
    assert_eq!(centro_do_canvas(0, 1, PPM), [0.0, 0.0]);
    // E o CONTROLO: com mais de um, eles separam-se — senão esta metade passaria sobre uma coluna
    // que põe os seis no mesmo sítio.
    let a = centro_do_canvas(0, 4, PPM);
    let b = centro_do_canvas(1, 4, PPM);
    assert!(
        (a[0] - b[0]).abs() > f64::from(super::super::super::LARGURA_PX) / f64::from(PPM),
        "dois canvas da cena lotada ficaram a menos de uma largura um do outro: {a:?} e {b:?}"
    );
    // ⛔ E a fileira é DEITADA: uma coluna põe a arte dobrada por cima do enquadramento (medido por
    // foto), e a régua que o diz é o `y` ficar igual.
    assert!(
        (a[1] - b[1]).abs() < 1e-9,
        "a cena lotada voltou a ser uma COLUNA: {a:?} e {b:?}"
    );
}

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
