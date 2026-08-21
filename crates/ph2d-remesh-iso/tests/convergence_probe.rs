//! **SONDA — o passe CONVERGE, ou sai no teto de rodadas?**
//!
//! ```text
//! cd .../Worktrees/line-sculpt3d && cargo test -p ph2d-remesh-iso --release \
//!     --test convergence_probe -- --ignored --nocapture
//! ```
//!
//! ⭐ **A pergunta que o censo de singularidades levantou.** A mesma esfera
//! remalhada a `α = 0,020` dá **8** singularidades e a `α = 0,010` dá **194** —
//! enquanto a esfera ESTRUTURADA dá 7 em qualquer resolução. Se a culpa fosse do
//! solver do campo, a estruturada fina também partiria. ⇒ *A hipótese é que o
//! passe do F1 sai pelo TETO DE RODADAS sem convergir, e entrega uma malha com
//! arestas de tamanhos muito diferentes.*
//!
//! O `Report::rounds` já diz quantas rodadas correram; o que falta é a
//! **dispersão** das arestas, que é o que a régua `mean_edge` do gate esconde.

use ph2d_mesh::{Mesh, shapes};
use ph2d_remesh_iso::{MAX_ROUNDS, remesh_isotropic, target_edge};

/// Média, desvio-padrão relativo e os extremos das arestas.
fn edge_stats(mesh: &Mesh) -> (f32, f32, f32, f32) {
    let pos = mesh.positions();
    let mut lens: Vec<f32> = Vec::new();
    for f in mesh.faces() {
        let v = f.verts();
        for k in 0..v.len() {
            let (a, b) = (pos[v[k] as usize], pos[v[(k + 1) % v.len()] as usize]);
            let d = [b[0] - a[0], b[1] - a[1], b[2] - a[2]];
            lens.push(d[0].mul_add(d[0], d[1].mul_add(d[1], d[2] * d[2])).sqrt());
        }
    }
    #[allow(clippy::cast_precision_loss)]
    let n = lens.len() as f32;
    let mean = lens.iter().sum::<f32>() / n;
    let var = lens.iter().map(|l| (l - mean) * (l - mean)).sum::<f32>() / n;
    let lo = lens.iter().copied().fold(f32::INFINITY, f32::min);
    let hi = lens.iter().copied().fold(0.0f32, f32::max);
    (mean, var.sqrt() / mean, lo, hi)
}

#[test]
#[ignore = "sonda -- convergencia do passe contra o alvo pedido"]
fn does_the_pass_converge_at_every_target() {
    println!(
        "{:<12} {:<8} {:<8} {:<9} {:<9} {:<9} {:<9} {:<9}",
        "alpha", "verts", "rodadas", "alvo", "media", "med/alvo", "disp", "max/min"
    );
    for alpha in [0.020f32, 0.017, 0.014, 0.012, 0.010, 0.008] {
        let mut m = shapes::uv_sphere(96, 144, 1.0);
        let want = target_edge(&m, alpha);
        let r = remesh_isotropic(&mut m, alpha);
        let (mean, disp, lo, hi) = edge_stats(&m);
        println!(
            "{alpha:<12} {:<8} {:<8} {want:<9.4} {mean:<9.4} {:<9.3} {disp:<9.3} {:<9.1}{}",
            r.verts_after,
            r.rounds,
            mean / want,
            hi / lo,
            if r.rounds >= MAX_ROUNDS {
                "  ⛔ BATEU NO TETO"
            } else {
                ""
            }
        );
    }
}
