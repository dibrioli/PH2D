//! O que a porta do gesto do Kanban diz (TrelloClone J3, `ph2d-kanban-hooks`).

/// A tradução de uma chave da porta, ou `None` se ela não é daqui.
pub(crate) fn tr(key: &str) -> Option<&'static str> {
    Some(match key.strip_prefix("kanban.hooks.")? {
        "stopped" => {
            "Automations stopped: rules and scripts kept triggering each other. Check them in the Automation panel."
        }
        "to_note" => "Convert to sticky note",
        _ => return None,
    })
}
