//! ⭐⭐⭐⭐ **A TINTA DEIXA DE SER DOS VÉRTICES** — *mesh colors*: a cor mora em
//! amostras nos **vértices**, nas **arestas** e no **interior** das faces.
//!
//! Clean-room de dois papers públicos — Yuksel, Keyser, House, *Mesh Colors*
//! (ACM TOG 29(2), 2010) e Yuksel, *Mesh Color Textures* (HPG 2017). A pesquisa
//! que escolheu esta família, a triagem de licença e a medição estão em
//! `docs/3D/27_o_estado_da_arte_de_onde_a_tinta_mora.md`.
//!
//! # ⭐⭐⭐ Por que esta família, em duas frases com número
//!
//! *A resolução da tinta deixa de ser a resolução da MALHA.* Medido no corpus
//! do dono (doc 27 §4): ao mesmo orçamento de memória, uma tinta sem costuras
//! lê **`379`** amostras por unidade de mundo no pior sítio da peça contra as
//! **`113`** do atlas `2048²` — e a dispersão do atlas **não é do solver, é o
//! Teorema Egregium**: quem achata um pedaço CURVO distorce área por obrigação.
//! Aqui nada é achatado: cada amostra vive na face, e um mapa afim num
//! triângulo preserva a razão de áreas **exactamente**.
//!
//! # ⭐⭐⭐ E o caso base É o produto que já shipa
//!
//! O nível é `lado = 2^k` **intervalos por aresta**, e a `lado = 1` só existem
//! as amostras de canto — **uma por vértice, na numeração da malha**. ⇒
//! [`Tinta`] ao nível zero é a `colors()` de hoje, ao bit, e não há migração
//! nenhuma a escrever. *Uma família nova cujo caso base é o produto actual
//! entra sem um degrau de formato.*
//!
//! ⚠️ **A escada é de potências de dois porque METADE tem de ser exacta:** as
//! amostras estão em `i/L`, e ficar com as de `i` par dá exactamente `i/(L/2)`,
//! **com as duas pontas preservadas**. Uma escada `2^r − 1` (a que o paper usa
//! para contar as amostras INTERIORES de uma aresta) descreve a mesma retícula,
//! e `lado` conta os intervalos porque é o que a aritmética do endereço usa.
//!
//! # ⛔⛔ A fronteira é PARTILHADA, e é isso que a separa do Ptex
//!
//! Uma amostra de aresta é guardada **uma vez**, e as duas faces que se tocam
//! ali leem **a mesma célula** — logo não há costura, nem sequer de filtragem,
//! e não há adjacência a consultar durante a leitura. O Ptex e o Htex duplicam
//! essa fronteira e pagam-na em buscas (`5` e `3` por ponto) e em
//! descontinuidade nos cantos. *É por isso que esta é a família certa para um
//! pincel e não só para um renderizador.*
//!
//! # O que esta crate NÃO faz
//!
//! Ela não sabe o que é uma `Mesh`, um pincel, uma câmera ou um device. Ela
//! recebe **faces** (fatias de índices) e posições, e devolve **índices de
//! amostra** e pesos. Quem os pinta, quem os sobe à placa e quem os re-amostra
//! quando a topologia muda são os chamadores.

#![forbid(unsafe_code)]

pub mod amostragem;
pub mod enderecos;
pub mod topo;
pub mod vizinhanca;

#[cfg(test)]
#[path = "enderecos_tests.rs"]
mod enderecos_tests;
#[cfg(test)]
#[path = "lib_tests.rs"]
mod lib_tests;
#[cfg(test)]
#[path = "vizinhanca_tests.rs"]
mod vizinhanca_tests;

pub use enderecos::{Sitio, indice, sitio_quad, sitio_tri, total};
pub use topo::{TRI, Topologia, cantos, interior_por_face};

/// A cor de quem ninguém pintou.
///
/// ⚠️ **Ela é o mesmo valor que a `ph2d_mesh::DEFAULT_COLOR`, e as duas têm de
/// concordar**: a `Tinta` ao nível zero é aquele plano. Há gate do lado da
/// `ph2d-mesh`, que é quem pode importar as duas.
pub const BRANCO: [f32; 3] = [1.0, 1.0, 1.0];

/// ⛔⛔ **O NÍVEL MÁXIMO, e ele nomeia DOIS recursos.**
///
/// | `k` | `lado` | amostras na peça de fábrica (`98 306` V) | plano em `f32×3` |
/// |---|---|---|---|
/// | `0` | `1` | `0,10 M` | `1,2 MB` |
/// | `1` | `2` | `0,39 M` | `4,7 MB` |
/// | `2` | `4` | `1,57 M` | `18,9 MB` |
/// | `3` | `8` | `6,29 M` | `75,5 MB` |
/// | `4` | `16` | `25,2 M` | `302 MB` |
/// | `5` | `32` | `101 M` | ⛔ `1,2 GB` |
///
/// A contagem é `≈ V · lado²` numa malha de triângulos (`V` vértices, `3V`
/// arestas, `2V` faces — a aritmética fecha e está no gate).
///
/// ⭐ **`5` é onde o ÍNDICE ainda cabe:** a `lado = 64` uma malha de `1 M`
/// vértices pede `4,1e9` amostras, e o `u32` que indexa este plano — e o buffer
/// da placa — para em `4,29e9`. ⚠️ **O tecto de MEMÓRIA é mais apertado que este
/// e é do CHAMADOR**, que é quem sabe quanta placa a cena tem: a tabela acima
/// existe para ele escolher, e é por isso que ela está aqui e não num comentário.
pub const NIVEL_MAX: u8 = 5;

/// ⭐ **O PLANO DE TINTA de uma malha.**
///
/// ⚠️ Ele guarda a [`Topologia`] porque o endereço de uma amostra de aresta
/// depende de qual aresta e de que lado — recalcular isso por amostra seria uma
/// busca por dab.
#[derive(Debug, Clone, PartialEq)]
pub struct Tinta {
    nivel: u8,
    topo: Topologia,
    amostras: Vec<[f32; 3]>,
}

impl Tinta {
    /// Uma tinta em branco sobre a topologia dada.
    ///
    /// ⚠️ `nivel` é **cortado** em [`NIVEL_MAX`] em vez de recusado: o chamador
    /// que pede `9` tem um defeito, e entregar-lhe um plano de `1,2 GB` ou um
    /// `panic` são as duas respostas piores. O nível efectivo lê-se em
    /// [`Self::nivel`].
    #[must_use]
    pub fn nova<'a>(verts: usize, faces: impl Iterator<Item = &'a [u32]>, nivel: u8) -> Self {
        let nivel = nivel.min(NIVEL_MAX);
        let lado = 1u32 << nivel;
        let topo = Topologia::nova(verts, faces, lado);
        let n = total(&topo, lado);
        Self {
            nivel,
            topo,
            amostras: vec![BRANCO; n],
        }
    }

    /// ⭐ **A tinta do nível ZERO a partir do plano por-vértice que já existe.**
    ///
    /// ⛔ Ela copia e não converte, porque **não há conversão**: ao nível zero as
    /// duas coisas são o mesmo vector.
    #[must_use]
    pub fn do_plano_por_vertice<'a>(
        cores: &[[f32; 3]],
        faces: impl Iterator<Item = &'a [u32]>,
    ) -> Self {
        let topo = Topologia::nova(cores.len(), faces, 1);
        Self {
            nivel: 0,
            topo,
            amostras: cores.to_vec(),
        }
    }

    /// O nível efectivo (`k`).
    #[must_use]
    pub fn nivel(&self) -> u8 {
        self.nivel
    }

    /// Intervalos por aresta — `2^k`.
    #[must_use]
    pub fn lado(&self) -> u32 {
        1u32 << self.nivel
    }

    /// A topologia derivada.
    #[must_use]
    pub fn topologia(&self) -> &Topologia {
        &self.topo
    }

    /// As amostras, na disposição do módulo [`enderecos`].
    #[must_use]
    pub fn amostras(&self) -> &[[f32; 3]] {
        &self.amostras
    }

    /// As amostras, para escrever.
    pub fn amostras_mut(&mut self) -> &mut [[f32; 3]] {
        &mut self.amostras
    }

    /// ⭐ **A amostra de um VÉRTICE é o próprio índice do vértice** — a lei que
    /// faz o nível zero ser a `colors()` de hoje.
    #[must_use]
    pub fn de_vertice(&self, v: usize) -> [f32; 3] {
        self.amostras[v]
    }

    /// O prefixo por-vértice do plano.
    ///
    /// ⭐ A `lado = 1` ele **é** o plano inteiro; acima disso é a fatia que um
    /// renderizador sem `lado` consegue desenhar, que é a rede de quem não tem
    /// a feature do device.
    #[must_use]
    pub fn plano_por_vertice(&self) -> &[[f32; 3]] {
        &self.amostras[..self.topo.verts]
    }

    /// O índice global de uma amostra da retícula de `face`.
    ///
    /// `cantos` são os índices de vértice da face, na ordem do percurso.
    #[must_use]
    pub fn indice_tri(&self, face: usize, cantos: &[u32], i: u32, j: u32, k: u32) -> u32 {
        let l = self.lado();
        indice(&self.topo, l, face, sitio_tri(l, i, j, k), cantos)
    }

    /// O índice global de uma amostra da retícula de um QUAD.
    #[must_use]
    pub fn indice_quad(&self, face: usize, cantos: &[u32], i: u32, j: u32) -> u32 {
        let l = self.lado();
        indice(&self.topo, l, face, sitio_quad(l, i, j), cantos)
    }
}
