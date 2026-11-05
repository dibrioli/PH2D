//! A Widget Gallery: as peças do cromo (barra de ferramentas e lista flutuante). Irmão do `galeria.rs`.

/// A tradução de uma chave daqui, ou `None`.
pub(crate) fn tr(key: &str) -> Option<&'static str> {
    Some(match key {
        "panel.widget_gallery.card.tool_bar" => "Tool Bar",
        "panel.widget_gallery.card.floating_list" => "Floating List",
        "panel.widget_gallery.cromo.tip.move" => "Move: drag the object",
        "panel.widget_gallery.cromo.tip.rotate" => "Rotate: turn the object",
        "panel.widget_gallery.cromo.tip.transform" => "Transform: move, turn and resize",
        "panel.widget_gallery.cromo.copy" => "Copy",
        "panel.widget_gallery.cromo.copy_key" => "Ctrl+C",
        "panel.widget_gallery.cromo.tip.copy" => "Copy the selection",
        "panel.widget_gallery.cromo.duplicate" => "Duplicate",
        "panel.widget_gallery.cromo.duplicate_key" => "Ctrl+D",
        "panel.widget_gallery.cromo.tip.duplicate" => "Make a second one",
        "panel.widget_gallery.cromo.delete" => "Delete",
        "panel.widget_gallery.cromo.delete_key" => "Del",
        "panel.widget_gallery.cromo.tip.delete" => "Remove the selection",
        _ => return None,
    })
}
