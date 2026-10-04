//! (W12, plano 30 §20) Os ORÁCULOS da triangulação: as versões de antes da W12, verbatim (só o nome muda)
//! — as novas dão o MESMO, ao bit, e os gates comparam-nas.

use std::collections::BTreeMap;

use spade::{ConstrainedDelaunayTriangulation, Point2, Triangulation};

use super::{FORA, NADA, TriError};
use crate::lattice::{P, len2, orient, to_world};

/// A fusão de antes (um `BTreeMap` das semi-arestas).
pub(super) fn merge_convex_labeled_antigo(
    pts: &[P],
    tris: &[[u32; 3]],
    labels: &[u16],
) -> (Vec<Vec<u32>>, Vec<u16>) {
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
        if a == b || labels[a] != labels[b] {
            continue;
        }
        if let Some(m) = try_merge_antigo(pts, &rings[a], &rings[b], u, w) {
            rings[a] = m;
            rings[b].clear();
            parent[b] = a;
        }
    }
    rings
        .into_iter()
        .zip(labels.iter().copied())
        .filter(|(r, _)| !r.is_empty())
        .unzip()
}

/// Funde os anéis `ra` e `rb` pela aresta partilhada `{u, w}`, se o resultado é estritamente convexo
/// nas duas pontas da aresta que desaparece.
fn try_merge_antigo(pts: &[P], ra: &[u32], rb: &[u32], u: u32, w: u32) -> Option<Vec<u32>> {
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

/// A triangulação dos pedaços de antes (o `BTreeMap` dos vértices e o das restrições).
#[allow(clippy::type_complexity)]
pub(super) fn triangulate_pieces_antigo(
    pieces: &[Vec<Vec<P>>],
) -> Result<(Vec<P>, Vec<[u32; 3]>, Vec<u16>), TriError> {
    // ⚠️ Os ESPIGÕES saem antes de tudo (ver `limpa_anel`).
    let limpos: Vec<Vec<Vec<P>>> = pieces
        .iter()
        .map(|rings| {
            rings
                .iter()
                .map(|r| super::limpa_anel(r))
                .filter(|r| r.len() >= 3)
                .collect()
        })
        .collect();
    let reparados;
    let pieces = if limpos.len() > 1 {
        reparados = repair_t_junctions_antigo(&limpos);
        &reparados[..]
    } else {
        &limpos[..]
    };
    // Os vértices únicos, pela ordem em que aparecem (BTreeMap: o índice não depende de hash).
    let mut index: BTreeMap<P, u32> = BTreeMap::new();
    let mut pts: Vec<P> = Vec::new();
    let mut edges: Vec<[usize; 2]> = Vec::new();
    // Os pedaços de cada restrição (uma fronteira comum aparece nos dois, em sentidos opostos, e
    // entra UMA vez na triangulação — pela ordem da 1.ª aparição).
    let mut owners: BTreeMap<(u32, u32), Vec<u16>> = BTreeMap::new();
    for (k, rings) in pieces.iter().enumerate() {
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
                if a == b {
                    continue;
                }
                let quem = owners.entry((a.min(b), a.max(b))).or_default();
                if quem.is_empty() {
                    edges.push([a as usize, b as usize]);
                }
                quem.push(k as u16);
            }
        }
    }
    if pts.len() < 3 {
        return Ok((pts, Vec::new(), Vec::new()));
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

    // A paridade: atravessar uma restrição troca o pedaço. A face de fora (índice 0) está fora.
    // O estado: `FORA`, ou o índice do pedaço; `NADA` = ainda não visitada.
    // ⚠️ «É restrição?» pergunta-se ao `spade` (a fonte da verdade de sempre, que sabe das que partiu);
    // os DONOS vêm da nossa tabela. Com um pedaço só, toda restrição alterna dentro/fora — a paridade
    // antiga, ao bit.
    let um_so = pieces.len() == 1;
    // A troca só é EXACTA por dono quando a aresta tem um dono (e o estado é ele ou fora) ou dois
    // distintos (e o estado é um deles); senão o triângulo do outro lado decide-se por ponto-no-
    // polígono exacto (`None`). ⚠️ Medido: quatro pedaços a encontrarem-se num ponto deixam uma aresta
    // de UMA unidade com os donos `[0, 0, 2, 5]`, e a troca por dono etiquetava um triângulo de fora.
    let cruza = |estado: u16, a: usize, b: usize, restricao: bool| -> Option<u16> {
        if !restricao {
            return Some(estado);
        }
        if um_so {
            return Some(if estado == 0 { FORA } else { 0 });
        }
        let (a, b) = (a as u32, b as u32);
        match owners.get(&(a.min(b), a.max(b))).map(Vec::as_slice) {
            Some(&[k]) if estado == k => Some(FORA),
            Some(&[k]) if estado == FORA => Some(k),
            Some(&[k, j]) if k != j && estado == k => Some(j),
            Some(&[k, j]) if k != j && estado == j => Some(k),
            _ => None,
        }
    };
    let pip = |f: [usize; 3]| super::point_in_pieces(pieces, [pts[f[0]], pts[f[1]], pts[f[2]]]);
    let nf = cdt.num_all_faces();
    let mut parity = vec![NADA; nf];
    let mut queue: Vec<usize> = Vec::new();
    for face in cdt.inner_faces() {
        let fi = face.fix().index();
        for e in face.adjacent_edges() {
            if e.rev().face().is_outer() && parity[fi] == NADA {
                let [a, b] = e.vertices().map(|v| v.fix().index());
                let r = cdt.is_constraint_edge(e.as_undirected().fix());
                parity[fi] = cruza(FORA, a, b, r)
                    .unwrap_or_else(|| pip(face.vertices().map(|v| v.fix().index())));
                queue.push(fi);
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
            if parity[oi] != NADA {
                continue;
            }
            let [a, b] = e.vertices().map(|v| v.fix().index());
            let r = cdt.is_constraint_edge(e.as_undirected().fix());
            parity[oi] = cruza(parity[fi], a, b, r)
                .unwrap_or_else(|| pip(inner.vertices().map(|v| v.fix().index())));
            queue.push(oi);
        }
    }

    let mut tris = Vec::new();
    let mut labels = Vec::new();
    for face in cdt.inner_faces() {
        let k = parity[face.fix().index()];
        if k == NADA || k == FORA {
            continue;
        }
        let [a, b, c] = face.vertices().map(|v| v.fix().index() as u32);
        let o = orient(pts[a as usize], pts[b as usize], pts[c as usize]);
        if o > 0 {
            tris.push([a, b, c]);
        } else if o < 0 {
            tris.push([a, c, b]);
        } else {
            continue;
        }
        labels.push(k);
    }
    Ok((pts, tris, labels))
}

fn repair_t_junctions_antigo(pieces: &[Vec<Vec<P>>]) -> Vec<Vec<Vec<P>>> {
    let todos: Vec<P> = pieces.iter().flatten().flatten().copied().collect();
    if todos.is_empty() {
        return pieces.to_vec();
    }
    let (lo, hi) = todos.iter().fold(
        ((i64::MAX, i64::MAX), (i64::MIN, i64::MIN)),
        |(lo, hi), &(x, y)| ((lo.0.min(x), lo.1.min(y)), (hi.0.max(x), hi.1.max(y))),
    );
    let ext = (hi.0 - lo.0).max(hi.1 - lo.1).max(1);
    let por_eixo = ((todos.len() as f64).sqrt().ceil() as i64).max(1);
    let lado = (ext / por_eixo).max(1);
    let balde = |p: P| ((p.0 - lo.0) / lado, (p.1 - lo.1) / lado);
    let mut baldes: BTreeMap<(i64, i64), Vec<P>> = BTreeMap::new();
    for &p in &todos {
        let lista = baldes.entry(balde(p)).or_default();
        if !lista.contains(&p) {
            lista.push(p);
        }
    }
    pieces
        .iter()
        .map(|rings| {
            rings
                .iter()
                .map(|ring| {
                    let n = ring.len();
                    let mut out = Vec::with_capacity(n);
                    for i in 0..n {
                        let (a, b) = (ring[i], ring[(i + 1) % n]);
                        out.push(a);
                        let ba = balde((a.0.min(b.0) - 2, a.1.min(b.1) - 2));
                        let bb = balde((a.0.max(b.0) + 2, a.1.max(b.1) + 2));
                        let mut meio: Vec<(i128, P)> = Vec::new();
                        for bx in ba.0.min(bb.0)..=ba.0.max(bb.0) {
                            for by in ba.1.min(bb.1)..=ba.1.max(bb.1) {
                                for &p in baldes.get(&(bx, by)).into_iter().flatten() {
                                    let o = orient(a, b, p);
                                    if p == a || p == b || o * o >= 4 * len2(a, b) {
                                        continue;
                                    }
                                    let t = i128::from(p.0 - a.0) * i128::from(b.0 - a.0)
                                        + i128::from(p.1 - a.1) * i128::from(b.1 - a.1);
                                    if t > 0 && t < len2(a, b) && !meio.iter().any(|m| m.1 == p) {
                                        meio.push((t, p));
                                    }
                                }
                            }
                        }
                        meio.sort_unstable();
                        out.extend(meio.into_iter().map(|m| m.1));
                    }
                    out
                })
                .collect()
        })
        .collect()
}

/// ⭐ (W12) **A triangulação, a reparação das junções em T e a fusão de agora são as de antes, ao bit**,
/// sobre os pedaços REAIS de cenas ao calhas (caixas rodadas, círculos e cápsulas; três raios; os dois
/// cantos; lamas que se sobrepõem — o que dá pedaços com dois donos e junções em T), com e sem fusão.
#[test]
fn a_triangulacao_e_a_fusao_de_agora_sao_as_de_antes_ao_bit() {
    use crate::{Area, Corner, Params, Shape};
    let mut s = 0x0012_5EEDu64;
    let mut r = move || {
        s = s
            .wrapping_mul(6_364_136_223_846_793_005)
            .wrapping_add(1_442_695_040_888_963_407);
        (s >> 11) as f64 / (1u64 << 53) as f64
    };
    let regiao = vec![[0.0, 0.0], [30.0, 0.0], [30.0, 20.0], [0.0, 20.0]];
    let (mut varios, mut inseridos, mut dois_donos, mut fundidos, mut triangulos) =
        (0usize, 0usize, 0usize, 0usize, 0usize);
    for caso in 0..36 {
        let mut forma = |r: &mut dyn FnMut() -> f64, k: usize| {
            let c = [r() * 30.0, r() * 20.0];
            match k % 3 {
                0 => Shape::Circle {
                    center: c,
                    radius: 0.2 + r() * 1.5,
                },
                1 => {
                    let (hx, hy, a) = (0.2 + r() * 1.8, 0.2 + r() * 1.8, r() * 3.0);
                    let (co, si) = (a.cos(), a.sin());
                    Shape::Convex(
                        [[-hx, -hy], [hx, -hy], [hx, hy], [-hx, hy]]
                            .iter()
                            .map(|p| [c[0] + p[0] * co - p[1] * si, c[1] + p[0] * si + p[1] * co])
                            .collect(),
                    )
                }
                _ => Shape::Capsule {
                    a: c,
                    b: [c[0] + r() * 3.0, c[1] + r() * 2.0],
                    radius: 0.2 + r() * 0.6,
                },
            }
        };
        let obs: Vec<Shape> = (0..25).map(|k| forma(&mut r, k)).collect();
        let areas: Vec<Area> = (0..caso % 7)
            .map(|k| Area {
                shape: forma(&mut r, k + 1),
                id: (k + 1) as u16,
            })
            .collect();
        let p = Params {
            agent_radius: [0.0, 0.3, 0.6][caso % 3],
            corner: if caso % 4 == 0 {
                Corner::Miter
            } else {
                Corner::Round
            },
            ..Params::default()
        };
        let Some((walk, aneis_das_areas)) = crate::chao_e_areas(&regiao, &obs, &areas, &p) else {
            continue;
        };
        let (_, aneis) = crate::pedacos(walk, &aneis_das_areas);
        if aneis.len() > 1 {
            varios += 1;
            let limpos: Vec<Vec<Vec<P>>> = aneis
                .iter()
                .map(|rs| {
                    rs.iter()
                        .map(|x| super::limpa_anel(x))
                        .filter(|x| x.len() >= 3)
                        .collect()
                })
                .collect();
            let novo = super::repair_t_junctions(&limpos);
            assert_eq!(
                novo,
                repair_t_junctions_antigo(&limpos),
                "caso {caso}: junções em T"
            );
            let conta = |v: &Vec<Vec<Vec<P>>>| v.iter().flatten().map(Vec::len).sum::<usize>();
            inseridos += conta(&novo) - conta(&limpos);
            let mut donos: BTreeMap<(P, P), Vec<usize>> = BTreeMap::new();
            for (k, rs) in novo.iter().enumerate() {
                for x in rs {
                    for i in 0..x.len() {
                        let (a, b) = (x[i], x[(i + 1) % x.len()]);
                        donos.entry((a.min(b), a.max(b))).or_default().push(k);
                    }
                }
            }
            dois_donos += donos
                .values()
                .filter(|d| d.len() == 2 && d[0] != d[1])
                .count();
        }
        let agora = super::triangulate_pieces(&aneis);
        assert_eq!(
            agora,
            triangulate_pieces_antigo(&aneis),
            "caso {caso}: a triangulação"
        );
        let (pts, tris, pedaco) = agora.expect("triangula");
        triangulos += tris.len();
        let fusao = super::merge_convex_labeled(&pts, &tris, &pedaco);
        assert_eq!(
            fusao,
            merge_convex_labeled_antigo(&pts, &tris, &pedaco),
            "caso {caso}: a fusão"
        );
        assert_eq!(
            super::merge_convex(&pts, &tris),
            merge_convex_labeled_antigo(&pts, &tris, &vec![0; tris.len()]).0,
            "caso {caso}: a fusão sem pedaços"
        );
        fundidos += tris.len() - fusao.0.len();
    }
    // CONTROLOS de população (medidos: 30 · 15 · 2 027 · 10 228 de 23 475).
    assert!(varios >= 25, "só {varios} casos com vários pedaços");
    assert!(inseridos >= 10, "só {inseridos} pontos das junções em T");
    assert!(
        dois_donos >= 1_500,
        "só {dois_donos} arestas com dois donos"
    );
    assert!(
        fundidos >= 8_000 && triangulos >= 20_000,
        "{fundidos} fusões de {triangulos}"
    );
}
