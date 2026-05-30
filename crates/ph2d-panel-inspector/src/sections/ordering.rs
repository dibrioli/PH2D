//! Ordering / Sorting — Inspector §7 section painter (Sprite Inspector
//! v2 W3). Renders the render-ready optional sorting components: Z Index
//! (tri-state attach via an "Override" toggle), Z as Relative, Show
//! Behind Parent, Order in Layer, Y-Sort, Sorting Group / Sort At Root,
//! Top Level. Control VALUES come from the `WidgetStore` (seeded by
//! `sync_ordering_fields`); `info` drives the conditional rows.
//!
//! Sorting Layer (dropdown) + Y-Sort Sort Point (segmented) + Custom
//! Axis are the next §7 increment (separate widget wiring).

use super::*;
use ph2d_editor_core::screens::hero::InspectorOrderingInfo;

#[allow(clippy::too_many_arguments)]
pub(crate) fn paint_ordering_section(
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    hit_index: &mut HitIndex,
    store: &WidgetStore,
    x: f32,
    w: f32,
    y: f32,
    info: &InspectorOrderingInfo,
) -> f32 {
    let header_h = TypeToken::Md.px() + Spacing::Md.px(); // LITERAL-PX-OK: section header band height
    let collapsed = store.is_collapsed(ids::INSP_LIVE_ORDERING_SECTION);
    let color_id = ids::INSP_LIVE_ORDERING_COLOR;
    let rgba = store
        .widget_color(color_id)
        .unwrap_or([0x88, 0x88, 0x88, 0xff]); // LITERAL-COLOR-OK: neutral default section accent
    let header = SectionHeader::new(ids::INSP_LIVE_ORDERING_SECTION, "Ordering")
        .collapsible(!collapsed)
        .color(rgba);
    let header_rect = Rect::new(x, y, w, header_h);
    paint_section_header(&header, header_rect, scene, text_system, theme);
    if let Some(circle_rect) = ph2d_editor_core::widget::color_circle_hit_rect(&header, header_rect)
    {
        hit_index.register(color_id, circle_rect);
    }
    if collapsed {
        return y + header_h;
    }
    let mut cur_y = y + header_h;
    let field_h = ROW_H_PX;
    let check_h = 18.0_f32; // LITERAL-PX-OK: Checkbox visual height (box 16 + label baseline)
    let row_gap = Spacing::Sm.px();
    let label_font = TypeToken::Sm.px();
    let label_color = resolve(ColorToken::Text2, theme);

    // --- helpers (closures keep this fn under the LOC cap) ---
    let checkbox_value = |store: &WidgetStore, id: NodeId, fallback: CheckboxValue| {
        store.checkbox(id).map_or(fallback, |(_, v)| v)
    };
    let paint_check = |scene: &mut VectorScene,
                       text_system: &mut TextSystem,
                       hit_index: &mut HitIndex,
                       row_y: f32,
                       id: NodeId,
                       label: &str|
     -> f32 {
        let (state, value) = store
            .checkbox(id)
            .unwrap_or((CheckboxState::Normal, CheckboxValue::Unchecked));
        let host = Rect::new(x, row_y, w, check_h);
        hit_index.register(id, host);
        let checkbox = Checkbox::new(id, label).state(state).value(value);
        paint_checkbox(&checkbox, host, scene, text_system, theme);
        row_y + check_h + row_gap
    };
    let paint_number = |scene: &mut VectorScene,
                        text_system: &mut TextSystem,
                        hit_index: &mut HitIndex,
                        row_y: f32,
                        id: NodeId,
                        label: &str|
     -> f32 {
        // Label column + a single chip filling the remaining width.
        let label_col_w = 96.0_f32; // LITERAL-PX-OK: §7 row-label column width
        let gap = Spacing::Md.px();
        paint_text(
            text_system,
            scene,
            label,
            x,
            row_y + (field_h - label_font) * 0.5,
            label_font,
            label_col_w,
            label_color,
        );
        let chip_x = x + label_col_w + gap;
        let chip_w = (w - label_col_w - gap).max(0.0);
        let rect = Rect::new(chip_x, row_y, chip_w, field_h);
        hit_index.register(id, rect);
        let (state, value, buffer, caret, anchor) = read_number_input(store, id);
        let input = NumberInput::new(id, "", value).step(1.0).state(state);
        paint_number_input_with_buffer(
            &input, Some(buffer), caret, anchor, rect, scene, text_system, theme,
        );
        row_y + field_h + row_gap
    };

    // --- Z Index (tri-state via Override toggle) ---
    cur_y = paint_check(
        scene,
        text_system,
        hit_index,
        cur_y,
        ids::INSP_ORDER_Z_OVERRIDE,
        "Override Z Index",
    );
    // The dependent Z rows follow the CURRENT toggle (store), so flipping
    // Override on/off shows/hides them the same frame. Falls back to the
    // snapshot on the first frame before sync seeds the store.
    let z_on = match checkbox_value(store, ids::INSP_ORDER_Z_OVERRIDE, CheckboxValue::Unchecked) {
        CheckboxValue::Checked => true,
        CheckboxValue::Unchecked => false,
        CheckboxValue::Indeterminate => info.z_index.is_some(),
    };
    if z_on {
        cur_y = paint_number(
            scene,
            text_system,
            hit_index,
            cur_y,
            ids::INSP_ORDER_Z_INDEX,
            "Z Index",
        );
        cur_y = paint_check(
            scene,
            text_system,
            hit_index,
            cur_y,
            ids::INSP_ORDER_Z_RELATIVE,
            "Z as Relative",
        );
    }

    cur_y = paint_check(
        scene,
        text_system,
        hit_index,
        cur_y,
        ids::INSP_ORDER_SHOW_BEHIND,
        "Show Behind Parent",
    );
    cur_y = paint_number(
        scene,
        text_system,
        hit_index,
        cur_y,
        ids::INSP_ORDER_ORDER_IN_LAYER,
        "Order in Layer",
    );
    cur_y = paint_check(
        scene,
        text_system,
        hit_index,
        cur_y,
        ids::INSP_ORDER_YSORT_ENABLED,
        "Y-Sort",
    );

    // --- Sorting Group + Sort At Root (dependent) ---
    cur_y = paint_check(
        scene,
        text_system,
        hit_index,
        cur_y,
        ids::INSP_ORDER_SORTING_GROUP,
        "Sorting Group",
    );
    let group_on = match checkbox_value(store, ids::INSP_ORDER_SORTING_GROUP, CheckboxValue::Unchecked)
    {
        CheckboxValue::Checked => true,
        CheckboxValue::Unchecked => false,
        CheckboxValue::Indeterminate => info.sorting_group,
    };
    if group_on {
        cur_y = paint_check(
            scene,
            text_system,
            hit_index,
            cur_y,
            ids::INSP_ORDER_SORT_AT_ROOT,
            "Sort At Root",
        );
    }

    cur_y = paint_check(
        scene,
        text_system,
        hit_index,
        cur_y,
        ids::INSP_ORDER_TOP_LEVEL,
        "Top Level",
    );

    cur_y - row_gap + SECTION_BOTTOM_PAD_PX
}
