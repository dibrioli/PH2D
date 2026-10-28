//! **[`Kanban`] → JSON no formato de exportação do Trello** — o inverso de [`crate::import_trello`]:
//! importar o que esta função escreve dá o MESMO Kanban (gate `round_trip_*`).
//!
//! As tabelas saem pela ordem dos ids (é a ordem em que a importação os volta a dar) e o `pos` é a
//! posição na ordem `FracKey` × 16384 (o passo do Trello). Apagados não saem; arquivados saem com
//! `closed`. O Trello não tem onde guardar: reacções, quem segue, «esconder marcados», a origem de
//! um cartão sem a ação de conversão, a atividade `Revived` e os milissegundos da criação.

use std::collections::{BTreeMap, BTreeSet};

use ph2d_kanban_model::{
    Activity, Background, Card, CoverFill, FieldKind, FieldValue, FracKey, Happened, Kanban, Link,
    Member, Moment, Subject,
};
use serde_json::{Map, Value, json};

use crate::trello::{
    CARD_SCHEME, FILE_SCHEME, GRADIENTS, format_time, hue_name, label_color_name, object_id,
};

/// O passo entre posições que o Trello usa.
pub(crate) const POS_STEP: f64 = 16384.0;

/// As posições do Trello de um grupo: a ordem `FracKey` (e o id desempata) × [`POS_STEP`].
pub(crate) fn positions<'a>(v: impl Iterator<Item = (u64, &'a FracKey)>) -> BTreeMap<u64, f64> {
    let mut v: Vec<_> = v.collect();
    v.sort_by(|a, b| a.1.cmp(b.1).then(a.0.cmp(&b.0)));
    v.into_iter()
        .enumerate()
        .map(|(i, (id, _))| (id, (i + 1) as f64 * POS_STEP))
        .collect()
}

/// Os ids de texto de uma exportação (a de um cartão leva a hora da criação, como no Trello).
pub(crate) struct Ids {
    board: String,
    at: BTreeMap<u64, Moment>,
    members: BTreeMap<Member, String>,
}

impl Ids {
    pub(crate) fn new(k: &Kanban) -> Self {
        let mut at = BTreeMap::new();
        at.extend(k.cards.values().map(|c| (c.id.0, c.created)));
        at.extend(k.comments.values().map(|c| (c.id.0, c.at)));
        at.extend(k.attachments.values().map(|a| (a.id.0, a.at)));
        at.extend(k.activities.values().map(|a| (a.id.0, a.at)));
        let mut people: BTreeSet<&Member> = BTreeSet::new();
        for c in k.cards.values().filter(|c| !c.meta.deleted) {
            people.extend(c.members.iter().chain(&c.votes));
        }
        people.extend(k.items.values().filter_map(|i| i.member.as_ref()));
        people.extend(k.comments.values().map(|c| &c.author));
        for a in k.activities.values() {
            people.insert(&a.by);
            if let Happened::MemberAdded(m) | Happened::MemberRemoved(m) = &a.what {
                people.insert(m);
            }
        }
        let members = people
            .into_iter()
            .enumerate()
            .map(|(i, m)| (m.clone(), format!("6d656d62{:016x}", i + 1)))
            .collect();
        Self {
            board: object_id(k.id.0, None),
            at,
            members,
        }
    }

    pub(crate) fn of(&self, raw: u64) -> String {
        object_id(raw, self.at.get(&raw).copied())
    }

    fn member(&self, m: &Member) -> String {
        self.members.get(m).cloned().unwrap_or_default()
    }

    fn person(&self, m: &Member) -> Value {
        json!({ "id": self.member(m), "username": m.0, "fullName": m.0 })
    }
}

fn time(m: Option<Moment>) -> Value {
    m.map_or(Value::Null, |m| Value::String(format_time(m)))
}

/// Exporta o Kanban no formato do «Export as JSON» do Trello (texto com indentação).
#[must_use]
pub fn export_trello_json(k: &Kanban) -> String {
    serde_json::to_string_pretty(&board(k)).unwrap_or_default()
}

fn board(k: &Kanban) -> Value {
    let ids = Ids::new(k);
    let background = match k.header.background {
        Background::Color(h) => hue_name(h).to_owned(),
        Background::Gradient(i) => GRADIENTS
            .get(usize::from(i))
            .copied()
            .unwrap_or("blue")
            .to_owned(),
        Background::Image(_) => "blue".to_owned(),
    };
    let labels: Vec<Value> = k
        .labels
        .values()
        .filter(|l| !l.meta.deleted)
        .map(|l| {
            json!({
                "id": ids.of(l.id.0), "idBoard": ids.board, "name": l.name,
                "color": l.color.map(label_color_name),
            })
        })
        .collect();
    let mut label_names = Map::new();
    for c in ph2d_kanban_model::LabelColor::all() {
        let name = k
            .labels
            .values()
            .find(|l| !l.meta.deleted && l.color == Some(c))
            .map_or("", |l| l.name.as_str());
        label_names.insert(label_color_name(c), Value::String(name.to_owned()));
    }
    let list_pos = positions(
        k.lists
            .values()
            .filter(|l| !l.meta.deleted)
            .map(|l| (l.id.0, &l.pos)),
    );
    let lists: Vec<Value> = k
        .lists
        .values()
        .filter(|l| !l.meta.deleted)
        .map(|l| {
            json!({
                "id": ids.of(l.id.0), "idBoard": ids.board, "name": l.title, "closed": l.archived,
                "pos": list_pos.get(&l.id.0), "softLimit": l.card_limit.map(|n| n.to_string()),
                "color": l.color.map(hue_name), "subscribed": false,
            })
        })
        .collect();
    let members: Vec<Value> = ids.members.keys().map(|m| ids.person(m)).collect();
    json!({
        "id": ids.board,
        "name": k.header.name,
        "desc": "",
        "closed": false,
        "starred": k.header.starred,
        "url": "",
        "prefs": { "background": background, "backgroundImage": null },
        "labelNames": label_names,
        "members": members,
        "labels": labels,
        "lists": lists,
        "customFields": fields(k, &ids),
        "cards": cards(k, &ids),
        "checklists": checklists(k, &ids),
        "actions": actions(k, &ids),
    })
}

fn fields(k: &Kanban, ids: &Ids) -> Vec<Value> {
    let alive = || k.fields.values().filter(|f| !f.meta.deleted);
    let pos = positions(alive().map(|f| (f.id.0, &f.pos)));
    alive()
        .map(|f| {
            let fid = ids.of(f.id.0);
            let (kind, options) = match &f.kind {
                FieldKind::Text => ("text", Vec::new()),
                FieldKind::Number => ("number", Vec::new()),
                FieldKind::Date => ("date", Vec::new()),
                FieldKind::Checkbox => ("checkbox", Vec::new()),
                FieldKind::Dropdown(opts) => (
                    "list",
                    // Saem pela ordem dos ids (a da reimportação); o `pos` guarda a do menu.
                    {
                        let mut v: Vec<_> = opts.iter().enumerate().collect();
                        v.sort_by_key(|(_, o)| o.id);
                        v
                    }
                    .into_iter()
                    .map(|(i, o)| {
                        json!({
                            "id": ids.of(o.id.0), "idCustomField": fid,
                            "value": { "text": o.text },
                            "color": o.color.map_or("none", hue_name),
                            "pos": (i + 1) as f64 * POS_STEP,
                        })
                    })
                    .collect(),
                ),
            };
            json!({
                "id": fid, "idModel": ids.board, "modelType": "board", "name": f.name,
                "type": kind, "pos": pos.get(&f.id.0), "options": options,
                "display": { "cardFront": f.on_front },
            })
        })
        .collect()
}

fn field_item(k: &Kanban, ids: &Ids, card: &Card, fid: u64, v: &FieldValue) -> Option<Value> {
    let def = k
        .fields
        .get(&ph2d_kanban_model::FieldId(fid))
        .filter(|f| !f.meta.deleted)?;
    let mut item = json!({
        "id": format!("{}{}", &ids.of(card.id.0)[..12], &ids.of(fid)[12..]),
        "idCustomField": ids.of(fid), "idModel": ids.of(card.id.0), "modelType": "card",
    });
    let value = match (v, &def.kind) {
        (FieldValue::Text(t), FieldKind::Text) => json!({ "text": t }),
        (FieldValue::Number(n), FieldKind::Number) => json!({ "number": n.to_string() }),
        (FieldValue::Date(d), FieldKind::Date) => json!({ "date": format_time(*d) }),
        (FieldValue::Checkbox(b), FieldKind::Checkbox) => json!({ "checked": b.to_string() }),
        (FieldValue::Option(o), FieldKind::Dropdown(opts)) if opts.iter().any(|x| x.id == *o) => {
            item["idValue"] = Value::String(ids.of(o.0));
            return Some(item);
        }
        _ => return None,
    };
    item["value"] = value;
    Some(item)
}

fn link_url(l: &Link) -> (String, bool) {
    match l {
        Link::Url(u) => (u.clone(), false),
        Link::File(f) => (format!("{FILE_SCHEME}{}", f.hex()), true),
        Link::Card(kb, c) => (format!("{CARD_SCHEME}{}/card/{}", kb.0, c.0), false),
        Link::Kanban(kb) => (format!("{CARD_SCHEME}{}", kb.0), false),
    }
}

fn cards(k: &Kanban, ids: &Ids) -> Vec<Value> {
    let alive: Vec<&Card> = k.cards.values().filter(|c| !c.meta.deleted).collect();
    let mut pos = BTreeMap::new();
    let lists: BTreeSet<_> = alive.iter().map(|c| c.list).collect();
    for l in lists {
        pos.extend(positions(
            alive
                .iter()
                .filter(|c| c.list == l)
                .map(|c| (c.id.0, &c.pos)),
        ));
    }
    let names = |v: &[Member]| -> Vec<String> { v.iter().map(|m| ids.member(m)).collect() };
    alive
        .iter()
        .map(|c| {
            let atts: Vec<_> = k
                .attachments
                .values()
                .filter(|a| a.card == c.id && !a.meta.deleted)
                .collect();
            let apos = positions(atts.iter().map(|a| (a.id.0, &a.pos)));
            let attachments: Vec<Value> = atts
                .iter()
                .map(|a| {
                    let (url, upload) = link_url(&a.link);
                    json!({
                        "id": ids.of(a.id.0), "name": a.name, "url": url, "isUpload": upload,
                        "fileName": upload.then_some(&a.name), "date": format_time(a.at),
                        "pos": apos.get(&a.id.0),
                    })
                })
                .collect();
            let cover = c.cover.and_then(|cv| {
                let (att, color) = match cv.fill {
                    CoverFill::Color(h) => (None, Some(hue_name(h))),
                    CoverFill::Image(f) => (
                        Some(
                            atts.iter()
                                .find(|a| a.link == Link::File(f))
                                .map(|a| ids.of(a.id.0))?,
                        ),
                        None,
                    ),
                };
                Some(json!({
                    "idAttachment": att, "color": color, "idUploadedBackground": null,
                    "size": if cv.full { "full" } else { "normal" }, "brightness": "dark",
                }))
            });
            let items: Vec<Value> = c
                .fields
                .iter()
                .filter_map(|(f, v)| field_item(k, ids, c, f.0, v))
                .collect();
            let labels: Vec<String> = c
                .labels
                .iter()
                .filter(|l| k.labels.get(l).is_some_and(|l| !l.meta.deleted))
                .map(|l| ids.of(l.0))
                .collect();
            json!({
                "id": ids.of(c.id.0), "idBoard": ids.board, "idList": ids.of(c.list.0),
                "name": c.title, "desc": c.description, "pos": pos.get(&c.id.0),
                "closed": c.archived, "idLabels": labels, "idMembers": names(&c.members),
                "idMembersVoted": names(&c.votes), "start": time(c.start),
                "due": time(c.due.map(|d| d.at)), "dueComplete": c.due.is_some_and(|d| d.done),
                "dueReminder": c.due.and_then(|d| d.remind_before_min),
                "cover": cover.unwrap_or(json!({
                    "idAttachment": null, "color": null, "idUploadedBackground": null,
                    "size": "normal", "brightness": "dark",
                })),
                "locationName": c.location, "customFieldItems": items,
                "attachments": attachments, "isTemplate": c.template, "subscribed": false,
            })
        })
        .collect()
}

fn checklists(k: &Kanban, ids: &Ids) -> Vec<Value> {
    let alive: Vec<_> = k
        .checklists
        .values()
        .filter(|c| !c.meta.deleted && k.cards.get(&c.card).is_some_and(|x| !x.meta.deleted))
        .collect();
    let mut pos = BTreeMap::new();
    for card in alive.iter().map(|c| c.card).collect::<BTreeSet<_>>() {
        pos.extend(positions(
            alive
                .iter()
                .filter(|c| c.card == card)
                .map(|c| (c.id.0, &c.pos)),
        ));
    }
    alive
        .iter()
        .map(|cl| {
            let its: Vec<_> = k
                .items
                .values()
                .filter(|i| i.checklist == cl.id && !i.meta.deleted)
                .collect();
            let ipos = positions(its.iter().map(|i| (i.id.0, &i.pos)));
            let items: Vec<Value> = its
                .iter()
                .map(|i| {
                    json!({
                        "id": ids.of(i.id.0), "idChecklist": ids.of(cl.id.0), "name": i.text,
                        "state": if i.done { "complete" } else { "incomplete" },
                        "pos": ipos.get(&i.id.0), "idMember": i.member.as_ref().map(|m| ids.member(m)),
                        "due": time(i.due),
                    })
                })
                .collect();
            json!({
                "id": ids.of(cl.id.0), "idBoard": ids.board, "idCard": ids.of(cl.card.0),
                "name": cl.title, "pos": pos.get(&cl.id.0), "checkItems": items,
            })
        })
        .collect()
}

fn actions(k: &Kanban, ids: &Ids) -> Vec<Value> {
    let mut out: Vec<(u64, Value)> = Vec::new();
    let board = json!({ "id": ids.board, "name": k.header.name });
    let card = |c: u64| {
        let name = k
            .cards
            .get(&ph2d_kanban_model::CardId(c))
            .map_or("", |c| c.title.as_str());
        json!({ "id": ids.of(c), "name": name })
    };
    for c in k.comments.values().filter(|c| !c.meta.deleted) {
        let mut data = json!({ "text": c.text, "card": card(c.card.0), "board": board });
        if let Some(e) = c.edited {
            data["dateLastEdited"] = Value::String(format_time(e));
        }
        out.push((
            c.id.0,
            envelope(ids, c.id.0, "commentCard", &c.author, c.at, data, None),
        ));
    }
    for a in k.activities.values().filter(|a| !a.meta.deleted) {
        if let Some(v) = activity(k, ids, a, &board, &card) {
            out.push((a.id.0, v));
        }
    }
    out.sort_by(|a, b| b.0.cmp(&a.0));
    out.into_iter().map(|(_, v)| v).collect()
}

fn envelope(
    ids: &Ids,
    raw: u64,
    t: &str,
    by: &Member,
    at: Moment,
    data: Value,
    member: Option<&Member>,
) -> Value {
    let mut v = json!({
        "id": ids.of(raw), "idMemberCreator": ids.member(by), "type": t,
        "date": format_time(at), "data": data, "memberCreator": ids.person(by),
    });
    if let Some(m) = member {
        v["member"] = ids.person(m);
    }
    v
}

fn activity(
    k: &Kanban,
    ids: &Ids,
    a: &Activity,
    board: &Value,
    card: &dyn Fn(u64) -> Value,
) -> Option<Value> {
    let named = |raw: u64, name: &str| json!({ "id": ids.of(raw), "name": name });
    let list_name = |l: u64| {
        k.lists
            .get(&ph2d_kanban_model::ListId(l))
            .map_or("", |l| l.title.as_str())
            .to_owned()
    };
    let (t, mut data, member) = match (&a.subject, &a.what) {
        (Subject::Card(c), Happened::Created { list: Some(l) }) => (
            "createCard",
            json!({ "card": card(c.0), "list": named(l.0, &list_name(l.0)) }),
            None,
        ),
        (Subject::Card(c), Happened::Converted { from }) => (
            "convertToCardFromCheckItem",
            json!({ "card": card(c.0), "cardSource": card(from.0) }),
            None,
        ),
        (Subject::Card(c), Happened::Deleted) => ("deleteCard", json!({ "card": card(c.0) }), None),
        (Subject::List(l), Happened::Created { list: None }) => (
            "createList",
            json!({ "list": named(l.0, &list_name(l.0)) }),
            None,
        ),
        (Subject::Kanban, Happened::Created { list: None }) => ("createBoard", json!({}), None),
        (Subject::Card(c), Happened::Moved { from, to }) => (
            "updateCard",
            json!({
                "card": card(c.0), "old": { "idList": ids.of(from.0) },
                "listBefore": named(from.0, &list_name(from.0)),
                "listAfter": named(to.0, &list_name(to.0)),
            }),
            None,
        ),
        (sub, Happened::Renamed { from }) => {
            let (t, key, obj) = subject(sub, board, card, &named, &list_name);
            (t, json!({ key: obj, "old": { "name": from } }), None)
        }
        (sub, Happened::Archived | Happened::Unarchived) => {
            let closed = a.what == Happened::Archived;
            let (t, key, mut obj) = subject(sub, board, card, &named, &list_name);
            obj["closed"] = Value::Bool(closed);
            (t, json!({ key: obj, "old": { "closed": !closed } }), None)
        }
        (Subject::Card(c), what) => card_update(ids, c.0, what, card)?,
        _ => return None,
    };
    if data.get("board").is_none() {
        data["board"] = board.clone();
    }
    Some(envelope(ids, a.id.0, t, &a.by, a.at, data, member.as_ref()))
}

type Named<'a> = &'a dyn Fn(u64, &str) -> Value;

fn subject(
    sub: &Subject,
    board: &Value,
    card: &dyn Fn(u64) -> Value,
    named: Named,
    list_name: &dyn Fn(u64) -> String,
) -> (&'static str, &'static str, Value) {
    match sub {
        Subject::Kanban => ("updateBoard", "board", board.clone()),
        Subject::List(l) => ("updateList", "list", named(l.0, &list_name(l.0))),
        Subject::Card(c) => ("updateCard", "card", card(c.0)),
    }
}

fn card_update(
    ids: &Ids,
    c: u64,
    what: &Happened,
    card: &dyn Fn(u64) -> Value,
) -> Option<(&'static str, Value, Option<Member>)> {
    let mut obj = card(c);
    let with = |obj: Value, old: Value| json!({ "card": obj, "old": old });
    Some(match what {
        Happened::DescriptionChanged => ("updateCard", with(obj, json!({ "desc": "" })), None),
        Happened::StartSet(t) => {
            obj["start"] = time(*t);
            ("updateCard", with(obj, json!({ "start": null })), None)
        }
        Happened::DueSet(t) => {
            obj["due"] = time(*t);
            ("updateCard", with(obj, json!({ "due": null })), None)
        }
        Happened::DueDone(b) => {
            obj["dueComplete"] = Value::Bool(*b);
            ("updateCard", with(obj, json!({ "dueComplete": !b })), None)
        }
        Happened::MemberAdded(m) => ("addMemberToCard", json!({ "card": obj }), Some(m.clone())),
        Happened::MemberRemoved(m) => (
            "removeMemberFromCard",
            json!({ "card": obj }),
            Some(m.clone()),
        ),
        Happened::ChecklistAdded { checklist, title }
        | Happened::ChecklistRemoved { checklist, title } => {
            let t = if matches!(what, Happened::ChecklistAdded { .. }) {
                "addChecklistToCard"
            } else {
                "removeChecklistFromCard"
            };
            (
                t,
                json!({ "card": obj, "checklist": { "id": ids.of(checklist.0), "name": title } }),
                None,
            )
        }
        Happened::ItemChecked { item, text, done } => (
            "updateCheckItemStateOnCard",
            json!({ "card": obj, "checkItem": {
                "id": ids.of(item.0), "name": text,
                "state": if *done { "complete" } else { "incomplete" },
            } }),
            None,
        ),
        Happened::Attached { attachment, name } | Happened::Detached { attachment, name } => {
            let t = if matches!(what, Happened::Attached { .. }) {
                "addAttachmentToCard"
            } else {
                "deleteAttachmentFromCard"
            };
            (
                t,
                json!({ "card": obj, "attachment": { "id": ids.of(attachment.0), "name": name } }),
                None,
            )
        }
        _ => return None,
    })
}
