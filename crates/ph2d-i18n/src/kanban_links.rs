//! O que os «Power-Ups» do Kanban dizem (TrelloClone W11): as ligações do cartão ao projecto e a nota
//! adesiva ↔ cartão.

/// A tradução de uma chave das ligações, ou `None` se ela não é daqui.
pub(crate) fn tr(key: &str) -> Option<&'static str> {
    Some(match key {
        "kanban.links.add" => "Scene object",
        "kanban.links.title" => "Project links",
        "kanban.links.broken" => "{name} (deleted)",
        "kanban.links.open" => "Open",
        "kanban.links.remove" => "Remove link",
        "kanban.links.kind.object" => "Object",
        "kanban.links.kind.script" => "Script",
        "kanban.links.kind.macro" => "Macro",
        "kanban.links.kind.span" => "Timeline",
        "kanban.links.kind.board" => "Board",
        "kanban.links.kind.element" => "Board element",
        "kanban.links.span_frames" => "frames {from}\u{2013}{to}",
        "kanban.links.cards_linking" => "{n} cards",
        "kanban.links.card_linking" => "1 card",
        "kanban.links.to_card" => "Convert to card",
        "kanban.links.to_card_title" => "Convert to card in\u{2026}",
        "kanban.links.to_card_empty" => "Create a Kanban with a list first.",
        "kanban.links.to_note" => "Convert to sticky note",
        "kanban.links.to_note_no_board" => "Open a board to place the sticky note.",
        _ => return None,
    })
}
