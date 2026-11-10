//! **As palavras dos efeitos de camada do Pixel** (P13 U-Camada): Layer Properties ▸ Effects ▸ Outline.

/// A tradução de uma chave daqui, ou `None`.
pub(crate) fn tr(key: &str) -> Option<&'static str> {
    Some(match key {
        "pixel.layer.effects" => "Effects",
        "pixel.layer.effects.outline" => "Outline",
        "pixel.layer.effects.outline_tip" => {
            "Draws an outline around this layer without changing its pixels. Turn it off any time."
        }
        "pixel.layer.effects.inside" => "Inside",
        "pixel.layer.effects.square" => "8 neighbors",
        "pixel.layer.effects.primary" => "Use Primary Color",
        "pixel.layer.effects.apply" => "Apply",
        "pixel.layer.effects.apply.tooltip" => {
            "Bakes the effects into every frame of this layer and removes them"
        }
        "pixel.layer.effects.remove" => "Remove",
        "panel.pixel.layer.effects_tip" => "This layer has effects. Click to see them",
        "pixel.history.layer_effects" => "Layer Effects",
        "pixel.history.layer_effects_apply" => "Apply Effects",
        _ => return None,
    })
}
