//! **Os acontecimentos do script** — os MESMOS do Butler ([`ph2d_kanban_auto::What`]), pelo nome
//! que `ph2d.kanban.on` escreve, e a tabela que o handler recebe.

use mlua::{Lua, Table};
use ph2d_kanban_auto::{Event, What};
use ph2d_kanban_dates::Zone;
use ph2d_kanban_model::KanbanSet;

use crate::card::card_ud;
use crate::handles::{kanban_ud, list_ud};
use crate::lane::color_name;

/// Pela ordem do tipo `KanbanEventName` do `ph2d.d.luau` (o teste confere-os).
pub const EVENT_NAMES: &[&str] = &[
    "card_created",
    "card_moved",
    "card_left",
    "label_added",
    "label_removed",
    "member_added",
    "member_removed",
    "due_set",
    "due_removed",
    "due_completed",
    "item_checked",
    "checklist_completed",
    "comment_added",
    "card_archived",
    "field_changed",
];

#[must_use]
pub fn event_name(w: &What) -> &'static str {
    match w {
        What::Created { .. } => "card_created",
        What::MovedIn { .. } => "card_moved",
        What::MovedOut { .. } => "card_left",
        What::LabelAdded(_) => "label_added",
        What::LabelRemoved(_) => "label_removed",
        What::MemberAdded(_) => "member_added",
        What::MemberRemoved(_) => "member_removed",
        What::DueSet(_) => "due_set",
        What::DueRemoved => "due_removed",
        What::DueDone(_) => "due_completed",
        What::ItemChecked { .. } => "item_checked",
        What::ChecklistComplete(_) => "checklist_completed",
        What::Commented(_) => "comment_added",
        What::Archived(_) => "card_archived",
        What::FieldChanged(_) => "field_changed",
    }
}

/// A tabela `KanbanEvent` (lida do projecto DEPOIS do gesto).
pub(crate) fn to_lua(lua: &Lua, e: &Event, set: &KanbanSet, zone: &Zone) -> mlua::Result<Table> {
    let t = lua.create_table()?;
    t.set("name", event_name(&e.what))?;
    t.set("kanban", kanban_ud(lua, e.kanban)?)?;
    t.set("card", card_ud(lua, e.kanban, e.card)?)?;
    t.set("by", e.by.0.clone())?;
    let kb = set.get(e.kanban);
    match &e.what {
        What::Created { list } | What::MovedOut { list } => {
            t.set("list", list_ud(lua, e.kanban, *list)?)?
        }
        What::MovedIn { list, from } => {
            t.set("list", list_ud(lua, e.kanban, *list)?)?;
            t.set("from", list_ud(lua, e.kanban, *from)?)?;
        }
        What::LabelAdded(l) | What::LabelRemoved(l) => {
            let name = kb.and_then(|k| k.labels.get(l)).map(|x| match x.color {
                Some(c) if x.name.is_empty() => color_name(c),
                _ => x.name.clone(),
            });
            t.set("label", name)?;
        }
        What::MemberAdded(m) | What::MemberRemoved(m) => t.set("member", m.0.clone())?,
        What::DueSet(m) => t.set("due", crate::dates::to_iso(*m, zone))?,
        What::DueRemoved => {}
        What::DueDone(d) => t.set("done", *d)?,
        What::ItemChecked { item, done } => {
            t.set(
                "item",
                kb.and_then(|k| k.items.get(item)).map(|i| i.text.clone()),
            )?;
            t.set("done", *done)?;
        }
        What::ChecklistComplete(c) => {
            t.set(
                "checklist",
                kb.and_then(|k| k.checklists.get(c))
                    .map(|x| x.title.clone()),
            )?;
        }
        What::Commented(c) => t.set(
            "comment",
            kb.and_then(|k| k.comments.get(c)).map(|x| x.text.clone()),
        )?,
        What::Archived(a) => t.set("archived", *a)?,
        What::FieldChanged(fd) => t.set(
            "field",
            kb.and_then(|k| k.fields.get(fd)).map(|x| x.name.clone()),
        )?,
    }
    Ok(t)
}
