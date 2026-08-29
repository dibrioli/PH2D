//! ⭐⭐ **AS RÉGUAS DA TENTATIVA** — *«esta saída é pior que a anterior?»*
//!
//! Irmão de [`super::retopo_extract`] por RESPONSABILIDADE: ele decide **o que tentar**,
//! estas medem **o que saiu**. ⛔⛔ A chave da frente é [`open_edges`] (bordo **+**
//! não-manifold) e não só o bordo: em 2026-08-28 o ficheiro que o artista exportou tinha
//! `19 786` quads impecáveis com **`2` arestas não-manifold** num ponto só, e o veto não
//! as via — *«furo» contava metade*.

use ph2d_mesh::Mesh;

/// A aresta mediana e a mais longa da saída.
pub(super) fn edges(mesh: &Mesh) -> (f32, f32) {
    let pos = mesh.positions();
    let mut e: Vec<f32> = Vec::new();
    for f in mesh.faces() {
        let v = f.verts();
        for k in 0..v.len() {
            let (a, b) = (pos[v[k] as usize], pos[v[(k + 1) % v.len()] as usize]);
            let d = [a[0] - b[0], a[1] - b[1], a[2] - b[2]];
            e.push(d[0].mul_add(d[0], d[1].mul_add(d[1], d[2] * d[2])).sqrt());
        }
    }
    e.sort_by(f32::total_cmp);
    (
        e.get(e.len() / 2).copied().unwrap_or(0.0),
        e.last().copied().unwrap_or(0.0),
    )
}

/// Arestas com uma face só — a assinatura da casca aberta.
/// ⭐⭐⭐ **A ORDEM DA ESCOLHA entre duas tentativas — `true` se `a` é PIOR que `b`.**
///
/// **Furos, depois faces `>60°`, depois o enviesamento mediano.** ⚠️ Os furos vêm primeiro
/// porque são o que o artista **vê** — foi a queixa dele três vezes seguidas
/// (*«furos nas pontas»*). *Uma ordem que pusesse o enviesamento à frente escolheria a peça
/// mais bonita com um buraco na ponta.*
///
/// ⛔⛔ **E «furo» conta as DUAS formas de a casca não fechar, desde 2026-08-28.** Até essa
/// data esta ordem via só as arestas de **bordo**; uma aresta **não-manifold** — três faces a
/// tocá-la — passava invisível, e o campo alinhado produz exactamente isso (medido:
/// `sculpt_hooked`, `1` não-manifold contra `0` do liso, com o alinhado a ganhar por
/// `0,2°` de enviesamento). ⚠️ **O artista vê o mesmo entalhe escuro nos dois casos** — e o
/// ficheiro que ele exportou em 28/08 tinha `19 786` quads impecáveis com **`2` arestas
/// não-manifold** num ponto só, três vértices de valência `2`–`3`. *Uma chave de desempate
/// que não vê metade do defeito escolhe a peça furada com toda a razão do mundo.*
///
/// ⚠️ **O desempate final é por `total_cmp`** e não por `<`: um `NaN` numa das medianas
/// tornaria a comparação não-reflexiva e a escolha dependeria da ordem dos argumentos.
pub(super) fn worse(
    a_mesh: &Mesh,
    a_over60: usize,
    a_skew: f32,
    b_mesh: &Mesh,
    b_over60: usize,
    b_skew: f32,
) -> bool {
    let (a_holes, b_holes) = (open_edges(a_mesh), open_edges(b_mesh));
    if a_holes != b_holes {
        return a_holes > b_holes;
    }
    if a_over60 != b_over60 {
        return a_over60 > b_over60;
    }
    a_skew.total_cmp(&b_skew) == core::cmp::Ordering::Greater
}

pub(super) fn boundary_edges(mesh: &Mesh) -> usize {
    edge_census(mesh).0
}

/// ⭐⭐⭐ **AS DUAS FORMAS DE A CASCA NÃO FECHAR, somadas** — a chave da frente de [`worse`].
///
/// ⚠️ **Uma aresta de bordo e uma não-manifold dão o MESMO report** (*«furos»*), e nenhuma
/// régua desta linha as somava: a escolha entre tentativas via só a primeira.
pub(super) fn open_edges(mesh: &Mesh) -> usize {
    let (bordo, nm) = edge_census(mesh);
    bordo + nm
}

/// `(arestas de bordo, arestas não-manifold)` — uma face só, ou mais de duas.
pub(super) fn edge_census(mesh: &Mesh) -> (usize, usize) {
    use std::collections::BTreeMap;
    let mut n: BTreeMap<(u32, u32), usize> = BTreeMap::new();
    for f in mesh.faces() {
        let v = f.verts();
        for k in 0..v.len() {
            let (a, b) = (v[k], v[(k + 1) % v.len()]);
            *n.entry(if a < b { (a, b) } else { (b, a) }).or_default() += 1;
        }
    }
    (
        n.values().filter(|c| **c == 1).count(),
        n.values().filter(|c| **c > 2).count(),
    )
}

/// Vértices com valência diferente de 4 — a grandeza que o pivô existiu para
/// derrubar. ⭐ Uma grade numa esfera admite **oito**.
pub(super) fn irregular(mesh: &Mesh) -> usize {
    let mut deg = vec![0usize; mesh.vert_count()];
    use std::collections::BTreeSet;
    let mut seen: BTreeSet<(u32, u32)> = BTreeSet::new();
    for f in mesh.faces() {
        let v = f.verts();
        for k in 0..v.len() {
            let (a, b) = (v[k], v[(k + 1) % v.len()]);
            if seen.insert(if a < b { (a, b) } else { (b, a) }) {
                deg[a as usize] += 1;
                deg[b as usize] += 1;
            }
        }
    }
    deg.iter().filter(|d| **d != 4 && **d > 0).count()
}

/// **A DIAGONAL da caixa da peça** — o denominador da fração absoluta, e a mesma
/// régua do irmão.
pub(super) fn span(mesh: &Mesh) -> f32 {
    let b = mesh.bounds();
    let d = [
        b.max[0] - b.min[0],
        b.max[1] - b.min[1],
        b.max[2] - b.min[2],
    ];
    d[0].mul_add(d[0], d[1].mul_add(d[1], d[2] * d[2])).sqrt()
}
