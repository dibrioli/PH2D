//! **A SONDA DA CADEIA COMPLETA** — e é a primeira que devolve MALHA.
//!
//! ```text
//! cd .../Worktrees/line-sculpt3d && cargo test -p ph2d-quadfill --release \
//!     --test pipeline -- --ignored --nocapture
//! ```

use ph2d_crossfield::{Dual, singularities, solve_miq};
use ph2d_mesh::{Mesh, shapes};
use ph2d_quadfill::{SMOOTHING_ROUNDS, fill};
use ph2d_quantize::{Budget, quantize_within};
use ph2d_trace::trace_patches;

fn run(name: &str, mesh: Mesh, target_edge: f32) {
    run_maybe_iso(name, mesh, target_edge, false);
}

/// ⭐ **A cadeia como o PRODUTO a vai correr** — com o F1 na frente.
///
/// ⚠️ **A diferença não é cosmética.** Sem o F1 a saída herda a densidade da
/// entrada, que é a propriedade inteira que aquele passe existe para destruir; e
/// foi só com ele ligado que o defeito de variedade do flip apareceu, porque é o
/// laço dele que o acumula.
fn run_iso(name: &str, mesh: Mesh, target_edge: f32) {
    run_maybe_iso(name, mesh, target_edge, true);
}

fn run_maybe_iso(name: &str, mesh: Mesh, target_edge: f32, iso: bool) {
    run_with_alpha(
        name,
        mesh,
        target_edge,
        iso.then_some(ph2d_remesh_iso::ALPHA),
    );
}

fn run_with_alpha(name: &str, mut mesh: Mesh, target_edge: f32, alpha: Option<f32>) {
    eprintln!("[f5] {name}: {} vertices…", mesh.vert_count());
    let t = std::time::Instant::now();
    if let Some(alpha) = alpha {
        let r = ph2d_remesh_iso::remesh_isotropic(&mut mesh, alpha);
        eprintln!(
            "[f5] {name}: iso a={alpha} {} -> {} vertices (aresta {:.4} vs alvo de quad {target_edge:.4})",
            r.verts_before,
            r.verts_after,
            ph2d_remesh_iso::target_edge(&mesh, alpha)
        );
    }
    mesh.triangulate();
    let dual = Dual::build(&mesh);
    let (field, _) = solve_miq(&dual);
    eprintln!(
        "[f5] {name}: campo em {:.0} ms",
        t.elapsed().as_secs_f64() * 1000.0
    );
    let (sing, sum) = singularities(&mesh, &dual, &field);
    let layout = trace_patches(&mesh, &dual, &field);
    // ⚠️ **O relatório do traçado imprime-se SEMPRE, mesmo quando o F5 recusa.**
    // A primeira versão só o imprimia no caminho feliz, e as duas corridas que
    // falharam não disseram *nada* sobre a causa — nem quantas singularidades,
    // nem quantas separatrizes ficaram soltas.
    let tr = &layout.report;
    println!(
        "{name:<16} sing={sing}(soma {sum}) patches={:<4} sep={:<4} soltas={:<3} \
         nao-disco={:<3} dissolv={:<3} promov={:<4} arcos={:<5} aneis-abertos={} val={:?}",
        tr.patches,
        tr.separatrices,
        tr.dangling,
        tr.non_disk,
        tr.dissolved,
        tr.promoted,
        tr.arcs,
        tr.open_rings,
        tr.valence
    );
    eprintln!(
        "[f5] {name}: tracado em {:.0} ms",
        t.elapsed().as_secs_f64() * 1000.0
    );
    let Ok(l) = layout.to_layout(target_edge) else {
        println!("{name:<16} layout invalido");
        return;
    };
    // ⚠️ **Orçamento explícito, e a sonda é quem o escolhe.** O relógio desta
    // cadeia é do LAYOUT e não do tamanho da malha: um layout sujo faz a busca do
    // F4 passar de segundos a mais de vinte minutos.
    //
    // ⭐ **E "sujo" tem causa nomeada desde 2026-08-21** (PLAN §4-septies): era a
    // colisão de diagonal do flip. Com ela curada e o F1 na frente, a esfera
    // sacudida caiu de **> 20 min** para **1,6 s** e a de 98 k de 33 s para 8,5 s,
    // **sem o F4 mudar uma linha**. *O F4 nunca foi lento; recebia lixo.*
    let (q, qr) = match quantize_within(&l, Budget::new(256, 512)) {
        Ok(v) => v,
        // ⚠️ **O ERRO, e não "nao quantiza".** Orçamento esgotado e layout
        // inviável são coisas diferentes e pedem curas diferentes; a primeira
        // versão desta linha imprimia as duas igual.
        Err(e) => {
            println!("{name:<16}   -> o F4 recusou: {e:?}");
            return;
        }
    };
    match fill(&mesh, &mesh, &layout, &q, SMOOTHING_ROUNDS) {
        Ok((_out, r)) => {
            let pct = if r.verts == 0 {
                0.0
            } else {
                100.0 * r.irregular as f64 / r.verts as f64
            };
            println!(
                "{:<16}   -> QUADS {:<6} nao-quads {:<3} \
                 verts {:<6} IRREGULARES {:<5} ({pct:.1} %) bordo {:<3} invertidas {:<6} prova {}",
                "",
                r.quads,
                r.non_quads,
                r.verts,
                r.irregular,
                r.boundary_edges,
                r.flipped,
                if qr.proved { "sim" } else { "NAO" }
            );
            // ⭐ **DE ONDE vêm os irregulares** — é o que decide de que FASE é a
            // dívida, e sem ele `47` só diz que há trabalho.
            let breakdown: Vec<String> = ph2d_quadfill::Provenance::NAMES
                .iter()
                .zip(r.by_provenance)
                .filter(|(_, n)| *n > 0)
                .map(|(name, n)| format!("{name} {n}"))
                .collect();
            println!(
                "{:<16}   -> IRREGULARES POR ORIGEM: {}",
                "",
                breakdown.join(" · ")
            );
            eprintln!("[f5] {name}: {:.0} ms", t.elapsed().as_secs_f64() * 1000.0);
        }
        Err(e) => println!("{name:<16} a montagem recusou: {e:?}"),
    }
}

#[test]
#[ignore = "sonda -- imprime a cadeia inteira ate' a malha, nao afirma um limite"]
fn the_chain_returns_a_quad_mesh() {
    // ⭐ As MESMAS malhas do corpus da bancada, e o mesmo alvo de densidade —
    // é o que torna estas linhas comparáveis com o §1.3 do PLAN.
    run("sphere_uv_96x144", shapes::uv_sphere(96, 144, 1.0), 0.05);
    run("torus_64x32", shapes::torus(64, 32, 1.0, 0.35), 0.05);
    run("sphere_sculpt_98k", shapes::sculpt_sphere(1.0), 0.05);
    run(
        "sphere_shuffled",
        shapes::uv_sphere_shuffled(96, 144, 1.0),
        0.05,
    );
    run(
        "sphere_noisy",
        shapes::uv_sphere_noisy(96, 144, 1.0, 0.02),
        0.05,
    );
    run("esfera fina", shapes::uv_sphere(48, 72, 1.0), 0.03);
}

#[test]
#[ignore = "sonda -- a cadeia COMPLETA (F1 na frente), como o produto a vai correr"]
fn the_product_chain_starts_with_the_isotropic_pass() {
    // ⭐ O `cube` só existe nesta lista: sem o F1 ele tem 8 vértices e o pipeline
    // devolve malha vazia. *É a fixtura que prova que a saída deixou de depender
    // da entrada.*
    run_iso("cube", shapes::cube(1.0), 0.05);
    run_iso("sphere_uv_96x144", shapes::uv_sphere(96, 144, 1.0), 0.05);
    run_iso("torus_64x32", shapes::torus(64, 32, 1.0, 0.35), 0.05);
    run_iso("sphere_sculpt_98k", shapes::sculpt_sphere(1.0), 0.05);
    run_iso(
        "sphere_shuffled",
        shapes::uv_sphere_shuffled(96, 144, 1.0),
        0.05,
    );
    run_iso(
        "sphere_noisy",
        shapes::uv_sphere_noisy(96, 144, 1.0, 0.02),
        0.05,
    );
}

#[test]
#[ignore = "sonda -- de que grao de TRIANGULO a grade de quads precisa"]
fn how_fine_must_the_triangle_mesh_be() {
    // ⭐ **A pergunta que a cadeia completa levantou.** Com `ALPHA = 0,02` a
    // aresta de triângulo de uma esfera unitária sai a **0,069** — mais GROSSA
    // que o alvo de quad de `0,050`. O traçado não pode resolver uma fronteira
    // de patch mais fina que os próprios triângulos, e é aí que se suspeita que
    // os irregulares subiram de 21 para 47.
    // ⚠️ **A faixa é curta de propósito.** Uma primeira varredura até `0,005`
    // ficou **doze minutos** num único ponto sem imprimir nada: abaixo de `0,010`
    // a malha de triângulos passa dos 200 mil e o traçado deixa de ser o passo
    // barato. *A pergunta é se o grão importa, não onde ele satura.*
    for alpha in [0.02f32, 0.014, 0.010] {
        run_with_alpha(
            &format!("esfera a={alpha}"),
            shapes::uv_sphere(96, 144, 1.0),
            0.05,
            Some(alpha),
        );
    }
}
