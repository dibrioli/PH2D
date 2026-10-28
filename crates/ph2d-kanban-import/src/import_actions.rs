//! **As ações do Trello → comentários e atividade.** Só entra o que o Trello registou (nada se
//! inventa): `commentCard` vira [`Comment`], as ações com par em [`Happened`] viram [`Activity`], e
//! o resto conta no relatório por tipo.
//!
//! Cada ação valida-se INTEIRA antes de dar um id a qualquer coisa: uma ação recusada a meio que já
//! tivesse gasto ids mudaria os ids de tudo o que vem depois, e a ida-e-volta deixaria de ser exacta.

use std::collections::BTreeMap;

use ph2d_kanban_model::{
    Activity, ActivityId, AttachmentId, CardId, CheckItemId, ChecklistId, Comment, CommentId,
    Happened, ListId, Member, Meta, Moment, Record, Subject,
};
use serde_json::Value;

use crate::import::{Ctx, arr, s};
use crate::report::LossKind;
use crate::trello::parse_time;

enum Sub<'a> {
    Kanban,
    List(&'a str),
    Card(&'a str),
}

/// Um [`Happened`] com os ids ainda do Trello.
enum Hap<'a> {
    Created(Option<&'a str>),
    Converted(&'a str),
    Moved(&'a str, &'a str),
    Renamed(String),
    Description,
    Start(Option<Moment>),
    Due(Option<Moment>),
    DueDone(bool),
    MemberAdded(Member),
    MemberRemoved(Member),
    Archived(bool),
    Deleted,
    Checklist(bool, &'a str, String),
    Item(&'a str, String, bool),
    Attachment(bool, &'a str, String),
}

fn id_in<'a>(d: &'a Value, obj: &str) -> Option<&'a str> {
    d.get(obj).and_then(|o| s(o, "id"))
}

fn name_in(d: &Value, obj: &str) -> String {
    d.get(obj)
        .and_then(|o| s(o, "name"))
        .unwrap_or("")
        .to_owned()
}

/// Uma data que pode ser `null` (tirada): `None` dentro = sem data; `None` fora = valor ruim.
fn opt_time(v: Option<&Value>) -> Option<Option<Moment>> {
    match v {
        None | Some(Value::Null) => Some(None),
        Some(Value::String(t)) => parse_time(t).map(Some),
        _ => None,
    }
}

/// O que uma `updateX` mudou, pela ordem fixa das chaves de `data.old`.
fn updates<'a>(d: &'a Value, obj: &str) -> Vec<Hap<'a>> {
    let Some(old) = d.get("old").and_then(Value::as_object) else {
        return Vec::new();
    };
    let now = d.get(obj).unwrap_or(&Value::Null);
    let mut out = Vec::new();
    let card = obj == "card";
    if card
        && old.contains_key("idList")
        && let (Some(a), Some(b)) = (id_in(d, "listBefore"), id_in(d, "listAfter"))
    {
        out.push(Hap::Moved(a, b));
    }
    if let Some(n) = old.get("name").and_then(Value::as_str) {
        out.push(Hap::Renamed(n.to_owned()));
    }
    if card && old.contains_key("desc") {
        out.push(Hap::Description);
    }
    if card
        && old.contains_key("start")
        && let Some(t) = opt_time(now.get("start"))
    {
        out.push(Hap::Start(t));
    }
    if card
        && old.contains_key("due")
        && let Some(t) = opt_time(now.get("due"))
    {
        out.push(Hap::Due(t));
    }
    if card
        && old.contains_key("dueComplete")
        && let Some(b) = now.get("dueComplete").and_then(Value::as_bool)
    {
        out.push(Hap::DueDone(b));
    }
    if old.contains_key("closed")
        && let Some(b) = now.get("closed").and_then(Value::as_bool)
    {
        out.push(Hap::Archived(b));
    }
    out
}

fn member_of(cx: &Ctx, a: &Value, d: &Value) -> Option<Member> {
    a.get("member")
        .and_then(|m| s(m, "username"))
        .map(|u| Member(u.to_owned()))
        .or_else(|| s(d, "idMember").and_then(|m| cx.members.get(m).cloned()))
}

fn parse<'a>(cx: &Ctx, t: &str, a: &'a Value, d: &'a Value) -> Option<(Sub<'a>, Vec<Hap<'a>>)> {
    let card = || id_in(d, "card").map(Sub::Card);
    Some(match t {
        "createCard" | "copyCard" => (card()?, vec![Hap::Created(Some(id_in(d, "list")?))]),
        "convertToCardFromCheckItem" => (card()?, vec![Hap::Converted(id_in(d, "cardSource")?)]),
        "deleteCard" => (card()?, vec![Hap::Deleted]),
        "createList" => (Sub::List(id_in(d, "list")?), vec![Hap::Created(None)]),
        "createBoard" => (Sub::Kanban, vec![Hap::Created(None)]),
        "updateCard" => (card()?, updates(d, "card")),
        "updateList" => (Sub::List(id_in(d, "list")?), updates(d, "list")),
        "updateBoard" => (Sub::Kanban, updates(d, "board")),
        "addMemberToCard" => (card()?, vec![Hap::MemberAdded(member_of(cx, a, d)?)]),
        "removeMemberFromCard" => (card()?, vec![Hap::MemberRemoved(member_of(cx, a, d)?)]),
        "addChecklistToCard" | "removeChecklistFromCard" => {
            let cl = id_in(d, "checklist")?;
            let added = t == "addChecklistToCard";
            (
                card()?,
                vec![Hap::Checklist(added, cl, name_in(d, "checklist"))],
            )
        }
        "updateCheckItemStateOnCard" => {
            let ci = d.get("checkItem")?;
            let done = s(ci, "state") == Some("complete");
            (
                card()?,
                vec![Hap::Item(s(ci, "id")?, name_in(d, "checkItem"), done)],
            )
        }
        "addAttachmentToCard" | "deleteAttachmentFromCard" => {
            let att = id_in(d, "attachment")?;
            let added = t == "addAttachmentToCard";
            (
                card()?,
                vec![Hap::Attachment(added, att, name_in(d, "attachment"))],
            )
        }
        _ => return None,
    })
}

fn resolve(cx: &mut Ctx, sub: &Sub, hap: Hap) -> (Subject, Happened) {
    let subject = match sub {
        Sub::Kanban => Subject::Kanban,
        Sub::List(l) => Subject::List(ListId(cx.alloc(l))),
        Sub::Card(c) => Subject::Card(CardId(cx.alloc(c))),
    };
    let what = match hap {
        Hap::Created(l) => Happened::Created {
            list: l.map(|l| ListId(cx.alloc(l))),
        },
        Hap::Converted(f) => Happened::Converted {
            from: CardId(cx.alloc(f)),
        },
        Hap::Moved(a, b) => Happened::Moved {
            from: ListId(cx.alloc(a)),
            to: ListId(cx.alloc(b)),
        },
        Hap::Renamed(from) => Happened::Renamed { from },
        Hap::Description => Happened::DescriptionChanged,
        Hap::Start(t) => Happened::StartSet(t),
        Hap::Due(t) => Happened::DueSet(t),
        Hap::DueDone(b) => Happened::DueDone(b),
        Hap::MemberAdded(m) => Happened::MemberAdded(m),
        Hap::MemberRemoved(m) => Happened::MemberRemoved(m),
        Hap::Archived(true) => Happened::Archived,
        Hap::Archived(false) => Happened::Unarchived,
        Hap::Deleted => Happened::Deleted,
        Hap::Checklist(added, c, title) => {
            let checklist = ChecklistId(cx.alloc(c));
            if added {
                Happened::ChecklistAdded { checklist, title }
            } else {
                Happened::ChecklistRemoved { checklist, title }
            }
        }
        Hap::Item(i, text, done) => Happened::ItemChecked {
            item: CheckItemId(cx.alloc(i)),
            text,
            done,
        },
        Hap::Attachment(added, att, name) => {
            let attachment = AttachmentId(cx.alloc(att));
            if added {
                Happened::Attached { attachment, name }
            } else {
                Happened::Detached { attachment, name }
            }
        }
    };
    (subject, what)
}

fn comment(cx: &mut Ctx, tid: &str, by: Member, at: Moment, d: &Value) {
    let Some(card) = id_in(d, "card")
        .and_then(|c| cx.known(c))
        .filter(|c| cx.cards.contains_key(c))
    else {
        cx.report
            .lose(LossKind::Orphan, format!("commentCard {tid}"));
        return;
    };
    let text = s(d, "text").unwrap_or("").to_owned();
    let edited = d
        .get("dateLastEdited")
        .and_then(Value::as_str)
        .and_then(parse_time);
    let id = cx.alloc(tid);
    cx.records.push(Record::Comment(Comment {
        id: CommentId(id),
        card: CardId(card),
        author: by,
        at,
        text,
        edited,
        reactions: Vec::new(),
        meta: Meta::default(),
    }));
}

/// As ações do quadro, da mais velha para a mais nova (o Trello escreve-as ao contrário).
pub(crate) fn actions(cx: &mut Ctx, board: &Value) {
    let mut unsupported: BTreeMap<String, usize> = BTreeMap::new();
    for a in arr(board, "actions").iter().rev() {
        let (Some(tid), Some(t)) = (s(a, "id"), s(a, "type")) else {
            cx.report.lose(LossKind::BadValue, "action");
            continue;
        };
        let d = a.get("data").unwrap_or(&Value::Null);
        let by = a
            .get("memberCreator")
            .and_then(|m| s(m, "username"))
            .map(|u| Member(u.to_owned()))
            .or_else(|| s(a, "idMemberCreator").and_then(|m| cx.members.get(m).cloned()));
        let parsed = if t == "commentCard" {
            None
        } else {
            match parse(cx, t, a, d) {
                Some((sub, haps)) if !haps.is_empty() => Some((sub, haps)),
                _ => {
                    *unsupported.entry(t.to_owned()).or_default() += 1;
                    continue;
                }
            }
        };
        let Some(by) = by else {
            cx.report.lose(LossKind::ActionWithoutAuthor, tid);
            continue;
        };
        let Some(at) = s(a, "date").and_then(parse_time) else {
            cx.report
                .lose(LossKind::BadValue, format!("{t} {tid}: date"));
            continue;
        };
        if cx.known(tid).is_some() {
            cx.report.lose(LossKind::DuplicateId, tid);
            continue;
        }
        let Some((sub, haps)) = parsed else {
            comment(cx, tid, by, at, d);
            continue;
        };
        let entries: Vec<(Subject, Happened)> =
            haps.into_iter().map(|h| resolve(cx, &sub, h)).collect();
        for (i, (subject, what)) in entries.into_iter().enumerate() {
            let id = if i == 0 { cx.alloc(tid) } else { cx.anon() };
            if let (Subject::Card(c), Happened::Converted { from }) = (subject, &what)
                && let Some(card) = cx.cards.get_mut(&c.0)
            {
                card.origin = Some(*from);
            }
            cx.records.push(Record::Activity(Activity {
                id: ActivityId(id),
                subject,
                by: by.clone(),
                at,
                what,
                meta: Meta::default(),
            }));
        }
    }
    for (t, n) in unsupported {
        cx.report
            .lose(LossKind::UnsupportedAction, format!("{t} ×{n}"));
    }
}
