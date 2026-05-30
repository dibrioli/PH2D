//! Sampling — Inspector §9 section painter (Sprite Inspector v2 W3,
//! spec §3.9 / §9.9). Texture Filter + Texture Repeat as snapshot-driven
//! segmented controls (the renderer applies Inherit/Nearest/Linear for
//! atlas sprites — mipmap/aniso map to their base filter, so the
//! segmented mirrors what actually renders). Anti-halo is an atlas-level
//! read-only label (spec §9.3).

use super::*;
use ph2d_editor_core::screens::hero::InspectorSamplingInfo;

/// Label-above row with two NumberInputs (X / Y) for a UV scale/offset
/// pair. Returns the next `y`.
#[allow(clippy::too_many_arguments)]
fn uv_pair_row(
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    hit_index: &mut HitIndex,
    store: &WidgetStore,
    x: f32,
    w: f32,
    y: f32,
    label: &str,
    id_x: NodeId,
    id_y: NodeId,
) -> f32 {
    let h = ROW_H_PX;
    let label_font = TypeToken::Sm.px();
    let label_h = label_font + Spacing::Xs.px();
    paint_text(
        text_system,
        scene,
        label,
        x,
        y + (label_h - label_font) * 0.5,
        label_font,
        w,
        resolve(ColorToken::Text2, theme),
    );
    let row_y = y + label_h;
    let gap = Spacing::Sm.px();
    let cw = ((w - gap) * 0.5).max(0.0);
    for (i, id) in [id_x, id_y].into_iter().enumerate() {
        let rect = Rect::new(x + (cw + gap) * i as f32, row_y, cw, h);
        hit_index.register(id, rect);
        let (state, value, buffer, caret, anchor) = read_number_input(store, id);
        let input = NumberInput::new(id, "", value).step(0.1).state(state); // LITERAL-PX-OK: UV step
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
    }
    row_y + h + Spacing::Sm.px()
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn paint_sampling_section(
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    hit_index: &mut HitIndex,
    store: &WidgetStore,
    x: f32,
    w: f32,
    y: f32,
    info: &InspectorSamplingInfo,
) -> f32 {
    let header_h = TypeToken::Md.px() + Spacing::Md.px(); // LITERAL-PX-OK: section header band height
    let collapsed = store.is_collapsed(ids::INSP_LIVE_SAMPLING_SECTION);
    let color_id = ids::INSP_LIVE_SAMPLING_COLOR;
    let rgba = store
        .widget_color(color_id)
        .unwrap_or([0x88, 0x88, 0x88, 0xff]); // LITERAL-COLOR-OK: neutral default section accent
    let header = SectionHeader::new(ids::INSP_LIVE_SAMPLING_SECTION, "Sampling")
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
    let h = ROW_H_PX;
    let row_gap = Spacing::Sm.px();
    let label_font = TypeToken::Sm.px();
    let label_color = resolve(ColorToken::Text2, theme);
    // The label sits on its OWN short row ABOVE the segmented (label-
    // above layout) — using a fraction of the control row overlapped
    // the text onto the buttons (smoke 2026-05-30).
    let label_h = label_font + Spacing::Xs.px();

    // Texture Filter — Inherit / Nearest / Linear (tags 0/1/2).
    paint_text(
        text_system,
        scene,
        "Texture Filter",
        x,
        yy + (label_h - label_font) * 0.5,
        label_font,
        w,
        label_color,
    );
    yy += label_h;
    let filter_rect = Rect::new(x, yy, w, h);
    let filter_tabs = Tabs::new(
        NodeId(0),
        "",
        vec![
            TabItem::new(ids::INSP_SAMPLE_FILTER[0], "Inherit"),
            TabItem::new(ids::INSP_SAMPLE_FILTER[1], "Nearest"),
            TabItem::new(ids::INSP_SAMPLE_FILTER[2], "Linear"),
        ],
    )
    .variant(TabsVariant::Segmented)
    .selected((info.filter_tag as usize).min(2));
    paint_tabs(&filter_tabs, filter_rect, scene, text_system, theme);
    for (i, item) in filter_tabs.items.iter().enumerate() {
        hit_index.register(item.id, filter_tabs.tab_rect(filter_rect, i));
    }
    yy += h + row_gap;

    // Texture Repeat — Inherit / Disabled / Enabled / Mirror (0/1/2/3).
    paint_text(
        text_system,
        scene,
        "Texture Repeat",
        x,
        yy + (label_h - label_font) * 0.5,
        label_font,
        w,
        label_color,
    );
    yy += label_h;
    let repeat_rect = Rect::new(x, yy, w, h);
    let repeat_tabs = Tabs::new(
        NodeId(0),
        "",
        vec![
            TabItem::new(ids::INSP_SAMPLE_REPEAT[0], "Inherit"),
            TabItem::new(ids::INSP_SAMPLE_REPEAT[1], "Clamp"),
            TabItem::new(ids::INSP_SAMPLE_REPEAT[2], "Repeat"),
            TabItem::new(ids::INSP_SAMPLE_REPEAT[3], "Mirror"),
        ],
    )
    .variant(TabsVariant::Segmented)
    .selected((info.repeat_tag as usize).min(3));
    paint_tabs(&repeat_tabs, repeat_rect, scene, text_system, theme);
    for (i, item) in repeat_tabs.items.iter().enumerate() {
        hit_index.register(item.id, repeat_tabs.tab_rect(repeat_rect, i));
    }
    yy += h + row_gap;

    // UV tiling (scale > 1 tiles) + scroll (offset) — W3 UvTransform. The
    // repeat segmented above picks how the tiled UV wraps.
    yy = uv_pair_row(
        scene,
        text_system,
        theme,
        hit_index,
        store,
        x,
        w,
        yy,
        "UV Scale",
        ids::INSP_SAMPLE_UV_SCALE_X,
        ids::INSP_SAMPLE_UV_SCALE_Y,
    );
    yy = uv_pair_row(
        scene,
        text_system,
        theme,
        hit_index,
        store,
        x,
        w,
        yy,
        "UV Offset",
        ids::INSP_SAMPLE_UV_OFFSET_X,
        ids::INSP_SAMPLE_UV_OFFSET_Y,
    );

    // Anti-halo / edge filtering — atlas-level, read-only (spec §9.3).
    paint_text(
        text_system,
        scene,
        "Anti-halo: enabled (atlas-level)",
        x,
        yy + (h - label_font) * 0.5,
        label_font,
        w,
        resolve(ColorToken::Text3, theme),
    );
    yy += h + SECTION_BOTTOM_PAD_PX;
    yy
}
