//! **As palavras do PIXEL ART** (ADR-0192) — o menu Add, as ferramentas e os avisos da família
//! `ph2d-app-pixel`. Os nomes das ferramentas e os atalhos são os do Aseprite.

/// A tradução de uma chave `object_add.pixel_sprite` / `pixel.*`, ou `None` se ela não é daqui.
pub(crate) fn tr(key: &str) -> Option<&'static str> {
    Some(match key {
        "object_add.pixel_sprite" => "Pixel Sprite",
        "pixel.tool.pencil" => "Pencil",
        "pixel.tool.eraser" => "Eraser",
        "pixel.tool.eyedropper" => "Eyedropper",
        "pixel.tool_switched" => "{tool} ({key})",
        "pixel.zoom" => "Zoom {n}00 %",
        "pixel.grid_on" => "Pixel grid on",
        "pixel.grid_off" => "Pixel grid off",
        "pixel.nothing_to_undo" => "Nothing to undo in this sprite",
        "pixel.nothing_to_redo" => "Nothing to redo in this sprite",
        "pixel.load_failed" => "The pixel sprites of this project could not be read: {e}",
        _ => return None,
    })
}
