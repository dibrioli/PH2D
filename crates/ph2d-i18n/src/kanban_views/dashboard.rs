//! As palavras do Painel do Kanban (chaves `kanban.views.dashboard.*`) — as do Dashboard do Trello.

/// A tradução de `dashboard.<x>` (já sem o prefixo `kanban.views.`), ou `None`.
pub(crate) fn tr(rest: &str) -> Option<&'static str> {
    let k = rest.strip_prefix("dashboard.")?;
    Some(match k {
        "per_list" => "Cards per list",
        "per_label" => "Cards per label",
        "per_member" => "Cards per member",
        "per_due" => "Cards per due date",
        "kpi.cards" => "Cards",
        "due.complete" => "Complete",
        "due.overdue" => "Overdue",
        "due.next_day" => "Due in 24 hours",
        "due.next_week" => "Due this week",
        "due.later" => "Due later",
        "due.none" => "No due date",
        "no_label" => "No label",
        "no_member" => "No member",
        "other" => "Other ({n})",
        "cards" => "cards",
        "bars" => "Bar",
        "ring" => "Pie",
        "tip" => "{n} cards",
        "tip_share" => "{n} cards \u{b7} {p}%",
        "tip_one" => "{n} card",
        "tip_share_one" => "{n} card \u{b7} {p}%",
        _ => return None,
    })
}
