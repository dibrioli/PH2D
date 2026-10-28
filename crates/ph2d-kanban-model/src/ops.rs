//! **As operações** — a ÚNICA porta de mudança de um [`Kanban`]. O desfazer guarda o inverso que
//! [`KanbanOp::apply`] devolve; a colaboração (W5) transmitirá as mesmas operações.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::activity::Activity;
use crate::kanban::{Header, Kanban};
use crate::records::{
    ActivityId, Attachment, AttachmentId, Card, CardId, CheckItem, CheckItemId, Checklist,
    ChecklistId, Comment, CommentId, FieldDef, FieldId, Label, LabelId, List, ListId, Meta,
};

/// Um registo inteiro, de qualquer tabela.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum Record {
    List(List),
    Card(Card),
    Label(Label),
    Checklist(Checklist),
    CheckItem(CheckItem),
    Comment(Comment),
    Attachment(Attachment),
    Field(FieldDef),
    /// v2 — uma entrada da história.
    Activity(Activity),
}

/// A chave de um registo, de qualquer tabela.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum RecordId {
    List(ListId),
    Card(CardId),
    Label(LabelId),
    Checklist(ChecklistId),
    CheckItem(CheckItemId),
    Comment(CommentId),
    Attachment(AttachmentId),
    Field(FieldId),
    Activity(ActivityId),
}

/// Uma mudança num Kanban.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum KanbanOp {
    /// Põe o registo (novo, ou a reposição de um estado anterior inteiro). Em caixa: um cartão é
    /// grande e um `Delete` é um id.
    Put(Box<Record>),
    /// Apaga (lápide) o registo.
    Delete(RecordId),
    /// Põe o cabeçalho (nome, favorito, fundo).
    Header(Header),
}

/// O que as tabelas têm em comum.
trait Rec: Clone + PartialEq + Sized {
    type Id: Copy + Ord;
    fn id(&self) -> Self::Id;
    fn meta(&self) -> &Meta;
    fn meta_mut(&mut self) -> &mut Meta;
    fn table(k: &mut Kanban) -> &mut BTreeMap<Self::Id, Self>;
    fn wrap(self) -> Record;
    fn key(id: Self::Id) -> RecordId;
    fn raw(id: Self::Id) -> u64;
}

macro_rules! rec {
    ($ty:ident, $id:ident, $table:ident, $variant:ident) => {
        impl Rec for $ty {
            type Id = $id;
            fn id(&self) -> $id {
                self.id
            }
            fn meta(&self) -> &Meta {
                &self.meta
            }
            fn meta_mut(&mut self) -> &mut Meta {
                &mut self.meta
            }
            fn table(k: &mut Kanban) -> &mut BTreeMap<$id, Self> {
                &mut k.$table
            }
            fn wrap(self) -> Record {
                Record::$variant(self)
            }
            fn key(id: $id) -> RecordId {
                RecordId::$variant(id)
            }
            fn raw(id: $id) -> u64 {
                id.0
            }
        }
    };
}

rec!(List, ListId, lists, List);
rec!(Card, CardId, cards, Card);
rec!(Label, LabelId, labels, Label);
rec!(Checklist, ChecklistId, checklists, Checklist);
rec!(CheckItem, CheckItemId, items, CheckItem);
rec!(Comment, CommentId, comments, Comment);
rec!(Attachment, AttachmentId, attachments, Attachment);
rec!(FieldDef, FieldId, fields, Field);
rec!(Activity, ActivityId, activities, Activity);

/// O bit dos ids sorteados numa sessão ao vivo (W5) — esses não sobem o contador local.
const SESSION_ID_BIT: u64 = 1 << 63;

fn put<R: Rec>(k: &mut Kanban, mut r: R) -> Option<KanbanOp> {
    let id = r.id();
    let raw = R::raw(id);
    if raw & SESSION_ID_BIT == 0 {
        k.next_id = k.next_id.max(raw);
    }
    let table = R::table(k);
    let prev = table.get(&id).cloned();
    if let Some(mut p) = prev.clone() {
        // O mesmo conteúdo não é mudança (um arrasto que voltou ao sítio não é passo de desfazer).
        // A lápide é conteúdo: reviver um apagado é mudança. Só a versão e o desempate não contam.
        let Meta { version, nonce, .. } = *r.meta();
        (p.meta_mut().version, p.meta_mut().nonce) = (version, nonce);
        if p == r {
            return None;
        }
    }
    let before = prev.as_ref().map_or(0, |p| p.meta().version);
    let m = r.meta_mut();
    m.version = before.max(m.version) + 1;
    table.insert(id, r);
    Some(match prev {
        Some(p) if !p.meta().deleted => KanbanOp::put(p.wrap()),
        _ => KanbanOp::Delete(R::key(id)),
    })
}

fn delete<R: Rec>(k: &mut Kanban, id: R::Id) -> Option<KanbanOp> {
    let r = R::table(k).get_mut(&id).filter(|r| !r.meta().deleted)?;
    let before = r.clone();
    let m = r.meta_mut();
    m.deleted = true;
    m.version += 1;
    Some(KanbanOp::put(before.wrap()))
}

impl KanbanOp {
    /// `Put` de um registo.
    #[must_use]
    pub fn put(r: Record) -> Self {
        Self::Put(Box::new(r))
    }

    /// Aplica a operação e devolve a que a desfaz. `None` = não mudou nada (id desconhecido, já
    /// apagado, ou o mesmo conteúdo) — quem chama não empilha um passo vazio.
    pub fn apply(self, k: &mut Kanban) -> Option<KanbanOp> {
        match self {
            KanbanOp::Put(r) => match *r {
                Record::List(r) => put(k, r),
                Record::Card(r) => put(k, r),
                Record::Label(r) => put(k, r),
                Record::Checklist(r) => put(k, r),
                Record::CheckItem(r) => put(k, r),
                Record::Comment(r) => put(k, r),
                Record::Attachment(r) => put(k, r),
                Record::Field(r) => put(k, r),
                Record::Activity(r) => put(k, r),
            },
            KanbanOp::Delete(id) => match id {
                RecordId::List(id) => delete::<List>(k, id),
                RecordId::Card(id) => delete::<Card>(k, id),
                RecordId::Label(id) => delete::<Label>(k, id),
                RecordId::Checklist(id) => delete::<Checklist>(k, id),
                RecordId::CheckItem(id) => delete::<CheckItem>(k, id),
                RecordId::Comment(id) => delete::<Comment>(k, id),
                RecordId::Attachment(id) => delete::<Attachment>(k, id),
                RecordId::Field(id) => delete::<FieldDef>(k, id),
                RecordId::Activity(id) => delete::<Activity>(k, id),
            },
            KanbanOp::Header(mut h) => {
                let mut prev = k.header.clone();
                prev.version = h.version;
                if prev == h {
                    return None;
                }
                prev.version = k.header.version;
                h.version = prev.version.max(h.version) + 1;
                k.header = h;
                Some(KanbanOp::Header(prev))
            }
        }
    }

    /// O registo que a operação muda (`None` = o cabeçalho).
    #[must_use]
    pub fn target(&self) -> Option<RecordId> {
        match self {
            KanbanOp::Put(r) => Some(r.id()),
            KanbanOp::Delete(id) => Some(*id),
            KanbanOp::Header(_) => None,
        }
    }
}

impl Record {
    /// A chave deste registo.
    #[must_use]
    pub fn id(&self) -> RecordId {
        match self {
            Record::List(r) => RecordId::List(r.id),
            Record::Card(r) => RecordId::Card(r.id),
            Record::Label(r) => RecordId::Label(r.id),
            Record::Checklist(r) => RecordId::Checklist(r.id),
            Record::CheckItem(r) => RecordId::CheckItem(r.id),
            Record::Comment(r) => RecordId::Comment(r.id),
            Record::Attachment(r) => RecordId::Attachment(r.id),
            Record::Field(r) => RecordId::Field(r.id),
            Record::Activity(r) => RecordId::Activity(r.id),
        }
    }
}

impl Kanban {
    /// O registo `id` tal como está agora (apagado incluído; `None` = nunca existiu).
    #[must_use]
    pub fn record(&self, id: RecordId) -> Option<Record> {
        match id {
            RecordId::List(id) => self.lists.get(&id).cloned().map(Record::List),
            RecordId::Card(id) => self.cards.get(&id).cloned().map(Record::Card),
            RecordId::Label(id) => self.labels.get(&id).cloned().map(Record::Label),
            RecordId::Checklist(id) => self.checklists.get(&id).cloned().map(Record::Checklist),
            RecordId::CheckItem(id) => self.items.get(&id).cloned().map(Record::CheckItem),
            RecordId::Comment(id) => self.comments.get(&id).cloned().map(Record::Comment),
            RecordId::Attachment(id) => self.attachments.get(&id).cloned().map(Record::Attachment),
            RecordId::Field(id) => self.fields.get(&id).cloned().map(Record::Field),
            RecordId::Activity(id) => self.activities.get(&id).cloned().map(Record::Activity),
        }
    }
}

/// Aplica um lote (um gesto) e devolve o lote inverso, já na ordem de desfazer.
pub fn apply_batch(k: &mut Kanban, ops: Vec<KanbanOp>) -> Vec<KanbanOp> {
    let mut inverse: Vec<KanbanOp> = ops.into_iter().filter_map(|op| op.apply(k)).collect();
    inverse.reverse();
    inverse
}

#[cfg(test)]
#[path = "ops_tests.rs"]
mod tests;
