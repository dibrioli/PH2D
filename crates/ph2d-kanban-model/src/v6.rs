//! **O formato v6, congelado** — só para LER ficheiros antigos (o degrau v6 → v7). Nunca se escreve.
//!
//! O v7 acrescentou `until` e `exceptions` no fim do [`Event`] (postcard é posicional). O resto lê-se
//! pelos tipos de hoje; o gate `v6_bytes_read_as_v7_equal` sobre a fixture gravada pelo código v6
//! guarda os dois lados. O v5 lê os eventos por [`EventV6`] também.

use std::collections::BTreeMap;

use serde::Deserialize;

use crate::activity::Activity;
use crate::agenda::{Block, Event, InboxItem, Repeat, Seen};
use crate::color::Hue;
use crate::files::{FileId, KanbanFile};
use crate::kanban::{Header, Kanban};
use crate::links::ProjectLink;
use crate::records::{
    ActivityId, Attachment, AttachmentId, Automation, AutomationId, BlockId, Card, CardId,
    CheckItem, CheckItemId, Checklist, ChecklistId, Comment, CommentId, EventId, FieldDef, FieldId,
    InboxId, KanbanId, Label, LabelId, List, ListId, Member, Meta, Mirror, MirrorId, Moment,
    ProjectLinkId, SeenId,
};

/// O evento até ao v6 (sem excepções).
#[derive(Deserialize)]
pub(crate) struct EventV6 {
    id: EventId,
    title: String,
    start: Moment,
    end: Moment,
    all_day: bool,
    repeat: Option<Repeat>,
    color: Option<Hue>,
    people: Vec<Member>,
    notes: String,
    remind_before_min: Option<u32>,
    meta: Meta,
}

impl EventV6 {
    pub(crate) fn upgrade(self) -> Event {
        let mut e = Event::new(self.id, &self.title, self.start, self.end, self.all_day);
        e.repeat = self.repeat;
        e.color = self.color;
        e.people = self.people;
        e.notes = self.notes;
        e.remind_before_min = self.remind_before_min;
        e.meta = self.meta;
        e
    }
}

/// Os eventos de uma tabela antiga, no tipo de hoje.
pub(crate) fn events(old: BTreeMap<EventId, EventV6>) -> BTreeMap<EventId, Event> {
    old.into_iter().map(|(k, e)| (k, e.upgrade())).collect()
}

#[derive(Deserialize)]
struct KanbanV6 {
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
    events: BTreeMap<EventId, EventV6>,
    inbox: BTreeMap<InboxId, InboxItem>,
    blocks: BTreeMap<BlockId, Block>,
    seen: BTreeMap<SeenId, Seen>,
    project_links: BTreeMap<ProjectLinkId, ProjectLink>,
}

/// O `KanbanSet` v6.
#[derive(Deserialize)]
pub(crate) struct SetV6 {
    kanbans: Vec<KanbanV6>,
    next_id: u64,
    files: BTreeMap<FileId, KanbanFile>,
    agenda: KanbanV6,
}

impl SetV6 {
    /// O v7 equivalente: eventos sem excepções.
    pub(crate) fn upgrade(self) -> (Vec<Kanban>, u64, BTreeMap<FileId, KanbanFile>, Kanban) {
        let kanbans = self.kanbans.into_iter().map(KanbanV6::upgrade).collect();
        (kanbans, self.next_id, self.files, self.agenda.upgrade())
    }
}

impl KanbanV6 {
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
        k.events = events(self.events);
        k.inbox = self.inbox;
        k.blocks = self.blocks;
        k.seen = self.seen;
        k.project_links = self.project_links;
        k
    }
}
