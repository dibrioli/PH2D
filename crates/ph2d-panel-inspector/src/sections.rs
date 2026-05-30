//! M14 live Inspector section painters (Name / Visibility / Transform
//! / Render Source). Migrated to the panel crate in ADR-0029 Phase C.1.

use crate::state::current_display_unit;
use ph2d_a11y::NodeId;
use ph2d_editor_core::icons::IconId;
use ph2d_editor_core::ids;
use ph2d_editor_core::interaction::{HitIndex, InteractiveState, WidgetStore};
use ph2d_editor_core::paint::{paint_text, resolve};
use ph2d_editor_core::screens::hero::{InspectorSpriteInfo, InspectorSpriteSource};
use ph2d_editor_core::widget::panel_chrome::{
    SECTION_BOTTOM_PAD_PX, SECTION_LABEL_TO_CONTROL_PX, paint_segmented_group_adaptive,
};
use ph2d_editor_core::widget::showcase::read_number_input;
use ph2d_editor_core::widget::{
    Button, ButtonKind, ButtonState, Checkbox, CheckboxState, CheckboxValue, IconButtonStyle,
    IconGlyph, NumberInput, SectionHeader, TextInput, TextInputState, paint_button, paint_checkbox,
    paint_icon_button, paint_number_input_with_buffer, paint_section_header,
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
    let row_h = 18.0_f32; // LITERAL-PX-OK: matches Checkbox visual height (box 16 + label baseline)
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
    y + row_h + SECTION_BOTTOM_PAD_PX
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

    // Canonical section header (ALL-CAPS + collapse chevron + color-dot
    // slot per UI canon — `docs/UI_Padrao/components/section_header.md`).
    // Reset is an ICON button (user feedback 2026-05-24).
    //
    // Order LEFT → RIGHT: title · reset icon · color dot. The color
    // dot lives at the FAR right (panel border), reset icon sits
    // just to its left. Pre-2026-05-24 the order was reversed; user
    // requested the swap.
    let header_h = TypeToken::Md.px() + Spacing::Md.px(); // LITERAL-PX-OK: section header band height
    let reset_size = header_h; // square icon button matching header height
    let collapsed = store.is_collapsed(ids::INSP_LIVE_TRANSFORM_SECTION);
    let color_id = ids::INSP_LIVE_TRANSFORM_COLOR;
    let rgba = store
        .widget_color(color_id)
        .unwrap_or([0x88, 0x88, 0x88, 0xff]); // LITERAL-COLOR-OK: neutral default for unconfigured section accent
    // Header rect spans the FULL panel width so paint_section_header
    // anchors the color dot at the right edge (panel border).
    let header = SectionHeader::new(ids::INSP_LIVE_TRANSFORM_SECTION, "Transform")
        .collapsible(!collapsed)
        .color(rgba);
    let header_rect = Rect::new(x, y, w, header_h);
    // Reserve for the color dot at the right edge (≈ Md pad + 14 px
    // dot diameter) — the reset icon slots just to the LEFT of it.
    let color_slot_w = Spacing::Md.px() + 14.0; // LITERAL-PX-OK: color dot diameter (2 * radius 7)
    paint_section_header(&header, header_rect, scene, text_system, theme);
    if let Some(circle_rect) = ph2d_editor_core::widget::color_circle_hit_rect(&header, header_rect)
    {
        hit_index.register(color_id, circle_rect);
    }
    let reset_rect = Rect::new(x + w - color_slot_w - reset_size, y, reset_size, reset_size);
    let reset_state = store
        .button_state(ids::INSP_TRANSFORM_RESET)
        .unwrap_or(ButtonState::Normal);
    hit_index.register(ids::INSP_TRANSFORM_RESET, reset_rect);
    paint_icon_button(
        reset_rect,
        IconGlyph::Builtin(IconId::Reset),
        IconButtonStyle::Plain,
        reset_state,
        scene,
        theme,
    );
    // Collapsed → return after painting just the header. Body fields
    // (Position / Rotation / Scale) are skipped so the section
    // visually folds to a single row. Click on the header toggles
    // back via the dispatch (apply_click → toggle_collapsed).
    if collapsed {
        return y + header_h;
    }
    // No inner separator — the orchestrator (paint.rs) draws ONE
    // separator AFTER this section's content. Pre-2026-05-24 this fn
    // also painted a separator between title and params, which broke
    // the "separators go BETWEEN sections" canon (DIRETRIZ §5.2).
    let mut cur_y = y + header_h;

    let col_gap = Spacing::Md.px();
    let tag_box_gap = Spacing::Xxs.px();
    let label_col_w = 78.0_f32; // LITERAL-PX-OK: row-label column width
    let axis_col_w = Spacing::Lg.px();
    let axis_label_font = TypeToken::Base.px();
    // Tighter than SECTION_LABEL_TO_CONTROL_PX (4 px): in narrow
    // label-above mode the label sits directly over the chip BORDER,
    // and the border itself already gives visual breathing room — 4 px
    // on top of that read as a "chasm" (user feedback 2026-05-24:
    // "esse espaço entre as labels e as caixas deve ser menor (metade)").
    // Halved to 2 px for the chip-specific case; buttons (Render
    // Source's Strategy/Pixel format) keep the full 4 px because the
    // ghost-button outline is softer than a chip border.
    let label_above_gap = SECTION_LABEL_TO_CONTROL_PX * 0.5;
    let chip_min_w = ph2d_editor_core::widget::NUMBER_INPUT_MIN_W_PX;

    // Per-SECTION narrow check: if the WIDEST row (2-chip Position) wouldn't
    // fit inline at MIN_W chips, the ENTIRE section uses stacked layout
    // (label-above). Keeps Rotation aligned with Position/Scale — user
    // feedback 2026-05-24: "a caixa única de Rotation deve se alinhar à
    // caixa de X à esquerda e à direita". Per-row narrow would let Rotation
    // go inline while Position stacks → misalignment.
    let widest_chips_n_f = 2.0_f32;
    let widest_inline_needed_w = label_col_w
        + col_gap
        + widest_chips_n_f * (axis_col_w + tag_box_gap + chip_min_w)
        + (widest_chips_n_f - 1.0) * col_gap;
    let section_narrow = w < widest_inline_needed_w;

    // Two-chip equal-split width — single source of truth for ALL rows.
    // Single-chip rows extend their chip to span (X start … Y end) =
    // 2*two_chip_w + col_gap, so the lone Rotation chip lines up with
    // X's left edge AND Y's right edge.
    let chips_avail_w_section = if section_narrow {
        w
    } else {
        w - label_col_w - col_gap
    };
    let two_chip_w =
        ((chips_avail_w_section - 2.0 * (axis_col_w + tag_box_gap) - col_gap) / 2.0).max(0.0); // no MIN_W floor here — never overflow rect

    let paint_row = |scene: &mut VectorScene,
                     text_system: &mut TextSystem,
                     hit_index: &mut HitIndex,
                     row_y: f32,
                     row_label: &str,
                     left_id: NodeId,
                     left_tag: &str,
                     left_color: ColorToken,
                     left_step: f64,
                     right: Option<(NodeId, &str, ColorToken, f64)>|
     -> f32 {
        let chips_origin_x = if section_narrow {
            x
        } else {
            x + label_col_w + col_gap
        };
        let label_h_used = if section_narrow { field_h } else { 0.0_f32 };
        let total_h = if section_narrow {
            field_h + label_above_gap + field_h
        } else {
            field_h
        };
        let chips_y = row_y + label_h_used + if section_narrow { label_above_gap } else { 0.0 };

        // Label — full-width on its own row when narrow; left column when inline.
        paint_text(
            text_system,
            scene,
            row_label,
            x,
            row_y + (field_h - label_font) * 0.5,
            label_font,
            if section_narrow { w } else { label_col_w },
            label_color,
        );

        // Single-chip rows (Rotation) span from X-chip start to Y-chip end —
        // alignment with the 2-chip rows above + below. The span must
        // include: chip_X + col_gap + Y-tag slot + chip_Y =
        // 2*two_chip_w + col_gap + axis_col_w + tag_box_gap.
        // The previous formula missed `axis_col_w + tag_box_gap` and
        // left Rotation ending short of Y-chip's right edge.
        let single_chip = right.is_none();
        let left_box_w = if single_chip {
            (two_chip_w * 2.0 + col_gap + axis_col_w + tag_box_gap).max(0.0)
        } else {
            two_chip_w
        };

        let left_tag_x = chips_origin_x;
        paint_text(
            text_system,
            scene,
            left_tag,
            left_tag_x,
            chips_y + (field_h - axis_label_font) * 0.5,
            axis_label_font,
            axis_col_w,
            resolve(left_color, theme),
        );
        let left_box_x = left_tag_x + axis_col_w + tag_box_gap;
        let left_rect = Rect::new(left_box_x, chips_y, left_box_w, field_h);
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
            let right_tag_x = left_box_x + two_chip_w + col_gap;
            paint_text(
                text_system,
                scene,
                right_tag,
                right_tag_x,
                chips_y + (field_h - axis_label_font) * 0.5,
                axis_label_font,
                axis_col_w,
                resolve(right_color, theme),
            );
            let right_box_x = right_tag_x + axis_col_w + tag_box_gap;
            let right_rect = Rect::new(right_box_x, chips_y, two_chip_w, field_h);
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
        total_h
    };

    let unit = current_display_unit();
    let (pos_label, pos_step) = match unit {
        ph2d_editor_core::project::DisplayUnit::Meters => ("Position (m)", 0.01_f64), // LITERAL-PX-OK: NumberInput step
        ph2d_editor_core::project::DisplayUnit::Pixels => ("Position (px)", 1.0_f64),
    };
    let h_pos = paint_row(
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
    cur_y += h_pos + row_gap;
    let h_rot = paint_row(
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
    cur_y += h_rot + row_gap;
    let h_scale = paint_row(
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
    cur_y += h_scale + row_gap;
    // Skew X/Y in degrees (ADR-0025-amendment-1). Authoring range is
    // clamped to ±~89.4° at the ECS-commit boundary; the slider itself
    // is unbounded so over-typing snaps back on re-sync.
    let h_skew = paint_row(
        scene,
        text_system,
        hit_index,
        cur_y,
        "Skew (\u{00b0})",
        ids::INSP_TRANSFORM_SKEW_X,
        "X",
        ColorToken::Danger,
        1.0, // LITERAL-PX-OK: skew degrees NumberInput step
        Some((ids::INSP_TRANSFORM_SKEW_Y, "Y", ColorToken::Success, 1.0)), // LITERAL-PX-OK: skew degrees NumberInput step
    );
    cur_y += h_skew + SECTION_BOTTOM_PAD_PX;

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
    // Match Transform's row-label style — Sm font, Text2 color — so
    // Render Source feels visually identical (user feedback 2026-05-24).
    let line_font = TypeToken::Sm.px();
    let label_font = TypeToken::Sm.px();
    let row_gap = Spacing::Xs.px();
    let row_h = line_font + row_gap;
    let header_h = TypeToken::Md.px() + Spacing::Md.px(); // LITERAL-PX-OK: section header band height
    let collapsed = store.is_collapsed(ids::INSP_LIVE_RENDER_SECTION);
    let color_id = ids::INSP_LIVE_RENDER_COLOR;
    let rgba = store
        .widget_color(color_id)
        .unwrap_or([0x88, 0x88, 0x88, 0xff]); // LITERAL-COLOR-OK: neutral default for unconfigured section accent
    let header = SectionHeader::new(ids::INSP_LIVE_RENDER_SECTION, "Render Source")
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
    // No inner separator — orchestrator draws it AFTER section content.
    let mut cur_y = y + header_h;

    // Adaptive label/value row: when there's room, paint inline
    // ("Storage   Atlas · key 0"); else fall back to stacked
    // (label above, value below). User feedback 2026-05-24:
    // "coloque numa mesma linha melhor escrito e formatado, e
    // tambem capaz de se adaptar a largura do painel".
    let paint_pair = |scene: &mut VectorScene,
                      text_system: &mut TextSystem,
                      label: &str,
                      value: &str,
                      yy: f32|
     -> f32 {
        let gap = Spacing::Md.px();
        let label_w = text_system.layout(label, label_font, f32::INFINITY).width();
        let value_w_natural = text_system.layout(value, line_font, f32::INFINITY).width();
        if label_w + gap + value_w_natural <= w {
            // Inline: label LEFT (Text2), value flush after the gap (Text1).
            paint_text(
                text_system,
                scene,
                label,
                x,
                yy,
                label_font,
                label_w,
                resolve(ColorToken::Text2, theme),
            );
            paint_text(
                text_system,
                scene,
                value,
                x + label_w + gap,
                yy,
                line_font,
                w - label_w - gap,
                resolve(ColorToken::Text1, theme),
            );
            yy + line_font + row_gap
        } else {
            // Stacked: label above, value below.
            paint_text(
                text_system,
                scene,
                label,
                x,
                yy,
                label_font,
                w,
                resolve(ColorToken::Text2, theme),
            );
            let yy2 = yy + label_font + 2.0;
            paint_text(
                text_system,
                scene,
                value,
                x,
                yy2,
                line_font,
                w,
                resolve(ColorToken::Text1, theme),
            );
            yy2 + row_h + row_gap
        }
    };

    paint_text(
        text_system,
        scene,
        "Strategy",
        x,
        cur_y,
        label_font,
        w,
        resolve(ColorToken::Text2, theme),
    );
    cur_y += label_font + SECTION_LABEL_TO_CONTROL_PX;
    let strategy_btn_h = ROW_H_PX;
    // Adaptive segmented GROUP — when the panel is narrow, drops
    // "Hand-packed" (the longest) to its own row instead of wrapping
    // the label. Returns the actual height used.
    let strategy_h = paint_segmented_group_adaptive(
        Rect::new(x, cur_y, w, strategy_btn_h),
        &[
            (
                "Atlas",
                matches!(info.source_kind, InspectorSpriteSource::Atlas { .. }),
                ids::INSP_RENDER_STRATEGY_ATLAS,
            ),
            (
                "Individual",
                matches!(info.source_kind, InspectorSpriteSource::Individual { .. }),
                ids::INSP_RENDER_STRATEGY_INDIVIDUAL,
            ),
            (
                "Hand-packed",
                matches!(info.source_kind, InspectorSpriteSource::HandPacked),
                ids::INSP_RENDER_STRATEGY_HANDPACKED,
            ),
        ],
        scene,
        text_system,
        theme,
        hit_index,
    );
    // Inter-row gap inside Render Source — matches Transform's row_gap
    // (SECTION_INNER_ROW_GAP_PX) so Render Source feels like Transform.
    cur_y += strategy_h + ph2d_editor_core::widget::panel_chrome::SECTION_INNER_ROW_GAP_PX;
    // Cleaner phrasing — strategy name + key/id separated by middle
    // dot (the only ASCII-safe non-ASCII glyph allowed in UI strings;
    // vide no_tofu_glyphs gate). Pre-canon was "Atlas key: 0" /
    // "Texture id: 5" with the strategy redundantly named.
    let storage_detail = match info.source_kind {
        InspectorSpriteSource::Atlas { key } => format!("Atlas \u{00b7} key {}", key),
        InspectorSpriteSource::Individual { texture_id } => {
            format!("Individual \u{00b7} texture {}", texture_id)
        }
        InspectorSpriteSource::HandPacked => "Hand-packed".to_string(),
    };
    cur_y = paint_pair(scene, text_system, "Storage", &storage_detail, cur_y);
    if let Some((pw, ph)) = info.source_pixels {
        let px_str = format!("{} \u{00d7} {} px", pw, ph);
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
        resolve(ColorToken::Text2, theme),
    );
    cur_y += label_font + SECTION_LABEL_TO_CONTROL_PX;
    let btn_h = ROW_H_PX;
    // Adaptive segmented GROUP — same canon as Strategy above.
    let fmt_h = paint_segmented_group_adaptive(
        Rect::new(x, cur_y, w, btn_h),
        &[
            ("RGBA8", true, ids::INSP_RENDER_FORMAT_RGBA8),
            ("RGBA16", false, ids::INSP_RENDER_FORMAT_RGBA16),
        ],
        scene,
        text_system,
        theme,
        hit_index,
    );
    cur_y += fmt_h + ph2d_editor_core::widget::panel_chrome::SECTION_INNER_ROW_GAP_PX;

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
    cur_y + reimport_h + SECTION_BOTTOM_PAD_PX
}
