//! **As palavras das rampas e do ciclo de cor do PIXEL ART, P13 U-Paleta** — em Plain English.

/// A tradução de uma chave daqui, ou `None`.
pub(crate) fn tr(key: &str) -> Option<&'static str> {
    Some(match key {
        "pixel.palette.new_ramp" => "New Ramp…",
        "pixel.ramp.make" => "Make Ramp",
        "pixel.ramp.remove" => "Remove Ramp",
        "pixel.ramp.create" => "Create",
        "pixel.ramp.base" => "Base: the primary color",
        "pixel.ramp.steps" => "Steps",
        "pixel.ramp.hue_shift" => "Hue shift",
        "pixel.ramp.chroma" => "Chroma",
        "pixel.ramp.light_min" => "Darkest",
        "pixel.ramp.light_max" => "Lightest",
        "pixel.cycle" => "Cycle",
        "pixel.cycle.period" => "Period (ms)",
        "pixel.cycle.direction" => "Direction",
        "pixel.cycle.direction.forward" => "Forward",
        "pixel.cycle.direction.reverse" => "Reverse",
        "pixel.cycle.direction.ping_pong" => "Ping-pong",
        "pixel.cycle.tip" => {
            "Rotate the colors of this ramp over time: the pixels keep their palette index, \
             and the index shows the next color of the ramp."
        }
        "pixel.cycle.rgb_off.tip" => {
            "Color cycling works only in Indexed mode: in RGB a pixel stores its color, not a \
             palette index."
        }
        _ => return None,
    })
}
