//! **`Kanban` e `KanbanList`** — pegas por id (userdata): cada campo lê a pista NA HORA, por isso o
//! script vê as próprias escritas. O `KanbanCard` mora em [`crate::card`].

use std::collections::BTreeSet;

use mlua::{AnyUserData, Lua, MetaMethod, UserData, UserDataFields, UserDataMethods, Value};
use ph2d_kanban_filter::{SortBy, search, sort_list};
use ph2d_kanban_model::{Card, CardId, FracKey, KanbanId, KanbanOp, List, ListId, Meta, Record};

use crate::card::{card_spec, card_ud};
use crate::lane::{Lane, color_name, err, with};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct KanbanH(pub KanbanId);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct ListH {
    pub kanban: KanbanId,
    pub list: ListId,
}

pub(crate) fn kanban_ud(lua: &Lua, k: KanbanId) -> mlua::Result<AnyUserData> {
    lua.create_userdata(KanbanH(k))
}

pub(crate) fn list_ud(lua: &Lua, kanban: KanbanId, list: ListId) -> mlua::Result<AnyUserData> {
    lua.create_userdata(ListH { kanban, list })
}

/// Um número do Luau que é id (inteiro ≥ 0).
pub(crate) fn id_of(v: &Value) -> Option<u64> {
    match v {
        Value::Integer(i) => u64::try_from(*i).ok(),
        #[expect(
            clippy::cast_possible_truncation,
            clippy::cast_sign_loss,
            reason = "inteiro ≥ 0"
        )]
        Value::Number(n) if n.fract() == 0.0 && *n >= 0.0 => Some(*n as u64),
        _ => None,
    }
}

/// Uma lista dada como pega ou como título (de uma lista do Kanban `home`).
pub(crate) fn list_arg(lane: &Lane, f: &str, home: KanbanId, v: &Value) -> mlua::Result<ListH> {
    match v {
        Value::UserData(u) => {
            let l = *u
                .borrow::<ListH>()
                .map_err(|_| err(f, "expected a list or a list's title"))?;
            lane.list(f, l.kanban, l.list)?;
            Ok(l)
        }
        Value::String(s) => {
            let name = s.to_string_lossy();
            let k = lane.kanban(f, home)?;
            let list = lane
                .list_named(f, home, &name)?
                .ok_or_else(|| err(f, format!("no list \"{name}\" in \"{}\"", k.header.name)))?;
            Ok(ListH { kanban: home, list })
        }
        other => Err(err(
            f,
            format!("got a {} — a list or a list's title", other.type_name()),
        )),
    }
}

/// Os cartões vivos de `k` pela ordem do quadro; `open` = só os à vista (lista e cartão abertos).
pub(crate) fn board_cards(
    lane: &Lane,
    f: &str,
    k: KanbanId,
    open: bool,
) -> mlua::Result<Vec<CardId>> {
    let kb = lane.kanban(f, k)?;
    let mut lists: Vec<&List> = kb.lists.values().filter(|l| !l.meta.deleted).collect();
    lists.sort_by(|a, b| a.pos.cmp(&b.pos).then(a.id.cmp(&b.id)));
    let mut out = Vec::new();
    for l in lists.into_iter().filter(|l| !(open && l.archived)) {
        let mut cs: Vec<&Card> = kb
            .cards
            .values()
            .filter(|c| c.list == l.id && !c.meta.deleted && !c.template && !(open && c.archived))
            .collect();
        cs.sort_by(|a, b| a.pos.cmp(&b.pos).then(a.id.cmp(&b.id)));
        out.extend(cs.into_iter().map(|c| c.id));
    }
    Ok(out)
}

/// Os cartões do projecto que a pesquisa acha (a lei do `ph2d-kanban-filter::search`).
pub(crate) fn found(lane: &Lane, query: &str) -> Vec<(KanbanId, CardId)> {
    search(&lane.set, query, lane.now, &lane.zone, &lane.me)
        .into_iter()
        .filter(|h| h.card.kanban != KanbanId::AGENDA)
        .map(|h| (h.card.kanban, h.card.card))
        .collect()
}

fn sort_by(f: &str, s: &str) -> mlua::Result<SortBy> {
    Ok(match s {
        "newest" => SortBy::Newest,
        "oldest" => SortBy::Oldest,
        "name" => SortBy::Name,
        "due" => SortBy::Due,
        _ => {
            return Err(err(
                f,
                format!("\"{s}\" — sort by \"newest\", \"oldest\", \"name\" or \"due\""),
            ));
        }
    })
}

fn not_empty(f: &str, what: &str, s: &str) -> mlua::Result<String> {
    let t = s.trim();
    if t.is_empty() {
        return Err(err(f, format!("the {what} cannot be empty")));
    }
    Ok(t.to_owned())
}

impl UserData for KanbanH {
    fn add_fields<F: UserDataFields<Self>>(fields: &mut F) {
        fields.add_field_method_get("id", |_, this| Ok(this.0.0));
        fields.add_field_method_get("name", |lua, this| {
            with(lua, "kanban.name", |l| {
                Ok(l.kanban("kanban.name", this.0)?.header.name.clone())
            })
        });
    }

    fn add_methods<M: UserDataMethods<Self>>(m: &mut M) {
        m.add_meta_method(MetaMethod::Eq, |_, a, b: AnyUserData| {
            Ok(b.borrow::<KanbanH>().is_ok_and(|b| *b == *a))
        });
        m.add_meta_method(MetaMethod::ToString, |lua, this, ()| {
            with(lua, "kanban", |l| {
                Ok(format!(
                    "Kanban \"{}\"",
                    l.kanban("kanban", this.0)?.header.name
                ))
            })
        });
        m.add_method("lists", |lua, this, ()| {
            let ids: Vec<ListId> = with(lua, "kanban:lists", |l| {
                Ok(l.kanban("kanban:lists", this.0)?
                    .lists_in_order()
                    .iter()
                    .map(|x| x.id)
                    .collect())
            })?;
            ids.into_iter()
                .map(|id| list_ud(lua, this.0, id))
                .collect::<mlua::Result<Vec<_>>>()
        });
        m.add_method("list", |lua, this, name: String| {
            let id = with(lua, "kanban:list", |l| {
                l.list_named("kanban:list", this.0, &name)
            })?;
            id.map(|id| list_ud(lua, this.0, id)).transpose()
        });
        m.add_method("cards", |lua, this, filter: Option<String>| {
            let f = "kanban:cards";
            let ids = with(lua, f, |l| match filter.as_deref().map(str::trim) {
                None | Some("") => board_cards(l, f, this.0, true),
                Some(q) => {
                    let hit: BTreeSet<CardId> = found(l, q)
                        .into_iter()
                        .filter(|(k, _)| *k == this.0)
                        .map(|(_, c)| c)
                        .collect();
                    Ok(board_cards(l, f, this.0, false)?
                        .into_iter()
                        .filter(|c| hit.contains(c))
                        .collect())
                }
            })?;
            ids.into_iter()
                .map(|c| card_ud(lua, this.0, c))
                .collect::<mlua::Result<Vec<_>>>()
        });
        m.add_method("labels", |lua, this, ()| {
            with(lua, "kanban:labels", |l| {
                let kb = l.kanban("kanban:labels", this.0)?;
                let mut out = Vec::new();
                for lb in kb.labels.values().filter(|x| !x.meta.deleted) {
                    let t = lua.create_table()?;
                    t.set("id", lb.id.0)?;
                    t.set("name", lb.name.clone())?;
                    t.set("color", lb.color.map(color_name))?;
                    out.push(t);
                }
                Ok(out)
            })
        });
        m.add_method("members", |lua, this, ()| {
            with(lua, "kanban:members", |l| {
                let kb = l.kanban("kanban:members", this.0)?;
                let live = kb.cards.values().filter(|c| !c.meta.deleted);
                let names: BTreeSet<String> = live
                    .flat_map(|c| c.members.iter())
                    .chain(kb.header.watchers.iter())
                    .filter(|m| !m.is_butler())
                    .map(|m| m.0.clone())
                    .collect();
                Ok(names.into_iter().collect::<Vec<_>>())
            })
        });
        m.add_method("add_list", |lua, this, name: String| {
            let f = "kanban:add_list";
            let id = with(lua, f, |l| {
                let title = not_empty(f, "list's name", &name)?;
                let kb = l.kanban(f, this.0)?;
                let list = List {
                    id: ListId(kb.mint()),
                    title,
                    pos: kb.pos_after_lists(),
                    color: None,
                    card_limit: None,
                    archived: false,
                    meta: Meta::default(),
                    watchers: Vec::new(),
                };
                let id = list.id;
                l.write(this.0, vec![KanbanOp::put(Record::List(list))]);
                Ok(id)
            })?;
            list_ud(lua, this.0, id)
        });
        m.add_method("sort", |lua, this, (list, by): (Value, String)| {
            let f = "kanban:sort";
            with(lua, f, |l| {
                let by = sort_by(f, &by)?;
                let target = list_arg(l, f, this.0, &list)?;
                if target.kanban != this.0 {
                    return Err(err(f, "that list is in another Kanban"));
                }
                let ops = sort_list(&l.set, this.0, target.list, by);
                l.write(this.0, ops);
                Ok(())
            })
        });
    }
}

impl UserData for ListH {
    fn add_fields<F: UserDataFields<Self>>(fields: &mut F) {
        fields.add_field_method_get("id", |_, this| Ok(this.list.0));
        fields.add_field_method_get("name", |lua, this| {
            with(lua, "list.name", |l| {
                Ok(l.list("list.name", this.kanban, this.list)?.title.clone())
            })
        });
        fields.add_field_method_get("archived", |lua, this| {
            with(lua, "list.archived", |l| {
                Ok(l.list("list.archived", this.kanban, this.list)?.archived)
            })
        });
        fields.add_field_method_get("kanban", |lua, this| kanban_ud(lua, this.kanban));
    }

    fn add_methods<M: UserDataMethods<Self>>(m: &mut M) {
        m.add_meta_method(MetaMethod::Eq, |_, a, b: AnyUserData| {
            Ok(b.borrow::<ListH>().is_ok_and(|b| *b == *a))
        });
        m.add_meta_method(MetaMethod::ToString, |lua, this, ()| {
            with(lua, "list", |l| {
                Ok(format!(
                    "List \"{}\"",
                    l.list("list", this.kanban, this.list)?.title
                ))
            })
        });
        m.add_method("cards", |lua, this, ()| {
            let ids: Vec<CardId> = with(lua, "list:cards", |l| {
                l.list("list:cards", this.kanban, this.list)?;
                let kb = l.kanban("list:cards", this.kanban)?;
                Ok(kb
                    .cards_in(this.list)
                    .iter()
                    .filter(|c| !c.template)
                    .map(|c| c.id)
                    .collect())
            })?;
            ids.into_iter()
                .map(|c| card_ud(lua, this.kanban, c))
                .collect::<mlua::Result<Vec<_>>>()
        });
        m.add_method("add_card", |lua, this, spec: Value| {
            let f = "list:add_card";
            let id = with(lua, f, |l| {
                l.list(f, this.kanban, this.list)?;
                let spec = card_spec(l, f, this.kanban, &spec)?;
                let kb = l.kanban(f, this.kanban)?;
                let pos = if spec.top {
                    let first = l.set.entries_in(this.kanban, this.list).first().copied();
                    FracKey::between(None, first.and_then(|e| kb.entry_pos(e)))
                } else {
                    kb.pos_after_cards(this.list)
                };
                let mut c = Card::new(CardId(kb.mint()), this.list, pos, &spec.title, l.now);
                spec.fill(&mut c);
                let id = c.id;
                l.write(this.kanban, vec![KanbanOp::put(Record::card(c))]);
                Ok(id)
            })?;
            card_ud(lua, this.kanban, id)
        });
    }
}
