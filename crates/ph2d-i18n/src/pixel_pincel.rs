//! **As palavras do PIXEL ART, P7** (ADR-0196): o pincel de padrão, os pincéis guardados e o
//! popup Dynamics do painel PIXEL — em inglês simples (o app é só em inglês, dono 31/10).

/// P13 U-Pintura: o ink Dither, o Push e a métrica do Color Mode (a família do pincel; aqui para
/// não crescer a `lib.rs`, no tecto).
#[path = "pixel_pintura.rs"]
mod pintura;

/// A tradução de uma chave daqui, ou `None`.
pub(crate) fn tr(key: &str) -> Option<&'static str> {
    Some(match key {
        "panel.pixel.brush.pattern_size" => "Image brush, {w} × {h}",
        "panel.pixel.pattern" => "Pattern",
        "panel.pixel.pattern.source" => "Fixed",
        "panel.pixel.pattern.source_tip" => "The pattern stays lined up with where you copied it",
        "panel.pixel.pattern.destination" => "Per stroke",
        "panel.pixel.pattern.destination_tip" => {
            "The pattern starts again where each stroke starts"
        }
        "panel.pixel.pattern.paint" => "Follows",
        "panel.pixel.pattern.paint_tip" => "The whole image moves with the brush",
        "panel.pixel.brush.flip_h" => "Flip H",
        "panel.pixel.brush.flip_v" => "Flip V",
        "panel.pixel.brush.rotate_left" => "Rotate Left",
        "panel.pixel.brush.rotate_right" => "Rotate Right",
        "panel.pixel.brush.saved_title" => "Saved Brushes",
        "panel.pixel.brush.slot" => "Alt+{n}",
        "panel.pixel.brush.slot_far" => "Brush {n}",
        "panel.pixel.brush.slot_shape" => "{shape}, {size} px",
        "panel.pixel.brush.slot_tip" => "Use this brush again",
        "panel.pixel.brush.locked_tip" => {
            "Locked: changes to the brush are not saved here (click to unlock)"
        }
        "panel.pixel.brush.unlocked_tip" => {
            "Unlocked: changes to the brush are saved here (click to lock)"
        }
        "panel.pixel.brush.delete_tip" => "Delete this saved brush",
        "panel.pixel.brush.save_here" => "Save Brush Here",
        "panel.pixel.brush.save_here_tip" => {
            "Keep the brush you hold in a new slot (Ctrl+B turns a selection into a brush and keeps it too)"
        }
        "panel.pixel.edit.new_brush" => "New Brush",
        "panel.pixel.dynamics" => "Dynamics",
        "panel.pixel.dynamics.tip" => "Change the brush with pen pressure or drawing speed",
        "panel.pixel.dynamics.stabilizer" => "Stabilizer",
        "panel.pixel.dynamics.stabilizer_len" => "Distance",
        "panel.pixel.dynamics.off" => "Off",
        "panel.pixel.dynamics.pressure" => "Pressure",
        "panel.pixel.dynamics.velocity" => "Speed",
        "panel.pixel.dynamics.size" => "Size from",
        "panel.pixel.dynamics.angle" => "Angle from",
        "panel.pixel.dynamics.gradient" => "Color from",
        "panel.pixel.dynamics.min_size" => "Min size",
        "panel.pixel.dynamics.min_angle" => "Min angle",
        "panel.pixel.dynamics.max_hint" => {
            "Light or slow gives the Min value; hard or fast gives the brush's own Size and Angle. Color goes from the background color to the front color"
        }
        "panel.pixel.dynamics.dither" => "Blend",
        "panel.pixel.dynamics.dither_none" => "Smooth",
        "panel.pixel.dynamics.dither_bayer2" => "2×2",
        "panel.pixel.dynamics.dither_bayer4" => "4×4",
        "panel.pixel.dynamics.dither_bayer8" => "8×8",
        "panel.pixel.dynamics.range_pressure" => "Pressure range",
        "panel.pixel.dynamics.range_velocity" => "Speed range",
        "panel.pixel.dynamics.range_hint" => {
            "Below the left handle counts as nothing, past the right handle as full"
        }
        _ => return pintura::tr(key),
    })
}
