//! O que só se faz tabela a tabela. ⛔ Os `match` são EXAUSTIVOS de propósito: uma tabela nova no
//! modelo não compila aqui até a réplica a cobrir (sem isto, os registos dela não viajariam).

use std::collections::BTreeSet;

use ph2d_kanban_model::{Background, CoverFill, FileId, Kanban, Link, Meta, RecordId};

/// Todos os registos de `k`, apagados incluídos.
pub(crate) fn all_ids(k: &Kanban) -> Vec<RecordId> {
    let mut v = Vec::new();
    v.extend(k.lists.keys().map(|i| RecordId::List(*i)));
    v.extend(k.cards.keys().map(|i| RecordId::Card(*i)));
    v.extend(k.labels.keys().map(|i| RecordId::Label(*i)));
    v.extend(k.checklists.keys().map(|i| RecordId::Checklist(*i)));
    v.extend(k.items.keys().map(|i| RecordId::CheckItem(*i)));
    v.extend(k.comments.keys().map(|i| RecordId::Comment(*i)));
    v.extend(k.attachments.keys().map(|i| RecordId::Attachment(*i)));
    v.extend(k.fields.keys().map(|i| RecordId::Field(*i)));
    v.extend(k.activities.keys().map(|i| RecordId::Activity(*i)));
    v.extend(k.mirrors.keys().map(|i| RecordId::Mirror(*i)));
    v.extend(k.automations.keys().map(|i| RecordId::Automation(*i)));
    v
}

/// O `meta` do registo `id` (`None` = não existe).
pub(crate) fn meta_mut(k: &mut Kanban, id: RecordId) -> Option<&mut Meta> {
    match id {
        RecordId::List(i) => k.lists.get_mut(&i).map(|r| &mut r.meta),
        RecordId::Card(i) => k.cards.get_mut(&i).map(|r| &mut r.meta),
        RecordId::Label(i) => k.labels.get_mut(&i).map(|r| &mut r.meta),
        RecordId::Checklist(i) => k.checklists.get_mut(&i).map(|r| &mut r.meta),
        RecordId::CheckItem(i) => k.items.get_mut(&i).map(|r| &mut r.meta),
        RecordId::Comment(i) => k.comments.get_mut(&i).map(|r| &mut r.meta),
        RecordId::Attachment(i) => k.attachments.get_mut(&i).map(|r| &mut r.meta),
        RecordId::Field(i) => k.fields.get_mut(&i).map(|r| &mut r.meta),
        RecordId::Activity(i) => k.activities.get_mut(&i).map(|r| &mut r.meta),
        RecordId::Mirror(i) => k.mirrors.get_mut(&i).map(|r| &mut r.meta),
        RecordId::Automation(i) => k.automations.get_mut(&i).map(|r| &mut r.meta),
    }
}

/// Os ficheiros que os registos VIVOS de UM Kanban usam — a regra de
/// `KanbanSet::files_in_use`, por Kanban (o gate `files_of_one_kanban_match_the_sets_rule` prende-as).
pub(crate) fn files_of(k: &Kanban) -> BTreeSet<FileId> {
    let mut used = BTreeSet::new();
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
    used
}
