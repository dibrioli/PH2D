//! **As palavras da luz na rampa** (P13 L): a linha *Light* do cartão Pixel Sprite.

/// A tradução de uma chave daqui, ou `None`.
pub(crate) fn tr(key: &str) -> Option<&'static str> {
    Some(match key {
        "card.insp.pixel_light" => "Light",
        "card.insp.pixel_light.smooth" => "Smooth",
        "card.insp.pixel_light.palette" => "Palette",
        "card.insp.pixel_light_tip" => {
            "Palette keeps a lit sprite inside its color ramps: light picks a brighter or darker step instead of tinting the color."
        }
        "pixel.history.light" => "Light",
        _ => return None,
    })
}
