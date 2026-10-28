//! **Item de checklist → cartão** (v2, o «Converter em cartão» do Trello): o cartão novo nasce na
//! lista do cartão de origem, logo ABAIXO dele, com o texto do item; o item sai. O responsável e a
//! data do item passam a membro e entrega do cartão (o Trello leva os dois).

use std::collections::BTreeMap;

use crate::kanban::Kanban;
use crate::ops::{KanbanOp, Record, RecordId};
use crate::records::{Card, CardId, CheckItemId, Due, Meta, Moment};
use ph2d_frac::FracKey;

/// O lote que converte o item `item` num cartão (um gesto, um passo de desfazer). `None` = o item,
/// a checklist ou o cartão de origem não existem vivos.
#[must_use]
pub fn convert_item_to_card(
    k: &Kanban,
    item: CheckItemId,
    created: Moment,
) -> Option<Vec<KanbanOp>> {
    let it = k.items.get(&item).filter(|i| !i.meta.deleted)?;
    let cl = k
        .checklists
        .get(&it.checklist)
        .filter(|c| !c.meta.deleted)?;
    let origin = k.cards.get(&cl.card).filter(|c| !c.meta.deleted)?;
    let next = k
        .cards
        .values()
        .filter(|c| {
            c.list == origin.list && !c.meta.deleted && c.id != origin.id && c.pos > origin.pos
        })
        .map(|c| &c.pos)
        .min();
    let card = Card {
        id: CardId(k.mint()),
        list: origin.list,
        pos: FracKey::between(Some(&origin.pos), next),
        title: it.text.clone(),
        description: String::new(),
        labels: Vec::new(),
        members: it.member.iter().cloned().collect(),
        start: None,
        due: it.due.map(|at| Due {
            at,
            done: false,
            remind_before_min: None,
        }),
        cover: None,
        fields: BTreeMap::new(),
        location: String::new(),
        votes: Vec::new(),
        created,
        archived: false,
        meta: Meta::default(),
        watchers: Vec::new(),
        origin: Some(origin.id),
        template: false,
    };
    Some(vec![
        KanbanOp::put(Record::Card(card)),
        KanbanOp::Delete(RecordId::CheckItem(item)),
    ])
}
