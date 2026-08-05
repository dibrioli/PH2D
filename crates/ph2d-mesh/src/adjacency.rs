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

/// Uma lista de adjacência em CSR **editável**. `values[starts[i]..][..lens[i]]`
/// são os vizinhos do item `i`.
///
/// ⚠️ **`starts` NÃO é mais uma soma de prefixos, e a mudança é o que torna uma
/// mudança de topologia uma EDIÇÃO em vez de uma malha nova.** No CSR canônico o
/// começo da linha `i + 1` é o fim da linha `i`, então crescer uma linha
/// significa deslocar todas as seguintes — `O(malha)` para acrescentar UM
/// vizinho. Aqui cada linha carrega o próprio começo e o próprio comprimento, e
/// uma linha que cresce **se muda para o fim** do vetor (a cópia é de meia dúzia
/// de índices, porque uma valência é meia dúzia). O rastro que ela deixa é
/// contado em [`Csr::dead`] e recolhido por [`Csr::compact_if_needed`].
///
/// ⚠️ **As linhas ficam em ORDEM CRESCENTE, e isso não é estética.** Uma linha
/// editada tem de ser *exatamente* a que uma reconstrução produziria: a soma das
/// normais de vértice percorre esta lista, e somar os mesmos números noutra
/// ordem dá outro último bit. Um incremental que "só" reordena o anel é um
/// incremental que diverge do `rebuild` no dígito que ninguém confere.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Csr {
    starts: Vec<u32>,
    lens: Vec<u32>,
    values: Vec<u32>,
    /// Entradas que nenhuma linha usa mais — o rastro das mudanças.
    dead: u32,
}

impl Csr {
    #[must_use]
    pub fn len(&self) -> usize {
        self.starts.len()
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
        let Some(&s) = self.starts.get(i) else {
            return &[];
        };
        let s = s as usize;
        &self.values[s..s + self.lens[i] as usize]
    }

    /// Total de entradas ALOCADAS — o que a sonda de memória multiplica por 4
    /// bytes. ⚠️ Inclui o rastro: é a pergunta de MEMÓRIA, não a de conteúdo.
    #[must_use]
    pub fn entry_count(&self) -> usize {
        self.values.len()
    }

    /// **Acrescenta `v` à linha `i`, mantendo-a crescente.** No-op se já estiver
    /// lá — um vizinho repetido não é um anel, é um bug a jusante.
    fn insert_sorted(&mut self, i: usize, v: u32) {
        let row = self.neighbours(i);
        let Err(at) = row.binary_search(&v) else {
            return;
        };
        let (s, n) = (self.starts[i] as usize, self.lens[i] as usize);
        // A linha se muda para o fim: o CSR canônico não tem folga por linha, e
        // inventar folga custaria um terceiro vetor de capacidades por vértice.
        // Copiar meia dúzia de índices é mais barato que 4 B/vértice para sempre.
        let dest = u32::try_from(self.values.len()).unwrap_or(u32::MAX);
        self.values.reserve(n + 1);
        for k in 0..at {
            self.values.push(self.values[s + k]);
        }
        self.values.push(v);
        for k in at..n {
            self.values.push(self.values[s + k]);
        }
        self.starts[i] = dest;
        self.lens[i] = u32::try_from(n + 1).unwrap_or(u32::MAX);
        self.dead = self.dead.saturating_add(u32::try_from(n).unwrap_or(0));
    }

    /// **Tira `v` da linha `i`**, preservando a ordem. No-op se não estiver lá.
    fn remove(&mut self, i: usize, v: u32) {
        let Ok(at) = self.neighbours(i).binary_search(&v) else {
            return;
        };
        let (s, n) = (self.starts[i] as usize, self.lens[i] as usize);
        self.values.copy_within(s + at + 1..s + n, s + at);
        self.lens[i] = u32::try_from(n - 1).unwrap_or(0);
        self.dead = self.dead.saturating_add(1);
    }

    /// **Reescreve a linha `i` inteira**, na ordem dada.
    ///
    /// ⚠️ Ela NÃO ordena: o anel de vértices tem a ordem em que as faces
    /// aparecem, e é essa ordem que uma reconstrução produz.
    fn set_row(&mut self, i: usize, row: &[u32]) {
        let n = self.lens[i] as usize;
        if row.len() <= n {
            let s = self.starts[i] as usize;
            self.values[s..s + row.len()].copy_from_slice(row);
            self.lens[i] = u32::try_from(row.len()).unwrap_or(0);
            self.dead = self
                .dead
                .saturating_add(u32::try_from(n - row.len()).unwrap_or(0));
            return;
        }
        let dest = u32::try_from(self.values.len()).unwrap_or(u32::MAX);
        self.values.extend_from_slice(row);
        self.starts[i] = dest;
        self.lens[i] = u32::try_from(row.len()).unwrap_or(u32::MAX);
        self.dead = self.dead.saturating_add(u32::try_from(n).unwrap_or(0));
    }

    /// **Recolhe o rastro quando ele passa da metade.**
    ///
    /// ⚠️ O gatilho é uma FRAÇÃO e não uma contagem: um limiar absoluto compacta
    /// demais numa malha pequena e nunca numa grande, que é o multiplicador
    /// disfarçado de teto que o ADR-0117 nomeia.
    fn compact_if_needed(&mut self) {
        if usize::try_from(self.dead).unwrap_or(0) * 2 <= self.values.len() {
            return;
        }
        let mut packed = Vec::with_capacity(self.values.len() - self.dead as usize);
        for i in 0..self.starts.len() {
            let (s, n) = (self.starts[i] as usize, self.lens[i] as usize);
            self.starts[i] = u32::try_from(packed.len()).unwrap_or(u32::MAX);
            packed.extend_from_slice(&self.values[s..s + n]);
        }
        self.values = packed;
        self.dead = 0;
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
    /// **Este vértice está na BORDA de uma malha aberta?**
    ///
    /// A identidade é a do `Mesh.js` do SculptGL (`vertOnEdge`, dentro do build
    /// do anel): *nº de faces ≠ nº de vizinhos únicos*. Num vértice interior o
    /// anel FECHA, e cada aresta é compartilhada por duas faces, então os dois
    /// números batem; na beira o anel é aberto e sobra um vizinho.
    ///
    /// ⚠️ **Não há campo novo nem memória nova** — o dado já estava nos dois
    /// CSRs, e a pergunta é `O(1)`. Materializá-lo seria guardar duas vezes um
    /// fato que a estrutura já contém, e que teria de ser reconstruído a cada
    /// mudança de topologia.
    ///
    /// ⚠️ **Vale para triângulo e para quad**, porque a conta é sobre o ANEL e
    /// não sobre a forma da face: um vértice interior de grade de quads tem
    /// quatro faces e quatro vizinhos; na beira, duas faces e três vizinhos.
    #[must_use]
    pub fn is_border(&self, v: usize) -> bool {
        self.vert_faces.neighbours(v).len() != self.vert_verts.neighbours(v).len()
    }

    /// Quantos vizinhos este vértice tem — a **valência**.
    ///
    /// Abaixo de três não há superfície em volta dele (é a ponta de uma tira ou
    /// um vértice colado), e é isso que o Smooth precisa saber para não o
    /// escorregar para a corda entre os dois únicos vizinhos.
    #[must_use]
    pub fn valence(&self, v: usize) -> usize {
        self.vert_verts.neighbours(v).len()
    }
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

    /// **Uma face TROCOU de vértices** — atualiza os dois anéis sem reconstruir
    /// coisa alguma. Devolve os vértices afetados, em ordem crescente.
    ///
    /// Cada `edit` é `(índice, como era, como ficou)`, e `after` é a lista de
    /// faces **já reescrita**. ⚠️ **As duas metades leem lados diferentes do
    /// tempo** — a de faces remove pelos vértices ANTIGOS, a de anéis re-deriva
    /// pelos NOVOS —, e é por isso que o "antes" vem por parâmetro em vez de a
    /// função o ler: guardar a lista inteira de faces para tê-lo seria `O(malha)`
    /// dentro da porta que existe para não ser.
    ///
    /// ⚠️ **O anel de vértices é RE-DERIVADO, não remendado**, e a assimetria é
    /// deliberada: quem está no anel de `v` é função de TODAS as faces de `v`, e
    /// um remendo por aresta teria de saber que a aresta `a—b` só morre se
    /// NENHUMA outra face a compartilha. Re-derivar meia dúzia de vizinhos a
    /// partir de meia dúzia de faces é mais barato que essa contabilidade, e não
    /// tem como sair de sincronia.
    pub fn relink(&mut self, edits: &[(u32, Face, Face)], after: &[Face]) -> Vec<u32> {
        let mut affected: Vec<u32> = Vec::new();
        for &(i, old, new) in edits {
            for &v in old.verts() {
                if !new.verts().contains(&v) {
                    self.vert_faces.remove(v as usize, i);
                }
                affected.push(v);
            }
            for &v in new.verts() {
                if !old.verts().contains(&v) {
                    self.vert_faces.insert_sorted(v as usize, i);
                }
                affected.push(v);
            }
        }
        affected.sort_unstable();
        affected.dedup();

        let mut ring = Vec::new();
        for &v in &affected {
            ring_of(v as usize, after, &self.vert_faces, &mut ring);
            self.vert_verts.set_row(v as usize, &ring);
        }
        self.vert_faces.compact_if_needed();
        self.vert_verts.compact_if_needed();
        affected
    }
}

/// Passe de contagem + passe de preenchimento — o CSR clássico, sem `Vec<Vec<_>>`
/// intermediário (que a 5 M faces seriam milhões de alocações).
fn build_vert_faces(vert_count: usize, faces: &[Face]) -> Csr {
    let mut starts = vec![0u32; vert_count + 1];
    for f in faces {
        for &v in f.verts() {
            starts[v as usize + 1] += 1;
        }
    }
    for i in 0..vert_count {
        starts[i + 1] += starts[i];
    }
    let total = starts[vert_count] as usize;
    let mut values = vec![0u32; total];
    // `cursor` anda dentro de cada faixa; no fim ele é o começo do vértice
    // seguinte, que é a invariante que o gate `the_freshly_built_csr_is_packed`
    // afirma. ⚠️ E porque as faces são percorridas em ordem, **cada linha nasce
    // CRESCENTE** — a propriedade de que a edição por inserção ordenada depende.
    let mut cursor: Vec<u32> = starts[..vert_count].to_vec();
    for (fi, f) in faces.iter().enumerate() {
        for &v in f.verts() {
            let slot = &mut cursor[v as usize];
            values[*slot as usize] = fi as u32;
            *slot += 1;
        }
    }
    let lens = (0..vert_count).map(|i| starts[i + 1] - starts[i]).collect();
    starts.truncate(vert_count);
    Csr {
        starts,
        lens,
        values,
        dead: 0,
    }
}

/// O anel de vértices sai do anel de faces: para cada face que toca `v`, os
/// vizinhos são o anterior e o seguinte no **ciclo** dela.
///
/// A deduplicação usa um carimbo (`stamp`) por vértice em vez de um `HashSet`:
/// zero alocação por vértice, e determinístico — a ordem de saída é a ordem em
/// que as faces aparecem, nunca a ordem de iteração de um mapa.
fn build_vert_verts(vert_count: usize, faces: &[Face], vf: &Csr) -> Csr {
    let mut stamp = vec![u32::MAX; vert_count];
    let mut starts = vec![0u32; vert_count + 1];

    // Passe 1 — conta os vizinhos únicos.
    for v in 0..vert_count {
        let mut n = 0u32;
        for_each_ring_neighbour(v, faces, vf, &mut stamp, v as u32, |_| n += 1);
        starts[v + 1] = n;
    }
    for i in 0..vert_count {
        starts[i + 1] += starts[i];
    }

    // Passe 2 — preenche. O carimbo é reiniciado porque o passe 1 o gastou; um
    // segundo vetor seria memória à toa.
    stamp.fill(u32::MAX);
    let total = starts[vert_count] as usize;
    let mut values = vec![0u32; total];
    for v in 0..vert_count {
        let mut w = starts[v] as usize;
        for_each_ring_neighbour(v, faces, vf, &mut stamp, v as u32, |nb| {
            values[w] = nb;
            w += 1;
        });
        debug_assert_eq!(w, starts[v + 1] as usize, "os dois passes discordaram");
    }

    let lens = (0..vert_count).map(|i| starts[i + 1] - starts[i]).collect();
    starts.truncate(vert_count);
    Csr {
        starts,
        lens,
        values,
        dead: 0,
    }
}

/// O anel de vértices de UM vértice, na mesma ordem que o
/// [`build_vert_verts`] produziria — a metade que a edição incremental precisa
/// re-derivar.
///
/// ⚠️ **A deduplicação é uma varredura linear e não um carimbo**, e é de
/// propósito: um carimbo é um vetor do tamanho da MALHA, e esta função existe
/// justamente para não pagar isso. Um anel tem meia dúzia de vizinhos.
fn ring_of(v: usize, faces: &[Face], vf: &Csr, out: &mut Vec<u32>) {
    out.clear();
    for &fi in vf.neighbours(v) {
        let vs = faces[fi as usize].verts();
        let Some(pos) = vs.iter().position(|&x| x as usize == v) else {
            continue;
        };
        let n = vs.len();
        for nb in [vs[(pos + n - 1) % n], vs[(pos + 1) % n]] {
            if nb as usize != v && !out.contains(&nb) {
                out.push(nb);
            }
        }
    }
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
