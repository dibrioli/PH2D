//! **O formato v1, congelado** — só para LER ficheiros antigos (o degrau v1 → hoje). Nunca se escreve.
//!
//! Estão aqui só os tipos cujo desenho o v2 mudou (postcard é posicional: um campo a mais desloca
//! tudo). Os registos que o v2 não tocou (etiqueta, checklist, item, comentário, anexo, campo, capa,
//! fundo — `[u8; 32]` e `FileId` são os mesmos 32 bytes) leem-se pelos tipos de hoje; o gate
//! `v1_bytes_read_as_v2_equal` sobre a fixture gravada pelo código v1 guarda os dois lados.

use std::collections::BTreeMap;

use ph2d_frac::FracKey;
use serde::Deserialize;

use crate::CardId;
use crate::kanban::{Background, Header, Kanban};
use crate::records::{
    Attachment, AttachmentId, CheckItem, CheckItemId, Checklist, ChecklistId, Comment, CommentId,
    Cover, Due, FieldDef, FieldId, FieldValue, KanbanId, Label, LabelId, ListId, Member, Meta,
    Moment,
};

#[derive(Deserialize)]
struct List {
    id: ListId,
    title: String,
    pos: FracKey,
    color: Option<crate::Hue>,
    card_limit: Option<u32>,
    archived: bool,
    meta: Meta,
}

#[derive(Deserialize)]
struct Card {
    id: CardId,
    list: ListId,
    pos: FracKey,
    title: String,
    description: String,
    labels: Vec<LabelId>,
    members: Vec<Member>,
    start: Option<Moment>,
    due: Option<Due>,
    cover: Option<Cover>,
    fields: BTreeMap<FieldId, FieldValue>,
    location: String,
    votes: Vec<Member>,
    created: Moment,
    archived: bool,
    meta: Meta,
}

#[derive(Deserialize)]
struct HeaderV1 {
    name: String,
    starred: bool,
    background: Background,
    version: u64,
}

#[derive(Deserialize)]
struct KanbanV1 {
    id: KanbanId,
    header: HeaderV1,
    lists: BTreeMap<ListId, List>,
    cards: BTreeMap<CardId, Card>,
    labels: BTreeMap<LabelId, Label>,
    checklists: BTreeMap<ChecklistId, Checklist>,
    items: BTreeMap<CheckItemId, CheckItem>,
    comments: BTreeMap<CommentId, Comment>,
    attachments: BTreeMap<AttachmentId, Attachment>,
    fields: BTreeMap<FieldId, FieldDef>,
    next_id: u64,
}

/// O `KanbanSet` v1: os Kanbans e o contador (sem ficheiros).
#[derive(Deserialize)]
pub(crate) struct SetV1 {
    kanbans: Vec<KanbanV1>,
    next_id: u64,
}

impl SetV1 {
    /// O v2 equivalente: os campos novos nascem vazios (ninguém segue nada, sem história, sem
    /// ficheiros guardados — as imagens v1 nunca chegaram a ter bytes no conjunto).
    pub(crate) fn upgrade(self) -> (Vec<Kanban>, u64) {
        let kanbans = self.kanbans.into_iter().map(KanbanV1::upgrade).collect();
        (kanbans, self.next_id)
    }
}

impl KanbanV1 {
    fn upgrade(self) -> Kanban {
        let h = self.header;
        let mut k = Kanban::new(self.id, &h.name);
        k.header = Header {
            name: h.name,
            starred: h.starred,
            background: h.background,
            version: h.version,
            watchers: Vec::new(),
            aging: None,
        };
        k.lists = self
            .lists
            .into_iter()
            .map(|(id, l)| (id, l.upgrade()))
            .collect();
        k.cards = self
            .cards
            .into_iter()
            .map(|(id, c)| (id, c.upgrade()))
            .collect();
        k.labels = self.labels;
        k.checklists = self.checklists;
        k.items = self.items;
        k.comments = self.comments;
        k.attachments = self.attachments;
        k.fields = self.fields;
        k.next_id = self.next_id;
        k
    }
}

impl List {
    fn upgrade(self) -> crate::List {
        crate::List {
            id: self.id,
            title: self.title,
            pos: self.pos,
            color: self.color,
            card_limit: self.card_limit,
            archived: self.archived,
            meta: self.meta,
            watchers: Vec::new(),
        }
    }
}

impl Card {
    fn upgrade(self) -> crate::Card {
        crate::Card {
            id: self.id,
            list: self.list,
            pos: self.pos,
            title: self.title,
            description: self.description,
            labels: self.labels,
            members: self.members,
            start: self.start,
            due: self.due,
            cover: self.cover,
            fields: self.fields,
            location: self.location,
            votes: self.votes,
            created: self.created,
            archived: self.archived,
            meta: self.meta,
            watchers: Vec::new(),
            origin: None,
            template: false,
        }
    }
}
