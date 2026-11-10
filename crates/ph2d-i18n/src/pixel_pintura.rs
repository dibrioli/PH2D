//! **O PIXEL ART, P13 U-Pintura**: o ink Dither, o Smudge ▸ Push, o Shading na rampa e a métrica do
//! Color Mode ▸ Indexed.

/// A tradução de uma chave desta fatia, ou `None`.
pub(crate) fn tr(key: &str) -> Option<&'static str> {
    Some(match key {
        "pixel.ink.dither" => "Dither",
        "pixel.ink.dither.level" => "Level",
        "pixel.ink.dither.matrix" => "Matrix",
        "pixel.ink.dither.matrix.bayer2" => "Bayer 2×2",
        "pixel.ink.dither.matrix.bayer4" => "Bayer 4×4",
        "pixel.ink.dither.matrix.bayer8" => "Bayer 8×8",
        "pixel.ink.dither.tooltip" => {
            "Paints a fixed pattern between the foreground and background colors; the pattern stays put on the canvas. Level 64 is all foreground, 0 all background."
        }
        "pixel.smudge.push" => "Push",
        "pixel.smudge.push.strength" => "Strength",
        "pixel.smudge.push.tooltip" => "Drags the color under the brush along the stroke",
        "pixel.history.push" => "Push Tool",
        "pixel.hint.push" => "{tool}: paint to push the color under the brush along the stroke",
        "pixel.ink.shading.tooltip" => {
            "Steps one shade along the palette ramp under the brush (left button one step forward, right button one step back); outside any ramp, along the selected colors."
        }
        "pixel.convert.metric" => "Color matching",
        "pixel.convert.metric.rgb" => "Exact RGB",
        "pixel.convert.metric.perceptual" => "Perceptual (OKLab)",
        "pixel.convert.metric.keep_ramps" => "Keep ramps",
        "pixel.convert.metric.keep_ramps.tip" => {
            "Each color goes to the ramp with the closest hue, and to the step with the closest lightness, so light and dark never swap inside a ramp."
        }
        _ => return None,
    })
}
