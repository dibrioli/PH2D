//! ⭐⭐ **O ALINHAMENTO NÃO PODE MUDAR O GÉNERO DA PEÇA** — e este ficheiro nasce
//! de o ter mudado.
//!
//! ⛔ **Medido em 2026-08-22, ao ligar o `ALIGN_WEIGHT`:** o toro saiu com
//! `V − E + F = 2` onde a topologia exige `0` — **e passou em todas as outras
//! réguas**: 100 % de quads, zero arestas de bordo, contagem de irregulares na
//! ordem certa. *Um remesh que fecha o buraco de um toro é indistinguível de um
//! bom remesh em toda coluna que não seja esta.*
//!
//! ⚠️ **A varredura do peso não continha um toro**, e é essa a lição de fixtura:
//! as cinco peças que escolheram o número eram todas de género 0, e nenhuma delas
//! podia exprimir o defeito (`reference_topic_fixture_discipline`).

use ph2d_crossfield::{Dual, Rounding, solve_miq_aligned};
use ph2d_mesh::{Mesh, shapes};
use ph2d_quadfill::{SMOOTHING_ROUNDS, fill};
use ph2d_quantize::quantize_within;
use ph2d_trace::trace_patches;

/// `V − E + F`, contando cada aresta uma vez.
fn euler(mesh: &Mesh) -> i64 {
    let mut edges = std::collections::BTreeSet::new();
    for f in mesh.faces() {
        let v = f.verts();
        for k in 0..v.len() {
            let (a, b) = (v[k], v[(k + 1) % v.len()]);
            edges.insert((a.min(b), a.max(b)));
        }
    }
    i64::try_from(mesh.vert_count()).unwrap() - i64::try_from(edges.len()).unwrap()
        + i64::try_from(mesh.face_count()).unwrap()
}

/// ⚠️ **Quantas arestas têm mais de duas faces** — a assinatura do NÃO-MANIFOLD.
///
/// ⭐ Ela existe porque `boundary_edges == 0` **não** quer dizer *"a malha está
/// bem"*: uma aresta com três faces também não é de bordo, e é uma das poucas
/// formas de a característica de Euler mudar sem a casca se abrir.
fn nonmanifold_edges(mesh: &Mesh) -> usize {
    use std::collections::BTreeMap;
    let mut e: BTreeMap<(u32, u32), usize> = BTreeMap::new();
    for f in mesh.faces() {
        let v = f.verts();
        for k in 0..v.len() {
            let (a, b) = (v[k], v[(k + 1) % v.len()]);
            *e.entry((a.min(b), a.max(b))).or_default() += 1;
        }
    }
    e.values().filter(|c| **c > 2).count()
}

/// A cadeia do produto, com o peso do alinhamento na mão.
fn chain(mesh: &Mesh, target_edge: f32, align: f32) -> Option<(Mesh, ph2d_quadfill::FillReport)> {
    let mut work = mesh.clone();
    ph2d_remesh_iso::remesh_isotropic(&mut work, ph2d_remesh_iso::ALPHA);
    work.triangulate();
    let dual = Dual::build(&work);
    let (field, _) = solve_miq_aligned(&dual, Rounding::default(), align);
    let layout = trace_patches(&work, &dual, &field);
    let l = layout.to_layout(target_edge).ok()?;
    let (q, _) = quantize_within(&l, ph2d_quantize::Budget::new(256, 512)).ok()?;
    fill(&work, mesh, &layout, &q, SMOOTHING_ROUNDS).ok()
}

/// ⭐⭐ **O GÉNERO SOBREVIVE AO PESO QUE SHIPA** — e é o guarda do
/// `ALIGN_WEIGHT`.
///
/// ⛔ **É a asserção que reprovou o `0,03` no dia em que ele foi escolhido.** A
/// varredura do peso (cinco fixturas, todas de género `0`) elegeu-o pelo relevo, e
/// este gate mostrou que ele parte **dois dos três toros** — `χ = 2` onde a
/// topologia exige `0`, com 100 % de quads, zero bordo e zero não-manifold. *Uma
/// peça pode passar em toda asserção e ter deixado de ser um toro.*
///
/// ⭐ **Ele varre `[0, ALIGN_WEIGHT]`, então liga-se sozinho** no dia em que a
/// constante sair de zero: quem a mover encontra este gate vermelho antes de
/// encontrar o artista. ⛔ A barra não se afrouxa — `χ` é topologia.
///
/// ⚠️ **Falta-lhe uma fixtura de propósito**, e o irmão `#[ignore]`
/// [`the_genus_survives_on_every_torus`] diz qual e porquê.
#[test]
fn the_genus_survives_the_shipped_weight() {
    for (name, mesh, edge, want) in [
        ("esfera 24x36", shapes::uv_sphere(24, 36, 1.0), 0.08, 2),
        ("toro 32x16", shapes::torus(32, 16, 1.0, 0.35), 0.06, 0),
        ("toro 64x32", shapes::torus(64, 32, 1.0, 0.35), 0.06, 0),
    ] {
        for w in [0.0f32, ph2d_crossfield::ALIGN_WEIGHT] {
            check_genus(name, &mesh, edge, want, w);
        }
    }
}

/// ⛔⛔ **VERMELHO PRÉ-EXISTENTE, medido em 2026-08-22 — o traçado PERDE ASAS.**
///
/// ⚠️ **O toro 48×24 sai com `χ = 2` HOJE, no produto, com o alinhamento
/// DESLIGADO.** Não é regressão desta linha nem do termo de alinhamento: é um
/// defeito antigo que ninguém tinha medido, porque **nenhuma fixtura gateada o
/// continha** — o único toro do corpus era o 32×16, e nele o defeito não aparece.
///
/// ⭐ **E a sonda [`where_is_the_genus_lost`] já disse ONDE:** o *complexo* de
/// patches (`V − E + F` sobre cantos, arcos e patches) já sai com `χ = 2` **antes**
/// de a montagem começar, com todos os arcos usados exactamente duas vezes e nenhum
/// patch não-disco. A malha de entrada tem `χ = 0`; a montagem realiza fielmente o
/// que o layout manda. ⇒ **A asa perde-se no F3, o traçado.**
///
/// ⭐⭐ **E desde 2026-08-22 a cadeia RECUSA em vez de mentir** — a cerca do
/// complexo (`LayoutError::GenusLost`) apanha exactamente estes casos. ⚠️ **Por
/// isso este gate exige uma MALHA**, não apenas uma que esteja certa: uma recusa
/// continua a ser o traçado a não saber decompor um toro, e um gate que aceitasse
/// a recusa ficaria verde sobre o bloqueio que ele existe para nomear.
///
/// ⚠️ **`#[ignore]` porque é vermelho, não porque é lento** — e fica assim para o
/// bloqueio ter endereço em vez de se dissolver numa nota. É o item nº 1 do
/// `PLAN.md` §4-duovicies, e é o que trava o `ALIGN_WEIGHT`.
///
/// ```text
/// cd .../Worktrees/line-sculpt3d && cargo test -p ph2d-quadfill --release \
///     --test alignment_topology -- --ignored the_genus_survives_on_every_torus
/// ```
#[test]
#[ignore = "VERMELHO PRE-EXISTENTE -- o tracado nao decompoe o toro 48x24, ver o doc"]
fn the_genus_survives_on_every_torus() {
    for (name, mesh, edge) in [
        ("toro 32x16", shapes::torus(32, 16, 1.0, 0.35), 0.06),
        ("toro 48x24", shapes::torus(48, 24, 1.0, 0.35), 0.06),
        ("toro 64x32", shapes::torus(64, 32, 1.0, 0.35), 0.06),
    ] {
        let out = chain(&mesh, edge, 0.0);
        assert!(
            out.is_some(),
            "{name}: a cadeia RECUSOU -- o tracado nao sabe decompor este toro"
        );
        check_genus(name, &mesh, edge, 0, 0.0);
    }
}

/// A asserção que os dois gates partilham — uma cópia por gate divergiria no dia
/// em que um deles passasse a aceitar não-manifold.
fn check_genus(name: &str, mesh: &Mesh, edge: f32, want: i64, w: f32) {
    let Some((out, r)) = chain(mesh, edge, w) else {
        // ⚠️ Uma recusa é um resultado legítimo (a porta do produto cai para o
        // campo liso); o que estes gates proíbem é uma malha **errada**.
        eprintln!("  {name} peso {w}: a cadeia RECUSOU — nada a afirmar");
        return;
    };
    let x = euler(&out);
    let nm = nonmanifold_edges(&out);
    eprintln!(
        "  {name} peso {w:<5}: X {x:>3} (quer {want}) | {} quads {} nao-quads \
         {} bordo {nm} nao-manifold {} irreg",
        r.quads, r.non_quads, r.boundary_edges, r.irregular
    );
    assert_eq!(
        nm, 0,
        "{name} peso {w}: {nm} aresta(s) com mais de duas faces -- a malha e' NAO-MANIFOLD"
    );
    assert_eq!(
        x, want,
        "{name} peso {w}: X = {x} e a topologia exige {want} -- o remesh mudou o genero"
    );
}

/// ⭐⭐ **SONDA — QUE PESOS DE ALINHAMENTO PRESERVAM O GÉNERO?**
///
/// ⚠️ **Ela é a varredura que faltava quando o `ALIGN_WEIGHT` foi escolhido.** As
/// cinco fixturas que votaram no número eram todas de género `0`; o toro é a única
/// peça do corpus em que a colagem errada dos patches tem onde se manifestar.
///
/// A coluna que decide é `X`; as outras estão lá para mostrar que **nenhuma delas
/// pisca** quando ela erra.
///
/// ```text
/// cd .../Worktrees/line-sculpt3d && cargo test -p ph2d-quadfill --release \
///     --test alignment_topology -- --ignored --nocapture
/// ```
#[test]
#[ignore = "sonda -- que pesos de alinhamento preservam o genero"]
fn which_alignment_weights_keep_the_genus() {
    for (name, mesh, edge, want) in [
        ("toro 32x16", shapes::torus(32, 16, 1.0, 0.35), 0.06, 0),
        ("toro 48x24", shapes::torus(48, 24, 1.0, 0.35), 0.06, 0),
        ("esfera 24x36", shapes::uv_sphere(24, 36, 1.0), 0.08, 2),
    ] {
        println!("── {name} (quer X = {want}) ──");
        for w in [0.0f32, 0.002, 0.005, 0.01, 0.02, 0.03, 0.05, 0.1, 0.2] {
            let Some((out, r)) = chain(&mesh, edge, w) else {
                println!("  peso {w:<5} | a cadeia RECUSOU");
                continue;
            };
            let x = euler(&out);
            println!(
                "  peso {w:<5} | X {x:>3} {} | {:>5} quads {:>3} irreg {:>2} bordo \
                 {:>2} nao-manifold",
                if x == want { "✓" } else { "⛔" },
                r.quads,
                r.irregular,
                r.boundary_edges,
                nonmanifold_edges(&out),
            );
        }
    }
}

/// ⭐⭐ **SONDA — A TOPOLOGIA PERDE-SE NO F3 OU NO F5?**
///
/// ⛔ **A pergunta que decide onde procurar**, e as duas hipóteses prescrevem
/// consertos em fases diferentes:
///
/// | onde | como se vê |
/// |---|---|
/// | **F3** (o traçado) | o COMPLEXO de patches já tem o `χ` errado — `V − E + F` sobre *cantos, arcos, patches* |
/// | **F5** (a montagem) | o complexo está certo e a malha sai errada — a colagem identificou coisas que não são a mesma |
///
/// ⚠️ **O complexo é um CW**: cada patch é uma face, cada arco uma aresta, cada
/// canto um vértice. Um arco fechado sobre um canto só continua a contar `1` e `1`,
/// e a fórmula aguenta.
///
/// ```text
/// cd .../Worktrees/line-sculpt3d && cargo test -p ph2d-quadfill --release \
///     --test alignment_topology -- --ignored --nocapture where_is_the_genus_lost
/// ```
#[test]
#[ignore = "sonda -- a topologia perde-se no tracado ou na montagem?"]
fn where_is_the_genus_lost() {
    for (name, mesh, edge, want) in [
        ("toro 32x16", shapes::torus(32, 16, 1.0, 0.35), 0.06, 0),
        ("toro 48x24", shapes::torus(48, 24, 1.0, 0.35), 0.06, 0),
        ("toro 64x32", shapes::torus(64, 32, 1.0, 0.35), 0.06, 0),
    ] {
        println!("── {name} (quer X = {want}) ──");
        for w in [0.0f32, 0.03] {
            let mut work = mesh.clone();
            ph2d_remesh_iso::remesh_isotropic(&mut work, ph2d_remesh_iso::ALPHA);
            work.triangulate();
            println!("  work X = {} ({} tris)", euler(&work), work.faces().len());
            let dual = Dual::build(&work);
            let (field, _) = solve_miq_aligned(&dual, Rounding::default(), w);
            let layout = trace_patches(&work, &dual, &field);

            // O complexo: cantos · arcos · patches.
            let faces = layout.side_arcs.len() as i64;
            let arcs = layout.arc_chain.len() as i64;
            let corners: std::collections::BTreeSet<u32> =
                layout.corners.iter().flatten().copied().collect();
            let complex = corners.len() as i64 - arcs + faces;
            // ⚠️ Quantos arcos cada patch NOMEIA, com multiplicidade — se um arco
            // aparecer mais ou menos de duas vezes, a colagem não é uma superfície.
            let mut used: std::collections::BTreeMap<u32, usize> =
                std::collections::BTreeMap::new();
            for p in &layout.side_arcs {
                for side in p {
                    for (a, _) in side {
                        *used.entry(*a).or_default() += 1;
                    }
                }
            }
            let once = used.values().filter(|c| **c == 1).count();
            let many = used.values().filter(|c| **c > 2).count();
            println!(
                "  peso {w:<5} | COMPLEXO X {complex:>3} {} (V {} E {arcs} F {faces}) \
                 | arcos usados 1x: {once}  >2x: {many}  nao-disco: {}",
                if complex == want { "✓" } else { "⛔" },
                corners.len(),
                layout.loops_per_patch.iter().filter(|l| **l != 1).count(),
            );
            let Some((out, _)) = chain(&mesh, edge, w) else {
                println!("  peso {w:<5} | a cadeia RECUSOU a jusante");
                continue;
            };
            println!(
                "  peso {w:<5} | MALHA    X {:>3} {}",
                euler(&out),
                if euler(&out) == want { "✓" } else { "⛔" }
            );
        }
    }
}

/// ⭐⭐ **SONDA — CADA PATCH É MESMO UM DISCO?**
///
/// ⛔ **A conta que a hipótese exige.** O complexo sai com `χ = 2` onde a peça é
/// `0`, com todos os arcos usados duas vezes e `loops_per_patch == 1` em todos —
/// ou seja, **o traçado afirma que todo patch é um disco**. Se dois deles forem
/// **anéis**, contá-los como discos sobre-estima o `χ` em exactamente `+1` cada, e
/// `+2` é a diferença medida.
///
/// ⚠️ **A régua é a do MESMO patch como sub-malha** (`V − E + F` sobre as faces que
/// ele contém), e não a contagem de fronteiras que o traçado publica: *o que se quer
/// saber é se o `loops_per_patch` está a mentir*, e perguntar-lhe seria pedir-lhe
/// para se auto-conferir.
///
/// | `χ` do patch | o que ele é |
/// |---|---|
/// | `1` | ⭐ um disco — o que a fase promete |
/// | `0` | ⛔ um ANEL (ou um cilindro): duas fronteiras |
/// | `< 0` | ⛔ pior: uma asa inteira dentro de um patch |
///
/// ```text
/// cd .../Worktrees/line-sculpt3d && cargo test -p ph2d-quadfill --release \
///     --test alignment_topology -- --ignored --nocapture is_every_patch_a_disk
/// ```
#[test]
#[ignore = "sonda -- cada patch e' mesmo um disco?"]
fn is_every_patch_a_disk() {
    for (name, mesh, w) in [
        (
            "toro 32x16 peso 0 (BOM)",
            shapes::torus(32, 16, 1.0, 0.35),
            0.0,
        ),
        (
            "toro 48x24 peso 0 (MAU)",
            shapes::torus(48, 24, 1.0, 0.35),
            0.0,
        ),
        (
            "toro 64x32 peso 0 (BOM)",
            shapes::torus(64, 32, 1.0, 0.35),
            0.0,
        ),
    ] {
        let mut work = mesh.clone();
        ph2d_remesh_iso::remesh_isotropic(&mut work, ph2d_remesh_iso::ALPHA);
        work.triangulate();
        let dual = Dual::build(&work);
        let (field, _) = solve_miq_aligned(&dual, Rounding::default(), w);
        let layout = trace_patches(&work, &dual, &field);

        let n = layout.side_arcs.len();
        let mut chis: Vec<i64> = Vec::with_capacity(n);
        for p in 0..n {
            let mut verts = std::collections::BTreeSet::new();
            let mut edges = std::collections::BTreeSet::new();
            let mut faces = 0i64;
            for (f, owner) in layout.face_patch.iter().enumerate() {
                if *owner as usize != p {
                    continue;
                }
                faces += 1;
                let v = work.faces()[f].verts();
                for k in 0..v.len() {
                    verts.insert(v[k]);
                    let (a, b) = (v[k], v[(k + 1) % v.len()]);
                    edges.insert((a.min(b), a.max(b)));
                }
            }
            chis.push(verts.len() as i64 - edges.len() as i64 + faces);
        }
        let discs = chis.iter().filter(|c| **c == 1).count();
        let rings = chis.iter().filter(|c| **c == 0).count();
        let worse = chis.iter().filter(|c| **c < 0).count();
        let sum: i64 = chis.iter().sum();
        println!(
            "── {name}: {n} patches | discos {discs}  ANEIS {rings}  pior {worse} \
             | Σχ(patch) {sum} | o tracado diz nao-disco: {}",
            layout.loops_per_patch.iter().filter(|l| **l != 1).count()
        );
        for (p, c) in chis.iter().enumerate() {
            if *c != 1 {
                println!(
                    "    ⛔ patch {p}: χ {c}, {} lados, loops_per_patch {}",
                    layout.side_arcs[p].len(),
                    layout.loops_per_patch[p]
                );
            }
        }
    }
}

/// ⭐⭐ **SONDA — O QUE O TRAÇADO DIZ DE SI PRÓPRIO nos três toros?**
///
/// ⛔ **A pergunta que escolhe a cura.** Sabe-se *onde* a asa se perde (um patch
/// engole-a) e que `dissolve` é a direcção errada. Falta saber **por que é que a
/// rede de paredes é frouxa ali** — e as hipóteses prescrevem trabalhos diferentes:
///
/// | se o relatório disser | então a cura é |
/// |---|---|
/// | poucas **singularidades** | o campo é liso demais ali; seria preciso semear paredes fora delas |
/// | muitas **`dangling`** | as paredes foram traçadas e **descartadas** por não fecharem — recuperá-las densifica exactamente onde falta |
/// | muitos **`promoted`** | a fronteira existe mas não tem cantos estruturais — outro problema |
///
/// ```text
/// cd .../Worktrees/line-sculpt3d && cargo test -p ph2d-quadfill --release \
///     --test alignment_topology -- --ignored --nocapture what_does_the_trace_say
/// ```
#[test]
#[ignore = "sonda -- o que o tracado diz de si proprio nos tres toros"]
fn what_does_the_trace_say() {
    for (name, mesh) in [
        ("toro 32x16 (BOM)", shapes::torus(32, 16, 1.0, 0.35)),
        ("toro 48x24 (MAU)", shapes::torus(48, 24, 1.0, 0.35)),
        ("toro 64x32 (BOM)", shapes::torus(64, 32, 1.0, 0.35)),
        ("esfera 24x36", shapes::uv_sphere(24, 36, 1.0)),
    ] {
        let mut work = mesh.clone();
        ph2d_remesh_iso::remesh_isotropic(&mut work, ph2d_remesh_iso::ALPHA);
        work.triangulate();
        let dual = Dual::build(&work);
        let (field, _) = solve_miq_aligned(&dual, Rounding::default(), 0.0);
        let layout = trace_patches(&work, &dual, &field);
        let r = &layout.report;
        println!(
            "── {name} ({} tris) ──\n  \
             singularidades {:<4} separatrizes {:<4} DANGLING {:<4} promovidos {:<4}\n  \
             patches {:<4} arcos {:<4} nao-disco {:<3} COM GENERO {:<3} dissolvidas {:<3} \
             rondas {:<3} aneis-abertos {}\n  \
             complexo X {:>3} (a peca e' {})  |  valencias {:?}",
            work.faces().len(),
            r.singularities,
            r.separatrices,
            r.dangling,
            r.promoted,
            r.patches,
            r.arcs,
            r.non_disk,
            r.with_genus,
            r.dissolved,
            r.rounds,
            r.open_rings,
            layout.complex_euler(),
            layout.mesh_chi,
            r.valence,
        );
    }
}

/// ⭐⭐ **SONDA — A LIMPEZA, RONDA A RONDA: em qual delas a asa nasce?**
///
/// ⛔ **O relatório mudou a suspeita de sítio.** No toro que falha, o traçado
/// produz **10 singularidades e 30 separatrizes** — tanto quanto os que passam — e
/// descarta **zero**. O que ele tem a mais é `dissolvidas 10, rondas 10`, contra
/// `1` e `0` nos bons. ⇒ *A rede de paredes estava boa; foi a LIMPEZA que a comeu.*
///
/// Esta sonda repete o laço de [`ph2d_trace::trace_patches`] à mão e imprime o
/// estado a cada ronda. A coluna que decide é `complexo`: a ronda em que ele deixa
/// de bater com o `χ` da peça é a ronda que criou a asa.
///
/// ```text
/// cd .../Worktrees/line-sculpt3d && cargo test -p ph2d-quadfill --release \
///     --test alignment_topology -- --ignored --nocapture where_does_the_cleanup_break_it
/// ```
#[test]
#[ignore = "sonda -- em que ronda da limpeza a asa nasce"]
fn where_does_the_cleanup_break_it() {
    use ph2d_trace::patches::{decompose, dissolve};
    use ph2d_trace::walk::Walker;

    for (name, mesh) in [
        ("toro 32x16 (BOM)", shapes::torus(32, 16, 1.0, 0.35)),
        ("toro 48x24 (MAU)", shapes::torus(48, 24, 1.0, 0.35)),
        ("esfera 48x72", shapes::uv_sphere(48, 72, 1.0)),
        ("esfera 24x36", shapes::uv_sphere(24, 36, 1.0)),
    ] {
        let mut work = mesh.clone();
        ph2d_remesh_iso::remesh_isotropic(&mut work, ph2d_remesh_iso::ALPHA);
        work.triangulate();
        let dual = Dual::build(&work);
        let (field, _) = solve_miq_aligned(&dual, Rounding::default(), 0.0);
        let walker = Walker::new(&work, &dual, &field);
        let (mut walls, base) = walker.trace_all();
        let mut out = decompose(&work, &walls, base.clone());
        println!("── {name} (a peca e' χ {}) ──", out.mesh_chi);
        for round in 0..20 {
            let victims = out.degenerate();
            println!(
                "  ronda {round:<2} | patches {:<3} arcos {:<3} complexo χ {:>3} {} \
                 | com-genero {} | degenerados {:?}",
                out.side_arcs.len(),
                out.arc_chain.len(),
                out.complex_euler(),
                if out.complex_euler() == out.mesh_chi {
                    "✓"
                } else {
                    "⛔"
                },
                out.report.with_genus,
                victims,
            );
            if victims.is_empty() || !dissolve(&mut walls, &out, &victims) {
                break;
            }
            out = decompose(&work, &walls, base.clone());
        }
    }
}

/// ⭐⭐ **SONDA — ALGUMA parede curaria a asa, ou nenhuma cura?**
///
/// ⛔ **A pergunta que resta depois de a limpeza ser inocentada.** O patch com
/// género existe já na ronda 0, nos DOIS toros: no que passa, **uma** dissolução
/// cura-o (χ 1 → 0); no que falha, dez não curam. O `dissolve` escolhe sempre o
/// **lado mais curto** — então a hipótese é que a escolha é que está errada, não a
/// operação.
///
/// Esta sonda tenta **cada lado** do patch com género, um de cada vez, a partir do
/// mesmo estado, e mede o que cada escolha dá. As duas leituras possíveis:
///
/// | resultado | o que significa |
/// |---|---|
/// | ⭐ algum lado leva `com-género` a `0` | a cura é **escolher**, e é barata |
/// | ⛔ nenhum leva | dissolver não alcança esta asa — é preciso **cortar**, e isso é trabalho novo |
///
/// ```text
/// cd .../Worktrees/line-sculpt3d && cargo test -p ph2d-quadfill --release \
///     --test alignment_topology -- --ignored --nocapture would_any_wall_cure_the_handle
/// ```
#[test]
#[ignore = "sonda -- alguma parede curaria a asa?"]
fn would_any_wall_cure_the_handle() {
    use ph2d_trace::patches::{decompose, dissolve};
    use ph2d_trace::walk::Walker;

    for (name, mesh) in [
        ("toro 32x16 (BOM)", shapes::torus(32, 16, 1.0, 0.35)),
        ("toro 48x24 (MAU)", shapes::torus(48, 24, 1.0, 0.35)),
    ] {
        let mut work = mesh.clone();
        ph2d_remesh_iso::remesh_isotropic(&mut work, ph2d_remesh_iso::ALPHA);
        work.triangulate();
        let dual = Dual::build(&work);
        let (field, _) = solve_miq_aligned(&dual, Rounding::default(), 0.0);
        let walker = Walker::new(&work, &dual, &field);
        let (walls, base) = walker.trace_all();
        let out = decompose(&work, &walls, base.clone());
        let Some(guilty) = (0..out.chi.len()).find(|&p| out.chi[p] != 1) else {
            println!("── {name}: nenhum patch com género na ronda 0");
            continue;
        };
        println!(
            "── {name} (a peca e' χ {}) | patch culpado {guilty}: χ {} com {} lados ──",
            out.mesh_chi,
            out.chi[guilty],
            out.side_arcs[guilty].len()
        );
        // ⚠️ **Cada tentativa parte do MESMO estado**, e é por isso que as paredes
        // são clonadas: sem isso a segunda escolha mediria o efeito das duas.
        for s in 0..out.side_arcs[guilty].len() {
            let mut trial = walls.clone();
            // Dissolve UM lado — o `dissolve` público escolhe o mais curto, então
            // aqui as arestas do lado `s` são removidas à mão.
            for &(a, _) in &out.side_arcs[guilty][s] {
                for e in &out.arc_edges[a as usize] {
                    trial.edges.remove(e);
                }
            }
            let next = decompose(&work, &trial, base.clone());
            println!(
                "    lado {s:<2} | patches {:<3} complexo χ {:>3} {} | com-genero {} \
                 | degenerados {}",
                next.side_arcs.len(),
                next.complex_euler(),
                if next.complex_euler() == next.mesh_chi {
                    "✓"
                } else {
                    "⛔"
                },
                next.report.with_genus,
                next.degenerate().len(),
            );
        }
        let _ = dissolve;
    }
}

/// ⭐⭐ **SONDA — e um PAR de paredes cura?**
///
/// ⛔ **A sonda irmã fechou meia pergunta:** no toro que passa, 3 dos 6 lados curam
/// e 3 não; no que falha, **nenhum dos 6**. O `dissolve` escolhe o mais curto, ou
/// seja, escolhe **por sorte**. Falta saber se a cura existe mais fundo:
///
/// | resultado | o que significa |
/// |---|---|
/// | ⭐ algum PAR cura | a cura é **procurar**, e o custo é uma busca pequena |
/// | ⛔ nenhum par cura | dissolver não alcança esta asa — é preciso **cortar** |
///
/// ```text
/// cd .../Worktrees/line-sculpt3d && cargo test -p ph2d-quadfill --release \
///     --test alignment_topology -- --ignored --nocapture would_a_pair_of_walls_cure_it
/// ```
#[test]
#[ignore = "sonda -- um par de paredes cura a asa?"]
fn would_a_pair_of_walls_cure_it() {
    use ph2d_trace::patches::decompose;
    use ph2d_trace::walk::Walker;

    let mesh = shapes::torus(48, 24, 1.0, 0.35);
    let mut work = mesh.clone();
    ph2d_remesh_iso::remesh_isotropic(&mut work, ph2d_remesh_iso::ALPHA);
    work.triangulate();
    let dual = Dual::build(&work);
    let (field, _) = solve_miq_aligned(&dual, Rounding::default(), 0.0);
    let walker = Walker::new(&work, &dual, &field);
    let (walls, base) = walker.trace_all();
    let out = decompose(&work, &walls, base.clone());
    let guilty = (0..out.chi.len())
        .find(|&p| out.chi[p] != 1)
        .expect("ha' um patch com genero");
    let n = out.side_arcs[guilty].len();
    println!(
        "── toro 48x24, patch {guilty}, {n} lados: os {} pares ──",
        n * (n - 1) / 2
    );
    let mut cured = 0usize;
    for i in 0..n {
        for j in (i + 1)..n {
            let mut trial = walls.clone();
            for s in [i, j] {
                for &(a, _) in &out.side_arcs[guilty][s] {
                    for e in &out.arc_edges[a as usize] {
                        trial.edges.remove(e);
                    }
                }
            }
            let next = decompose(&work, &trial, base.clone());
            let ok = next.complex_euler() == next.mesh_chi && next.report.with_genus == 0;
            if ok {
                cured += 1;
            }
            println!(
                "    {i}+{j} | patches {:<3} complexo χ {:>3} {} | com-genero {} | degenerados {}",
                next.side_arcs.len(),
                next.complex_euler(),
                if ok { "⭐" } else { "⛔" },
                next.report.with_genus,
                next.degenerate().len(),
            );
        }
    }
    println!("  ⇒ pares que curam: {cured} de {}", n * (n - 1) / 2);
}

/// ⭐⭐ **SONDA — já existem paredes INTERIORES a um patch?**
///
/// ⛔ **A pergunta que precifica o corte.** O `boundary_loops` só conta uma aresta
/// de parede como fronteira quando a face do outro lado é de **outro** patch:
///
/// ```text
///     walls.blocks(a, b) && half[(b, a)] pertence a outro patch
/// ```
///
/// ⇒ uma **ponte** interior a um patch — que é o que corta uma asa — seria
/// invisível. Tirar a segunda condição é a representação *"cortar e abrir"*, mas ela
/// mexe numa rotina com dez gates em cima, e o preço depende desta contagem:
///
/// | se hoje houver | então tirar a condição |
/// |---|---|
/// | ⭐ **zero** paredes interiores | é **inerte** — nada existente muda, e só as pontes novas passam a ser vistas |
/// | algumas | muda decomposições que hoje funcionam, e cada uma precisa de ser medida |
///
/// ⛔ **A primeira versão desta sonda contou errado**, e o erro é instrutivo: ela
/// media as paredes **CRUAS** contra os patches **LIMPOS** — duas coisas que não se
/// correspondem, porque a limpeza dissolve paredes. Dava `204` onde o número é `18`.
/// *A contagem tem de ser feita onde os dois existem ao mesmo tempo*, e por isso ela
/// vive agora no `TraceReport::interior_walls`, dentro do `decompose`.
///
/// ```text
/// cd .../Worktrees/line-sculpt3d && cargo test -p ph2d-quadfill --release \
///     --test alignment_topology -- --ignored --nocapture are_there_interior_walls_today
/// ```
#[test]
#[ignore = "sonda -- ja' existem paredes interiores a um patch?"]
fn are_there_interior_walls_today() {
    use ph2d_trace::patches::decompose;
    use ph2d_trace::walk::Walker;

    for (name, mesh) in [
        ("esfera 24x36", shapes::uv_sphere(24, 36, 1.0)),
        ("esfera 48x72", shapes::uv_sphere(48, 72, 1.0)),
        ("toro 32x16", shapes::torus(32, 16, 1.0, 0.35)),
        ("toro 48x24", shapes::torus(48, 24, 1.0, 0.35)),
        ("toro 64x32", shapes::torus(64, 32, 1.0, 0.35)),
        ("cubo subdividido", shapes::sphere_with_triangles(4000, 1.0)),
    ] {
        let mut work = mesh.clone();
        ph2d_remesh_iso::remesh_isotropic(&mut work, ph2d_remesh_iso::ALPHA);
        work.triangulate();
        let dual = Dual::build(&work);
        let (field, _) = solve_miq_aligned(&dual, Rounding::default(), 0.0);
        let walker = Walker::new(&work, &dual, &field);
        let (walls, base) = walker.trace_all();
        let raw = decompose(&work, &walls, base);
        let cleaned = ph2d_trace::trace_patches(&work, &dual, &field);
        println!(
            "  {name:<18} | paredes {:<5} | CRU: interiores {:<4} patches {:<3} \
             | LIMPO: interiores {:<4} patches {:<3} ({} rondas)",
            walls.edges.len(),
            raw.report.interior_walls,
            raw.side_arcs.len(),
            cleaned.report.interior_walls,
            cleaned.side_arcs.len(),
            cleaned.report.rounds,
        );
    }
}
