//! **As palavras da janela SLICES do PIXEL ART, P11b** — os nomes do Aseprite, em Plain English.

/// As da janela UNDO HISTORY (P12 H), outra janela flutuante do Pixel.
mod history;
mod prefs;
/// As do cartão USER DATA (P12 U).
mod user_data;

/// A tradução de uma chave daqui, ou `None`.
pub(crate) fn tr(key: &str) -> Option<&'static str> {
    Some(match key {
        "panel.pixel_slices.title" => "Slices",
        "panel.pixel.slices.sec_list" => "Slices",
        "panel.pixel.slices.sec_props" => "Slice Properties",
        "panel.pixel.slices.all" => "All",
        "panel.pixel.slices.none" => "None",
        "panel.pixel.slices.empty" => "No slices yet. Drag on the canvas to make one.",
        "panel.pixel.slices.empty_tool" => {
            "No slices yet. Pick the Slice tool, then drag on the canvas."
        }
        "panel.pixel.slices.transform" => "Transform",
        "panel.pixel.slices.transform_tip" => "Transform pixels along slice modification",
        "panel.pixel.slices.name" => "Name",
        "panel.pixel.slices.color" => "Color",
        "panel.pixel.slices.user_data" => "User Data",
        "panel.pixel.slices.bounds" => "Bounds (px)",
        "panel.pixel.slices.x" => "X",
        "panel.pixel.slices.y" => "Y",
        "panel.pixel.slices.w" => "W",
        "panel.pixel.slices.h" => "H",
        "panel.pixel.slices.nine" => "9-Slices",
        "panel.pixel.slices.pivot" => "Pivot",
        "panel.pixel.slices.delete" => "Delete",
        "panel.pixel.slices.duplicate" => "Duplicate",
        "panel.pixel.slices.no_selection" => "Select a slice to edit it.",
        "panel.pixel.slices.no_value" => "–",
        "panel.pixel.slices.menu_properties" => "Slice Properties...",
        "panel.pixel.slices.menu_duplicate" => "Duplicate Slice",
        "panel.pixel.slices.menu_delete" => "Delete Slice",
        "panel.pixel.slices.menu_remove_key" => "Remove Change on This Frame",
        _ => {
            return history::tr(key)
                .or_else(|| user_data::tr(key))
                .or_else(|| prefs::tr(key));
        }
    })
}
