//! ⭐ **A dica da linha ESMAECIDA do painel Painter** (`panel.painter_layers.inerte.*`): o que liga o controlo.
//! Tabela IRMÃ da [`super`], partida dela por secção (tecto de tamanho; `CLAUDE.md` §0.2).

/// A tradução de uma chave `panel.painter_layers.inerte.*`, ou `None` se ela não é daqui.
pub(crate) fn tr(key: &str) -> Option<&'static str> {
    Some(match key {
        // ── A dica da linha ESMAECIDA: o que liga o controlo (doc 46 §2-3; a lei de cada uma é a
        // `ph2d_tool_painter::Dependente`, e o painel a mostra ao passar o rato) ───────────────────
        "panel.painter_layers.inerte.taper_length" => "Acts when the Taper has a length",
        "panel.painter_layers.inerte.strength_below_full" => "Acts when Strength is below 100%",
        "panel.painter_layers.inerte.accumulate_on" => "Acts when Accumulate is on",
        "panel.painter_layers.inerte.jitter_above_zero" => "Acts when Jitter is above 0",
        "panel.painter_layers.inerte.randomize_above_zero" => {
            "Acts when Hue, Saturation or Value is above 0"
        }
        "panel.painter_layers.inerte.dash_ratio_below_full" => "Acts when Dash Ratio is below 100%",
        "panel.painter_layers.inerte.paper_and_relief" => {
            "Acts when a Paper is chosen and Relief is above 0"
        }
        "panel.painter_layers.inerte.relief_above_zero" => "Acts when Relief is above 0",
        "panel.painter_layers.inerte.relief_and_no_grain" => {
            "Acts when Relief is above 0 and no Grain is chosen"
        }
        "panel.painter_layers.inerte.charge_below_full" => "Acts when Charge is below 100%",
        "panel.painter_layers.inerte.grain_chosen" => "Acts when a Grain is chosen",
        "panel.painter_layers.inerte.pressure_opacity_on" => {
            "Acts when Pressure opacity is above 0"
        }
        "panel.painter_layers.inerte.pressure_size_on" => "Acts when Pressure size is above 0",
        "panel.painter_layers.inerte.pressure_on" => {
            "Acts when something reads the pressure: Pressure size, opacity, softness or scatter, a Grain depth or tip images by pressure"
        }
        "panel.painter_layers.inerte.shape_automatic_off" => {
            "Acts when the Shape's Automatic is off"
        }
        "panel.painter_layers.inerte.spray_count_above_one" => "Acts when Count is above 1",
        "panel.painter_layers.inerte.accumulate_off" => "Acts when Accumulate is off",
        "panel.painter_layers.inerte.hardness_above_zero" => {
            "Acts when Hardness is above 0: it only softens the brush"
        }
        "panel.painter_layers.inerte.speed_on" => {
            "Acts when Speed size, Speed opacity or Speed softness is above 0"
        }
        "panel.painter_layers.inerte.wash_ignores_dab_opacity" => {
            "The wash does not read the dab opacity"
        }
        "panel.painter_layers.inerte.color_owned_by_ramp" => {
            "Acts when no Color Ramp or Texture Color paints the colour"
        }
        // O fim do traço (L5), o traço como uma camada (L8) e o modo sobre o vazio (L7).
        "panel.painter_layers.inerte.taper_end_length" => {
            "Acts with a Tail length on a freehand stroke"
        }
        "panel.painter_layers.inerte.fall_off_length" => "Acts with a Fall-off length",
        "panel.painter_layers.inerte.color_blend_mode" => {
            "Acts with a colour Blend (Multiply, Color, Hue…)"
        }
        "panel.painter_layers.inerte.freehand_paint_stroke" => "Acts on a freehand paint stroke",
        // A profundidade do grão por dab (L9).
        "panel.painter_layers.inerte.depth_jitter_above_zero" => {
            "Acts when Depth jitter is above 0"
        }
        "panel.painter_layers.inerte.tilt_law_on" => {
            "Acts when the pen tilt changes the dab: Tilt width, length, smooth or opacity, or Rotation: Pen tilt"
        }
        "panel.painter_layers.inerte.follow_nothing_to_turn" => {
            "Acts with a Shape texture, a flattened tip or a Grain that follows the dab"
        }
        "panel.painter_layers.inerte.method_lays_a_path" => {
            "Acts on a path with Spacing above 2.5%, with the Paint tool, the Eraser or the Mask"
        }
        "panel.painter_layers.inerte.bristles_no_grain" => {
            "Acts with Grain None and the Paint tool or the Eraser"
        }
        "panel.painter_layers.inerte.texture_color_needs_a_shape" => {
            "Acts with a Shape texture: a pattern or an image"
        }
        "panel.painter_layers.inerte.texture_colors_media" => {
            "Acts in Digital and Impasto: Wet Paint and Watercolor paint the brush colour"
        }
        "panel.painter_layers.inerte.eraser_on" => "Acts with the Eraser",
        "panel.painter_layers.inerte.wet_paint_tool" => "Acts with the Paint tool and the Eraser",
        "panel.painter_layers.inerte.paint_tool_only" => "Acts with the Paint tool",
        "panel.painter_layers.inerte.tilt_on" => "Acts when Tilt is on",
        "panel.painter_layers.inerte.paper_chosen" => "Acts when a Paper is chosen",
        "panel.painter_layers.inerte.detail_above_minimum" => {
            "Acts when Detail is above its minimum"
        }
        _ => return None,
    })
}
