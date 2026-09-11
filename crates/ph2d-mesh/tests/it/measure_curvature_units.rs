//! **De que UNIDADE é a curvatura que o Cavity publica?** — a sonda que decide
//! se o SSS pré-integrado pode reusar aquele canal.
//!
//! ```text
//! cargo test -p ph2d-mesh --release --test measure_curvature_units -- --nocapture
//! ```
//!
//! O [`05.1`](../../../docs/3D/05-Shading/05.1-Shader-de-runtime.md) §2a diz, sobre
//! o SSS por LUT de Penner: *"a **curvatura** já a temos: ela é calculada por
//! vértice para o Cavity (item 4). **Um dado, dois usos.**"*
//!
//! ⚠️ **Essa frase é uma AFIRMAÇÃO SOBRE UM NÚMERO, e o `CLAUDE.md` §0 manda medir
//! antes de a tomar como spec.** As duas grandezas têm dimensão diferente:
//!
//! | | o que é | dimensão |
//! |---|---|---|
//! | o canal do **Cavity** | `dot(centroide − p, n) / raio_médio` | **adimensional** (por construção — o módulo `curvature.rs` explica que a divisão existe para isso) |
//! | o eixo da **LUT de Penner** | `1/r`, o raio de curvatura em unidades de MUNDO | **1/comprimento** |
//!
//! A difusão sub-superficial tem uma **escala física** (milímetros de pele), então
//! `raio_de_espalhamento × curvatura` é o número adimensional que a LUT indexa —
//! e para formá-lo é preciso um `1/R` de verdade. Um canal invariante de escala
//! **não pode** produzi-lo: a mesma cabeça esculpida 10× maior espalharia igual,
//! quando ela deveria espalhar 10× menos em termos relativos.
//!
//! Esta sonda mede as duas colunas lado a lado numa esfera de raio conhecido, que
//! é a única forma cujo `1/R` **o gate pode afirmar por fora do codebase**.

use ph2d_mesh::{Mesh, shapes};

/// A curvatura de MUNDO, **pela porta do produto**.
///
/// ⚠️ A primeira versão desta sonda trazia a fórmula copiada aqui — e ela já
/// estava DESATUALIZADA em relação ao produto: usava o quadrado do raio médio,
/// que carrega um viés de Jensen de 4% num anel anisotrópico. A tabela impressa
/// teria contradito os números que shipam, e ninguém saberia qual acreditar.
fn world_curvature_at(m: &Mesh, v: usize) -> f32 {
    ph2d_mesh::world_curvature_at(m.positions(), m.normals(), &m.adjacency().vert_verts, v)
}

/// A mediana do interior (o equador de uma UV-sphere; os polos têm valência
/// diferente e não são representativos).
fn median_of(vals: impl Iterator<Item = f32>) -> f32 {
    let mut v: Vec<f32> = vals.collect();
    v.sort_by(f32::total_cmp);
    v[v.len() / 2]
}

fn sphere(rings: usize, segs: usize, radius: f32) -> Mesh {
    let mut m = shapes::uv_sphere(rings, segs, radius);
    m.triangulate();
    m
}

#[test]
#[ignore = "sonda de medição; roda sob demanda"]
fn measure_which_curvature_the_sss_needs() {
    println!("\n=== A ESCALA: a MESMA forma em três tamanhos (48x72) ===");
    println!(
        "{:>8} | {:>14} | {:>14} | {:>10}",
        "raio R", "curv (cavity)", "kappa (mundo)", "-1/R"
    );
    for r in [1.0f32, 2.0, 4.0] {
        let m = sphere(48, 72, r);
        let cav = median_of(m.curvatures().iter().copied());
        let world = median_of((0..m.vert_count()).map(|v| world_curvature_at(&m, v)));
        println!(
            "{r:>8.1} | {cav:>14.6} | {world:>14.6} | {:>10.6}",
            -1.0 / r
        );
    }

    println!("\n=== A TESSELACAO: o MESMO tamanho em tres densidades (R = 1) ===");
    println!(
        "{:>12} | {:>14} | {:>14} | {:>10}",
        "malha", "curv (cavity)", "kappa (mundo)", "-1/R"
    );
    for (rings, segs) in [(24usize, 36usize), (48, 72), (96, 144)] {
        let m = sphere(rings, segs, 1.0);
        let cav = median_of(m.curvatures().iter().copied());
        let world = median_of((0..m.vert_count()).map(|v| world_curvature_at(&m, v)));
        println!(
            "{rings:>5}x{segs:<6} | {cav:>14.6} | {world:>14.6} | {:>10.6}",
            -1.0f32
        );
    }

    println!(
        "\nLEITURA: se a coluna `curv` NAO se move com o raio, ela e' adimensional\n\
         e NAO serve de eixo para a LUT de Penner. Se `kappa` acompanha -1/R nas\n\
         duas tabelas, ela e' a grandeza que o SSS pede.\n"
    );
}
