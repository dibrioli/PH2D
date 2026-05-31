//! Showcase `paint_slider_section` painter.
//!
//! Extracted from monolithic `showcase.rs` in Wave 6+7 Phase 1.C.

use super::*;

#[allow(clippy::too_many_arguments)]
pub(super) fn paint_slider_section(
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
        ids::INSP_SECTION_SLIDER,
        ids::INSP_SECTION_SLIDER_COLOR,
        "Slider",
        1,
    );
    if !open {
        return y;
    }
    let (_, value) = store
        .slider(ids::INSP_SAMPLE_SLIDER)
        .unwrap_or((SliderState::Normal, 0.62)); // LITERAL-PX-OK: slider default ratio (showcase demo seed value)
    let r = Rect::new(x, y, w, FIELD_H);
    let slider_h = paint_slider_with_chip(
        r,
        "Speed",
        value,
        ids::INSP_SAMPLE_SLIDER,
        ids::INSP_SAMPLE_SLIDER_CHIP,
        store,
        hit_index,
        scene,
        text_system,
        theme,
    );
    y += slider_h;
    y
}
