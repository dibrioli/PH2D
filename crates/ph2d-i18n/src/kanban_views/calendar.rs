//! As palavras da vista `calendar` do Kanban (chaves `kanban.views.calendar.*`). Os nomes dos
//! meses e dos dias da semana são os do verso (`kanban.back.month.*`, `kanban.back.wd.*`).

/// A tradução de `calendar.<x>` (já sem o prefixo `kanban.views.`), ou `None`.
pub(crate) fn tr(rest: &str) -> Option<&'static str> {
    let k = rest.strip_prefix("calendar.")?;
    Some(match k {
        "title" => "{month} {y}",
        "week_title" => "{a} \u{2013} {b}",
        "today" => "Today",
        "month" => "Month",
        "week" => "Week",
        "more" => "+{n} more",
        "add" => "Add a card",
        "add_card" => "Add card",
        "cancel" => "Cancel",
        "placeholder" => "Enter a title for this card\u{2026}",
        "list" => "List",
        "at" => "{time} {title}",
        "day" => "Day",
        "agenda" => "Agenda",
        "day_title" => "{wd}, {day}",
        "span" => "{a} \u{2013} {b}",
        "hour_am" => "{h} AM",
        "hour_pm" => "{h} PM",
        "all_day" => "All day",
        "nothing" => "Nothing scheduled.",
        _ => return None,
    })
}
