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

fn run(name: &str, mut mesh: Mesh, target_edge: f32) {
    eprintln!("[f5] {name}: {} vertices…", mesh.vert_count());
    let t = std::time::Instant::now();
    mesh.triangulate();
    let dual = Dual::build(&mesh);
    let (field, _) = solve_miq(&dual);
    let (sing, sum) = singularities(&mesh, &dual, &field);
    let layout = trace_patches(&mesh, &dual, &field);
    let Ok(l) = layout.to_layout(target_edge) else {
        println!("{name:<16} layout invalido");
        return;
    };
    // ⚠️ **Orçamento explícito, e a sonda é quem o escolhe.** Medido: a esfera
    // EMBARALHADA (mesma forma, ordem de índice trocada) produz um layout mais
    // sujo, e a busca do F4 com o orçamento cheio passa de segundos a **mais de
    // vinte minutos** — enquanto a esfera de 98 k vértices resolve em 41 s. *O
    // relógio desta cadeia é do layout, não do tamanho da malha.*
    let Ok((q, qr)) = quantize_within(&l, Budget::new(256, 512)) else {
        println!("{name:<16} nao quantiza");
        return;
    };
    match fill(&mesh, &layout, &q, SMOOTHING_ROUNDS) {
        Ok((_out, r)) => {
            let pct = if r.verts == 0 {
                0.0
            } else {
                100.0 * r.irregular as f64 / r.verts as f64
            };
            println!(
                "{name:<16} sing={sing}(soma {sum}) patches={:<4} | QUADS {:<6} nao-quads {:<3} \
                 verts {:<6} IRREGULARES {:<5} ({pct:.1} %) bordo {:<3} invertidas {:<6} prova {}",
                layout.side_arcs.len(),
                r.quads,
                r.non_quads,
                r.verts,
                r.irregular,
                r.boundary_edges,
                r.flipped,
                if qr.proved { "sim" } else { "NAO" }
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
