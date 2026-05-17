//! Showcase `paint_switches_section` painter.
//!
//! Extracted from monolithic `showcase.rs` in Wave 6+7 Phase 1.C.

use super::*;

#[allow(clippy::too_many_arguments)]
pub(super) fn paint_switches_section(
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
        ids::INSP_SECTION_SWITCHES,
        ids::INSP_SECTION_SWITCHES_COLOR,
        "Switches",
        3,
    );
    if !open {
        return y;
    }

    // Checkbox.
    let r = Rect::new(x, y, w, Density::Compact.row_h_px());
    hit_index.register(ids::INSP_SAMPLE_CHECKBOX, r);
    let (cb_state, cb_value) = store
        .checkbox(ids::INSP_SAMPLE_CHECKBOX)
        .unwrap_or((CheckboxState::Normal, CheckboxValue::Checked));
    let cb = Checkbox::new(ids::INSP_SAMPLE_CHECKBOX, "Hot reload on save")
        .state(cb_state)
        .value(cb_value);
    paint_checkbox(&cb, r, scene, text_system, theme);
    y += Density::Compact.row_h_px() + ROW_GAP;

    // Toggle.
    let toggle_w = TypeToken::Xl3.px(); // LITERAL-PX-OK: toggle widget width (matches TypeToken::Xl3 = 44px by coincidence; chrome-specific)
    let row_h = Density::Compact.row_h_px();
    let tr = Rect::new(x + w - toggle_w, y, toggle_w, row_h);
    hit_index.register(ids::INSP_SAMPLE_TOGGLE, tr);
    let (tg_state, tg_on) = store
        .toggle(ids::INSP_SAMPLE_TOGGLE)
        .unwrap_or((ToggleState::Normal, true));
    let toggle = Toggle::new(ids::INSP_SAMPLE_TOGGLE, "Snap to grid")
        .state(tg_state)
        .on(tg_on);
    paint_toggle(&toggle, tr, scene, theme);
    paint_left_label(
        scene,
        text_system,
        theme,
        x,
        "Snap to grid",
        w - toggle_w - Spacing::Sm.px(),
        y,
        row_h,
    );
    y += row_h + ROW_GAP;

    // Segmented RadioGroup. We use the per-tab pressed-button trick
    // (mirrors the Inspector-tabs pattern in `paint_inspector_tabs`
    // pre-cleanup) so selection survives across frames without
    // adding a typed RadioGroup state to the store.
    let selected = active_index(
        store,
        &[
            ids::INSP_SAMPLE_RADIO_A,
            ids::INSP_SAMPLE_RADIO_B,
            ids::INSP_SAMPLE_RADIO_C,
        ],
    )
    .unwrap_or(0);
    let selected_label: &str = ["Low", "Mid", "High"][selected];
    let rg = RadioGroup::new(
        NodeId(0),
        "Quality",
        vec![
            RadioOption::new(ids::INSP_SAMPLE_RADIO_A, "Low".to_string(), "Low"),
            RadioOption::new(ids::INSP_SAMPLE_RADIO_B, "Mid".to_string(), "Mid"),
            RadioOption::new(ids::INSP_SAMPLE_RADIO_C, "High".to_string(), "High"),
        ],
    )
    .orientation(RadioOrientation::Segmented)
    .selected(selected_label.to_string());
    let r = Rect::new(x, y, w, ROW_H_PX);
    paint_radio_group_with_labels(&rg, r, scene, text_system, theme);
    // Register per-option hit rects.
    for (i, id) in [
        ids::INSP_SAMPLE_RADIO_A,
        ids::INSP_SAMPLE_RADIO_B,
        ids::INSP_SAMPLE_RADIO_C,
    ]
    .iter()
    .enumerate()
    {
        hit_index.register(*id, rg.option_rect(r, i));
    }
    y + ROW_H_PX
}
