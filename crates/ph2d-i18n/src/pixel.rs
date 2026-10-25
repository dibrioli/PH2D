//! **As palavras do PIXEL ART** (ADR-0192) — o menu Add, as ferramentas e os avisos da família
//! `ph2d-app-pixel`. Os nomes das ferramentas e os atalhos são os do Aseprite.

/// A tradução de uma chave `object_add.pixel_sprite` / `pixel.*`, ou `None` se ela não é daqui.
pub(crate) fn tr(key: &str) -> Option<&'static str> {
    Some(match key {
        "object_add.pixel_sprite" => "Pixel Sprite",
        "pixel.tool.pencil" => "Pencil",
        "pixel.tool.eraser" => "Eraser",
        "pixel.tool.eyedropper" => "Eyedropper",
        "pixel.tool.hand" => "Hand",
        "pixel.tool_switched" => "{tool} ({key})",
        "pixel.zoom" => "Zoom {n}00 %",
        "pixel.grid_on" => "Pixel grid on",
        "pixel.grid_off" => "Pixel grid off",
        "pixel.nothing_to_undo" => "Nothing to undo in this sprite",
        "pixel.nothing_to_redo" => "Nothing to redo in this sprite",
        "panel.pixel.title" => "Pixel",
        "panel.pixel.empty" => {
            "Select a Pixel Sprite and press Tab (Edit) to draw with these tools and colors."
        }
        "panel.pixel.swap" => "Swap (X)",
        "panel.pixel.swap_tip" => "Swap the foreground and background colors (X)",
        "panel.pixel.palette" => "Palette",
        "pixel.new_sprite.title" => "New Sprite",
        "pixel.new_sprite.width" => "Width",
        "pixel.new_sprite.height" => "Height",
        "pixel.new_sprite.color_mode" => "Color Mode",
        "pixel.new_sprite.mode.rgba" => "RGBA",
        "pixel.new_sprite.mode.gray" => "Grayscale",
        "pixel.new_sprite.mode.indexed" => "Indexed",
        "pixel.new_sprite.background" => "Background",
        "pixel.new_sprite.bg.transparent" => "Transparent",
        "pixel.new_sprite.bg.white" => "White",
        "pixel.new_sprite.bg.black" => "Black",
        "pixel.new_sprite.ok" => "OK",
        "pixel.new_sprite.cancel" => "Cancel",
        "pixel.load_failed" => "The pixel sprites of this project could not be read: {e}",
        _ => return None,
    })
}
