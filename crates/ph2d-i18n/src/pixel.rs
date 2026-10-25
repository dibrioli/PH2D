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
        "pixel.tool.spray" => "Spray",
        "pixel.tool.bucket" => "Paint Bucket",
        "pixel.tool.line" => "Line",
        "pixel.tool.curve" => "Curve",
        "pixel.tool.rectangle" => "Rectangle",
        "pixel.tool.filled_rectangle" => "Filled Rectangle",
        "pixel.tool.ellipse" => "Ellipse",
        "pixel.tool.filled_ellipse" => "Filled Ellipse",
        "pixel.tool.contour" => "Contour",
        "pixel.tool.polygon" => "Polygon",
        "pixel.tool_switched" => "{tool} ({key})",
        "pixel.shortcut_shift" => "Shift+{key}",
        "pixel.zoom" => "Zoom {n}00 %",
        "pixel.grid_on" => "Pixel grid on",
        "pixel.grid_off" => "Pixel grid off",
        "pixel.nothing_to_undo" => "Nothing to undo in this sprite",
        "pixel.nothing_to_redo" => "Nothing to redo in this sprite",
        "panel.pixel_tools.title" => "Pixel",
        "panel.pixel.empty" => {
            "Select a Pixel Sprite and press Tab (Draw) to draw with these colors."
        }
        "panel.pixel.fg" => "Foreground color",
        "panel.pixel.bg" => "Background color",
        "pixel.hint.pencil" => "{tool}: drag to draw · right button draws the background color",
        "pixel.hint.eraser" => {
            "{tool}: drag to erase · right button turns the foreground color into the background"
        }
        "pixel.hint.eyedropper" => {
            "{tool}: click picks the foreground · right button the background"
        }
        "pixel.hint.hand" => "{tool}: drag to move the view · hold Space with any tool",
        "pixel.hint.spray" => "{tool}: hold to spray · Width and Speed in the Pixel panel",
        "pixel.hint.bucket" => {
            "{tool}: click fills the area · Tolerance, Contiguous and 4/8 in the Pixel panel"
        }
        "pixel.hint.line" => "{tool}: drag a line · Shift snaps to pixel-art angles",
        "pixel.hint.curve" => {
            "{tool}: drag the ends, then move to bend and click to finish · Esc cancels"
        }
        "pixel.hint.rectangle" => {
            "{tool}: drag a box · Shift = square · Ctrl = from the center · Space moves it"
        }
        "pixel.hint.ellipse" => {
            "{tool}: drag a box · Shift = circle · Ctrl = from the center · Space moves it"
        }
        "pixel.hint.contour" => "{tool}: draw around an area; releasing closes and fills it",
        "pixel.hint.polygon" => {
            "{tool}: click each corner · click the first corner or press Enter to fill · Esc cancels"
        }
        "pixel.hint.axis" => "Drag the symmetry axis · it snaps to whole and half pixels",
        "panel.pixel.brush" => "Brush",
        "panel.pixel.shape" => "Shape",
        "panel.pixel.shape.circle" => "Circle",
        "panel.pixel.shape.square" => "Square",
        "panel.pixel.shape.line" => "Line",
        "panel.pixel.size" => "Size",
        "panel.pixel.angle" => "Angle",
        "panel.pixel.pixel_perfect" => "Pixel-perfect",
        "panel.pixel.corner" => "Corner radius",
        "panel.pixel.spray_width" => "Spray width",
        "panel.pixel.spray_speed" => "Spray speed",
        "panel.pixel.bucket" => "Paint Bucket",
        "panel.pixel.tolerance" => "Tolerance",
        "panel.pixel.contiguous" => "Contiguous",
        "panel.pixel.connectivity" => "Connectivity",
        "panel.pixel.connect4" => "4",
        "panel.pixel.connect8" => "8",
        "panel.pixel.sample" => "Sample",
        "panel.pixel.sample.layer" => "Active layer",
        "panel.pixel.sample.visible" => "All visible",
        "panel.pixel.symmetry" => "Symmetry",
        "panel.pixel.sym_h" => "Horizontal",
        "panel.pixel.sym_v" => "Vertical",
        "panel.pixel.sym_d1" => "Diagonal \\",
        "panel.pixel.sym_d2" => "Diagonal /",
        "panel.pixel.tiled" => "Tiled Mode",
        "panel.pixel.tile_x" => "Tile X",
        "panel.pixel.tile_y" => "Tile Y",
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
