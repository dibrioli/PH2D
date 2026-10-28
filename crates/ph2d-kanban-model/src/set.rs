//! **Os Kanbans do projecto** e os bytes com versão PRÓPRIA que viajam no `.ph2dproj` (campo irmão
//! do `boards`): mudar o formato daqui sobe [`FORMAT_VERSION`], nunca o `PROJECT_SCHEMA`.

use std::collections::{BTreeMap, BTreeSet};
use std::sync::Arc;

use serde::{Deserialize, Serialize};

use crate::files::{FileId, KanbanFile};
use crate::kanban::{Background, Kanban};
use crate::ops::{KanbanOp, apply_batch};
use crate::records::{CoverFill, KanbanId, Link};

/// A versão dos bytes de [`KanbanSet::to_bytes`]. v2 (W3): seguir, atividade, origem do cartão,
/// ficheiros por conteúdo. v3 (W6): modelos de cartão, envelhecimento, espelhos. v4 (W8): as
/// automações. v5 (W9): a agenda do projecto (eventos, Inbox, Planner, lidas). v6 (W11): as ligações
/// dos cartões ao projecto. v7 (W9 P2): as excepções das séries de eventos. As antigas leem-se pelos
/// degraus de [`crate::v1`]..[`crate::v6`].
pub const FORMAT_VERSION: u32 = 7;

/// Os Kanbans de um projecto, pela ordem em que nasceram.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct KanbanSet {
    kanbans: Vec<Kanban>,
    next_id: u64,
    /// Os bytes das imagens e anexos, uma vez por conteúdo (v2).
    files: BTreeMap<FileId, KanbanFile>,
    /// A agenda do projecto (v5, [`KanbanId::AGENDA`]): fora de [`Self::iter`].
    agenda: Kanban,
}

impl Default for KanbanSet {
    fn default() -> Self {
        Self::with_kanbans(Vec::new(), 0, BTreeMap::new())
    }
}

/// O que [`KanbanSet::to_bytes`] escreve: o conjunto sem os ficheiros órfãos, por empréstimo (os
/// mesmos bytes que o `KanbanSet`, campo a campo).
#[derive(Serialize)]
struct Saved<'a> {
    kanbans: &'a [Kanban],
    next_id: u64,
    files: BTreeMap<&'a FileId, &'a KanbanFile>,
    agenda: &'a Kanban,
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
        if k.id == KanbanId::AGENDA {
            self.agenda = k;
            return;
        }
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
        if id == KanbanId::AGENDA {
            return Some(&self.agenda);
        }
        self.kanbans.iter().find(|k| k.id == id)
    }

    /// A agenda do projecto (eventos, Inbox, Planner, lidas).
    #[must_use]
    pub fn agenda(&self) -> &Kanban {
        &self.agenda
    }

    fn with_kanbans(
        kanbans: Vec<Kanban>,
        next_id: u64,
        files: BTreeMap<FileId, KanbanFile>,
    ) -> Self {
        Self {
            kanbans,
            next_id,
            files,
            agenda: Kanban::new(KanbanId::AGENDA, ""),
        }
    }

    /// O Kanban `id` para o [`crate::History`] dele mudar (só por ops: não há outra porta).
    pub fn doc_mut(&mut self, id: KanbanId) -> Option<&mut Kanban> {
        if id == KanbanId::AGENDA {
            return Some(&mut self.agenda);
        }
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
        let used = self.files_in_use();
        let saved = Saved {
            kanbans: &self.kanbans,
            next_id: self.next_id,
            files: self
                .files
                .iter()
                .filter(|(id, _)| used.contains(id))
                .collect(),
            agenda: &self.agenda,
        };
        postcard::to_allocvec(&(FORMAT_VERSION, saved)).map_err(|e| e.to_string())
    }

    /// Guarda os bytes de um ficheiro (imagem: `size` = o tamanho natural) e devolve a chave. O
    /// mesmo conteúdo duas vezes é UM ficheiro.
    pub fn put_file(&mut self, bytes: impl Into<Arc<[u8]>>, size: Option<[u32; 2]>) -> FileId {
        let f = KanbanFile::new(bytes, size);
        let id = f.id();
        self.files.entry(id).or_insert(f);
        id
    }

    /// [`Self::put_file`] com a chave já calculada ([`FileId::of`], num fio de fundo: o sha-256 de um
    /// anexo grande não pesa na janela). ⛔ `id` TEM de ser a chave dos bytes (verificado em debug).
    pub fn put_file_keyed(
        &mut self,
        id: FileId,
        bytes: impl Into<Arc<[u8]>>,
        size: Option<[u32; 2]>,
    ) -> FileId {
        let f = KanbanFile::new(bytes, size);
        debug_assert_eq!(
            id,
            f.id(),
            "put_file_keyed: a chave não é o sha-256 dos bytes"
        );
        self.files.entry(id).or_insert(f);
        id
    }

    #[must_use]
    pub fn file(&self, id: FileId) -> Option<&KanbanFile> {
        self.files.get(&id)
    }

    /// Os ficheiros que algum registo VIVO usa (capa, anexo, fundo; arquivado conta). Os outros
    /// ficam na memória (o desfazer da sessão ainda os pode querer) e não vão para o disco.
    #[must_use]
    pub fn files_in_use(&self) -> BTreeSet<FileId> {
        let mut used = BTreeSet::new();
        for k in &self.kanbans {
            if let Background::Image(f) = k.header.background {
                used.insert(f);
            }
            for c in k.cards.values().filter(|c| !c.meta.deleted) {
                if let Some(CoverFill::Image(f)) = c.cover.map(|c| c.fill) {
                    used.insert(f);
                }
            }
            for a in k.attachments.values().filter(|a| !a.meta.deleted) {
                if let Link::File(f) = a.link {
                    used.insert(f);
                }
            }
        }
        used
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
            6 => postcard::from_bytes::<(u32, crate::v6::SetV6)>(bytes)
                .map(|(_, old)| {
                    let (kanbans, next_id, files, agenda) = old.upgrade();
                    let mut set = Self::with_kanbans(kanbans, next_id, files);
                    set.agenda = agenda;
                    set
                })
                .map_err(|e| e.to_string()),
            5 => postcard::from_bytes::<(u32, crate::v5::SetV5)>(bytes)
                .map(|(_, old)| {
                    let (kanbans, next_id, files, agenda) = old.upgrade();
                    let mut set = Self::with_kanbans(kanbans, next_id, files);
                    set.agenda = agenda;
                    set
                })
                .map_err(|e| e.to_string()),
            4 => postcard::from_bytes::<(u32, crate::v4::SetV4)>(bytes)
                .map(|(_, old)| {
                    let (kanbans, next_id, files) = old.upgrade();
                    Self::with_kanbans(kanbans, next_id, files)
                })
                .map_err(|e| e.to_string()),
            3 => postcard::from_bytes::<(u32, crate::v3::SetV3)>(bytes)
                .map(|(_, old)| {
                    let (kanbans, next_id, files) = old.upgrade();
                    Self::with_kanbans(kanbans, next_id, files)
                })
                .map_err(|e| e.to_string()),
            2 => postcard::from_bytes::<(u32, crate::v2::SetV2)>(bytes)
                .map(|(_, old)| {
                    let (kanbans, next_id, files) = old.upgrade();
                    Self::with_kanbans(kanbans, next_id, files)
                })
                .map_err(|e| e.to_string()),
            1 => postcard::from_bytes::<(u32, crate::v1::SetV1)>(bytes)
                .map(|(_, old)| {
                    let (kanbans, next_id) = old.upgrade();
                    Self::with_kanbans(kanbans, next_id, BTreeMap::new())
                })
                .map_err(|e| e.to_string()),
            v => Err(format!("kanban format version {v} != {FORMAT_VERSION}")),
        }
    }
}

#[cfg(test)]
#[path = "set_tests.rs"]
pub(crate) mod tests;
