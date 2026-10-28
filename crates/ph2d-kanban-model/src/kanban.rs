//! **Um Kanban**: o cabeçalho e as tabelas de registos. Lê-se à vontade; muda-se só por
//! [`crate::KanbanOp`].

use std::collections::BTreeMap;

use ph2d_frac::FracKey;
use serde::{Deserialize, Serialize};

use crate::color::Hue;
use crate::records::{
    Attachment, AttachmentId, Card, CardId, CheckItem, CheckItemId, Checklist, ChecklistId,
    Comment, CommentId, FieldDef, FieldId, KanbanId, Label, LabelId, List, ListId,
};

/// O fundo do Kanban.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Background {
    Color(Hue),
    /// Um dos gradientes dos tokens do Kanban, pelo índice.
    Gradient(u16),
    /// Uma imagem do projecto pela chave do conteúdo (ADR-0184).
    Image([u8; 32]),
}

/// O que é do Kanban inteiro (um registo só, com versão própria).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Header {
    pub name: String,
    pub starred: bool,
    pub background: Background,
    pub version: u64,
}

/// Um Kanban do projecto.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Kanban {
    pub id: KanbanId,
    pub header: Header,
    pub lists: BTreeMap<ListId, List>,
    pub cards: BTreeMap<CardId, Card>,
    pub labels: BTreeMap<LabelId, Label>,
    pub checklists: BTreeMap<ChecklistId, Checklist>,
    pub items: BTreeMap<CheckItemId, CheckItem>,
    pub comments: BTreeMap<CommentId, Comment>,
    pub attachments: BTreeMap<AttachmentId, Attachment>,
    pub fields: BTreeMap<FieldId, FieldDef>,
    /// O maior id local já dado a um registo (os ids são de UM espaço por Kanban).
    pub(crate) next_id: u64,
}

impl Kanban {
    /// Um Kanban vazio, de fundo azul (o de nascença do Trello).
    #[must_use]
    pub fn new(id: KanbanId, name: &str) -> Self {
        Self {
            id,
            header: Header {
                name: name.to_owned(),
                starred: false,
                background: Background::Color(Hue::Blue),
                version: 0,
            },
            lists: BTreeMap::new(),
            cards: BTreeMap::new(),
            labels: BTreeMap::new(),
            checklists: BTreeMap::new(),
            items: BTreeMap::new(),
            comments: BTreeMap::new(),
            attachments: BTreeMap::new(),
            fields: BTreeMap::new(),
            next_id: 0,
        }
    }

    /// Um id novo para um registo deste Kanban. Não muda o documento: o `Put` que o usa sobe o
    /// contador (dois ids pedidos sem `Put` no meio são o MESMO — peça-os todos com [`Self::mint_n`]).
    #[must_use]
    pub fn mint(&self) -> u64 {
        self.next_id + 1
    }

    /// `n` ids novos e seguidos, para um gesto que cria vários registos.
    #[must_use]
    pub fn mint_n(&self, n: usize) -> Vec<u64> {
        (1..=n as u64).map(|i| self.next_id + i).collect()
    }

    /// As listas vivas e não arquivadas, pela ordem (`pos`, e o id desempata).
    #[must_use]
    pub fn lists_in_order(&self) -> Vec<&List> {
        let mut v: Vec<&List> = self
            .lists
            .values()
            .filter(|l| !l.meta.deleted && !l.archived)
            .collect();
        v.sort_by(|a, b| a.pos.cmp(&b.pos).then(a.id.cmp(&b.id)));
        v
    }

    /// Os cartões vivos e não arquivados de `list`, pela ordem.
    #[must_use]
    pub fn cards_in(&self, list: ListId) -> Vec<&Card> {
        let mut v: Vec<&Card> = self
            .cards
            .values()
            .filter(|c| c.list == list && !c.meta.deleted && !c.archived)
            .collect();
        v.sort_by(|a, b| a.pos.cmp(&b.pos).then(a.id.cmp(&b.id)));
        v
    }

    /// A posição depois da última lista viva (arquivadas contam: desarquivar não colide).
    #[must_use]
    pub fn pos_after_lists(&self) -> FracKey {
        let last = self
            .lists
            .values()
            .filter(|l| !l.meta.deleted)
            .map(|l| &l.pos)
            .max();
        FracKey::between(last, None)
    }

    /// A posição depois do último cartão de `list`.
    #[must_use]
    pub fn pos_after_cards(&self, list: ListId) -> FracKey {
        let last = self
            .cards
            .values()
            .filter(|c| c.list == list && !c.meta.deleted)
            .map(|c| &c.pos)
            .max();
        FracKey::between(last, None)
    }
}
