//! Adjacência em **CSR** — vértice → faces e vértice → vértices.
//!
//! Adaptado de `reference/sculptgl/src/mesh/Mesh.js` (`initFaceRings`,
//! `initVertexRings`), MIT — ver `LICENSES/sculptgl-MIT.txt`.
//!
//! **Duas divergências deliberadas do original**, as duas para tirar caso
//! especial em vez de acrescentar:
//!
//! 1. O SculptGL guarda pares `(start, count)` intercalados; aqui é o CSR
//!    canônico — um `offsets` de `n+1` entradas, onde `count = off[i+1] − off[i]`.
//!    A mesma informação num array em vez de dois, e o `n+1` remove o ramo de
//!    "último vértice".
//! 2. O anel de vértices do original decide os vizinhos por uma cadeia de
//!    comparações contra o índice do próprio vértice. Aqui é o **ciclo da face**:
//!    os vizinhos de `v` numa face são o anterior e o seguinte na ordem dela.
//!    É a mesma resposta para tri e quad, escrita uma vez.
//!
//! ⚠️ **Custo, e é ele que a `measure_memory` reporta.** Numa malha fechada de
//! triângulos com `V` vértices há `≈2V` faces e `≈3V` arestas, então
//! `vert_face ≈ 6V` índices e `vert_vert ≈ 6V` — juntos ~56 B por vértice, mais
//! que os 24 B de posição+normal. O anel de VÉRTICES só é lido por Smooth/Relax
//! e pela topologia; se o número apertar, é ele que vira preguiçoso (o padrão
//! dos planos do impasto), e a decisão sai da sonda, não daqui.

use crate::face::Face;

/// Uma lista de adjacência em CSR. `values[offsets[i]..offsets[i + 1]]` são os
/// vizinhos do item `i`.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Csr {
    offsets: Vec<u32>,
    values: Vec<u32>,
}

impl Csr {
    #[must_use]
    pub fn len(&self) -> usize {
        self.offsets.len().saturating_sub(1)
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Os vizinhos de `i`. Fora de alcance devolve vazio — um vértice que não
    /// existe não tem vizinhos, e isso é mais útil que um pânico no meio de um
    /// laço de pincel.
    #[must_use]
    pub fn neighbours(&self, i: usize) -> &[u32] {
        if i + 1 >= self.offsets.len() {
            return &[];
        }
        let s = self.offsets[i] as usize;
        let e = self.offsets[i + 1] as usize;
        &self.values[s..e]
    }

    /// Total de entradas — o que a sonda de memória multiplica por 4 bytes.
    #[must_use]
    pub fn entry_count(&self) -> usize {
        self.values.len()
    }
}

/// Os dois anéis que a malha carrega.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Adjacency {
    /// vértice → faces que o contêm.
    pub vert_faces: Csr,
    /// vértice → vértices ligados por uma aresta.
    pub vert_verts: Csr,
}

impl Adjacency {
    /// Constrói os dois anéis. `vert_count` é autoritativo: um vértice sem
    /// nenhuma face (solto num OBJ) existe e tem anel vazio, em vez de
    /// desaparecer e deslocar todos os índices depois dele.
    #[must_use]
    pub fn build(vert_count: usize, faces: &[Face]) -> Self {
        let vert_faces = build_vert_faces(vert_count, faces);
        let vert_verts = build_vert_verts(vert_count, faces, &vert_faces);
        Self {
            vert_faces,
            vert_verts,
        }
    }
}

/// Passe de contagem + passe de preenchimento — o CSR clássico, sem `Vec<Vec<_>>`
/// intermediário (que a 5 M faces seriam milhões de alocações).
fn build_vert_faces(vert_count: usize, faces: &[Face]) -> Csr {
    let mut offsets = vec![0u32; vert_count + 1];
    for f in faces {
        for &v in f.verts() {
            offsets[v as usize + 1] += 1;
        }
    }
    for i in 0..vert_count {
        offsets[i + 1] += offsets[i];
    }
    let total = offsets[vert_count] as usize;
    let mut values = vec![0u32; total];
    // `cursor` anda dentro de cada faixa; no fim ele é o offset do vértice
    // seguinte, que é a invariante que o gate `the_csr_offsets_are_a_prefix_sum`
    // afirma.
    let mut cursor: Vec<u32> = offsets[..vert_count].to_vec();
    for (fi, f) in faces.iter().enumerate() {
        for &v in f.verts() {
            let slot = &mut cursor[v as usize];
            values[*slot as usize] = fi as u32;
            *slot += 1;
        }
    }
    Csr { offsets, values }
}

/// O anel de vértices sai do anel de faces: para cada face que toca `v`, os
/// vizinhos são o anterior e o seguinte no **ciclo** dela.
///
/// A deduplicação usa um carimbo (`stamp`) por vértice em vez de um `HashSet`:
/// zero alocação por vértice, e determinístico — a ordem de saída é a ordem em
/// que as faces aparecem, nunca a ordem de iteração de um mapa.
fn build_vert_verts(vert_count: usize, faces: &[Face], vf: &Csr) -> Csr {
    let mut stamp = vec![u32::MAX; vert_count];
    let mut offsets = vec![0u32; vert_count + 1];

    // Passe 1 — conta os vizinhos únicos.
    for v in 0..vert_count {
        let mut n = 0u32;
        for_each_ring_neighbour(v, faces, vf, &mut stamp, v as u32, |_| n += 1);
        offsets[v + 1] = n;
    }
    for i in 0..vert_count {
        offsets[i + 1] += offsets[i];
    }

    // Passe 2 — preenche. O carimbo é reiniciado porque o passe 1 o gastou; um
    // segundo vetor seria memória à toa.
    stamp.fill(u32::MAX);
    let total = offsets[vert_count] as usize;
    let mut values = vec![0u32; total];
    for v in 0..vert_count {
        let mut w = offsets[v] as usize;
        for_each_ring_neighbour(v, faces, vf, &mut stamp, v as u32, |nb| {
            values[w] = nb;
            w += 1;
        });
        debug_assert_eq!(w, offsets[v + 1] as usize, "os dois passes discordaram");
    }

    Csr { offsets, values }
}

/// Chama `f` uma vez por vizinho ÚNICO de `v`, na ordem das faces.
///
/// `stamp[u] == epoch` significa "já visto nesta volta". `epoch` é o próprio
/// índice do vértice, que é único por definição — sem contador global e sem
/// risco de dois passes se confundirem.
fn for_each_ring_neighbour(
    v: usize,
    faces: &[Face],
    vf: &Csr,
    stamp: &mut [u32],
    epoch: u32,
    mut f: impl FnMut(u32),
) {
    // O próprio vértice nunca é vizinho de si.
    stamp[v] = epoch;
    for &fi in vf.neighbours(v) {
        let face = faces[fi as usize];
        let vs = face.verts();
        let Some(pos) = vs.iter().position(|&x| x as usize == v) else {
            continue;
        };
        let n = vs.len();
        for nb in [vs[(pos + n - 1) % n], vs[(pos + 1) % n]] {
            if stamp[nb as usize] != epoch {
                stamp[nb as usize] = epoch;
                f(nb);
            }
        }
    }
}

#[cfg(test)]
#[path = "adjacency_tests.rs"]
mod tests;
