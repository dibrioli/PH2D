//! Showcase `paint_color_section` painter.
//!
//! Extracted from monolithic `showcase.rs` in Wave 6+7 Phase 1.C.

use super::*;

#[allow(clippy::too_many_arguments)]
pub(super) fn paint_color_section(
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    hit_index: &mut HitIndex,
    store: &WidgetStore,
    x: f32,
    w: f32,
    y: f32,
) -> f32 {
    let (y, open) = paint_collapsible_header(
        scene,
        text_system,
        theme,
        hit_index,
        store,
        x,
        w,
        y,
        ids::INSP_SECTION_COLOR,
        ids::INSP_SECTION_COLOR_COLOR,
        "Color",
        1,
    );
    if !open {
        return y;
    }
    let sw_h = ROW_H_PX;
    let label_w = 80.0_f32; // LITERAL-PX-OK: showcase label column width
    paint_left_label(scene, text_system, theme, x, "Tint", label_w, y, sw_h);
    let sw_size = Spacing::Xl3.px();
    let sr = Rect::new(x + w - sw_size, y, sw_size, sw_h);
    hit_index.register(ids::INSP_SAMPLE_SWATCH, sr);
    // Read the swatch's live color from `widget_colors` so the
    // picker's edits propagate back through hero's per-frame
    // mirror. Fallback to the initial purple if nothing was seeded.
    let rgba = store
        .widget_color(ids::INSP_SAMPLE_SWATCH)
        .unwrap_or([120, 60, 200, 255]);
    let mut tint = ColorSwatch::new(ids::INSP_SAMPLE_SWATCH, "Tint", rgba);
    tint.size = SwatchSize::Md;
    paint_color_swatch(&tint, sr, scene, theme);
    y + sw_h
}
