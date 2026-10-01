//! Dos anéis à malha: triangulação de Delaunay COM RESTRIÇÕES (as arestas dos anéis), a classificação
//! dentro/fora por PARIDADE, e a fusão gulosa em convexos (Hertel–Mehlhorn).

use std::collections::BTreeMap;

use spade::{ConstrainedDelaunayTriangulation, Point2, Triangulation};

use crate::lattice::{P, len2, orient, to_world};

/// Porque a triangulação recusou.
#[derive(Clone, Debug, PartialEq)]
pub enum TriError {
    /// O `spade` recusou um vértice (fora do alcance dele) — não acontece na grelha de metros.
    Insertion(String),
    /// Duas arestas de anel cruzam-se — o resultado da união/diferença do Clipper não é simples.
    /// ⛔ Recusado em voz alta: com uma restrição a menos, uma parede abria-se e o agente passava.
    Crossing { count: usize },
    /// O `spade` juntou dois vértices que a grelha separava (não acontece: a entrada é única).
    Merged { input: usize, kept: usize },
}

/// Os anéis da grelha → os vértices (únicos, em metros) e os triângulos DENTRO, anti-horários.
pub fn triangulate(rings: &[Vec<P>]) -> Result<(Vec<P>, Vec<[u32; 3]>), TriError> {
    // Os vértices únicos, pela ordem em que aparecem (BTreeMap: o índice não depende de hash).
    let mut index: BTreeMap<P, u32> = BTreeMap::new();
    let mut pts: Vec<P> = Vec::new();
    let mut edges: Vec<[usize; 2]> = Vec::new();
    for ring in rings {
        let n = ring.len();
        if n < 3 {
            continue;
        }
        let ids: Vec<u32> = ring
            .iter()
            .map(|&p| {
                *index.entry(p).or_insert_with(|| {
                    pts.push(p);
                    (pts.len() - 1) as u32
                })
            })
            .collect();
        for i in 0..n {
            let (a, b) = (ids[i], ids[(i + 1) % n]);
            if a != b {
                edges.push([a as usize, b as usize]);
            }
        }
    }
    if pts.len() < 3 {
        return Ok((pts, Vec::new()));
    }
    let verts: Vec<Point2<f64>> = pts
        .iter()
        .map(|&p| {
            let w = to_world(p);
            Point2::new(w[0], w[1])
        })
        .collect();
    let mut crossings = 0usize;
    let cdt: ConstrainedDelaunayTriangulation<Point2<f64>> =
        ConstrainedDelaunayTriangulation::try_bulk_load_cdt(verts, edges, |_| crossings += 1)
            .map_err(|e| TriError::Insertion(format!("{e:?}")))?;
    if crossings > 0 {
        return Err(TriError::Crossing { count: crossings });
    }
    if cdt.num_vertices() != pts.len() {
        return Err(TriError::Merged {
            input: pts.len(),
            kept: cdt.num_vertices(),
        });
    }

    // A paridade: atravessar uma restrição troca dentro/fora. A face de fora (índice 0) está fora.
    let nf = cdt.num_all_faces();
    let mut parity = vec![u8::MAX; nf];
    let mut queue: Vec<usize> = Vec::new();
    for face in cdt.inner_faces() {
        let fi = face.fix().index();
        for e in face.adjacent_edges() {
            if e.rev().face().is_outer() {
                let p = u8::from(cdt.is_constraint_edge(e.as_undirected().fix()));
                if parity[fi] == u8::MAX {
                    parity[fi] = p;
                    queue.push(fi);
                }
            }
        }
    }
    // O índice de uma face → a pega fixa dela (o `spade` não constrói uma pega a partir do índice).
    let mut by_index = vec![None; nf];
    for f in cdt.fixed_inner_faces() {
        by_index[f.index()] = Some(f);
    }
    let mut head = 0;
    while head < queue.len() {
        let fi = queue[head];
        head += 1;
        let Some(fixed) = by_index[fi] else { continue };
        let handle = cdt.face(fixed);
        for e in handle.adjacent_edges() {
            let other = e.rev().face();
            let Some(inner) = other.as_inner() else {
                continue;
            };
            let oi = inner.fix().index();
            if parity[oi] != u8::MAX {
                continue;
            }
            let flip = u8::from(cdt.is_constraint_edge(e.as_undirected().fix()));
            parity[oi] = parity[fi] ^ flip;
            queue.push(oi);
        }
    }

    let mut tris = Vec::new();
    for face in cdt.inner_faces() {
        if parity[face.fix().index()] != 1 {
            continue;
        }
        let [a, b, c] = face.vertices().map(|v| v.fix().index() as u32);
        let o = orient(pts[a as usize], pts[b as usize], pts[c as usize]);
        if o > 0 {
            tris.push([a, b, c]);
        } else if o < 0 {
            tris.push([a, c, b]);
        }
    }
    Ok((pts, tris))
}

/// A fusão gulosa de Hertel–Mehlhorn: tira as diagonais (as arestas interiores que não são parede),
/// da MAIS LONGA para a mais curta, sempre que o polígono que fica é ESTRITAMENTE convexo nas duas
/// pontas — a decisão é exacta (orientação inteira). A ordem é total (comprimento, depois os
/// índices) ⇒ a mesma entrada dá a mesma malha em toda máquina.
pub fn merge_convex(pts: &[P], tris: &[[u32; 3]]) -> Vec<Vec<u32>> {
    let mut rings: Vec<Vec<u32>> = tris.iter().map(|t| t.to_vec()).collect();
    let mut owner: BTreeMap<(u32, u32), usize> = BTreeMap::new();
    for (ti, t) in tris.iter().enumerate() {
        for i in 0..3 {
            owner.insert((t[i], t[(i + 1) % 3]), ti);
        }
    }
    let mut diags: Vec<(i128, u32, u32, usize, usize)> = Vec::new();
    for (&(u, w), &ta) in &owner {
        if u < w
            && let Some(&tb) = owner.get(&(w, u))
        {
            diags.push((len2(pts[u as usize], pts[w as usize]), u, w, ta, tb));
        }
    }
    diags.sort_unstable_by(|a, b| b.0.cmp(&a.0).then(a.1.cmp(&b.1)).then(a.2.cmp(&b.2)));
    let mut parent: Vec<usize> = (0..rings.len()).collect();
    fn find(parent: &mut [usize], mut x: usize) -> usize {
        while parent[x] != x {
            parent[x] = parent[parent[x]];
            x = parent[x];
        }
        x
    }
    for &(_, u, w, ta, tb) in &diags {
        let a = find(&mut parent, ta);
        let b = find(&mut parent, tb);
        if a == b {
            continue;
        }
        if let Some(m) = try_merge(pts, &rings[a], &rings[b], u, w) {
            rings[a] = m;
            rings[b].clear();
            parent[b] = a;
        }
    }
    rings.into_iter().filter(|r| !r.is_empty()).collect()
}

/// Funde os anéis `ra` e `rb` pela aresta partilhada `{u, w}`, se o resultado é estritamente convexo
/// nas duas pontas da aresta que desaparece.
fn try_merge(pts: &[P], ra: &[u32], rb: &[u32], u: u32, w: u32) -> Option<Vec<u32>> {
    let na = ra.len();
    let nb = rb.len();
    // Em `ra` a aresta é x → y; em `rb` é y → x.
    let i = (0..na).find(|&i| {
        let (p, q) = (ra[i], ra[(i + 1) % na]);
        (p == u && q == w) || (p == w && q == u)
    })?;
    let (x, y) = (ra[i], ra[(i + 1) % na]);
    let j = (0..nb).find(|&j| rb[j] == y && rb[(j + 1) % nb] == x)?;
    // ra a partir de y, até x (inclusive): y, a1, …, x
    let mut m: Vec<u32> = (0..na).map(|k| ra[(i + 1 + k) % na]).collect();
    // rb a partir de x: x, b1, …, y — entra só o interior b1..bk.
    let interior: Vec<u32> = (1..nb - 1).map(|k| rb[(j + 1 + k) % nb]).collect();
    // Convexidade em x: anterior = o que vem antes de x em m (m[na-2]), seguinte = b1 (ou y).
    let after_x = interior.first().copied().unwrap_or(y);
    let before_y = interior.last().copied().unwrap_or(x);
    let px = pts[x as usize];
    let py = pts[y as usize];
    let prev_x = pts[m[na - 2] as usize];
    let next_y = pts[m[1 % na] as usize];
    if orient(prev_x, px, pts[after_x as usize]) <= 0 {
        return None;
    }
    if orient(pts[before_y as usize], py, next_y) <= 0 {
        return None;
    }
    m.extend(interior);
    Some(m)
}
