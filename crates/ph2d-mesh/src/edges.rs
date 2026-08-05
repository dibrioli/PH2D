//! **O GRAFO DE ARESTAS** — o pré-requisito que a W6.0 nomeou e não precisou.
//!
//! Adaptado de `reference/sculptgl/src/mesh/Mesh.js` (`initEdges`, `_edges`,
//! `_faceEdges`), MIT — ver `LICENSES/sculptgl-MIT.txt`.
//!
//! Duas perguntas, e nenhuma delas a [`crate::Adjacency`] responde:
//!
//! 1. **Qual é a aresta do canto `k` da face `f`?** ([`Edges::face_edge`]) —
//!    subdividir precisa que as duas faces que compartilham uma aresta ponham o
//!    vértice novo no MESMO lugar, e "o mesmo lugar" é um índice de aresta.
//! 2. **Quantas faces usam esta aresta?** ([`Edges::valence`]) — `1` é borda,
//!    `2` é interior manifold, `≥ 3` é não-manifold. As três recebem regras
//!    diferentes de subdivisão, e a do meio é a única que suaviza.
//!
//! ⚠️ **Ele NÃO é guardado na malha, e isso é uma decisão medida em custo.** A
//! `Adjacency` e a `Octree` são reconstruídas por todo `Mesh::rebuild` porque
//! todo dab as consulta; o grafo de arestas hoje tem **um** consumidor
//! (subdividir), que roda quando o artista aperta um botão. Guardá-lo cobraria
//! a construção em cada `rebuild` — incluindo o de todo undo — para responder
//! uma pergunta que ninguém faz naquele instante. Se um consumidor por-frame
//! nascer, ele vira campo **com a medição ao lado**, não antes.
//!
//! ⚠️ **A numeração é DETERMINÍSTICA e derivada, não sorteada.** Uma aresta
//! `{lo, hi}` pertence ao vértice de índice MENOR, e o id dela é
//! `owned_start[lo] + o rank de hi entre os vizinhos de lo maiores que lo` — a
//! ordem do anel CSR, que é ela própria função das faces. Duas construções
//! sobre a mesma malha dão os mesmos ids, então nada que os guarde entre
//! chamadas pode divergir.

use crate::adjacency::Adjacency;
use crate::face::{Face, TRI};

/// **SÓ A NUMERAÇÃO** — a metade barata do grafo.
///
/// ⚠️ **Ela existe porque as duas metades do [`Edges`] têm preços muito
/// diferentes e consumidores diferentes.** Medido a 98k
/// (`measure_what_the_edge_graph_is_made_of`): o prefixo custa **0,184 ms** e o
/// mapa `(face, canto) → id` custa **1,498 ms** — 89% do total. O refino precisa
/// do id para MARCAR umas poucas arestas da região, e pagava a malha inteira por
/// isso; com a numeração sozinha ele pergunta `id_of` sob demanda, `O(valência)`.
///
/// A numeração é a MESMA (o [`Edges`] a contém e delega), então nada que guarde
/// um id entre as duas rotas pode divergir.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct EdgeIds {
    /// Onde começam as arestas POSSUÍDAS por cada vértice, com o `n + 1` do CSR.
    owned_start: Vec<u32>,
}

impl EdgeIds {
    /// Constrói a numeração — um passe sobre os VÉRTICES, e mais nada.
    #[must_use]
    pub fn build(adjacency: &Adjacency) -> Self {
        let n = adjacency.vert_verts.len();
        let mut owned_start = Vec::with_capacity(n + 1);
        let mut total = 0u32;
        for v in 0..n {
            owned_start.push(total);
            // Só as arestas que ESTE vértice possui: as que vão para um índice
            // maior. Cada aresta é possuída por exatamente um dos dois lados,
            // e é isso que a conta uma vez.
            total += u32::try_from(
                adjacency
                    .vert_verts
                    .neighbours(v)
                    .iter()
                    .filter(|&&w| w as usize > v)
                    .count(),
            )
            .unwrap_or(u32::MAX);
        }
        owned_start.push(total);
        Self { owned_start }
    }

    /// O índice da aresta `{a, b}`, ou `None` se ela não existe no anel.
    ///
    /// ⚠️ O `None` **não** é um caso normal: toda aresta de face está no anel de
    /// vértices por construção. Ele existe porque um chamador pode perguntar por
    /// um par que não é aresta, e devolver um índice inventado seria pior.
    #[must_use]
    pub fn id_of(&self, adjacency: &Adjacency, a: u32, b: u32) -> Option<u32> {
        let (lo, hi) = if a < b { (a, b) } else { (b, a) };
        let mut rank = 0u32;
        for &w in adjacency.vert_verts.neighbours(lo as usize) {
            if w <= lo {
                continue;
            }
            if w == hi {
                return Some(self.owned_start[lo as usize] + rank);
            }
            rank += 1;
        }
        None
    }

    /// Quantas arestas a malha tem.
    #[must_use]
    pub fn len(&self) -> usize {
        self.owned_start.last().copied().unwrap_or(0) as usize
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Bytes segurados — a sonda de memória os soma.
    #[must_use]
    pub fn capacity_bytes(&self) -> usize {
        self.owned_start.capacity() * size_of::<u32>()
    }
}

/// O grafo de arestas de uma malha — ver o cabeçalho.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Edges {
    /// A numeração — a porta única de *qual é o id desta aresta*.
    ids: EdgeIds,
    /// Por face e por canto, o índice da aresta que sai daquele canto. O slot 3
    /// de um triângulo é [`TRI`], como no [`Face`] que ele espelha.
    face_edges: Vec<[u32; 4]>,
    /// Quantas faces usam cada aresta.
    valence: Vec<u32>,
}

impl Edges {
    /// Constrói o grafo a partir das faces e do anel de vértices.
    ///
    /// ⚠️ **A `adjacency` TEM de ser a destas `faces`.** Ela é a fonte da
    /// numeração, então um par desalinhado não falha — devolve ids que não
    /// descrevem esta malha, que é a forma silenciosa de errar. Quem chama por
    /// [`crate::Mesh::edges`] não consegue desalinhar; um chamador direto
    /// consegue, e é por isso que este parágrafo existe.
    #[must_use]
    pub fn build(faces: &[Face], adjacency: &Adjacency) -> Self {
        let ids = EdgeIds::build(adjacency);
        let total = ids.len();
        let mut out = Self {
            ids,
            face_edges: vec![[TRI; 4]; faces.len()],
            valence: vec![0; total],
        };
        for (f, face) in faces.iter().enumerate() {
            let verts = face.verts();
            for k in 0..verts.len() {
                // A aresta do canto `k` vai daquele vértice ao SEGUINTE no ciclo
                // da face — a mesma regra para tri e quad, escrita uma vez (a
                // divergência nº 2 que a `Adjacency` já tomou do original).
                let a = verts[k];
                let b = verts[(k + 1) % verts.len()];
                let Some(e) = out.id_of(adjacency, a, b) else {
                    continue;
                };
                out.face_edges[f][k] = e;
                out.valence[e as usize] += 1;
            }
        }
        out
    }

    /// A numeração sozinha — ver [`EdgeIds`].
    #[must_use]
    pub fn ids(&self) -> &EdgeIds {
        &self.ids
    }

    /// O índice da aresta `{a, b}`, ou `None` se ela não existe no anel.
    /// Delega para a [`EdgeIds`]: uma pergunta, uma resposta.
    #[must_use]
    pub fn id_of(&self, adjacency: &Adjacency, a: u32, b: u32) -> Option<u32> {
        self.ids.id_of(adjacency, a, b)
    }

    /// Quantas arestas a malha tem.
    #[must_use]
    pub fn len(&self) -> usize {
        self.valence.len()
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.valence.is_empty()
    }

    /// A aresta do canto `k` da face `f`, ou `None` no slot 3 de um triângulo.
    #[must_use]
    pub fn face_edge(&self, f: usize, k: usize) -> Option<u32> {
        let e = *self.face_edges.get(f)?.get(k)?;
        (e != TRI).then_some(e)
    }

    /// Quantas faces usam a aresta `e` — **`1` é borda**, `2` é interior
    /// manifold, `≥ 3` é não-manifold.
    #[must_use]
    pub fn valence(&self, e: u32) -> u32 {
        self.valence.get(e as usize).copied().unwrap_or(0)
    }

    /// Bytes segurados — a sonda de memória os soma.
    #[must_use]
    pub fn capacity_bytes(&self) -> usize {
        self.ids.capacity_bytes()
            + self.face_edges.capacity() * size_of::<[u32; 4]>()
            + self.valence.capacity() * size_of::<u32>()
    }
}

#[cfg(test)]
#[path = "edges_tests.rs"]
mod tests;
