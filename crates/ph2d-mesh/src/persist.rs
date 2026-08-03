//! **A METADE AUTORADA de uma malha** — o que um documento guarda, e o que ele
//! manda RECONSTRUIR.
//!
//! Uma [`crate::Mesh`] carrega nove campos e o artista autorou **quatro**:
//! posições, faces, cor e máscara. Os outros cinco — normais de vértice, normais
//! de face, adjacência CSR, octree e caixa — são **função** desses quatro, e
//! `Mesh::from_parts` é quem os calcula.
//!
//! ⚠️ **Guardar um derivado não é só desperdício de bytes, é uma SEGUNDA CÓPIA de
//! um fato.** Um octree gravado hoje e lido por um binário cujo `Octree` mudou de
//! critério descreveria uma partição que a malha não tem — e nada na tela diria
//! por quê, porque uma consulta errada devolve um triângulo *plausível*. É a
//! mesma lei que mantém o mundo do rapier fora do arquivo (ADR-0131 D2: *o mundo
//! é derivado dos componentes*) e o grafo do Motion como texto re-cozido.
//!
//! ⚠️ **E é por isso que a validação acontece na LEITURA, não na escrita:**
//! `from_data` passa pelo mesmo `from_parts` que um OBJ de terceiro atravessa,
//! então um documento corrompido é recusado com a mesma mensagem — em vez de
//! virar uma leitura fora de alcance a três waves de distância.
//!
//! ⚠️ **Mudou a forma de qualquer tipo daqui? BUMPE o `SCULPT_DOC_VERSION` do
//! shell.** Isto não é pedido de disciplina: o gate `the_shape_of_a_saved_mesh_is_pinned`
//! prende o tamanho codificado de uma malha-fixture, então acrescentar um campo
//! deixa a suíte VERMELHA com a instrução dentro da mensagem.

use serde::{Deserialize, Serialize};

use crate::{Face, Mesh, MeshError, Pose};

/// A malha como um documento a guarda: **só o que o artista autorou**.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MeshData {
    pub positions: Vec<[f32; 3]>,
    pub faces: Vec<Face>,
    /// `None` = nunca pintada (a malha nasce sem o plano, e é isso que o
    /// documento preserva — materializá-lo no load alocaria um plano que o
    /// artista não pediu).
    pub colors: Option<Vec<[f32; 3]>>,
    /// `None` = ninguém mascarou.
    pub masks: Option<Vec<f32>>,
}

/// O detalhe de um nível de multiresolução contra o nível abaixo.
///
/// ⚠️ **Ele é AUTORADO, e não derivado** — é a diferença que o artista esculpiu
/// no nível de cima, projetada no frame local do de baixo. Sem ele, subir um
/// nível depois de um load devolveria a subdivisão lisa e **apagaria o trabalho
/// fino**, que é exatamente o que a multiresolução existe para guardar.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct DetailData {
    pub xyz: Vec<[f32; 3]>,
    pub colors: Option<Vec<[f32; 3]>>,
    pub masks: Option<Vec<f32>>,
}

/// A pilha inteira: os níveis, os detalhes entre eles, e onde a mão está.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct StackData {
    pub levels: Vec<MeshData>,
    /// `details[i]` é o detalhe do nível `i` contra o `i − 1`; o do nível 0 é
    /// vazio e nunca lido — a mesma invariante do [`Multires`] vivo.
    pub details: Vec<DetailData>,
    pub sel: u32,
}

/// A pose como um documento a guarda.
///
/// ⚠️ **A escala viaja crua e é re-CLAMPADA na leitura** (`Pose::new`), nunca
/// escrita de volta no campo privado: o piso que impede a divisão por zero é
/// invariante do tipo, e um documento é entrada de terceiro como qualquer outra.
/// *Estabelecer o invariante* custa uma chamada; *assumi-lo* custa um pick que
/// devolve infinito.
#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct PoseData {
    pub translation: [f32; 3],
    pub scale: f32,
}

/// Por que um documento foi recusado.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum DocError {
    /// A geometria não passou na validação que todo OBJ importado atravessa.
    Mesh(MeshError),
    /// Um plano por-vértice não mede a malha.
    PlaneLen {
        plane: &'static str,
        len: usize,
        verts: usize,
    },
    /// A pilha tem um número de detalhes que não casa com o de níveis.
    StackShape { levels: usize, details: usize },
    /// O nível selecionado não existe.
    LevelOutOfRange { sel: usize, levels: usize },
    /// Uma pilha sem nível nenhum — não há malha a mostrar.
    EmptyStack,
}

impl core::fmt::Display for DocError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::Mesh(e) => write!(f, "{e}"),
            Self::PlaneLen { plane, len, verts } => write!(
                f,
                "o plano de {plane} mede {len} e a malha tem {verts} vértices"
            ),
            Self::StackShape { levels, details } => write!(
                f,
                "a pilha tem {levels} níveis e {details} detalhes — têm de ser iguais"
            ),
            Self::LevelOutOfRange { sel, levels } => {
                write!(f, "nível selecionado {sel} numa pilha de {levels}")
            }
            Self::EmptyStack => write!(f, "pilha sem nível nenhum"),
        }
    }
}

impl core::error::Error for DocError {}

impl From<MeshError> for DocError {
    fn from(e: MeshError) -> Self {
        Self::Mesh(e)
    }
}

impl Mesh {
    /// A metade autorada desta malha — ver o doc do módulo.
    #[must_use]
    pub fn to_data(&self) -> MeshData {
        MeshData {
            positions: self.positions().to_vec(),
            faces: self.faces().to_vec(),
            colors: self.colors().map(<[[f32; 3]]>::to_vec),
            masks: self.masks().map(<[f32]>::to_vec),
        }
    }

    /// Reconstrói a malha a partir da metade autorada — **derivando** normais,
    /// adjacência, octree e caixa.
    ///
    /// # Errors
    /// Geometria inválida ou plano por-vértice com o tamanho errado.
    pub fn from_data(data: MeshData) -> Result<Self, DocError> {
        let verts = data.positions.len();
        let mut mesh = Self::from_parts(data.positions, data.faces)?;
        if let Some(colors) = data.colors {
            if colors.len() != verts {
                return Err(DocError::PlaneLen {
                    plane: "cor",
                    len: colors.len(),
                    verts,
                });
            }
            mesh.colors_mut().copy_from_slice(&colors);
        }
        if let Some(masks) = data.masks {
            if masks.len() != verts {
                return Err(DocError::PlaneLen {
                    plane: "máscara",
                    len: masks.len(),
                    verts,
                });
            }
            mesh.put_masks(masks);
        }
        Ok(mesh)
    }
}

impl Pose {
    /// A pose como um documento a guarda.
    #[must_use]
    pub fn to_data(&self) -> PoseData {
        PoseData {
            translation: self.translation,
            scale: self.scale(),
        }
    }

    /// Lê uma pose de um documento, **re-estabelecendo** o piso da escala.
    #[must_use]
    pub fn from_data(data: PoseData) -> Self {
        Self::new(data.translation, data.scale)
    }
}

// ⚠️ `Multires::to_data`/`from_data` NÃO moram aqui: eles precisam de
// `levels`/`details`/`sel`, que são privados do `multires.rs`, e um wrapper
// daqui só empurraria a mesma pergunta para um método que teria de abrir a
// privacidade. Eles são `pub` no filho `multires_persist.rs` — o tipo é público
// e re-exportado, então o módulo ser privado não os esconde de ninguém.

#[cfg(test)]
#[path = "persist_tests.rs"]
mod tests;
