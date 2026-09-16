use ph2d_text::TextSystem;
use ph2d_tokens::TypeToken;
const CHAVES: &[&str] = &[
    "panel.painter_layers.brush.blend",
    "panel.painter_layers.brush.color",
    "panel.painter_layers.brush.paint_mode",
    "panel.painter_layers.brush.preset",
    "panel.painter_layers.grain.grain",
    "panel.painter_layers.grain.mapping",
    "panel.painter_layers.line.type",
    "panel.painter_layers.paper.color",
    "panel.painter_layers.paper.mapping",
    "panel.painter_layers.paper.paper",
    "panel.painter_layers.shape.falloff",
    "panel.painter_layers.shape.follow",
    "panel.painter_layers.shape.texture",
    "panel.painter_layers.stroke.jitter.unit",
    "panel.painter_layers.stroke.method",
];
#[test]
fn probe2() {
    let mut ts = TextSystem::new();
    let fonte = TypeToken::Sm.px();
    println!("\n== coluna literal de 60 px (paint_brush_rows::LABEL_W) ==");
    let mut cortados = 0;
    for k in CHAVES {
        let t = ph2d_i18n::tr(k);
        let w = ts.prefix_width(t, fonte);
        if w > 60.0 {
            cortados += 1;
            println!("   CORTA {w:6.1} > 60.0  {t}");
        }
    }
    println!(
        "   => {cortados} de {} cortados na coluna de 60",
        CHAVES.len()
    );
    for p in [220.0_f32, 245.0, 273.3, 304.0] {
        let linha = p - 2.0 * ph2d_tokens::PANEL_HEAD_PAD_PX;
        let mais = CHAVES
            .iter()
            .map(|k| ts.prefix_width(ph2d_i18n::tr(k), fonte))
            .fold(0.0_f32, f32::max);
        let porta = ph2d_editor_core::widget::property_label_col_w_for(
            0.0,
            linha,
            Some(mais),
            Some(ph2d_editor_core::widget::NUMBER_INPUT_MIN_W_PX),
        );
        let n = CHAVES
            .iter()
            .filter(|k| ts.prefix_width(ph2d_i18n::tr(k), fonte) > porta)
            .count();
        println!(
            "   painel {p}: linha {linha:.1} porta {porta:.1} -> {n} cortados (mais largo {mais:.1})"
        );
    }
}
