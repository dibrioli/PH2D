//! **AS SONDAS DA DENSIDADE E DO RELÓGIO** — irmãs de
//! [`super::global_retopo_probes`].
//!
//! ⭐⭐ **O corte foi forçado pela HR-18 (683 linhas contra 600), e ele é de
//! ASSUNTO:** lá as sondas do **defeito** (dobras, buracos, proveniência); aqui as
//! da **densidade** — até onde o slider pode ir, quanto custa a quantização, e se o
//! `Follow Curvature` chega à saída.
//!
//! ⚠️ **O que estas quatro têm em comum é a régua do CLAUDE.md §0.0:** nenhum teto
//! deste módulo se escreve sem uma tabela ao lado, e um teto legítimo diz **de que
//! recurso** ele é. Medido aqui: o da resolução é o **relógio da quantização**, e de
//! mais nada.
//!
//! ⚠️ Elas são `#[ignore]` porque imprimem tabelas e demoram minutos — uma sonda
//! que afirma é um gate, e mora noutro ficheiro.

/// ⭐⭐ **SONDA — ATÉ ONDE A CADEIA GLOBAL RESOLVE, E O QUE A LIMITA.**
///
/// ⛔ **O teto do slider é do motor ERRADO.** A [`ph2d_quadflow::edge_for_detail`]
/// ancora o extremo fino em `FLOOR_IN_INPUT_EDGES = 3,0` arestas da entrada, e esse
/// número foi **medido para o porte local**: ali um quad mais fino que o triângulo
/// de entrada faz a extração devolver um ciclo de 352 lados com 58 % do volume
/// perdido (a foto de 2026-08-19). ⚠️ **A cadeia global não extrai de retícula
/// nenhuma** — ela reamostra arcos por comprimento e amostra o interior dentro de
/// um triângulo achatado —, então esse piso não é dela. *O caminho mais limitado
/// está a definir o teto do outro* (CLAUDE.md §0.0).
///
/// ⭐ **E ela varre o outro eixo junto:** o F1 remalha para `ALPHA × diagonal`, uma
/// **constante**. Pedir quads mais finos que os triângulos do F1 é pedir detalhe a
/// uma malha que já o deitou fora — então a sonda acopla os dois e mede o preço.
///
/// ```text
/// \
///   cargo test -p ph2d-host-desktop --release --bins \
///   how_fine_can_the_global_chain_go -- --ignored --nocapture
/// ```
#[test]
#[ignore = "sonda -- ate' onde a cadeia global resolve"]
fn how_fine_can_the_global_chain_go() {
    for (name, which) in [("amassada", 0u8), ("com BICO", 1)] {
        let reference = if which == 1 {
            crate::sculpt3d::fixtures::hooked_sphere()
        } else {
            crate::sculpt3d::fixtures::wrinkled_sphere()
        };
        let b = reference.bounds();
        let diag = {
            let d = [
                b.max[0] - b.min[0],
                b.max[1] - b.min[1],
                b.max[2] - b.min[2],
            ];
            d[0].mul_add(d[0], d[1].mul_add(d[1], d[2] * d[2])).sqrt()
        };
        println!(
            "── {name}: diagonal {diag:.3}, hoje o slider vai ate' {:.4} ──",
            ph2d_quadflow::edge_for_detail(&reference, 1.0)
        );
        // ⚠️ A varredura é em FRAÇÃO DA DIAGONAL, que é a régua do F1 — assim as
        // duas densidades são comparáveis sem passar por uma média de arestas.
        for frac in [0.020f32, 0.014, 0.010, 0.007, 0.005] {
            let target = diag * frac;
            let alpha = (frac * 0.5).min(ph2d_remesh_iso::ALPHA);
            let t0 = std::time::Instant::now();
            let mut work = reference.clone();
            ph2d_remesh_iso::remesh_isotropic(&mut work, alpha);
            work.triangulate();
            let f1 = t0.elapsed().as_secs_f64() * 1000.0;
            // ⚠️ **Imprimir ANTES das fases caras.** A primeira corrida desta sonda
            // estourou dez minutos sem uma linha de saída, e *uma sonda que só fala
            // no fim não diz onde parou*.
            println!(
                "  frac {frac:.4} alvo {target:.4} | work {:>7} tris em {f1:.0} ms...",
                work.faces().len()
            );
            let dual = ph2d_crossfield::Dual::build(&work);
            let (field, _) = ph2d_crossfield::solve_miq(&dual);
            let layout = ph2d_trace::trace_patches(&work, &dual, &field);
            let Ok(spec) = layout.to_layout(target) else {
                println!("  frac {frac:.4} alvo {target:.4} | o layout RECUSOU");
                continue;
            };
            let q0 = std::time::Instant::now();
            let quant =
                match ph2d_quantize::quantize_within(&spec, ph2d_quantize::Budget::new(256, 512)) {
                    Ok((q, r)) => {
                        if !r.proved {
                            println!("  frac {frac:.4} | ⚠️ a quantizacao NAO provou o otimo");
                        }
                        q
                    }
                    Err(e) => {
                        println!("  frac {frac:.4} alvo {target:.4} | a quantizacao RECUSOU {e:?}");
                        continue;
                    }
                };
            let qms = q0.elapsed().as_secs_f64() * 1000.0;
            let f0 = std::time::Instant::now();
            match ph2d_quadfill::fill(
                &work,
                &reference,
                &layout,
                &quant,
                ph2d_quadfill::SMOOTHING_ROUNDS,
            ) {
                #[allow(clippy::cast_precision_loss)]
                Ok((_out, r)) => println!(
                    "  frac {frac:.4} alvo {target:.4} | work {:>7} tris | {:>6} QUADS \
                     {:>4} irreg | dobras {:>4} ({:.2} %) | med {:.2}x max {:.2}x | \
                     patches {:>4} | F1 {f1:>6.0} F4 {qms:>6.0} F5 {:>6.0} ms",
                    work.faces().len(),
                    r.quads,
                    r.irregular,
                    r.folded_local,
                    100.0 * r.folded_local as f64 / r.quads.max(1) as f64,
                    r.edge_median / target,
                    r.edge_max / target,
                    r.patches,
                    f0.elapsed().as_secs_f64() * 1000.0,
                ),
                Err(e) => println!("  frac {frac:.4} alvo {target:.4} | a montagem RECUSOU {e:?}"),
            }
        }
    }
}

/// ⭐⭐ **SONDA — QUANTO CUSTA *PROVAR* O ÓTIMO, E QUANTO ELE COMPRA.**
///
/// ⛔ **É a parede do slider, medida.** Ao pedir uma grade 4,7× mais fina do que o
/// slider hoje oferece (`6 291` quads na esfera amassada), a cadeia gastou
/// **181 702 ms na quantização** contra `477 ms` de remalha e `412 ms` de
/// montagem. *O que limita a resolução não é a geometria nem a memória: é a BUSCA.*
///
/// ⭐ A busca é **opcional** — o doc do [`ph2d_quantize::Budget`] di-lo com todas
/// as letras: gastar as expansões devolve *"uma resposta válida sem prova"*, e o
/// `gap` do relatório diz quão longe do ótimo ela está. Esta sonda mede as duas
/// pontas para que o teto seja escrito a partir de um número.
///
/// ```text
/// \
///   cargo test -p ph2d-host-desktop --release --bins \
///   what_does_proving_the_optimum_cost -- --ignored --nocapture
/// ```
#[test]
#[ignore = "sonda -- o preco da PROVA na quantizacao"]
fn what_does_proving_the_optimum_cost() {
    let reference = crate::sculpt3d::fixtures::wrinkled_sphere();
    let b = reference.bounds();
    let diag = {
        let d = [
            b.max[0] - b.min[0],
            b.max[1] - b.min[1],
            b.max[2] - b.min[2],
        ];
        d[0].mul_add(d[0], d[1].mul_add(d[1], d[2] * d[2])).sqrt()
    };
    for frac in [0.020f32, 0.030, 0.045] {
        let target = diag * frac;
        let mut work = reference.clone();
        ph2d_remesh_iso::remesh_isotropic(&mut work, (frac * 0.5).min(ph2d_remesh_iso::ALPHA));
        work.triangulate();
        let dual = ph2d_crossfield::Dual::build(&work);
        let (field, _) = ph2d_crossfield::solve_miq(&dual);
        let layout = ph2d_trace::trace_patches(&work, &dual, &field);
        let Ok(spec) = layout.to_layout(target) else {
            println!("frac {frac:.3} | o layout RECUSOU");
            continue;
        };
        println!(
            "── frac {frac:.3} alvo {target:.4}: {} patches, {} arcos, work {} tris ──",
            layout.side_arcs.len(),
            layout.arc_chain.len(),
            work.faces().len()
        );
        // ⚠️ **`0` expansões NÃO é «sem resposta»** — é o mergulho sozinho, que é
        // obrigatório e devolve uma quantização válida com o `gap` certificado.
        for (name, budget) in [
            ("SEM busca ", ph2d_quantize::Budget::new(0, 8)),
            ("busca curta", ph2d_quantize::Budget::new(16, 64)),
            ("busca cheia", ph2d_quantize::Budget::new(256, 512)),
        ] {
            let t0 = std::time::Instant::now();
            match ph2d_quantize::quantize_within(&spec, budget) {
                Ok((quant, r)) => {
                    let ms = t0.elapsed().as_secs_f64() * 1000.0;
                    let quads: u32 = quant.arc.iter().sum();
                    println!(
                        "  {name} | {:>8.0} ms | custo {:>10.3} gap {:>8.3} {} | exp {:>4} \
                         solves {:>4} aum {:>8} | Σsegmentos {quads}",
                        ms,
                        r.cost,
                        r.gap,
                        if r.proved { "PROVADO" } else { "sem prova" },
                        r.expansions,
                        r.solves,
                        r.augmentations,
                    );
                }
                Err(e) => println!("  {name} | RECUSOU {e:?}"),
            }
        }
    }
}

/// ⭐⭐ **SONDA — O OUTRO EIXO: MAIS QUADS COM O MESMO MAPA DE PATCHES.**
///
/// ⛔ **A primeira tentativa de subir o teto acoplou a remalha ao alvo, e a
/// quantização passou a 176 SEGUNDOS** — 271 patches, 778 arcos. *Mais patches é o
/// eixo caro.*
///
/// ⭐ **Este é o barato:** o `work` fica onde está (`ALPHA × diagonal`), o layout
/// fica com os mesmos 28 patches e 67 arcos, e só o **alvo de aresta** desce. A
/// rede da quantização não cresce em nós — só os inteiros ficam maiores.
///
/// ⭐⭐ **E o detalhe NÃO se perde**, ao contrário do que a intuição diz: todo ponto
/// de interior é reprojectado sobre a malha **ORIGINAL** do artista
/// (`Points::push_on(surface, …)`), não sobre a remalhada. Quads mais finos apanham
/// detalhe que o `work` já tinha deitado fora.
///
/// ```text
/// \
///   cargo test -p ph2d-host-desktop --release --bins \
///   how_many_quads_fit_without_growing_the_patch_map -- --ignored --nocapture
/// ```
#[test]
#[ignore = "sonda -- mais quads com o mesmo mapa de patches"]
fn how_many_quads_fit_without_growing_the_patch_map() {
    for (name, which) in [("amassada", 0u8), ("com BICO", 1)] {
        let reference = if which == 1 {
            crate::sculpt3d::fixtures::hooked_sphere()
        } else {
            crate::sculpt3d::fixtures::wrinkled_sphere()
        };
        let hoje = ph2d_quadflow::edge_for_detail(&reference, 1.0);
        let mut work = reference.clone();
        ph2d_remesh_iso::remesh_isotropic(&mut work, ph2d_remesh_iso::ALPHA);
        work.triangulate();
        let dual = ph2d_crossfield::Dual::build(&work);
        let (field, _) = ph2d_crossfield::solve_miq(&dual);
        let layout = ph2d_trace::trace_patches(&work, &dual, &field);
        println!(
            "── {name}: work {} tris, {} patches, {} arcos | o slider hoje pára em {hoje:.4} ──",
            work.faces().len(),
            layout.side_arcs.len(),
            layout.arc_chain.len()
        );
        for mult in [1.0f32, 0.7, 0.5, 0.35, 0.25, 0.18, 0.125] {
            let target = hoje * mult;
            let Ok(spec) = layout.to_layout(target) else {
                println!("  {mult:.3}x alvo {target:.4} | o layout RECUSOU");
                continue;
            };
            let t0 = std::time::Instant::now();
            let (quant, qr) =
                match ph2d_quantize::quantize_within(&spec, ph2d_quantize::Budget::new(256, 512)) {
                    Ok(v) => v,
                    Err(e) => {
                        println!("  {mult:.3}x alvo {target:.4} | a quantizacao RECUSOU {e:?}");
                        continue;
                    }
                };
            let qms = t0.elapsed().as_secs_f64() * 1000.0;
            let f0 = std::time::Instant::now();
            match ph2d_quadfill::fill(
                &work,
                &reference,
                &layout,
                &quant,
                ph2d_quadfill::SMOOTHING_ROUNDS,
            ) {
                #[allow(clippy::cast_precision_loss)]
                Ok((_out, r)) => println!(
                    "  {mult:.3}x alvo {target:.4} | {:>6} QUADS {:>4} irreg | dobras {:>4} \
                     ({:.2} %) | med {:.2}x max {:.2}x {:?} | F4 {qms:>7.0} {} F5 {:>6.0} ms",
                    r.quads,
                    r.irregular,
                    r.folded_local,
                    100.0 * r.folded_local as f64 / r.quads.max(1) as f64,
                    r.edge_median / target,
                    r.edge_max / target,
                    r.edge_long_prov,
                    if qr.proved { "prova" } else { "⚠️SEM " },
                    f0.elapsed().as_secs_f64() * 1000.0,
                ),
                Err(e) => println!("  {mult:.3}x alvo {target:.4} | a montagem RECUSOU {e:?}"),
            }
        }
    }
}

/// ⭐⭐ **SONDA — O `Follow Curvature` FAZ ALGUMA COISA NA CADEIA GLOBAL?**
///
/// ⛔ **Até 2026-08-21 a resposta era NÃO**, e o artista reportou-o exactamente
/// assim. O knob era lido só pelo motor local; o motor por omissão é o global, e o
/// log limitava-se a avisar. *Um aviso no terminal não é uma feature.*
///
/// ⭐ **A régua não é *"a saída mudou"*** — mudar é fácil e não prova nada. A
/// pergunta é se os quads **menores caem onde a curvatura é alta**, e ela responde-a
/// comparando a curvatura média dos vértices das faces **pequenas** com a das
/// faces **grandes**. Sem graduação as duas são iguais; com ela, a das pequenas tem
/// de ser maior.
///
/// ```text
/// \
///   cargo test -p ph2d-host-desktop --release --bins \
///   does_follow_curvature_reach_the_global_chain -- --ignored --nocapture
/// ```
#[test]
#[ignore = "sonda -- o Follow Curvature chega a' cadeia global?"]
fn does_follow_curvature_reach_the_global_chain() {
    for (name, which) in [("com CRISTAS", 2u8), ("amassada", 0)] {
        let reference = if which == 2 {
            crate::sculpt3d::fixtures::ridged_sphere()
        } else {
            crate::sculpt3d::fixtures::wrinkled_sphere()
        };
        let target = ph2d_quadflow::edge_for_detail_with(
            &reference,
            0.5,
            ph2d_quadflow::GLOBAL_FLOOR_IN_INPUT_EDGES,
        );
        let mut work = reference.clone();
        ph2d_remesh_iso::remesh_isotropic(&mut work, ph2d_remesh_iso::ALPHA);
        work.triangulate();
        let dual = ph2d_crossfield::Dual::build(&work);
        let (field, _) = ph2d_crossfield::solve_miq(&dual);
        println!("── {name} (alvo {target:.4}) ──");
        // ⭐⭐ **O `-1.0` é um campo SINTÉTICO e brutal** — `alvo/3` num hemisfério e
        // `alvo×3` no outro, ou seja **9× de contraste**. Ele não é um modo do
        // produto: é o controlo que separa *"a graduação não chega à saída"* de *"a
        // curvatura desta peça não pede muito"*. Se nem ele mover a dispersão dos
        // quads, o limite é do LAYOUT e não do campo.
        for adapt in [0.0f32, 0.5, 1.0, -1.0] {
            let mut layout = ph2d_trace::trace_patches(&work, &dual, &field);
            if adapt < 0.0 {
                let pos = work.positions();
                let sizes: Vec<f32> = (0..work.vert_count())
                    .map(|v| {
                        if pos[v][1] > 0.0 {
                            target / 3.0
                        } else {
                            target * 3.0
                        }
                    })
                    .collect();
                layout.grade(&work, &sizes, target);
            } else if adapt > 0.0 {
                let sizing = ph2d_quadflow::ScaleField::adaptive_with(
                    &work,
                    target,
                    adapt,
                    ph2d_quadflow::GLOBAL_FLOOR_IN_INPUT_EDGES,
                );
                let sizes: Vec<f32> = (0..sizing.len()).map(|v| sizing.at(v)).collect();
                let mut sorted = sizes.clone();
                sorted.sort_by(f32::total_cmp);
                let (lo, hi) = sizing.range();
                println!(
                    "    campo de tamanho: min {lo:.4} mediana {:.4} max {hi:.4} | alvo {target:.4}",
                    sorted[sorted.len() / 2]
                );
                layout.grade(&work, &sizes, target);
            }
            let Ok(spec) = layout.to_layout(target) else {
                println!("  adapt {adapt:.2} | o layout RECUSOU");
                continue;
            };
            let Ok((quant, _)) =
                ph2d_quantize::quantize_within(&spec, ph2d_quantize::Budget::new(256, 512))
            else {
                println!("  adapt {adapt:.2} | a quantizacao RECUSOU");
                continue;
            };
            let Ok((out, r)) = ph2d_quadfill::fill(
                &work,
                &reference,
                &layout,
                &quant,
                ph2d_quadfill::SMOOTHING_ROUNDS,
            ) else {
                println!("  adapt {adapt:.2} | a montagem RECUSOU");
                continue;
            };
            // ⭐ Por face: a área e a curvatura média (|κ| da referência, no ponto
            // mais próximo de cada vértice — aqui basta a da saída reprojectada).
            let curv = out.curvatures();
            let pos = out.positions();
            let mut rows: Vec<(f32, f32)> = out
                .faces()
                .iter()
                .map(|f| {
                    let v = f.verts();
                    let mut per = 0.0f32;
                    let mut k = 0.0f32;
                    for i in 0..v.len() {
                        let (a, b) = (pos[v[i] as usize], pos[v[(i + 1) % v.len()] as usize]);
                        let d = [b[0] - a[0], b[1] - a[1], b[2] - a[2]];
                        per += d[0].mul_add(d[0], d[1].mul_add(d[1], d[2] * d[2])).sqrt();
                        k += curv[v[i] as usize].abs();
                    }
                    #[allow(clippy::cast_precision_loss)]
                    (per / v.len() as f32, k / v.len() as f32)
                })
                .collect();
            rows.sort_by(|a, b| a.0.total_cmp(&b.0));
            let q = rows.len() / 4;
            #[allow(clippy::cast_precision_loss)]
            let mean =
                |s: &[(f32, f32)]| s.iter().map(|r| f64::from(r.1)).sum::<f64>() / s.len() as f64;
            let (small, big) = (mean(&rows[..q]), mean(&rows[rows.len() - q..]));
            println!(
                "  adapt {adapt:.2} | {:>6} quads | aresta menor 25 % {:.4} maior 25 % {:.4} \
                 ({:.2}x) | ⭐ curvatura: pequenas {small:.2} grandes {big:.2} = {:.2}x",
                r.quads,
                rows[q].0,
                rows[rows.len() - q].0,
                rows[rows.len() - q].0 / rows[q].0.max(1.0e-9),
                small / big.max(1.0e-9),
            );
        }
    }
}

/// ⭐⭐ **SONDA — A ORELHA SOBREVIVE?** (as fotos de 2026-08-22)
///
/// ⛔ **A terceira foto mostra a orelha ACHATADA**, e nenhuma régua do relatório a
/// veria: 100 % de quads, casca fechada, contagem de irregulares boa. *Uma peça
/// pode passar em toda asserção e ter perdido a forma.*
///
/// ⭐ **A régua nova é a [`ph2d_quadfill::detail_lost`], e o SENTIDO é o ponto
/// inteiro**: `referência → saída`. O contrário é tautológico — a última coisa que
/// a montagem faz é pousar cada ponto na referência.
///
/// ⭐⭐ **E a coluna que decide é a `F1`**: a mesma régua aplicada à malha
/// remalhada, **antes** de a cadeia começar. Se a orelha já morreu ali, nenhum
/// conserto a jusante a traz de volta — e a cura é outra (o `α` do F1), não o
/// traçado nem a montagem.
///
/// ```text
/// \
///   cargo test -p ph2d-host-desktop --release --bins \
///   does_the_ear_survive_the_chain -- --ignored --nocapture
/// ```
#[test]
#[ignore = "sonda -- a orelha sobrevive a' cadeia?"]
fn does_the_ear_survive_the_chain() {
    let reference = crate::sculpt3d::fixtures::eared_sphere();
    println!(
        "── orelha: {} vertices, {} faces ──",
        reference.vert_count(),
        reference.faces().len()
    );
    for alpha_mult in [1.0f32, 0.5] {
        let alpha = ph2d_remesh_iso::ALPHA * alpha_mult;
        let mut work = reference.clone();
        ph2d_remesh_iso::remesh_isotropic(&mut work, alpha);
        work.triangulate();
        // ⭐⭐ **O QUE O F1 JÁ CUSTOU**, antes de a cadeia começar.
        let (f1_p95, f1_max) = ph2d_quadfill::detail_lost(&reference, &work);
        println!(
            "  α {:.4} | work {:>7} tris | ⭐ o F1 sozinho ja' perdeu p95 {:.3} % max {:.3} % \
             da diagonal",
            alpha,
            work.faces().len(),
            100.0 * f1_p95,
            100.0 * f1_max
        );
        let dual = ph2d_crossfield::Dual::build(&work);
        let (field, _) = ph2d_crossfield::solve_miq(&dual);
        let layout = ph2d_trace::trace_patches(&work, &dual, &field);
        for detail in [1.0f32] {
            let target = ph2d_quadflow::edge_for_detail_with(
                &reference,
                detail,
                ph2d_quadflow::GLOBAL_FLOOR_IN_INPUT_EDGES,
            );
            let Ok(spec) = layout.to_layout(target) else {
                println!("    d={detail:.2} | o layout RECUSOU");
                continue;
            };
            let Ok((quant, _)) =
                ph2d_quantize::quantize_within(&spec, ph2d_quantize::Budget::new(256, 512))
            else {
                println!("    d={detail:.2} | a quantizacao RECUSOU");
                continue;
            };
            match ph2d_quadfill::fill(
                &work,
                &reference,
                &layout,
                &quant,
                ph2d_quadfill::SMOOTHING_ROUNDS,
            ) {
                #[allow(clippy::cast_precision_loss)]
                Ok((out, r)) => {
                    let (p95, mx) = ph2d_quadfill::detail_lost(&reference, &out);
                    println!(
                        "    d={detail:.2} alvo {target:.4} | {:>6} quads {:>4} irreg {:>3} bordo \
                         | dobras {:>4} ({:.2} %) | ⭐ PERDEU p95 {:.3} % max {:.3} % | patches {}",
                        r.quads,
                        r.irregular,
                        r.boundary_edges,
                        r.folded_local,
                        100.0 * r.folded_local as f64 / r.quads.max(1) as f64,
                        100.0 * p95,
                        100.0 * mx,
                        r.patches,
                    );
                }
                Err(e) => println!("    d={detail:.2} | a montagem RECUSOU {e:?}"),
            }
        }
    }
}
