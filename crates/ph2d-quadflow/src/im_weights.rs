//! **OS PESOS DO LAPLACIANO E AS ÁREAS DUAIS** — porte de
//! `generate_adjacency_matrix_cotan` (`adjacency.cpp`) e
//! `compute_dual_vertex_areas` (`meshstats.cpp`), `instant-meshes`,
//! BSD-3-Clause.
//!
//! ⚠️ **A minha versão usava peso 1 em toda aresta, e isso não é um detalhe.** O
//! peso cotangente é o que faz a suavização respeitar a **geometria** em vez da
//! contagem de vizinhos: numa malha com triângulos esticados — o polo de uma
//! esfera UV, a beira de um vinco esculpido — dez vizinhos apertados de um lado
//! passam a valer o que valem, e não dez vezes o vizinho largo do outro lado.
//! Sem isso o campo vira ali, e cada vez que ele vira nasce um par de
//! singularidades.

use ph2d_mesh::Mesh;

/// Uma vizinhança com o peso do Laplaciano.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Link {
    /// O vizinho.
    pub id: u32,
    /// O peso `½(cot α + cot β)` da aresta.
    pub weight: f32,
}

/// **OS PESOS COTANGENTE** de cada aresta, por vértice.
///
/// Para a aresta `(a,b)`, `½·cot(ângulo oposto)` de **cada** triângulo que a usa
/// — que somado dá o `½(cot α + cot β)` da referência.
///
/// ⚠️ **O peso pode ser NEGATIVO**, num triângulo obtuso, e isso é correto: é o
/// Laplaciano cotangente, não uma média. A referência não o corta, e cortá-lo
/// mudaria o operador.
#[must_use]
pub fn cotangent_adjacency(mesh: &Mesh) -> Vec<Vec<Link>> {
    let n = mesh.vert_count();
    let p = mesh.positions();
    let mut weight: std::collections::BTreeMap<(u32, u32), f32> = std::collections::BTreeMap::new();
    for f in mesh.faces() {
        let v = f.verts();
        for k in 1..v.len() - 1 {
            let tri = [v[0], v[k], v[k + 1]];
            for e in 0..3usize {
                let (a, b, c) = (tri[e], tri[(e + 1) % 3], tri[(e + 2) % 3]);
                let d0 = sub(p[a as usize], p[c as usize]);
                let d1 = sub(p[b as usize], p[c as usize]);
                let sin_alpha = norm(cross(d0, d1));
                let cot = if sin_alpha > 1.0e-20 {
                    dot(d0, d1) / sin_alpha
                } else {
                    0.0
                };
                let key = if a < b { (a, b) } else { (b, a) };
                *weight.entry(key).or_insert(0.0) += 0.5 * cot;
            }
        }
    }
    let mut adj: Vec<Vec<Link>> = vec![Vec::new(); n];
    for ((a, b), w) in weight {
        adj[a as usize].push(Link { id: b, weight: w });
        adj[b as usize].push(Link { id: a, weight: w });
    }
    for list in &mut adj {
        list.sort_by_key(|l| l.id);
    }
    adj
}

/// **A ÁREA DUAL de cada vértice** — a célula baricêntrica.
///
/// ⚠️ **É exatamente um TERÇO da área de cada triângulo incidente**, e a
/// referência chega ao mesmo número pelo caminho longo (as duas meias-fatias
/// entre o centro da face e os pontos médios das duas arestas). Somam a mesma
/// coisa; o caminho curto é o que se pode conferir de cabeça.
#[must_use]
pub fn dual_vertex_areas(mesh: &Mesh) -> Vec<f32> {
    let mut a = vec![0.0f32; mesh.vert_count()];
    let p = mesh.positions();
    for f in mesh.faces() {
        let v = f.verts();
        for k in 1..v.len() - 1 {
            let tri = [v[0], v[k], v[k + 1]];
            let area = 0.5
                * norm(cross(
                    sub(p[tri[1] as usize], p[tri[0] as usize]),
                    sub(p[tri[2] as usize], p[tri[0] as usize]),
                ));
            for &t in &tri {
                a[t as usize] += area / 3.0;
            }
        }
    }
    a
}

fn dot(a: [f32; 3], b: [f32; 3]) -> f32 {
    a[0].mul_add(b[0], a[1].mul_add(b[1], a[2] * b[2]))
}

fn cross(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [
        a[1].mul_add(b[2], -(a[2] * b[1])),
        a[2].mul_add(b[0], -(a[0] * b[2])),
        a[0].mul_add(b[1], -(a[1] * b[0])),
    ]
}

fn sub(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}

fn norm(a: [f32; 3]) -> f32 {
    dot(a, a).sqrt()
}
