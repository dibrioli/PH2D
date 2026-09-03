//! Showcase `paint_slider_section` painter.
//!
//! Extracted from monolithic `showcase.rs` in Wave 6+7 Phase 1.C.

use super::*;

#[allow(clippy::too_many_arguments)]
pub(super) fn paint_slider_section(
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    hit_index: &mut HitIndex,
    store: &WidgetStore,
    x: f32,
    w: f32,
    y: f32,
) -> f32 {
    let (mut y, open) = paint_collapsible_header(
        scene,
        text_system,
        theme,
        hit_index,
        store,
        x,
        w,
        y,
        ids::INSP_SECTION_SLIDER,
        ids::INSP_SECTION_SLIDER_COLOR,
        "Slider",
        1,
    );
    if !open {
        return y;
    }
    let (_, value) = store
        .slider(ids::INSP_SAMPLE_SLIDER)
        .unwrap_or((SliderState::Normal, 0.62)); // LITERAL-PX-OK: slider default ratio (showcase demo seed value)
    let r = Rect::new(x, y, w, field_h());
    let slider_h = paint_slider_with_chip(
        r,
        "Speed",
        value,
        ids::INSP_SAMPLE_SLIDER,
        ids::INSP_SAMPLE_SLIDER_CHIP,
        store,
        hit_index,
        scene,
        text_system,
        theme,
    );
    y += slider_h;

    // ⭐⭐ **A CAIXA ÚNICA — o padrão do app desde 2026-09-02**, logo abaixo do widget que ela
    // substitui, e pela razão que esta galeria existe: ela é a fonte única de verdade do que o
    // editor **É**, e os agentes periféricos copiam a decoração daqui.
    //
    // ⚠️ **Amostra ESTÁTICA, sem `hit_index`.** A galeria mostra a forma; o gesto vive no produto e
    // na bancada (`ph2d-panel-widget-lab`). Registar aqui um alvo arrastável poria dois sliders a
    // competir pelo ponteiro dentro da mesma secção.
    //
    // ⚠️ E ela lê o `paint::slider_style()` — o que o artista escolheu —, então esta amostra muda
    // com a preferência. *Uma galeria que mostrasse o default fixo mentiria a quem customizou.*
    y += Spacing::Sm.px();
    let style = crate::paint::slider_style();
    let r = Rect::new(x, y, w, style.row_h_px());
    crate::widget::paint_property_box(
        scene,
        text_system,
        theme,
        r,
        crate::widget::PropertyBox {
            label: "Speed",
            value: "62%",
            t: value,
            state: crate::widget::PropertyBoxState::Normal,
            accent: ColorToken::Accent,
            decorator: true,
            // Amostra: a coluna do valor sai do texto. A LINHA do produto reserva-a, para os
            // números alinharem entre linhas — ver o doc do campo.
            value_w: None,
        },
        style,
    );
    y += style.row_h_px();
    y
}
