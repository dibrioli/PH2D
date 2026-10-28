//! **O Kanban no Quadro** (TrelloClone W4, ADR-0195 §3): pôr um Kanban no canvas e abri-lo na aba.

/// A tradução de uma chave do Kanban no Quadro, ou `None` se ela não é daqui.
pub(crate) fn tr(key: &str) -> Option<&'static str> {
    Some(match key {
        "board.tool.kanban" => "Kanban",
        "board.kanban.new_name" => "Kanban",
        "board.kanban.open_tab" => "Open in tab",
        "board.kanban.no_tab" => "Opening a Kanban in its own tab isn\u{2019}t available yet",
        _ => return None,
    })
}
