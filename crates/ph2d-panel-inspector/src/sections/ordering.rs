//! Ordering / Sorting — Inspector §7 section painter (Sprite Inspector
//! v2 W3). Renders the render-ready optional sorting components. Control
//! VALUES come from the `WidgetStore` (seeded by `sync_ordering_fields`)
//! or the snapshot; `info` drives the conditional rows.

use super::*;
use ph2d_editor_core::interaction::InteractiveState;
use ph2d_editor_core::screens::hero::InspectorOrderingInfo;
use ph2d_editor_core::widget::{Dropdown, DropdownOption, DropdownState, paint_dropdown_chip};

/// Canonical project sorting layers (spec §5.2 default set). Value ==
/// label so the dropdown's `selected` matches by name. Custom project
/// layer lists are a follow-up (no Project Settings UI yet).
pub(crate) const LAYER_LABELS: [&str; 5] =
    ["Background", "Midground", "Default", "Foreground", "UI"];

fn label_color(theme: Theme) -> VelloColor {
    resolve(ColorToken::Text2, theme)
}

/// One left-label + checkbox row. Returns the next `y`.
#[allow(clippy::too_many_arguments)]
fn check_row(
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    hit_index: &mut HitIndex,
    store: &WidgetStore,
    x: f32,
    w: f32,
    y: f32,
    id: NodeId,
    label: &str,
) -> f32 {
    let h = 18.0_f32; // LITERAL-PX-OK: Checkbox visual height (box 16 + label baseline)
    let (state, value) = store
        .checkbox(id)
        .unwrap_or((CheckboxState::Normal, CheckboxValue::Unchecked));
    let host = Rect::new(x, y, w, h);
    hit_index.register(id, host);
    let cb = Checkbox::new(id, label).state(state).value(value);
    paint_checkbox(&cb, host, scene, text_system, theme);
    y + h + Spacing::Sm.px()
}

/// One left-label + single NumberInput row. Returns the next `y`.
#[allow(clippy::too_many_arguments)]
fn number_row(
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    hit_index: &mut HitIndex,
    store: &WidgetStore,
    x: f32,
    w: f32,
    y: f32,
    id: NodeId,
    label: &str,
) -> f32 {
    let h = ROW_H_PX;
    let label_col_w = 96.0_f32; // LITERAL-PX-OK: §7 row-label column width
    let gap = Spacing::Md.px();
    paint_text(
        text_system,
        scene,
        label,
        x,
        y + (h - TypeToken::Sm.px()) * 0.5,
        TypeToken::Sm.px(),
        label_col_w,
        label_color(theme),
    );
    let chip_x = x + label_col_w + gap;
    let chip_w = (w - label_col_w - gap).max(0.0);
    let rect = Rect::new(chip_x, y, chip_w, h);
    hit_index.register(id, rect);
    let (state, value, buffer, caret, anchor) = read_number_input(store, id);
    let input = NumberInput::new(id, "", value).step(1.0).state(state);
    paint_number_input_with_buffer(
        &input,
        Some(buffer),
        caret,
        anchor,
        rect,
        scene,
        text_system,
        theme,
    );
    y + h + Spacing::Sm.px()
}

/// Sorting Layer dropdown row. Stashes the open popover for the deferred
/// pass in `paint_inspector`. Returns the next `y`.
#[allow(clippy::too_many_arguments)]
fn layer_row(
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    hit_index: &mut HitIndex,
    store: &WidgetStore,
    x: f32,
    w: f32,
    y: f32,
    fallback_idx: usize,
) -> f32 {
    let h = ROW_H_PX;
    let label_col_w = 96.0_f32; // LITERAL-PX-OK: §7 row-label column width
    let gap = Spacing::Md.px();
    paint_text(
        text_system,
        scene,
        "Sorting Layer",
        x,
        y + (h - TypeToken::Sm.px()) * 0.5,
        TypeToken::Sm.px(),
        label_col_w,
        label_color(theme),
    );
    let chip_x = x + label_col_w + gap;
    let chip_w = (w - label_col_w - gap).max(0.0);
    let rect = Rect::new(chip_x, y, chip_w, h);
    hit_index.register(ids::INSP_ORDER_SORTING_LAYER, rect);
    let (state, open, sel) = match store.get(ids::INSP_ORDER_SORTING_LAYER) {
        Some(InteractiveState::Dropdown {
            state,
            open,
            selected_index,
        }) => (*state, *open, selected_index.unwrap_or(fallback_idx)),
        _ => (DropdownState::Normal, false, fallback_idx),
    };
    let label = LAYER_LABELS.get(sel).copied().unwrap_or("Default");
    let dd = Dropdown::new(ids::INSP_ORDER_SORTING_LAYER, "", layer_options())
        .selected(label)
        .open(open)
        .state(state);
    paint_dropdown_chip(&dd, rect, scene, text_system, theme);
    if open {
        crate::state::set_pending_ordering_dd(Some((sel, rect)));
    }
    y + h + Spacing::Sm.px()
}

/// Canonical dropdown options (value == label).
pub(crate) fn layer_options() -> Vec<DropdownOption<&'static str>> {
    LAYER_LABELS
        .iter()
        .enumerate()
        .map(|(i, name)| DropdownOption::new(ids::INSP_ORDER_LAYER_OPT[i], *name, *name))
        .collect()
}

/// Y-Sort Sort Point segmented (Center/Pivot/Custom) + Custom Axis
/// (shown only on Custom). Snapshot-driven selection. Returns next `y`.
#[allow(clippy::too_many_arguments)]
fn ysort_point_rows(
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
    let h = ROW_H_PX;
    let rect = Rect::new(x, y, w, h);
    let tabs = Tabs::new(
        NodeId(0),
        "",
        vec![
            TabItem::new(ids::INSP_ORDER_SP_CENTER, "Center"),
            TabItem::new(ids::INSP_ORDER_SP_PIVOT, "Pivot"),
            TabItem::new(ids::INSP_ORDER_SP_CUSTOM, "Custom"),
        ],
    )
    .variant(TabsVariant::Segmented)
    .selected(info.y_sort_point as usize);
    paint_tabs(&tabs, rect, scene, text_system, theme);
    for (i, item) in tabs.items.iter().enumerate() {
        hit_index.register(item.id, tabs.tab_rect(rect, i));
    }
    let mut cur_y = y + h + Spacing::Sm.px();
    // Custom Axis — only when Sort Point = Custom (tag 2).
    if info.y_sort_point == 2 {
        cur_y = number_row(
            scene,
            text_system,
            theme,
            hit_index,
            store,
            x,
            w,
            cur_y,
            ids::INSP_ORDER_AXIS_X,
            "Axis X",
        );
        cur_y = number_row(
            scene,
            text_system,
            theme,
            hit_index,
            store,
            x,
            w,
            cur_y,
            ids::INSP_ORDER_AXIS_Y,
            "Axis Y",
        );
    }
    cur_y
}

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
    let mut yy = y + header_h;
    let live =
        |store: &WidgetStore, id: NodeId, snap: bool| match store.checkbox(id).map(|(_, v)| v) {
            Some(CheckboxValue::Checked) => true,
            Some(CheckboxValue::Unchecked) => false,
            _ => snap,
        };
    // Macros (not closures) so each row is a direct fn call that
    // re-borrows `scene`/`text_system`/`hit_index` per invocation.
    macro_rules! cb {
        ($yy:expr, $id:expr, $l:expr) => {
            check_row(
                scene,
                text_system,
                theme,
                hit_index,
                store,
                x,
                w,
                $yy,
                $id,
                $l,
            )
        };
    }
    macro_rules! ni {
        ($yy:expr, $id:expr, $l:expr) => {
            number_row(
                scene,
                text_system,
                theme,
                hit_index,
                store,
                x,
                w,
                $yy,
                $id,
                $l,
            )
        };
    }

    // Z Index — the field IS the override (no separate toggle, Godot-
    // style): a non-zero value attaches `ZIndexOverride`, `0` detaches it
    // (= default, pure DFS / hierarchy order — `0` and absent sort
    // identically). Z as Relative pairs with it.
    yy = ni!(yy, ids::INSP_ORDER_Z_INDEX, "Z Index");
    yy = cb!(yy, ids::INSP_ORDER_Z_RELATIVE, "Z as Relative");
    yy = cb!(yy, ids::INSP_ORDER_SHOW_BEHIND, "Show Behind Parent");
    yy = ni!(yy, ids::INSP_ORDER_ORDER_IN_LAYER, "Order in Layer");
    yy = layer_row(
        scene,
        text_system,
        theme,
        hit_index,
        store,
        x,
        w,
        yy,
        info.sorting_layer as usize,
    );
    yy = cb!(yy, ids::INSP_ORDER_YSORT_ENABLED, "Y-Sort");
    if live(store, ids::INSP_ORDER_YSORT_ENABLED, info.y_sort_enabled) {
        yy = ysort_point_rows(scene, text_system, theme, hit_index, store, x, w, yy, info);
    }
    yy = cb!(yy, ids::INSP_ORDER_SORTING_GROUP, "Sorting Group");
    if live(store, ids::INSP_ORDER_SORTING_GROUP, info.sorting_group) {
        yy = cb!(yy, ids::INSP_ORDER_SORT_AT_ROOT, "Sort At Root");
    }
    yy = cb!(yy, ids::INSP_ORDER_TOP_LEVEL, "Top Level");

    yy - Spacing::Sm.px() + SECTION_BOTTOM_PAD_PX
}
