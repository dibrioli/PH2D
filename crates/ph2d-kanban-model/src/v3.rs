//! **O formato v3, congelado** — só para LER ficheiros antigos (o degrau v3 → v4). Nunca se escreve.
//!
//! O v4 só acrescentou a tabela `automations` no fim do Kanban (postcard é posicional). O resto lê-se
//! pelos tipos de hoje; o gate `v3_bytes_read_as_v4_equal` sobre a fixture gravada pelo código v3
//! guarda os dois lados.

use std::collections::BTreeMap;

use serde::Deserialize;

use crate::activity::Activity;
use crate::files::{FileId, KanbanFile};
use crate::kanban::{Header, Kanban};
use crate::records::{
    ActivityId, Attachment, AttachmentId, Card, CardId, CheckItem, CheckItemId, Checklist,
    ChecklistId, Comment, CommentId, FieldDef, FieldId, KanbanId, Label, LabelId, List, ListId,
    Mirror, MirrorId,
};

#[derive(Deserialize)]
struct KanbanV3 {
    id: KanbanId,
    header: Header,
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
    mirrors: BTreeMap<MirrorId, Mirror>,
}

/// O `KanbanSet` v3.
#[derive(Deserialize)]
pub(crate) struct SetV3 {
    kanbans: Vec<KanbanV3>,
    next_id: u64,
    files: BTreeMap<FileId, KanbanFile>,
}

impl SetV3 {
    /// O v4 equivalente: sem automações.
    pub(crate) fn upgrade(self) -> (Vec<Kanban>, u64, BTreeMap<FileId, KanbanFile>) {
        let kanbans = self.kanbans.into_iter().map(KanbanV3::upgrade).collect();
        (kanbans, self.next_id, self.files)
    }
}

impl KanbanV3 {
    fn upgrade(self) -> Kanban {
        let mut k = Kanban::new(self.id, &self.header.name);
        k.header = self.header;
        k.lists = self.lists;
        k.cards = self.cards;
        k.labels = self.labels;
        k.checklists = self.checklists;
        k.items = self.items;
        k.comments = self.comments;
        k.attachments = self.attachments;
        k.fields = self.fields;
        k.next_id = self.next_id;
        k.activities = self.activities;
        k.mirrors = self.mirrors;
        k
    }
}
