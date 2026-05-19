//! M14 live Inspector section painters (Name / Visibility / Transform
//! / Render Source). Migrated to the panel crate in ADR-0029 Phase C.1.

use crate::state::current_display_unit;
use ph2d_a11y::NodeId;
use ph2d_editor_core::ids;
use ph2d_editor_core::interaction::{HitIndex, InteractiveState, WidgetStore};
use ph2d_editor_core::paint::{paint_text, paint_text_title, resolve};
use ph2d_editor_core::screens::hero::{InspectorSpriteInfo, InspectorSpriteSource};
use ph2d_editor_core::widget::showcase::{paint_section_separator, read_number_input};
use ph2d_editor_core::widget::{
    Button, ButtonKind, ButtonState, Checkbox, CheckboxState, CheckboxValue, NumberInput,
    TextInput, TextInputState, paint_button, paint_checkbox, paint_number_input_with_buffer,
    paint_text_input_with_buffer,
};
use ph2d_editor_core::zones::Rect;
use ph2d_text::TextSystem;
use ph2d_tokens::{ColorToken, ROW_H_PX, Spacing, Theme, TypeToken};
use ph2d_vector::VectorScene;

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
    let host = Rect::new(x, y, w, row_h);
    hit_index.register(ids::INSP_ENTITY_NAME, host);
    let (state, text, caret, anchor) = match store.get(ids::INSP_ENTITY_NAME) {
        Some(InteractiveState::TextInput {
            state,
            text,
            caret,
            selection_anchor,
        }) => (*state, Some(text.as_str()), *caret, *selection_anchor),
        _ => (TextInputState::Normal, None, 0, None),
    };
    let input = TextInput::new(ids::INSP_ENTITY_NAME, "")
        .placeholder("Name\u{2026}")
        .state(state);
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
    y + row_h + Spacing::Sm.px()
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
    let row_h = 24.0_f32; // LITERAL-PX-OK: compact checkbox row
    let (state, value) = match store.checkbox(ids::INSP_VISIBILITY_CHECK) {
        Some(pair) => pair,
        None => (CheckboxState::Normal, CheckboxValue::Checked),
    };
    let host = Rect::new(x, y, w, row_h);
    hit_index.register(ids::INSP_VISIBILITY_CHECK, host);
    let checkbox = Checkbox::new(ids::INSP_VISIBILITY_CHECK, "Visible")
        .state(state)
        .value(value);
    paint_checkbox(&checkbox, host, scene, text_system, theme);
    y + row_h + Spacing::Sm.px()
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn paint_transform_section(
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    hit_index: &mut HitIndex,
    store: &WidgetStore,
    x: f32,
    w: f32,
    y: f32,
) -> f32 {
    let label_font = TypeToken::Sm.px();
    let field_h = ROW_H_PX;
    let row_gap = Spacing::Sm.px();
    let label_color = resolve(ColorToken::Text2, theme);

    paint_text_title(
        text_system,
        scene,
        "Transform",
        x,
        y,
        TypeToken::Md.px(),
        w - 90.0, // LITERAL-PX-OK: width budget for title, reserves 90px for the Reset button
        resolve(ColorToken::Text1, theme),
    );
    let reset_w = 80.0_f32; // LITERAL-PX-OK: Reset button explicit width
    let reset_h = 24.0_f32; // LITERAL-PX-OK: Reset button compact height
    let reset_rect = Rect::new(x + w - reset_w, y - Spacing::Xxs.px(), reset_w, reset_h);
    let reset_state = store
        .button_state(ids::INSP_TRANSFORM_RESET)
        .unwrap_or(ButtonState::Normal);
    hit_index.register(ids::INSP_TRANSFORM_RESET, reset_rect);
    let reset_btn = Button::new(ids::INSP_TRANSFORM_RESET, "Reset")
        .kind(ButtonKind::Default)
        .state(reset_state);
    paint_button(&reset_btn, reset_rect, scene, text_system, theme);
    let mut cur_y = y + TypeToken::Md.px() + 10.0; // LITERAL-PX-OK: title baseline + breathing gap
    cur_y = paint_section_separator(scene, theme, x, w, cur_y);

    let col_gap = Spacing::Md.px();
    let tag_box_gap = Spacing::Xxs.px();
    let label_col_w = 78.0_f32; // LITERAL-PX-OK: row-label column width
    let axis_col_w = Spacing::Lg.px();
    let non_box_w = label_col_w + col_gap * 2.0 + (axis_col_w + tag_box_gap) * 2.0;
    let box_col_w = ((w - non_box_w) * 0.5).max(40.0); // LITERAL-PX-OK: minimum field box width
    let axis_label_font = TypeToken::Base.px();

    let paint_row = |scene: &mut VectorScene,
                     text_system: &mut TextSystem,
                     hit_index: &mut HitIndex,
                     row_y: f32,
                     row_label: &str,
                     left_id: NodeId,
                     left_tag: &str,
                     left_color: ColorToken,
                     left_step: f64,
                     right: Option<(NodeId, &str, ColorToken, f64)>| {
        paint_text(
            text_system,
            scene,
            row_label,
            x,
            row_y + (field_h - label_font) * 0.5,
            label_font,
            label_col_w,
            label_color,
        );
        let left_tag_x = x + label_col_w + col_gap;
        paint_text(
            text_system,
            scene,
            left_tag,
            left_tag_x,
            row_y + (field_h - axis_label_font) * 0.5,
            axis_label_font,
            axis_col_w,
            resolve(left_color, theme),
        );
        let left_box_x = left_tag_x + axis_col_w + tag_box_gap;
        let left_rect = Rect::new(left_box_x, row_y, box_col_w, field_h);
        hit_index.register(left_id, left_rect);
        let (state, value, buffer, caret, anchor) = read_number_input(store, left_id);
        let input = NumberInput::new(left_id, "", value)
            .step(left_step)
            .state(state);
        paint_number_input_with_buffer(
            &input,
            Some(buffer),
            caret,
            anchor,
            left_rect,
            scene,
            text_system,
            theme,
        );
        if let Some((right_id, right_tag, right_color, right_step)) = right {
            let right_tag_x = left_box_x + box_col_w + col_gap;
            paint_text(
                text_system,
                scene,
                right_tag,
                right_tag_x,
                row_y + (field_h - axis_label_font) * 0.5,
                axis_label_font,
                axis_col_w,
                resolve(right_color, theme),
            );
            let right_box_x = right_tag_x + axis_col_w + tag_box_gap;
            let right_rect = Rect::new(right_box_x, row_y, box_col_w, field_h);
            hit_index.register(right_id, right_rect);
            let (r_state, r_value, r_buffer, r_caret, r_anchor) =
                read_number_input(store, right_id);
            let r_input = NumberInput::new(right_id, "", r_value)
                .step(right_step)
                .state(r_state);
            paint_number_input_with_buffer(
                &r_input,
                Some(r_buffer),
                r_caret,
                r_anchor,
                right_rect,
                scene,
                text_system,
                theme,
            );
        }
    };

    let unit = current_display_unit();
    let (pos_label, pos_step) = match unit {
        ph2d_editor_core::project::DisplayUnit::Meters => ("Position (m)", 0.01_f64), // LITERAL-PX-OK: NumberInput step
        ph2d_editor_core::project::DisplayUnit::Pixels => ("Position (px)", 1.0_f64),
    };
    paint_row(
        scene,
        text_system,
        hit_index,
        cur_y,
        pos_label,
        ids::INSP_TRANSFORM_POS_X,
        "X",
        ColorToken::Danger,
        pos_step,
        Some((
            ids::INSP_TRANSFORM_POS_Y,
            "Y",
            ColorToken::Success,
            pos_step,
        )),
    );
    cur_y += field_h + row_gap;
    paint_row(
        scene,
        text_system,
        hit_index,
        cur_y,
        "Rotation (°)",
        ids::INSP_TRANSFORM_ROT,
        "",
        ColorToken::Text3,
        1.0,
        None,
    );
    cur_y += field_h + row_gap;
    paint_row(
        scene,
        text_system,
        hit_index,
        cur_y,
        "Scale",
        ids::INSP_TRANSFORM_SCALE_X,
        "X",
        ColorToken::Danger,
        0.1, // LITERAL-PX-OK: scale NumberInput step
        Some((ids::INSP_TRANSFORM_SCALE_Y, "Y", ColorToken::Success, 0.1)), // LITERAL-PX-OK: scale NumberInput step
    );
    cur_y += field_h + Spacing::Xs.px();

    cur_y
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn paint_render_source_section(
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    hit_index: &mut HitIndex,
    store: &WidgetStore,
    x: f32,
    w: f32,
    y: f32,
    info: &InspectorSpriteInfo,
) -> f32 {
    let line_font = TypeToken::Sm.px();
    let label_font = TypeToken::Xs.px();
    let row_gap = Spacing::Xs.px();
    let row_h = line_font + row_gap;
    paint_text_title(
        text_system,
        scene,
        "Render Source",
        x,
        y,
        TypeToken::Md.px(),
        w,
        resolve(ColorToken::Text1, theme),
    );
    let mut cur_y = y + TypeToken::Md.px() + Spacing::Md.px();
    cur_y = paint_section_separator(scene, theme, x, w, cur_y);

    let paint_pair = |scene: &mut VectorScene,
                      text_system: &mut TextSystem,
                      label: &str,
                      value: &str,
                      mut yy: f32|
     -> f32 {
        paint_text(
            text_system,
            scene,
            label,
            x,
            yy,
            label_font,
            w,
            resolve(ColorToken::Text3, theme),
        );
        yy += label_font + 2.0;
        paint_text(
            text_system,
            scene,
            value,
            x,
            yy,
            line_font,
            w,
            resolve(ColorToken::Text1, theme),
        );
        yy + row_h + row_gap
    };

    paint_text(
        text_system,
        scene,
        "Strategy",
        x,
        cur_y,
        label_font,
        w,
        resolve(ColorToken::Text3, theme),
    );
    cur_y += label_font + Spacing::Xs.px();
    let strategy_btn_h = ROW_H_PX;
    let strategy_gap = Spacing::Sm.px();
    let strategy_btn_w = ((w - strategy_gap * 2.0) / 3.0).max(40.0); // LITERAL-PX-OK: 3-segment strategy button minimum width
    let strategy_buttons = [
        (
            ids::INSP_RENDER_STRATEGY_ATLAS,
            "Atlas",
            matches!(info.source_kind, InspectorSpriteSource::Atlas { .. }),
        ),
        (
            ids::INSP_RENDER_STRATEGY_INDIVIDUAL,
            "Individual",
            matches!(info.source_kind, InspectorSpriteSource::Individual { .. }),
        ),
        (
            ids::INSP_RENDER_STRATEGY_HANDPACKED,
            "Hand-packed",
            matches!(info.source_kind, InspectorSpriteSource::HandPacked),
        ),
    ];
    for (i, (id, label, pressed)) in strategy_buttons.into_iter().enumerate() {
        let bx = x + (strategy_btn_w + strategy_gap) * i as f32;
        let r = Rect::new(bx, cur_y, strategy_btn_w, strategy_btn_h);
        hit_index.register(id, r);
        let state = if pressed {
            ButtonState::Pressed
        } else {
            store.button_state(id).unwrap_or(ButtonState::Normal)
        };
        let btn = Button::new(id, label)
            .kind(ButtonKind::Default)
            .state(state);
        paint_button(&btn, r, scene, text_system, theme);
    }
    cur_y += strategy_btn_h + Spacing::Md.px();
    let storage_detail = match info.source_kind {
        InspectorSpriteSource::Atlas { key } => format!("Atlas key: {}", key),
        InspectorSpriteSource::Individual { texture_id } => {
            format!("Texture id: {}", texture_id)
        }
        InspectorSpriteSource::HandPacked => "Hand-packed (atlas asset)".to_string(),
    };
    cur_y = paint_pair(scene, text_system, "Storage", &storage_detail, cur_y);
    if let Some((pw, ph)) = info.source_pixels {
        let px_str = format!("{} × {} px", pw, ph);
        cur_y = paint_pair(scene, text_system, "Source", &px_str, cur_y);
    }

    paint_text(
        text_system,
        scene,
        "Pixel format",
        x,
        cur_y,
        label_font,
        w,
        resolve(ColorToken::Text3, theme),
    );
    cur_y += label_font + Spacing::Xs.px();
    let btn_h = ROW_H_PX;
    let gap = Spacing::Sm.px();
    let half_w = (w - gap) * 0.5;
    let rgba8_rect = Rect::new(x, cur_y, half_w, btn_h);
    let rgba16_rect = Rect::new(x + half_w + gap, cur_y, half_w, btn_h);
    let rgba8_state = store
        .button_state(ids::INSP_RENDER_FORMAT_RGBA8)
        .unwrap_or(ButtonState::Pressed);
    let rgba16_state = ButtonState::Disabled;
    hit_index.register(ids::INSP_RENDER_FORMAT_RGBA8, rgba8_rect);
    hit_index.register(ids::INSP_RENDER_FORMAT_RGBA16, rgba16_rect);
    let rgba8_btn = Button::new(ids::INSP_RENDER_FORMAT_RGBA8, "RGBA8")
        .kind(ButtonKind::Default)
        .state(rgba8_state);
    let rgba16_btn = Button::new(ids::INSP_RENDER_FORMAT_RGBA16, "RGBA16")
        .kind(ButtonKind::Default)
        .state(rgba16_state);
    paint_button(&rgba8_btn, rgba8_rect, scene, text_system, theme);
    paint_button(&rgba16_btn, rgba16_rect, scene, text_system, theme);
    cur_y += btn_h + Spacing::Md.px();

    let reimport_h = 30.0_f32; // LITERAL-PX-OK: Reimport button height
    let btn_rect = Rect::new(x, cur_y, w, reimport_h);
    let id = ids::INSP_RENDER_SOURCE_REIMPORT;
    let state = if !info.can_reimport {
        ButtonState::Disabled
    } else {
        store.button_state(id).unwrap_or(ButtonState::Normal)
    };
    hit_index.register(id, btn_rect);
    let btn = Button::new(id, "Reimport at current px/m")
        .kind(ButtonKind::Default)
        .state(state);
    paint_button(&btn, btn_rect, scene, text_system, theme);
    cur_y + reimport_h + Spacing::Xs.px()
}
