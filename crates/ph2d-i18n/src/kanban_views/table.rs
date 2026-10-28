//! As palavras da vista `table` do Kanban (chaves `kanban.views.table.*`) — as do Trello.

/// A tradução de `table.<x>` (já sem o prefixo `kanban.views.`), ou `None`.
pub(crate) fn tr(rest: &str) -> Option<&'static str> {
    let k = rest.strip_prefix("table.")?;
    Some(match k {
        "col.card" => "Card",
        "col.list" => "List",
        "col.labels" => "Labels",
        "col.members" => "Members",
        "col.due" => "Due date",
        "more" => "+{n}",
        "none" => "\u{2014}",
        _ => return None,
    })
}
