//! Number rows + origin/aabb/overlay/opacity/labeled-toggle painters.
//!
//! Ported verbatim from
//! `ph2d_editor_core::grid_snap::panel::paint_rows` during ADR-0029
//! Phase C.4.

use crate::ids;
use crate::layout::{LABEL_COL_W, LABEL_FONT_SIZE, ROW_GAP, ROW_H};
use crate::paint_helpers::toggle_state;
use crate::state::{meters_to_display, unit_suffix_paren};
use ph2d_editor_core::NodeId;
use ph2d_editor_core::grid_snap::GridSnapState;
use ph2d_editor_core::interaction::{HitIndex, WidgetStore};
use ph2d_editor_core::paint::{paint_text, resolve};
use ph2d_editor_core::widget::{
    NumberInput, TextInputState, Toggle, paint_number_input_with_buffer, paint_toggle,
};
use ph2d_editor_core::zones::Rect;
use ph2d_text::TextSystem;
use ph2d_tokens::{ColorToken, Spacing, Theme};
use ph2d_vector::VectorScene;

pub(crate) fn read_number_input(
    store: &WidgetStore,
    id: NodeId,
) -> (TextInputState, f64, &str, usize, Option<usize>) {
    store
        .number_input(id)
        .unwrap_or((TextInputState::Normal, 0.0, "", 0, None))
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn paint_number_row(
    label: &str,
    id: NodeId,
    x: f32,
    w: f32,
    y: f32,
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    hit_index: &mut HitIndex,
    store: &WidgetStore,
) -> f32 {
    let (state, value, buffer, caret, anchor) = read_number_input(store, id);
    paint_number_row_value(
        label,
        id,
        value,
        Some(buffer),
        caret,
        anchor,
        state,
        x,
        w,
        y,
        scene,
        text_system,
        theme,
        hit_index,
    )
}

/// Like [`paint_number_row`] but takes an explicit displayed value
/// (caller-supplied — typically read from state, not the store).
/// Used for "universal" origin / spacing rows that mirror the
/// ACTIVE kind's cfg field even though one NodeId is shared.
#[allow(clippy::too_many_arguments)]
pub(crate) fn paint_number_row_from_state(
    label: &str,
    id: NodeId,
    value: f64,
    x: f32,
    w: f32,
    y: f32,
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    hit_index: &mut HitIndex,
    store: &WidgetStore,
) -> f32 {
    let (state, _, buffer, caret, anchor) = read_number_input(store, id);
    // Keep the live buffer for in-progress edits; otherwise display
    // the state-supplied value (so switching kinds repaints with the
    // new kind's origin).
    let buffer_arg = if state == TextInputState::Focused {
        Some(buffer)
    } else {
        None
    };
    paint_number_row_value(
        label,
        id,
        value,
        buffer_arg,
        caret,
        anchor,
        state,
        x,
        w,
        y,
        scene,
        text_system,
        theme,
        hit_index,
    )
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn paint_number_row_value(
    label: &str,
    id: NodeId,
    value: f64,
    buffer: Option<&str>,
    caret: usize,
    anchor: Option<usize>,
    state: TextInputState,
    x: f32,
    w: f32,
    y: f32,
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    hit_index: &mut HitIndex,
) -> f32 {
    paint_text(
        text_system,
        scene,
        label,
        x,
        y + (ROW_H - LABEL_FONT_SIZE) * 0.5,
        LABEL_FONT_SIZE,
        LABEL_COL_W - Spacing::Sm.px(),
        resolve(ColorToken::Text1, theme),
    );
    let input_rect = Rect::new(x + LABEL_COL_W, y, w - LABEL_COL_W, ROW_H);
    let input = NumberInput::new(id, "", value).state(state);
    paint_number_input_with_buffer(
        &input,
        buffer,
        caret,
        anchor,
        input_rect,
        scene,
        text_system,
        theme,
    );
    hit_index.register(id, input_rect);
    y + ROW_H + ROW_GAP
}

/// Paint Origin X + Origin Y rows reading current values from
/// `state.active_origin()`. Returns Y after the second row.
#[allow(clippy::too_many_arguments)]
pub(crate) fn paint_origin_rows(
    x: f32,
    w: f32,
    y: f32,
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    hit_index: &mut HitIndex,
    store: &WidgetStore,
    state: &GridSnapState,
) -> f32 {
    let origin = state.active_origin();
    let suffix = unit_suffix_paren();
    let y = paint_number_row_from_state(
        &format!("Origin X{suffix}"),
        ids::GS_CFG_ORIGIN_X,
        meters_to_display(origin[0]),
        x,
        w,
        y,
        scene,
        text_system,
        theme,
        hit_index,
        store,
    );
    paint_number_row_from_state(
        &format!("Origin Y{suffix}"),
        ids::GS_CFG_ORIGIN_Y,
        meters_to_display(origin[1]),
        x,
        w,
        y,
        scene,
        text_system,
        theme,
        hit_index,
        store,
    )
}

/// Paint a labeled AABB (min X / min Y / max X / max Y) as 4
/// stacked NumberInput rows reading current values from state.
/// Used by Quadtree and Voronoi for their `bounds` field.
#[allow(clippy::too_many_arguments)]
pub(crate) fn paint_aabb_rows(
    label_prefix: &str,
    min_x_id: NodeId,
    min_y_id: NodeId,
    max_x_id: NodeId,
    max_y_id: NodeId,
    min: ph2d_grid::Vec2,
    max: ph2d_grid::Vec2,
    x: f32,
    w: f32,
    y: f32,
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    hit_index: &mut HitIndex,
    store: &WidgetStore,
) -> f32 {
    let suffix = unit_suffix_paren();
    let y = paint_number_row_from_state(
        &format!("{label_prefix} min X{suffix}"),
        min_x_id,
        meters_to_display(min[0]),
        x,
        w,
        y,
        scene,
        text_system,
        theme,
        hit_index,
        store,
    );
    let y = paint_number_row_from_state(
        &format!("{label_prefix} min Y{suffix}"),
        min_y_id,
        meters_to_display(min[1]),
        x,
        w,
        y,
        scene,
        text_system,
        theme,
        hit_index,
        store,
    );
    let y = paint_number_row_from_state(
        &format!("{label_prefix} max X{suffix}"),
        max_x_id,
        meters_to_display(max[0]),
        x,
        w,
        y,
        scene,
        text_system,
        theme,
        hit_index,
        store,
    );
    paint_number_row_from_state(
        &format!("{label_prefix} max Y{suffix}"),
        max_y_id,
        meters_to_display(max[1]),
        x,
        w,
        y,
        scene,
        text_system,
        theme,
        hit_index,
        store,
    )
}

pub(crate) fn paint_show_overlay_row(
    row: Rect,
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    hit_index: &mut HitIndex,
    store: &WidgetStore,
    state: &GridSnapState,
) {
    paint_labeled_toggle(
        "Show grid",
        ids::GS_SHOW_OVERLAY,
        state.show_overlay,
        row,
        scene,
        text_system,
        theme,
        hit_index,
        store,
    );
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn paint_opacity_slider_row(
    row: Rect,
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    hit_index: &mut HitIndex,
    store: &WidgetStore,
    state: &GridSnapState,
) {
    // Canonical label + track + chip — the same painter the Widget
    // Gallery slider uses, so this matches every other slider in the
    // app. (Was a one-off label + bare `paint_slider`.)
    let value = store
        .slider(ids::GS_OPACITY_SLIDER)
        .map(|(_, v)| v)
        .unwrap_or(state.opacity);
    ph2d_editor_core::widget::paint_slider_with_chip(
        row,
        "Opacity",
        value,
        ids::GS_OPACITY_SLIDER,
        ph2d_editor_core::NodeId(0),
        store,
        hit_index,
        scene,
        text_system,
        theme,
    );
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn paint_labeled_toggle(
    label: &str,
    id: NodeId,
    on: bool,
    row: Rect,
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    hit_index: &mut HitIndex,
    store: &WidgetStore,
) {
    paint_text(
        text_system,
        scene,
        label,
        row.x,
        row.y + (row.h - LABEL_FONT_SIZE) * 0.5,
        LABEL_FONT_SIZE,
        row.w - 60.0,
        resolve(ColorToken::Text1, theme),
    );

    let toggle_w = 40.0;
    let toggle_h = 20.0;
    let toggle_rect = Rect::new(
        row.x + row.w - toggle_w - 4.0,
        row.y + (row.h - toggle_h) * 0.5,
        toggle_w,
        toggle_h,
    );
    let toggle = Toggle {
        id,
        label: String::new(),
        on,
        state: toggle_state(store, id),
    };
    paint_toggle(&toggle, toggle_rect, scene, theme);
    hit_index.register(id, toggle_rect);
}

// Re-export `button_state` so `paint_kinds.rs` can use it without
// reaching across modules; mirrors the legacy `super::*` glob.
pub(crate) use crate::paint_helpers::button_state;
