//! **Os Kanbans do projecto** e os bytes com versão PRÓPRIA que viajam no `.ph2dproj` (campo irmão
//! do `boards`): mudar o formato daqui sobe [`FORMAT_VERSION`], nunca o `PROJECT_SCHEMA`.

use serde::{Deserialize, Serialize};

use crate::kanban::Kanban;
use crate::ops::{KanbanOp, apply_batch};
use crate::records::KanbanId;

/// A versão dos bytes de [`KanbanSet::to_bytes`].
pub const FORMAT_VERSION: u32 = 1;

/// Os Kanbans de um projecto, pela ordem em que nasceram.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct KanbanSet {
    kanbans: Vec<Kanban>,
    next_id: u64,
}

impl KanbanSet {
    /// Cria um Kanban vazio e devolve a chave dele.
    pub fn create(&mut self, name: &str) -> KanbanId {
        self.next_id += 1;
        let id = KanbanId(self.next_id);
        self.kanbans.push(Kanban::new(id, name));
        id
    }

    /// Põe um Kanban inteiro (o desfazer de um apagar, uma importação). Substitui o de mesma chave.
    pub fn insert(&mut self, k: Kanban) {
        self.next_id = self.next_id.max(k.id.0);
        match self.kanbans.iter_mut().find(|x| x.id == k.id) {
            Some(slot) => *slot = k,
            None => self.kanbans.push(k),
        }
    }

    /// Tira o Kanban `id` do projecto e devolve-o (para o desfazer o repor).
    pub fn remove(&mut self, id: KanbanId) -> Option<Kanban> {
        let i = self.kanbans.iter().position(|k| k.id == id)?;
        Some(self.kanbans.remove(i))
    }

    #[must_use]
    pub fn get(&self, id: KanbanId) -> Option<&Kanban> {
        self.kanbans.iter().find(|k| k.id == id)
    }

    /// O Kanban `id` para o [`crate::History`] dele mudar (só por ops: não há outra porta).
    pub fn doc_mut(&mut self, id: KanbanId) -> Option<&mut Kanban> {
        self.kanbans.iter_mut().find(|k| k.id == id)
    }

    /// Aplica um lote ao Kanban `id` e devolve o inverso (vazio se não existe ou nada mudou).
    pub fn apply(&mut self, id: KanbanId, ops: Vec<KanbanOp>) -> Vec<KanbanOp> {
        self.doc_mut(id)
            .map_or_else(Vec::new, |k| apply_batch(k, ops))
    }

    pub fn iter(&self) -> impl Iterator<Item = &Kanban> {
        self.kanbans.iter()
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.kanbans.is_empty()
    }

    /// Os bytes do conjunto: a versão e o conjunto, em postcard.
    ///
    /// # Errors
    /// Se o postcard recusar (não acontece com estes tipos; a porta é a do Quadro).
    pub fn to_bytes(&self) -> Result<Vec<u8>, String> {
        postcard::to_allocvec(&(FORMAT_VERSION, self)).map_err(|e| e.to_string())
    }

    /// Lê o que [`Self::to_bytes`] escreveu. Bytes vazios = projecto sem Kanbans.
    ///
    /// # Errors
    /// Versão desconhecida (um ficheiro de um PH2D mais novo) ou bytes corrompidos.
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, String> {
        if bytes.is_empty() {
            return Ok(Self::default());
        }
        let (version, _) = postcard::take_from_bytes::<u32>(bytes).map_err(|e| e.to_string())?;
        match version {
            FORMAT_VERSION => postcard::from_bytes::<(u32, Self)>(bytes)
                .map(|(_, set)| set)
                .map_err(|e| e.to_string()),
            v => Err(format!("kanban format version {v} != {FORMAT_VERSION}")),
        }
    }
}

#[cfg(test)]
#[path = "set_tests.rs"]
mod tests;
