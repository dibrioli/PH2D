//! Identity (Name + Visibility) — Inspector section painter (split from sections.rs,
//! architecture_panel_loc_cap). Logic verbatim; behavior unchanged.

use super::*;
use ph2d_i18n::tr;

#[allow(clippy::too_many_arguments)]
pub(crate) fn paint_entity_name_row(
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    hit_index: &mut HitIndex,
    store: &WidgetStore,
    x: f32,
    w: f32,
    y: f32,
) -> f32 {
    let row_h = ROW_H_PX;
    // A coluna de animação, pela porta.
    let (control_w, dot) = ph2d_editor_core::widget::form_row_columns(x, w, y, row_h);
    let host = Rect::new(x, y, control_w, row_h);
    hit_index.register(core_ids::INSP_ENTITY_NAME, host);
    let (state, text, caret, anchor) = match store.get(core_ids::INSP_ENTITY_NAME) {
        Some(InteractiveState::TextInput {
            state,
            text,
            caret,
            selection_anchor,
        }) => (*state, Some(text.as_str()), *caret, *selection_anchor),
        _ => (TextInputState::Normal, None, 0, None),
    };
    let input = TextInput::new(core_ids::INSP_ENTITY_NAME, "")
        .placeholder(tr("panel.inspector.identity.name"))
        .visual((state, store.hover_live(core_ids::INSP_ENTITY_NAME)));
    paint_text_input_with_buffer(
        &input,
        text,
        Some(caret),
        anchor,
        host,
        scene,
        text_system,
        theme,
    );
    ph2d_editor_core::widget::paint_decorator_dot(scene, theme, dot);
    y + row_h + SECTION_BOTTOM_PAD_PX
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn paint_visibility_row(
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    hit_index: &mut HitIndex,
    store: &WidgetStore,
    x: f32,
    w: f32,
    y: f32,
) -> f32 {
    // Tight row that matches the checkbox's actual visual height (~18
    // px = label baseline + box). Pre-canon was 24 px → extra ~6 px
    // padding below the checkbox before SECTION_BOTTOM_PAD_PX,
    // making the gap to the separator visibly larger than Transform's
    // (user 2026-05-24: "espaço entre visible e separador fora do
    // padrão"). Now both sections finish at the same visual rhythm.
    let row_h = ph2d_tokens::ROW_H_PX; // ⛔ era `18.0`, o MESMO literal em TREZE sitios: a linha de marcar e' uma linha de propriedade, e a altura dela e' a do app (report do dono 2026-09-15: a marca enchia a caixa toda)
    let (_, value) = match store.checkbox(ids::INSP_VISIBILITY_CHECK) {
        Some(pair) => pair,
        None => (CheckboxState::Normal, CheckboxValue::Checked),
    };
    let host = Rect::new(x, y, w, row_h);
    hit_index.register(ids::INSP_VISIBILITY_CHECK, host);
    let checkbox = Checkbox::new(
        ids::INSP_VISIBILITY_CHECK,
        tr("panel.inspector.identity.visible"),
    )
    .visual(store.checkbox_visual(ids::INSP_VISIBILITY_CHECK))
    .value(value);
    paint_checkbox(&checkbox, host, scene, text_system, theme);
    y + row_h + SECTION_BOTTOM_PAD_PX
}
