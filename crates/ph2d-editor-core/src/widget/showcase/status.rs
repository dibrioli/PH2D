//! Showcase `paint_status_section` painter.
//!
//! Extracted from monolithic `showcase.rs` in Wave 6+7 Phase 1.C.

use super::*;

#[allow(clippy::too_many_arguments)]
pub(super) fn paint_status_section(
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
        ids::INSP_SECTION_STATUS,
        ids::INSP_SECTION_STATUS_COLOR,
        "Status",
        3,
    );
    if !open {
        return y;
    }

    // ProgressBar — determinate, 60 %.
    let bar = ProgressBar::new(NodeId(0), "Build")
        .determinate(0.6) // LITERAL-PX-OK: progress ratio (demo value)
        .show_percent(true);
    let bar_rect = Rect::new(x, y, w, Spacing::Lg.px());
    paint_progress_bar(&bar, bar_rect, scene, text_system, theme);
    y += Spacing::Lg.px() + ROW_GAP;

    // LevelMeter — vertical stereo peak meter (Success/Warn/Danger zones).
    let meter = LevelMeter::new(NodeId(0), "Level").levels(0.55, 0.85); // LITERAL-PX-OK: demo levels
    let meter_w = 16.0_f32; // LITERAL-PX-OK: showcase meter width (chrome demo)
    let meter_h = 60.0_f32; // LITERAL-PX-OK: showcase meter height (chrome demo)
    paint_level_meter(&meter, Rect::new(x, y, meter_w, meter_h), scene, theme);
    y += meter_h + ROW_GAP;

    // Spinner + caption.
    let spin_rect = Rect::new(x, y, Radius::Xl2.px(), Radius::Xl2.px());
    paint_spinner(&Spinner::new(NodeId(0), "Loading"), spin_rect, scene, theme);
    paint_left_label(
        scene,
        text_system,
        theme,
        x + 28.0, // LITERAL-PX-OK: spinner+caption offset (Spinner 20 + gap ~8 chrome composite)
        "Loading…",
        w - 28.0, // LITERAL-PX-OK: caption width budget
        y,
        Radius::Xl2.px(),
    );
    y += Radius::Xl2.px() + ROW_GAP;

    // Tag chips — Accent + Success + Warn.
    let chip_w = 50.0_f32; // LITERAL-PX-OK: tag chip width (chrome-specific)
    let chip_h = TypeToken::Lg.px();
    let gap = Spacing::Sm.px();
    for (i, (label, tone)) in [
        ("PRF", TagTone::Accent),
        ("OK", TagTone::Success),
        ("HOT", TagTone::Warn),
    ]
    .iter()
    .enumerate()
    {
        let cr = Rect::new(x + (chip_w + gap) * i as f32, y, chip_w, chip_h);
        let tag = Tag::new(NodeId(0), *label).tone(*tone);
        paint_tag(&tag, cr, scene, text_system, theme);
    }
    y + chip_h
}
