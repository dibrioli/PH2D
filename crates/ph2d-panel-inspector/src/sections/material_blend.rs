//! Material & Blend — Inspector §10 section painter (Sprite Inspector
//! v2, spec §3.10). The Blend Mode is a 6-way segmented control over the
//! optional `BlendMode` component (Mix is the default = component
//! absent). Material / InstanceShaderParams are a future shader runtime
//! (the renderer is fixed-function today), so the Material slot is a
//! read-only placeholder. Snapshot-driven like §9 Sampling.

use super::*;
use ph2d_editor_core::screens::hero::InspectorBlendInfo;
use ph2d_editor_core::widget::{SegmentedAdaptive, SegmentedOption, paint_segmented_adaptive};

/// Blend-mode segmented labels, indexed by `BlendMode::tag()` (0..5).
/// Hardcoded here (not from `ph2d_ecs::BlendMode`) so the panel crate
/// stays loose-coupled from `ph2d-ecs` — the snapshot carries only the
/// resolved tag. English per HR-15 (i18n migrates in W7).
const BLEND_LABELS: [&str; 6] = ["Mix", "Add", "Subtract", "Multiply", "Screen", "Premult"];

#[allow(clippy::too_many_arguments)]
pub(crate) fn paint_material_blend_section(
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    hit_index: &mut HitIndex,
    store: &WidgetStore,
    x: f32,
    w: f32,
    y: f32,
    info: &InspectorBlendInfo,
) -> f32 {
    let header_h = TypeToken::Md.px() + Spacing::Md.px(); // LITERAL-PX-OK: section header band height
    let collapsed = store.is_collapsed(ids::INSP_LIVE_BLEND_SECTION);
    let color_id = ids::INSP_LIVE_BLEND_COLOR;
    let rgba = store
        .widget_color(color_id)
        .unwrap_or([0x88, 0x88, 0x88, 0xff]); // LITERAL-COLOR-OK: neutral default section accent
    let header = SectionHeader::new(ids::INSP_LIVE_BLEND_SECTION, "Material & Blend")
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
    let label_h = label_font + Spacing::Xs.px();

    // Blend Mode — 6-way segmented (Mix/Add/Subtract/Multiply/Screen/
    // Premult); `SegmentedAdaptive` reflows the wider labels onto extra
    // rows at the Inspector's narrow width. Selecting Mix detaches the
    // optional component (default).
    paint_text(
        text_system,
        scene,
        "Blend Mode",
        x,
        yy + (label_h - label_font) * 0.5,
        label_font,
        w,
        label_color,
    );
    yy += label_h;
    let seg = SegmentedAdaptive::new(
        ids::INSP_LIVE_BLEND_SECTION,
        "Blend Mode",
        ids::INSP_SAMPLE_BLEND
            .iter()
            .zip(BLEND_LABELS)
            .map(|(&id, label)| SegmentedOption::new(id, label))
            .collect(),
    )
    .selected((info.blend_tag as usize).min(BLEND_LABELS.len() - 1));
    let seg_h = paint_segmented_adaptive(
        &seg,
        Rect::new(x, yy, w, h),
        scene,
        text_system,
        theme,
        store,
        hit_index,
    );
    yy += seg_h + row_gap;

    // Material slot — read-only placeholder. The renderer is
    // fixed-function (no material / shader runtime yet, spec §3.10);
    // Material + InstanceShaderParams land with that future runtime.
    paint_text(
        text_system,
        scene,
        "Material: Default \u{00b7} shader runtime pending",
        x,
        yy + (h - label_font) * 0.5,
        label_font,
        w,
        resolve(ColorToken::Text3, theme),
    );
    yy += h + SECTION_BOTTOM_PAD_PX;
    yy
}
