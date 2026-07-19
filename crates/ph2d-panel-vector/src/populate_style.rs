//! O registro dos widgets de **ESTILO** do painel Vector — irmão de `populate.rs` pelo teto
//! de 600 LOC daquele arquivo. Traço (largura, cap, join, dash, gap), as PONTAS e o
//! preenchimento: a família que o artista pensa como "com que cor e com que caneta".

use super::{
    GRAD_ANGLE_SLIDER_SCALE, GRAD_INFLUENCE_SLIDER_SCALE, GRAD_JITTER_SLIDER_SCALE, button, ids,
    number_field, slider_chip,
};
use ph2d_editor_core::interaction::{InteractiveState, WidgetStore};
use ph2d_editor_core::widget::DropdownState;
use ph2d_tool_vector::params;
use ph2d_tool_vector::params::{
    DASH_SLIDER_OFFSET, DASH_SLIDER_SCALE, GAP_DEFAULT, GAP_SLIDER_OFFSET, GAP_SLIDER_SCALE,
    OPACITY_SLIDER_OFFSET, OPACITY_SLIDER_SCALE, gap_to_slider,
};

/// Gradiente (Angle / Influence / Jitter), opacidades e o traço (cap/join/dash).
pub(super) fn populate_style(store: &mut WidgetStore) {
    // Linear-gradient Angle slider (track 0..1 → 0..360°) + its chip.
    slider_chip(
        store,
        ids::VECTOR_GRAD_ANGLE,
        ids::VECTOR_GRAD_ANGLE_NUM,
        0.0,
        0.0,
        GRAD_ANGLE_SLIDER_SCALE,
        0.0,
    );
    // Multi-point Influence (track 0..1 → 0..4); seeded at the default 1.0.
    slider_chip(
        store,
        ids::VECTOR_GRAD_INFLUENCE,
        ids::VECTOR_GRAD_INFLUENCE_NUM,
        1.0 / GRAD_INFLUENCE_SLIDER_SCALE,
        1.0,
        GRAD_INFLUENCE_SLIDER_SCALE,
        0.0,
    );
    // Multi-point Jitter (track 0..1 → 0..1); seeded at the default 0.0 (smooth).
    slider_chip(
        store,
        ids::VECTOR_GRAD_JITTER,
        ids::VECTOR_GRAD_JITTER_NUM,
        0.0,
        0.0,
        GRAD_JITTER_SLIDER_SCALE,
        0.0,
    );

    // Stroke / Fill Opacity sliders (0..100 %) — seeded at full opacity, matching
    // the tool's default opaque stroke/fill.
    slider_chip(
        store,
        ids::VECTOR_STROKE_OPACITY,
        ids::VECTOR_STROKE_OPACITY_NUM,
        1.0,
        100.0, // LITERAL-PX-OK: initial opacity display = 100 %
        OPACITY_SLIDER_SCALE,
        OPACITY_SLIDER_OFFSET,
    );
    slider_chip(
        store,
        ids::VECTOR_FILL_OPACITY,
        ids::VECTOR_FILL_OPACITY_NUM,
        1.0,
        100.0, // LITERAL-PX-OK: initial opacity display = 100 %
        OPACITY_SLIDER_SCALE,
        OPACITY_SLIDER_OFFSET,
    );

    // Stroke Cap / Join segmented buttons + Dash length slider (0 px = solid).
    button(store, ids::VECTOR_CAP_BUTT);
    button(store, ids::VECTOR_CAP_ROUND);
    button(store, ids::VECTOR_CAP_SQUARE);
    button(store, ids::VECTOR_JOIN_MITER);
    button(store, ids::VECTOR_JOIN_ROUND);
    button(store, ids::VECTOR_JOIN_BEVEL);
    slider_chip(
        store,
        ids::VECTOR_DASH,
        ids::VECTOR_DASH_NUM,
        0.0,
        0.0,
        DASH_SLIDER_SCALE,
        DASH_SLIDER_OFFSET,
    );
    slider_chip(
        store,
        ids::VECTOR_GAP,
        ids::VECTOR_GAP_NUM,
        gap_to_slider(GAP_DEFAULT),
        GAP_DEFAULT,
        GAP_SLIDER_SCALE,
        GAP_SLIDER_OFFSET,
    );
    populate_markers(store);
}

/// As **pontas do traço**: os dois chips (`Dropdown` — abrir/fechar/roda vêm de graça do
/// dispatch genérico) + as opções do popover, uma por ponta de `ALL_MARKERS` e por slot.
///
/// Registradas por ÍNDICE, como as formas do catálogo: uma ponta nova entra em
/// `ALL_MARKERS` e já nasce clicável, sem tocar aqui. Os slots existem sempre (o
/// `populate` é estático); o PAINT decide quais registram hit.
fn populate_markers(store: &mut WidgetStore) {
    for slot in 0..ids::MARKER_SLOTS {
        store.register_if_absent(
            crate::paint_markers::marker_dd_id(slot),
            InteractiveState::Dropdown {
                state: DropdownState::Normal,
                open: false,
                selected_index: None,
            },
        );
        for i in 0..ids::MAX_MARKER_OPTIONS {
            button(store, ids::vector_marker_option_id(slot, i));
        }
    }
    // **Tamanho / arredondamento** da ponta: caixas numéricas de faixa FIXA. O
    // `set_number_range` não é opcional — sem ele o arrasto escala errado (o gotcha da caixa
    // limitada). O valor é re-semeado com o efetivo da tool a cada frame (Fase B do paint).
    for (id, field) in [
        (ids::VECTOR_MARKER_SCALE, &params::MARKER_SCALE),
        (ids::VECTOR_MARKER_ROUND, &params::MARKER_ROUND),
    ] {
        number_field(store, id, field.min, field.max, field.step, field.min);
    }
    // **Both Ends** — um botão. Registrar aqui é o que o torna clicável: pintar e dar
    // hit-rect não basta (a gate `architecture_panel_wiring_parity` exige o registro DENTRO
    // do `populate.rs`, e ela tem razão — sem `InteractiveState` o widget nunca é focável e
    // Down/Up jamais disparam).
    button(store, ids::VECTOR_MARKER_BOTH);
}
