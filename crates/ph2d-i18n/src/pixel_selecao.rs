//! **As palavras do PIXEL ART, P6** (ADR-0192): as ferramentas de selecção, a moldura solta e os
//! menus Select, Edit e Sprite do painel PIXEL — os nomes do Aseprite.

/// A tradução de uma chave daqui, ou `None`.
pub(crate) fn tr(key: &str) -> Option<&'static str> {
    Some(match key {
        "pixel.tool.rect_marquee" => "Rectangular Marquee",
        "pixel.tool.ellipse_marquee" => "Elliptical Marquee",
        "pixel.tool.lasso" => "Lasso",
        "pixel.tool.polygon_lasso" => "Polygonal Lasso",
        "pixel.tool.magic_wand" => "Magic Wand",
        "pixel.tool.transform" => "Transform",
        // O nome de uma forma é o do Vector; este só vale sem a ponte dele.
        "pixel.tool.shape" => "Shape",
        "pixel.hint.shape" => {
            "{tool}: drag the box · Shift keeps it square · Fill and its options are in the Pixel panel"
        }
        "pixel.hint.live_shape" => {
            "{tool}: drag a handle or the box to adjust \u{00b7} Shift: 15\u{00b0} steps \u{00b7} Alt: one corner \u{00b7} Double-click: reset \u{00b7} Ctrl+Z: undo the last adjustment \u{00b7} Enter: make it pixels \u{00b7} Esc: drop it \u{00b7} Click outside: draw another"
        }
        // O comprimento na etiqueta do arrasto da forma viva: um píxel do Vector é uma célula.
        "pixel.shape.length" => "{n} px",
        "panel.pixel.shape_options" => "Shape",
        "panel.pixel.auto_select_layer" => "Auto Select Layer",
        "panel.pixel.shape_fill" => "Fill",
        "pixel.hint.transform" => {
            "{tool}: drag a corner or side to scale · drag just outside a corner to rotate · drag inside to move · Enter applies, Esc cancels"
        }
        "pixel.hint.marquee" => {
            "{tool}: drag to select · a click deselects · Shift adds, Alt+Shift subtracts, Ctrl+Shift intersects"
        }
        "pixel.hint.lasso" => {
            "{tool}: draw around the area to select · Shift adds, Alt+Shift subtracts"
        }
        "pixel.hint.polygon_lasso" => {
            "{tool}: click each corner · click the last corner again or press Enter to close"
        }
        "pixel.hint.magic_wand" => {
            "{tool}: click a color to select it · Tolerance in the Pixel panel"
        }
        "panel.pixel.select.mode" => "Selection mode",
        "panel.pixel.select.replace" => "Replace",
        "panel.pixel.select.add" => "Add",
        "panel.pixel.select.subtract" => "Subtract",
        "panel.pixel.select.intersect" => "Intersect",
        "panel.pixel.select.mods_hint" => {
            "Shift adds · Alt+Shift subtracts · Ctrl+Shift intersects · Ctrl drags the pixels"
        }
        "panel.pixel.wand.tolerance" => "Tolerance",
        "panel.pixel.resample" => "Rotation",
        "panel.pixel.resample.fast" => "Fast",
        "panel.pixel.resample.rotsprite" => "RotSprite",
        "panel.pixel.transform" => "Transform",
        "panel.pixel.transform_hint" => {
            "Drag the handles to scale (Shift keeps the ratio, Ctrl from the center), just outside a corner to rotate, Skew in the field · Enter applies"
        }
        "panel.pixel.xform.x" => "X",
        "panel.pixel.xform.y" => "Y",
        "panel.pixel.xform.w" => "Width",
        "panel.pixel.xform.h" => "Height",
        "panel.pixel.xform.angle" => "Angle",
        "panel.pixel.xform.skew" => "Skew",
        "panel.pixel.select" => "Select",
        "panel.pixel.select.all" => "All",
        "panel.pixel.select.deselect" => "Deselect",
        "panel.pixel.select.reselect" => "Reselect",
        "panel.pixel.select.inverse" => "Inverse",
        "panel.pixel.select.color_range" => "Color Range…",
        "panel.pixel.select.expand" => "Expand…",
        "panel.pixel.select.contract" => "Contract…",
        "panel.pixel.select.border" => "Border…",
        "panel.pixel.edit" => "Edit",
        "panel.pixel.edit.copy" => "Copy",
        "panel.pixel.edit.cut" => "Cut",
        "panel.pixel.edit.copy_merged" => "Copy Merged",
        "panel.pixel.edit.paste" => "Paste",
        "panel.pixel.edit.clear" => "Delete",
        "panel.pixel.edit.transform" => "Transform",
        "panel.pixel.edit.flip_h" => "Flip H",
        "panel.pixel.edit.flip_v" => "Flip V",
        "panel.pixel.edit.rotate180" => "Rotate 180",
        "panel.pixel.edit.rotate_cw" => "Rotate 90 CW",
        "panel.pixel.edit.rotate_ccw" => "Rotate 90 CCW",
        "panel.pixel.edit.shift_left" => "Shift Left",
        "panel.pixel.edit.shift_right" => "Shift Right",
        "panel.pixel.edit.shift_up" => "Shift Up",
        "panel.pixel.edit.shift_down" => "Shift Down",
        "panel.pixel.edit.apply" => "Apply",
        "panel.pixel.edit.cancel" => "Cancel",
        "panel.pixel.sprite.canvas_size" => "Canvas Size…",
        "panel.pixel.sprite.sprite_size" => "Sprite Size…",
        "panel.pixel.sprite.rotate180" => "Canvas 180",
        "panel.pixel.sprite.rotate_cw" => "Canvas 90 CW",
        "panel.pixel.sprite.rotate_ccw" => "Canvas 90 CCW",
        "panel.pixel.sprite.flip_h" => "Flip Canvas H",
        "panel.pixel.sprite.flip_v" => "Flip Canvas V",
        "panel.pixel.sprite.crop" => "Crop",
        "panel.pixel.sprite.trim" => "Trim",
        "pixel.select.refuse.none" => "This needs a selection",
        "pixel.select.refuse.nothing" => "Nothing to transform here",
        "pixel.edit.refuse.clipboard" => "Nothing copied yet",
        "pixel.sprite.refuse.trim" => "Nothing to trim",
        _ => return None,
    })
}
