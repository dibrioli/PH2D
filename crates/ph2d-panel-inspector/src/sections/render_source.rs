//! Render Source (+ Region) — Inspector section painter (split from sections.rs,
//! architecture_panel_loc_cap). Logic verbatim; behavior unchanged.

use super::*;

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
        // W2.T2: tier-cooked KTX2 — read-only marker, no key/id shown.
        InspectorSpriteSource::CookedTexture => "Cooked texture".to_string(),
    };
    cur_y = paint_pair(scene, text_system, "Storage", &storage_detail, cur_y);
    if let Some((pw, ph)) = info.source_pixels {
        let px_str = format!("{} \u{00d7} {} px", pw, ph);
        cur_y = paint_pair(scene, text_system, "Source", &px_str, cur_y);
    }

    // Region sampling (spec §3.3) — hidden for Hand-packed (it brings its
    // own rect from the asset). Toggle + (when on) X/Y/W/H px inputs +
    // Filter Clip. Renders via the extract `region_subrect` (W2.T2.4).
    if !matches!(info.source_kind, InspectorSpriteSource::HandPacked) {
        let cb_h = 18.0_f32; // LITERAL-PX-OK: Checkbox visual height
        let (re_state, re_value) = store
            .checkbox(ids::INSP_REGION_ENABLED)
            .unwrap_or((CheckboxState::Normal, CheckboxValue::Unchecked));
        let re_rect = Rect::new(x, cur_y, w, cb_h);
        hit_index.register(ids::INSP_REGION_ENABLED, re_rect);
        paint_checkbox(
            &Checkbox::new(ids::INSP_REGION_ENABLED, "Region")
                .state(re_state)
                .value(re_value),
            re_rect,
            scene,
            text_system,
            theme,
        );
        cur_y += cb_h + row_gap;

        if matches!(re_value, CheckboxValue::Checked) {
            let field_h = ROW_H_PX;
            let cell_gap = Spacing::Md.px();
            let cell_w = ((w - cell_gap) * 0.5).max(0.0);
            let axis_w = Spacing::Lg.px(); // mini X/Y/W/H label column
            let num_cell = |scene: &mut VectorScene,
                            text_system: &mut TextSystem,
                            hit_index: &mut HitIndex,
                            cell: Rect,
                            axis: &str,
                            id: NodeId| {
                paint_text(
                    text_system,
                    scene,
                    axis,
                    cell.x,
                    cell.y + (cell.h - label_font) * 0.5,
                    label_font,
                    axis_w,
                    resolve(ColorToken::Text2, theme),
                );
                let input_rect =
                    Rect::new(cell.x + axis_w, cell.y, (cell.w - axis_w).max(0.0), cell.h);
                hit_index.register(id, input_rect);
                let (state, value, buffer, caret, anchor) = read_number_input(store, id);
                let input = NumberInput::new(id, "", value).step(1.0).state(state);
                paint_number_input_with_buffer(
                    &input,
                    Some(buffer),
                    caret,
                    anchor,
                    input_rect,
                    scene,
                    text_system,
                    theme,
                );
            };
            num_cell(
                scene,
                text_system,
                hit_index,
                Rect::new(x, cur_y, cell_w, field_h),
                "X",
                ids::INSP_REGION_X,
            );
            num_cell(
                scene,
                text_system,
                hit_index,
                Rect::new(x + cell_w + cell_gap, cur_y, cell_w, field_h),
                "Y",
                ids::INSP_REGION_Y,
            );
            cur_y += field_h + row_gap;
            num_cell(
                scene,
                text_system,
                hit_index,
                Rect::new(x, cur_y, cell_w, field_h),
                "W",
                ids::INSP_REGION_W,
            );
            num_cell(
                scene,
                text_system,
                hit_index,
                Rect::new(x + cell_w + cell_gap, cur_y, cell_w, field_h),
                "H",
                ids::INSP_REGION_H,
            );
            cur_y += field_h + row_gap;

            let (fc_state, fc_value) = store
                .checkbox(ids::INSP_REGION_FILTER_CLIP)
                .unwrap_or((CheckboxState::Normal, CheckboxValue::Checked));
            let fc_rect = Rect::new(x, cur_y, w, cb_h);
            hit_index.register(ids::INSP_REGION_FILTER_CLIP, fc_rect);
            paint_checkbox(
                &Checkbox::new(ids::INSP_REGION_FILTER_CLIP, "Filter Clip")
                    .state(fc_state)
                    .value(fc_value),
                fc_rect,
                scene,
                text_system,
                theme,
            );
            cur_y += cb_h + row_gap;
        }
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
