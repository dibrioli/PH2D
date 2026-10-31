//! O que a FRENTE de um cartão do Kanban diz (TrelloClone W3): o distintivo das datas e os campos
//! personalizados mostrados na frente.

/// A tradução de uma chave da frente do cartão, ou `None` se ela não é daqui.
pub(crate) fn tr(key: &str) -> Option<&'static str> {
    Some(match key {
        "kanban.front.range" => "{start} \u{2013} {due}",
        "kanban.front.starts" => "Starts: {date}",
        "kanban.front.started" => "Started: {date}",
        "kanban.front.field" => "{name}: {value}",
        _ => return None,
    })
}
