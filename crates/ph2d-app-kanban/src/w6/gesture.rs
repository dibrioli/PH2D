//! ⭐ **Os gestos da W6 que escrevem** — pelas portas puras do modelo: um gesto entre Kanbans é um
//! `Plan` aplicado com UMA hora em cada histórico tocado; o desfazer de um Kanban leva junto os
//! passos dos outros com a MESMA hora (o gesto inteiro), e nunca um passo alheio.

use crate::docs::KanbanDocs;
use ph2d_editor_core::documents::undo_tick;
use ph2d_kanban_model::{CardId, CardRef, Entry, FracKey, KanbanOp, ListId, Plan, RecordId};

/// Aplica o gesto em nome de «eu», pela porta única ([`crate::hooks::write`]). `true` = escreveu.
pub(crate) fn apply(docs: &mut KanbanDocs, plan: Option<Plan>) -> bool {
    crate::hooks::write(docs, plan)
}

/// ⭐ O desfazer (ou refazer) do Kanban da vista `id`: o passo mais recente DELE, e com ele os
/// passos de outros Kanbans que nasceram no mesmo gesto (a mesma hora). `true` = desfez.
pub(crate) fn undo(docs: &mut KanbanDocs, id: u64, redo: bool) -> bool {
    let Some(kid) = docs.kanban_of(id) else {
        return false;
    };
    if let Some(t) = docs.tab_mut(id) {
        t.field = None;
        t.w6.quick = None;
    }
    let when = |h: &ph2d_kanban_model::History| if redo { h.redo_at() } else { h.undo_at() };
    let Some(at) = docs.histories.get(&kid).and_then(when) else {
        return false;
    };
    let tick = undo_tick();
    let mut set = docs.set.borrow_mut();
    let mut did = false;
    for (k, h) in &mut docs.histories {
        if when(h) != Some(at) {
            continue;
        }
        if let Some(doc) = set.doc_mut(*k) {
            did |= if redo { h.redo(doc, tick) } else { h.undo(doc, tick) };
        }
    }
    did
}

/// A linha (cartão ou espelho) `row` da vista `id`, como a disposição a conhece.
fn entry(docs: &KanbanDocs, id: u64, row: CardId) -> Option<Entry> {
    let t = docs.tabs.iter().find(|t| t.id() == id)?;
    Some(match t.rows.mirrors.get(&row) {
        Some((m, of)) => Entry::Mirror { id: *m, of: *of },
        None => Entry::Card(row),
    })
}

/// ⭐ Leva a linha `row` à lista `list`, no índice `index` entre as linhas MOSTRADAS (sem ela) —
/// com o filtro, entre as que se vêem; um espelho anda como um cartão. Um passo.
pub(crate) fn move_row(docs: &mut KanbanDocs, id: u64, row: CardId, list: ListId, index: usize) -> bool {
    let Some(e) = entry(docs, id, row) else {
        return false;
    };
    let Some(t) = docs.tabs.iter().find(|t| t.id() == id) else {
        return false;
    };
    let shown: Vec<CardId> = t
        .rows
        .lists
        .get(&list)
        .map(|v| v.iter().copied().filter(|c| *c != row).collect())
        .unwrap_or_default();
    let mirrors = t.rows.mirrors.clone();
    let kid = t.view.kanban;
    let op = {
        let set = docs.set.borrow();
        let Some(k) = set.get(kid) else {
            return false;
        };
        let pos_of = |c: CardId| -> Option<FracKey> {
            match mirrors.get(&c) {
                Some((m, _)) => k.mirrors.get(m).map(|m| m.pos.clone()),
                None => k.cards.get(&c).map(|c| c.pos.clone()),
            }
        };
        let i = index.min(shown.len());
        let lo = i.checked_sub(1).and_then(|j| pos_of(shown[j]));
        let hi = shown.get(i).and_then(|c| pos_of(*c));
        let (cur_list, cur) = match e {
            Entry::Card(c) => k.cards.get(&c).map(|c| (c.list, c.pos.clone())),
            Entry::Mirror { id: m, .. } => k.mirrors.get(&m).map(|m| (m.list, m.pos.clone())),
        }
        .unzip();
        let stays = cur_list == Some(list)
            && cur.as_ref().is_some_and(|p| {
                lo.as_ref().is_none_or(|lo| lo < p) && hi.as_ref().is_none_or(|hi| p < hi)
            });
        if stays {
            return false;
        }
        k.entry_moved(e, list, FracKey::between(lo.as_ref(), hi.as_ref()))
    };
    let Some(op) = op else {
        return false;
    };
    docs.with_doc(id, |_, doc| doc.apply(vec![op])).unwrap_or(false)
}

/// ⭐ Arquivar a linha `row` (o `C`, a edição rápida, o verso): um cartão arquiva-se E leva os
/// espelhos dele; um espelho só sai da lista.
pub(crate) fn archive_row(docs: &mut KanbanDocs, id: u64, row: CardId) -> bool {
    let Some(kid) = docs.kanban_of(id) else {
        return false;
    };
    match entry(docs, id, row) {
        Some(Entry::Mirror { id: m, .. }) => {
            let mut plan = Plan::default();
            plan.push(kid, KanbanOp::Delete(RecordId::Mirror(m)));
            apply(docs, Some(plan))
        }
        Some(Entry::Card(c)) => {
            let plan = ph2d_kanban_model::remove_card(
                &docs.set.borrow(),
                CardRef { kanban: kid, card: c },
                true,
            );
            apply(docs, plan)
        }
        None => false,
    }
}

/// O cartão que a linha `row` da vista `id` mostra (o original de um espelho).
pub(crate) fn shows(docs: &KanbanDocs, id: u64, row: CardId) -> Option<CardRef> {
    let t = docs.tabs.iter().find(|t| t.id() == id)?;
    Some(t.rows.shows(t.view.kanban, row))
}

/// Abre o verso do que a linha `row` mostra: o original de um espelho abre na aba do Kanban dele.
pub(crate) fn open_back(docs: &mut KanbanDocs, id: u64, row: CardId) {
    let Some(r) = shows(docs, id, row) else {
        return;
    };
    if docs.kanban_of(id) == Some(r.kanban) {
        crate::back::open(docs, id, r.card);
    } else {
        docs.open(r.kanban);
        crate::back::open(docs, r.kanban.0, r.card);
    }
}
