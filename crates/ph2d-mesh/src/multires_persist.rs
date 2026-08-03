//! **A pilha como um documento a guarda.**
//!
//! Filho (`#[path]`) de [`super`] porque `levels`/`details`/`sel` e o próprio
//! [`Details`] são privados — e é assim que se quer: *o que uma pilha É* é
//! assunto desta crate, e um serializador escrito no shell seria uma **segunda
//! casa** que sabe do que ela é feita, divergindo no dia em que a codificação do
//! detalhe mudar. Arquivo próprio pelo teto de LOC (HR-18).

use super::{Details, Multires};
use crate::Mesh;
use crate::persist::{DetailData, DocError, StackData};

impl Multires {
    /// A pilha inteira como um documento a guarda — ver [`crate::persist`].
    #[must_use]
    pub fn to_data(&self) -> StackData {
        StackData {
            levels: self.levels.iter().map(Mesh::to_data).collect(),
            details: self
                .details
                .iter()
                .map(|d| DetailData {
                    xyz: d.xyz.clone(),
                    colors: d.colors.clone(),
                    masks: d.masks.clone(),
                })
                .collect(),
            sel: self.sel as u32,
        }
    }

    /// Reconstrói a pilha, **derivando** o que é derivável em cada nível.
    ///
    /// # Errors
    /// Pilha vazia, contagem de detalhes que não casa com a de níveis, nível
    /// selecionado fora de alcance, ou um nível cuja geometria não valida.
    pub fn from_data(data: StackData) -> Result<Self, DocError> {
        if data.levels.is_empty() {
            return Err(DocError::EmptyStack);
        }
        // ⚠️ As três checagens são feitas ANTES de reconstruir um único nível, e
        // não porque validar cedo é elegante: reconstruir é `O(vértices)` com
        // octree e adjacência por nível, e um documento truncado não merece esse
        // trabalho antes de ser recusado.
        if data.levels.len() != data.details.len() {
            return Err(DocError::StackShape {
                levels: data.levels.len(),
                details: data.details.len(),
            });
        }
        let sel = data.sel as usize;
        if sel >= data.levels.len() {
            return Err(DocError::LevelOutOfRange {
                sel,
                levels: data.levels.len(),
            });
        }
        let mut levels = Vec::with_capacity(data.levels.len());
        for level in data.levels {
            levels.push(Mesh::from_data(level)?);
        }
        let details = data
            .details
            .into_iter()
            .map(|d| Details {
                xyz: d.xyz,
                colors: d.colors,
                masks: d.masks,
            })
            .collect();
        Ok(Self {
            levels,
            details,
            sel,
        })
    }
}
