//! **`KanbanCard`** — a pega de um cartão: os campos legíveis e as escritas (cada uma um lote na
//! pista, com o autor do script).

use mlua::{AnyUserData, Lua, MetaMethod, Table, UserData, UserDataFields, UserDataMethods, Value};
use ph2d_kanban_filter::normalize;
use ph2d_kanban_model::{
    Card, CardId, CardRef, CheckItem, Comment, CommentId, Due, FieldDef, FieldKind, FieldValue,
    FracKey, KanbanId, KanbanOp, LabelId, Member, Meta, Moment, Place, Record, move_card,
    remove_card,
};

use crate::dates;
use crate::handles::{kanban_ud, list_arg, list_ud};
use crate::lane::{Lane, color_name, err, with};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct CardH {
    pub kanban: KanbanId,
    pub card: CardId,
}

pub(crate) fn card_ud(lua: &Lua, kanban: KanbanId, card: CardId) -> mlua::Result<AnyUserData> {
    lua.create_userdata(CardH { kanban, card })
}

/// O que `list:add_card` pede.
pub(crate) struct CardSpec {
    pub title: String,
    pub top: bool,
    description: Option<String>,
    labels: Vec<LabelId>,
    members: Vec<Member>,
    due: Option<Moment>,
    start: Option<Moment>,
}

impl CardSpec {
    pub fn fill(self, c: &mut Card) {
        if let Some(d) = self.description {
            c.description = d;
        }
        c.labels = self.labels;
        c.members = self.members;
        c.due = self.due.map(|at| Due {
            at,
            done: false,
            remind_before_min: None,
        });
        c.start = self.start;
    }
}

fn strings(f: &str, what: &str, v: Value) -> mlua::Result<Vec<String>> {
    match v {
        Value::Nil => Ok(Vec::new()),
        Value::Table(t) => t
            .sequence_values::<String>()
            .collect::<mlua::Result<_>>()
            .map_err(|_| err(f, format!("{what} is a list of texts"))),
        other => Err(err(
            f,
            format!("{what} is a list of texts, got a {}", other.type_name()),
        )),
    }
}

fn labels(lane: &Lane, f: &str, k: KanbanId, names: &[String]) -> mlua::Result<Vec<LabelId>> {
    let mut out = Vec::new();
    for n in names {
        let id = lane.label_named(f, k, n)?.id;
        if !out.contains(&id) {
            out.push(id);
        }
    }
    Ok(out)
}

fn date(lane: &Lane, f: &str, what: &str, v: &Value) -> mlua::Result<Moment> {
    dates::from_lua(v, &lane.zone).map_err(|e| err(f, format!("{what}: {e}")))
}

fn opt_date(lane: &Lane, f: &str, what: &str, v: &Value) -> mlua::Result<Option<Moment>> {
    match v {
        Value::Nil => Ok(None),
        v => date(lane, f, what, v).map(Some),
    }
}

pub(crate) fn card_spec(lane: &Lane, f: &str, k: KanbanId, v: &Value) -> mlua::Result<CardSpec> {
    let t = match v {
        Value::String(s) => {
            let title = s.to_string_lossy().trim().to_owned();
            if title.is_empty() {
                return Err(err(f, "the card's title cannot be empty"));
            }
            return Ok(CardSpec {
                title,
                top: false,
                description: None,
                labels: Vec::new(),
                members: Vec::new(),
                due: None,
                start: None,
            });
        }
        Value::Table(t) => t,
        other => {
            return Err(err(
                f,
                format!("got a {} — a title or a table", other.type_name()),
            ));
        }
    };
    let title: String = t
        .get::<Option<String>>("title")?
        .unwrap_or_default()
        .trim()
        .to_owned();
    if title.is_empty() {
        return Err(err(f, "the card needs a title"));
    }
    Ok(CardSpec {
        title,
        top: t.get::<Option<bool>>("top")?.unwrap_or(false),
        description: t.get("description")?,
        labels: labels(lane, f, k, &strings(f, "labels", t.get("labels")?)?)?,
        members: strings(f, "members", t.get("members")?)?
            .into_iter()
            .map(Member)
            .collect(),
        due: opt_date(lane, f, "due", &t.get("due")?)?,
        start: opt_date(lane, f, "start", &t.get("start")?)?,
    })
}

fn field_value(lane: &Lane, lua: &Lua, def: &FieldDef, v: &FieldValue) -> mlua::Result<Value> {
    use mlua::IntoLua;
    match v {
        FieldValue::Text(s) => s.clone().into_lua(lua),
        FieldValue::Number(n) => n.into_lua(lua),
        FieldValue::Date(m) => dates::to_iso(*m, &lane.zone).into_lua(lua),
        FieldValue::Checkbox(b) => b.into_lua(lua),
        FieldValue::Option(o) => match &def.kind {
            FieldKind::Dropdown(opts) => opts
                .iter()
                .find(|x| x.id == *o)
                .map(|x| x.text.clone())
                .into_lua(lua),
            _ => Ok(Value::Nil),
        },
    }
}

/// O valor de um campo personalizado vindo do script; `None` = limpar (`false` fora da caixa).
fn to_field(lane: &Lane, f: &str, def: &FieldDef, v: &Value) -> mlua::Result<Option<FieldValue>> {
    let bad = |want: &str| {
        err(
            f,
            format!(
                "field \"{}\" takes {want}, got a {}",
                def.name,
                v.type_name()
            ),
        )
    };
    if matches!(v, Value::Boolean(false)) && def.kind != FieldKind::Checkbox {
        return Ok(None);
    }
    Ok(Some(match (&def.kind, v) {
        (FieldKind::Text, Value::String(s)) => FieldValue::Text(s.to_string_lossy()),
        (FieldKind::Number, Value::Integer(i)) =>
        {
            #[expect(clippy::cast_precision_loss, reason = "o número do campo é f64")]
            FieldValue::Number(*i as f64)
        }
        (FieldKind::Number, Value::Number(n)) if n.is_finite() => FieldValue::Number(*n),
        (FieldKind::Date, v) => FieldValue::Date(date(lane, f, &def.name, v)?),
        (FieldKind::Checkbox, Value::Boolean(b)) => FieldValue::Checkbox(*b),
        (FieldKind::Dropdown(opts), Value::String(s)) => {
            let want = normalize(&s.to_string_lossy());
            let o = opts
                .iter()
                .find(|o| normalize(&o.text) == want)
                .ok_or_else(|| {
                    let has: Vec<&str> = opts.iter().map(|o| o.text.as_str()).collect();
                    err(
                        f,
                        format!(
                            "field \"{}\" has no option \"{}\" ({})",
                            def.name,
                            s.to_string_lossy(),
                            has.join(", ")
                        ),
                    )
                })?;
            FieldValue::Option(o.id)
        }
        (FieldKind::Text, _) => return Err(bad("text")),
        (FieldKind::Number, _) => return Err(bad("a number")),
        (FieldKind::Checkbox, _) => return Err(bad("true or false")),
        (FieldKind::Dropdown(_), _) => return Err(bad("an option's text")),
    }))
}

impl CardH {
    fn edit(
        self,
        lua: &Lua,
        f: &str,
        change: impl FnOnce(&Lane, &mut Card) -> mlua::Result<()>,
    ) -> mlua::Result<()> {
        with(lua, f, |l| {
            let mut c = l.card(f, self.kanban, self.card)?.clone();
            let before = c.clone();
            change(l, &mut c)?;
            if c != before {
                l.write(self.kanban, vec![KanbanOp::put(Record::card(c))]);
            }
            Ok(())
        })
    }

    fn set(self, lua: &Lua, t: &Table) -> mlua::Result<()> {
        let f = "card:set";
        self.edit(lua, f, |l, c| {
            if let Some(title) = t.get::<Option<String>>("title")? {
                let title = title.trim();
                if title.is_empty() {
                    return Err(err(f, "the card's title cannot be empty"));
                }
                title.clone_into(&mut c.title);
            }
            if let Some(d) = t.get::<Option<String>>("description")? {
                c.description = d;
            }
            match t.get::<Value>("due")? {
                Value::Nil => {}
                Value::Boolean(false) => c.due = None,
                v => {
                    let at = date(l, f, "due", &v)?;
                    let keep = c.due.map(|d| (d.done, d.remind_before_min));
                    let (done, remind_before_min) = keep.unwrap_or((false, None));
                    c.due = Some(Due {
                        at,
                        done,
                        remind_before_min,
                    });
                }
            }
            match t.get::<Value>("start")? {
                Value::Nil => {}
                Value::Boolean(false) => c.start = None,
                v => c.start = Some(date(l, f, "start", &v)?),
            }
            if let Some(done) = t.get::<Option<bool>>("done")? {
                let d = c
                    .due
                    .as_mut()
                    .ok_or_else(|| err(f, "`done` needs a due date"))?;
                d.done = done;
            }
            let v = t.get::<Value>("labels")?;
            if !v.is_nil() {
                c.labels = labels(l, f, self.kanban, &strings(f, "labels", v)?)?;
            }
            let v = t.get::<Value>("members")?;
            if !v.is_nil() {
                c.members = strings(f, "members", v)?.into_iter().map(Member).collect();
            }
            if let Some(fields) = t.get::<Option<Table>>("fields")? {
                let kb = l.kanban(f, self.kanban)?;
                for pair in fields.pairs::<String, Value>() {
                    let (name, v) = pair?;
                    let want = normalize(&name);
                    let def = kb
                        .fields
                        .values()
                        .find(|d| !d.meta.deleted && normalize(&d.name) == want)
                        .ok_or_else(|| {
                            err(
                                f,
                                format!("no custom field \"{name}\" in \"{}\"", kb.header.name),
                            )
                        })?;
                    match to_field(l, f, def, &v)? {
                        Some(x) => {
                            c.fields.insert(def.id, x);
                        }
                        None => {
                            c.fields.remove(&def.id);
                        }
                    }
                }
            }
            Ok(())
        })
    }

    fn move_to(&mut self, lua: &Lua, list: &Value, position: Option<i64>) -> mlua::Result<()> {
        let f = "card:move";
        let me = *self;
        let moved = with(lua, f, |l| {
            l.card(f, me.kanban, me.card)?;
            let to = list_arg(l, f, me.kanban, list)?;
            let kb = l.kanban(f, to.kanban)?;
            let others: Vec<_> = l
                .set
                .entries_in(to.kanban, to.list)
                .into_iter()
                .filter(|e| {
                    !(to.kanban == me.kanban && *e == ph2d_kanban_model::Entry::Card(me.card))
                })
                .collect();
            let at = match position {
                None => others.len(),
                Some(p) if p >= 1 => usize::try_from(p - 1)
                    .unwrap_or(usize::MAX)
                    .min(others.len()),
                Some(p) => return Err(err(f, format!("position {p} — 1 is the top"))),
            };
            let before = at.checked_sub(1).and_then(|i| kb.entry_pos(others[i]));
            let after = others.get(at).and_then(|e| kb.entry_pos(*e));
            let place = Place {
                kanban: to.kanban,
                list: to.list,
                pos: FracKey::between(before, after),
            };
            let src = CardRef {
                kanban: me.kanban,
                card: me.card,
            };
            let plan =
                move_card(&l.set, src, &place).ok_or_else(|| err(f, "the card cannot go there"))?;
            let new_id = (to.kanban != me.kanban)
                .then(|| {
                    plan.ops(to.kanban).iter().find_map(|op| match op {
                        KanbanOp::Put(r) => match &**r {
                            Record::Card(c) => Some(c.id),
                            _ => None,
                        },
                        _ => None,
                    })
                })
                .flatten();
            let by = l.by.clone();
            l.write_plan(plan, &by);
            Ok(new_id.map(|c| CardH {
                kanban: to.kanban,
                card: c,
            }))
        })?;
        if let Some(h) = moved {
            *self = h;
        }
        Ok(())
    }

    fn check(self, lua: &Lua, item: &str, done: bool) -> mlua::Result<()> {
        let f = "card:check";
        with(lua, f, |l| {
            let kb = l.kanban(f, self.kanban)?;
            let want = normalize(item);
            let mut lists: Vec<_> = kb
                .checklists
                .values()
                .filter(|c| c.card == self.card && !c.meta.deleted)
                .collect();
            lists.sort_by(|a, b| a.pos.cmp(&b.pos).then(a.id.cmp(&b.id)));
            let found = lists.iter().find_map(|cl| {
                let mut items: Vec<&CheckItem> = kb
                    .items
                    .values()
                    .filter(|i| i.checklist == cl.id && !i.meta.deleted)
                    .collect();
                items.sort_by(|a, b| a.pos.cmp(&b.pos).then(a.id.cmp(&b.id)));
                items
                    .into_iter()
                    .find(|i| normalize(&i.text) == want)
                    .cloned()
            });
            let it = found
                .ok_or_else(|| err(f, format!("the card has no checklist item \"{item}\"")))?;
            if it.done != done {
                l.write(
                    self.kanban,
                    vec![KanbanOp::put(Record::CheckItem(CheckItem { done, ..it }))],
                );
            }
            Ok(())
        })
    }
}

impl UserData for CardH {
    #[expect(clippy::too_many_lines, reason = "um campo por linha da tabela da API")]
    fn add_fields<F: UserDataFields<Self>>(fields: &mut F) {
        fn read<R>(
            lua: &Lua,
            h: CardH,
            f: &str,
            get: impl FnOnce(&Lane, &Card) -> mlua::Result<R>,
        ) -> mlua::Result<R> {
            with(lua, f, |l| {
                let c = l.card(f, h.kanban, h.card)?;
                get(l, c)
            })
        }
        fields.add_field_method_get("id", |_, h| Ok(h.card.0));
        fields.add_field_method_get("title", |lua, h| {
            read(lua, *h, "card.title", |_, c| Ok(c.title.clone()))
        });
        fields.add_field_method_get("description", |lua, h| {
            read(lua, *h, "card.description", |_, c| {
                Ok(c.description.clone())
            })
        });
        fields.add_field_method_get("labels", |lua, h| {
            read(lua, *h, "card.labels", |l, c| {
                let kb = l.kanban("card.labels", h.kanban)?;
                Ok(c.labels
                    .iter()
                    .filter_map(|id| kb.labels.get(id).filter(|x| !x.meta.deleted))
                    .map(|x| match x.color {
                        Some(col) if x.name.is_empty() => color_name(col),
                        _ => x.name.clone(),
                    })
                    .collect::<Vec<_>>())
            })
        });
        fields.add_field_method_get("members", |lua, h| {
            read(lua, *h, "card.members", |_, c| {
                Ok(c.members.iter().map(|m| m.0.clone()).collect::<Vec<_>>())
            })
        });
        fields.add_field_method_get("due", |lua, h| {
            read(lua, *h, "card.due", |l, c| {
                Ok(c.due.map(|d| dates::to_iso(d.at, &l.zone)))
            })
        });
        fields.add_field_method_get("start", |lua, h| {
            read(lua, *h, "card.start", |l, c| {
                Ok(c.start.map(|m| dates::to_iso(m, &l.zone)))
            })
        });
        fields.add_field_method_get("done", |lua, h| {
            read(lua, *h, "card.done", |_, c| {
                Ok(c.due.is_some_and(|d| d.done))
            })
        });
        fields.add_field_method_get("archived", |lua, h| {
            read(lua, *h, "card.archived", |_, c| Ok(c.archived))
        });
        fields.add_field_method_get("list", |lua, h| {
            let list = read(lua, *h, "card.list", |_, c| Ok(c.list))?;
            list_ud(lua, h.kanban, list)
        });
        fields.add_field_method_get("kanban", |lua, h| kanban_ud(lua, h.kanban));
        fields.add_field_method_get("url", |_, h| {
            Ok(format!("ph2d://kanban/{}/card/{}", h.kanban.0, h.card.0))
        });
        fields.add_field_method_get("fields", |lua, h| {
            read(lua, *h, "card.fields", |l, c| {
                let kb = l.kanban("card.fields", h.kanban)?;
                let t = lua.create_table()?;
                for (id, v) in &c.fields {
                    if let Some(def) = kb.fields.get(id).filter(|d| !d.meta.deleted) {
                        t.set(def.name.clone(), field_value(l, lua, def, v)?)?;
                    }
                }
                Ok(t)
            })
        });
        fields.add_field_method_get("checklists", |lua, h| {
            read(lua, *h, "card.checklists", |l, c| {
                let kb = l.kanban("card.checklists", h.kanban)?;
                let mut lists: Vec<_> = kb
                    .checklists
                    .values()
                    .filter(|x| x.card == c.id && !x.meta.deleted)
                    .collect();
                lists.sort_by(|a, b| a.pos.cmp(&b.pos).then(a.id.cmp(&b.id)));
                let out = lua.create_table()?;
                for cl in lists {
                    let mut items: Vec<&CheckItem> = kb
                        .items
                        .values()
                        .filter(|i| i.checklist == cl.id && !i.meta.deleted)
                        .collect();
                    items.sort_by(|a, b| a.pos.cmp(&b.pos).then(a.id.cmp(&b.id)));
                    let it = lua.create_table()?;
                    for i in items {
                        let x = lua.create_table()?;
                        x.set("text", i.text.clone())?;
                        x.set("done", i.done)?;
                        it.push(x)?;
                    }
                    let t = lua.create_table()?;
                    t.set("title", cl.title.clone())?;
                    t.set("items", it)?;
                    out.push(t)?;
                }
                Ok(out)
            })
        });
    }

    fn add_methods<M: UserDataMethods<Self>>(m: &mut M) {
        m.add_meta_method(MetaMethod::Eq, |_, a, b: AnyUserData| {
            Ok(b.borrow::<CardH>().is_ok_and(|b| *b == *a))
        });
        m.add_meta_method(MetaMethod::ToString, |lua, h, ()| {
            with(lua, "card", |l| {
                Ok(format!(
                    "Card \"{}\"",
                    l.card("card", h.kanban, h.card)?.title
                ))
            })
        });
        m.add_method_mut("move", |lua, h, (list, position): (Value, Option<i64>)| {
            h.move_to(lua, &list, position)
        });
        m.add_method("set", |lua, h, t: Table| h.set(lua, &t));
        m.add_method("add_label", |lua, h, name: String| {
            let f = "card:add_label";
            h.edit(lua, f, |l, c| {
                let id = l.label_named(f, h.kanban, &name)?.id;
                if !c.labels.contains(&id) {
                    c.labels.push(id);
                }
                Ok(())
            })
        });
        m.add_method("remove_label", |lua, h, name: String| {
            let f = "card:remove_label";
            h.edit(lua, f, |l, c| {
                let id = l.label_named(f, h.kanban, &name)?.id;
                c.labels.retain(|x| *x != id);
                Ok(())
            })
        });
        m.add_method("add_member", |lua, h, name: String| {
            let f = "card:add_member";
            h.edit(lua, f, |_, c| {
                let name = name.trim();
                if name.is_empty() {
                    return Err(err(f, "the member's name cannot be empty"));
                }
                let m = Member(name.to_owned());
                if !c.members.contains(&m) {
                    c.members.push(m);
                }
                Ok(())
            })
        });
        m.add_method("comment", |lua, h, text: String| {
            let f = "card:comment";
            with(lua, f, |l| {
                if text.trim().is_empty() {
                    return Err(err(f, "the comment cannot be empty"));
                }
                l.card(f, h.kanban, h.card)?;
                let kb = l.kanban(f, h.kanban)?;
                let c = Comment {
                    id: CommentId(kb.mint()),
                    card: h.card,
                    author: l.by.clone(),
                    at: l.now,
                    text,
                    edited: None,
                    reactions: Vec::new(),
                    meta: Meta::default(),
                };
                l.write(h.kanban, vec![KanbanOp::put(Record::Comment(c))]);
                Ok(())
            })
        });
        m.add_method("check", |lua, h, (item, done): (String, Option<bool>)| {
            h.check(lua, &item, done.unwrap_or(true))
        });
        m.add_method("archive", |lua, h, ()| {
            let f = "card:archive";
            with(lua, f, |l| {
                l.card(f, h.kanban, h.card)?;
                if let Some(p) = remove_card(
                    &l.set,
                    CardRef {
                        kanban: h.kanban,
                        card: h.card,
                    },
                    true,
                ) {
                    let by = l.by.clone();
                    l.write_plan(p, &by);
                }
                Ok(())
            })
        });
    }
}
