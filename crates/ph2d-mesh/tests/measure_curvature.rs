//! **A sonda que decide os dois números da cavidade** — o GANHO do shader e o
//! preço dela num dab.
//!
//! ```text
//! cargo test -p ph2d-mesh --release --test measure_curvature -- --nocapture
//! ```
//!
//! Duas perguntas, e nenhuma delas se responde por raciocínio:
//!
//! 1. **Que faixa a curvatura de fato ocupa numa malha esculpida?** A aritmética
//!    diz `−h/(2R)` numa esfera, mas o que decide o ganho é o PERCENTIL — quanto
//!    da superfície precisa saturar para o canal *ler* sem virar carvão.
//! 2. **Quanto ela acrescenta a um dab?** Ela cavalga a lista que o
//!    `refresh_region` já construiu (88% do custo daquele passe é DESCOBRIR a
//!    vizinhança — W1/M3), então a hipótese é *quase nada*. Hipótese não é
//!    medição.

use std::time::Instant;

use ph2d_mesh::{Mesh, QueryScratch, RegionScratch, shapes};

fn median(mut v: Vec<f64>) -> f64 {
    if v.len() > 1 {
        v.remove(0);
    }
    v.sort_by(f64::total_cmp);
    v[v.len() / 2]
}

/// Uma esfera triangulada **esculpida**: alguns traços de Draw a empurram para
/// fora e para dentro, de raios diferentes.
///
/// ⚠️ A esfera CRUA não serve para a pergunta 1: ela tem uma curvatura só, e a
/// distribuição dela é um pico. O que decide o ganho é a mistura de superfície
/// lisa com vinco, que é o que uma escultura é.
fn sculpted(rings: usize, segs: usize) -> Mesh {
    let mut m = shapes::uv_sphere(rings, segs, 1.0);
    m.triangulate();
    let mut q = QueryScratch::default();
    let mut scratch = RegionScratch::default();
    let mut moved = Vec::new();
    // Sete toques em pontos espalhados, alternando sinal e raio — o que uma mão
    // faz nos primeiros segundos.
    for i in 0..7usize {
        let seed = (i * 7919) % m.vert_count();
        let center = m.positions()[seed];
        let radius = 0.10 + 0.06 * (i % 3) as f32;
        let push = if i % 2 == 0 { 0.06 } else { -0.05 };
        m.verts_in_sphere(center, radius, &mut q, &mut moved);
        let hits: Vec<u32> = moved.clone();
        for &v in &hits {
            let n = m.normals()[v as usize];
            let p = m.positions()[v as usize];
            let d = [p[0] - center[0], p[1] - center[1], p[2] - center[2]];
            let t = 1.0 - (d[0] * d[0] + d[1] * d[1] + d[2] * d[2]).sqrt() / radius;
            let w = (t.max(0.0)).powi(2);
            let q = &mut m.positions_mut()[v as usize];
            q[0] += n[0] * push * w;
            q[1] += n[1] * push * w;
            q[2] += n[2] * push * w;
        }
        m.refresh_region(&hits, &mut scratch);
    }
    m
}

/// **A DISTRIBUIÇÃO** — de onde o `CAVITY_GAIN` sai.
#[test]
fn measure_curvature_distribution() {
    for (label, mesh) in [
        ("esfera crua 48x72", {
            let mut m = shapes::uv_sphere(48, 72, 1.0);
            m.triangulate();
            m
        }),
        ("esculpida 48x72", sculpted(48, 72)),
        ("esculpida 96x144", sculpted(96, 144)),
    ] {
        let mut k: Vec<f32> = mesh.curvatures().to_vec();
        k.sort_by(f32::total_cmp);
        let at = |p: f64| k[((k.len() - 1) as f64 * p) as usize];
        let mut abs: Vec<f32> = k.iter().map(|x| x.abs()).collect();
        abs.sort_by(f32::total_cmp);
        let abs_at = |p: f64| abs[((abs.len() - 1) as f64 * p) as usize];
        println!(
            "{label:<18} n={:>7}  p01={:+.4} p10={:+.4} p50={:+.4} p90={:+.4} p99={:+.4} | \
             |k| p90={:.4} p99={:.4} max={:.4}  ⇒ ganho p/ saturar p99: {:.1}",
            k.len(),
            at(0.01),
            at(0.10),
            at(0.50),
            at(0.90),
            at(0.99),
            abs_at(0.90),
            abs_at(0.99),
            abs[abs.len() - 1],
            1.0 / abs_at(0.99).max(1e-6),
        );
    }
}

/// **O PREÇO** — quanto a curvatura acrescenta a um `refresh_region`.
///
/// O A/B é costas-com-costas dentro da MESMA corrida, sobre o mesmo estado, e
/// não duas execuções: esta máquina é compartilhada, e um A/B cross-run
/// atribuiria a carga das outras linhas ao ganho (a lição do doc 28 §5.46 da
/// `line/Painter`).
///
/// ⚠️ O braço "sem" não é uma versão sem curvatura do produto — ela não existe.
/// Ele é o custo do `curvature_of` sozinho, medido sobre a MESMA lista, e o que
/// se afirma é a razão dele para o passe inteiro.
#[test]
fn measure_curvature_cost_of_a_dab() {
    for (verts_label, rings, segs) in [("~100k", 220, 440), ("~1M", 700, 1400)] {
        let mut mesh = shapes::uv_sphere(rings, segs, 1.0);
        mesh.triangulate();
        let adj = ph2d_mesh::Adjacency::build(mesh.vert_count(), mesh.faces());
        let mut scratch = RegionScratch::default();
        let mut q = QueryScratch::default();

        for frac in [0.05f32, 0.15] {
            let radius = mesh.bounds().longest_edge() * frac;
            let center = mesh.positions()[mesh.vert_count() / 3];
            let mut moved = Vec::new();
            mesh.verts_in_sphere(center, radius, &mut q, &mut moved);

            let mut whole = Vec::new();
            let mut only_k = Vec::new();
            let mut out = Vec::new();
            for _ in 0..9 {
                let t0 = Instant::now();
                mesh.refresh_region(&moved, &mut scratch);
                whole.push(t0.elapsed().as_secs_f64() * 1e3);

                let refreshed = scratch.refreshed().to_vec();
                let t1 = Instant::now();
                ph2d_mesh::curvature_of(
                    mesh.positions(),
                    mesh.normals(),
                    &adj.vert_verts,
                    &refreshed,
                    &mut out,
                );
                only_k.push(t1.elapsed().as_secs_f64() * 1e3);
            }
            let w = median(whole);
            let k = median(only_k);
            println!(
                "{verts_label:<6} ({:>8} vertices) raio {frac:.2}: passe {w:.3} ms, \
                 curvatura {k:.3} ms = {:.1}% dele ({} vertices refrescados)",
                mesh.vert_count(),
                100.0 * k / w,
                scratch.refreshed().len(),
            );
        }
    }
}
