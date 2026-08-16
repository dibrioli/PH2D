//! **O QUE O LAPLACIANO POR COTANGENTES COMPRA NO SMOOTH** — a sonda que decide
//! onde o operador de Meyer 2003 vive, depois de a normal do Inflate ter sido
//! recusada.
//!
//! ```text
//! cargo test -p ph2d-mesh --release --test measure_cotangent_smoothing -- --ignored --nocapture
//! ```
//!
//! O §4 do plano 21 diz que Meyer 2003 dá *"laplaciano por **cotangentes**,
//! normal de curvatura média"* para **"Inflate · o operador dos dois acima"** — e
//! os dois acima são Taubin 1995 (λ|μ) e Desbrun 1999 (fairing implícito). Ou
//! seja: o cotangente é o **operador sobre o qual o par λ|μ deveria correr**, e
//! o nosso corre sobre o UNIFORME.
//!
//! # A propriedade que o paper reivindica, e que esta sonda mede
//!
//! O laplaciano **uniforme** aponta para o centroide do anel, e num anel
//! ANISOTRÓPICO esse centroide tem uma componente **TANGENCIAL** grande: os
//! vértices deslizam pela superfície, a parametrização se distorce e o detalhe
//! do artista escorre de lado. O operador por cotangentes é (a primeira ordem)
//! **puramente normal** — é essa a razão de ele existir.
//!
//! ⚠️ **A fixture tem de ter anel anisotrópico.** Numa esfera de anéis quadrados
//! os dois coincidem por simetria e a sonda mediria zero. A `uv_sphere` serve
//! **por acidente da própria construção**: perto dos polos os quads são fatias
//! finas (o passo em longitude encolhe com `sin θ` e o em latitude não), e a
//! anisotropia local chega a uma ordem de grandeza sem ninguém a fabricar.
//!
//! # ⚠️ O que a sonda NÃO pode responder
//!
//! Ela mede a MALHA. Se o desenho fica melhor é o smoke que decide — e é por
//! isso que a tabela imprime as três colunas em vez de eleger uma.

use ph2d_mesh::{Mesh, shapes};

/// Um passe de suavização com o alvo que a closure escolher, força cheia.
///
/// ⚠️ **Jacobi, não Gauss-Seidel:** todo alvo é lido do estado do INÍCIO do
/// passe. Ler posições já escritas faria o resultado depender da ordem de
/// varredura, e as duas colunas deixariam de ser comparáveis entre si.
fn pass(m: &mut Mesh, target: impl Fn(&Mesh, usize) -> Option<[f32; 3]>) {
    let src: Vec<[f32; 3]> = m.positions().to_vec();
    let mut out = src.clone();
    for (v, o) in out.iter_mut().enumerate() {
        if let Some(t) = target(m, v) {
            *o = t;
        }
    }
    m.positions_mut().copy_from_slice(&out);
    let all: Vec<u32> = (0..out.len() as u32).collect();
    m.refresh_region(&all, &mut ph2d_mesh::RegionScratch::default());
}

fn uniform_target(m: &Mesh, v: usize) -> Option<[f32; 3]> {
    let p = m.positions()[v];
    let t = ph2d_mesh::ring_average(m.adjacency(), v as u32, p, |nb| m.positions()[nb as usize]);
    Some(t)
}

fn cotangent_target(m: &Mesh, v: usize) -> Option<[f32; 3]> {
    ph2d_mesh::cotangent_ring_average_at(m.positions(), m.faces(), m.adjacency(), v)
}

fn len(v: [f32; 3]) -> f32 {
    (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt()
}

/// O deslocamento TANGENCIAL acumulado — a componente do movimento total que é
/// perpendicular à normal ORIGINAL. É o que "distorcer a parametrização"
/// significa em número.
fn drift(before: &[[f32; 3]], nrm: &[[f32; 3]], after: &[[f32; 3]]) -> (f32, f32) {
    let mut sum = 0.0f32;
    let mut worst = 0.0f32;
    for i in 0..before.len() {
        let d = [
            after[i][0] - before[i][0],
            after[i][1] - before[i][1],
            after[i][2] - before[i][2],
        ];
        let n = nrm[i];
        let along = d[0] * n[0] + d[1] * n[1] + d[2] * n[2];
        let tang = [
            d[0] - along * n[0],
            d[1] - along * n[1],
            d[2] - along * n[2],
        ];
        let t = len(tang);
        sum += t;
        worst = worst.max(t);
    }
    #[allow(clippy::cast_precision_loss)] // LITERAL-PX-OK: contagem de vértices.
    (sum / before.len() as f32, worst)
}

fn mean_radius(m: &Mesh) -> f32 {
    #[allow(clippy::cast_precision_loss)] // LITERAL-PX-OK: contagem de vértices.
    let n = m.positions().len() as f32;
    m.positions().iter().map(|p| len(*p)).sum::<f32>() / n
}

/// **A TABELA.** N passes de cada operador na MESMA malha, lado a lado.
#[test]
#[ignore = "sonda: imprime, não afirma"]
fn measure_what_the_cotangent_weighting_buys() {
    for passes in [1usize, 4, 16] {
        println!("\n=== {passes} passe(s) de suavizacao, forca cheia ===");
        println!(
            "{:<12} {:>10} {:>12} {:>12} {:>10}",
            "operador", "raio medio", "deriva tang", "pior tang", "recusas"
        );
        for (name, cot) in [("uniforme", false), ("cotangente", true)] {
            let mut m = shapes::uv_sphere(24, 32, 1.0);
            let before = m.positions().to_vec();
            let nrm = m.normals().to_vec();
            let mut refused = 0usize;
            for _ in 0..passes {
                if cot {
                    pass(&mut m, cotangent_target);
                } else {
                    pass(&mut m, uniform_target);
                }
            }
            if cot {
                refused = (0..m.positions().len())
                    .filter(|&v| cotangent_target(&m, v).is_none())
                    .count();
            }
            let (mean_t, worst_t) = drift(&before, &nrm, m.positions());
            println!(
                "{name:<12} {:>10.6} {:>12.6} {:>12.6} {:>10}",
                mean_radius(&m),
                mean_t,
                worst_t,
                refused
            );
        }
    }
    println!();
}

/// **E NUMA MALHA QUE UM SOLVER DE VERDADE PRODUZ** — irregular de propósito.
///
/// ⚠️ **É aqui que a guarda do `Σw` deixa de ser teórica:** uma malha embaralhada
/// tem triângulos obtusos, e uma cotangente negativa é o que faz um alvo saltar
/// para fora da superfície.
#[test]
#[ignore = "sonda: imprime, não afirma"]
fn measure_the_same_thing_on_an_irregular_mesh() {
    println!("\n=== malha IRREGULAR (uv_sphere_shuffled) ===");
    println!(
        "{:<12} {:>10} {:>12} {:>12} {:>10}",
        "operador", "raio medio", "deriva tang", "pior tang", "recusas"
    );
    for (name, cot) in [("uniforme", false), ("cotangente", true)] {
        let mut m = shapes::uv_sphere_shuffled(24, 32, 1.0);
        let before = m.positions().to_vec();
        let nrm = m.normals().to_vec();
        let refused_before = (0..m.positions().len())
            .filter(|&v| cotangent_target(&m, v).is_none())
            .count();
        for _ in 0..4 {
            if cot {
                pass(&mut m, cotangent_target);
            } else {
                pass(&mut m, uniform_target);
            }
        }
        let (mean_t, worst_t) = drift(&before, &nrm, m.positions());
        println!(
            "{name:<12} {:>10.6} {:>12.6} {:>12.6} {:>10}",
            mean_radius(&m),
            mean_t,
            worst_t,
            if cot { refused_before } else { 0 }
        );
    }
    println!();
}
