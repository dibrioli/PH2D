//! ⭐ **As palavras da PONTA** (as lacunas L9 · L18 · L19 dos pincéis, `docs/Painter/Pinceis/03_MOTORES.md` §4): o
//! modo do grão e a profundidade por dab, a ponta de píxel, as imagens da ponta. Tabela IRMÃ da
//! [`super::painter_layers`] por isolamento (`CLAUDE.md` §0.2): outras linhas mexem naquela ao mesmo tempo.

/// A tradução de uma chave `panel.painter_ponta.*`, ou `None` se ela não é daqui.
pub(crate) fn tr(key: &str) -> Option<&'static str> {
    Some(match key {
        "panel.painter_ponta.pixel" => "Pixel tip",
        "panel.painter_ponta.pixel.off" => "Off",
        "panel.painter_ponta.pixel.aligned" => "Pixel",
        "panel.painter_ponta.pixel.perfect" => "Pixel-perfect",
        "panel.painter_ponta.grain_mode" => "Grain mode",
        "panel.painter_ponta.grain_mode.multiply" => "Multiply",
        "panel.painter_ponta.grain_mode.subtract" => "Subtract",
        "panel.painter_ponta.grain_mode.darken" => "Darken",
        "panel.painter_ponta.grain_mode.color_burn" => "Color Burn",
        "panel.painter_ponta.grain_mode.height" => "Height",
        "panel.painter_ponta.grain_mode.linear_height" => "Linear Height",
        "panel.painter_ponta.depth_min" => "Minimum depth",
        "panel.painter_ponta.depth_jitter" => "Depth jitter",
        "panel.painter_ponta.depth_control" => "Depth control",
        "panel.painter_ponta.depth_control.random" => "Random",
        "panel.painter_ponta.depth_control.pressure" => "Pen pressure",
        "panel.painter_ponta.hose_add" => "Add image",
        "panel.painter_ponta.hose_clear" => "Clear images",
        "panel.painter_ponta.hose_mode" => "Image order",
        "panel.painter_ponta.hose_mode.random" => "Random (weight)",
        "panel.painter_ponta.hose_mode.sequence" => "Sequence",
        "panel.painter_ponta.hose_mode.pressure" => "Pen pressure",
        "panel.painter_ponta.hose_mode.angle" => "Stroke angle",
        "panel.painter_ponta.hose_weight_1" => "Weight 1",
        "panel.painter_ponta.hose_weight_2" => "Weight 2",
        "panel.painter_ponta.hose_weight_3" => "Weight 3",
        "panel.painter_ponta.hose_weight_4" => "Weight 4",
        "panel.painter_ponta.hose_weight_5" => "Weight 5",
        "panel.painter_ponta.hose_weight_6" => "Weight 6",
        "panel.painter_ponta.hose_weight_7" => "Weight 7",
        "panel.painter_ponta.hose_weight_8" => "Weight 8",
        _ => return None,
    })
}
