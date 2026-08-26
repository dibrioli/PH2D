//! ⭐⭐⭐ **PARTIR O QUE NÃO É MANIFOLD** — a reparação de porta que a cadeia de
//! retopologia precisava e que não existia.
//!
//! # ⛔⛔ Por que ela existe, com a cadeia inteira medida
//!
//! Uma aresta **não-manifold** é reclamada por três ou mais faces. ⚠️ **Todo o resto do
//! motor assume duas**: o mapa de meias-arestas que o layout percorre é `(a, b) → face`,
//! **uma face por aresta dirigida**, e com três a reclamar a mesma ele guarda uma e as
//! outras desaparecem. *A travessia entra na face errada ou não acha nenhuma, e morre.*
//!
//! Medido em 2026-08-25 na escultura do artista (`docs/3D/quad-remesh/ACHADO_ordem_das_fases.md`
//! §11): **2 arestas não-manifold, a raio `1,30×`** — a ponta —, e os furos da saída a raio
//! `1,29×`. **O mesmo sítio.** A cascata:
//!
//! > não-manifold ⇒ o mapa de meias-arestas mente ali ⇒ a travessia de fronteira morre ⇒
//! > laços de um vértice ⇒ patches acusados de degenerados sem o serem ⇒ a limpeza persegue
//! > fantasmas ⇒ o mapa recebe uma descrição que não é a do layout ⇒ **furo na ponta**.
//!
//! # ⭐ A lei, e por que é a dos VÉRTICES e não a das arestas
//!
//! A cura óbvia seria *«partir a aresta má»*. ⛔ Ela não basta: uma aresta é ambígua porque
//! as faces à volta dela **não formam um leque**, e o mesmo defeito aparece num **vértice**
//! sozinho (duas superfícies que se tocam num ponto — o «laço de gravata»), onde não há
//! aresta má nenhuma.
//!
//! ⇒ A lei é sobre o vértice: **duas faces do anel de `v` pertencem à mesma cópia de `v`
//! quando se alcançam por arestas que têm exactamente DUAS faces**. Uma aresta ambígua
//! deixa de ligar, e as faces de cada lado dela caem em cópias diferentes — a aresta parte-se
//! sozinha, como consequência. ⭐ *Uma lei sobre a conectividade explica os dois defeitos; uma
//! lei sobre arestas explica um.*
//!
//! ⚠️ **Nada de geometria se move.** As cópias nascem na mesma posição; o que muda é quem
//! é vizinho de quem. *A peça que o artista vê é a mesma, byte a byte, nas posições.*

use std::collections::BTreeMap;

use crate::{Face, Mesh};

/// O que a reparação fez.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct ManifoldReport {
    /// ⛔ Arestas reclamadas por **três ou mais** faces, antes.
    pub bad_edges_before: usize,
    /// ⛔ E depois — ⚠️ **`0` é a única resposta boa**, e a barra do gate.
    pub bad_edges_after: usize,
    /// ⭐ Vértices que ganharam pelo menos uma cópia.
    pub split_verts: usize,
    /// Cópias criadas ao todo.
    pub copies: usize,
}

/// União-busca sobre os índices locais do anel de um vértice.
struct Uf(Vec<u32>);

impl Uf {
    fn new(n: usize) -> Self {
        #[allow(clippy::cast_possible_truncation)]
        Self((0..n as u32).collect())
    }
    fn root(&mut self, mut a: u32) -> u32 {
        while self.0[a as usize] != a {
            let g = self.0[self.0[a as usize] as usize];
            self.0[a as usize] = g;
            a = g;
        }
        a
    }
    fn join(&mut self, a: u32, b: u32) {
        let (ra, rb) = (self.root(a), self.root(b));
        if ra != rb {
            self.0[rb as usize] = ra;
        }
    }
}

/// As faces de cada aresta, como par ordenado.
fn edge_faces(mesh: &Mesh) -> BTreeMap<(u32, u32), Vec<u32>> {
    let mut out: BTreeMap<(u32, u32), Vec<u32>> = BTreeMap::new();
    for (fi, f) in mesh.faces().iter().enumerate() {
        let v = f.verts();
        for k in 0..v.len() {
            let (a, b) = (v[k], v[(k + 1) % v.len()]);
            #[allow(clippy::cast_possible_truncation)]
            out.entry(if a < b { (a, b) } else { (b, a) })
                .or_default()
                .push(fi as u32);
        }
    }
    out
}

/// ⭐ **Quantas arestas são reclamadas por três ou mais faces** — a régua.
#[must_use]
pub fn non_manifold_edges(mesh: &Mesh) -> usize {
    edge_faces(mesh).values().filter(|f| f.len() >= 3).count()
}

/// ⭐⭐⭐ **REMOVE A ALETA: numa aresta reclamada por três faces, deita fora as que sobram.**
///
/// ⛔⛔ **Partir uma aresta ambígua numa peça FECHADA abre-a, e o rasgo é pior que o
/// defeito.** Medido 2026-08-25 na escultura do artista, em três variantes:
///
/// | variante | bordo da saída | `χ` | transições inexactas |
/// |---|---|---|---|
/// | ⭐ não reparar | **`8`** | **`1`** | ⛔ `8` |
/// | partir antes do remalhe | ⛔ `148` | ⛔ `−16` | ⭐ `0` |
/// | partir + fechar buracos | ⛔ **saída VAZIA** | — | `0` |
/// | partir depois do remalhe | `8` | `0` | ⛔ `12` |
///
/// ⭐ **A partição CURA a raiz** (as transições inexactas vão a `0`) e paga com um rasgo em
/// todas as formas. ⇒ *a pergunta deixou de ser «reparar?» e passou a ser «reparar COMO».*
///
/// ⚠️ **Esta é a outra resposta:** numa peça fechada, uma aresta com três faces tem **duas**
/// que são a superfície e uma que é uma aleta. Deitar a aleta fora mantém a peça fechada.
/// ⛔ **Ela perde geometria**, e é por isso que a contagem sai no relatório.
pub fn drop_extra_faces(mesh: &mut Mesh) -> ManifoldReport {
    let mut rep = ManifoldReport::default();
    let ef = edge_faces(mesh);
    rep.bad_edges_before = ef.values().filter(|f| f.len() >= 3).count();
    if rep.bad_edges_before == 0 {
        return rep;
    }
    // ⚠️ **A face que sai é a que tem MENOS vizinhos manifold** — ou seja, a que está menos
    // costurada à superfície. *Escolher a primeira por índice deitaria fora a superfície e
    // ficaria com a aleta.*
    let mut manifold_neighbours: BTreeMap<u32, usize> = BTreeMap::new();
    for who in ef.values().filter(|f| f.len() == 2) {
        for &f in who.iter() {
            *manifold_neighbours.entry(f).or_default() += 1;
        }
    }
    let mut drop: std::collections::BTreeSet<u32> = std::collections::BTreeSet::new();
    for who in ef.values().filter(|f| f.len() >= 3) {
        let mut by_grip: Vec<u32> = who.clone();
        by_grip.sort_by_key(|f| {
            (
                std::cmp::Reverse(manifold_neighbours.get(f).copied().unwrap_or(0)),
                *f,
            )
        });
        for &f in by_grip.iter().skip(2) {
            drop.insert(f);
        }
    }
    rep.copies = drop.len();
    let faces: Vec<Face> = mesh
        .faces()
        .iter()
        .enumerate()
        .filter(|(fi, _)| u32::try_from(*fi).is_ok_and(|f| !drop.contains(&f)))
        .map(|(_, f)| *f)
        .collect();
    if let Ok(next) = Mesh::from_parts(mesh.positions().to_vec(), faces) {
        *mesh = next;
    }
    rep.bad_edges_after = non_manifold_edges(mesh);
    rep
}

/// O que a remoção de folhas de espessura zero fez.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct DoubledReport {
    /// ⛔ Arestas reclamadas por três ou mais faces, antes.
    pub bad_edges_before: usize,
    /// ⛔ E depois.
    pub bad_edges_after: usize,
    /// ⛔ Arestas de **bordo** antes — a régua que decide se a operação é aceite.
    pub border_before: usize,
    /// ⛔ E depois. ⚠️ **Subir é o critério de recusa**, não um aviso.
    pub border_after: usize,
    /// ⭐ Pares `(triângulo, espelho)` removidos — cada par é uma folha de espessura zero.
    pub mirror_pairs: usize,
    /// Cópias com a **mesma** orientação deitadas fora (lixo puro: `n` iguais viram `1`).
    pub same_winding_dropped: usize,
    /// ⚠️ **A operação foi DESFEITA** porque abriria a peça.
    pub refused: bool,
}

/// Quantas arestas têm **uma só** face — o bordo.
#[must_use]
pub fn border_edges(mesh: &Mesh) -> usize {
    edge_faces(mesh).values().filter(|f| f.len() == 1).count()
}

/// A rotação canónica de um ciclo — **preserva o sentido**, ao contrário de ordenar.
fn cycle_key(v: &[u32]) -> [u32; 3] {
    let mut c = [v[0], v[1], v[2 % v.len()]];
    let m = c.iter().copied().min().unwrap_or(c[0]);
    while c[0] != m {
        c = [c[1], c[2], c[0]];
    }
    c
}

/// ⭐⭐⭐ **REMOVE AS FOLHAS DE ESPESSURA ZERO** — um triângulo mais o seu espelho.
///
/// # ⛔⛔ Por que esta é a cura, e as outras quatro não eram
///
/// Em 2026-08-25 esta linha construiu quatro reparações não-manifold e as quatro saíram
/// **piores que o defeito** (`docs/3D/quad-remesh/ACHADO_ordem_das_fases.md` §12), todas
/// desenhadas a partir do **nome** — *«uma aleta»*, *«duas folhas»*. ⭐ A sonda
/// `manifold_census` mediu a estrutura e o nome estava errado: na escultura do artista as
/// **4** arestas ambíguas são as arestas de **4 faces repetidas**, e a coluna que decide
/// tudo é a orientação — `0` com a mesma, **`4` com orientação OPOSTA**.
///
/// ⇒ Não são duplicatas de lixo (que se deduplicam), nem um leque (que se parte): são um
/// **par `(triângulo, espelho)`**, uma bolsa de volume zero. ⚠️ *É por isso que as quatro
/// tentativas abriram a peça: apagar UMA das duas cópias tira metade de uma superfície
/// fechada.* Apagar **as duas** não tira superfície nenhuma — a bolsa não encerra volume.
///
/// # ⚠️ Ela recusa-se a si própria
///
/// A bolsa pode partilhar arestas com a superfície de verdade, e nem toda folha dupla é
/// auto-fechada. ⇒ a operação **mede o bordo antes e depois** e **desfaz-se** se o bordo
/// subir. *A régua da recusa é a mesma que reprovou as outras quatro, e agora corre dentro
/// da própria cura em vez de depois dela.*
///
/// ⚠️ **Inerte numa malha sem repetição**: sai byte-idêntica, e há gate.
pub fn drop_doubled_faces(mesh: &mut Mesh) -> DoubledReport {
    let mut rep = DoubledReport {
        bad_edges_before: non_manifold_edges(mesh),
        border_before: border_edges(mesh),
        ..DoubledReport::default()
    };

    // Agrupa por CONJUNTO de vértices; dentro do grupo, separa por sentido.
    let mut by_set: BTreeMap<[u32; 3], Vec<u32>> = BTreeMap::new();
    for (fi, f) in mesh.faces().iter().enumerate() {
        let mut k = [f.verts()[0], f.verts()[1], f.verts()[2 % f.verts().len()]];
        k.sort_unstable();
        #[allow(clippy::cast_possible_truncation)]
        by_set.entry(k).or_default().push(fi as u32);
    }

    let mut drop: std::collections::BTreeSet<u32> = std::collections::BTreeSet::new();
    for group in by_set.values().filter(|g| g.len() > 1) {
        let mut forward: Vec<u32> = Vec::new();
        let mut backward: Vec<u32> = Vec::new();
        let first = cycle_key(mesh.faces()[group[0] as usize].verts());
        for &fi in group {
            if cycle_key(mesh.faces()[fi as usize].verts()) == first {
                forward.push(fi);
            } else {
                backward.push(fi);
            }
        }
        // ⭐ Os pares `(face, espelho)` saem **aos dois**.
        let pairs = forward.len().min(backward.len());
        rep.mirror_pairs += pairs;
        for i in 0..pairs {
            drop.insert(forward[i]);
            drop.insert(backward[i]);
        }
        // O que sobra do mesmo lado é repetição pura: fica **uma**.
        for side in [&forward[pairs..], &backward[pairs..]] {
            for &fi in side.iter().skip(1) {
                drop.insert(fi);
                rep.same_winding_dropped += 1;
            }
        }
    }

    if drop.is_empty() {
        rep.bad_edges_after = rep.bad_edges_before;
        rep.border_after = rep.border_before;
        return rep;
    }

    let faces: Vec<Face> = mesh
        .faces()
        .iter()
        .enumerate()
        .filter(|(fi, _)| u32::try_from(*fi).is_ok_and(|f| !drop.contains(&f)))
        .map(|(_, f)| *f)
        .collect();
    let Ok(next) = Mesh::from_parts(mesh.positions().to_vec(), faces) else {
        rep.refused = true;
        rep.bad_edges_after = rep.bad_edges_before;
        rep.border_after = rep.border_before;
        return rep;
    };
    rep.border_after = border_edges(&next);
    rep.bad_edges_after = non_manifold_edges(&next);
    // ⛔ **A recusa.** Abrir a peça é pior que a aresta ambígua — foi isso que as quatro
    // variantes de 25/08 mediram, e a régua vive agora aqui dentro.
    if rep.border_after > rep.border_before {
        rep.refused = true;
        rep.bad_edges_after = rep.bad_edges_before;
        rep.border_after = rep.border_before;
        return rep;
    }
    *mesh = next;
    rep
}

/// ⭐⭐⭐ **PARTE os vértices até a malha ser manifold** — ver o doc do módulo.
///
/// ⚠️ **É idempotente e inerte numa malha já manifold**: sem aresta ambígua, todo anel é
/// uma componente só, ninguém é duplicado, e a malha sai **byte-idêntica**. Há gate.
pub fn split_non_manifold(mesh: &mut Mesh) -> ManifoldReport {
    let mut rep = ManifoldReport::default();
    let ef = edge_faces(mesh);
    rep.bad_edges_before = ef.values().filter(|f| f.len() >= 3).count();
    if rep.bad_edges_before == 0 {
        return rep;
    }

    // Por vértice, as faces do anel dele — e o índice local de cada uma.
    let n = mesh.positions().len();
    let mut ring: Vec<Vec<u32>> = vec![Vec::new(); n];
    for (fi, f) in mesh.faces().iter().enumerate() {
        for &v in f.verts() {
            #[allow(clippy::cast_possible_truncation)]
            ring[v as usize].push(fi as u32);
        }
    }

    // ⭐ **Só uma aresta com EXACTAMENTE duas faces liga.** É esta linha que faz a
    // ambígua deixar de ligar, e é dela que a partição sai.
    let mut copy_of: Vec<BTreeMap<u32, u32>> = vec![BTreeMap::new(); n];
    let mut positions = mesh.positions().to_vec();
    for v in 0..n {
        let faces = &ring[v];
        let local: BTreeMap<u32, u32> = faces
            .iter()
            .enumerate()
            .filter_map(|(i, &f)| u32::try_from(i).ok().map(|i| (f, i)))
            .collect();
        let mut uf = Uf::new(faces.len());
        for (&(a, b), who) in &ef {
            if who.len() != 2 || (a as usize != v && b as usize != v) {
                continue;
            }
            if let (Some(&x), Some(&y)) = (local.get(&who[0]), local.get(&who[1])) {
                uf.join(x, y);
            }
        }
        // A primeira componente fica com o vértice original; as outras ganham cópia.
        let mut seat: BTreeMap<u32, u32> = BTreeMap::new();
        for (i, &f) in faces.iter().enumerate() {
            #[allow(clippy::cast_possible_truncation)]
            let r = uf.root(i as u32);
            let slot = match seat.get(&r) {
                Some(&s) => s,
                None => {
                    // ⭐ **A primeira componente fica com o vértice original.** É isso que
                    // torna a operação inerte numa malha já manifold: com uma componente
                    // só, nenhuma posição é acrescentada e os índices não se mexem.
                    #[allow(clippy::cast_possible_truncation)]
                    let s = if seat.is_empty() {
                        v as u32
                    } else {
                        positions.push(positions[v]);
                        rep.copies += 1;
                        (positions.len() - 1) as u32
                    };
                    seat.insert(r, s);
                    s
                }
            };
            copy_of[v].insert(f, slot);
        }
        if seat.len() > 1 {
            rep.split_verts += 1;
        }
    }

    let faces: Vec<Face> = mesh
        .faces()
        .iter()
        .enumerate()
        .map(|(fi, f)| {
            #[allow(clippy::cast_possible_truncation)]
            let fi = fi as u32;
            let v = f.verts();
            let at = |k: usize| copy_of[v[k] as usize].get(&fi).copied().unwrap_or(v[k]);
            if v.len() == 3 {
                Face::tri(at(0), at(1), at(2))
            } else {
                Face::quad(at(0), at(1), at(2), at(3))
            }
        })
        .collect();

    if let Ok(next) = Mesh::from_parts(positions, faces) {
        *mesh = next;
    }
    rep.bad_edges_after = non_manifold_edges(mesh);
    rep
}

#[cfg(test)]
#[path = "manifold_tests.rs"]
mod tests;
