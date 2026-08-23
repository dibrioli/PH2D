//! ⭐⭐⭐ **G1 — A MALHA CORTADA:** cada patch passa a ter os seus próprios vértices, e
//! as costuras ficam numa tabela explícita.
//!
//! # ⚠️⚠️ Por que um mapa `global → local` NÃO chega
//!
//! A construção óbvia — «por patch, um `BTreeMap` de vértice global para índice
//! local» — é a que o [`ph2d_quadfill::param`] usa, e ela **não corta**. ⛔ O traçado
//! abre um patch-anel com uma **ponte**: um arco que o **mesmo** patch percorre dos
//! dois lados. Um mapa por vértice dá a esse vértice **um** índice local, e o anel
//! continua anel — *a fase seguinte resolveria um domínio que não é um disco, e o
//! `(u, v)` não fecharia.*
//!
//! ⭐ **O corte certo é por SECTOR do leque.** Dois cantos no mesmo vértice global e no
//! mesmo patch são a mesma cópia **se e só se** se chega de um ao outro andando por
//! faces daquele patch **sem atravessar uma aresta de arco**. É isso que a união-busca
//! abaixo calcula, e é a diferença entre cortar e não cortar.
//!
//! # ⭐⭐ A régua: cada patch cortado tem de ser um DISCO
//!
//! `χ = V − E + F` sobre cada patch cortado vale **`1`** exactamente quando ele é um
//! disco. ⛔ *Sub-cortar* deixa um anel (`χ = 0`); *sobre-cortar* parte-o em dois
//! (`χ = 2`). **Uma régua apanha os dois erros**, e é por isso que ela é esta e não uma
//! contagem de vértices — ver [`CutReport::discs`].

use std::collections::{BTreeMap, BTreeSet};

use ph2d_mesh::Mesh;
use ph2d_trace::PatchLayout;

/// Um lado de uma costura.
#[derive(Debug, Clone)]
pub struct SeamSide {
    /// O patch deste lado. ⚠️ **Pode ser o MESMO nos dois lados** — é exactamente o
    /// caso da ponte que abre um anel, e não é um erro.
    pub patch: u32,
    /// Por posição na cadeia canónica do arco, o vértice **local** deste lado.
    ///
    /// ⚠️ `None` numa posição significa que aquele vértice não foi alcançado por
    /// nenhuma face deste lado — *é uma resposta e não um zero*, e o
    /// [`CutReport::orphan_seam_vertices`] conta-a.
    pub local: Vec<Option<u32>>,
}

/// **UMA COSTURA** — uma cadeia de vértices, e os dois lados que a partilham.
#[derive(Debug, Clone)]
pub struct Seam {
    /// O arco no [`PatchLayout::arc_chain`], ou **`None`** quando é um corte que esta
    /// fase abriu para tornar um patch num disco — ver [`CutReport::opened`].
    ///
    /// ⚠️ **Os dois casos têm de ser distinguíveis:** um corte interno não tem
    /// quantização atrás dele, então a fase que lê isto não lhe pode pedir um número
    /// inteiro de segmentos. *Guardá-los sem distinção seria pedi-lo.*
    pub arc: Option<u32>,
    /// A cadeia de vértices **globais**, na ordem canónica.
    ///
    /// ⭐ *A costura traz a própria cadeia* — quem a lê não precisa de voltar ao
    /// layout, e um corte interno não tem lá entrada nenhuma.
    pub chain: Vec<u32>,
    /// Os dois lados.
    pub side: [SeamSide; 2],
}

/// A malha cortada: um disco por patch, mais as costuras.
#[derive(Debug, Clone, Default)]
pub struct CutMesh {
    /// Por patch, por vértice local, o vértice **global** de que ele é cópia.
    ///
    /// ⭐ *É o único sítio onde a identidade original sobrevive*, e é por ele que a
    /// fase seguinte lê posições e escreve de volta.
    pub origin: Vec<Vec<u32>>,
    /// Por patch, os triângulos em índices locais.
    pub tris: Vec<Vec<[u32; 3]>>,
    /// Por patch, a face da malha de que cada triângulo veio — o índice com que se lê
    /// o campo (`PatchLayout::face_dir`).
    pub tri_face: Vec<Vec<u32>>,
    /// As costuras, uma por arco de fronteira interior.
    pub seams: Vec<Seam>,
}

/// O que o corte mediu de si próprio.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct CutReport {
    /// Quantos patches saíram.
    pub patches: usize,
    /// ⭐⭐⭐ **Quantos deles são DISCOS** (`χ = 1`). Igual a [`Self::patches`] é o
    /// único resultado bom.
    pub discs: usize,
    /// Quantos saíram com `χ ≠ 1`, por classe: `0` anel ou pior (`χ ≤ 0`) · `1`
    /// partido em dois ou mais (`χ ≥ 2`).
    pub not_discs: [usize; 2],
    /// Quantos vértices a malha cortada tem ao todo. ⚠️ **Nunca menor que o original**
    /// — cortar duplica, nunca funde.
    pub verts: usize,
    /// Quantas faces entraram. ⛔ Tem de bater com a malha: uma face perdida é um
    /// patch a menos algures, sem erro nenhum a acusar.
    pub faces: usize,
    /// ⚠️ Faces que não são triângulos e ficaram **de fora**.
    pub non_tris: usize,
    /// Arcos que só têm **um** lado (fronteira da peça, ou um arco que nenhuma face
    /// reclama). *Não entram em [`CutMesh::seams`].*
    pub open_arcs: usize,
    /// ⚠️ Posições de cadeia que nenhum lado alcançou — ver [`SeamSide::local`].
    pub orphan_seam_vertices: usize,
    /// ⭐⭐⭐ **Quantos patches esta fase teve de ABRIR** por conta própria, por não
    /// virem discos do traçado.
    ///
    /// ⛔ **Medido 2026-08-23:** o toro `32×16` entrega um patch com `χ = 0` — 666
    /// faces, 16 arcos e **nenhum arco repetido**, ou seja o F3 nunca lhe construiu a
    /// ponte. *A dívida é do traçado; abri-lo aqui é o contrato desta fase, não uma
    /// desculpa para a dívida.*
    pub opened: usize,
    /// ⛔ Patches que ficaram sem ser discos **mesmo depois** de abertos. `> 0` é um
    /// resultado vermelho e a fase seguinte não os pode parametrizar.
    pub unopened: usize,
}

/// União-busca com compressão de caminho — o suficiente para juntar cantos.
struct Find(Vec<u32>);

impl Find {
    fn new(n: usize) -> Self {
        #[allow(clippy::cast_possible_truncation)]
        Self((0..n as u32).collect())
    }
    fn root(&mut self, mut x: u32) -> u32 {
        while self.0[x as usize] != x {
            let up = self.0[x as usize];
            self.0[x as usize] = self.0[up as usize];
            x = self.0[x as usize];
        }
        x
    }
    fn join(&mut self, a: u32, b: u32) {
        let (ra, rb) = (self.root(a), self.root(b));
        if ra != rb {
            self.0[ra as usize] = rb;
        }
    }
}

fn key(a: u32, b: u32) -> (u32, u32) {
    (a.min(b), a.max(b))
}

/// Um patch já cortado, em índices locais.
struct Built {
    origin: Vec<u32>,
    tris: Vec<[u32; 3]>,
    /// `(face, ranhura) -> índice local`.
    corner: BTreeMap<(u32, u32), u32>,
    /// `χ = V − E + F`. **`1` é o disco.**
    chi: i64,
}

/// Constrói um patch cortando ao longo de `blocked`.
///
/// ⭐ **É chamada MAIS DO QUE UMA VEZ** — abrir um anel é acrescentar arestas a
/// `blocked` e voltar a chamar. *A união-busca não precisa de saber a diferença entre
/// uma aresta de arco e um corte que esta fase inventou.*
fn build_patch(mesh: &Mesh, faces: &[u32], blocked: &BTreeSet<(u32, u32)>) -> Built {
    let mut find = Find::new(faces.len() * 3);
    let mut owners: BTreeMap<(u32, u32), Vec<usize>> = BTreeMap::new();
    for (i, &f) in faces.iter().enumerate() {
        let v = mesh.faces()[f as usize].verts();
        for k in 0..3 {
            owners.entry(key(v[k], v[(k + 1) % 3])).or_default().push(i);
        }
    }
    // ⭐⭐⭐ **A JUNÇÃO, e é a linha que corta:** dois cantos no mesmo vértice global
    // fundem-se **só** se a aresta que liga as duas faces não estiver bloqueada. *Um
    // `BTreeMap` de vértice global para índice local fundiria-os sempre.*
    for (&(a, b), sites) in &owners {
        if blocked.contains(&(a, b)) || sites.len() != 2 {
            continue;
        }
        let (i, j) = (sites[0], sites[1]);
        let vi = mesh.faces()[faces[i] as usize].verts();
        let vj = mesh.faces()[faces[j] as usize].verts();
        for g in [a, b] {
            let (Some(ci), Some(cj)) = (
                vi.iter().position(|&x| x == g),
                vj.iter().position(|&x| x == g),
            ) else {
                continue;
            };
            #[allow(clippy::cast_possible_truncation)]
            find.join((i * 3 + ci) as u32, (j * 3 + cj) as u32);
        }
    }

    let mut slot: BTreeMap<u32, u32> = BTreeMap::new();
    let mut origin: Vec<u32> = Vec::new();
    let mut tris: Vec<[u32; 3]> = Vec::with_capacity(faces.len());
    let mut corner: BTreeMap<(u32, u32), u32> = BTreeMap::new();
    for (i, &f) in faces.iter().enumerate() {
        let v = mesh.faces()[f as usize].verts();
        let mut t = [0u32; 3];
        for k in 0..3 {
            #[allow(clippy::cast_possible_truncation)]
            let r = find.root((i * 3 + k) as u32);
            let next = u32::try_from(origin.len()).unwrap_or(u32::MAX);
            let l = *slot.entry(r).or_insert_with(|| {
                origin.push(v[k]);
                next
            });
            t[k] = l;
            #[allow(clippy::cast_possible_truncation)]
            corner.insert((f, k as u32), l);
        }
        tris.push(t);
    }

    let mut edges: BTreeSet<(u32, u32)> = BTreeSet::new();
    for t in &tris {
        for k in 0..3 {
            edges.insert(key(t[k], t[(k + 1) % 3]));
        }
    }
    #[allow(clippy::cast_possible_wrap)]
    let chi = origin.len() as i64 - edges.len() as i64 + tris.len() as i64;
    Built {
        origin,
        tris,
        corner,
        chi,
    }
}

/// As voltas de bordo de um patch já cortado, em vértices **globais**.
///
/// ⚠️ Uma aresta é de bordo quando **uma só** face do patch a usa. Um disco tem uma
/// volta; um anel tem duas — e é a existência da segunda que dá por onde cortar.
fn boundary_loops(built: &Built) -> Vec<BTreeSet<u32>> {
    let mut count: BTreeMap<(u32, u32), usize> = BTreeMap::new();
    for t in &built.tris {
        for k in 0..3 {
            *count.entry(key(t[k], t[(k + 1) % 3])).or_default() += 1;
        }
    }
    let mut adj: BTreeMap<u32, Vec<u32>> = BTreeMap::new();
    for (&(a, b), &n) in &count {
        if n == 1 {
            adj.entry(a).or_default().push(b);
            adj.entry(b).or_default().push(a);
        }
    }
    // Componentes ligadas do grafo de bordo — cada uma é uma volta.
    let mut seen: BTreeSet<u32> = BTreeSet::new();
    let mut out: Vec<BTreeSet<u32>> = Vec::new();
    for &start in adj.keys() {
        if !seen.insert(start) {
            continue;
        }
        let mut loop_verts = BTreeSet::from([built.origin[start as usize]]);
        let mut queue = std::collections::VecDeque::from([start]);
        while let Some(x) = queue.pop_front() {
            for &y in adj.get(&x).into_iter().flatten() {
                if seen.insert(y) {
                    loop_verts.insert(built.origin[y as usize]);
                    queue.push_back(y);
                }
            }
        }
        out.push(loop_verts);
    }
    out
}

/// O caminho de vértices **globais** mais curto (em nº de arestas) de `from` até
/// `to`, andando só por arestas de `faces`.
///
/// ⚠️ **Devolve a cadeia inteira**, pontas incluídas — é ela que vai ser cortada.
fn path_between(
    mesh: &Mesh,
    faces: &[u32],
    from: &BTreeSet<u32>,
    to: &BTreeSet<u32>,
) -> Option<Vec<u32>> {
    let mut adj: BTreeMap<u32, BTreeSet<u32>> = BTreeMap::new();
    for &f in faces {
        let v = mesh.faces()[f as usize].verts();
        for k in 0..3 {
            let (a, b) = (v[k], v[(k + 1) % 3]);
            adj.entry(a).or_default().insert(b);
            adj.entry(b).or_default().insert(a);
        }
    }
    let mut prev: BTreeMap<u32, u32> = BTreeMap::new();
    let mut seen: BTreeSet<u32> = from.iter().copied().collect();
    let mut queue: std::collections::VecDeque<u32> = from.iter().copied().collect();
    let mut hit = None;
    while let Some(x) = queue.pop_front() {
        if to.contains(&x) && !from.contains(&x) {
            hit = Some(x);
            break;
        }
        for &y in adj.get(&x).into_iter().flatten() {
            if seen.insert(y) {
                prev.insert(y, x);
                queue.push_back(y);
            }
        }
    }
    let mut cur = hit?;
    let mut chain = vec![cur];
    while let Some(&p) = prev.get(&cur) {
        chain.push(p);
        cur = p;
    }
    chain.reverse();
    (chain.len() >= 2).then_some(chain)
}

/// ⭐⭐⭐ **CORTA A MALHA AO LONGO DAS FRONTEIRAS DE PATCH.**
///
/// ⚠️ **A malha tem de vir triangulada** — faces com outra contagem ficam de fora e
/// aparecem em [`CutReport::non_tris`]. *Contá-las é o que impede «o patch saiu
/// pequeno» de se ler como geometria.*
#[must_use]
pub fn cut_along_patches(mesh: &Mesh, layout: &PatchLayout) -> (CutMesh, CutReport) {
    let patches = layout.side_arcs.len();
    let mut rep = CutReport {
        patches,
        ..CutReport::default()
    };

    // ── As arestas que são de ARCO. É atravessá-las que se proíbe.
    let mut seam_edges: BTreeSet<(u32, u32)> = BTreeSet::new();
    for chain in &layout.arc_chain {
        for w in chain.windows(2) {
            seam_edges.insert(key(w[0], w[1]));
        }
    }

    // ── Os triângulos de cada patch, e a face de origem de cada um.
    let mut faces_of: Vec<Vec<u32>> = vec![Vec::new(); patches];
    for (f, &p) in layout.face_patch.iter().enumerate() {
        let Some(face) = mesh.faces().get(f) else {
            continue;
        };
        if face.verts().len() != 3 {
            rep.non_tris += 1;
            continue;
        }
        if let (Some(slot), Ok(f)) = (faces_of.get_mut(p as usize), u32::try_from(f)) {
            slot.push(f);
            rep.faces += 1;
        }
    }

    // ── Quem está de que lado de cada aresta de arco: `(aresta) -> [(patch, face)]`.
    let mut across: BTreeMap<(u32, u32), Vec<(u32, u32)>> = BTreeMap::new();

    let mut out = CutMesh {
        origin: Vec::with_capacity(patches),
        tris: Vec::with_capacity(patches),
        tri_face: Vec::with_capacity(patches),
        seams: Vec::new(),
    };
    // Por patch: `(face, ranhura) -> índice local`, para a tabela de costuras.
    let mut corner_local: Vec<BTreeMap<(u32, u32), u32>> = vec![BTreeMap::new(); patches];

    // ⭐⭐⭐ **QUANTAS VEZES SE TENTA ABRIR UM PATCH.** Cada ronda liga duas voltas de
    // bordo, logo baixa o género/nº de furos em UM. ⚠️ O teto é uma rede contra um
    // caso que não fecha, e ele CONTA-SE (`unopened`) em vez de ficar em silêncio.
    const OPEN_TRIES: usize = 8;

    for (p, faces) in faces_of.iter().enumerate() {
        // Que faces DESTE patch tocam cada aresta — serve as duas tabelas de costura.
        let mut owners_of: BTreeMap<(u32, u32), Vec<u32>> = BTreeMap::new();
        for &f in faces {
            let v = mesh.faces()[f as usize].verts();
            for k in 0..3 {
                owners_of
                    .entry(key(v[k], v[(k + 1) % 3]))
                    .or_default()
                    .push(f);
            }
        }
        // As arestas por onde este patch NÃO pode fundir: as de arco, mais os cortes
        // que esta fase abrir.
        let mut blocked = seam_edges.clone();
        let mut cuts: Vec<Vec<u32>> = Vec::new();
        let mut built = build_patch(mesh, faces, &blocked);
        for _ in 0..OPEN_TRIES {
            if built.chi == 1 {
                break;
            }
            let loops = boundary_loops(&built);
            if loops.len() < 2 {
                break;
            }
            let Some(chain) = path_between(mesh, faces, &loops[0], &loops[1]) else {
                break;
            };
            let mut next_blocked = blocked.clone();
            for w in chain.windows(2) {
                next_blocked.insert(key(w[0], w[1]));
            }
            let next = build_patch(mesh, faces, &next_blocked);
            // ⛔⛔ **A MELHORIA TEM DE SER ESTRITA.** Um corte que não sobe o `χ` não
            // abriu nada e só acrescentou costura — *e um laço que aceita o empate
            // corta a peça toda sem nunca fechar*. É a mesma guarda que a ponte do
            // traçado já usa.
            if next.chi <= built.chi {
                break;
            }
            blocked = next_blocked;
            built = next;
            cuts.push(chain);
        }

        if !cuts.is_empty() {
            rep.opened += 1;
        }
        if built.chi == 1 {
            rep.discs += 1;
        } else {
            rep.unopened += 1;
            if built.chi <= 0 {
                rep.not_discs[0] += 1;
            } else {
                rep.not_discs[1] += 1;
            }
        }
        rep.verts += built.origin.len();

        // ── As costuras INTERNAS deste patch, lidas com a mesma lei das de arco.
        #[allow(clippy::cast_possible_truncation)]
        let pid = p as u32;
        for chain in cuts {
            let mut side: [Vec<Option<u32>>; 2] =
                [vec![None; chain.len()], vec![None; chain.len()]];
            for (i, w) in chain.windows(2).enumerate() {
                let Some(sites) = owners_of.get(&key(w[0], w[1])) else {
                    continue;
                };
                for (s, &f) in sites.iter().take(2).enumerate() {
                    let v = mesh.faces()[f as usize].verts();
                    for (g, pos) in [(w[0], i), (w[1], i + 1)] {
                        let Some(k) = v.iter().position(|&x| x == g) else {
                            continue;
                        };
                        #[allow(clippy::cast_possible_truncation)]
                        if let Some(slot) = side[s].get_mut(pos) {
                            *slot = built.corner.get(&(f, k as u32)).copied();
                        }
                    }
                }
            }
            out.seams.push(Seam {
                arc: None,
                chain,
                side: [
                    SeamSide {
                        patch: pid,
                        local: side[0].clone(),
                    },
                    SeamSide {
                        patch: pid,
                        local: side[1].clone(),
                    },
                ],
            });
        }

        for (&(a, b), sites) in &owners_of {
            if seam_edges.contains(&(a, b)) {
                for &f in sites {
                    if layout.face_patch.get(f as usize).copied() == Some(pid) {
                        across.entry((a, b)).or_default().push((pid, f));
                    }
                }
            }
        }

        corner_local[p] = built.corner;
        out.origin.push(built.origin);
        out.tris.push(built.tris);
        out.tri_face.push(faces.clone());
    }

    // ── ⭐ A TABELA DE COSTURAS, lida das faces que ladeiam cada aresta de arco.
    for (a, chain) in layout.arc_chain.iter().enumerate() {
        let Ok(arc) = u32::try_from(a) else {
            continue;
        };
        // Os dois lados do arco: patch + a face que lhe toca, por aresta da cadeia.
        let mut sides: Vec<(u32, Vec<Option<u32>>)> = Vec::new();
        for w in chain.windows(2) {
            let Some(here) = across.get(&key(w[0], w[1])) else {
                continue;
            };
            for (slot_i, &(p, f)) in here.iter().enumerate() {
                if sides.len() <= slot_i {
                    sides.push((p, vec![None; chain.len()]));
                }
                let Some((sp, marks)) = sides.get_mut(slot_i) else {
                    continue;
                };
                *sp = p;
                let v = mesh.faces()[f as usize].verts();
                for g in [w[0], w[1]] {
                    let Some(k) = v.iter().position(|&x| x == g) else {
                        continue;
                    };
                    let Some(pos) = chain.iter().position(|&x| x == g) else {
                        continue;
                    };
                    #[allow(clippy::cast_possible_truncation)]
                    let l = corner_local
                        .get(p as usize)
                        .and_then(|m| m.get(&(f, k as u32)).copied());
                    if let Some(m) = marks.get_mut(pos) {
                        *m = l;
                    }
                }
            }
        }
        if sides.len() < 2 {
            rep.open_arcs += 1;
            continue;
        }
        for (_, marks) in sides.iter().take(2) {
            rep.orphan_seam_vertices += marks.iter().filter(|m| m.is_none()).count();
        }
        let mut it = sides.into_iter();
        let (p0, l0) = it.next().unwrap_or((0, Vec::new()));
        let (p1, l1) = it.next().unwrap_or((0, Vec::new()));
        out.seams.push(Seam {
            arc: Some(arc),
            chain: chain.clone(),
            side: [
                SeamSide {
                    patch: p0,
                    local: l0,
                },
                SeamSide {
                    patch: p1,
                    local: l1,
                },
            ],
        });
    }

    (out, rep)
}

#[cfg(test)]
#[path = "cut_tests.rs"]
mod tests;
