//! **AS SONDAS DA FOTO** — o que o artista viu, em números.
//!
//! ⭐⭐ **O corte contra o [`super::retopo_field_probes`] foi forçado pela HR-18**
//! (683 contra 600), e é de ASSUNTO: lá as sondas do **campo** (o relevo, o peso do
//! alinhamento); aqui as do **defeito geométrico** — a aresta que atravessa a peça e
//! o leque que se fecha num ponto.
//!
//! ⛔ **Elas nascem de três fotos e da palavra «muito ruim»** (2026-08-22), sobre
//! uma saída que passava em **todas** as réguas desta linha: 100 % de quads, casca
//! fechada, `χ` exacta, densidade no alvo, 15 irregulares, valência máxima 6 e a
//! forma preservada a `0,085 %` da diagonal. *Uma peça pode passar em toda asserção
//! e estar visivelmente destruída.*

/// ⭐⭐ **SONDA — O QUE A FOTO MOSTRA, em números** (as três de 2026-08-22).
///
/// ⛔ **O artista mandou três fotos e a palavra «muito ruim», e NENHUMA régua desta
/// linha piscou.** 100 % de quads · casca fechada · característica de Euler exacta ·
/// densidade no alvo · contagem de irregulares perto do chão. *Uma peça pode passar
/// em toda asserção e estar visivelmente destruída* — e é a terceira vez que esta
/// linha paga essa lição.
///
/// As duas setas verdes apontam para coisas diferentes, e esta sonda separa-as:
///
/// | o que se vê na foto | a grandeza que o mede |
/// |---|---|
/// | ⭐ **o LEQUE** — vinte e tal arestas a convergir num ponto, em plena região lisa | a **valência máxima** de um vértice da saída |
/// | ⭐ **o RASGO** — o entalhe no vinco da orelha | a **aresta máxima** em fração da diagonal da peça |
/// | a faixa comprimida ao lado do leque | a razão entre a aresta **máxima** e a **mediana** |
///
/// ⚠️ **A valência não é a contagem de irregulares.** Vinte vértices de valência 5
/// são invisíveis; **um** de valência 20 é o defeito da foto — e as duas coisas dão
/// o mesmo número na coluna `irregular`.
///
/// ```text
/// \
///   cargo test -p ph2d-host-desktop --release --bins \
///   what_do_the_photos_measure -- --ignored --nocapture
/// ```
#[test]
#[ignore = "sonda -- o que as fotos do artista medem"]
fn what_do_the_photos_measure() {
    use std::collections::BTreeMap;

    for (name, reference) in [
        ("ORELHA", crate::sculpt3d::fixtures::eared_sphere()),
        ("GANCHO", crate::sculpt3d::fixtures::hooked_sphere()),
        ("ENRUGADA", crate::sculpt3d::fixtures::wrinkled_sphere()),
    ] {
        let diag = {
            let b = reference.bounds();
            let d = [
                b.max[0] - b.min[0],
                b.max[1] - b.min[1],
                b.max[2] - b.min[2],
            ];
            d[0].mul_add(d[0], d[1].mul_add(d[1], d[2] * d[2])).sqrt()
        };
        println!("── {name} (diagonal {diag:.3}) ──");
        for detail in [0.25f32, 0.5, 1.0] {
            let target = ph2d_quadflow::edge_for_detail_with(
                &reference,
                detail,
                ph2d_quadflow::GLOBAL_FLOOR_IN_INPUT_EDGES,
            );
            let mut work = reference.clone();
            ph2d_remesh_iso::remesh_isotropic(&mut work, ph2d_remesh_iso::ALPHA);
            work.triangulate();
            // ⭐⭐ **O CONTROLO, e ele vem ANTES de acusar a cadeia.** Se a aresta
            // longa já estiver na fixtura ou na saída do F1, nada a jusante a criou
            // — e a cura é noutra fase. *Correr o controlo antes de atribuir a causa
            // é a lição que esta linha já pagou duas vezes hoje.*
            let long = |m: &ph2d_mesh::Mesh| -> f32 {
                let q = m.positions();
                let mut w = 0.0f32;
                for f in m.faces() {
                    let v = f.verts();
                    for k in 0..v.len() {
                        let (a, b) = (v[k] as usize, v[(k + 1) % v.len()] as usize);
                        let d = [q[b][0] - q[a][0], q[b][1] - q[a][1], q[b][2] - q[a][2]];
                        w = w.max(d[0].mul_add(d[0], d[1].mul_add(d[1], d[2] * d[2])).sqrt());
                    }
                }
                w
            };
            println!(
                "      CONTROLO: maior aresta da fixtura {:.3} ({:.1}%) · do F1 {:.3} ({:.1}%)",
                long(&reference),
                100.0 * long(&reference) / diag,
                long(&work),
                100.0 * long(&work) / diag,
            );
            let dual = ph2d_crossfield::Dual::build(&work);
            let (field, _) = ph2d_crossfield::solve_miq(&dual);
            let layout = ph2d_trace::trace_patches(&work, &dual, &field);
            let Ok(spec) = layout.to_layout(target) else {
                println!("  d={detail:.2} | o layout RECUSOU");
                continue;
            };
            let Ok((quant, _)) =
                ph2d_quantize::quantize_within(&spec, ph2d_quantize::Budget::new(256, 512))
            else {
                println!("  d={detail:.2} | a quantizacao RECUSOU");
                continue;
            };
            let Ok((out, r)) = ph2d_quadfill::fill(
                &work,
                &reference,
                &layout,
                &quant,
                ph2d_quadfill::SMOOTHING_ROUNDS,
            ) else {
                println!("  d={detail:.2} | a montagem RECUSOU");
                continue;
            };
            // ⭐ A valência de cada vértice da SAÍDA — quantas arestas o tocam.
            let mut deg: BTreeMap<u32, usize> = BTreeMap::new();
            let mut seen: std::collections::BTreeSet<(u32, u32)> =
                std::collections::BTreeSet::new();
            for f in out.faces() {
                let v = f.verts();
                for k in 0..v.len() {
                    let (a, b) = (v[k], v[(k + 1) % v.len()]);
                    if seen.insert((a.min(b), a.max(b))) {
                        *deg.entry(a).or_default() += 1;
                        *deg.entry(b).or_default() += 1;
                    }
                }
            }
            let mut hist: BTreeMap<usize, usize> = BTreeMap::new();
            for d in deg.values() {
                *hist.entry(*d).or_default() += 1;
            }
            let worst_val = hist.keys().copied().max().unwrap_or(0);
            let big = hist
                .iter()
                .filter(|(k, _)| **k >= 7)
                .map(|(_, v)| v)
                .sum::<usize>();
            // ⭐ Os lados dos patches — de onde a valência do leque vem.
            let worst_side = layout.side_arcs.iter().map(Vec::len).max().unwrap_or(0);
            println!(
                "  d={detail:.2} alvo {target:.4} | {:>6} quads {:>4} irreg \
                 | ⭐VALENCIA max {worst_val:<3} (>=7: {big:<4}) patch max {worst_side:<3} \
                 | ⭐ARESTA max {:.1}% da peca (mediana {:.2}x do alvo, max {:.1}x) \
                 | dobras {}",
                r.quads,
                r.irregular,
                100.0 * r.edge_max / diag,
                r.edge_median / target,
                r.edge_max / target,
                r.folded_local,
            );
            if worst_val >= 7 {
                println!("      histograma de valencia: {hist:?}");
            }
            // ⭐⭐ **O ACHATAMENTO FALHOU?** A grade de um patch nasce sobre a
            // superfície através do [`ph2d_quadfill`]`::PatchParam` — Tutte + coordenadas
            // de valor médio. ⚠️ Se ele não converge ou uma amostra cai fora do
            // domínio, o ponto vem de outro sítio — e **é aí que dois vizinhos de
            // grade podem acabar em lados opostos da peça**.
            println!(
                "      ACHATAMENTO: {} de {} patches · amostras {} com {} FALHAS · residuo {:.2e} em {} rondas",
                r.flattened,
                r.patches,
                r.sampled,
                r.sample_misses,
                r.flatten_residual,
                r.flatten_rounds,
            );
            // ⭐⭐ **OS ARCOS MAIS LONGOS, e quantos segmentos cada um recebeu.**
            // Um arco cuja **cadeia** tem poucos vértices não tem por onde ser
            // reamostrado: a reamostragem anda sobre a cadeia, e entre dois vértices
            // dela só existe a corda. *O `arc_length` mede a curva; o `arc_chain`
            // mede quantos pontos existem para a descrever.*
            {
                let mut idx: Vec<usize> = (0..layout.arc_length.len()).collect();
                idx.sort_by(|&a, &b| layout.arc_length[b].total_cmp(&layout.arc_length[a]));
                for &a in idx.iter().take(3) {
                    let tau = layout.arc_tau[a].last().copied().unwrap_or(0.0);
                    println!(
                        "      arco #{a}: comprimento {:.3} · tau {tau:.3} · cadeia {} vertices                          · alvo {:.1} · recebeu {} segmentos",
                        layout.arc_length[a],
                        layout.arc_chain[a].len(),
                        tau / target,
                        quant.arc[a],
                    );
                }
            }
            // ⭐⭐ **ONDE ela está.** A proveniência diz de que peça do leque o
            // vértice veio; ela **não** diz em que sítio da escultura o defeito
            // mora — e é o sítio que a foto mostra. *Duas perguntas, dois números.*
            let pos = out.positions();
            let mut worst = (0.0f32, 0u32, 0u32);
            for f in out.faces() {
                let v = f.verts();
                for k in 0..v.len() {
                    let (a, b) = (v[k] as usize, v[(k + 1) % v.len()] as usize);
                    let d = [
                        pos[b][0] - pos[a][0],
                        pos[b][1] - pos[a][1],
                        pos[b][2] - pos[a][2],
                    ];
                    let l = d[0].mul_add(d[0], d[1].mul_add(d[1], d[2] * d[2])).sqrt();
                    if l > worst.0 {
                        worst = (l, v[k], v[(k + 1) % v.len()]);
                    }
                }
            }
            let (pa, pb) = (pos[worst.1 as usize], pos[worst.2 as usize]);
            println!(
                "      a MAIOR aresta: {:.3} de ({:.2}, {:.2}, {:.2}) a ({:.2}, {:.2}, {:.2})                  | raios {:.2} e {:.2}",
                worst.0,
                pa[0],
                pa[1],
                pa[2],
                pb[0],
                pb[1],
                pb[2],
                pa[0]
                    .mul_add(pa[0], pa[1].mul_add(pa[1], pa[2] * pa[2]))
                    .sqrt(),
                pb[0]
                    .mul_add(pb[0], pb[1].mul_add(pb[1], pb[2] * pb[2]))
                    .sqrt(),
            );
            // ⭐⭐ **DE QUEM É A ARESTA LONGA.** A decomposição por proveniência já
            // existia e ninguém lhe tinha perguntado isto: se ela for do `centro` ou
            // do `raio`, a dívida é do **leque** do F5; se for do `arco`, é de um
            // arco que o F4 esmagou; se for da `grade`, é bug desta crate.
            println!(
                "      arestas longas por proveniencia: {:?}",
                ph2d_quadfill::Provenance::NAMES
                    .iter()
                    .zip(r.edge_long_prov)
                    .collect::<Vec<_>>()
            );
        }
    }
}

/// ⛔⛔ **VERMELHO — A ORELHA SAI COM UMA ARESTA DE 56 % DA PEÇA** (as fotos de
/// 2026-08-22).
///
/// ⚠️ **O artista mandou três fotos e a palavra «muito ruim», e NENHUMA régua desta
/// linha piscou:** 100 % de quads · casca fechada · característica de Euler exacta ·
/// densidade no alvo (mediana `1,02×`) · 15 irregulares · valência máxima **6** · e
/// a forma preservada (`detail_lost` p95 `0,085 %`). *Uma peça pode passar em toda
/// asserção e estar visivelmente destruída* — é a terceira vez que esta linha paga
/// esta lição, e desta vez a régua **existia** e não estava apontada para esta peça.
///
/// ⭐ **A barra é a que o relatório já define** (`QuadRemeshReport::edge_max_span`,
/// `≤ 0,20`), e ela só corria sobre a `wrinkled_sphere` — que mede `6 %`.
///
/// | fixtura | aresta máxima, em fração da peça | dobras (d = 1,0) |
/// |---|---|---|
/// | ⛔ **orelha** | **56,5 % · 56,3 % · 57,0 %** nos três níveis do slider | ⛔ **2 204** (4,85 %) |
/// | gancho | 9,2 % · 8,0 % · 11,6 % | 19 |
/// | enrugada | 10,0 % · 6,4 % · 6,1 % | 70 |
///
/// ⭐⭐ **O que se sabe da aresta, medido:** ela liga dois pontos **ANTIPODAIS**,
/// ambos a raio `1,00` — na parte lisa da esfera, não no vinco da orelha — e o
/// comprimento dela é `1,98`, o **diâmetro**. ⚠️ Ela **não encolhe** quando o alvo
/// encolhe 10×: *não é falta de resolução, é um ponto no sítio errado.*
///
/// ⭐ **E o CONTROLO ilibou as fases de montante:** a maior aresta da fixtura é
/// `1,8 %` e a da saída do F1 é `3,3 %`. A cadeia cria-a.
///
/// ⛔ **Uma hipótese foi construída, medida e REJEITADA:** as falhas de amostragem
/// do achatamento (**3 329**, `8,4 %`, e **zero** nas outras duas fixturas) pareciam
/// a causa — o recurso delas é um ponto de Coons no **espaço**, que passa perto do
/// centro da peça. Curá-las com um recuo para dentro do domínio levou as falhas a
/// **0** e ⛔ **não moveu a aresta**, com as dobras a **subir 25 %**. *Elas
/// acompanham o defeito naquela peça; não o causam.*
///
/// ⇒ **A causa continua por achar**, e é o item nº 1. Este gate é o endereço dela.
///
/// ```text
/// \
///   cargo test -p ph2d-host-desktop --release --bins \
///   the_ear_does_not_ship_an_edge_across_the_piece -- --ignored --nocapture
/// ```
#[test]
#[ignore = "VERMELHO -- a orelha sai com uma aresta de 56% da peca, ver o doc"]
fn the_ear_does_not_ship_an_edge_across_the_piece() {
    let reference = crate::sculpt3d::fixtures::eared_sphere();
    let b = reference.bounds();
    let d = [
        b.max[0] - b.min[0],
        b.max[1] - b.min[1],
        b.max[2] - b.min[2],
    ];
    let diag = d[0].mul_add(d[0], d[1].mul_add(d[1], d[2] * d[2])).sqrt();
    let mut work = reference.clone();
    ph2d_remesh_iso::remesh_isotropic(&mut work, ph2d_remesh_iso::ALPHA);
    work.triangulate();
    let dual = ph2d_crossfield::Dual::build(&work);
    let (field, _) = ph2d_crossfield::solve_miq(&dual);
    let layout = ph2d_trace::trace_patches(&work, &dual, &field);
    for detail in [0.25f32, 0.5, 1.0] {
        let target = ph2d_quadflow::edge_for_detail_with(
            &reference,
            detail,
            ph2d_quadflow::GLOBAL_FLOOR_IN_INPUT_EDGES,
        );
        let spec = layout.to_layout(target).expect("o layout fecha");
        let (quant, _) =
            ph2d_quantize::quantize_within(&spec, ph2d_quantize::Budget::new(256, 512))
                .expect("quantiza");
        let (_, r) = ph2d_quadfill::fill(
            &work,
            &reference,
            &layout,
            &quant,
            ph2d_quadfill::SMOOTHING_ROUNDS,
        )
        .expect("monta");
        let span = r.edge_max / diag;
        eprintln!(
            "[f5] orelha d={detail:.2}: aresta maxima {:.1} % da peca ({} quads, {} dobras)",
            100.0 * span,
            r.quads,
            r.folded_local,
        );
        // ⚠️ **A barra é a mesma do `QuadRemeshReport::edge_max_span`**, e ⛔ ela
        // não se afrouxa: ela afirma *"nada atravessa a peça"*, que é um facto
        // absoluto e não uma preferência.
        assert!(
            span <= 0.20,
            "d={detail:.2}: uma aresta atravessa {:.1} % da peca (barra 20 %)",
            100.0 * span
        );
    }
}
