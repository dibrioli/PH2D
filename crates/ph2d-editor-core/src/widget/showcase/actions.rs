//! Showcase `paint_actions_section` painter.
//!
//! Extracted from monolithic `showcase.rs` in Wave 6+7 Phase 1.C.

use super::*;

#[allow(clippy::too_many_arguments)]
pub(super) fn paint_actions_section(
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
        ids::INSP_SECTION_ACTIONS,
        ids::INSP_SECTION_ACTIONS_COLOR,
        "Actions",
        4,
    );
    if !open {
        return y;
    }
    let btn_h = 30.0_f32; // LITERAL-PX-OK: action button height (distinct from ROW_H_PX)
    let gap = Spacing::Sm.px();
    // Three labelled buttons.
    let trio_w = (w - gap * 2.0) / 3.0; // LITERAL-PX-OK: button count divisor
    let trio = [
        (ids::INSP_SAMPLE_BTN_PRIMARY, "Save", ButtonKind::Accent),
        (
            ids::INSP_SAMPLE_BTN_SECONDARY,
            "Cancel",
            ButtonKind::Default,
        ),
        (ids::INSP_SAMPLE_BTN_DANGER, "Delete", ButtonKind::Danger),
    ];
    for (i, (id, label, kind)) in trio.iter().enumerate() {
        let r = Rect::new(x + (trio_w + gap) * i as f32, y, trio_w, btn_h);
        hit_index.register(*id, r);
        let state = store.button_state(*id).unwrap_or(ButtonState::Normal);
        let btn = Button::new(*id, *label).kind(*kind).state(state);
        paint_button(&btn, r, scene, text_system, theme);
    }
    y += btn_h + ROW_GAP;

    // Icon button + Tag (removable) on one row.
    let icon_size = Spacing::Xl3.px();
    let ir = Rect::new(x, y, icon_size, icon_size);
    hit_index.register(ids::INSP_SAMPLE_BTN_ICON, ir);
    let icon_state = store
        .button_state(ids::INSP_SAMPLE_BTN_ICON)
        .unwrap_or(ButtonState::Normal);
    let icon_btn = Button::new(ids::INSP_SAMPLE_BTN_ICON, "")
        .kind(ButtonKind::IconOnly {
            icon: IconId::Settings,
        })
        .state(icon_state);
    paint_button(&icon_btn, ir, scene, text_system, theme);

    let tag_w = 80.0_f32; // LITERAL-PX-OK: showcase tag chip width
    let tag_h = Density::Compact.row_h_px();
    let tr = Rect::new(
        x + icon_size + Spacing::Md.px(),
        y + (icon_size - tag_h) * 0.5,
        tag_w,
        tag_h,
    );
    let tag_state = match store.get(ids::INSP_SAMPLE_TAG_REMOVE) {
        Some(InteractiveState::Tag { state }) => *state,
        _ => TagState::Normal,
    };
    let tag = Tag::new(ids::INSP_SAMPLE_TAG_REMOVE, "filter")
        .tone(TagTone::Accent)
        .removable(true)
        .state(tag_state);
    paint_tag(&tag, tr, scene, text_system, theme);
    if let Some(close_r) = tag.close_rect(tr) {
        hit_index.register(ids::INSP_SAMPLE_TAG_REMOVE, close_r);
    }
    y + icon_size
}
