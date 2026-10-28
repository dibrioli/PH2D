//! O que o KANBAN diz (TrelloClone W2): a aba, as listas, os cartões, os menus e o arquivo.

/// A tradução de uma chave do Kanban, ou `None` se ela não é daqui.
pub(crate) fn tr(key: &str) -> Option<&'static str> {
    Some(match key {
        "kanban.tab.home" => "Kanbans",
        "kanban.cmd.new" => "New Kanban",
        "kanban.cmd.home" => "Kanbans\u{2026}",
        "kanban.default_name" => "Kanban {n}",
        "kanban.home.title" => "Your Kanbans",
        "kanban.home.create" => "Create new Kanban",
        "kanban.missing" => "This Kanban no longer exists.",
        "kanban.add_card" => "Add a card",
        "kanban.add_card_button" => "Add card",
        "kanban.add_list" => "Add another list",
        "kanban.add_list_button" => "Add list",
        "kanban.menu.list_title" => "List actions",
        "kanban.menu.board_title" => "Menu",
        "kanban.menu.add_card" => "Add card",
        "kanban.menu.collapse" => "Collapse list",
        "kanban.menu.archive_list" => "Archive this list",
        "kanban.menu.archive_cards" => "Archive all cards in this list",
        "kanban.menu.color" => "Change list color",
        "kanban.menu.no_color" => "Remove color",
        "kanban.menu.archived" => "Archived items",
        "kanban.menu.collapse_all" => "Collapse all lists",
        "kanban.menu.expand_all" => "Expand all lists",
        "kanban.archive.title" => "Archived items",
        "kanban.archive.cards" => "Cards",
        "kanban.archive.lists" => "Lists",
        "kanban.archive.restore" => "Restore",
        "kanban.archive.delete" => "Delete",
        "kanban.archive.empty" => "No archived items",
        _ => return None,
    })
}
