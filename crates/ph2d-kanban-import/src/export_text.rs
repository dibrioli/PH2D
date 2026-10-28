//! **CSV e markdown** — as duas exportações para gente e folhas de cálculo.
//!
//! CSV: uma linha por cartão (vivo ou arquivado), as colunas do CSV do Trello Premium, RFC 4180
//! (CRLF; aspas em volta do campo que tem vírgula, aspas ou quebra; aspas dobradas por dentro).
//! Markdown: o Kanban como documento, determinístico (o mesmo Kanban dá os mesmos bytes).

use std::fmt::Write as _;

use ph2d_kanban_model::{Card, Kanban, List, ListId, Member};

use crate::trello::{format_time, label_color_name, object_id};

/// As colunas do CSV, pela ordem.
pub const CSV_COLUMNS: [&str; 15] = [
    "Card ID",
    "Card Name",
    "Card URL",
    "Card Description",
    "Labels",
    "Members",
    "Due Date",
    "Start Date",
    "Due Complete",
    "Checklist Items Total",
    "Checklist Items Completed",
    "Attachment Count",
    "List Name",
    "Archived",
    "Created",
];

fn lists_sorted(k: &Kanban) -> Vec<&List> {
    let mut v: Vec<&List> = k.lists.values().filter(|l| !l.meta.deleted).collect();
    v.sort_by(|a, b| a.pos.cmp(&b.pos).then(a.id.cmp(&b.id)));
    v
}

/// Os cartões vivos (arquivados incluídos) de `list`, pela ordem.
fn cards_of(k: &Kanban, list: ListId) -> Vec<&Card> {
    let mut v: Vec<&Card> = k
        .cards
        .values()
        .filter(|c| c.list == list && !c.meta.deleted)
        .collect();
    v.sort_by(|a, b| a.pos.cmp(&b.pos).then(a.id.cmp(&b.id)));
    v
}

/// Cartões cuja lista não existe viva (vão no fim do CSV, sem nome de lista).
fn homeless(k: &Kanban) -> Vec<&Card> {
    k.cards
        .values()
        .filter(|c| !c.meta.deleted && k.lists.get(&c.list).is_none_or(|l| l.meta.deleted))
        .collect()
}

fn label_text(k: &Kanban, c: &Card) -> Vec<String> {
    c.labels
        .iter()
        .filter_map(|l| k.labels.get(l).filter(|l| !l.meta.deleted))
        .map(|l| match (l.name.is_empty(), l.color) {
            (_, None) => l.name.clone(),
            (true, Some(col)) => label_color_name(col),
            (false, Some(col)) => format!("{} ({})", l.name, label_color_name(col)),
        })
        .collect()
}

fn people(v: &[Member]) -> String {
    v.iter()
        .map(|m| m.0.as_str())
        .collect::<Vec<_>>()
        .join(", ")
}

/// (total, feitos) dos itens das checklists vivas do cartão.
fn check_counts(k: &Kanban, c: &Card) -> (usize, usize) {
    let mut n = (0, 0);
    for cl in k
        .checklists
        .values()
        .filter(|cl| cl.card == c.id && !cl.meta.deleted)
    {
        for it in k
            .items
            .values()
            .filter(|i| i.checklist == cl.id && !i.meta.deleted)
        {
            n.0 += 1;
            n.1 += usize::from(it.done);
        }
    }
    n
}

fn csv_field(out: &mut String, f: &str) {
    if f.contains([',', '"', '\r', '\n']) {
        out.push('"');
        out.push_str(&f.replace('"', "\"\""));
        out.push('"');
    } else {
        out.push_str(f);
    }
}

fn csv_row(out: &mut String, row: &[String]) {
    for (i, f) in row.iter().enumerate() {
        if i > 0 {
            out.push(',');
        }
        csv_field(out, f);
    }
    out.push_str("\r\n");
}

/// Exporta o Kanban como CSV (cabeçalho + uma linha por cartão).
#[must_use]
pub fn export_csv(k: &Kanban) -> String {
    let mut out = String::new();
    csv_row(&mut out, &CSV_COLUMNS.map(str::to_owned));
    let mut rows: Vec<(&Card, &str)> = Vec::new();
    for l in lists_sorted(k) {
        rows.extend(cards_of(k, l.id).into_iter().map(|c| (c, l.title.as_str())));
    }
    rows.extend(homeless(k).into_iter().map(|c| (c, "")));
    for (c, list) in rows {
        let (total, done) = check_counts(k, c);
        let atts = k
            .attachments
            .values()
            .filter(|a| a.card == c.id && !a.meta.deleted)
            .count();
        csv_row(
            &mut out,
            &[
                object_id(c.id.0, Some(c.created)),
                c.title.clone(),
                String::new(),
                c.description.clone(),
                label_text(k, c).join(", "),
                people(&c.members),
                c.due.map(|d| format_time(d.at)).unwrap_or_default(),
                c.start.map(format_time).unwrap_or_default(),
                c.due.is_some_and(|d| d.done).to_string(),
                total.to_string(),
                done.to_string(),
                atts.to_string(),
                list.to_owned(),
                c.archived.to_string(),
                format_time(c.created),
            ],
        );
    }
    out
}

/// Texto de uma linha de markdown, sem que vire formatação.
fn md_escape(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for ch in s.chars() {
        match ch {
            '\\' | '`' | '*' | '_' | '[' | ']' | '<' | '>' | '#' | '|' | '~' => {
                out.push('\\');
                out.push(ch);
            }
            '\r' | '\n' => out.push(' '),
            _ => out.push(ch),
        }
    }
    out
}

fn md_card(out: &mut String, k: &Kanban, c: &Card, list: Option<&str>) {
    let mark = if c.due.is_some_and(|d| d.done) {
        'x'
    } else {
        ' '
    };
    let _ = write!(out, "- [{mark}] {}", md_escape(&c.title));
    let mut meta: Vec<String> = Vec::new();
    if let Some(l) = list {
        meta.push(format!("in {}", md_escape(l)));
    }
    let labels = label_text(k, c);
    if !labels.is_empty() {
        meta.push(md_escape(&labels.join(", ")));
    }
    if !c.members.is_empty() {
        let ms: Vec<String> = c
            .members
            .iter()
            .map(|m| format!("@{}", md_escape(&m.0)))
            .collect();
        meta.push(ms.join(" "));
    }
    if let Some(s) = c.start {
        meta.push(format!("start {}", format_time(s)));
    }
    if let Some(d) = c.due {
        meta.push(format!("due {}", format_time(d.at)));
    }
    if !meta.is_empty() {
        let _ = write!(out, " — {}", meta.join(" · "));
    }
    out.push('\n');
    let desc = c.description.trim_end();
    if !desc.is_empty() {
        out.push('\n');
        for line in desc.lines() {
            if line.trim().is_empty() {
                out.push('\n');
            } else {
                let _ = writeln!(out, "  {line}");
            }
        }
        out.push('\n');
    }
    let mut cls: Vec<_> = k
        .checklists
        .values()
        .filter(|cl| cl.card == c.id && !cl.meta.deleted)
        .collect();
    cls.sort_by(|a, b| a.pos.cmp(&b.pos).then(a.id.cmp(&b.id)));
    for cl in cls {
        let _ = writeln!(out, "  - {}", md_escape(&cl.title));
        let mut its: Vec<_> = k
            .items
            .values()
            .filter(|i| i.checklist == cl.id && !i.meta.deleted)
            .collect();
        its.sort_by(|a, b| a.pos.cmp(&b.pos).then(a.id.cmp(&b.id)));
        for it in its {
            let mark = if it.done { 'x' } else { ' ' };
            let _ = write!(out, "    - [{mark}] {}", md_escape(&it.text));
            let mut meta = Vec::new();
            if let Some(m) = &it.member {
                meta.push(format!("@{}", md_escape(&m.0)));
            }
            if let Some(d) = it.due {
                meta.push(format!("due {}", format_time(d)));
            }
            if !meta.is_empty() {
                let _ = write!(out, " — {}", meta.join(" · "));
            }
            out.push('\n');
        }
    }
}

/// Exporta o Kanban como um documento markdown: `#` o Kanban, `##` cada lista, `- [ ]` cada cartão
/// (marcado = entrega concluída), e no fim `## Archived` com o que está arquivado.
#[must_use]
pub fn export_markdown(k: &Kanban) -> String {
    let mut out = format!("# {}\n", md_escape(&k.header.name));
    let lists = lists_sorted(k);
    let section = |out: &mut String, l: &List, level: &str| {
        let _ = write!(out, "\n{level} {}\n\n", md_escape(&l.title));
        for c in cards_of(k, l.id).into_iter().filter(|c| !c.archived) {
            md_card(out, k, c, None);
        }
    };
    for l in lists.iter().filter(|l| !l.archived) {
        section(&mut out, l, "##");
    }
    let archived_lists: Vec<_> = lists.iter().filter(|l| l.archived).collect();
    let archived_cards: Vec<(&Card, &str)> = lists
        .iter()
        .flat_map(|l| {
            cards_of(k, l.id)
                .into_iter()
                .filter(|c| c.archived)
                .map(|c| (c, l.title.as_str()))
        })
        .collect();
    if archived_lists.is_empty() && archived_cards.is_empty() {
        return out;
    }
    out.push_str("\n## Archived\n");
    for l in archived_lists {
        section(&mut out, l, "###");
    }
    if !archived_cards.is_empty() {
        out.push_str("\n### Archived cards\n\n");
        for (c, list) in archived_cards {
            md_card(&mut out, k, c, Some(list));
        }
    }
    out
}
