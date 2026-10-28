//! **As variáveis** de um comentário ou título escrito por uma automação (as do Butler):
//! `{cardname}`, `{listname}`, `{boardname}`, `{username}`, `{date}`, `{time}`, `{duedate}`.
//! Datas em ISO 8601 (`2026-10-28`, `14:05`), que não dependem da língua. Uma variável desconhecida
//! fica como está.

use ph2d_kanban_dates::{CivilDate, civil};
use ph2d_kanban_model::{CardId, KanbanId, KanbanSet};

use crate::cond::Cx;

/// As variáveis que [`expand`] conhece (para a ajuda da UI).
pub const VARIABLES: [&str; 7] = [
    "cardname",
    "listname",
    "boardname",
    "username",
    "date",
    "time",
    "duedate",
];

fn iso(d: CivilDate) -> String {
    format!("{:04}-{:02}-{:02}", d.year, d.month, d.day)
}

/// `text` com as variáveis trocadas pelo cartão `card` do Kanban `kanban` (sem cartão: vazias).
#[must_use]
pub fn expand(
    text: &str,
    set: &KanbanSet,
    kanban: KanbanId,
    card: Option<CardId>,
    cx: &Cx,
) -> String {
    let k = set.get(kanban);
    let c = card.and_then(|c| k?.cards.get(&c));
    let value = |name: &str| -> Option<String> {
        let now = civil(cx.now.0, cx.zone);
        Some(match name {
            "cardname" => c.map(|c| c.title.clone()).unwrap_or_default(),
            "listname" => c
                .and_then(|c| k?.lists.get(&c.list))
                .map(|l| l.title.clone())
                .unwrap_or_default(),
            "boardname" => k.map(|k| k.header.name.clone()).unwrap_or_default(),
            "username" => cx.me.0.clone(),
            "date" => iso(now.date),
            "time" => format!("{:02}:{:02}", now.hour, now.minute),
            "duedate" => c
                .and_then(|c| c.due)
                .map(|d| iso(civil(d.at.0, cx.zone).date))
                .unwrap_or_default(),
            _ => return None,
        })
    };
    let mut out = String::with_capacity(text.len());
    let mut rest = text;
    while let Some(open) = rest.find('{') {
        out.push_str(&rest[..open]);
        let tail = &rest[open..];
        let Some(close) = tail.find('}') else {
            out.push_str(tail);
            return out;
        };
        match value(&tail[1..close]) {
            Some(v) => out.push_str(&v),
            None => out.push_str(&tail[..=close]),
        }
        rest = &tail[close + 1..];
    }
    out.push_str(rest);
    out
}
