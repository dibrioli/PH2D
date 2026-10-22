//! ⭐ **As palavras da MARCA e da cor pela mão** (as lacunas L4 e L11 dos pincéis, `docs/Painter/Pinceis/03_MOTORES.md`
//! §4): as linhas que o cartão Jitter e a secção Randomize Color do painel Painter acrescentam. Tabela IRMÃ da
//! [`super::painter_layers`] por isolamento (`CLAUDE.md` §0.2): outra linha mexe naquela ao mesmo tempo.

/// A tradução de uma chave `panel.painter_marca.*`, ou `None` se ela não é daqui.
pub(crate) fn tr(key: &str) -> Option<&'static str> {
    Some(match key {
        "panel.painter_marca.opacity" => "Opacity jitter",
        "panel.painter_marca.count" => "Count jitter",
        "panel.painter_marca.roundness" => "Roundness",
        "panel.painter_marca.flip_x" => "Flip X",
        "panel.painter_marca.flip_y" => "Flip Y",
        "panel.painter_marca.angle_per_stroke" => "Stroke angle",
        "panel.painter_marca.across" => "Scatter",
        "panel.painter_marca.particle" => "Particle size",
        "panel.painter_marca.secondary" => "2nd color",
        "panel.painter_marca.secondary_jitter" => "Random 2nd",
        "panel.painter_marca.pressure_secondary" => "Pressure 2nd",
        "panel.painter_marca.pressure_hue" => "Pressure hue",
        "panel.painter_marca.pressure_sat" => "Pressure sat.",
        "panel.painter_marca.pressure_val" => "Pressure value",
        _ => return None,
    })
}
