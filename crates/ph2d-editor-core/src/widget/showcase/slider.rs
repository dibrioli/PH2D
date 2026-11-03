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
    let (y, open) = paint_collapsible_header(
        scene,
        text_system,
        theme,
        hit_index,
        store,
        x,
        w,
        y,
        ids::INSP_SECTION_SLIDER,
        "Slider",
        1,
    );
    if !open {
        return y;
    }
    paint_slider_body(scene, text_system, theme, hit_index, store, x, w, y)
}

/// O CORPO da secção «Slider», sem o cabeçalho, a partir de `y`; devolve o fundo. O cartão
/// da galeria chama-o (`SHOWCASE_CARDS`, doc 08 §1.2); a janela flutuante, pelo `*_section`.
#[allow(clippy::too_many_arguments)]
pub fn paint_slider_body(
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    hit_index: &mut HitIndex,
    store: &WidgetStore,
    x: f32,
    w: f32,
    mut y: f32,
) -> f32 {
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
    let vao = super::encaixe::no_cartao(m::FIELD_ROW_GAP, Spacing::Sm.px());

    // A barra de DUAS pegas (intervalo `lo..hi`, Pixel P8 — a dinâmica do pincel) e a caixa da
    // CURVA de cor (Pixel P7 — Color Curve). Amostras ESTÁTICAS, sem `hit_index`: o gesto vive no
    // produto (`ph2d-panel-pixel-tools`, o diálogo Color Curve), a galeria mostra a forma.
    y += vao;
    let (lo, hi) = (0.25, 0.75); // LITERAL-PX-OK: intervalo de amostra da galeria (fracções da trilha)
    crate::widget::paint_range_slider(Rect::new(x, y, w, field_h()), lo, hi, None, scene, theme);
    y += field_h() + row_gap();
    let curve_h = field_h() * 4.0; // LITERAL-PX-OK: quatro filas, a altura da caixa no diálogo Color Curve
    let pontos = [(64, 40), (192, 220)];
    crate::widget::paint_curve_editor(Rect::new(x, y, w, curve_h), &pontos, None, scene, theme);
    y += curve_h;

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
    //
    // ⛔⛔ **E ela só aparece no REDESENHO** (Enio, 2026-09-03: *«por enquanto permanece a
    // antiga»*). A galeria é a fonte de verdade do que o editor **É** — mostrar aqui um widget que
    // o app não pinta seria anunciar uma linguagem que ninguém vê. ⚠️ Foi o **censo** que o
    // encontrou, não a leitura: o gate do interruptor media os três pintores, e este sítio chama o
    // pintor da caixa **directamente**.
    if !crate::paint::ui_is_redesign() {
        return y;
    }
    y += vao;
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
            animado: None,
            // Amostra: a coluna do valor sai do texto. A LINHA do produto reserva-a, para os
            // números alinharem entre linhas — ver o doc do campo.
            value_w: None,
        },
        style,
    );
    y += style.row_h_px();
    y
}
