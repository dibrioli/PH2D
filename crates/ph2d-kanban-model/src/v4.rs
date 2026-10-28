//! **O formato v4, congelado** — só para LER ficheiros antigos (o degrau v4 → v5). Nunca se escreve.
//!
//! O v5 acrescentou quatro tabelas no fim do Kanban (`events`, `inbox`, `blocks`, `seen`) e a agenda
//! do projecto no fim do conjunto (postcard é posicional). O resto lê-se pelos tipos de hoje; o gate
//! `v4_bytes_read_as_v5_equal` sobre a fixture gravada pelo código v4 guarda os dois lados.

use std::collections::BTreeMap;

use serde::Deserialize;

use crate::activity::Activity;
use crate::files::{FileId, KanbanFile};
use crate::kanban::{Header, Kanban};
use crate::records::{
    ActivityId, Attachment, AttachmentId, Automation, AutomationId, Card, CardId, CheckItem,
    CheckItemId, Checklist, ChecklistId, Comment, CommentId, FieldDef, FieldId, KanbanId, Label,
    LabelId, List, ListId, Mirror, MirrorId,
};

#[derive(Deserialize)]
struct KanbanV4 {
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
    automations: BTreeMap<AutomationId, Automation>,
}

/// O `KanbanSet` v4.
#[derive(Deserialize)]
pub(crate) struct SetV4 {
    kanbans: Vec<KanbanV4>,
    next_id: u64,
    files: BTreeMap<FileId, KanbanFile>,
}

impl SetV4 {
    /// O v5 equivalente: sem eventos, Inbox, blocos nem lidas; a agenda vazia.
    pub(crate) fn upgrade(self) -> (Vec<Kanban>, u64, BTreeMap<FileId, KanbanFile>) {
        let kanbans = self.kanbans.into_iter().map(KanbanV4::upgrade).collect();
        (kanbans, self.next_id, self.files)
    }
}

impl KanbanV4 {
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
        k.automations = self.automations;
        k
    }
}
