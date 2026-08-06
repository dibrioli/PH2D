//! **A SONDA DO ALPHA** — a cobertura de cada padrão, e a relação entre o
//! tamanho de uma feature e a aresta da malha.
//!
//! Rodar: `cargo test -p ph2d-sculpt3d --release --test measure_alpha -- --ignored --nocapture`
//!
//! ⚠️ **A segunda tabela é a que decide um número de produto.** Um padrão é uma
//! função contínua e a malha o amostra nos VÉRTICES: se a célula do padrão for
//! da ordem da aresta, nenhum vértice cai no miolo de uma feature e o que chega
//! ao barro é ruído por vértice — não uma textura. É o Nyquist de sempre, e ele
//! decide o default da pista de escala e o que o smoke tem de dizer ao artista.

use ph2d_mesh::{Mesh, shapes};
use ph2d_sculpt3d::Alpha;

/// A aresta MEDIANA de uma malha, em unidades de objeto.
fn median_edge(mesh: &Mesh) -> f32 {
    let pos = mesh.positions();
    let ring = mesh.adjacency();
    let mut lens = Vec::new();
    for v in 0..pos.len() {
        for &n in ring.vert_verts.neighbours(v) {
            if n as usize > v {
                let (a, b) = (pos[v], pos[n as usize]);
                let d = [b[0] - a[0], b[1] - a[1], b[2] - a[2]];
                lens.push(d[2].mul_add(d[2], d[0].mul_add(d[0], d[1] * d[1])).sqrt());
            }
        }
    }
    lens.sort_by(f32::total_cmp);
    lens[lens.len() / 2]
}

/// Uma nuvem determinística de amostras contínuas.
fn cloud(n: usize) -> Vec<[f32; 3]> {
    (0..n)
        .map(|i| {
            let t = i as f32;
            [
                t * 0.073_137_1 - 3.0,
                t * 0.041_231_5 + 1.7,
                t * 0.098_765_4 - 0.9,
            ]
        })
        .collect()
}

#[test]
#[ignore = "sonda"]
fn measure_alpha_coverage() {
    let pts = cloud(200_000);
    println!("\n== COBERTURA (a força aparente que o pincel guarda ao armar o alpha) ==");
    println!("{:<8} {:>8} {:>8} {:>8}", "padrão", "média", ">0,9", "<0,1");
    for a in Alpha::ALL {
        let mut sum = 0.0f64;
        let (mut hi, mut lo) = (0usize, 0usize);
        for &p in &pts {
            let w = a.weight_at(p, 0.25);
            sum += f64::from(w);
            if w > 0.9 {
                hi += 1;
            }
            if w < 0.1 {
                lo += 1;
            }
        }
        let n = pts.len() as f64;
        println!(
            "{:<8} {:>8.3} {:>7.1}% {:>7.1}%",
            a.label(),
            sum / n,
            hi as f64 / n * 100.0,
            lo as f64 / n * 100.0
        );
    }
}

/// **O PADRÃO ESTÁ RESOLVIDO PELA MALHA?** — a tabela que decide a faixa da
/// pista de escala.
///
/// O oráculo é a **correlação entre vértices VIZINHOS**. Um padrão resolvido
/// varia devagar comparado à aresta, então dois vértices ligados vêem
/// praticamente o mesmo valor (correlação → 1); um padrão mais fino que a malha
/// dá a cada vértice um valor independente do vizinho (correlação → 0) e o que
/// chega ao barro é **chuvisco por vértice, não textura**. É o Nyquist de
/// sempre, medido na régua que a malha de fato tem.
///
/// ⚠️ **A primeira versão desta sonda media outra coisa** — a fração de vértices
/// com valor "decidido" (perto de 0 ou de 1) — e dava ~95% em TODA escala,
/// porque o Pores é esparso: quase todo vértice está perto de zero por estar
/// fora de um poro, resolvido ou não. Um oráculo que não distingue *fora do
/// padrão* de *aliasing do padrão* não podia responder à pergunta.
#[test]
#[ignore = "sonda"]
fn measure_alpha_against_mesh_resolution() {
    println!("\n== O PADRÃO CONTRA A MALHA (Pores, esfera unitária) ==");
    println!(
        "{:<18} {:>7} {:>8} {:>7} {:>10} {:>10}",
        "malha", "verts", "aresta", "escala", "célula/ar", "correl.viz"
    );
    for (u, v) in [(32, 48), (64, 96), (128, 192), (160, 240), (192, 288)] {
        let mesh = shapes::uv_sphere(u, v, 1.0);
        let edge = median_edge(&mesh);
        for scale in [0.03_f32, 0.06, 0.12, 0.25, 0.5, 1.0] {
            let (mut a, mut b) = (Vec::new(), Vec::new());
            let ring = mesh.adjacency();
            for i in 0..mesh.vert_count() {
                for &n in ring.vert_verts.neighbours(i) {
                    if n as usize > i {
                        a.push(Alpha::Pores.weight_at(mesh.positions()[i], scale));
                        b.push(Alpha::Pores.weight_at(mesh.positions()[n as usize], scale));
                    }
                }
            }
            println!(
                "{:<18} {:>7} {:>8.4} {:>7.2} {:>10.1} {:>10.2}",
                format!("uv_sphere({u},{v})"),
                mesh.vert_count(),
                edge,
                scale,
                scale / edge,
                correlation(&a, &b)
            );
        }
    }
}

/// Correlação de Pearson.
fn correlation(a: &[f32], b: &[f32]) -> f32 {
    let n = a.len() as f32;
    let (ma, mb) = (a.iter().sum::<f32>() / n, b.iter().sum::<f32>() / n);
    let (mut cov, mut va, mut vb) = (0.0f32, 0.0f32, 0.0f32);
    for (&x, &y) in a.iter().zip(b) {
        let (dx, dy) = (x - ma, y - mb);
        cov += dx * dy;
        va += dx * dx;
        vb += dy * dy;
    }
    let d = (va * vb).sqrt();
    if d <= 0.0 { 0.0 } else { cov / d }
}

/// O custo de avaliar o alpha, contra o de não avaliar.
#[test]
#[ignore = "sonda"]
fn measure_alpha_cost() {
    use std::time::Instant;
    let pts = cloud(1_000_000);
    println!("\n== CUSTO (1 M avaliações) ==");
    for a in Alpha::ALL {
        let t = Instant::now();
        let mut acc = 0.0f32;
        for &p in &pts {
            acc += a.weight_at(p, 0.25);
        }
        let ms = t.elapsed().as_secs_f64() * 1000.0;
        // São exatamente 1 M avaliações, então `ms` e `ns/avaliação` são o MESMO
        // número — a primeira versão desta linha multiplicava por mil e
        // reportava 85 420 ns por avaliação de um ruído de valor.
        println!(
            "{:<8} {:>8.2} ms  ({:>5.1} ns/avaliação)  [soma {acc:.0}]",
            a.label(),
            ms,
            ms
        );
    }
}
