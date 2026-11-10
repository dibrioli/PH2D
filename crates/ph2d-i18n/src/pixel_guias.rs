//! **As palavras das guias do PIXEL ART, P13 U-Guias** — View ▸ Guides, em Plain English.

/// A tradução de uma chave daqui, ou `None`.
pub(crate) fn tr(key: &str) -> Option<&'static str> {
    Some(match key {
        "pixel.guides" => "Guides",
        "pixel.guides.none" => "No guides",
        "pixel.guides.perspective_1" => "Perspective (1 point)",
        "pixel.guides.perspective_2" => "Perspective (2 points)",
        "pixel.guides.perspective_3" => "Perspective (3 points)",
        "pixel.guides.iso" => "Isometric 2:1",
        "pixel.guides.tip" => {
            "Hold Shift while drawing a line to follow the nearest guide: toward a vanishing \
             point, or along a 2:1 isometric axis. Drag the round handles to move the guides."
        }
        "pixel.history.guides" => "Move Guide",
        _ => return None,
    })
}
