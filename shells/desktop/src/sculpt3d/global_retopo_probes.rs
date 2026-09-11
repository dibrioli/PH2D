//! **AS SONDAS DA CADEIA GLOBAL, SEM GPU** — irmãs de [`super::global_retopo`].
//!
//! ⭐⭐ **O corte foi forçado pela HR-18 (607 linhas contra 600), mas ele é de
//! ASSUNTO:** lá moram os **gates do gesto**, que exigem uma cena e portanto um
//! device; aqui as **sondas da cadeia**, que não exigem nenhum.
//!
//! ⚠️ **E essa diferença é o achado que criou este ficheiro.** A cadeia F1..F5 e o
//! porte do Instant Meshes são aritmética pura sobre uma [`ph2d_mesh::Mesh`] — só
//! o *gesto* precisa de GPU. Enquanto a única porta para medir foi a cena, **toda**
//! medição herdou o `skip` gracioso da máquina sem adapter, e *skip gracioso não é
//! verde*.
//!
//! ⚠️ Elas continuam `#[ignore]` porque **imprimem tabelas e não afirmam nada** —
//! uma sonda que afirma é um gate, e mora no irmão.

use super::global_retopo::open_edges;

/// ⭐⭐ **SONDA — OS DOIS MOTORES NA MESMA PEÇA, SEM GPU E COM A RÉGUA CERTA.**
///
/// ⚠️ **Ela não precisa de device, e isso é o ponto.** A cadeia F1..F5 e o porte
/// do Instant Meshes são aritmética pura sobre uma [`ph2d_mesh::Mesh`]; só o
/// *gesto* precisa de GPU. Enquanto a única porta para medir foi a cena, toda
/// medição herdou o `skip` gracioso — e *skip gracioso não é verde*.
///
/// ⛔ **A pergunta com que ela nasceu — *"de onde vêm os buracos?"* — não tinha
/// resposta porque não havia buracos.** O censo abaixo mostra os arcos todos com
/// uso `2`, `boundary_edges = 0` nas quatro densidades, e o que a foto do Enio
/// mostra são **faces DOBRADAS**: eu li a coluna errada de uma tabela de sonda.
/// *Uma sonda que imprime seis números lado a lado precisa de rótulo em cada um.*
///
/// ```text
/// cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-sculpt3d && \
///   cargo test -p ph2d-host-desktop --release --bins \
///   the_two_engines_on_the_same_piece_without_a_device -- --ignored --nocapture
/// ```
#[test]
#[ignore = "sonda -- os dois motores na mesma peca, sem GPU"]
fn the_two_engines_on_the_same_piece_without_a_device() {
    for (name, which) in [
        ("ORELHA", 3u8),
        ("com BICO", 1),
        ("amassada", 0),
        ("com CRISTAS", 2),
    ] {
        for detail in [0.5f32, 1.0] {
            let reference = match which {
                1 => crate::sculpt3d::fixtures::hooked_sphere(),
                2 => crate::sculpt3d::fixtures::ridged_sphere(),
                3 => crate::sculpt3d::fixtures::eared_sphere(),
                _ => crate::sculpt3d::fixtures::wrinkled_sphere(),
            };
            // ⭐ O piso da cadeia GLOBAL, que é o que o produto usa desde 22/08.
            let target = ph2d_quadflow::edge_for_detail_with(
                &reference,
                detail,
                ph2d_quadflow::GLOBAL_FLOOR_IN_INPUT_EDGES,
            );

            // ── O motor LOCAL (porte BSD do Instant Meshes), pela mesma régua.
            let scale = ph2d_quadflow::ScaleField::adaptive(&reference, target, 0.0);
            let (orient, pos) = ph2d_quadflow::solve_fields(&reference, &scale);
            let local = ph2d_quadflow::extract(&reference, &orient, &pos, &scale)
                .map(|q| {
                    let f = ph2d_quadfill::folded_against(&reference, &q.mesh);
                    // ⭐⭐ **A régua do RELEVO no motor que é conhecido por o
                    // seguir.** O Instant Meshes suaviza um campo semeado na
                    // superfície; a nossa cadeia minimiza só suavidade. Se os dois
                    // derem o mesmo número, ou a régua não mede nada ou o
                    // diagnóstico está errado — *um controlo positivo é o que
                    // separa as duas hipóteses*.
                    let (deg, conf) = ph2d_quadfill::follows_relief(&reference, &q.mesh);
                    (q.quads, q.non_quads, q.holes, f, deg, conf)
                })
                .unwrap_or((0, 0, 0, 0, 45.0, 0.0));

            // ── A cadeia GLOBAL.
            let mut work = reference.clone();
            ph2d_remesh_iso::remesh_isotropic(&mut work, ph2d_remesh_iso::ALPHA);
            work.triangulate();
            let dual = ph2d_crossfield::Dual::build(&work);
            let (field, _) = ph2d_crossfield::solve_miq(&dual);
            let layout = ph2d_trace::trace_patches(&work, &dual, &field);
            // ⭐ O CENSO: quantos LADOS de patch citam cada arco. `2` é a lei, e
            // qualquer outro valor prevê arestas de bordo na saída.
            let mut used = vec![0usize; layout.arc_chain.len()];
            for sides in &layout.side_arcs {
                for side in sides {
                    for &(a, _) in side {
                        used[a as usize] += 1;
                    }
                }
            }
            let census: std::collections::BTreeMap<usize, usize> =
                used.iter()
                    .fold(std::collections::BTreeMap::new(), |mut m, &c| {
                        *m.entry(c).or_default() += 1;
                        m
                    });
            // ⭐⭐ **O CENSO virou ASSERÇÃO.** Ele nasceu como coluna impressa para
            // responder *"de onde vêm os buracos?"*, e a resposta foi que **não
            // havia buracos**: todo arco é citado por exactamente dois lados de
            // patch, em toda densidade e nas três fixturas. ⚠️ *Uma coluna que
            // imprime sempre o mesmo número é um invariante à espera de virar
            // barra* — e esta custa uma linha.
            assert_eq!(
                census.keys().copied().collect::<Vec<_>>(),
                vec![2],
                "{name} d={detail:.2}: ha' arco citado por um numero de lados != 2 \
                 ({census:?}) -- cada um vira aresta de bordo na saida"
            );
            let Ok(spec) = layout.to_layout(target) else {
                println!("{name:<12} d={detail:.2} | o layout RECUSOU");
                continue;
            };
            let Ok((quant, _)) =
                ph2d_quantize::quantize_within(&spec, ph2d_quantize::Budget::new(256, 512))
            else {
                println!("{name:<12} d={detail:.2} | a quantizacao RECUSOU");
                continue;
            };
            #[allow(clippy::cast_precision_loss)]
            match ph2d_quadfill::fill(
                &work,
                &reference,
                &layout,
                &quant,
                ph2d_quadfill::SMOOTHING_ROUNDS,
            ) {
                Ok((out, r)) => {
                    let (deg, conf) = ph2d_quadfill::follows_relief(&reference, &out);
                    println!(
                        "{name:<12} d={detail:.2} alvo {target:.4} | GLOBAL {:<5} quads \
                         {:<4} irreg {:<3} bordo | DOBRAS ref {:<4} ({:.1} %) vizinho {:<4} {:?} | med {:.2}x max {:.2}x \
                         | ⭐RELEVO {:>5.1}° (conf {:.2}) | ACHATOU {}/{} | arcos {:<4} promovidos {}",
                        r.quads,
                        r.irregular,
                        r.boundary_edges,
                        r.folded,
                        100.0 * r.folded as f64 / r.quads.max(1) as f64,
                        r.folded_local,
                        r.folded_prov,
                        r.edge_median / target,
                        r.edge_max / target,
                        deg,
                        conf,
                        r.flattened,
                        r.patches,
                        layout.arc_chain.len(),
                        layout.report.promoted,
                    );
                    debug_assert_eq!(
                        r.boundary_edges,
                        open_edges(&out),
                        "as duas reguas de bordo"
                    );
                }
                Err(e) => println!("{name:<12} d={detail:.2} | a montagem RECUSOU: {e:?}"),
            }
            #[allow(clippy::cast_precision_loss)]
            {
                println!(
                    "             {:>17} | LOCAL  {:<5} quads ({:.0}% quads) \
                     {:>28} {:<3} bordo {:<4} DOBRADAS | ⭐RELEVO {:>5.1}° (conf {:.2})",
                    "",
                    local.0,
                    100.0 * local.0 as f64 / (local.0 + local.1).max(1) as f64,
                    "",
                    local.2,
                    local.3,
                    local.4,
                    local.5,
                );
            }
        }
    }
}

/// ⭐⭐ **SONDA — DE QUEM É A DOBRA: DA CONSTRUÇÃO OU DA PROJEÇÃO?**
///
/// ⚠️ **A esfera com BICO dobra 5 a 7 % das faces; a amassada e a com cristas
/// dobram ZERO.** As três atravessam exactamente o mesmo código, então a
/// diferença é da **peça**, e a peça que difere é a que tem um pescoço fino e
/// comprido. Duas hipóteses explicam isso, e elas pedem curas opostas:
///
/// | # | hipótese | o que a curaria |
/// |---|---|---|
/// | **H1** | a superfície de Coons/leque **diverge da forma** sobre um patch grande e curvo | parametrização por patch (§4-quindecies) |
/// | **H2** | a **projeção salta o pescoço** — o ponto mais próximo está do OUTRO lado do gancho | projeção com direção, ou banda local ao patch |
///
/// ⭐ **Dois experimentos controlados, e cada um isola uma delas:**
///
/// 1. **Trocar a superfície de projeção** de `reference` para `work` (a remalhada
///    isotrópica). ⚠️ **A malha que INDEXA continua a ser `work` nos dois casos** —
///    é a assinatura de duas portas que torna esta troca exprimível sem repetir o
///    defeito de 2026-08-21. Se as dobras caírem, a dívida é da projeção (**H2**).
/// 2. **Varrer as rondas de alisamento** `0 · 6 · 24`. Se `0` já dobra tanto
///    quanto `6`, o estrago vem PRONTO da construção (**H1**) e o alisamento
///    apenas remedeia.
///
/// ```text
/// \
///   cargo test -p ph2d-host-desktop --release --bins \
///   whose_fold_is_it_the_construction_or_the_projection -- --ignored --nocapture
/// ```
#[test]
#[ignore = "sonda -- a dobra e' da construcao ou da projecao?"]
fn whose_fold_is_it_the_construction_or_the_projection() {
    for (name, which) in [("com BICO", 1u8), ("amassada", 0)] {
        let reference = if which == 1 {
            crate::sculpt3d::fixtures::hooked_sphere()
        } else {
            crate::sculpt3d::fixtures::wrinkled_sphere()
        };
        let target = ph2d_quadflow::edge_for_detail(&reference, 0.5);
        let mut work = reference.clone();
        ph2d_remesh_iso::remesh_isotropic(&mut work, ph2d_remesh_iso::ALPHA);
        work.triangulate();
        let dual = ph2d_crossfield::Dual::build(&work);
        let (field, _) = ph2d_crossfield::solve_miq(&dual);
        let layout = ph2d_trace::trace_patches(&work, &dual, &field);
        let Ok(spec) = layout.to_layout(target) else {
            continue;
        };
        let Ok((quant, _)) =
            ph2d_quantize::quantize_within(&spec, ph2d_quantize::Budget::new(256, 512))
        else {
            continue;
        };
        // ⭐ **O DÉFICE DE CADA ARCO** — o que o comprimento dele pedia contra o
        // que o F4 lhe deu. Um arco longo com poucos segmentos é uma **corda** a
        // atravessar uma região curva, e a grade de Coons construída sobre ela
        // nasce do lado errado da forma. ⚠️ É a fronteira do patch: o alisamento
        // move o interior e **não a desfaz**.
        let mut deficit: Vec<(f32, usize, u32, f32)> = layout
            .arc_length
            .iter()
            .enumerate()
            .map(|(a, &l)| {
                let want = l / target;
                #[allow(clippy::cast_precision_loss)]
                let got = quant.arc[a];
                #[allow(clippy::cast_precision_loss)]
                (want / (got as f32).max(1.0), a, got, l)
            })
            .collect();
        deficit.sort_by(|x, y| y.0.total_cmp(&x.0));
        println!(
            "{name:<10} arcos com o alvo > 2x o dado: {} de {} | piores: {}",
            deficit.iter().filter(|d| d.0 > 2.0).count(),
            deficit.len(),
            deficit
                .iter()
                .take(4)
                .map(|(r, a, g, l)| format!(
                    "#{a} pedia {:.1} deu {g} ({r:.1}x, len {l:.3})",
                    l / target
                ))
                .collect::<Vec<_>>()
                .join(" · ")
        );
        for (surface_name, surface) in [("ORIGINAL", &reference), ("remalhada", &work)] {
            for rounds in [0usize, 6, 24] {
                match ph2d_quadfill::fill(&work, surface, &layout, &quant, rounds) {
                    // ⚠️ **As dobras são sempre contadas contra a ORIGINAL**, mesmo
                    // quando a projeção foi noutra: senão a coluna mediria duas
                    // coisas diferentes e as linhas deixariam de se comparar.
                    Ok((out, r)) => {
                        #[allow(clippy::cast_precision_loss)]
                        let pct = 100.0 * ph2d_quadfill::folded_against(&reference, &out) as f64
                            / r.quads.max(1) as f64;
                        println!(
                            "{name:<10} projeta na {surface_name:<10} alis={rounds:<3} | \
                             {:<5} quads DOBRADAS {:<4} ({pct:.1} %) | med {:.2}x max {:.2}x",
                            r.quads,
                            ph2d_quadfill::folded_against(&reference, &out),
                            r.edge_median / target,
                            r.edge_max / target,
                        );
                    }
                    Err(e) => println!("{name:<10} {surface_name} alis={rounds} | recusou {e:?}"),
                }
            }
        }
        println!("── {name} ──");
    }
}
