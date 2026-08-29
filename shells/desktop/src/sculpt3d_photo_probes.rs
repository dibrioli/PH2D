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
            // ⭐⭐ **OS RAIOS DO LEQUE.** Num patch de valência `n` o F5 põe um
            // centro e `n` raios de comprimento `e_j`; cada sector é uma grade
            // `e_{j+1} × e_j`. ⛔ **Um `e_j` de 1 faz o sector ter UMA célula de
            // fundo** — e o quad dela liga um ponto lá longe do lado `j` a um ponto
            // lá longe do lado `j+1`. *Se o patch for grande, esse quad atravessa-o.*
            for (pp, e) in quant.corners.iter().enumerate() {
                let n = layout.side_arcs[pp].len();
                let seg: Vec<u32> = (0..n)
                    .map(|i| {
                        layout.side_arcs[pp][i]
                            .iter()
                            .map(|&(a, _)| quant.arc[a as usize])
                            .sum()
                    })
                    .collect();
                let len: Vec<f32> = (0..n)
                    .map(|i| {
                        layout.side_arcs[pp][i]
                            .iter()
                            .map(|&(a, _)| layout.arc_length[a as usize])
                            .sum()
                    })
                    .collect();
                let lo = len.iter().copied().fold(f32::MAX, f32::min);
                let hi = len.iter().copied().fold(0.0f32, f32::max);
                let ratio = lo / hi.max(1.0e-9);
                let _ = &seg;
                // ⭐ Só os suspeitos: lado mais curto abaixo de um décimo do mais
                // longo, **ou** um raio de leque a `1`.
                if ratio > 0.10 && e.iter().copied().min().unwrap_or(9) > 1 {
                    continue;
                }
                println!(
                    "      patch {pp:<3} ({n} lados) curto/longo ⭐{ratio:.4}                      | raio min {} | perimetro {:.2} ({:.0}% da peca) | raios {e:?}",
                    e.iter().copied().min().unwrap_or(0),
                    len.iter().sum::<f32>(),
                    100.0 * len.iter().sum::<f32>() / diag,
                );
                // ⭐⭐ **O ÂNGULO EM CADA CANTO.** Um canto de patch é onde a
                // fronteira **vira**; perto de 180° ela passa direito, e o «canto»
                // ali é um artefacto — os dois lados que ele separa deviam ser UM.
                // ⛔ *Um canto a mais não é um vértice a mais: é um lado partido em
                // dois, e um deles pode sair uma lasca.*
                if len.iter().sum::<f32>() > 4.0 * diag {
                    let dir_at = |j: usize, last: bool| -> Option<[f32; 3]> {
                        let side = &layout.side_arcs[pp][j];
                        let (a, rev) = if last { *side.last()? } else { *side.first()? };
                        let c = &layout.arc_chain[a as usize];
                        if c.len() < 2 {
                            return None;
                        }
                        let (u, v) = match (last, rev) {
                            (true, false) => (c[c.len() - 2], c[c.len() - 1]),
                            (true, true) => (c[1], c[0]),
                            (false, false) => (c[1], c[0]),
                            (false, true) => (c[c.len() - 2], c[c.len() - 1]),
                        };
                        let (a3, b3) = (work.positions()[u as usize], work.positions()[v as usize]);
                        Some([b3[0] - a3[0], b3[1] - a3[1], b3[2] - a3[2]])
                    };
                    let ang: Vec<String> = (0..n)
                        .map(|i| {
                            let (Some(inc), Some(out)) =
                                (dir_at(i, true), dir_at((i + 1) % n, false))
                            else {
                                return String::from("?");
                            };
                            let nrm = |d: [f32; 3]| {
                                let l = d[0]
                                    .mul_add(d[0], d[1].mul_add(d[1], d[2] * d[2]))
                                    .sqrt()
                                    .max(1.0e-9);
                                [d[0] / l, d[1] / l, d[2] / l]
                            };
                            let (a3, b3) = (nrm(inc), nrm(out));
                            let c = a3[0]
                                .mul_add(b3[0], a3[1].mul_add(b3[1], a3[2] * b3[2]))
                                .clamp(-1.0, 1.0);
                            format!("{:.0}", 180.0 - c.acos().to_degrees())
                        })
                        .collect();
                    // ⭐⭐ **OS ARCOS DE CADA LADO.** Se o mesmo id aparecer em dois
                    // lados, este patch é um **anel cortado e aberto** pela ponte: a
                    // fronteira percorre a ponte duas vezes, e é isso que faz o
                    // «hairpin» de 14°. *Um leque sobre uma fronteira que dobra
                    // sobre si mesma não é o preenchimento certo.*
                    let ids: Vec<Vec<u32>> = (0..n)
                        .map(|i| layout.side_arcs[pp][i].iter().map(|&(a, _)| a).collect())
                        .collect();
                    let flat: Vec<u32> = ids.iter().flatten().copied().collect();
                    let repetido: Vec<u32> = flat
                        .iter()
                        .copied()
                        .filter(|a| flat.iter().filter(|b| *b == a).count() > 1)
                        .collect();
                    println!(
                        "        ⭐ arcos por lado: {ids:?} | REPETIDOS no mesmo patch: {repetido:?}"
                    );
                    println!(
                        "        ⭐ angulos nos cantos: {:?} graus | lados {:?}",
                        ang,
                        len.iter().map(|v| format!("{v:.2}")).collect::<Vec<_>>()
                    );
                }
            }
            // ⭐⭐ **OS PATCHES RETANGULARES, e se os LADOS OPOSTOS estão longe.**
            // A face da aresta liga **dois pares** de pontos — cada par junto, os
            // pares antipodais. Isso é a assinatura de uma grade construída entre
            // dois lados que não deviam estar frente a frente.
            for (pp, sides) in layout.side_arcs.iter().enumerate() {
                if sides.len() != 4 {
                    continue;
                }
                // O ponto médio geométrico de cada lado, pela cadeia dos arcos dele.
                let mid = |i: usize| -> [f32; 3] {
                    let mut acc = [0.0f64; 3];
                    let mut n = 0usize;
                    for &(a, _) in &sides[i] {
                        for &v in &layout.arc_chain[a as usize] {
                            let q = work.positions()[v as usize];
                            for k in 0..3 {
                                acc[k] += f64::from(q[k]);
                            }
                            n += 1;
                        }
                    }
                    let n = n.max(1) as f64;
                    [
                        (acc[0] / n) as f32,
                        (acc[1] / n) as f32,
                        (acc[2] / n) as f32,
                    ]
                };
                let gap = |i: usize, j: usize| -> f32 {
                    let (a, b) = (mid(i), mid(j));
                    let d = [b[0] - a[0], b[1] - a[1], b[2] - a[2]];
                    d[0].mul_add(d[0], d[1].mul_add(d[1], d[2] * d[2])).sqrt()
                };
                let (g02, g13) = (gap(0, 2), gap(1, 3));
                if g02.max(g13) < 0.35 * diag {
                    continue;
                }
                let seg = |i: usize| -> u32 {
                    sides[i].iter().map(|&(a, _)| quant.arc[a as usize]).sum()
                };
                println!(
                    "      ⛔ patch {pp} (4 lados): vao 0-2 {:.2} ({:.0}% da peca) · 1-3 {:.2}                      ({:.0}%) | segmentos {} {} {} {}",
                    g02,
                    100.0 * g02 / diag,
                    g13,
                    100.0 * g13 / diag,
                    seg(0),
                    seg(1),
                    seg(2),
                    seg(3),
                );
            }
            // ⭐⭐ **A FACE INTEIRA, e não só a aresta.** Se três dos quatro cantos
            // estão juntos e um está longe, o defeito é **um ponto** e a pergunta
            // passa a ser *qual construção o pôs lá*; se dois e dois, é a face que
            // liga duas regiões, e a pergunta é do **índice**.
            for f in out.faces() {
                let v = f.verts();
                if !v.contains(&worst.1) || !v.contains(&worst.2) {
                    continue;
                }
                let raios: Vec<String> = v
                    .iter()
                    .map(|&i| {
                        let q = pos[i as usize];
                        format!(
                            "({:.2},{:.2},{:.2}) r={:.2}",
                            q[0],
                            q[1],
                            q[2],
                            q[0].mul_add(q[0], q[1].mul_add(q[1], q[2] * q[2])).sqrt()
                        )
                    })
                    .collect();
                println!("      a FACE da aresta: {:?}\n        ids {v:?}", raios);
                break;
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

/// ⭐⭐⭐ **VERDE desde 2026-08-22** — a aresta de **56 %** que o artista fotografou
/// mede hoje **12,4 % · 7,8 % · 5,5 %** nos três níveis do slider, e as dobras
/// caíram de **2 204 para 171**. O que a curou foi o **campo**: `ALIGN_WEIGHT`
/// deixou de ser zero, e o número dele saiu do gabarito do oráculo (`PLAN.md`
/// §4-tritricies), não de uma varredura no fim da cadeia.
///
/// ⛔ **O que ela foi** (as fotos de 2026-08-22).
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
/// # ⭐⭐⭐ A CAUSA, achada em 2026-08-22 — **um patch que dá três voltas à peça**
///
/// ⛔ **`patch 1` da orelha tem SEIS lados de comprimento**
/// `6,78 · 0,27 · 2,15 · 6,80 · 0,09 · 2,15` — perímetro **18,2**, que é **520 % da
/// diagonal da peça**. Os dois lados de `6,8` são quase a circunferência inteira
/// (`6,28`); os de `0,09` e `0,27` são lascas.
///
/// ⭐ **E o número separa-o de tudo o resto com margem:** em três fixturas e
/// dezenas de patches, o segundo maior perímetro é **230 %**. *Um contra 2,3× de
/// folga não é uma cauda de distribuição: é outra coisa.*
///
/// ⇒ **O mecanismo, do perímetro à aresta:** as lascas recebem `2` segmentos e os
/// lados longos `40`. A lei do leque (`L_i = e_{i−1} + e_{i+1}`) resolve isso com
/// raios `[1, 39, 1, 1, 39, 1]` — **quatro dos seis raios valem `1`**. Um raio de
/// `1` faz o sector ter **uma célula de fundo**, e o quad dessa célula liga um ponto
/// lá longe de um lado a um ponto lá longe do seguinte. Num patch que dá três voltas
/// à peça, esse quad **atravessa-a** — e as duas pontas saem antipodais, ambas a
/// raio `1,00`, exactamente como a foto mostra.
///
/// ⚠️ **É por isso que ela não encolhe com o slider:** a `d = 1,0` os lados longos
/// passam a `384` segmentos e os raios continuam `[6, 384, 10, 1, 379, 4]` — o
/// sector continua a ter **um** de fundo.
///
/// # ⭐⭐⭐ E o patch é um ANEL CORTADO E ABERTO — a cadeia causal fecha
///
/// Os lados de `patch 1`, pelos arcos que os compõem:
///
/// ```text
///     [16,17,4,5]   [6]   [7]   [8..14]   [15]   [7]
///        6,78      0,27  2,15    6,80     0,09   2,15
/// ```
///
/// ⭐⭐ **O arco `7` aparece DUAS vezes** — e os dois lados de `2,15` são ele, ida e
/// volta. Os ângulos nos cantos confirmam-no: `[74, 77, ⭐14, 118, 70, ⭐26]`, com os
/// dois *hairpins* exactamente nas pontas da ponte. ⇒ **Este patch é o anel que a
/// ponte abriu** (`PLAN.md` §4-sexvicies), e os lados de `6,78`/`6,80` são as duas
/// fronteiras dele — quase a circunferência da esfera cada uma.
///
/// ⭐⭐ **E o controlo mostra que a ponte não é opcional:** com ela desligada, a
/// orelha **RECUSA** nos três níveis do slider (`GenusLost`) — o layout dela não
/// fecha de outra maneira. *A ponte é o que a faz existir; o defeito é como o patch
/// dela é preenchido.*
///
/// # ⇒ Onde o conserto mora, por eliminação
///
/// | candidato | porque NÃO |
/// |---|---|
/// | a quantização dar mais segmentos à lasca | ⛔ a densidade dela **está certa**: `0,27` de comprimento com `2` segmentos é `0,135` cada, contra um alvo de `0,167` |
/// | `dissolve` o patch | ⛔ ele **junta** patches, e este já dá três voltas à peça |
/// | desligar a ponte | ⛔ a orelha deixa de fechar |
///
/// ⭐ **Sobra o PREENCHIMENTO.** A lei do leque (`L_i = e_{i−1} + e_{i+1}`) é a da
/// referência e está certa para um `n`-gono de verdade; ⛔ **um anel cortado não é
/// um hexágono** — duas das seis arestas dele são a *mesma* curva. O preenchimento
/// certo para ele é uma **faixa**: uma grade entre as duas fronteiras, com a ponte
/// como costura. *É o que o `fill_rectangle` já faz para `n = 4`, e é preciso
/// reconhecer este caso para lá o mandar.*
///
/// ```text
/// \
///   cargo test -p ph2d-host-desktop --release --bins \
///   the_ear_does_not_ship_an_edge_across_the_piece -- --ignored --nocapture
/// ```
///
/// ```text
/// \
///   cargo test -p ph2d-host-desktop --release --bins \
///   the_ear_does_not_ship_an_edge_across_the_piece -- --ignored --nocapture
/// ```
#[test]
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

/// ⭐⭐⭐ **SONDA — A PEÇA DO ARTISTA PELO BOTÃO, e não por uma cópia da ordem dele.**
///
/// ⛔ **Ela existe porque as outras sondas desta linha NÃO são o botão.** O
/// [`ph2d_quadchain::quads_from_mesh`] remalha para o `target_edge` que recebe e corre a
/// cadeia **uma** vez; o botão remalha com [`ph2d_remesh_iso::ALPHA`] **fixo**, tira o alvo
/// do slider ([`ph2d_quadflow::edge_for_detail_with`]) e corre **duas ou três** tentativas
/// que uma medição escolhe. *Duas ordens diferentes com o mesmo nome dão dois números, e o
/// que o artista vê é o da que ele carrega.*
///
/// ⚠️ Ela chama [`crate::sculpt3d::Sculpt3dScene::quad_remesh`] — a **porta** —, então
/// nenhuma lei é reescrita aqui.
///
/// ```text
/// \
///   env PH2D_PIECE=/caminho/peca.obj PH2D_DETAIL=0.5 \
///   cargo test -p ph2d-host-desktop --release --bins \
///   the_artists_piece_through_the_button -- --ignored --nocapture
/// ```
#[test]
#[ignore = "sonda -- a peca do artista pelo BOTAO (PH2D_PIECE=<obj>)"]
fn the_artists_piece_through_the_button() {
    let Ok(path) = std::env::var("PH2D_PIECE") else {
        eprintln!("sem PH2D_PIECE -- nada a medir");
        return;
    };
    let Ok(gpu) = ph2d_gpu::GpuContext::new(ph2d_gpu::GpuContext::default_instance(), None) else {
        eprintln!("no GPU adapter on this machine -- nothing to assert");
        return;
    };
    // ⭐⭐⭐ **A FIXTURA SINTÉTICA `espinhos:<n>`** — ver [`spiked_ball`]. ⛔ Ela existe porque
    // *a fixtura só prova o que ela contém*: a peça que o artista fotografou em 28/08 é a
    // saída partida, e a **entrada** dele não está nesta árvore. Sem uma peça com pontas
    // finas, toda medição desta wave mediria uma bola.
    let piece = if let Some(n) = path.strip_prefix("espinhos:") {
        spiked_ball(
            n.parse().unwrap_or(6),
            std::env::var("PH2D_SPIKE_SIGMA")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(0.10f32),
        )
    } else {
        let text = std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("{path}: {e}"));
        ph2d_mesh::import_obj(&text)
            .unwrap_or_else(|e| panic!("{path} nao e' um OBJ deste leitor: {e:?}"))
            .into_iter()
            .next()
            .unwrap_or_else(|| panic!("{path} nao tem peca dentro"))
            .mesh
    };
    let detail: f32 = std::env::var("PH2D_DETAIL")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(0.5);
    let adapt: f32 = std::env::var("PH2D_ADAPT")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(0.0);

    // ⭐⭐ **O CONTROLO vem ANTES de acusar a cadeia** — se a entrada já é aberta ou
    // não-manifold, o que sai dela não foi a cadeia que abriu.
    census("ENTRADA", &piece);
    islands("ENTRADA", &piece);

    let mut scene = crate::sculpt3d::Sculpt3dScene::new(&gpu.device, piece.clone(), 1.0);
    scene.viewport = (900, 700);
    // ⭐⭐⭐ **A FASE ZERO, medida ao lado do alvo que o slider pede.** ⛔ *Uma cadeia cuja
    // malha de trabalho é mais grossa que o alvo não pode entregar a densidade pedida* — e
    // as duas grandezas nunca tinham sido impressas na mesma linha.
    {
        let reference = scene.mesh().clone();
        let target = ph2d_quadflow::edge_for_detail_with(
            &reference,
            detail,
            ph2d_quadflow::GLOBAL_FLOOR_IN_INPUT_EDGES,
        );
        let mut work = reference.clone();
        ph2d_remesh_iso::remesh_isotropic(&mut work, ph2d_remesh_iso::ALPHA);
        work.triangulate();
        eprintln!(
            "   CENA: {} verts {} faces | aresta media {:.5} | alvo do slider {:.5}",
            reference.vert_count(),
            reference.face_count(),
            ph2d_quadflow::mean_edge(&reference),
            target,
        );
        eprintln!(
            "   F1 (ALPHA={:.4}): {} verts {} faces | aresta media {:.5} | ALVO/F1 = {:.2}x",
            ph2d_remesh_iso::ALPHA,
            work.vert_count(),
            work.face_count(),
            ph2d_quadflow::mean_edge(&work),
            target / ph2d_quadflow::mean_edge(&work),
        );
        census("F1", &work);
        islands("F1", &work);
    }
    // ⭐⭐⭐ **QUAL das duas portas** — o painel tem um dropdown, e o `Global` é o de
    // omissão ([`ph2d_panel_sculpt3d::state::RetopoMode`]). ⛔ `PH2D_PROBE_LOCAL=1` corre a
    // outra. *Chamar a `quad_remesh` directamente é correr o motor LOCAL, que é o que o
    // artista tem quando o dropdown diz `Fast` — e não o que ele tem por omissão.*
    let local = std::env::var("PH2D_PROBE_LOCAL").as_deref() == Ok("1");
    eprintln!(
        "   MOTOR: {}",
        if local { "Fast (local)" } else { "Even Grid (global)" }
    );
    // ⭐⭐⭐ **QUANTOS CLIQUES** — o artista carrega outra vez quando o resultado parece
    // errado, e o alvo do 2.º clique sai da malha que o 1.º deixou. *Um botão cujo repetir
    // destrói a peça é um defeito de produto, não uma escolha do artista.*
    let presses: usize = std::env::var("PH2D_PRESSES")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(1);
    let mut r = None;
    for press in 1..=presses {
        let clock = std::time::Instant::now();
        let one = if local {
            scene.quad_remesh(detail, adapt)
        } else {
            scene.quad_remesh_global(detail, adapt)
        }
        .unwrap_or_else(|e| panic!("o botao recusou no clique {press}: {e:?}"));
        eprintln!(
            "   CLIQUE {press}: alvo {:.5} -> {} quads ({} nao-quads) em {:.0} ms",
            one.edge,
            one.quads,
            one.non_quads,
            clock.elapsed().as_secs_f32() * 1000.0,
        );
        census(&format!("  apos {press}"), scene.mesh());
        r = Some(one);
    }
    let r = r.expect("pelo menos um clique");
    let out = scene.mesh().clone();

    eprintln!(
        "BOTAO d={detail:.2} a={adapt:.2} -> {} quads ({} nao-quads) [campo {}]",
        r.quads,
        r.non_quads,
        if r.aligned { "alinhado" } else { "liso" }
    );
    eprintln!(
        "   alvo {:.5} | aresta mediana {:.2}x max {:.2}x | a maior atravessa {:.1} % da peca",
        r.edge,
        r.edge_median_ratio,
        r.edge_max_ratio,
        100.0 * r.edge_max_span
    );
    eprintln!(
        "   forma: aspecto p50 {:.2} p99 {:.2} | enviesamento p50 {:.1} p99 {:.1} | >60 {} | dobras {}",
        r.shape.aspect_p50,
        r.shape.aspect_p99,
        r.shape.skew_p50,
        r.shape.skew_p99,
        r.shape.skew_over_60,
        r.folded,
    );
    census("SAIDA", &out);
    islands("SAIDA", &out);
    relief_density("SAIDA", &out);
    holes("SAIDA", &out);
}

/// ⭐⭐⭐ **UMA BOLA COM ESPINHOS** — a fixtura que CONTÉM o fenómeno de 2026-08-29.
///
/// ⛔⛔ **Ela existe porque a peça do artista não está nesta árvore.** As fotos dele mostram
/// uma bola com cinco ou seis **espinhos longos e finos**, e o remesh **amputa-os**: os três
/// piores sítios da saída estão a `r = 1,80`, `1,73` e `1,53` (o corpo tem `r ≈ 1`), com
/// `16` e `12` vértices irregulares cada um e arestas de `0,34` contra uma mediana de
/// `0,031`. ⚠️ *Uma esfera lisa não contém nada disto, e era sobre esferas lisas que toda
/// medição desta linha corria.*
///
/// A construção: uma `uv_sphere` fina, e cada vértice é empurrado para fora ao longo do seu
/// raio por um envelope **gaussiano** centrado em cada direcção de espinho. ⚠️ O expoente do
/// envelope é o que torna o espinho **fino**: `exp(−(θ/σ)²)` com `σ` pequeno dá uma agulha,
/// que é exactamente o que a foto mostra.
fn spiked_ball(n: usize, sigma: f32) -> ph2d_mesh::Mesh {
    let mut mesh = ph2d_mesh::shapes::uv_sphere(96, 144, 1.0);
    // ⚠️ **Direcções ESPALHADAS e não num eixo** — a espiral de Fibonacci evita que dois
    // espinhos caiam na mesma fileira de vértices da esfera, o que os tornaria o mesmo
    // espinho gordo em vez de dois finos.
    let golden = std::f32::consts::PI * (3.0 - 5.0f32.sqrt());
    let dirs: Vec<[f32; 3]> = (0..n.max(1))
        .map(|i| {
            let y = 1.0 - 2.0 * (i as f32 + 0.5) / n.max(1) as f32;
            let r = (1.0 - y * y).max(0.0).sqrt();
            let a = golden * i as f32;
            [r * a.cos(), y, r * a.sin()]
        })
        .collect();
    // ⭐⭐⭐ **OS NÚMEROS SÃO OS DA PEÇA DELE**, medidos no `.obj` que ele exportou: o
    // espinho mais longo vai a `r = 1,81` de um corpo de `r ~ 1`, e o **raio local** dele cai
    // de `0,147` (a 65 % do comprimento) para **`0,037`** (a 97 %). O F1 remalha com aresta
    // `0,089`, que e' **2,4x** a espessura da ponta.
    // A 1a redaccao usou `sigma = 0,22` e deu cones GORDOS: a cadeia devolvia casca fechada
    // em todas as densidades e a fixtura **nao continha o fenomeno**.
    let reach = 0.85f32;
    let pos = mesh.positions_mut();
    for p in pos.iter_mut() {
        let len = p[0].mul_add(p[0], p[1].mul_add(p[1], p[2] * p[2])).sqrt();
        if len < 1.0e-6 {
            continue;
        }
        let u = [p[0] / len, p[1] / len, p[2] / len];
        let mut grow = 0.0f32;
        for d in &dirs {
            let c = u[0].mul_add(d[0], u[1].mul_add(d[1], u[2] * d[2])).clamp(-1.0, 1.0);
            let ang = c.acos();
            grow = grow.max(reach * (-(ang / sigma) * (ang / sigma)).exp());
        }
        let k = 1.0 + grow * 1.2;
        for i in 0..3 {
            p[i] = u[i] * len * k;
        }
    }
    mesh.rebuild();
    mesh
}

/// ⭐⭐⭐ **AS ILHAS** — componentes ligados por aresta, e o que cada uma pesa.
///
/// ⛔ **Ela existe por uma foto (Enio, 28/08, «faces completamente soltas»):** a peça dele
/// saiu com o casco fechado (`χ = 2`, zero bordo, zero não-manifold) **e** uma ilha de
/// **duas** faces a flutuar — o MESMO quadrado emitido duas vezes, um virado ao contrário
/// do outro. ⚠️ *Nenhuma régua desta linha a via: todas mediam a malha inteira como um
/// objecto só.*
fn islands(tag: &str, mesh: &ph2d_mesh::Mesh) {
    use std::collections::BTreeMap;
    let mut owners: BTreeMap<(u32, u32), Vec<usize>> = BTreeMap::new();
    for (fi, f) in mesh.faces().iter().enumerate() {
        let v = f.verts();
        for k in 0..v.len() {
            let (a, b) = (v[k], v[(k + 1) % v.len()]);
            owners.entry((a.min(b), a.max(b))).or_default().push(fi);
        }
    }
    let mut parent: Vec<usize> = (0..mesh.face_count()).collect();
    fn find(parent: &mut [usize], mut x: usize) -> usize {
        while parent[x] != x {
            parent[x] = parent[parent[x]];
            x = parent[x];
        }
        x
    }
    for fs in owners.values() {
        for w in fs.windows(2) {
            let (a, b) = (find(&mut parent[..], w[0]), find(&mut parent[..], w[1]));
            if a != b {
                parent[a] = b;
            }
        }
    }
    let mut size: BTreeMap<usize, usize> = BTreeMap::new();
    for i in 0..mesh.face_count() {
        let r = find(&mut parent[..], i);
        *size.entry(r).or_default() += 1;
    }
    let mut counts: Vec<usize> = size.values().copied().collect();
    counts.sort_unstable_by(|a, b| b.cmp(a));
    eprintln!("   {tag}: {} ilha(s) -> {:?}", counts.len(), &counts[..counts.len().min(6)]);
    if counts.len() > 1 {
        let pos = mesh.positions();
        for (root, n) in &size {
            if *n > counts[0] / 2 {
                continue;
            }
            let mut c = [0.0f32; 3];
            let mut m = 0usize;
            for i in 0..mesh.face_count() {
                if find(&mut parent[..], i) != *root {
                    continue;
                }
                for &v in mesh.faces()[i].verts() {
                    let p = pos[v as usize];
                    for k in 0..3 {
                        c[k] += p[k];
                    }
                    m += 1;
                }
            }
            let inv = 1.0 / m.max(1) as f32;
            eprintln!(
                "      ilha SOLTA de {n} faces em ({:.3}, {:.3}, {:.3})",
                c[0] * inv,
                c[1] * inv,
                c[2] * inv
            );
        }
    }
}

/// ⭐⭐⭐ **A DENSIDADE SEGUE A FORMA?** — o expoente de `aresta ∼ curvatura^n`.
///
/// ⛔ **Ela é a régua do report de 2026-08-28** (*«as pontas finas… têm menos densidade de
/// faces e perdem detalhes»*), e nenhuma régua desta linha a tinha: todas mediam a aresta
/// **global** (mediana, máxima), que não muda quando a grade ignora a forma.
///
/// `0` = grade **uniforme** · negativo = mais fina onde a forma aperta. ⚠️ A faixa de
/// curvatura entre a 1.ª e a 8.ª banda sai ao lado, porque *um expoente sobre uma faixa de
/// `1,1×` não diz nada.*
fn relief_density(tag: &str, mesh: &ph2d_mesh::Mesh) {
    let curv = mesh.curvatures();
    let pos = mesh.positions();
    let mut rows: Vec<(f32, f32)> = Vec::with_capacity(mesh.face_count());
    for f in mesh.faces() {
        let v = f.verts();
        let mut k = 0.0f32;
        let mut e = 0.0f32;
        for i in 0..v.len() {
            k += curv.get(v[i] as usize).copied().unwrap_or(0.0).abs();
            let (a, b) = (pos[v[i] as usize], pos[v[(i + 1) % v.len()] as usize]);
            let d = [a[0] - b[0], a[1] - b[1], a[2] - b[2]];
            e += d[0].mul_add(d[0], d[1].mul_add(d[1], d[2] * d[2])).sqrt();
        }
        let n = v.len() as f32;
        if k > 1.0e-9 {
            rows.push((k / n, e / n));
        }
    }
    if rows.len() < 64 {
        eprintln!("   {tag}: poucas faces com curvatura para medir o expoente");
        return;
    }
    rows.sort_by(|a, b| a.0.total_cmp(&b.0));
    let bands = 8usize;
    let mut xs: Vec<f64> = Vec::new();
    let mut ys: Vec<f64> = Vec::new();
    for b in 0..bands {
        let (lo, hi) = (b * rows.len() / bands, (b + 1) * rows.len() / bands);
        let seg = &rows[lo..hi];
        let mut ks: Vec<f32> = seg.iter().map(|r| r.0).collect();
        let mut es: Vec<f32> = seg.iter().map(|r| r.1).collect();
        ks.sort_by(f32::total_cmp);
        es.sort_by(f32::total_cmp);
        xs.push(f64::from(ks[ks.len() / 2]).ln());
        ys.push(f64::from(es[es.len() / 2]).ln());
    }
    let mx = xs.iter().sum::<f64>() / xs.len() as f64;
    let my = ys.iter().sum::<f64>() / ys.len() as f64;
    let num: f64 = xs.iter().zip(&ys).map(|(x, y)| (x - mx) * (y - my)).sum();
    let den: f64 = xs.iter().map(|x| (x - mx) * (x - mx)).sum();
    eprintln!(
        "   {tag}: aresta ~ curvatura^{:.3}  (0 = UNIFORME) | faixa de curvatura {:.1}x | \
         banda fina {:.5} vs chapada {:.5}",
        if den > 0.0 { num / den } else { 0.0 },
        (xs[xs.len() - 1] - xs[0]).exp(),
        ys[ys.len() - 1].exp(),
        ys[0].exp(),
    );
}

/// A contagem de arestas de uma malha — bordo, não-manifold e `χ`.
fn census(tag: &str, mesh: &ph2d_mesh::Mesh) {
    use std::collections::BTreeMap;
    let mut n: BTreeMap<(u32, u32), usize> = BTreeMap::new();
    for f in mesh.faces() {
        let v = f.verts();
        for k in 0..v.len() {
            let (a, b) = (v[k], v[(k + 1) % v.len()]);
            *n.entry((a.min(b), a.max(b))).or_default() += 1;
        }
    }
    // ⚠️ A valência conta **arestas únicas**, não incidências de face.
    let mut deg: BTreeMap<u32, usize> = BTreeMap::new();
    for (a, b) in n.keys() {
        *deg.entry(*a).or_default() += 1;
        *deg.entry(*b).or_default() += 1;
    }
    let bordo = n.values().filter(|c| **c == 1).count();
    let nm = n.values().filter(|c| **c > 2).count();
    let chi = mesh.vert_count() as i64 - n.len() as i64 + mesh.face_count() as i64;
    let worst = deg.values().copied().max().unwrap_or(0);
    let irregular = deg.values().filter(|d| **d != 4).count();
    eprintln!(
        "   {tag}: {} verts {} faces | X = {chi} | {bordo} bordo | {nm} nao-manifold | \
         valencia max {worst} | {irregular} irregulares",
        mesh.vert_count(),
        mesh.face_count(),
    );
}

/// **ONDE os furos estão** — um por linha, com o perímetro e o centro.
///
/// ⚠️ *Uma contagem de arestas de bordo diz que há furo; ela não diz que ele está na
/// PONTA*, que foi a palavra do artista.
fn holes(tag: &str, mesh: &ph2d_mesh::Mesh) {
    use std::collections::BTreeMap;
    let mut n: BTreeMap<(u32, u32), usize> = BTreeMap::new();
    for f in mesh.faces() {
        let v = f.verts();
        for k in 0..v.len() {
            let (a, b) = (v[k], v[(k + 1) % v.len()]);
            *n.entry((a.min(b), a.max(b))).or_default() += 1;
        }
    }
    let open: Vec<(u32, u32)> = n
        .into_iter()
        .filter(|(_, c)| *c == 1)
        .map(|(e, _)| e)
        .collect();
    if open.is_empty() {
        return;
    }
    // Componentes ligados das arestas abertas — cada um é um furo.
    let mut next: BTreeMap<u32, Vec<u32>> = BTreeMap::new();
    for &(a, b) in &open {
        next.entry(a).or_default().push(b);
        next.entry(b).or_default().push(a);
    }
    let pos = mesh.positions();
    let mut seen: std::collections::BTreeSet<u32> = std::collections::BTreeSet::new();
    let mut loops = 0usize;
    for &start in next.keys() {
        if !seen.insert(start) {
            continue;
        }
        let mut stack = vec![start];
        let mut verts = vec![start];
        while let Some(v) = stack.pop() {
            for &w in next.get(&v).map_or(&[][..], Vec::as_slice) {
                if seen.insert(w) {
                    verts.push(w);
                    stack.push(w);
                }
            }
        }
        let mut c = [0.0f32; 3];
        for &v in &verts {
            let p = pos[v as usize];
            for k in 0..3 {
                c[k] += p[k];
            }
        }
        let inv = 1.0 / verts.len() as f32;
        loops += 1;
        eprintln!(
            "   {tag}: furo #{loops} com {} vertices, centro ({:.3}, {:.3}, {:.3})",
            verts.len(),
            c[0] * inv,
            c[1] * inv,
            c[2] * inv,
        );
    }
}

/// ⭐⭐⭐ **SONDA — a FASE ZERO preserva a topologia que recebe?**
///
/// ⛔⛔ **Reproduzido com a peça do artista em 2026-08-29:** ela entra `χ = 2`, fechada,
/// zero não-manifold — e a `remesh_isotropic` devolve **`χ = 6` com uma aresta não-manifold**.
/// A jusante o `ph2d-gridmap` entra em `index out of bounds` (`assembly.rs:193`), que é o
/// estouro que este repo tinha **sem endereço** desde 26/08.
///
/// ⚠️ **Ela é PURA — não precisa de GPU**, e por isso pode virar gate assim que a fixtura
/// sintética reproduzir o fenómeno.
///
/// ```text
/// \
///   cargo test -p ph2d-host-desktop --release --bins \
///   does_phase_zero_keep_the_topology -- --ignored --nocapture
/// ```
#[test]
#[ignore = "sonda -- a fase zero preserva a topologia?"]
fn does_phase_zero_keep_the_topology() {
    let mut cases: Vec<(String, ph2d_mesh::Mesh)> = Vec::new();
    for sigma in [0.30f32, 0.20, 0.14, 0.10, 0.07, 0.05] {
        cases.push((format!("espinhos sigma={sigma:.2}"), spiked_ball(6, sigma)));
    }
    if let Ok(path) = std::env::var("PH2D_PIECE")
        && let Ok(text) = std::fs::read_to_string(&path)
        && let Ok(pieces) = ph2d_mesh::import_obj(&text)
        && let Some(p) = pieces.into_iter().next()
    {
        cases.push((format!("ARTISTA {path}"), p.mesh));
    }
    for (name, piece) in cases {
        eprintln!("── {name}");
        census("  entrada", &piece);
        let mut work = piece.clone();
        ph2d_remesh_iso::remesh_isotropic(&mut work, ph2d_remesh_iso::ALPHA);
        work.triangulate();
        census("  fase zero", &work);
        islands("  fase zero", &work);
    }
}
