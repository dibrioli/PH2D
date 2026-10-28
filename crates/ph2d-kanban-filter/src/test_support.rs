//! Cenas dos testes, montadas SÓ por ops (a porta real).

use ph2d_kanban_dates::Zone;
use ph2d_kanban_model::{
    Card, CardId, Hue, KanbanId, KanbanOp, KanbanSet, Label, LabelColor, LabelId, List, ListId,
    Member, Meta, Moment, Record, Shade,
};

pub(crate) const DAY: i64 = 24 * 60 * 60 * 1000;
/// «Agora»: 2026-10-28 12:00 UTC.
pub(crate) const NOW: Moment = Moment(1_793_188_800_000);

pub(crate) fn utc() -> Zone {
    Zone::utc()
}

pub(crate) fn me() -> Member {
    Member("Enio".to_owned())
}

pub(crate) fn add_list(set: &mut KanbanSet, k: KanbanId, title: &str) -> ListId {
    let kd = set.get(k).unwrap();
    let id = ListId(kd.mint());
    let l = List {
        id,
        title: title.to_owned(),
        pos: kd.pos_after_lists(),
        color: None,
        card_limit: None,
        archived: false,
        meta: Meta::default(),
        watchers: Vec::new(),
    };
    set.apply(k, vec![KanbanOp::put(Record::List(l))]);
    id
}

pub(crate) fn add_label(set: &mut KanbanSet, k: KanbanId, name: &str, hue: Hue) -> LabelId {
    let id = LabelId(set.get(k).unwrap().mint());
    let l = Label {
        id,
        name: name.to_owned(),
        color: Some(LabelColor {
            hue,
            shade: Shade::Normal,
        }),
        meta: Meta::default(),
    };
    set.apply(k, vec![KanbanOp::put(Record::Label(l))]);
    id
}

/// Um cartão `title` criado em `created`, mexido por `edit` antes de entrar.
pub(crate) fn add_card(
    set: &mut KanbanSet,
    k: KanbanId,
    list: ListId,
    title: &str,
    created: Moment,
    edit: impl FnOnce(&mut Card),
) -> CardId {
    let kd = set.get(k).unwrap();
    let id = CardId(kd.mint());
    let mut c = Card::new(id, list, kd.pos_after_cards(list), title, created);
    edit(&mut c);
    set.apply(k, vec![KanbanOp::put(Record::Card(c))]);
    id
}
