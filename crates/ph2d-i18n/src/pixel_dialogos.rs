//! **As palavras do PIXEL ART, P6** (ADR-0196) — os nomes do Aseprite.

/// A tradução de uma chave daqui, ou `None`.
pub(crate) fn tr(key: &str) -> Option<&'static str> {
    Some(match key {
        // Select ▸ Color Range.
        "pixel.edit_dialog.color_range.title" => "Color Range",
        "pixel.edit_dialog.color_range.note" => "Color: the foreground color",
        "pixel.edit_dialog.tolerance" => "Tolerance",
        // Select ▸ Modify.
        "pixel.edit_dialog.modify.expand" => "Expand Selection",
        "pixel.edit_dialog.modify.contract" => "Contract Selection",
        "pixel.edit_dialog.modify.border" => "Border Selection",
        "pixel.edit_dialog.modify.expand_by" => "Expand by",
        "pixel.edit_dialog.modify.contract_by" => "Contract by",
        "pixel.edit_dialog.modify.border_width" => "Width",
        "pixel.edit_dialog.brush" => "Brush",
        "pixel.edit_dialog.brush.square" => "Square",
        "pixel.edit_dialog.brush.circle" => "Circle",
        // Sprite ▸ Canvas Size.
        "pixel.edit_dialog.canvas.title" => "Canvas Size",
        "pixel.edit_dialog.width" => "Width",
        "pixel.edit_dialog.height" => "Height",
        "pixel.edit_dialog.left" => "Left",
        "pixel.edit_dialog.top" => "Top",
        "pixel.edit_dialog.right" => "Right",
        "pixel.edit_dialog.bottom" => "Bottom",
        "pixel.edit_dialog.anchor" => "Anchor",
        "pixel.edit_dialog.anchor.nw" => "NW",
        "pixel.edit_dialog.anchor.n" => "N",
        "pixel.edit_dialog.anchor.ne" => "NE",
        "pixel.edit_dialog.anchor.w" => "W",
        "pixel.edit_dialog.anchor.c" => "Center",
        "pixel.edit_dialog.anchor.e" => "E",
        "pixel.edit_dialog.anchor.sw" => "SW",
        "pixel.edit_dialog.anchor.s" => "S",
        "pixel.edit_dialog.anchor.se" => "SE",
        // Sprite ▸ Sprite Size.
        "pixel.edit_dialog.sprite.title" => "Sprite Size",
        "pixel.edit_dialog.lock_ratio" => "Lock Ratio",
        "pixel.edit_dialog.width_scale" => "Width scale",
        "pixel.edit_dialog.height_scale" => "Height scale",
        "pixel.edit_dialog.interpolation" => "Interpolation",
        "pixel.edit_dialog.method.nearest" => "Nearest-neighbor",
        "pixel.edit_dialog.method.bilinear" => "Bilinear",
        "pixel.edit_dialog.method.rotsprite" => "RotSprite",
        _ => return None,
    })
}
