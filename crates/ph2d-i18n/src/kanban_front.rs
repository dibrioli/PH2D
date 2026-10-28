//! O que a FRENTE de um cartão do Kanban diz (TrelloClone W3): o distintivo das datas e os campos
//! personalizados mostrados na frente.

/// A tradução de uma chave da frente do cartão, ou `None` se ela não é daqui.
pub(crate) fn tr(key: &str) -> Option<&'static str> {
    Some(match key {
        "kanban.front.month.1" => "Jan",
        "kanban.front.month.2" => "Feb",
        "kanban.front.month.3" => "Mar",
        "kanban.front.month.4" => "Apr",
        "kanban.front.month.5" => "May",
        "kanban.front.month.6" => "Jun",
        "kanban.front.month.7" => "Jul",
        "kanban.front.month.8" => "Aug",
        "kanban.front.month.9" => "Sep",
        "kanban.front.month.10" => "Oct",
        "kanban.front.month.11" => "Nov",
        "kanban.front.month.12" => "Dec",
        "kanban.front.date" => "{month} {day}",
        "kanban.front.date_year" => "{month} {day}, {year}",
        "kanban.front.time" => "{hour}:{minute}",
        "kanban.front.yesterday" => "Yesterday {time}",
        "kanban.front.today" => "Today {time}",
        "kanban.front.tomorrow" => "Tomorrow {time}",
        "kanban.front.range" => "{start} \u{2013} {due}",
        "kanban.front.starts" => "Starts: {date}",
        "kanban.front.started" => "Started: {date}",
        "kanban.front.field" => "{name}: {value}",
        _ => return None,
    })
}
