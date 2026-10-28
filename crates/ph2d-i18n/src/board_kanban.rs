//! **O Kanban no Quadro** (TrelloClone W4, ADR-0195 §3): pôr um Kanban no canvas e abri-lo na aba.

/// A tradução de uma chave do Kanban no Quadro, ou `None` se ela não é daqui.
pub(crate) fn tr(key: &str) -> Option<&'static str> {
    Some(match key {
        "board.tool.kanban" => "Kanban",
        "board.kanban.new_name" => "Kanban",
        "board.kanban.open_tab" => "Open in tab",
        "board.kanban.no_tab" => "Opening a Kanban in its own tab isn\u{2019}t available yet",
        // Cena 30 do Quadro (W4): o Kanban da cena 1 ao lado de três notas.
        "board.smoke.w4k.board" => "Production board",
        "board.smoke.w4k.n1" => "Pitch to the studio on Friday",
        "board.smoke.w4k.n2" => "Animatic \u{2014} v2",
        "board.smoke.w4k.n3" => "Voice \u{2014} schedule",
        // W4, lote V — o «…» da aba do Kanban liga-a ao Quadro.
        "kanban.menu.show_on_board" => "Show on board",
        "kanban.menu.place_on_board" => "Place on board",
        "kanban.menu.boards_title" => "Show on which board?",
        "kanban.board.nowhere" => "This Kanban isn\u{2019}t on any board yet \u{2014} use \u{201c}Place on board\u{201d}",
        _ => return None,
    })
}
