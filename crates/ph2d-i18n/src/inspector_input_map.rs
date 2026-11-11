//! **As palavras do Inspector com a aba Input Map à frente** (onda 4, item 7b): a acção aberta em cartões.
//! Ficheiro próprio (um dono por ficheiro): as do teclado moram em `input_map_keyboard.rs`.

/// A tradução de uma chave `panel.inspector.input_map.*`, ou `None` se ela não é daqui.
pub(crate) fn tr(key: &str) -> Option<&'static str> {
    Some(match key {
        "panel.inspector.input_map.pick" => "Pick an action in the list.",
        "panel.inspector.input_map.action" => "Action",
        "panel.inspector.input_map.name" => "Name",
        "panel.inspector.input_map.group" => "Group",
        "panel.inspector.input_map.yours" => "Your actions",
        "panel.inspector.input_map.read_by" => "Read by",
        "panel.inspector.input_map.keys" => "Keys",
        "panel.inspector.input_map.no_keys" => "No keys yet.",
        "panel.inspector.input_map.stick" => "Stick",
        _ => return None,
    })
}
