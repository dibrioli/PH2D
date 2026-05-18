//! Showcase `paint_inputs_section` painter.
//!
//! Extracted from monolithic `showcase.rs` in Wave 6+7 Phase 1.C.

use super::*;

#[allow(clippy::too_many_arguments)]
pub(super) fn paint_inputs_section(
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
        ids::INSP_SECTION_INPUTS,
        ids::INSP_SECTION_INPUTS_COLOR,
        "Inputs",
        4,
    );
    if !open {
        return y;
    }

    // TextInput.
    let r = Rect::new(x, y, w, FIELD_H);
    hit_index.register(ids::INSP_SAMPLE_TEXT, r);
    let (ti_state, ti_text, ti_caret, ti_anchor) = read_text_input(store, ids::INSP_SAMPLE_TEXT);
    let input = TextInput::new(ids::INSP_SAMPLE_TEXT, "Name")
        .placeholder("Entity name")
        .state(ti_state);
    paint_text_input_with_buffer(
        &input,
        Some(ti_text),
        Some(ti_caret),
        ti_anchor,
        r,
        scene,
        text_system,
        theme,
    );
    y += FIELD_H + ROW_GAP;

    // TextArea (3 lines tall).
    let area_h = 60.0_f32; // LITERAL-PX-OK: 3-line TextArea showcase height (chrome-specific demo dim)
    let r = Rect::new(x, y, w, area_h);
    hit_index.register(ids::INSP_SAMPLE_TEXTAREA, r);
    let (ta_state, ta_text, ta_caret, ta_anchor) =
        read_text_input(store, ids::INSP_SAMPLE_TEXTAREA);
    let mut ta = TextArea::new(ids::INSP_SAMPLE_TEXTAREA, "Notes")
        .placeholder("Notes…")
        .state(ta_state);
    ta.value = ta_text.to_string();
    paint_text_area_with_state(&ta, Some(ta_caret), ta_anchor, r, scene, text_system, theme);
    y += area_h + ROW_GAP;

    // Combobox.
    let r = Rect::new(x, y, w, FIELD_H);
    hit_index.register(ids::INSP_SAMPLE_COMBO, r);
    let (cb_state, cb_open, cb_query, cb_caret, cb_anchor) =
        read_combobox(store, ids::INSP_SAMPLE_COMBO);
    let combo = Combobox::new(
        ids::INSP_SAMPLE_COMBO,
        "Asset",
        vec![
            ComboboxOption::new(ids::INSP_SAMPLE_COMBO_OPT_A, "spike.gltf"),
            ComboboxOption::new(ids::INSP_SAMPLE_COMBO_OPT_B, "block.gltf"),
            ComboboxOption::new(ids::INSP_SAMPLE_COMBO_OPT_C, "decal.png"),
        ],
    )
    .query(cb_query)
    .open(cb_open)
    .state(cb_state);
    paint_combobox_with_state(&combo, cb_caret, cb_anchor, r, scene, text_system, theme);
    y += FIELD_H + ROW_GAP;

    // NumberInput.
    let label_w = 80.0_f32; // LITERAL-PX-OK: showcase label column width (demo geometry)
    let chip_w = (w - label_w - Spacing::Md.px()).max(40.0); // LITERAL-PX-OK: min chip width (demo)
    let r = Rect::new(x + label_w + Spacing::Md.px(), y, chip_w, FIELD_H);
    hit_index.register(ids::INSP_SAMPLE_NUMBER, r);
    paint_left_label(scene, text_system, theme, x, "Value", label_w, y, FIELD_H);
    let (n_state, n_value, n_buffer, n_caret, n_anchor) =
        read_number_input(store, ids::INSP_SAMPLE_NUMBER);
    let num = NumberInput::new(ids::INSP_SAMPLE_NUMBER, "Value", n_value).state(n_state);
    paint_number_input_with_buffer(
        &num,
        Some(n_buffer),
        n_caret,
        n_anchor,
        r,
        scene,
        text_system,
        theme,
    );
    y + FIELD_H
}
