//! **DO TRAÇADO PARA OS PATCHES** — recortar, achar cantos, cortar em arcos.
//!
//! # Os quatro passos
//!
//! 1. **Inundação**: duas faces são do mesmo patch se a aresta entre elas não é
//!    parede.
//! 2. **Fronteira**: percorrida **pivotando em torno do vértice dentro do
//!    patch**, nunca saltando de aresta em aresta.
//! 3. **Cantos**: o **ângulo interno daquele patch naquele vértice**.
//! 4. **Arcos**: a fronteira partida em todo vértice que seja canto de **algum**
//!    patch — é isso que faz nascer a junção em T.
//!
//! ⚠️ **O passo 2 é onde a versão ingénua se parte.** Andar de aresta em aresta
//! escolhendo *"a próxima que ainda não usei"* é ambíguo num vértice onde quatro
//! arestas de fronteira se cruzam, e ali o laço salta para o patch errado — sem
//! erro nenhum, só com uma fronteira que descreve outra coisa. Pivotar dentro do
//! patch não tem essa escolha para fazer.

use std::collections::{BTreeMap, BTreeSet, VecDeque};

use ph2d_mesh::Mesh;

use crate::TraceReport;
use crate::walk::Walls;

/// ⚠️ **Quantos quartos de volta um vértice de fronteira tem de valer para NÃO
/// ser canto.** Um lado reto passa `180°`, que são **dois** quartos; qualquer
/// outra contagem é uma quina.
const FLAT_QUARTERS: i32 = 2;

/// **A DECOMPOSIÇÃO** — o que o F4 vai consumir.
#[derive(Debug, Clone, PartialEq)]
pub struct PatchLayout {
    /// Por face, o patch a que ela pertence.
    pub face_patch: Vec<u32>,
    /// Por patch, por lado, os arcos que o compõem: o id e **se o lado o
    /// percorre ao contrário** da ordem canónica do arco.
    ///
    /// ⚠️ **O sentido é obrigatório e não é decoração.** Os dois patches que
    /// partilham um arco percorrem-no em sentidos opostos; sem o registar, o F5
    /// amostraria os pontos de um deles ao contrário e os dois lados da divisa
    /// deixariam de coincidir — a malha sairia **rasgada** ao longo de cada
    /// fronteira de patch, sem nenhum erro a acusar.
    pub side_arcs: Vec<Vec<Vec<(u32, bool)>>>,
    /// Por arco, a cadeia ORDENADA de vértices de malha, na ordem canónica.
    pub arc_chain: Vec<Vec<u32>>,
    /// Por patch, os vértices-canto, na ordem da fronteira.
    pub corners: Vec<Vec<u32>>,
    /// Por arco, o comprimento geométrico.
    pub arc_length: Vec<f32>,
    /// Por arco, as arestas de malha que o compõem. ⚠️ É o que permite **desfazer**
    /// uma parede: sem isto, um patch degenerado só se pode contar, não curar.
    pub arc_edges: Vec<BTreeSet<(u32, u32)>>,
    /// O que o traçado e a decomposição mediram.
    pub report: TraceReport,
}

/// **RECORTA a malha nas paredes** e devolve a decomposição.
#[must_use]
pub fn decompose(mesh: &Mesh, walls: &Walls, mut report: TraceReport) -> PatchLayout {
    let faces = mesh.faces();
    let pos = mesh.positions();
    // Aresta dirigida -> face que a contém na sua orientação. Numa malha fechada
    // e coerentemente orientada, cada uma pertence a exatamente uma face.
    let mut half: BTreeMap<(u32, u32), u32> = BTreeMap::new();
    for (fi, f) in faces.iter().enumerate() {
        let v = f.verts();
        for k in 0..v.len() {
            half.insert(
                (v[k], v[(k + 1) % v.len()]),
                u32::try_from(fi).unwrap_or(u32::MAX),
            );
        }
    }

    let face_patch = flood(faces, walls, &half);
    let n_patches = face_patch
        .iter()
        .copied()
        .max()
        .map_or(0, |m| m as usize + 1);

    // As fronteiras, patch a patch.
    let loops: Vec<Vec<Vec<u32>>> = (0..n_patches)
        .map(|p| {
            boundary_loops(
                faces,
                &half,
                &face_patch,
                walls,
                u32::try_from(p).unwrap_or(0),
            )
        })
        .collect();

    // ⭐ **UM CANTO SÓ EXISTE ONDE A PAREDE SE RAMIFICA** — ver [`is_corner`].
    let branching = walls.branching();

    // ⚠️ **O canto é do par (patch, vértice)**, mas o CORTE em arcos é global: um
    // vértice que seja canto de qualquer patch parte a fronteira de todos os que
    // passam por ele. É isso que faz nascer a junção em T — o lado de um patch
    // com dois arcos, porque o vizinho tem um canto no meio dele.
    let mut any_corner: BTreeSet<u32> = BTreeSet::new();
    // ⚠️ **O canto PROMOVIDO é do patch que o promoveu, e só dele.** Ele parte a
    // fronteira de todos (é `any_corner`), mas só fecha um lado de quem precisou
    // dele — para o vizinho, a fronteira passa direito por ali. *É exactamente a
    // relação de uma junção em T, e por isso a lista é por patch.*
    let mut mine_corner: Vec<BTreeSet<u32>> = vec![BTreeSet::new(); n_patches];
    let mut promoted = 0usize;
    for (pi, ls) in loops.iter().enumerate() {
        let p = u32::try_from(pi).unwrap_or(0);
        for lp in ls {
            let structural: Vec<u32> = lp
                .iter()
                .copied()
                .filter(|&v| is_corner(mesh, faces, &face_patch, &branching, p, v))
                .collect();
            any_corner.extend(&structural);
            mine_corner[pi].extend(&structural);
            // ⭐ **A PROMOÇÃO — e ela é o degrau, não a regra.** Ver
            // [`MIN_PATCH_CORNERS`].
            if structural.len() < MIN_PATCH_CORNERS {
                let got = promote(
                    mesh,
                    faces,
                    &face_patch,
                    p,
                    lp,
                    MIN_PATCH_CORNERS - structural.len(),
                    &mut mine_corner[pi],
                );
                promoted += got;
            }
            any_corner.extend(mine_corner[pi].iter().copied());
        }
    }
    report.promoted = promoted;

    // Os arcos: a fronteira partida em TODO canto de QUALQUER patch.
    let mut arc_id: BTreeMap<BTreeSet<(u32, u32)>, u32> = BTreeMap::new();
    let mut arc_length: Vec<f32> = Vec::new();
    let mut arc_edges: Vec<BTreeSet<(u32, u32)>> = Vec::new();
    let mut arc_chain: Vec<Vec<u32>> = Vec::new();
    let mut side_arcs: Vec<Vec<Vec<(u32, bool)>>> = Vec::with_capacity(n_patches);
    let mut corners: Vec<Vec<u32>> = Vec::with_capacity(n_patches);
    for (p, ls) in loops.iter().enumerate() {
        if ls.len() != 1 {
            report.non_disk += 1;
        }
        let mut patch_sides: Vec<Vec<(u32, bool)>> = Vec::new();
        let mut patch_corners: Vec<u32> = Vec::new();
        let mut flat = 0usize;
        for lp in ls {
            let mine: Vec<bool> = lp.iter().map(|v| mine_corner[p].contains(v)).collect();
            let cuts: Vec<usize> = (0..lp.len())
                .filter(|&i| any_corner.contains(&lp[i]))
                .collect();
            if cuts.is_empty() {
                continue;
            }
            let mut open: Vec<(u32, bool)> = Vec::new();
            for k in 0..cuts.len() {
                let (i, j) = (cuts[k], cuts[(k + 1) % cuts.len()]);
                let chain = chain_between(lp, i, j);
                // ⚠️ **A chave é o CONJUNTO de arestas, não a sequência.** Os
                // dois patches que partilham um arco percorrem-no em sentidos
                // OPOSTOS (a fronteira de cada um roda no seu próprio sentido),
                // então a lista ordenada de um é a do outro ao contrário — e o
                // mesmo arco ganharia dois ids. O sintoma é `ArcUse { uses: 1 }`
                // no F4: um arco de bordo onde não há bordo nenhum.
                let key: BTreeSet<(u32, u32)> = chain
                    .windows(2)
                    .map(|w| (w[0].min(w[1]), w[0].max(w[1])))
                    .collect();
                let id = *arc_id.entry(key.clone()).or_insert_with(|| {
                    let len = chain
                        .windows(2)
                        .map(|w| dist(pos[w[0] as usize], pos[w[1] as usize]))
                        .sum();
                    arc_length.push(len);
                    arc_edges.push(key);
                    arc_chain.push(chain.clone());
                    u32::try_from(arc_length.len() - 1).unwrap_or(u32::MAX)
                });
                // ⚠️ O sentido: este lado percorre o arco na ordem canónica ou ao
                // contrário? A resposta é a ponta por onde ele entra.
                let reversed = arc_chain[id as usize].first() != chain.first();
                open.push((id, reversed));
                // O lado FECHA num canto DESTE patch, não num canto qualquer.
                if mine[j] {
                    patch_sides.push(std::mem::take(&mut open));
                    patch_corners.push(lp[j]);
                }
            }
            if !open.is_empty() {
                // O resto emenda no primeiro lado do mesmo laço.
                if let Some(first) = patch_sides.first_mut() {
                    let mut merged = std::mem::take(&mut open);
                    merged.append(first);
                    *first = merged;
                } else {
                    patch_sides.push(std::mem::take(&mut open));
                }
            }
            flat += lp.len();
        }
        let _ = flat;
        *report.valence.entry(patch_sides.len()).or_default() += 1;
        side_arcs.push(patch_sides);
        corners.push(patch_corners);
    }
    report.patches = n_patches;
    report.arcs = arc_length.len();

    PatchLayout {
        face_patch,
        side_arcs,
        arc_chain,
        corners,
        arc_length,
        arc_edges,
        report,
    }
}

impl PatchLayout {
    /// Por patch, por lado, só os **ids** de arco — o que o F4 consome.
    #[must_use]
    pub fn sides(&self) -> Vec<Vec<Vec<u32>>> {
        self.side_arcs
            .iter()
            .map(|sides| {
                sides
                    .iter()
                    .map(|side| side.iter().map(|&(a, _)| a).collect())
                    .collect()
            })
            .collect()
    }

    /// **OS PATCHES DEGENERADOS** — menos de três lados.
    ///
    /// ⚠️ **Um patch de dois lados não é um erro de contagem, é uma lasca**: duas
    /// separatrizes que correram quase juntas e prenderam uma tira entre elas. Ele
    /// não tem cantos que cheguem para a lei do F4 (`L_i = e_{i-1} + e_{i+1}` pede
    /// pelo menos três), e é por isso que o layout inteiro é recusado por causa de
    /// **um** deles.
    #[must_use]
    pub fn degenerate(&self) -> Vec<usize> {
        (0..self.side_arcs.len())
            .filter(|&p| self.side_arcs[p].len() < 3)
            .collect()
    }
}

/// **DISSOLVE** a parede que separa cada patch degenerado do vizinho mais barato.
///
/// ⚠️ **Apagar a parede é a cura certa, e apagar o patch não seria.** A lasca não
/// é uma região a mais na superfície: ela é uma parede a mais. Removida a parede,
/// as faces dela juntam-se ao vizinho na inundação seguinte e a contagem de
/// patches cai sozinha — nenhuma face muda de sítio, nenhuma geometria se toca.
///
/// Devolve `false` quando não havia nada para dissolver, que é o sinal de paragem.
#[must_use]
pub fn dissolve(walls: &mut Walls, layout: &PatchLayout, victims: &[usize]) -> bool {
    let mut removed = false;
    for &p in victims {
        // O lado mais CURTO: dissolver o maior mudaria a forma do vizinho muito
        // mais do que o necessário para a lasca desaparecer.
        let target = layout.side_arcs[p]
            .iter()
            .min_by(|a, b| {
                let la: f32 = a.iter().map(|&(i, _)| layout.arc_length[i as usize]).sum();
                let lb: f32 = b.iter().map(|&(i, _)| layout.arc_length[i as usize]).sum();
                la.total_cmp(&lb)
            })
            .cloned();
        let arcs: Vec<u32> = match target {
            Some(side) => side.into_iter().map(|(a, _)| a).collect(),
            // ⚠️ Sem lados nenhuns não há o que escolher: a fronteira inteira sai.
            // É o patch cuja fronteira não tinha um único canto — uma faixa
            // fechada, que não é disco e não tem lados por definição.
            None => (0..layout.arc_edges.len())
                .map(|i| u32::try_from(i).unwrap_or(0))
                .filter(|&i| layout.side_arcs[p].iter().flatten().any(|&(j, _)| j == i))
                .collect(),
        };
        for a in arcs {
            for e in &layout.arc_edges[a as usize] {
                removed |= walls.edges.remove(e);
            }
        }
    }
    removed
}

/// **INUNDAÇÃO** — faces vizinhas por uma aresta que não é parede são o mesmo
/// patch.
fn flood(faces: &[ph2d_mesh::Face], walls: &Walls, half: &BTreeMap<(u32, u32), u32>) -> Vec<u32> {
    let mut patch = vec![u32::MAX; faces.len()];
    let mut next = 0u32;
    let mut queue: VecDeque<u32> = VecDeque::new();
    for start in 0..faces.len() {
        if patch[start] != u32::MAX {
            continue;
        }
        patch[start] = next;
        queue.push_back(u32::try_from(start).unwrap_or(0));
        while let Some(f) = queue.pop_front() {
            let v = faces[f as usize].verts();
            for k in 0..v.len() {
                let (a, b) = (v[k], v[(k + 1) % v.len()]);
                if walls.blocks(a, b) {
                    continue;
                }
                // A face do outro lado é a que contém a aresta ao contrário.
                let Some(&g) = half.get(&(b, a)) else {
                    continue;
                };
                if patch[g as usize] == u32::MAX {
                    patch[g as usize] = next;
                    queue.push_back(g);
                }
            }
        }
        next += 1;
    }
    patch
}

/// **OS LAÇOS DE FRONTEIRA de um patch**, percorridos por pivô.
fn boundary_loops(
    faces: &[ph2d_mesh::Face],
    half: &BTreeMap<(u32, u32), u32>,
    face_patch: &[u32],
    walls: &Walls,
    p: u32,
) -> Vec<Vec<u32>> {
    // As meias-arestas de fronteira: a face de dentro é do patch, a de fora não.
    let outside = |a: u32, b: u32| -> bool {
        walls.blocks(a, b)
            && half
                .get(&(b, a))
                .is_none_or(|&g| face_patch[g as usize] != p)
    };
    let mut start_set: BTreeSet<(u32, u32)> = BTreeSet::new();
    for (fi, f) in faces.iter().enumerate() {
        if face_patch[fi] != p {
            continue;
        }
        let v = f.verts();
        for k in 0..v.len() {
            let (a, b) = (v[k], v[(k + 1) % v.len()]);
            if outside(a, b) {
                start_set.insert((a, b));
            }
        }
    }
    let mut unused = start_set.clone();
    let mut out = Vec::new();
    while let Some(&first) = unused.iter().next() {
        let mut lp: Vec<u32> = Vec::new();
        let mut e = first;
        for _ in 0..=start_set.len() {
            if !unused.remove(&e) {
                break;
            }
            lp.push(e.0);
            // Pivô em torno de `e.1`, por dentro do patch, até achar a próxima
            // aresta de fronteira.
            let mut cur = e;
            let mut found = None;
            for _ in 0..64 {
                let Some(&f) = half.get(&cur) else { break };
                let c = after(faces, f, cur.1);
                if outside(cur.1, c) {
                    found = Some((cur.1, c));
                    break;
                }
                cur = (c, cur.1);
            }
            let Some(nxt) = found else { break };
            if nxt == first {
                break;
            }
            e = nxt;
        }
        if !lp.is_empty() {
            out.push(lp);
        }
    }
    out
}

/// O vértice que vem depois de `v` na face `f`.
fn after(faces: &[ph2d_mesh::Face], f: u32, v: u32) -> u32 {
    let t = faces[f as usize].verts();
    t.iter()
        .position(|&x| x == v)
        .map_or(v, |k| t[(k + 1) % t.len()])
}

/// **QUANTOS QUARTOS DE VOLTA** o patch `p` ocupa em volta do vértice `v`.
///
/// ⚠️ **É a soma dos ângulos das faces DESTE patch em `v`**, e nada mais. Um lado
/// reto dá `≈180°` (dois quartos); uma quina dá `≈90°` (um); um vértice em que o
/// patch se dobra para trás dá três.
fn quarters(mesh: &Mesh, faces: &[ph2d_mesh::Face], face_patch: &[u32], p: u32, v: u32) -> i32 {
    let pos = mesh.positions();
    let adj = mesh.adjacency();
    let mut total = 0.0f32;
    for &f in adj.vert_faces.neighbours(v as usize) {
        if face_patch[f as usize] != p {
            continue;
        }
        let t = faces[f as usize].verts();
        let Some(k) = t.iter().position(|&x| x == v) else {
            continue;
        };
        let a = t[(k + t.len() - 1) % t.len()];
        let b = t[(k + 1) % t.len()];
        total += angle(pos[v as usize], pos[a as usize], pos[b as usize]);
    }
    #[allow(clippy::cast_possible_truncation)]
    let q = (total / core::f32::consts::FRAC_PI_2).round() as i32;
    q
}

/// **ESTE VÉRTICE É CANTO DO PATCH `p`?** — a estrutura decide, a geometria
/// desempata.
///
/// ⭐⭐ **A primeira porta é a RAMIFICAÇÃO, e ela é a correção de 2026-08-21.** Um
/// vértice no interior de uma separatriz (ramificação `2`) tem a fronteira do
/// patch a passar **direito** por ele — em termos de layout, uma separatriz é uma
/// linha da grade e não vira. Se ele ali virar, quem virou foi a POLILINHA sobre
/// as arestas da malha, não a estrutura.
///
/// ⚠️ **E isso não era detalhe: era o maior balde.** Censo dos cantos do layout
/// (`tests/corner_census.rs`), antes desta porta:
///
/// | malha | cantos | singularidade | junção | ⛔ **artefacto** |
/// |---|---|---|---|---|
/// | esfera 96×144 + F1 | 52 | 6 | 19 | **27** |
/// | toro 64×32 + F1 | 72 | 8 | 27 | **37** |
/// | esfera 98 k + F1 | 68 | 6 | 25 | **37** |
///
/// ⛔ **Os artefactos eram TODOS irregulares na malha final** — cada um é um
/// vértice de valência 3 que não corresponde a nada.
///
/// A segunda porta continua a ser o ângulo, e ela é necessária: numa junção em T
/// (ramificação `3`) os dois patches que ladeiam o pé têm quina e o terceiro —
/// o do lado de lá da parede que continua — tem a fronteira reta. *A estrutura
/// diz ONDE pode haver canto; a geometria diz para QUEM ele é.*
fn is_corner(
    mesh: &Mesh,
    faces: &[ph2d_mesh::Face],
    face_patch: &[u32],
    branching: &BTreeMap<u32, usize>,
    p: u32,
    v: u32,
) -> bool {
    branching.get(&v).copied().unwrap_or(0) > 2
        && quarters(mesh, faces, face_patch, p, v) != FLAT_QUARTERS
}

/// **QUANTOS CANTOS um patch precisa de ter, no mínimo.**
///
/// ⛔ **Ele é um piso de VALIDADE, e a tentação de o usar como alvo de QUALIDADE
/// foi construída, MEDIDA e rejeitada.** Um laço com menos de três cantos não
/// descreve um patch — e a limpeza de degenerados dissolvia-o em cascata: com a
/// porta estrutural sozinha, a esfera 24×36 colapsava de 14 patches para **1, com
/// zero arcos**.
///
/// ⚠️ **`3` e não `4`, e as duas razões saíram da mesma tabela.** Pedir quatro
/// parecia melhor — um patch de três lados produz, por construção, um irregular no
/// centro (o leque do F5 põe lá um vértice de valência igual à do patch). Medido
/// em 2026-08-21, com a cadeia completa e o mesmo alvo de densidade:
///
/// | malha | piso **4** | piso **3** |
/// |---|---|---|
/// | esfera 96×144 | 13 irreg. · 2 623 quads | 14 · **4 922** |
/// | toro 64×32 | 23 · 3 666 | 24 · **5 071** |
/// | esfera ruidosa | 20 · 3 949 | 20 · **4 503** |
/// | `cube` | 48 · 2 778 | 48 · **3 020** |
/// | esfera 98 k | ⛔ **o F4 recusa: `Infeasible`** | ✅ **21** · 5 978 |
/// | esfera sacudida | ⛔ **`Infeasible`** | ✅ **14** · 2 568 |
///
/// ⭐ **Duas leituras, e as duas contra o `4`:**
/// 1. ⛔ **Promover um quarto canto num patch que estruturalmente tem três torna o
///    sistema INVIÁVEL** — não é orçamento, é o fluxo a não fechar. Duas das seis
///    malhas do corpus deixavam de quantizar. *A promoção de mais 10 a 14 cantos
///    por malha impõe restrições que os arcos partilhados não conseguem satisfazer
///    ao mesmo tempo.*
/// 2. ⚠️ **E não comprava qualidade nenhuma:** a contagem de irregulares fica
///    dentro de **um** nas malhas que fechavam dos dois lados — e o piso 4
///    **distorcia a densidade**, entregando 2 623 quads onde o alvo pede ~5 000.
///
/// ⇒ *O que a estrutura não dá, a promoção não inventa.* O piso fica no mínimo que
/// a lei do F4 exige, e a dívida que sobra é do traçado.
const MIN_PATCH_CORNERS: usize = 3;

/// **PROMOVE os vértices que mais viram** até o laço ter cantos suficientes.
///
/// ⚠️ **É o degrau que a porta estrutural precisa, e ele é DELIBERADAMENTE o
/// segundo a falar.** A regra é a estrutura (uma parede que não se ramifica não
/// vira); esta função só existe para o caso em que a estrutura **não chega** para
/// o laço ser um patch. Cada promoção é um vértice irregular a mais na malha
/// final, e é por isso que ela é contada em [`crate::TraceReport::promoted`]:
/// *um remendo que ninguém conta vira a regra sem que ninguém decida.*
///
/// A ordem é por quanto o patch vira ali (`|quartos − 2|`), com o índice do
/// vértice a desempatar — sem o desempate a promoção dependeria da ordem de ponto
/// flutuante e a decomposição deixaria de ser reprodutível (HR-5).
fn promote(
    mesh: &Mesh,
    faces: &[ph2d_mesh::Face],
    face_patch: &[u32],
    p: u32,
    lp: &[u32],
    want: usize,
    any_corner: &mut BTreeSet<u32>,
) -> usize {
    let mut by_turn: Vec<(i32, u32)> = lp
        .iter()
        .filter(|v| !any_corner.contains(v))
        .map(|&v| {
            (
                -(quarters(mesh, faces, face_patch, p, v) - FLAT_QUARTERS).abs(),
                v,
            )
        })
        .collect();
    by_turn.sort_unstable();
    let taken = by_turn.into_iter().take(want);
    let mut n = 0usize;
    for (_, v) in taken {
        any_corner.insert(v);
        n += 1;
    }
    n
}

/// O ângulo em `o`, entre `a` e `b`.
fn angle(o: [f32; 3], a: [f32; 3], b: [f32; 3]) -> f32 {
    let u = norm(sub(a, o));
    let w = norm(sub(b, o));
    let c = (u[0].mul_add(w[0], u[1].mul_add(w[1], u[2] * w[2]))).clamp(-1.0, 1.0);
    c.acos()
}

/// A cadeia de vértices do laço, de `i` até `j`, dando a volta se preciso.
fn chain_between(lp: &[u32], i: usize, j: usize) -> Vec<u32> {
    let n = lp.len();
    let mut out = Vec::new();
    let mut t = i;
    loop {
        out.push(lp[t]);
        if t == j && out.len() > 1 {
            break;
        }
        t = (t + 1) % n;
        if out.len() > n {
            break;
        }
    }
    out
}

fn sub(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}

fn norm(v: [f32; 3]) -> [f32; 3] {
    let n = v[0].mul_add(v[0], v[1].mul_add(v[1], v[2] * v[2])).sqrt();
    if n > 1e-20 {
        [v[0] / n, v[1] / n, v[2] / n]
    } else {
        [1.0, 0.0, 0.0]
    }
}

fn dist(a: [f32; 3], b: [f32; 3]) -> f32 {
    let d = sub(a, b);
    d[0].mul_add(d[0], d[1].mul_add(d[1], d[2] * d[2])).sqrt()
}
