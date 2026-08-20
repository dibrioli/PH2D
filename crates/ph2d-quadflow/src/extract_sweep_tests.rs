//! **A SONDA DA FAIXA** — a qualidade da saída ao longo do slider inteiro.
//!
//! ⚠️ **Os gates da [`super`] medem UM tamanho de quad (`0,18`), e a queixa do
//! artista foi sobre a FAIXA:** *"valores baixos de resolution destroem o
//! objeto"* (Enio, 2026-08-19, foto). Um gate que mede um ponto de um slider
//! não fala sobre o slider — e a faixa é o produto, não o ponto.
//!
//! Esta sonda percorre a faixa que o painel oferece (`0,02 … 1,00`) e imprime,
//! por degrau, as grandezas que separam *uma malha* de *um monte de polígonos*:
//! as arestas de BORDA (buracos), as componentes (pedaços soltos), o maior ciclo
//! (os leques), a característica de Euler e o volume com sinal.

use std::collections::BTreeMap;

use ph2d_mesh::Mesh;

use super::sphere;
use crate::extract::extract;
use crate::scale::ScaleField;

/// O que uma corrida diz sobre a saída.
struct Quality {
    verts: usize,
    faces: usize,
    quads_pct: f64,
    chi: i64,
    boundary: usize,
    non_manifold: usize,
    flipped: usize,
    components: usize,
    max_sides: usize,
    volume: f64,
}

fn measure(q: &crate::extract::Quadrangulation) -> Quality {
    let mesh = &q.mesh;
    let mut undirected: BTreeMap<(u32, u32), usize> = BTreeMap::new();
    let mut directed: BTreeMap<(u32, u32), usize> = BTreeMap::new();
    for f in mesh.faces() {
        let v = f.verts();
        for i in 0..v.len() {
            let (a, b) = (v[i], v[(i + 1) % v.len()]);
            *directed.entry((a, b)).or_insert(0) += 1;
            let key = if a < b { (a, b) } else { (b, a) };
            *undirected.entry(key).or_insert(0) += 1;
        }
    }

    // As componentes conexas do grafo de arestas.
    let n = mesh.vert_count();
    let mut adj: Vec<Vec<u32>> = vec![Vec::new(); n];
    for (a, b) in undirected.keys() {
        adj[*a as usize].push(*b);
        adj[*b as usize].push(*a);
    }
    let mut comp = vec![false; n];
    let mut components = 0usize;
    for s in 0..n {
        if comp[s] {
            continue;
        }
        components += 1;
        let mut queue = vec![s];
        comp[s] = true;
        let mut head = 0;
        while head < queue.len() {
            let v = queue[head];
            head += 1;
            for &w in &adj[v] {
                if !comp[w as usize] {
                    comp[w as usize] = true;
                    queue.push(w as usize);
                }
            }
        }
    }

    let pos = mesh.positions();
    let mut volume = 0.0f64;
    for f in mesh.faces() {
        let v = f.verts();
        for k in 1..v.len() - 1 {
            let (a, b, c) = (
                pos[v[0] as usize],
                pos[v[k] as usize],
                pos[v[k + 1] as usize],
            );
            volume += f64::from(a[0].mul_add(
                b[1].mul_add(c[2], -(b[2] * c[1])),
                a[1].mul_add(
                    b[2].mul_add(c[0], -(b[0] * c[2])),
                    a[2] * b[0].mul_add(c[1], -(b[1] * c[0])),
                ),
            )) / 6.0;
        }
    }

    Quality {
        verts: n,
        faces: mesh.faces().len(),
        quads_pct: q.quad_fraction() * 100.0,
        chi: n as i64 - undirected.len() as i64 + mesh.faces().len() as i64,
        boundary: undirected.values().filter(|c| **c == 1).count(),
        non_manifold: undirected.values().filter(|c| **c > 2).count(),
        flipped: directed.values().filter(|c| **c > 1).count(),
        components,
        max_sides: q.max_sides,
        volume,
    }
}

fn header(name: &str, mesh: &Mesh) {
    eprintln!(
        "\n[quadflow] === {name} ({} vertices, {} faces) ===",
        mesh.vert_count(),
        mesh.faces().len()
    );
    eprintln!(
        "[quadflow]  edge |  saida |  faces | quads% |  chi | borda | n-manif | invert | comp | maior | volume"
    );
}

fn row(edge: f32, q: &Quality) {
    eprintln!(
        "[quadflow]  {edge:.2} | {:6} | {:6} | {:5.1}% | {:4} | {:5} | {:7} | {:6} | {:4} | {:5} | {:+.4}",
        q.verts,
        q.faces,
        q.quads_pct,
        q.chi,
        q.boundary,
        q.non_manifold,
        q.flipped,
        q.components,
        q.max_sides,
        q.volume
    );
}

/// **A FAIXA INTEIRA DO SLIDER**, sobre a esfera das fixtures.
///
/// ⚠️ `#[ignore]`: é medição, e imprime uma tabela.
#[test]
#[ignore = "sonda -- a qualidade ao longo do slider (CLAUDE.md §0.0)"]
fn measure_the_quality_across_the_slider() {
    let mesh = sphere();
    header("esfera 48x64", &mesh);
    for edge in [0.02f32, 0.05, 0.08, 0.12, 0.18, 0.25, 0.35, 0.5, 0.7, 1.0] {
        let scale = ScaleField::uniform(&mesh, edge);
        let (o, p) = crate::solve::solve_fields(&mesh, &scale);
        let q = extract(&mesh, &o, &p, &scale).expect("extraiu");
        row(edge, &measure(&q));
    }
}

/// **A MESMA FAIXA sobre a malha que o ARTISTA vê** — a `=35` abre uma
/// `uv_sphere(96,144)` amassada, e o gate de fixture não tem o que o smoke tem.
#[test]
#[ignore = "sonda -- a faixa sobre a malha do produto (CLAUDE.md §0.0)"]
fn measure_the_quality_on_the_product_mesh() {
    let mesh = ph2d_mesh::shapes::uv_sphere(96, 144, 1.0);
    header("uv_sphere 96x144 (a malha da cena =35)", &mesh);
    for edge in [0.02f32, 0.05, 0.08, 0.12, 0.18, 0.25, 0.35, 0.5, 0.7, 1.0] {
        let scale = ScaleField::uniform(&mesh, edge);
        let (o, p) = crate::solve::solve_fields(&mesh, &scale);
        let q = extract(&mesh, &o, &p, &scale).expect("extraiu");
        row(edge, &measure(&q));
    }
}

/// **A ESFERA AMASSADA** — sete sulcos paralelos, como os da cena `=35`.
///
/// ⚠️ **A cena abre AMASSADA de propósito**, e uma sonda sobre a esfera lisa não
/// tem o que ela tem: os sulcos são onde a curvatura aperta, e é lá que o campo
/// cruzado decide. Aqui os sulcos são analíticos (uma calha gaussiana ao longo
/// de meridianos vizinhos) — o brush de verdade vive noutra crate, e o que esta
/// sonda precisa é do FENÔMENO, não do gesto.
fn wrinkled() -> Mesh {
    let mut mesh = ph2d_mesh::shapes::uv_sphere(96, 144, 1.0);
    let normals: Vec<[f32; 3]> = mesh.normals().to_vec();
    let grooves: Vec<(f32, f32)> = (0..7)
        .map(|k| {
            (
                -0.45 + 0.15 * k as f32,
                0.20 * (0.32f32).powf(k as f32 / 6.0),
            )
        })
        .collect();
    for (i, p) in mesh.positions_mut().iter_mut().enumerate() {
        if p[2] <= 0.0 {
            continue;
        }
        let mut depth = 0.0f32;
        for &(v, amp) in &grooves {
            let d = (p[1] - v) / 0.10;
            depth += amp * (-d * d).exp();
        }
        let n = normals[i];
        *p = [
            depth.mul_add(-n[0], p[0]),
            depth.mul_add(-n[1], p[1]),
            depth.mul_add(-n[2], p[2]),
        ];
    }
    mesh.rebuild();
    mesh
}

/// **A FAIXA sobre a malha AMASSADA** — a que o smoke do Enio de facto retopologiza.
#[test]
#[ignore = "sonda -- a faixa sobre a malha amassada da =35 (CLAUDE.md §0.0)"]
fn measure_the_quality_on_the_wrinkled_mesh() {
    let mesh = wrinkled();
    header("uv_sphere 96x144 AMASSADA (a cena =35 de facto)", &mesh);
    for edge in [0.02f32, 0.05, 0.08, 0.12, 0.18, 0.25, 0.35, 0.5, 0.7, 1.0] {
        let scale = ScaleField::uniform(&mesh, edge);
        let (o, p) = crate::solve::solve_fields(&mesh, &scale);
        let q = extract(&mesh, &o, &p, &scale).expect("extraiu");
        row(edge, &measure(&q));
    }
}

/// **ONDE O GRAFO PERDE ARESTA** — a valência das células antes e depois da poda.
///
/// ⚠️ **Um nó de grade tem QUATRO vizinhas.** Toda célula com menos que isso é um
/// buraco no grafo, e o passeio de faces contorna o buraco — é dali que saem os
/// ciclos de 44 lados, e um ciclo de 44 lados vira **42 triângulos em leque**,
/// que é exatamente o objeto espetado da foto do Enio (2026-08-19).
#[test]
#[ignore = "sonda -- a valencia do grafo de celulas (CLAUDE.md §0.0)"]
fn measure_the_cell_valence() {
    let mesh = wrinkled();
    eprintln!("\n[quadflow] === valencia das celulas (esfera amassada) ===");
    eprintln!(
        "[quadflow]  edge | celulas |  grau0 |  grau1 |  grau2 |  grau3 |  grau4 |  grau5+ | podadas"
    );
    for edge in [0.05f32, 0.08, 0.12, 0.18, 0.25] {
        let scale = ScaleField::uniform(&mesh, edge);
        let (orient, pos) = crate::solve::solve_fields(&mesh, &scale);
        let cells = crate::extract::cluster_lattice(&mesh, &orient, &pos, &scale);
        let c = crate::extract::collapse(&mesh, &pos, &orient, &cells);
        let mut graph = crate::extract::graph::neighbour_graph(&mesh, &c, &scale);
        let mut hist = [0usize; 6];
        for g in &graph {
            hist[g.len().min(5)] += 1;
        }
        let before = graph.iter().filter(|g| !g.is_empty()).count();
        crate::extract::faces::prune_dangling(&mut graph);
        let after = graph.iter().filter(|g| !g.is_empty()).count();
        eprintln!(
            "[quadflow]  {edge:.2} | {:7} | {:6} | {:6} | {:6} | {:6} | {:6} | {:7} | {:7}",
            c.verts.len(),
            hist[0],
            hist[1],
            hist[2],
            hist[3],
            hist[4],
            hist[5],
            before - after
        );
    }
}

/// **AS DUAS LEIS DE LIGAÇÃO, LADO A LADO** — o cone geométrico contra o passo
/// da retícula.
///
/// ⚠️ **É a medição que escolhe o default do [`crate::extract::Linking`]**, e
/// ela roda sobre as três fixturas porque uma lei que ganha numa esfera lisa e
/// perde numa amassada não é uma lei.
#[test]
#[ignore = "sonda -- cone contra passo de reticula (CLAUDE.md §0.0)"]
fn measure_the_two_linking_laws() {
    use crate::extract::{Clustering, Linking, extract_tuned};
    for (name, mesh) in [
        ("esfera 48x64", sphere()),
        ("uv 96x144", ph2d_mesh::shapes::uv_sphere(96, 144, 1.0)),
        ("uv 96x144 amassada", wrinkled()),
    ] {
        eprintln!("\n[quadflow] === {name} ===");
        eprintln!(
            "[quadflow]  edge |     lei |  saida |  faces | quads% |  chi | borda | n-manif | invert | comp | maior | volume"
        );
        for edge in [0.05f32, 0.08, 0.12, 0.18, 0.25, 0.35] {
            let scale = ScaleField::uniform(&mesh, edge);
            let (o, p) = crate::solve::solve_fields(&mesh, &scale);
            for (tag, link) in [("cone", Linking::Cone), ("passo", Linking::LatticeStep)] {
                let q = extract_tuned(&mesh, &o, &p, &scale, Clustering::Lattice, link)
                    .expect("extraiu");
                let m = measure(&q);
                eprintln!(
                    "[quadflow]  {edge:.2} | {tag:>7} | {:6} | {:6} | {:5.1}% | {:4} | {:5} | {:7} | {:6} | {:4} | {:5} | {:+.4}",
                    m.verts,
                    m.faces,
                    m.quads_pct,
                    m.chi,
                    m.boundary,
                    m.non_manifold,
                    m.flipped,
                    m.components,
                    m.max_sides,
                    m.volume
                );
            }
        }
    }
}

/// A aresta MÉDIA da entrada — a régua que diz o que a malha é capaz de resolver.
fn mean_edge(mesh: &Mesh) -> f32 {
    let mut sum = 0.0f64;
    let mut count = 0usize;
    let p = mesh.positions();
    for f in mesh.faces() {
        let v = f.verts();
        for i in 0..v.len() {
            let (a, b) = (p[v[i] as usize], p[v[(i + 1) % v.len()] as usize]);
            let d = [a[0] - b[0], a[1] - b[1], a[2] - b[2]];
            sum += f64::from(d[0].mul_add(d[0], d[1].mul_add(d[1], d[2] * d[2])).sqrt());
            count += 1;
        }
    }
    (sum / count.max(1) as f64) as f32
}

/// **ONDE A QUALIDADE DESABA, em unidades da ARESTA DE ENTRADA.**
///
/// ⚠️ **A hipótese que esta sonda testa:** um remesh não pode resolver uma grade
/// mais fina do que a malha que ele lê. Abaixo de um certo múltiplo da aresta de
/// entrada a célula contém um vértice ou dois, o campo não tem o que quantizar, e
/// a extração devolve buracos — que o passeio contorna em ciclos gigantes, que
/// viram LEQUES de triângulos. É o objeto espetado da foto.
#[test]
#[ignore = "sonda -- o piso do slider em unidades da aresta de entrada (CLAUDE.md §0.0)"]
fn measure_where_quality_collapses() {
    for (name, mesh) in [
        ("esfera 48x64", sphere()),
        ("uv 96x144", ph2d_mesh::shapes::uv_sphere(96, 144, 1.0)),
        ("uv 96x144 amassada", wrinkled()),
    ] {
        let e = mean_edge(&mesh);
        eprintln!("\n[quadflow] === {name}: aresta media da entrada {e:.4} ===");
        eprintln!("[quadflow]  razao |   edge |  saida |  faces | quads% |  chi | maior | volume");
        for ratio in [1.0f32, 1.25, 1.5, 1.75, 2.0, 2.5, 3.0, 4.0, 6.0, 8.0] {
            let edge = e * ratio;
            let scale = ScaleField::uniform(&mesh, edge);
            let (o, p) = crate::solve::solve_fields(&mesh, &scale);
            let q = extract(&mesh, &o, &p, &scale).expect("extraiu");
            let m = measure(&q);
            eprintln!(
                "[quadflow]  {ratio:5.2}x | {edge:.4} | {:6} | {:6} | {:5.1}% | {:4} | {:5} | {:+.4}",
                m.verts, m.faces, m.quads_pct, m.chi, m.max_sides, m.volume
            );
        }
    }
}

/// **A HIERARQUIA AINDA PAGA?** — o caminho plano contra o hierárquico, em TODAS
/// as grandezas e não só na fração de quads.
///
/// ⚠️ **O gate `the_hierarchy_pays_and_the_number_is_here` foi calibrado com o
/// grafo do CONE e o leque**, e as duas coisas mudaram. Uma barra de razão
/// sobrevive à mudança do denominador sem que ninguém repare — esta sonda existe
/// para dizer se o gate ainda mede o que o nome dele diz.
#[test]
#[ignore = "sonda -- o caminho plano contra o hierarquico (CLAUDE.md §0.0)"]
fn measure_whether_the_hierarchy_still_pays() {
    use std::time::Instant;
    for (name, mesh) in [
        ("esfera 48x64", sphere()),
        ("uv 96x144 amassada", wrinkled()),
    ] {
        let e = mean_edge(&mesh);
        eprintln!("\n[quadflow] === {name} ===");
        eprintln!(
            "[quadflow]  razao |    caminho |  saida |  faces | quads% |  chi | maior | volume |     ms"
        );
        for ratio in [3.0f32, 4.0, 6.0] {
            let edge = e * ratio;
            let scale = ScaleField::uniform(&mesh, edge);
            let t = Instant::now();
            let o = crate::orientation::solve_orientation(&mesh, 32);
            let p = crate::position::solve_position(&mesh, &o, &scale, 32);
            let flat_ms = t.elapsed().as_secs_f64() * 1000.0;
            let fq = extract(&mesh, &o, &p, &scale).expect("plano");
            let t2 = Instant::now();
            let (oh, ph) = crate::solve::solve_fields(&mesh, &scale);
            let deep_ms = t2.elapsed().as_secs_f64() * 1000.0;
            let dq = extract(&mesh, &oh, &ph, &scale).expect("hierarquico");
            for (tag, q, ms) in [("plano", &fq, flat_ms), ("hierarq", &dq, deep_ms)] {
                let m = measure(q);
                eprintln!(
                    "[quadflow]  {ratio:5.2}x | {tag:>10} | {:6} | {:6} | {:5.1}% | {:4} | {:5} | {:+.4} | {ms:6.0}",
                    m.verts, m.faces, m.quads_pct, m.chi, m.max_sides, m.volume
                );
            }
        }
    }
}

/// **QUANTO A FORMA ANDA, em função do tamanho do quad** — a régua da A4.
///
/// ⚠️ **A barra de 1 % do ADR-0160 §4 foi escrita sem um tamanho de quad ao
/// lado**, e a distância de Hausdorff de uma grade de lado `s` sobre uma
/// superfície de raio de curvatura `R` não pode ser menor que a **flecha**
/// `s²/8R` — é geometria, não qualidade de implementação. Uma barra sem o `s` ao
/// lado mede o slider, não o algoritmo.
#[test]
#[ignore = "sonda -- a barra da A4 em funcao do tamanho do quad (CLAUDE.md §0.0)"]
fn measure_the_shape_drift_against_quad_size() {
    for (name, mesh) in [
        ("esfera 48x64", sphere()),
        ("toro 64x32", ph2d_mesh::shapes::torus(64, 32, 1.0, 0.35)),
        ("uv 96x144 amassada", wrinkled()),
    ] {
        let e = mean_edge(&mesh);
        let (floor, ceiling) = crate::scale::resolvable_edge_range(&mesh);
        eprintln!(
            "\n[quadflow] === {name}: aresta {e:.4} | faixa legal [{floor:.4}, {ceiling:.4}] ==="
        );
        eprintln!("[quadflow]  razao |   edge | saida->entrada | entrada->saida | flecha/diag");
        for ratio in [3.0f32, 4.0, 6.0, 8.0] {
            let edge = e * ratio;
            let scale = ScaleField::uniform(&mesh, edge);
            let (o, p) = crate::solve::solve_fields(&mesh, &scale);
            let q = extract(&mesh, &o, &p, &scale).expect("extraiu");
            let diag = super::bbox_diagonal(&mesh);
            let a = super::one_sided(&q.mesh, &mesh) / diag;
            let b = super::one_sided(&mesh, &q.mesh) / diag;
            eprintln!(
                "[quadflow]  {ratio:5.2}x | {edge:.4} |         {a:.4} |         {b:.4} | {:.4}",
                edge * edge / (8.0 * 0.35) / diag
            );
        }
    }
}

/// **DE ONDE VÊM OS TRIÂNGULOS QUE SOBRAM** — a anatomia do resíduo da A1.
///
/// ⚠️ **A pergunta é se o resto é BARATO ou se é a Q4.** Um triângulo que sobrou
/// por ter perdido o par no guloso é barato de curar; um que sobrou por ser o
/// único da vizinhança é uma singularidade do campo, e quem a fecha é o fluxo de
/// custo mínimo. A cura errada custa uma wave.
#[test]
#[ignore = "sonda -- a anatomia do residuo de triangulos (CLAUDE.md §0.0)"]
fn measure_where_the_leftover_triangles_come_from() {
    use std::collections::BTreeMap;
    for (name, mesh) in [
        ("esfera 48x64", sphere()),
        ("uv 96x144 amassada", wrinkled()),
    ] {
        let e = mean_edge(&mesh);
        eprintln!("\n[quadflow] === {name} ===");
        eprintln!("[quadflow]  razao | faces | tris | ciclos-3 | tri COM vizinho tri | isolados");
        for ratio in [3.0f32, 4.0, 6.0] {
            let scale = ScaleField::uniform(&mesh, e * ratio);
            let (o, p) = crate::solve::solve_fields(&mesh, &scale);
            let q = extract(&mesh, &o, &p, &scale).expect("extraiu");

            let tris: Vec<usize> = (0..q.mesh.faces().len())
                .filter(|i| q.mesh.faces()[*i].verts().len() == 3)
                .collect();
            // Aresta -> faces triangulares que a usam.
            let mut owner: BTreeMap<(u32, u32), Vec<usize>> = BTreeMap::new();
            for &i in &tris {
                let v = q.mesh.faces()[i].verts();
                for k in 0..3 {
                    let (a, b) = (v[k], v[(k + 1) % 3]);
                    owner
                        .entry(if a < b { (a, b) } else { (b, a) })
                        .or_default()
                        .push(i);
                }
            }
            let paired: std::collections::BTreeSet<usize> = owner
                .values()
                .filter(|w| w.len() == 2)
                .flat_map(|w| w.iter().copied())
                .collect();
            eprintln!(
                "[quadflow]  {ratio:5.2}x | {:5} | {:4} | {:8} | {:19} | {:8}",
                q.mesh.faces().len(),
                tris.len(),
                "-",
                paired.len(),
                tris.len() - paired.len()
            );
        }
    }
}

/// **O QUE A RELAXAÇÃO COMPRA, e por quantas passadas** — a sonda que escolhe o
/// [`crate::relax::RELAX_PASSES`].
#[test]
#[ignore = "sonda -- a relaxacao paga? (CLAUDE.md §0.0)"]
fn measure_the_relaxation() {
    use std::time::Instant;
    for (name, mesh) in [
        ("esfera 48x64", sphere()),
        ("toro 64x32", ph2d_mesh::shapes::torus(64, 32, 1.0, 0.35)),
        ("uv 96x144 amassada", wrinkled()),
    ] {
        let e = mean_edge(&mesh);
        let edge = 3.0 * e;
        let scale = ScaleField::uniform(&mesh, edge);
        let (o, p) = crate::solve::solve_fields(&mesh, &scale);
        let base = extract(&mesh, &o, &p, &scale).expect("extraiu").mesh;
        eprintln!("\n[quadflow] === {name} (quad {edge:.4}) ===");
        eprintln!(
            "[quadflow]  passadas | desvio de aresta | hausdorff (em quads) |  volume |     ms"
        );
        for passes in [0usize, 1, 2, 4, 8, 16] {
            let mut m = base.clone();
            let t = Instant::now();
            crate::relax::relax(&mut m, &mesh, passes);
            let ms = t.elapsed().as_secs_f64() * 1000.0;
            let h = super::one_sided(&m, &mesh).max(super::one_sided(&mesh, &m)) / edge;
            let mut vol = 0.0f64;
            let pos = m.positions();
            for f in m.faces() {
                let v = f.verts();
                for k in 1..v.len() - 1 {
                    let (a, b, c) = (
                        pos[v[0] as usize],
                        pos[v[k] as usize],
                        pos[v[k + 1] as usize],
                    );
                    vol += f64::from(a[0].mul_add(
                        b[1].mul_add(c[2], -(b[2] * c[1])),
                        a[1].mul_add(
                            b[2].mul_add(c[0], -(b[0] * c[2])),
                            a[2] * b[0].mul_add(c[1], -(b[1] * c[0])),
                        ),
                    )) / 6.0;
                }
            }
            eprintln!(
                "[quadflow]  {passes:8} |            {:.3} |                {h:.3} | {vol:7.4} | {ms:6.1}",
                crate::relax::edge_length_spread(&m)
            );
        }
    }
}

/// **QUANTAS VARREDURAS POR NÍVEL** — a sonda que escolhe o
/// [`crate::solve::SWEEPS_PER_LEVEL`], sobre a malha do PRODUTO.
///
/// ⚠️ **A justificativa daquela constante DISSOLVEU-SE em 2026-08-19**: ela dizia
/// *"2 é o último degrau que cabe no kill-criterion de 3 s"*, e o grafo por passo
/// de retícula cortou a extração de ~1,4 s para 0,02 s. Um teto que sumiu deixa
/// um número escolhido por um motivo que já não existe.
#[test]
#[ignore = "medicao de relogio -- rode sozinho, na maquina calma (CLAUDE.md §5.0)"]
fn measure_the_sweeps_per_level() {
    use std::time::Instant;
    let mesh = ph2d_mesh::shapes::sculpt_sphere(1.0);
    let edge = 3.0 * mean_edge(&mesh);
    let scale = ScaleField::uniform(&mesh, edge);
    eprintln!("\n[quadflow] === sculpt_sphere (98 306 vertices), quad {edge:.4} ===");
    eprintln!("[quadflow]  varreduras | quads% | maior | desvio | forma (quads) |  total s");
    for sweeps in [1usize, 2, 4, 8, 16] {
        let t = Instant::now();
        let (o, p) =
            crate::solve::solve_fields_with(&mesh, &scale, sweeps, crate::hierarchy::COARSEST);
        let mut q = extract(&mesh, &o, &p, &scale).expect("extraiu");
        crate::relax::relax(&mut q.mesh, &mesh, crate::relax::RELAX_PASSES);
        let total = t.elapsed().as_secs_f64();
        let h = super::one_sided(&q.mesh, &mesh).max(super::one_sided(&mesh, &q.mesh)) / edge;
        eprintln!(
            "[quadflow]  {sweeps:10} | {:5.1}% | {:5} | {:6.3} | {h:13.3} | {total:7.2}",
            q.quad_fraction() * 100.0,
            q.max_sides,
            crate::relax::edge_length_spread(&q.mesh)
        );
    }
}
