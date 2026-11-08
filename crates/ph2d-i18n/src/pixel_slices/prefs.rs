//! **As palavras da janela GRID & BACKGROUND do PIXEL ART, P12 P** (Aseprite O18/O19) e do popup
//! do onion skin (O21).

/// A tradução de uma chave daqui, ou `None`.
pub(crate) fn tr(key: &str) -> Option<&'static str> {
    Some(match key {
        "panel.pixel_prefs.title" => "Grid & Background",
        "panel.pixel.edit.grid_settings" => "Grid & Background…",
        "pixel.keys.view.grid_settings" => "Grid & Background…",
        "panel.pixel_prefs.sec_scope" => "Settings for",
        "panel.pixel_prefs.scope_doc" => "Active Document",
        "panel.pixel_prefs.scope_new" => "New Documents",
        "panel.pixel_prefs.no_document" => "Open a Pixel Sprite in Draw to edit its settings.",
        "panel.pixel_prefs.set_defaults" => "Set as Defaults",
        "panel.pixel_prefs.reset" => "Reset",
        "panel.pixel_prefs.sec_grid" => "Grid",
        "panel.pixel_prefs.sec_pixel_grid" => "Pixel Grid",
        "panel.pixel_prefs.sec_checker" => "Checkerboard",
        "panel.pixel_prefs.visible" => "Visible",
        "panel.pixel_prefs.origin" => "Origin",
        "panel.pixel_prefs.size" => "Size",
        "panel.pixel_prefs.x" => "X",
        "panel.pixel_prefs.y" => "Y",
        "panel.pixel_prefs.w" => "W",
        "panel.pixel_prefs.h" => "H",
        "panel.pixel_prefs.color" => "Color",
        "panel.pixel_prefs.color_a" => "Color 1",
        "panel.pixel_prefs.color_b" => "Color 2",
        "panel.pixel_prefs.opacity" => "Opacity",
        "panel.pixel_prefs.opacity_glyph" => "O",
        "panel.pixel_prefs.auto" => "Auto opacity (fades when lines get close)",
        "panel.pixel_prefs.apply_zoom" => "Apply Zoom (size in sprite pixels)",
        _ => return None,
    })
}
