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
///
/// ⛔⛔⛔ **E DESDE 2026-08-30 A SEGUNDA CHAVE É [`components`]** — o report do artista com
/// foto (*«péssimo»*, um quad a flutuar solto ao lado de uma ponta). ⚠️ **Nenhuma das duas
/// chaves que existiam o via:** um pedaço que se desprende sai **fechado**, logo `0` arestas
/// de bordo e `0` não-manifold, e o `open_edges` — que a nota acima chama de *«as DUAS formas
/// de a casca não fechar»* — dá **zero** nas duas peças. *Uma superfície fechada pode conter
/// uma segunda superfície fechada, e contar arestas nunca o revela.*
///
/// ⚠️ **A ORDEM é `furos → peças → >60° → enviesamento`, e o lugar da chave nova é uma
/// decisão, não um acaso:** os furos ficam à frente porque *foi isso que se mediu* (a queixa
/// do artista três vezes seguidas), e ⛔ **não existe medição nenhuma que ordene um estilhaço
/// contra um furo** — inventá-la aqui seria escolher por conforto. O estilhaço não precisa
/// de ganhar a chave da frente para ser apanhado: [`shattered`] veta-o **depois** da
/// escolha, e um veto absoluto não depende de ordenação nenhuma.
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
    let (a_parts, b_parts) = (components(a_mesh), components(b_mesh));
    if a_parts != b_parts {
        return a_parts > b_parts;
    }
    if a_over60 != b_over60 {
        return a_over60 > b_over60;
    }
    a_skew.total_cmp(&b_skew) == core::cmp::Ordering::Greater
}

/// ⭐⭐⭐ **EM QUANTAS PEÇAS a malha é** — componentes ligados por **ARESTA**.
///
/// ⚠️ **Por aresta e não por vértice, e a diferença é o que o artista vê:** dois sacos
/// fechados que se tocam num vértice só são, para quem olha, duas peças — a união por
/// vértice diria `1` e daria a peça partida por boa.
///
/// ⚠️ **Uma aresta não-manifold não parte nada:** as três faces que a partilham entram
/// todas no mesmo grupo (o mapa guarda a PRIMEIRA face de cada aresta, e as seguintes
/// unem-se a ela) — é por isso que esta régua e o [`open_edges`] medem coisas
/// independentes, e é por isso que as duas têm de existir.
pub(super) fn components(mesh: &Mesh) -> usize {
    use std::collections::{BTreeMap, BTreeSet};
    let n = mesh.face_count();
    if n == 0 {
        return 0;
    }
    fn find(parent: &mut [usize], mut x: usize) -> usize {
        while parent[x] != x {
            parent[x] = parent[parent[x]];
            x = parent[x];
        }
        x
    }
    let mut parent: Vec<usize> = (0..n).collect();
    let mut first: BTreeMap<(u32, u32), usize> = BTreeMap::new();
    for (fi, f) in mesh.faces().iter().enumerate() {
        let v = f.verts();
        for k in 0..v.len() {
            let (a, b) = (v[k], v[(k + 1) % v.len()]);
            let key = if a < b { (a, b) } else { (b, a) };
            if let Some(&other) = first.get(&key) {
                let (ra, rb) = (find(&mut parent, other), find(&mut parent, fi));
                if ra != rb {
                    parent[ra] = rb;
                }
            } else {
                first.insert(key, fi);
            }
        }
    }
    (0..n)
        .map(|i| find(&mut parent, i))
        .collect::<BTreeSet<_>>()
        .len()
}

/// ⭐⭐⭐ **O VETO — a retopologia ESTILHAÇOU a peça?** `Some((peças, eram))` quando sim.
///
/// ⛔⛔⛔ **Reproduzido em 2026-08-30 com a peça do artista, e é a foto dele:** ao carregar
/// no botão uma **segunda** vez, a saída vem com `2` peças — um pedaço solto de `22` faces a
/// flutuar — `χ` de `2` para `4`, e a ponta mais longa cortada de `−0,2 %` para **`−35,0 %`**.
/// ⚠️ **Um clique só não o faz**: o insumo do segundo clique é a saída do primeiro, e é a
/// re-entrada que parte a peça.
///
/// ⚠️ **É RELATIVO à entrada, nunca absoluto:** uma cena com dois objectos soltos entra com
/// `2` peças e tem todo o direito de sair com `2`. *O que o botão não pode é devolver mais
/// peças do que recebeu.*
///
/// ⚠️ **O veto é a ÚLTIMA palavra e não uma candidata:** o [`worse`] escolhe entre tentativas
/// e só sabe dizer qual é a melhor; quando **todas** estilhaçam, a melhor delas ainda é uma
/// peça partida. *Uma escada de candidatas nunca compara com o que o artista já tinha na
/// mão* — e o que ele tinha é o que fica.
pub(super) fn shattered(out: &Mesh, reference: &Mesh) -> Option<(usize, usize)> {
    let (saiu, entrou) = (components(out), components(reference));
    (saiu > entrou).then_some((saiu, entrou))
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

#[cfg(test)]
#[path = "sculpt3d_retopo_rulers_tests.rs"]
mod tests;
