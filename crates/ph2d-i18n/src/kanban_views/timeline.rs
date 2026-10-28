//! As palavras da vista `timeline` do Kanban (chaves `kanban.views.timeline.*`) — as da Timeline do
//! Trello.

/// A tradução de `timeline.<x>` (já sem o prefixo `kanban.views.`), ou `None`.
pub(crate) fn tr(rest: &str) -> Option<&'static str> {
    let k = rest.strip_prefix("timeline.")?;
    Some(match k {
        "group_by" => "Group by",
        "group.list" => "List",
        "group.member" => "Members",
        "group.label" => "Labels",
        "zoom.day" => "Day",
        "zoom.week" => "Week",
        "zoom.month" => "Month",
        "zoom.quarter" => "Quarter",
        "today" => "Today",
        "no_members" => "No members",
        "no_labels" => "No labels",
        "unnamed_label" => "Unnamed label",
        "no_dates" => "No dates",
        "no_dates_n" => "No dates \u{b7} {n}",
        "no_dates_hint" => "Drag a card onto a day to schedule it.",
        "no_dates_empty" => "Every card here has a date.",
        "month_year" => "{month} {y}",
        "quarter" => "Q{q} {y}",
        "day" => "{wd} {d}",
        _ => return None,
    })
}
