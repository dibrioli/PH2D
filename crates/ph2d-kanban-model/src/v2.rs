//! **O formato v2, congelado** — só para LER ficheiros antigos (o degrau v2 → v3). Nunca se escreve.
//!
//! O v3 mudou três desenhos (postcard é posicional): o cartão ganhou `template`, o cabeçalho
//! `aging` e o Kanban a tabela `mirrors`. O resto lê-se pelos tipos de hoje; o gate
//! `v2_bytes_read_as_v3_equal` sobre a fixture gravada pelo código v2 guarda os dois lados.

use std::collections::BTreeMap;

use ph2d_frac::FracKey;
use serde::Deserialize;

use crate::activity::Activity;
use crate::files::{FileId, KanbanFile};
use crate::kanban::{Background, Header, Kanban};
use crate::records::{
    ActivityId, Attachment, AttachmentId, CardId, CheckItem, CheckItemId, Checklist, ChecklistId,
    Comment, CommentId, Cover, Due, FieldDef, FieldId, FieldValue, KanbanId, Label, LabelId, List,
    ListId, Member, Meta, Moment,
};

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
    watchers: Vec<Member>,
    origin: Option<CardId>,
}

#[derive(Deserialize)]
struct HeaderV2 {
    name: String,
    starred: bool,
    background: Background,
    version: u64,
    watchers: Vec<Member>,
}

#[derive(Deserialize)]
struct KanbanV2 {
    id: KanbanId,
    header: HeaderV2,
    lists: BTreeMap<ListId, List>,
    cards: BTreeMap<CardId, Card>,
    labels: BTreeMap<LabelId, Label>,
    checklists: BTreeMap<ChecklistId, Checklist>,
    items: BTreeMap<CheckItemId, CheckItem>,
    comments: BTreeMap<CommentId, Comment>,
    attachments: BTreeMap<AttachmentId, Attachment>,
    fields: BTreeMap<FieldId, FieldDef>,
    next_id: u64,
    activities: BTreeMap<ActivityId, Activity>,
}

/// O `KanbanSet` v2.
#[derive(Deserialize)]
pub(crate) struct SetV2 {
    kanbans: Vec<KanbanV2>,
    next_id: u64,
    files: BTreeMap<FileId, KanbanFile>,
}

impl SetV2 {
    /// O v3 equivalente: nenhum modelo, envelhecimento desligado, sem espelhos.
    pub(crate) fn upgrade(self) -> (Vec<Kanban>, u64, BTreeMap<FileId, KanbanFile>) {
        let kanbans = self.kanbans.into_iter().map(KanbanV2::upgrade).collect();
        (kanbans, self.next_id, self.files)
    }
}

impl KanbanV2 {
    fn upgrade(self) -> Kanban {
        let h = self.header;
        let mut k = Kanban::new(self.id, &h.name);
        k.header = Header {
            name: h.name,
            starred: h.starred,
            background: h.background,
            version: h.version,
            watchers: h.watchers,
            aging: None,
        };
        k.lists = self.lists;
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
        k.activities = self.activities;
        k
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
            watchers: self.watchers,
            origin: self.origin,
            template: false,
        }
    }
}
