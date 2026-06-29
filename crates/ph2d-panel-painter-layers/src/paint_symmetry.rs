//! Brush panel — the collapsible **Symmetry** section (drawing mirror / radial), painted just above
//! Tiling and collapsed by default. Built on the same canonical widgets as the rest of the app
//! (Inspector / Widget Gallery): the shared collapsible header + checkbox rows, a **segmented**
//! [`RadioGroup`] for the X / Y / Custom axis (the Inspector "Sort Point" pattern), a limited
//! slider-with-chip for the radial segment count, and labelled mode-entry buttons (the colour-picker
//! analogue) to draw the custom line / pick the radial centre. Split from [`crate::paint_brush`] for
//! the LOC cap; every control is `populate`-registered + dispatched in [`crate::event`].

use ph2d_editor_core::ids as core_ids;
use ph2d_editor_core::panel::PaintCtx;
use ph2d_editor_core::widget::{
    Button, ButtonKind, ButtonState, DEFAULT_CHIP_W, DEFAULT_LABEL_W, RadioGroup, RadioOption,
    RadioOrientation, paint_button, paint_radio_group_with_labels,
    paint_slider_with_chip_layout_adaptive,
};
use ph2d_editor_core::zones::Rect;
use ph2d_tokens::{ROW_H_PX, Spacing};
use ph2d_tool_painter::BrushSettings;

use crate::paint_brush_top::{paint_checkbox_row, paint_collapsible_section};

/// Radial segment-count UI range (mirrors `ph2d_painter_brush::SYMMETRY_{MIN,MAX}_SEGMENTS`).
const SEG_MIN: f32 = 3.0;
const SEG_MAX: f32 = 12.0;

/// Paint the collapsible **Symmetry** section. Collapsed → just the header. Expanded → the "Use
/// Symmetry" master checkbox, then (only while enabled) the "Circular" checkbox and the mode-specific
/// controls: radial shows a 3–12 segment slider + a "Pick Center" button; mirror shows the X/Y/Custom
/// segmented control + (for Custom) a "Draw Custom Line" button.
pub(crate) fn paint_symmetry_section(
    ctx: &mut PaintCtx,
    theme: ph2d_tokens::Theme,
    x: f32,
    content_w: f32,
    y: f32,
    brush: BrushSettings,
) -> f32 {
    let (mut y, collapsed) = paint_collapsible_section(
        ctx,
        theme,
        x,
        content_w,
        y,
        "Symmetry",
        core_ids::PAINTER_BRUSH_SYMMETRY_SECTION,
        core_ids::PAINTER_BRUSH_SYMMETRY_SECTION_COLOR,
        core_ids::PAINTER_BRUSH_SYMMETRY_RESET,
    );
    if collapsed {
        return y;
    }
    y = paint_checkbox_row(
        ctx,
        theme,
        x,
        content_w,
        y,
        core_ids::PAINTER_BRUSH_SYMMETRY_USE,
        "Use Symmetry",
        brush.symmetry_enabled,
    );
    // The rest of the controls only make sense once symmetry is on.
    if !brush.symmetry_enabled {
        return y;
    }
    y = paint_checkbox_row(
        ctx,
        theme,
        x,
        content_w,
        y,
        core_ids::PAINTER_BRUSH_SYMMETRY_CIRCULAR,
        "Circular",
        brush.symmetry_circular,
    );
    if brush.symmetry_circular {
        y = paint_segments_row(ctx, theme, x, content_w, y, brush.symmetry_segments);
        // "Pick Center" — the canvas click sets the rosette centre (default = canvas centre).
        y = paint_pick_button(
            ctx,
            theme,
            x,
            content_w,
            y,
            core_ids::PAINTER_BRUSH_SYMMETRY_PICK_CENTER,
            "Pick Center",
            brush.symmetry_pick_center,
        );
    } else {
        y = paint_axis_segment(ctx, theme, x, content_w, y, brush.symmetry_axis);
        // Custom axis → the "Draw Custom Line" button arms the on-canvas line draw.
        if brush.symmetry_axis == 2 {
            y = paint_pick_button(
                ctx,
                theme,
                x,
                content_w,
                y,
                core_ids::PAINTER_BRUSH_SYMMETRY_DRAW_LINE,
                "Draw Custom Line",
                brush.symmetry_pick_line,
            );
        }
    }
    y
}

/// The X / Y / Custom mirror-axis **segmented** control (the canonical `RadioGroup::Segmented`, as in
/// the Inspector's "Sort Point"). Each option is a `Button`-backed id forwarded on Click.
fn paint_axis_segment(
    ctx: &mut PaintCtx,
    theme: ph2d_tokens::Theme,
    x: f32,
    content_w: f32,
    y: f32,
    axis: u8,
) -> f32 {
    let selected = match axis {
        1 => "Y",
        2 => "Custom",
        _ => "X",
    };
    let rg = RadioGroup::new(
        ph2d_a11y::NodeId(0),
        "Axis",
        vec![
            RadioOption::new(core_ids::PAINTER_BRUSH_SYMMETRY_AXIS_X, "X", "X"),
            RadioOption::new(core_ids::PAINTER_BRUSH_SYMMETRY_AXIS_Y, "Y", "Y"),
            RadioOption::new(
                core_ids::PAINTER_BRUSH_SYMMETRY_AXIS_CUSTOM,
                "Custom",
                "Custom",
            ),
        ],
    )
    .orientation(RadioOrientation::Segmented)
    .selected(selected);
    let rect = Rect::new(x, y, content_w, ROW_H_PX);
    {
        let scene = &mut *ctx.scene;
        let text_system = &mut *ctx.text_system;
        paint_radio_group_with_labels(&rg, rect, scene, text_system, theme);
    }
    for (i, id) in core_ids::PAINTER_BRUSH_SYMMETRY_AXES.iter().enumerate() {
        ctx.host
            .hit_index_mut()
            .register(*id, rg.option_rect(rect, i));
    }
    y + ROW_H_PX + Spacing::Sm.px()
}

/// The radial **segment-count** slider (`3..=12`, integer). Canonical slider-with-chip; the `0..1`
/// track positions the thumb while the chip shows the whole-number count (the chip is
/// `link_slider_number_mapped_integer`-linked in `populate`, so an edit maps back to the track).
fn paint_segments_row(
    ctx: &mut PaintCtx,
    theme: ph2d_tokens::Theme,
    x: f32,
    content_w: f32,
    y: f32,
    segments: u32,
) -> f32 {
    let seg = (segments as f32).clamp(SEG_MIN, SEG_MAX);
    let track = (seg - SEG_MIN) / (SEG_MAX - SEG_MIN);
    let display = format!("{}", seg as u32);
    let scene = &mut *ctx.scene;
    let text_system = &mut *ctx.text_system;
    let (store, hit_index) = ctx.host.store_and_hit_index_mut();
    let used = paint_slider_with_chip_layout_adaptive(
        Rect::new(x, y, content_w, ROW_H_PX),
        "Segments",
        track,
        f64::from(seg),
        Some(&display),
        core_ids::PAINTER_BRUSH_SYMMETRY_SEGMENTS,
        core_ids::PAINTER_BRUSH_SYMMETRY_SEGMENTS_CHIP,
        DEFAULT_LABEL_W,
        DEFAULT_CHIP_W,
        store,
        hit_index,
        scene,
        text_system,
        theme,
    );
    y + used + Spacing::Xs.px()
}

/// A full-width labelled **mode-entry** button (the colour-picker-button analogue): pressing it arms a
/// canvas pick. Highlighted (`Accent`) while its mode is armed, so the artist sees it's waiting for a
/// canvas gesture. The click forwards as a plain `Click`; the tool arms / toggles the pick mode.
#[allow(clippy::too_many_arguments)]
fn paint_pick_button(
    ctx: &mut PaintCtx,
    theme: ph2d_tokens::Theme,
    x: f32,
    content_w: f32,
    y: f32,
    id: ph2d_a11y::NodeId,
    label: &str,
    armed: bool,
) -> f32 {
    let rect = Rect::new(x, y, content_w, ROW_H_PX);
    let state = ctx
        .host
        .store()
        .button_state(id)
        .unwrap_or(ButtonState::Normal);
    let kind = if armed {
        ButtonKind::Accent
    } else {
        ButtonKind::Default
    };
    let btn = Button::new(id, label).kind(kind).state(state);
    {
        let scene = &mut *ctx.scene;
        let text_system = &mut *ctx.text_system;
        paint_button(&btn, rect, scene, text_system, theme);
    }
    ctx.host.hit_index_mut().register(id, rect);
    y + ROW_H_PX + Spacing::Sm.px()
}
