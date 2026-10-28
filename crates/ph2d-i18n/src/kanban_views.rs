//! O que as VISTAS do Kanban dizem (TrelloClone W7): Tabela, Calendário, Linha do tempo e Painel.
//! Uma tabela por vista (`kanban_views/<vista>.rs`); aqui os nomes do seletor.

#[path = "kanban_views/calendar.rs"]
mod calendar;
#[path = "kanban_views/dashboard.rs"]
mod dashboard;
#[path = "kanban_views/table.rs"]
mod table;
#[path = "kanban_views/timeline.rs"]
mod timeline;

/// A tradução de uma chave das vistas, ou `None` se ela não é daqui.
pub(crate) fn tr(key: &str) -> Option<&'static str> {
    let rest = key.strip_prefix("kanban.views.")?;
    Some(match rest {
        "board" => "Board",
        "table" => "Table",
        "calendar" => "Calendar",
        "timeline" => "Timeline",
        "dashboard" => "Dashboard",
        "no_cards" => "No cards match.",
        _ => {
            return table::tr(rest)
                .or_else(|| calendar::tr(rest))
                .or_else(|| timeline::tr(rest))
                .or_else(|| dashboard::tr(rest));
        }
    })
}
