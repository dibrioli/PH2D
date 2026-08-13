//! **OS RASCUNHOS** que as consultas e o refresh por região reusam entre dabs.
//!
//! ⚠️ **Corte por ASSUNTO, não por tamanho:** o irmão responde *o que uma malha
//! É*; isto responde *que memória um GESTO sobre ela reaproveita*. Os dois tipos
//! aqui não descrevem a peça — eles existem para o custo de uma consulta ser
//! função da PEGADA e não da malha, e alocá-los por movimento do mouse é o que
//! transforma um gesto em serrilhado.

use crate::octree::RefitScratch;

/// Buffers reutilizados entre consultas — a consulta é feita por movimento do
/// mouse, e alocar por dab é o que transforma um gesto em serrilhado.
#[derive(Clone, Debug, Default)]
pub struct QueryScratch {
    pub(crate) faces: Vec<u32>,
    pub(crate) seen: Vec<u32>,
    pub(crate) epoch: u32,
}

/// Buffers reutilizados pelo [`Mesh::refresh_region`].
///
/// Os `*_seen` são vetores do TAMANHO da malha, mas o passe só os toca onde
/// escreveu e os limpa na saída — é o que torna o custo função da pegada e não
/// da malha, e é a razão de eles viverem aqui em vez de nascerem por dab.
#[derive(Clone, Debug, Default)]
pub struct RegionScratch {
    pub(crate) faces: Vec<u32>,
    pub(crate) verts: Vec<u32>,
    pub(crate) face_seen: Vec<bool>,
    pub(crate) vert_seen: Vec<bool>,
    /// Saída contígua das portas paralelas, reusada pelas duas metades do passe.
    pub(crate) tmp: Vec<[f32; 3]>,
    /// A mesma coisa para a CURVATURA, que é escalar. ⚠️ Vetor próprio e não um
    /// reuso do [`Self::tmp`]: a curvatura roda **depois** das normais e as lê,
    /// então os dois estão vivos ao mesmo tempo.
    pub(crate) tmp_k: Vec<f32>,
    /// O irmão do `tmp_k` para a curvatura de MUNDO — mesma lista de vértices,
    /// mesmo gather, saída própria (o `curvature_of` devolve o par).
    pub(crate) tmp_kw: Vec<f32>,
    pub(crate) refit: RefitScratch,
}

impl RegionScratch {
    /// Os vértices cuja NORMAL o último `refresh_region` recomputou.
    ///
    /// ⚠️ **É um superconjunto de "quem se moveu", e a diferença é visível.** Um
    /// vizinho parado ao lado de uma face que girou tem a normal mudada; quem
    /// subir para a GPU só a lista de movidos deixa a normal velha exatamente na
    /// BORDA do pincel, que é onde o artista está olhando. Esta é a lista que o
    /// upload incremental consome.
    #[must_use]
    pub fn refreshed(&self) -> &[u32] {
        &self.verts
    }

    /// Declara que nada foi refrescado (um dab que não tocou geometria).
    ///
    /// Existe porque a alternativa é o chamador ler a lista do dab ANTERIOR e
    /// subir uma região que ninguém mexeu — barato, mas mentiroso, e a mentira
    /// vira um gate verde sobre um upload que não acompanha o produto.
    pub fn forget(&mut self) {
        self.verts.clear();
    }

    /// ⚠️ **`resize` e não `vec![]`, e a diferença aparece na topologia dinâmica:**
    /// a malha muda de tamanho a cada dab, e re-alocar dois vetores do tamanho
    /// dela por dab é `O(malha)` entrando pela porta dos fundos justamente na
    /// wave que existe para tirá-lo. Crescer preserva a capacidade e zera só a
    /// cauda — e as entradas antigas já são `false`, porque o passe limpa o que
    /// sujou antes de sair.
    pub(crate) fn reset(&mut self, faces: usize, verts: usize) {
        self.face_seen.resize(faces, false);
        self.vert_seen.resize(verts, false);
    }
}
