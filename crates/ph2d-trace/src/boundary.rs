//! **A FRONTEIRA DE UM PATCH e os CANTOS dela** — o passeio que transforma um
//! conjunto de faces numa palavra de arcos.
//!
//! ⭐⭐ **O corte contra o `patches` é de ASSUNTO, e foi forçado pela HR-18**
//! (726 contra 700): lá mora **o que se recorta** (o flood, a dissolução, a lista de
//! degenerados); aqui **como se lê a borda do que foi recortado**.
//!
//! ⚠️ **É aqui que mora a decisão mais cara desta fase:** o que conta como
//! **fronteira** decide se uma parede interior a um patch é vista ou ignorada — e foi
//! ignorá-la que fez um toro sair sem o buraco (`PLAN.md` §4-sexvicies). O parâmetro
//! `cut_open` do `boundary_loops` é essa decisão, e quem a toma é o laço de limpeza.

use std::collections::{BTreeMap, BTreeSet};

use ph2d_mesh::Mesh;

use super::patches::FLAT_QUARTERS;
use crate::walk::Walls;

/// **OS LAÇOS DE FRONTEIRA de um patch**, percorridos por pivô.
pub(crate) fn boundary_loops(
    faces: &[ph2d_mesh::Face],
    half: &BTreeMap<(u32, u32), u32>,
    face_patch: &[u32],
    walls: &Walls,
    p: u32,
    cut_open: bool,
) -> Vec<Vec<u32>> {
    // As meias-arestas de fronteira: a face de dentro é do patch, a de fora não.
    //
    // ⭐⭐ **Com `cut_open`, a segunda condição cai.** Uma parede com o mesmo patch
    // dos dois lados — a ponte que liga as duas fronteiras de um anel — passa a ser
    // percorrida **dos dois lados**, que é a representação *"cortar e abrir"*: o anel
    // vira um disco cuja palavra de fronteira atravessa a ponte duas vezes.
    //
    // ⛔ **Ela NÃO se liga em todo patch**, e a razão é medida: no toro 32×16 isso
    // levava o complexo de `0` para `−1`. Quem decide é o laço de limpeza, sob a
    // mesma guarda de sempre — *a ponte é uma cura, e uma cura que piora a topologia
    // é recusada.*
    let outside = |a: u32, b: u32| -> bool {
        walls.blocks(a, b)
            && (cut_open
                || half
                    .get(&(b, a))
                    .is_none_or(|&g| face_patch[g as usize] != p))
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
pub(crate) fn after(faces: &[ph2d_mesh::Face], f: u32, v: u32) -> u32 {
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
pub(crate) fn quarters(
    mesh: &Mesh,
    faces: &[ph2d_mesh::Face],
    face_patch: &[u32],
    p: u32,
    v: u32,
) -> i32 {
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
pub(crate) fn is_corner(
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
pub(crate) const MIN_PATCH_CORNERS: usize = 3;

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
pub(crate) fn promote(
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
pub(crate) fn angle(o: [f32; 3], a: [f32; 3], b: [f32; 3]) -> f32 {
    let u = norm(sub(a, o));
    let w = norm(sub(b, o));
    let c = (u[0].mul_add(w[0], u[1].mul_add(w[1], u[2] * w[2]))).clamp(-1.0, 1.0);
    c.acos()
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

pub(crate) fn dist(a: [f32; 3], b: [f32; 3]) -> f32 {
    let d = sub(a, b);
    d[0].mul_add(d[0], d[1].mul_add(d[1], d[2] * d[2])).sqrt()
}
