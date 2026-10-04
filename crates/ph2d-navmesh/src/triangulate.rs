//! Dos anéis à malha: triangulação de Delaunay COM RESTRIÇÕES (as arestas dos anéis), a classificação
//! dentro/fora por PARIDADE, e a fusão gulosa em convexos (Hertel–Mehlhorn).

use spade::{ConstrainedDelaunayTriangulation, Point2, Triangulation};

use crate::lattice::{P, len2, orient, to_world};

/// O estado de uma face ainda não visitada, e o de uma face FORA de todo pedaço.
const NADA: u16 = u16::MAX;
const FORA: u16 = u16::MAX - 1;

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
    let (pts, tris, _) = triangulate_pieces(std::slice::from_ref(&rings.to_vec()))?;
    Ok((pts, tris))
}

/// ⭐ (W7) **Vários PEDAÇOS disjuntos** (o chão comum e uma peça por área de custo, cada um com os
/// seus anéis) numa só triangulação → os vértices, os triângulos e o PEDAÇO de cada triângulo.
///
/// A paridade generaliza-se sem ler a orientação dos anéis: o estado de uma face é «em que pedaço
/// estou» (ou fora), e atravessar uma restrição troca-o pelos pedaços que a têm — a de UM pedaço
/// entra/sai dele, a de DOIS passa de um ao outro. Com um pedaço só é a paridade de sempre, ao bit.
///
/// ⚠️ Os anéis de pedaços diferentes vêm de operações diferentes do Clipper e uma fronteira comum pode
/// sair partida num lado e inteira no outro (uma junção em T): os vértices de um pedaço que caem
/// EXACTAMENTE numa aresta de outro entram nela antes da triangulação (orientação inteira).
#[allow(clippy::type_complexity)]
pub fn triangulate_pieces(
    pieces: &[Vec<Vec<P>>],
) -> Result<(Vec<P>, Vec<[u32; 3]>, Vec<u16>), TriError> {
    // ⚠️ Os ESPIGÕES saem antes de tudo (ver `limpa_anel`).
    let limpos: Vec<Vec<Vec<P>>> = pieces
        .iter()
        .map(|rings| {
            rings
                .iter()
                .map(|r| limpa_anel(r))
                .filter(|r| r.len() >= 3)
                .collect()
        })
        .collect();
    let reparados;
    let pieces = if limpos.len() > 1 {
        reparados = repair_t_junctions(&limpos);
        &reparados[..]
    } else {
        &limpos[..]
    };
    // Os vértices únicos, pela ordem em que aparecem: cada ponto pela 1.ª aparição (W12: duas ordenações
    // em vez de um `BTreeMap` — o oráculo em `triangulate_oraculo.rs`).
    let todos: Vec<P> = pieces
        .iter()
        .flatten()
        .filter(|r| r.len() >= 3)
        .flatten()
        .copied()
        .collect();
    let mut ord: Vec<(P, u32)> = todos
        .iter()
        .enumerate()
        .map(|(i, &p)| (p, i as u32))
        .collect();
    ord.sort_unstable();
    let mut grupos: Vec<(u32, u32)> = Vec::new(); // (1.ª aparição, início do grupo em `ord`)
    for (g, w) in ord.iter().enumerate() {
        if g == 0 || ord[g - 1].0 != w.0 {
            grupos.push((w.1, g as u32));
        }
    }
    grupos.sort_unstable();
    let mut id_de = vec![0u32; todos.len()];
    let mut pts: Vec<P> = Vec::with_capacity(grupos.len());
    for (id, &(_, g)) in grupos.iter().enumerate() {
        let p = ord[g as usize].0;
        pts.push(p);
        for w in ord[g as usize..].iter().take_while(|w| w.0 == p) {
            id_de[w.1 as usize] = id as u32;
        }
    }
    // Os pedaços de cada restrição (uma fronteira comum aparece nos dois, em sentidos opostos, e
    // entra UMA vez na triangulação — pela ordem da 1.ª aparição): `(chave, ordem, pedaço)` ordenado.
    let mut arestas: Vec<((u32, u32), u32, u16)> = Vec::new();
    // O sentido de cada aparição (a restrição entra pelo da 1.ª).
    let mut sentido: Vec<[usize; 2]> = Vec::new();
    let mut s = 0;
    for (k, rings) in pieces.iter().enumerate() {
        for ring in rings.iter().filter(|r| r.len() >= 3) {
            let n = ring.len();
            for i in 0..n {
                let (a, b) = (id_de[s + i], id_de[s + (i + 1) % n]);
                if a != b {
                    arestas.push(((a.min(b), a.max(b)), sentido.len() as u32, k as u16));
                    sentido.push([a as usize, b as usize]);
                }
            }
            s += n;
        }
    }
    arestas.sort_unstable();
    let mut primeiras: Vec<u32> = Vec::new();
    for (g, w) in arestas.iter().enumerate() {
        if g == 0 || arestas[g - 1].0 != w.0 {
            primeiras.push(w.1);
        }
    }
    primeiras.sort_unstable();
    let edges: Vec<[usize; 2]> = primeiras.iter().map(|&o| sentido[o as usize]).collect();
    let donos: Vec<u16> = arestas.iter().map(|w| w.2).collect();
    let owners = |a: u32, b: u32| {
        let c = (a.min(b), a.max(b));
        let i = arestas.partition_point(|w| w.0 < c);
        let f = arestas.partition_point(|w| w.0 <= c);
        &donos[i..f]
    };
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
        match owners(a as u32, b as u32) {
            &[k] if estado == k => Some(FORA),
            &[k] if estado == FORA => Some(k),
            &[k, j] if k != j && estado == k => Some(j),
            &[k, j] if k != j && estado == j => Some(k),
            _ => None,
        }
    };
    let pip = |f: [usize; 3]| point_in_pieces(pieces, [pts[f[0]], pts[f[1]], pts[f[2]]]);
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

/// Um anel sem pontos repetidos e sem ESPIGÕES — um vértice onde o anel volta para trás pela MESMA
/// recta (orientação inteira `0` e o sentido a inverter). ⚠️ Medido (a cena `PH2D_NAV_SMOKE=4`, o furo
/// da lava a cortar a quina de quatro mosaicos com as paredes do recinto encostadas): o Clipper, que
/// preserva os colineares, devolveu o anel andável de um mosaico a ir até `(0, 0)` e voltar sobre o
/// fundo dele; as duas arestas sobrepostas entravam como restrições e a paridade etiquetava o entalhe
/// DENTRO da lava como chão. Tirar um espigão pode criar outro ao lado: repete até não haver.
fn limpa_anel(ring: &[P]) -> Vec<P> {
    let mut v: Vec<P> = ring.to_vec();
    loop {
        let n = v.len();
        if n < 3 {
            return v;
        }
        let mut tirou = false;
        let mut i = 0;
        while i < v.len() && v.len() >= 3 {
            let m = v.len();
            let (a, b, c) = (v[(i + m - 1) % m], v[i], v[(i + 1) % m]);
            let volta = orient(a, b, c) == 0
                && i128::from(b.0 - a.0) * i128::from(c.0 - b.0)
                    + i128::from(b.1 - a.1) * i128::from(c.1 - b.1)
                    <= 0;
            if b == a || volta {
                v.remove(i);
                tirou = true;
            } else {
                i += 1;
            }
        }
        if !tirou {
            return v;
        }
    }
}

/// O pedaço que contém o BARICENTRO do triângulo (ou `FORA`), por paridade de cruzamentos sobre os
/// anéis de cada pedaço, em inteiros exactos (as coordenadas ×3). O baricentro de um triângulo não
/// degenerado está no interior dele, logo nunca em cima de uma restrição — a pergunta não tem empate.
fn point_in_pieces(pieces: &[Vec<Vec<P>>], tri: [P; 3]) -> u16 {
    let q = (
        i128::from(tri[0].0) + i128::from(tri[1].0) + i128::from(tri[2].0),
        i128::from(tri[0].1) + i128::from(tri[1].1) + i128::from(tri[2].1),
    );
    for (k, rings) in pieces.iter().enumerate() {
        let mut dentro = false;
        for ring in rings {
            let n = ring.len();
            for i in 0..n {
                let (a, b) = (ring[i], ring[(i + 1) % n]);
                let (ax, ay) = (3 * i128::from(a.0), 3 * i128::from(a.1));
                let (bx, by) = (3 * i128::from(b.0), 3 * i128::from(b.1));
                if (ay > q.1) != (by > q.1) {
                    // q.x < ax + (q.y − ay)(bx − ax)/(by − ay), sem dividir.
                    let lhs = (q.0 - ax) * (by - ay);
                    let rhs = (q.1 - ay) * (bx - ax);
                    if (by > ay && lhs < rhs) || (by < ay && lhs > rhs) {
                        dentro = !dentro;
                    }
                }
            }
        }
        if dentro {
            return k as u16;
        }
    }
    FORA
}

/// As junções em T: todo vértice a MENOS DE DUAS UNIDADES da grelha do interior de uma aresta (de outro
/// pedaço, ou de outro anel do mesmo) entra nela, pela ordem ao longo da aresta. A procura dos
/// candidatos é por uma grelha de baldes sobre os vértices (`~1` por balde).
///
/// ⚠️ «Perto» e não «em cima»: a área 3 cortada pela fronteira da área 2 tem um vértice no CRUZAMENTO,
/// arredondado pelo Clipper (até `√2/2` unidade fora da recta), e o pedaço da área 2, que não sabe da
/// 3, não o tem — as duas arestas cruzavam-se a `< 15 µm` (medido: a 1.ª corrida da sonda
/// `medir_custo`). E a aresta do outro lado pode ser ela própria um PEDAÇO, com as pontas noutros
/// cruzamentos arredondados (mais `√2/2`): o pior caso é `√2` unidade — medido um a `1,016` (gate
/// `areas::cada_poligono_sabe…`, semente 13). Duas unidades são `30 µm`, abaixo de toda forma real.
fn repair_t_junctions(pieces: &[Vec<Vec<P>>]) -> Vec<Vec<Vec<P>>> {
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
    // (W12) Os baldes numa grelha contígua (era um `BTreeMap` de listas): os pontos únicos, por balde.
    // A ordem dentro de um balde não conta — os candidatos de uma aresta ordenam-se por `(t, p)`.
    let (nbx, nby) = (balde(hi).0 + 1, balde(hi).1 + 1);
    let mut unicos = todos.clone();
    unicos.sort_unstable();
    unicos.dedup();
    let celula = |b: (i64, i64)| (b.1 * nbx + b.0) as usize;
    let mut off = vec![0u32; (nbx * nby) as usize + 1];
    for &p in &unicos {
        off[celula(balde(p)) + 1] += 1;
    }
    for c in 0..(nbx * nby) as usize {
        off[c + 1] += off[c];
    }
    let mut cursor = off.clone();
    let mut itens = vec![(0i64, 0i64); unicos.len()];
    for &p in &unicos {
        let c = celula(balde(p));
        itens[cursor[c] as usize] = p;
        cursor[c] += 1;
    }
    let no_balde = |bx: i64, by: i64| -> &[P] {
        if bx < 0 || by < 0 || bx >= nbx || by >= nby {
            return &[];
        }
        let c = celula((bx, by));
        &itens[off[c] as usize..off[c + 1] as usize]
    };
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
                                for &p in no_balde(bx, by) {
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

/// A fusão gulosa de Hertel–Mehlhorn: tira as diagonais (as arestas interiores que não são parede),
/// da MAIS LONGA para a mais curta, sempre que o polígono que fica é ESTRITAMENTE convexo nas duas
/// pontas — a decisão é exacta (orientação inteira). A ordem é total (comprimento, depois os
/// índices) ⇒ a mesma entrada dá a mesma malha em toda máquina.
pub fn merge_convex(pts: &[P], tris: &[[u32; 3]]) -> Vec<Vec<u32>> {
    merge_convex_labeled(pts, tris, &vec![0; tris.len()]).0
}

/// (W7) [`merge_convex`] que só tira uma diagonal entre triângulos do MESMO pedaço (a fronteira de uma
/// área de custo fica aresta da malha) → os anéis e o pedaço de cada um.
pub fn merge_convex_labeled(
    pts: &[P],
    tris: &[[u32; 3]],
    labels: &[u16],
) -> (Vec<Vec<u32>>, Vec<u16>) {
    // (W12) Um anel só ganha memória própria quando funde; até lá é o triângulo (`vivo` e vazio).
    let mut rings: Vec<Vec<u32>> = vec![Vec::new(); tris.len()];
    let mut vivo = vec![true; tris.len()];
    // As semi-arestas `(u, w, triângulo)`, ordenadas; numa repetida vale a do ÚLTIMO triângulo (W12: era
    // um `BTreeMap` com `insert`, `0,053 ms` da fusão — o oráculo em `triangulate_oraculo.rs`).
    let mut semi: Vec<(u32, u32, u32)> = Vec::with_capacity(3 * tris.len());
    for (ti, t) in tris.iter().enumerate() {
        for i in 0..3 {
            semi.push((t[i], t[(i + 1) % 3], ti as u32));
        }
    }
    semi.sort_unstable();
    let ultima =
        |k: usize| k + 1 == semi.len() || (semi[k + 1].0, semi[k + 1].1) != (semi[k].0, semi[k].1);
    let mut diags: Vec<(i128, u32, u32, usize, usize)> = Vec::new();
    for (k, &(u, w, ta)) in semi.iter().enumerate() {
        if u < w && ultima(k) {
            let fim = semi.partition_point(|x| (x.0, x.1) <= (w, u));
            if fim > 0 && (semi[fim - 1].0, semi[fim - 1].1) == (w, u) {
                let tb = semi[fim - 1].2;
                diags.push((
                    len2(pts[u as usize], pts[w as usize]),
                    u,
                    w,
                    ta as usize,
                    tb as usize,
                ));
            }
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
        let anel = |i: usize| -> &[u32] {
            if rings[i].is_empty() {
                &tris[i]
            } else {
                &rings[i]
            }
        };
        if let Some(m) = try_merge(pts, anel(a), anel(b), u, w) {
            rings[a] = m;
            rings[b] = Vec::new();
            vivo[b] = false;
            parent[b] = a;
        }
    }
    rings
        .into_iter()
        .enumerate()
        .zip(labels.iter().copied())
        .filter(|((i, _), _)| vivo[*i])
        .map(|((i, r), l)| (if r.is_empty() { tris[i].to_vec() } else { r }, l))
        .unzip()
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
    // m = ra a partir de y, até x (inclusive): y, a1, …, x; e o interior de rb a partir de x: b1..bk.
    // (W12) Decide-se ANTES de copiar: a maioria das tentativas é recusada.
    let m_k = |k: usize| ra[(i + 1 + k) % na];
    let interior = |k: usize| rb[(j + 1 + k) % nb];
    // Convexidade em x: anterior = o que vem antes de x em m (m[na-2]), seguinte = b1 (ou y).
    let (after_x, before_y) = if nb > 2 {
        (interior(1), interior(nb - 2))
    } else {
        (y, x)
    };
    let px = pts[x as usize];
    let py = pts[y as usize];
    let prev_x = pts[m_k(na - 2) as usize];
    let next_y = pts[m_k(1 % na) as usize];
    if orient(prev_x, px, pts[after_x as usize]) <= 0 {
        return None;
    }
    if orient(pts[before_y as usize], py, next_y) <= 0 {
        return None;
    }
    let mut m: Vec<u32> = Vec::with_capacity(na + nb - 2);
    m.extend((0..na).map(m_k));
    m.extend((1..nb - 1).map(interior));
    Some(m)
}

#[cfg(test)]
#[path = "triangulate_oraculo.rs"]
mod oraculo;
