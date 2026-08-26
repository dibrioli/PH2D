//! ⭐⭐⭐ **A RÉGUA DOS EDGE LOOPS** — a única queixa do artista que nunca teve número.
//!
//! ```text
//! cargo run --release -p ph2d-quadextract --example loop_census -- <quad.obj> [original.obj]
//! ```
//!
//! ⛔⛔ **Por que ela existe.** Das quatro queixas de 2026-08-25, três foram medidas e duas
//! curadas. A que sobra — *«os edge loops que normalmente são gerados em áreas de transição
//! de topologia ainda não estão no estado da arte»* — **não tinha régua nenhuma**, e uma
//! queixa sem régua não se pode fechar nem refutar. Em 2026-08-26 o artista chamou o
//! resultado de *«pro»* **excluindo os loops**: ⇒ eles são, por palavras dele, o que separa
//! este módulo do nível seguinte.
//!
//! # ⭐ O que é um LOOP, e por que ele PARA
//!
//! Um edge loop atravessa um vértice tomando a aresta **oposta**. Isso só está definido num
//! vértice de **valência 4**: com quatro arestas, a oposta a `e` é a única que não partilha
//! quad nenhum com `e` naquele vértice. ⇒ **um loop morre numa singularidade**, e é
//! exactamente isso que *«áreas de transição de topologia»* quer dizer.
//!
//! ⚠️ **A régua é a DISTRIBUIÇÃO dos comprimentos, não a média.** Uma malha com muitos loops
//! curtos e alguns longos tem a mesma média de uma com todos médios, e as duas leem-se de
//! maneira oposta no visor do artista.
//!
//! ⚠️ **E ela vem em ARESTAS, não em unidades de mundo:** um loop de `40` arestas numa malha
//! de `600` quads e outro de `40` numa de `6 000` descrevem coisas diferentes — por isso a
//! contagem de quads sai ao lado, e a coluna que se compara entre malhas é a **fracção** de
//! arestas em loops longos.

use std::collections::BTreeMap;

fn load(name: &str) -> ph2d_mesh::Mesh {
    let text = std::fs::read_to_string(name).unwrap_or_else(|e| panic!("{name}: {e}"));
    ph2d_mesh::import_obj(&text)
        .unwrap_or_else(|e| panic!("{name} nao e' um OBJ deste leitor: {e:?}"))
        .into_iter()
        .next()
        .unwrap_or_else(|| panic!("{name} nao tem peca dentro"))
        .mesh
}

type Edge = (u32, u32);

fn key(a: u32, b: u32) -> Edge {
    if a < b { (a, b) } else { (b, a) }
}

/// ⭐ **A aresta OPOSTA a `e` no vértice `v`** — a única que não partilha quad com ela ali.
///
/// `None` quando `v` não tem valência 4, que é o que faz o loop **parar numa singularidade**.
fn across(
    at_vert: &BTreeMap<u32, Vec<Edge>>,
    faces_of_edge: &BTreeMap<Edge, Vec<u32>>,
    quads: &[[u32; 4]],
    v: u32,
    e: Edge,
) -> Option<Edge> {
    let inc = at_vert.get(&v)?;
    if inc.len() != 4 {
        return None;
    }
    // Os quads que tocam `e` — as arestas deles em `v` são as VIZINHAS de `e`.
    let mut neighbour: std::collections::BTreeSet<Edge> = std::collections::BTreeSet::new();
    for &f in faces_of_edge.get(&e)? {
        let q = quads[f as usize];
        for k in 0..4 {
            let (a, b) = (q[k], q[(k + 1) % 4]);
            if a == v || b == v {
                neighbour.insert(key(a, b));
            }
        }
    }
    inc.iter()
        .copied()
        .find(|c| *c != e && !neighbour.contains(c))
}

fn main() {
    let mut args = std::env::args().skip(1);
    let name = args.next().unwrap_or_else(|| {
        panic!("uso: loop_census <quad.obj> [original.obj]");
    });
    let mesh = load(&name);

    let quads: Vec<[u32; 4]> = mesh
        .faces()
        .iter()
        .filter(|f| f.verts().len() == 4)
        .map(|f| {
            let v = f.verts();
            [v[0], v[1], v[2], v[3]]
        })
        .collect();
    let non_quads = mesh.face_count() - quads.len();

    let mut faces_of_edge: BTreeMap<Edge, Vec<u32>> = BTreeMap::new();
    for (fi, q) in quads.iter().enumerate() {
        for k in 0..4 {
            #[allow(clippy::cast_possible_truncation)]
            faces_of_edge
                .entry(key(q[k], q[(k + 1) % 4]))
                .or_default()
                .push(fi as u32);
        }
    }
    let mut at_vert: BTreeMap<u32, Vec<Edge>> = BTreeMap::new();
    for e in faces_of_edge.keys() {
        at_vert.entry(e.0).or_default().push(*e);
        at_vert.entry(e.1).or_default().push(*e);
    }

    // Cada aresta pertence a UM loop. Percorre-se em ambos os sentidos até parar.
    let mut seen: std::collections::BTreeSet<Edge> = std::collections::BTreeSet::new();
    let mut lengths: Vec<usize> = Vec::new();
    let mut closed = 0usize;
    for &start in faces_of_edge.keys() {
        if seen.contains(&start) {
            continue;
        }
        let mut chain: Vec<Edge> = vec![start];
        seen.insert(start);
        let mut fechou = false;
        // Os dois sentidos, a partir de cada ponta da aresta.
        for &from in &[start.1, start.0] {
            let (mut e, mut v) = (start, from);
            while let Some(next) = across(&at_vert, &faces_of_edge, &quads, v, e) {
                if next == start {
                    fechou = true;
                    break;
                }
                if !seen.insert(next) {
                    break;
                }
                chain.push(next);
                v = if next.0 == v { next.1 } else { next.0 };
                e = next;
            }
            if fechou {
                break;
            }
        }
        if fechou {
            closed += 1;
        }
        lengths.push(chain.len());
    }

    lengths.sort_unstable();
    let total_edges: usize = lengths.iter().sum();
    let at = |q: usize| lengths.get(lengths.len() * q / 100).copied().unwrap_or(0);
    // ⭐ A FRACÇÃO de arestas que vive em loops longos — a coluna comparável entre malhas.
    let frac = |n: usize| {
        let s: usize = lengths.iter().filter(|l| **l >= n).sum();
        100.0 * s as f64 / total_edges.max(1) as f64
    };
    // ⛔ Um loop de UMA aresta é uma aresta cercada de singularidades dos dois lados.
    let solitarias = lengths.iter().filter(|l| **l == 1).count();

    println!(
        "{name}: {} quads ({non_quads} nao-quads), {} arestas, {} LOOPS ({closed} fechados)",
        quads.len(),
        total_edges,
        lengths.len()
    );
    println!(
        "  ⭐⭐⭐ COMPRIMENTO dos loops (arestas): p50 {} p90 {} max {} · ⛔ {solitarias} de UMA aresta",
        at(50),
        at(90),
        lengths.last().copied().unwrap_or(0)
    );
    println!(
        "  ⭐⭐ FRACCAO das arestas em loops de >= 8: {:.1}% · >= 16: {:.1}% · >= 32: {:.1}%",
        frac(8),
        frac(16),
        frac(32)
    );

    if let Some(orig) = args.next() {
        let src = load(&orig);
        let (relief, conf) = ph2d_quadfill::follows_relief(&src, &mesh);
        let shape = ph2d_quadfill::quad_shape(&mesh);
        println!(
            "  ⭐⭐⭐ OBEDECE AO RELEVO: {relief:.1}° (confianca {conf:.2}) — ⚠️ 22,5° = «nao olhou» \
             | enviesamento p50 {:.1}° p99 {:.1}° (>60: {})",
            shape.skew_p50, shape.skew_p99, shape.skew_over_60
        );
    }
}
