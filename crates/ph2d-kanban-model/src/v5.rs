//! **O formato v5, congelado** — só para LER ficheiros antigos (o degrau v5 → v6). Nunca se escreve.
//!
//! O v6 acrescentou a tabela `project_links` no fim do Kanban (postcard é posicional). O resto lê-se
//! pelos tipos de hoje (os eventos pelo [`crate::v6::EventV6`]); o gate `v5_bytes_read_as_v6_equal` sobre a fixture gravada pelo código v5
//! guarda os dois lados.

use std::collections::BTreeMap;

use serde::Deserialize;

use crate::activity::Activity;
use crate::agenda::{Block, InboxItem, Seen};
use crate::files::{FileId, KanbanFile};
use crate::kanban::{Header, Kanban};
use crate::records::{
    ActivityId, Attachment, AttachmentId, Automation, AutomationId, BlockId, Card, CardId,
    CheckItem, CheckItemId, Checklist, ChecklistId, Comment, CommentId, EventId, FieldDef, FieldId,
    InboxId, KanbanId, Label, LabelId, List, ListId, Mirror, MirrorId, SeenId,
};

#[derive(Deserialize)]
struct KanbanV5 {
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
    events: BTreeMap<EventId, crate::v6::EventV6>,
    inbox: BTreeMap<InboxId, InboxItem>,
    blocks: BTreeMap<BlockId, Block>,
    seen: BTreeMap<SeenId, Seen>,
}

/// O `KanbanSet` v5.
#[derive(Deserialize)]
pub(crate) struct SetV5 {
    kanbans: Vec<KanbanV5>,
    next_id: u64,
    files: BTreeMap<FileId, KanbanFile>,
    agenda: KanbanV5,
}

impl SetV5 {
    /// O v6 equivalente: sem ligações ao projecto.
    pub(crate) fn upgrade(self) -> (Vec<Kanban>, u64, BTreeMap<FileId, KanbanFile>, Kanban) {
        let kanbans = self.kanbans.into_iter().map(KanbanV5::upgrade).collect();
        (kanbans, self.next_id, self.files, self.agenda.upgrade())
    }
}

impl KanbanV5 {
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
        k.events = crate::v6::events(self.events);
        k.inbox = self.inbox;
        k.blocks = self.blocks;
        k.seen = self.seen;
        k
    }
}
