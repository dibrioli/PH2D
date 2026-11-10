//! **As palavras do Tween… do Pixel** (P13 T): os quadros do meio entre dois cels.

/// A tradução de uma chave daqui, ou `None`.
pub(crate) fn tr(key: &str) -> Option<&'static str> {
    Some(match key {
        "panel.pixel.timeline.tween" => {
            "Tween: make the in-between frames from the first selected cel to the last one"
        }
        "pixel.tween.title" => "Tween",
        "pixel.tween.frames" => "Frames",
        "pixel.tween.rotate" => "Rotate (degrees)",
        "pixel.tween.ease" => "Ease",
        "pixel.tween.ease.linear" => "Linear",
        "pixel.tween.ease.in" => "Ease In",
        "pixel.tween.ease.out" => "Ease Out",
        "pixel.tween.ease.in_out" => "Ease In-Out",
        _ => return None,
    })
}
